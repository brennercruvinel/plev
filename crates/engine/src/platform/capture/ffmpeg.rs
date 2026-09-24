//! Capture through ffmpeg as a child process, one per device: the
//! platform's input (`avfoundation` on macOS, `dshow` on windows, `v4l2`
//! and `alsa` on linux) read by ffmpeg, the frames scaled and cropped to
//! the asked size and rate and written raw (`-f rawvideo -pix_fmt rgba`),
//! the samples written raw (`-f f32le -ac 1 -ar <rate>`), both to its
//! stdout. A thread per child reads the pipe: the camera's newest frame
//! replaces the one nobody took (a slot of one), the microphone's chunks
//! go into a ring bounded to two seconds. A watcher thread per child
//! asks it every few milliseconds whether it ended, reads what it said
//! (every line of stderr, since the last one is often a generic
//! "Input/output error"), kills it by its pid when no frame came within
//! the deadline, and stopping kills it and reaps it: the device is
//! released when the process is gone. A parent that dies without
//! stopping leaves ffmpeg writing into a pipe nobody reads: its next
//! write fails and it ends.
//!
//! The form is the showcase's ffmpeg worker and the CUPS printer (a child
//! with a deadline, killed by its pid), not a dependency on either.
//! `PLEV_FFMPEG` names another program (a test's fake); without it
//! `ffmpeg` is looked up on the PATH by the spawn. Nothing runs until a
//! device is started.
//!
//! The command line and the reading of stderr are `args.rs`, the child
//! and its threads `child.rs`.
//!
//! What is read here and was not seen on a device (the probe of task-76
//! item 10; no device was opened): the input syntax of each platform from
//! `ffmpeg-devices(1)`; the texts of avfoundation's refusals from the
//! strings of libavdevice 63.1.101 ("Invalid device index", "Video device
//! not found", "Audio device not found", "No AV capture device found",
//! "Failed to create AV capture input device: %s", "Selected framerate
//! (%f) is not supported by the device.", "Supported modes:" and "
//! %dx%d@[%f %f]fps"). ffmpeg asks the system for no permission itself:
//! the `%s` of the creation failure is the system's description, and what
//! it says when the permission is denied ("not authorized", "Cannot use")
//! is a guess. A camera that does not take the default rate (29.97)
//! lists its modes and ends; the capture starts once more with a mode it
//! listed.

use std::path::{Path, PathBuf};
use std::process::Child;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use super::super::menu::locked;
use super::{
    AudioChunk, Capture, CaptureError, CaptureSettings, CaptureState, DeviceKind, Frame, slot,
};

mod args;
mod child;

pub use args::Input;
use args::args;
use child::{Shared, Watch, launch};

/// The variable that names another ffmpeg (a test's fake).
pub const FFMPEG_ENV: &str = "PLEV_FFMPEG";

/// How long a device may take to hand its first frame or samples: a
/// camera warming up, and a person answering the system's prompt.
const FIRST: Duration = Duration::from_secs(15);

/// One device's capture: its child (none once it ended or was stopped)
/// and what its threads share.
struct Run {
    shared: Arc<Shared>,
    child: Arc<Mutex<Option<Child>>>,
}

/// Camera and microphone through ffmpeg.
pub struct FfmpegCapture {
    program: PathBuf,
    input: Input,
    first: Duration,
    runs: [Option<Run>; 2],
}

impl Default for FfmpegCapture {
    fn default() -> FfmpegCapture {
        FfmpegCapture::new()
    }
}

impl FfmpegCapture {
    /// ffmpeg, or what `PLEV_FFMPEG` names, for this platform's input.
    /// Runs nothing.
    pub fn new() -> FfmpegCapture {
        let program = std::env::var_os(FFMPEG_ENV)
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("ffmpeg"));
        FfmpegCapture::with(program)
    }

    /// This program (a test's fake).
    pub fn with(program: impl Into<PathBuf>) -> FfmpegCapture {
        FfmpegCapture {
            program: program.into(),
            input: Input::HERE,
            first: FIRST,
            runs: [None, None],
        }
    }

    /// Another platform's input (its arguments, for a test).
    pub fn input(mut self, input: Input) -> FfmpegCapture {
        self.input = input;
        self
    }

    /// How long a device may take to hand anything before it is killed
    /// and `Failed`.
    pub fn first_frame(mut self, deadline: Duration) -> FfmpegCapture {
        self.first = deadline;
        self
    }

    pub fn program(&self) -> &Path {
        &self.program
    }

    /// The pid of the child capturing `kind` now, if one runs.
    pub fn pid(&self, kind: DeviceKind) -> Option<u32> {
        let run = self.runs[slot(kind)].as_ref()?;
        locked(&run.child)
            .as_ref()
            .map(|_| run.shared.pid.load(Ordering::SeqCst))
    }
}

