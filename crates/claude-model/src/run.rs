//! one run of the child: the prompt written to its stdin, its stdout and
//! stderr read to the end, and a deadline past which it is killed.

use std::io::{ErrorKind, Read, Write};
use std::path::Path;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use propose::ModelError;

/// how often the child is asked whether it has exited.
const POLL: Duration = Duration::from_millis(10);
/// how long the end of its output is waited for once it has exited: a
/// grandchild that keeps a pipe open holds the call this long at most,
/// and what the child wrote is kept.
const GRACE: Duration = Duration::from_secs(1);

/// what a child that exited left.
pub(crate) struct Exit {
    pub status: ExitStatus,
    pub stdout: String,
    pub stderr: String,
}

/// a pipe read on a thread of its own, into a buffer shared with the
/// caller, until its end.
struct Drain {
    bytes: Arc<Mutex<Vec<u8>>>,
    end: Receiver<()>,
}

impl Drain {
    fn new(from: Option<impl Read + Send + 'static>) -> Drain {
        let bytes = Arc::new(Mutex::new(Vec::new()));
        let (tx, end) = mpsc::channel();
        if let Some(mut from) = from {
            let into = Arc::clone(&bytes);
            thread::spawn(move || {
                let mut chunk = [0u8; 8192];
                loop {
                    match from.read(&mut chunk) {
                        Ok(0) => break,
                        Ok(n) => {
                            if let Ok(mut all) = into.lock() {
                                all.extend_from_slice(&chunk[..n]);
                            }
                        }
                        Err(e) if e.kind() == ErrorKind::Interrupted => {}
                        Err(_) => break,
                    }
                }
                let _ = tx.send(());
            });
        }
        Drain { bytes, end }
    }

    /// what was read: all of it once the pipe ends, or what came before
    /// `until`.
    fn text(self, until: Instant) -> String {
        let _ = self
            .end
            .recv_timeout(until.saturating_duration_since(Instant::now()));
        let bytes = self.bytes.lock().map(|b| b.clone()).unwrap_or_default();
        String::from_utf8_lossy(&bytes).into_owned()
    }
}

/// kills `child` by its pid and reaps it.
fn kill(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

/// runs `command` with `input` on its stdin and waits for it at most
/// `timeout`: past it, the child is killed and the error says so.
pub(crate) fn run(
    mut command: Command,
    program: &Path,
    input: &str,
    timeout: Duration,
) -> Result<Exit, ModelError> {
    let name = program.display();
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| ModelError(format!("could not start {name}: {e}")))?;
    let pid = child.id();
    if let Some(mut stdin) = child.stdin.take() {
        let input = input.to_owned();
        // a prompt is larger than a pipe's buffer: written on a thread, so
        // a child that answers before it reads everything cannot block the
        // wait. the pipe closes when the thread ends.
        thread::spawn(move || {
            let _ = stdin.write_all(input.as_bytes());
        });
    }
    let stdout = Drain::new(child.stdout.take());
    let stderr = Drain::new(child.stderr.take());
    let deadline = Instant::now() + timeout;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if Instant::now() >= deadline => {
                kill(&mut child);
                return Err(ModelError(format!(
                    "{name} did not answer in {} s: killed (pid {pid})",
                    timeout.as_secs_f64()
                )));
            }
            Ok(None) => thread::sleep(POLL),
            Err(e) => {
                kill(&mut child);
                return Err(ModelError(format!("could not wait for {name}: {e}")));
            }
        }
    };
    let until = Instant::now() + GRACE;
    Ok(Exit {
        status,
        stdout: stdout.text(until),
        stderr: stderr.text(until),
    })
}
