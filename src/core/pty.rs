use std::collections::VecDeque;
use std::ffi::CString;
use std::sync::{Arc, Mutex};
use std::thread;

/// Shared PTY state consumed by the terminal plugin.
#[derive(Debug, Default, Clone)]
pub struct PtyState {
    pub lines: VecDeque<String>,
    pub last_error: Option<String>,
}

const PTY_RING: usize = 120;

/// Starts a background PTY session and continuously captures shell output.
pub fn start_pty_stream(command: &str) -> Arc<Mutex<PtyState>> {
    let state = Arc::new(Mutex::new(PtyState::default()));
    let state_ref = Arc::clone(&state);
    let command = command.to_string();

    thread::spawn(move || {
        if let Err(err) = run_pty_capture(&command, &state_ref) {
            let mut s = state_ref.lock().unwrap();
            s.last_error = Some(err);
        }
    });

    state
}

/// Renders current PTY lines for panel display.
pub fn render_pty_panel(state: &PtyState) -> String {
    if let Some(err) = &state.last_error {
        return format!("PTY error\n{err}");
    }

    if state.lines.is_empty() {
        return "PTY: aguardando dados...".to_string();
    }

    state
        .lines
        .iter()
        .rev()
        .take(22)
        .cloned()
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join("\n")
}

/// Spawns a real PTY via forkpty and captures child shell output line-by-line.
fn run_pty_capture(command: &str, state: &Arc<Mutex<PtyState>>) -> Result<(), String> {
    let mut master_fd: libc::c_int = 0;

    // SAFETY: forkpty initializes a PTY pair and forks current process.
    let pid = unsafe {
        libc::forkpty(
            &mut master_fd,
            std::ptr::null_mut(),
            std::ptr::null(),
            std::ptr::null(),
        )
    };

    if pid < 0 {
        return Err("forkpty failed".to_string());
    }

    if pid == 0 {
        // Child: execute shell command attached to PTY slave.
        let shell = CString::new("/bin/sh").map_err(|_| "invalid shell path")?;
        let arg0 = CString::new("sh").map_err(|_| "invalid arg0")?;
        let arg1 = CString::new("-lc").map_err(|_| "invalid arg1")?;
        let script = CString::new(command).map_err(|_| "invalid command script")?;

        let argv = [arg0.as_ptr(), arg1.as_ptr(), script.as_ptr(), std::ptr::null()];

        // SAFETY: pointers are valid for this scope; on success this call never returns.
        unsafe {
            libc::execvp(shell.as_ptr(), argv.as_ptr());
            libc::_exit(127);
        }
    }

    // Parent: read PTY master output.
    let mut read_buf = [0u8; 2048];
    let mut pending = String::new();

    loop {
        // SAFETY: master_fd is a valid file descriptor in parent process.
        let n = unsafe { libc::read(master_fd, read_buf.as_mut_ptr() as *mut libc::c_void, read_buf.len()) };

        if n == 0 {
            return Ok(());
        }

        if n < 0 {
            return Err("read from PTY master failed".to_string());
        }

        let chunk = String::from_utf8_lossy(&read_buf[..n as usize]);
        pending.push_str(&chunk);

        while let Some(pos) = pending.find('\n') {
            let mut line = pending.drain(..=pos).collect::<String>();
            line = line.trim_end_matches(['\r', '\n']).to_string();
            if line.is_empty() {
                continue;
            }

            let mut s = state.lock().unwrap();
            if s.lines.len() >= PTY_RING {
                s.lines.pop_front();
            }
            s.lines.push_back(line);
        }
    }
}
