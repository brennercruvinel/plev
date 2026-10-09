//! One device's child and its threads: ffmpeg started with its pipes,
//! stdout read into frames (the newest replacing the untaken) or chunks
//! (a ring bounded to two seconds), stderr into lines, and the watcher
//! that asks the child whether it ended, applies the deadline, reads the
//! state from what it said, and starts a camera once more with a mode it
//! listed.

use std::collections::VecDeque;
use std::io::{BufRead, BufReader, ErrorKind, Read};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStderr, ChildStdout, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use super::super::super::menu::locked;
use super::super::{AudioChunk, CaptureError, CaptureSettings, CaptureState, DeviceKind, Frame};
use super::args::{Input, Picked, args, classify, parse_modes, pick_mode, rate_arg, refused_mode};

/// How often the watcher asks the child whether it ended.
const POLL: Duration = Duration::from_millis(20);

/// How long the end of stderr is waited for once the child ended.
const GRACE: Duration = Duration::from_secs(1);

/// The lines of stderr kept for the state: a camera's list of modes is
/// a line per format and rate.
const STDERR_LINES: usize = 128;

/// What one child's threads share with the capture.
#[derive(Debug)]
pub(super) struct Shared {
    pub kind: DeviceKind,
    pub state: Mutex<CaptureState>,
    /// The camera's newest frame not taken yet.
    pub frame: Mutex<Option<Frame>>,
    /// The microphone's chunks not taken yet.
    pub audio: Mutex<VecDeque<AudioChunk>>,
    /// The last lines of stderr, of the child running now.
    pub stderr: Mutex<VecDeque<String>>,
    /// The capture was stopped: the threads end, the state stays `Off`.
    pub stopping: AtomicBool,
    pub pid: AtomicU32,
}

impl Shared {
    /// A device just asked for: `Starting`, nothing handed yet.
    pub fn new(kind: DeviceKind) -> Shared {
        Shared {
            kind,
            state: Mutex::new(CaptureState::Starting),
            frame: Mutex::new(None),
            audio: Mutex::new(VecDeque::new()),
            stderr: Mutex::new(VecDeque::new()),
            stopping: AtomicBool::new(false),
            pid: AtomicU32::new(0),
        }
    }

    fn set(&self, state: CaptureState) {
        log::info!("capture: {} {state:?}", self.kind.noun());
        *locked(&self.state) = state;
    }

    /// The first frame or samples: the device is on.
    fn arrived(&self) {
        let mut state = locked(&self.state);
        if *state == CaptureState::Starting {
            log::info!("capture: {} on", self.kind.noun());
            *state = CaptureState::On;
        }
    }

    fn lines(&self) -> Vec<String> {
        locked(&self.stderr).iter().cloned().collect()
    }

    pub fn stopping(&self) -> bool {
        self.stopping.load(Ordering::SeqCst)
    }
}

/// Starts ffmpeg with `args` and the threads that read it: stdout into
/// frames or chunks, stderr into lines. The receiver ends when stderr
/// does.
pub(super) fn launch(
    program: &Path,
    args: &[String],
    settings: &CaptureSettings,
    shared: &Arc<Shared>,
) -> Result<(Child, Receiver<()>), CaptureError> {
    log::info!(
        "capture: {} with {} {}",
        shared.kind.noun(),
        program.display(),
        args.join(" ")
    );
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| CaptureError::Io(format!("could not start {}: {e}", program.display())))?;
    shared.pid.store(child.id(), Ordering::SeqCst);
    let (tx, ended) = mpsc::channel();
    if let Some(err) = child.stderr.take() {
        let into = Arc::clone(shared);
        thread::spawn(move || {
            read_lines(err, &into);
            let _ = tx.send(());
        });
    }
    if let Some(out) = child.stdout.take() {
        let into = Arc::clone(shared);
        let settings = settings.clone();
        thread::spawn(move || match into.kind {
            DeviceKind::Camera => read_frames(out, &into, &settings),
            DeviceKind::Microphone => read_samples(out, &into, &settings),
        });
    }
    Ok((child, ended))
}

/// Frames of `frame_bytes` each off the pipe; the newest replaces the one
/// nobody took, whose buffer the next read reuses.
fn read_frames(mut out: ChildStdout, shared: &Shared, settings: &CaptureSettings) {
    let bytes = settings.frame_bytes();
    let mut buf = vec![0u8; bytes];
    while out.read_exact(&mut buf).is_ok() && !shared.stopping() {
        let frame = Frame {
            width: settings.width,
            height: settings.height,
            rgba: buf,
            at: Instant::now(),
        };
        let old = locked(&shared.frame).replace(frame);
        buf = old.map(|f| f.rgba).unwrap_or_else(|| vec![0u8; bytes]);
        shared.arrived();
    }
}

