//! Run a child process to completion with a timeout, capturing stdout and
//! stderr. Shared by the embedder bridge and the frame decoder.
//!
//! Both pipes are drained on their own threads from the start: a child
//! that writes more than the OS pipe buffer (64 KiB on macOS; one PNG
//! frame is already past that) would otherwise block on `write` while we
//! waited for it to exit, and neither side would ever finish.

use std::io::Read;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

/// Why a run did not produce stdout.
#[derive(Debug)]
pub enum RunError {
    /// `spawn` failed (program missing, not executable…).
    Spawn(std::io::Error),
    /// The child ran past `timeout` and was killed.
    Timeout,
    /// Non-zero exit; carries stderr, trimmed.
    Failed(String),
}

/// Read a pipe to the end on its own thread.
fn drain<R: Read + Send + 'static>(pipe: Option<R>) -> std::thread::JoinHandle<Vec<u8>> {
    std::thread::spawn(move || {
        let mut buf = Vec::new();
        if let Some(mut r) = pipe {
            let _ = r.read_to_end(&mut buf);
        }
        buf
    })
}

/// Spawn `program args…` with stdin closed, wait at most `timeout`, and
/// return stdout on a zero exit. A timeout kills and reaps the child.
pub fn run(program: &str, args: &[String], timeout: Duration) -> Result<Vec<u8>, RunError> {
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(RunError::Spawn)?;
    let stdout = drain(child.stdout.take());
    let stderr = drain(child.stderr.take());
    let status = wait_timeout(&mut child, timeout);
    let out = stdout.join().unwrap_or_default();
    let err = stderr.join().unwrap_or_default();
    match status {
        Some(Ok(s)) if s.success() => Ok(out),
        Some(Ok(_)) => Err(RunError::Failed(
            String::from_utf8_lossy(&err).trim().to_string(),
        )),
        Some(Err(e)) => Err(RunError::Spawn(e)),
        None => Err(RunError::Timeout),
    }
}

/// Poll `try_wait` until exit or `timeout`; on timeout kill + reap and
/// return `None`. The pipes are being drained elsewhere, so the child can
/// never block on a full pipe while we poll.
fn wait_timeout(
    child: &mut Child,
    timeout: Duration,
) -> Option<std::io::Result<std::process::ExitStatus>> {
    let deadline = std::time::Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Some(Ok(status)),
            Ok(None) => {}
            Err(e) => return Some(Err(e)),
        }
        if std::time::Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return None;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn captures_stdout_larger_than_a_pipe_buffer() {
        // 200 KiB through the pipe: would deadlock without the drain
        // thread.
        let out = run(
            "sh",
            &["-c".into(), "head -c 204800 /dev/zero".into()],
            Duration::from_secs(10),
        )
        .unwrap();
        assert_eq!(out.len(), 204_800);
    }

    #[test]
    fn nonzero_exit_carries_stderr() {
        let err = run(
            "sh",
            &["-c".into(), "echo boom >&2; exit 3".into()],
            Duration::from_secs(10),
        )
        .unwrap_err();
        assert!(matches!(err, RunError::Failed(ref s) if s == "boom"));
    }

    #[test]
    fn timeout_kills_the_child() {
        let t0 = std::time::Instant::now();
        let err = run(
            "sh",
            &["-c".into(), "sleep 30".into()],
            Duration::from_millis(200),
        )
        .unwrap_err();
        assert!(matches!(err, RunError::Timeout));
        assert!(t0.elapsed() < Duration::from_secs(5));
    }

    #[test]
    fn missing_program_is_a_spawn_error() {
        let err = run("/definitely/not/a/program", &[], Duration::from_secs(1)).unwrap_err();
        assert!(matches!(err, RunError::Spawn(_)));
    }
}
