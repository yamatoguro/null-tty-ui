use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Shared file navigation state rendered by the file_navigation plugin.
#[derive(Debug, Clone)]
pub struct FileNavState {
    pub root: PathBuf,
    pub lines: VecDeque<String>,
    pub last_error: Option<String>,
}

impl FileNavState {
    fn new(root: PathBuf) -> Self {
        Self {
            root,
            lines: VecDeque::new(),
            last_error: None,
        }
    }
}

const FILE_RING: usize = 64;

/// Starts a background poller that refreshes directory listing at a fixed cadence.
pub fn start_file_nav_poller(root: PathBuf, interval: Duration) -> Arc<Mutex<FileNavState>> {
    let state = Arc::new(Mutex::new(FileNavState::new(root.clone())));
    let state_ref = Arc::clone(&state);

    thread::spawn(move || {
        loop {
            match list_dir_lines(&root) {
                Ok(lines) => {
                    let mut s = state_ref.lock().unwrap();
                    s.lines.clear();
                    for line in lines.into_iter().take(FILE_RING) {
                        s.lines.push_back(line);
                    }
                    s.last_error = None;
                }
                Err(err) => {
                    let mut s = state_ref.lock().unwrap();
                    s.last_error = Some(err);
                }
            }
            thread::sleep(interval);
        }
    });

    state
}

/// Renders the current file navigation snapshot for panel display.
pub fn render_file_nav_panel(state: &FileNavState) -> String {
    if let Some(err) = &state.last_error {
        return format!("file_navigation: erro\n{err}");
    }

    if state.lines.is_empty() {
        return format!("root: {}\n(no entries)", state.root.display());
    }

    let mut out = vec![format!("root: {}", state.root.display())];
    out.extend(state.lines.iter().cloned());
    out.join("\n")
}

/// Lists one directory level with folders first and basic file size metadata.
fn list_dir_lines(root: &PathBuf) -> Result<Vec<String>, String> {
    let read = fs::read_dir(root)
        .map_err(|e| format!("cannot read {}: {e}", root.display()))?;

    let mut dirs = Vec::new();
    let mut files = Vec::new();

    for entry in read {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if path.is_dir() {
            dirs.push(format!("[D] {name}"));
        } else {
            let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
            files.push(format!("[F] {name} ({})", human_size(size)));
        }
    }

    dirs.sort();
    files.sort();

    dirs.extend(files);
    Ok(dirs)
}

/// Formats file sizes into compact human-friendly units.
fn human_size(bytes: u64) -> String {
    if bytes >= 1_000_000_000 {
        format!("{:.1}GB", bytes as f64 / 1_000_000_000.0)
    } else if bytes >= 1_000_000 {
        format!("{:.1}MB", bytes as f64 / 1_000_000.0)
    } else if bytes >= 1_000 {
        format!("{:.1}KB", bytes as f64 / 1_000.0)
    } else {
        format!("{bytes}B")
    }
}
