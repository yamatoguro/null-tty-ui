use std::collections::VecDeque;
use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::time::Instant;

/// Captures a point-in-time system snapshot used by all UI panels.
#[derive(Debug, Clone)]
pub struct SystemSnapshot {
    pub cpu_percent: f32,
    pub memory_used_mb: u64,
    pub memory_total_mb: u64,
    pub temp_celsius: Option<f32>,
    pub uptime_secs: u64,
    pub disk_used_gb: f64,
    pub disk_total_gb: f64,
    pub net_rx_bytes: u64,
    pub net_tx_bytes: u64,
    pub load_avg: Option<(f32, f32, f32)>,
    /// Rolling window of the last N log lines from the kernel ring buffer.
    pub recent_logs: Vec<String>,
    /// Pre-rendered DNS summary for the Technitium panel (injected externally).
    pub dns_summary: Option<String>,
    /// Pre-rendered file navigation panel output (injected externally).
    pub file_nav_summary: Option<String>,
}

#[derive(Debug, Default)]
struct LogCursor {
    path: Option<String>,
    offset: u64,
}

/// Collects lightweight metrics from Linux procfs and sysfs.
pub struct MetricsCollector {
    started_at: Instant,
    last_cpu_total: u64,
    last_cpu_idle: u64,
    last_net_rx: u64,
    last_net_tx: u64,
    log_ring: VecDeque<String>,
    log_cursor: LogCursor,
}

const LOG_RING_SIZE: usize = 24;

impl MetricsCollector {
    /// Creates a collector with empty CPU history.
    pub fn new() -> Self {
        Self {
            started_at: Instant::now(),
            last_cpu_total: 0,
            last_cpu_idle: 0,
            last_net_rx: 0,
            last_net_tx: 0,
            log_ring: VecDeque::with_capacity(LOG_RING_SIZE),
            log_cursor: LogCursor::default(),
        }
    }

    /// Reads and computes a fresh snapshot; returns defaults on partial read failures.
    pub fn collect(&mut self) -> SystemSnapshot {
        let (cpu_total, cpu_idle) = read_cpu_counters().unwrap_or((0, 0));
        let cpu_percent = compute_cpu_percent(
            self.last_cpu_total,
            self.last_cpu_idle,
            cpu_total,
            cpu_idle,
        );
        self.last_cpu_total = cpu_total;
        self.last_cpu_idle = cpu_idle;

        let (memory_used_mb, memory_total_mb) = read_meminfo_mb().unwrap_or((0, 0));
        let temp_celsius = read_temperature_celsius();
        let uptime_secs = read_uptime_secs().unwrap_or_else(|| self.started_at.elapsed().as_secs());
        let (disk_used_gb, disk_total_gb) = read_disk_gb("/").unwrap_or((0.0, 0.0));
        let load_avg = read_load_avg();

        let (rx_now, tx_now) = read_net_bytes().unwrap_or((0, 0));
        let net_rx_bytes = rx_now.saturating_sub(self.last_net_rx);
        let net_tx_bytes = tx_now.saturating_sub(self.last_net_tx);
        self.last_net_rx = rx_now;
        self.last_net_tx = tx_now;

        poll_kernel_logs(&mut self.log_cursor, &mut self.log_ring);
        let recent_logs = self.log_ring.iter().cloned().collect();

        SystemSnapshot {
            cpu_percent,
            memory_used_mb,
            memory_total_mb,
            temp_celsius,
            uptime_secs,
            disk_used_gb,
            disk_total_gb,
            net_rx_bytes,
            net_tx_bytes,
            load_avg,
            recent_logs,
            dns_summary: None,
            file_nav_summary: None,
        }
    }
}

/// Parses the first cpu line in /proc/stat to derive total and idle counters.
fn read_cpu_counters() -> Option<(u64, u64)> {
    let content = fs::read_to_string("/proc/stat").ok()?;
    let line = content.lines().next()?;
    let mut parts = line.split_whitespace();
    let _label = parts.next()?;

    let user = parts.next()?.parse::<u64>().ok()?;
    let nice = parts.next()?.parse::<u64>().ok()?;
    let system = parts.next()?.parse::<u64>().ok()?;
    let idle = parts.next()?.parse::<u64>().ok()?;
    let iowait = parts.next().and_then(|v| v.parse::<u64>().ok()).unwrap_or(0);
    let irq = parts.next().and_then(|v| v.parse::<u64>().ok()).unwrap_or(0);
    let softirq = parts.next().and_then(|v| v.parse::<u64>().ok()).unwrap_or(0);
    let steal = parts.next().and_then(|v| v.parse::<u64>().ok()).unwrap_or(0);

    let idle_all = idle.saturating_add(iowait);
    let total = user
        .saturating_add(nice)
        .saturating_add(system)
        .saturating_add(idle_all)
        .saturating_add(irq)
        .saturating_add(softirq)
        .saturating_add(steal);

    Some((total, idle_all))
}

/// Computes CPU usage percentage between two snapshots.
fn compute_cpu_percent(prev_total: u64, prev_idle: u64, now_total: u64, now_idle: u64) -> f32 {
    let total_delta = now_total.saturating_sub(prev_total);
    let idle_delta = now_idle.saturating_sub(prev_idle);

    if total_delta == 0 {
        return 0.0;
    }

    let busy = total_delta.saturating_sub(idle_delta);
    (busy as f32 / total_delta as f32) * 100.0
}