/// Samples off the pipe, in chunks of a twentieth of a second, into the
/// ring bounded to two seconds.
fn read_samples(mut out: ChildStdout, shared: &Shared, settings: &CaptureSettings) {
    let chunk_bytes = settings.chunk_samples() * 4;
    let mut pending: Vec<u8> = Vec::with_capacity(chunk_bytes * 2);
    let mut read = [0u8; 8192];
    loop {
        let n = match out.read(&mut read) {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(_) => break,
        };
        if shared.stopping() {
            break;
        }
        pending.extend_from_slice(&read[..n]);
        while pending.len() >= chunk_bytes {
            let samples = pending[..chunk_bytes]
                .chunks_exact(4)
                .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
                .collect();
            pending.drain(..chunk_bytes);
            let chunk = AudioChunk {
                samples,
                rate: settings.rate,
                at: Instant::now(),
            };
            push_bounded(&mut locked(&shared.audio), chunk, settings.ring_samples());
            shared.arrived();
        }
    }
}

/// A bounded ring of chunks: past `max_samples` the oldest chunks go.
fn push_bounded(ring: &mut VecDeque<AudioChunk>, chunk: AudioChunk, max_samples: usize) {
    ring.push_back(chunk);
    let mut held: usize = ring.iter().map(|c| c.samples.len()).sum();
    while held > max_samples && ring.len() > 1 {
        if let Some(old) = ring.pop_front() {
            held -= old.samples.len();
        }
    }
}

/// stderr, line by line, the last ones kept.
fn read_lines(err: ChildStderr, shared: &Shared) {
    for line in BufReader::new(err).lines() {
        let Ok(line) = line else { break };
        log::debug!("capture: {}: {line}", shared.kind.noun());
        let mut lines = locked(&shared.stderr);
        if lines.len() == STDERR_LINES {
            lines.pop_front();
        }
        lines.push_back(line);
    }
}

/// The watcher of one device's child.
pub(super) struct Watch {
    pub program: PathBuf,
    pub input: Input,
    pub settings: CaptureSettings,
    pub first: Duration,
    pub shared: Arc<Shared>,
    pub child: Arc<Mutex<Option<Child>>>,
}

impl Watch {
    /// Until the child ends or the capture stops: the deadline for the
    /// first frame, the state from what the child said, and one more
    /// start with a mode the camera listed when it refused its default.
    pub fn run(self, mut ended: Receiver<()>) {
        let mut started = Instant::now();
        let mut retried = false;
        loop {
            thread::sleep(POLL);
            if self.shared.stopping() {
                return;
            }
            let mut slot = locked(&self.child);
            let Some(child) = slot.as_mut() else {
                return;
            };
            let exited = match child.try_wait() {
                Ok(None) => false,
                Ok(Some(_)) | Err(_) => true,
            };
            let starting = *locked(&self.shared.state) == CaptureState::Starting;
            if !exited && !(starting && started.elapsed() >= self.first) {
                continue;
            }
            if !exited {
                let _ = child.kill();
            }
            let _ = child.wait();
            *slot = None;
            drop(slot);
            let _ = ended.recv_timeout(GRACE);
            if self.shared.stopping() {
                return;
            }
            let lines = self.shared.lines();
            if !exited {
                self.shared.set(self.past_deadline(&lines));
                return;
            }
            if starting && !retried && refused_mode(&lines) {
                if let Some(mode) = pick_mode(&parse_modes(&lines), &self.settings) {
                    retried = true;
                    match self.again(mode) {
                        Ok(Some(next)) => {
                            ended = next;
                            started = Instant::now();
                            continue;
                        }
                        Ok(None) => return,
                        Err(e) => {
                            self.shared.set(CaptureState::Failed(e.to_string()));
                            return;
                        }
                    }
                }
            }
            let state = match classify(&lines) {
                CaptureState::Failed(why) if !starting => {
                    CaptureState::Failed(format!("the {} stopped: {why}", self.shared.kind.noun()))
                }
                other => other,
            };
            self.shared.set(state);
            return;
        }
    }

    /// A child killed at the deadline: what it said, if it said why, else
    /// that nothing came.
    fn past_deadline(&self, lines: &[String]) -> CaptureState {
        match classify(lines) {
            CaptureState::Failed(_) => CaptureState::Failed(format!(
                "nothing from the {} in {} s{}",
                self.shared.kind.noun(),
                self.first.as_secs(),
                lines
                    .last()
                    .map(|l| format!(": {}", l.trim()))
                    .unwrap_or_default()
            )),
            other => other,
        }
    }

    /// The camera once more, with a mode it listed; `None` when the
    /// capture was stopped meanwhile.
    fn again(&self, mode: Picked) -> Result<Option<Receiver<()>>, CaptureError> {
        log::info!(
            "capture: the camera refused its default, again at {}x{} and {} fps",
            mode.width,
            mode.height,
            rate_arg(mode.rate)
        );
        let args = args(self.input, self.shared.kind, &self.settings, Some(mode))?;
        let mut slot = locked(&self.child);
        if self.shared.stopping() {
            return Ok(None);
        }
        locked(&self.shared.stderr).clear();
        let (child, ended) = launch(&self.program, &args, &self.settings, &self.shared)?;
        *slot = Some(child);
        Ok(Some(ended))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_ring_drops_the_oldest_past_its_bound() {
        let chunk = |v: f32| AudioChunk {
            samples: vec![v; 10],
            rate: 100,
            at: Instant::now(),
        };
        let mut ring = VecDeque::new();
        for v in 0..10 {
            push_bounded(&mut ring, chunk(v as f32), 35);
        }
        assert_eq!(ring.len(), 3);
        assert_eq!(ring.front().map(|c| c.samples[0]), Some(7.0));
    }
}
