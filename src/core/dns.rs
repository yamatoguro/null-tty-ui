use std::collections::VecDeque;
use std::net::TcpStream;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// A single DNS stat sample polled from the Technitium API.
#[derive(Debug, Clone)]
pub struct DnsSample {
    pub total_queries: u64,
    pub blocked: u64,
    pub allowed: u64,
    pub cache_hits: u64,
    pub timestamp_secs: u64,
}

/// Shared DNS data structure updated by the polling thread.
#[derive(Debug, Default, Clone)]
pub struct DnsState {
    pub samples: VecDeque<DnsSample>,
    pub last_error: Option<String>,
}

const SAMPLE_WINDOW: usize = 60;

/// Launches a background thread that polls Technitium at the given interval.
/// Returns a thread-safe handle to the shared DNS state.
pub fn start_poller(host: &str, port: u16, token: &str, interval: Duration) -> Arc<Mutex<DnsState>> {
    let state = Arc::new(Mutex::new(DnsState::default()));
    let state_ref = Arc::clone(&state);
    let host = host.to_string();
    let token = token.to_string();

    thread::spawn(move || {
        loop {
            match fetch_stats(&host, port, &token) {
                Ok(sample) => {
                    let mut s = state_ref.lock().unwrap();
                    if s.samples.len() >= SAMPLE_WINDOW {
                        s.samples.pop_front();
                    }
                    s.samples.push_back(sample);
                    s.last_error = None;
                }
                Err(e) => {
                    let mut s = state_ref.lock().unwrap();
                    s.last_error = Some(e.to_string());
                }
            }
            thread::sleep(interval);
        }
    });

    state
}

/// Renders the DNS state into a human-readable panel string with an ASCII bar chart.
pub fn render_dns_panel(state: &DnsState) -> String {
    if let Some(err) = &state.last_error {
        return format!("DNS: offline\n{err}");
    }

    let samples = &state.samples;
    if samples.is_empty() {
        return "DNS: no data yet".to_string();
    }

    let last = samples.back().unwrap();
    let total = last.total_queries;
    let blocked = last.blocked;
    let allowed = last.allowed;
    let cache = last.cache_hits;
    let age_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs().saturating_sub(last.timestamp_secs))
        .unwrap_or(0);

    let block_pct = if total > 0 { blocked * 100 / total } else { 0 };
    let cache_pct = if total > 0 { cache * 100 / total } else { 0 };
    let alert = if block_pct >= 70 {
        "ALERT high block rate"
    } else if block_pct >= 40 {
        "WARN elevated block rate"
    } else {
        "OK"
    };

    // Build a mini sparkline from total query counts over time.
    let sparkline = build_sparkline(samples.iter().map(|s| s.total_queries));

    format!(
        "total   {:>7}  allowed {:>7}\nblocked {:>7}  ({block_pct}%)  cache {cache_pct}%\nalert   {}\nage     {}s\n\nqueries/min:\n{sparkline}",
        total, allowed, blocked, alert, age_secs
    )
}

/// Builds a fixed-width ASCII sparkline from a series of u64 values.
fn build_sparkline(values: impl Iterator<Item = u64>) -> String {
    let bars = " ▁▂▃▄▅▆▇█";
    let vals: Vec<u64> = values.collect();
    if vals.is_empty() {
        return String::new();
    }

    let max = *vals.iter().max().unwrap_or(&1);
    let max = max.max(1);

    vals.iter()
        .map(|v| {
            let idx = (*v * 8 / max) as usize;
            bars.chars().nth(idx.min(8)).unwrap_or(' ')
        })
        .collect()
}

/// Performs a minimal HTTP/1.0 GET to the Technitium stats endpoint using raw TCP.
/// No async runtime required; runs in a dedicated background thread.
fn fetch_stats(host: &str, port: u16, token: &str) -> anyhow::Result<DnsSample> {
    use anyhow::Context;

    let addr = format!("{host}:{port}");
    let path = format!("/api/dashboard/stats/get?token={token}&type=LastMinute");

    let mut stream = TcpStream::connect(&addr)
        .with_context(|| format!("tcp connect to {addr}"))?;
    stream.set_read_timeout(Some(Duration::from_secs(4)))?;
    stream.set_write_timeout(Some(Duration::from_secs(4)))?;

    let request = format!("GET {path} HTTP/1.0\r\nHost: {host}\r\nConnection: close\r\n\r\n");
    stream.write_all(request.as_bytes())?;

    let mut response = String::new();
    stream.read_to_string(&mut response)?;

    parse_stats_response(&response)
}

/// Extracts stat fields from a minimal JSON response body without pulling in a JSON crate.
fn parse_stats_response(raw: &str) -> anyhow::Result<DnsSample> {
    use anyhow::bail;

    // Split headers from body on blank line.
    let body = if let Some(idx) = raw.find("\r\n\r\n") {
        &raw[idx + 4..]
    } else if let Some(idx) = raw.find("\n\n") {
        &raw[idx + 2..]
    } else {
        bail!("no body separator in HTTP response");
    };

    fn extract_u64(body: &str, key: &str) -> u64 {
        let search = format!("\"{}\":", key);
        body.find(search.as_str())
            .and_then(|i| {
                let rest = &body[i + search.len()..];
                let rest = rest.trim_start();
                rest.split(|c: char| !c.is_ascii_digit()).next()
            })
            .and_then(|s| s.parse().ok())
            .unwrap_or(0)
    }

    let total = extract_u64(body, "totalQueries");
    let blocked = extract_u64(body, "totalBlocked");
    let cache = extract_u64(body, "totalCachedHits");
    let allowed = total.saturating_sub(blocked);
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    Ok(DnsSample {
        total_queries: total,
        blocked,
        allowed,
        cache_hits: cache,
        timestamp_secs: ts,
    })
}