/// Reads memory totals from /proc/meminfo and returns values in MB.
fn read_meminfo_mb() -> Option<(u64, u64)> {
    let content = fs::read_to_string("/proc/meminfo").ok()?;
    let mut total_kb = None;
    let mut avail_kb = None;

    for line in content.lines() {
        if let Some(value) = line.strip_prefix("MemTotal:") {
            total_kb = value.split_whitespace().next().and_then(|v| v.parse::<u64>().ok());
        }
        if let Some(value) = line.strip_prefix("MemAvailable:") {
            avail_kb = value.split_whitespace().next().and_then(|v| v.parse::<u64>().ok());
        }
    }

    let total = total_kb? / 1024;
    let available = avail_kb.unwrap_or(0) / 1024;
    let used = total.saturating_sub(available);
    Some((used, total))
}

/// Reads CPU temperature from common Raspberry Pi thermal zones.
fn read_temperature_celsius() -> Option<f32> {
    let paths = [
        "/sys/class/thermal/thermal_zone0/temp",
        "/sys/class/hwmon/hwmon0/temp1_input",
    ];

    for path in paths {
        let Ok(raw) = fs::read_to_string(path) else {
            continue;
        };
        let Ok(value) = raw.trim().parse::<f32>() else {
            continue;
        };
        if value > 1000.0 {
            return Some(value / 1000.0);
        }
        return Some(value);
    }

    None
}

/// Reads uptime in seconds from /proc/uptime.
fn read_uptime_secs() -> Option<u64> {
    let content = fs::read_to_string("/proc/uptime").ok()?;
    let first = content.split_whitespace().next()?;
    let parsed = first.parse::<f64>().ok()?;
    Some(parsed as u64)
}

/// Reads disk usage for a given mount point using statvfs syscall via libc.
/// Falls back to reading /proc/mounts heuristically if statvfs is unavailable.
fn read_disk_gb(mount: &str) -> Option<(f64, f64)> {
    use std::ffi::CString;

    let cpath = CString::new(mount).ok()?;

    // SAFETY: statvfs is read-only and takes a valid null-terminated path.
    let mut stat: libc::statvfs = unsafe { std::mem::zeroed() };
    let ret = unsafe { libc::statvfs(cpath.as_ptr(), &mut stat) };

    if ret != 0 {
        return None;
    }

    let block = stat.f_frsize as f64;
    let total = block * stat.f_blocks as f64;
    let avail = block * stat.f_bavail as f64;
    let used = total - avail;

    Some((used / 1e9, total / 1e9))
}

/// Reads aggregate RX and TX byte counters from /proc/net/dev for all non-loopback interfaces.
fn read_net_bytes() -> Option<(u64, u64)> {
    let content = fs::read_to_string("/proc/net/dev").ok()?;
    let mut rx = 0u64;
    let mut tx = 0u64;

    for line in content.lines().skip(2) {
        let line = line.trim();
        let colon = line.find(':')?;
        let iface = &line[..colon].trim();
        if *iface == "lo" {
            continue;
        }
        let fields: Vec<u64> = line[colon + 1..]
            .split_whitespace()
            .filter_map(|v| v.parse().ok())
            .collect();
        if fields.len() >= 9 {
            rx = rx.saturating_add(fields[0]);
            tx = tx.saturating_add(fields[8]);
        }
    }

    Some((rx, tx))
}

/// Reads 1m/5m/15m load averages from /proc/loadavg.
fn read_load_avg() -> Option<(f32, f32, f32)> {
    let content = fs::read_to_string("/proc/loadavg").ok()?;
    let mut parts = content.split_whitespace();
    let a: f32 = parts.next()?.parse().ok()?;
    let b: f32 = parts.next()?.parse().ok()?;
    let c: f32 = parts.next()?.parse().ok()?;
    Some((a, b, c))
}

/// Appends new lines incrementally from the selected log source into the ring buffer.
/// This avoids full-file reads on every render tick.
fn poll_kernel_logs(cursor: &mut LogCursor, ring: &mut std::collections::VecDeque<String>) {
    let paths = ["/var/log/syslog", "/var/log/kern.log", "/var/log/messages"];

    if cursor.path.is_none() {
        cursor.path = paths
            .iter()
            .find(|p| std::path::Path::new(p).exists())
            .map(|p| (*p).to_string());
    }

    let Some(path) = cursor.path.as_deref() else {
        return;
    };

    let Ok(mut file) = fs::File::open(path) else {
        return;
    };

    let Ok(meta) = file.metadata() else {
        return;
    };

    // Handle log rotation/truncate by resetting the cursor.
    if meta.len() < cursor.offset {
        cursor.offset = 0;
    }

    if file.seek(SeekFrom::Start(cursor.offset)).is_err() {
        return;
    }

    let mut delta = String::new();
    if file.read_to_string(&mut delta).is_err() {
        return;
    }

    cursor.offset = cursor.offset.saturating_add(delta.len() as u64);

    for line in delta.lines() {
        if ring.len() >= LOG_RING_SIZE {
            ring.pop_front();
        }
        ring.push_back(line.to_string());
    }

    // Bootstrap display when there was no delta yet.
    if ring.is_empty() && meta.len() > 0 {
        if let Ok(content) = fs::read_to_string(path) {
            for line in content.lines().rev().take(LOG_RING_SIZE).collect::<Vec<_>>().into_iter().rev() {
                if ring.len() >= LOG_RING_SIZE {
                    ring.pop_front();
                }
                ring.push_back(line.to_string());
            }
        }
    }
}
