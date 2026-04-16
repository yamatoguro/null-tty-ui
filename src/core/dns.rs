use std::collections::VecDeque;
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use anyhow::{bail, Context};
use serde_json::Value;

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
        let mut last_logged_error: Option<String> = None;
        loop {
            match fetch_stats(&host, port, &token) {
                Ok(sample) => {
                    let mut s = state_ref.lock().unwrap();
                    if s.samples.len() >= SAMPLE_WINDOW {
                        s.samples.pop_front();
                    }
                    s.samples.push_back(sample);
                    s.last_error = None;

                    if last_logged_error.is_some() {
                        append_dns_log("[dns] poller recovered and data collection resumed");
                        last_logged_error = None;
                    }
                }
                Err(e) => {
                    let mut s = state_ref.lock().unwrap();
                    let msg = e.to_string();
                    s.last_error = Some(msg.clone());

                    if last_logged_error.as_deref() != Some(msg.as_str()) {
                        append_dns_log(&format!("[dns] poller error: {msg}"));
                        last_logged_error = Some(msg);
                    }
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

/// Returns latest total queries value, if available.
pub fn latest_total_queries(state: &DnsState) -> Option<u64> {
    state.samples.back().map(|s| s.total_queries)
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
    let addr = format!("{host}:{port}");
    let paths = build_candidate_paths(token);
    let mut last_err: Option<anyhow::Error> = None;

    for path in paths {
        match fetch_stats_from_path(host, &addr, &path) {
            Ok(sample) => return Ok(sample),
            Err(err) => last_err = Some(err),
        }
    }

    Err(last_err.unwrap_or_else(|| anyhow::anyhow!("no DNS endpoint candidates available")))
}

fn fetch_stats_from_path(host: &str, addr: &str, path: &str) -> anyhow::Result<DnsSample> {
    let mut stream = TcpStream::connect(addr).with_context(|| format!("tcp connect to {addr}"))?;
    stream.set_read_timeout(Some(Duration::from_secs(4)))?;
    stream.set_write_timeout(Some(Duration::from_secs(4)))?;

    let request = format!("GET {path} HTTP/1.0\r\nHost: {host}\r\nConnection: close\r\n\r\n");
    stream.write_all(request.as_bytes())?;

    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    parse_stats_response(&response).with_context(|| format!("while parsing endpoint {path}"))
}

fn build_candidate_paths(token: &str) -> Vec<String> {
    let mut paths = vec![
        "/api/dashboard/stats/get?type=LastMinute".to_string(),
        "/api/dashboard/stats".to_string(),
    ];

    if !token.trim().is_empty() {
        paths.insert(0, format!("/api/dashboard/stats/get?token={token}&type=LastMinute"));
        paths.insert(1, format!("/api/dashboard/stats?token={token}"));
    }

    paths
}

/// Extracts DNS stats from JSON body returned by Technitium endpoints.
fn parse_stats_response(raw: &str) -> anyhow::Result<DnsSample> {
    // Split headers from body on blank line.
    let (status_line, body) = split_http_response(raw)?;
    if !status_line.contains(" 200 ") {
        bail!("http status not ok: {status_line}");
    }

    let value: Value = serde_json::from_str(body).with_context(|| "invalid JSON in DNS response")?;

    if let Some(err_msg) = extract_string_by_keys(&value, &["error", "message", "statusText"]) {
        if err_msg.to_ascii_lowercase().contains("error") {
            bail!("api error: {err_msg}");
        }
    }

    let total = extract_u64_by_keys(&value, &["totalQueries", "queries", "totalRequests"]).unwrap_or(0);
    let blocked = extract_u64_by_keys(&value, &["totalBlocked", "blocked", "blockedQueries"]).unwrap_or(0);
    let cache = extract_u64_by_keys(&value, &["totalCachedHits", "cacheHits", "cachedQueries"]).unwrap_or(0);
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

fn split_http_response(raw: &str) -> anyhow::Result<(&str, &str)> {
    let status_line = raw.lines().next().unwrap_or("unknown-status");

    if let Some(idx) = raw.find("\r\n\r\n") {
        return Ok((status_line, &raw[idx + 4..]));
    }
    if let Some(idx) = raw.find("\n\n") {
        return Ok((status_line, &raw[idx + 2..]));
    }

    bail!("no body separator in HTTP response")
}

fn extract_u64_by_keys(root: &Value, keys: &[&str]) -> Option<u64> {
    keys.iter().find_map(|key| extract_u64_recursive(root, key))
}

fn extract_u64_recursive(value: &Value, key: &str) -> Option<u64> {
    match value {
        Value::Object(map) => {
            if let Some(candidate) = map.get(key) {
                if let Some(n) = to_u64(candidate) {
                    return Some(n);
                }
            }
            map.values().find_map(|v| extract_u64_recursive(v, key))
        }
        Value::Array(items) => items.iter().find_map(|v| extract_u64_recursive(v, key)),
        _ => None,
    }
}

fn extract_string_by_keys(root: &Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| extract_string_recursive(root, key))
}

fn extract_string_recursive(value: &Value, key: &str) -> Option<String> {
    match value {
        Value::Object(map) => {
            if let Some(candidate) = map.get(key) {
                if let Some(s) = candidate.as_str() {
                    return Some(s.to_string());
                }
            }
            map.values().find_map(|v| extract_string_recursive(v, key))
        }
        Value::Array(items) => items.iter().find_map(|v| extract_string_recursive(v, key)),
        _ => None,
    }
}

fn to_u64(value: &Value) -> Option<u64> {
    match value {
        Value::Number(n) => n.as_u64().or_else(|| n.as_i64().and_then(|v| u64::try_from(v).ok())),
        Value::String(s) => s.parse::<u64>().ok(),
        _ => None,
    }
}

fn append_dns_log(message: &str) {
    let log_path = "/tmp/nullbyteui/startup-diagnostics.log";
    if let Some(parent) = std::path::Path::new(log_path).parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(log_path) {
        let _ = writeln!(file, "{message}");
    }
}
