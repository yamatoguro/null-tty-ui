use std::fs::{self, OpenOptions};
use std::io::Write;
use std::time::{Duration, Instant};

/// Defines target thresholds for runtime health checks.
#[derive(Debug, Clone, Copy)]
pub struct PerformanceTargets {
    pub min_fps: f32,
    pub max_process_cpu_percent: f32,
    pub max_process_rss_mb: u64,
}

impl Default for PerformanceTargets {
    fn default() -> Self {
        Self {
            min_fps: 2.0,
            max_process_cpu_percent: 20.0,
            max_process_rss_mb: 180,
        }
    }
}

/// Tracks render cadence and process resource usage and writes periodic diagnostics.
pub struct RuntimeMonitor {
    started_at: Instant,
    last_report: Instant,
    last_frame_mark: Instant,
    frames_since_last: u64,
    last_proc_ticks: Option<u64>,
    diag_log_path: Option<String>,
    targets: PerformanceTargets,
    clk_tck: f64,
}

impl RuntimeMonitor {
    /// Creates a monitor with optional file logging.
    pub fn new(diag_log_path: Option<String>, targets: PerformanceTargets) -> Self {
        let clk_tck = unsafe { libc::sysconf(libc::_SC_CLK_TCK) };
        let clk_tck = if clk_tck > 0 { clk_tck as f64 } else { 100.0 };

        Self {
            started_at: Instant::now(),
            last_report: Instant::now(),
            last_frame_mark: Instant::now(),
            frames_since_last: 0,
            last_proc_ticks: None,
            diag_log_path,
            targets,
            clk_tck,
        }
    }

    /// Increments the rendered frame counter.
    pub fn on_frame_rendered(&mut self) {
        self.frames_since_last = self.frames_since_last.saturating_add(1);
    }

    /// Emits a periodic diagnostics line every report_interval.
    pub fn report_if_due(&mut self, report_interval: Duration) {
        let now = Instant::now();
        if now.duration_since(self.last_report) < report_interval {
            return;
        }

        let frame_dt = now.duration_since(self.last_frame_mark).as_secs_f32();
        let fps = if frame_dt > 0.0 {
            self.frames_since_last as f32 / frame_dt
        } else {
            0.0
        };

        let rss_mb = read_process_rss_mb().unwrap_or(0);
        let cpu_percent = self.read_process_cpu_percent(now).unwrap_or(0.0);

        let fps_ok = fps >= self.targets.min_fps;
        let cpu_ok = cpu_percent <= self.targets.max_process_cpu_percent;
        let mem_ok = rss_mb <= self.targets.max_process_rss_mb;

        let line = format!(
            "uptime_s={} fps={:.2} target_fps>={:.2} proc_cpu={:.2}% target_cpu<={:.2}% proc_rss={}MB target_rss<={}MB status={}",
            self.started_at.elapsed().as_secs(),
            fps,
            self.targets.min_fps,
            cpu_percent,
            self.targets.max_process_cpu_percent,
            rss_mb,
            self.targets.max_process_rss_mb,
            if fps_ok && cpu_ok && mem_ok { "ok" } else { "degraded" }
        );

        self.append_log_line(&line);

        self.last_report = now;
        self.last_frame_mark = now;
        self.frames_since_last = 0;
    }

    /// Writes one diagnostics line to stderr and optional file.
    fn append_log_line(&self, line: &str) {
        eprintln!("[diagnostics] {line}");

        let Some(path) = &self.diag_log_path else {
            return;
        };

        let parent = std::path::Path::new(path).parent();
        if let Some(parent) = parent {
            let _ = fs::create_dir_all(parent);
        }

        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
            let _ = writeln!(file, "{line}");
        }
    }

    /// Reads CPU usage of this process between reports.
    fn read_process_cpu_percent(&mut self, now: Instant) -> Option<f32> {
        let current_ticks = read_process_cpu_ticks()?;
        let elapsed = now.duration_since(self.last_report).as_secs_f64();

        let prev = self.last_proc_ticks.replace(current_ticks)?;
        if elapsed <= 0.0 {
            return Some(0.0);
        }

        let delta_ticks = current_ticks.saturating_sub(prev) as f64;
        let cpu_seconds = delta_ticks / self.clk_tck;
        Some(((cpu_seconds / elapsed) * 100.0) as f32)
    }
}

/// Parses process CPU ticks from /proc/self/stat (utime + stime).
fn read_process_cpu_ticks() -> Option<u64> {
    let stat = fs::read_to_string("/proc/self/stat").ok()?;

    // Field parsing after the command name and state section.
    let end_comm = stat.rfind(')')?;
    let rest = stat.get(end_comm + 2..)?;
    let fields: Vec<&str> = rest.split_whitespace().collect();

    // After removing first two fields, utime and stime map to indexes 11 and 12.
    let utime = fields.get(11)?.parse::<u64>().ok()?;
    let stime = fields.get(12)?.parse::<u64>().ok()?;
    Some(utime.saturating_add(stime))
}

/// Reads VmRSS from /proc/self/status and returns MB.
fn read_process_rss_mb() -> Option<u64> {
    let status = fs::read_to_string("/proc/self/status").ok()?;
    for line in status.lines() {
        if let Some(value) = line.strip_prefix("VmRSS:") {
            let kb = value
                .split_whitespace()
                .next()
                .and_then(|v| v.parse::<u64>().ok())?;
            return Some(kb / 1024);
        }
    }
    None
}