impl Capture for FfmpegCapture {
    fn start(&mut self, kind: DeviceKind, settings: &CaptureSettings) -> Result<(), CaptureError> {
        if self.state(kind).is_live() {
            return Ok(());
        }
        self.stop(kind);
        let args = args(self.input, kind, settings, None)?;
        let shared = Arc::new(Shared::new(kind));
        let (child, ended) = launch(&self.program, &args, settings, &shared)?;
        let child = Arc::new(Mutex::new(Some(child)));
        let watch = Watch {
            program: self.program.clone(),
            input: self.input,
            settings: settings.clone(),
            first: self.first,
            shared: Arc::clone(&shared),
            child: Arc::clone(&child),
        };
        let spawned = thread::Builder::new()
            .name(format!("capture-{}", kind.noun()))
            .spawn(move || watch.run(ended));
        let run = Run { shared, child };
        if spawned.is_err() {
            stop_run(&run);
            return Err(CaptureError::Io(
                "could not start a thread to watch ffmpeg".to_owned(),
            ));
        }
        self.runs[slot(kind)] = Some(run);
        Ok(())
    }

    fn stop(&mut self, kind: DeviceKind) {
        if let Some(run) = self.runs[slot(kind)].take() {
            stop_run(&run);
        }
    }

    fn state(&self, kind: DeviceKind) -> CaptureState {
        self.runs[slot(kind)]
            .as_ref()
            .map(|run| locked(&run.shared.state).clone())
            .unwrap_or(CaptureState::Off)
    }

    fn latest_frame(&mut self) -> Option<Frame> {
        let run = self.runs[slot(DeviceKind::Camera)].as_ref()?;
        locked(&run.shared.frame).take()
    }

    fn take_audio(&mut self) -> Vec<AudioChunk> {
        self.runs[slot(DeviceKind::Microphone)]
            .as_ref()
            .map(|run| locked(&run.shared.audio).drain(..).collect())
            .unwrap_or_default()
    }
}

/// A capture that goes away releases its devices.
impl Drop for FfmpegCapture {
    fn drop(&mut self) {
        for kind in DeviceKind::ALL {
            self.stop(kind);
        }
    }
}

/// Stops a run: the threads told, the child killed by its pid and
/// reaped, what nobody took dropped.
fn stop_run(run: &Run) {
    let mut child = locked(&run.child);
    run.shared.stopping.store(true, Ordering::SeqCst);
    if let Some(mut c) = child.take() {
        let _ = c.kill();
        let _ = c.wait();
        log::info!(
            "capture: {} stopped, pid {} killed",
            run.shared.kind.noun(),
            c.id()
        );
    }
    drop(child);
    locked(&run.shared.frame).take();
    locked(&run.shared.audio).clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn building_the_capture_runs_nothing() {
        let capture = FfmpegCapture::with("/nonexistent/ffmpeg")
            .input(Input::Video4Linux)
            .first_frame(Duration::from_secs(3));
        assert_eq!(capture.program(), Path::new("/nonexistent/ffmpeg"));
        assert_eq!(capture.first, Duration::from_secs(3));
        assert_eq!(capture.pid(DeviceKind::Camera), None);
        assert_eq!(capture.state(DeviceKind::Camera), CaptureState::Off);
        let mut missing = capture;
        let err = missing
            .start(DeviceKind::Camera, &CaptureSettings::default())
            .unwrap_err();
        assert!(
            matches!(&err, CaptureError::Io(why) if why.contains("could not start")),
            "{err}"
        );
        assert_eq!(missing.state(DeviceKind::Camera), CaptureState::Off);
        if cfg!(target_os = "macos") {
            assert_eq!(Input::HERE, Input::AvFoundation);
        }
    }
}
