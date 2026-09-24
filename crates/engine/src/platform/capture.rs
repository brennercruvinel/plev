//! Camera and microphone as capabilities (task-76 item 10): capture
//! behind a trait, so an app shows a camera's frames and a microphone's
//! level and a test never opens a device.
//!
//! A [`Capture`] starts and stops each kind of device, never blocking the
//! app's loop: `start` returns at once, the device answers later, and
//! what it came to is a [`CaptureState`] (`Starting`, `On`, `Denied`,
//! `NoDevice`, `Failed`), projected by the app instead of an error. The
//! camera hands [`Frame`]s of a fixed size in RGBA, newest wins: a frame
//! nobody took is replaced by the next, so the memory holds one frame
//! whatever the loop does. The microphone hands [`AudioChunk`]s of mono
//! f32 samples into a bounded ring: the oldest go first when nobody
//! takes them.
//!
//! [`FfmpegCapture`] is the desktop one: ffmpeg as a child process per
//! device, reading the platform's input (`avfoundation` on macOS,
//! `dshow` on windows, `v4l2` and `alsa` on linux) and writing raw frames
//! and samples to its stdout (`ffmpeg.rs`), killed by its pid when the
//! capture stops, the engine taking no dependency for capture.
//! [`FakeCapture`] makes synthetic frames and a sine without a process;
//! [`NoCapture`] is wasm and mobile: `Unsupported`. [`default_capture`]
//! chooses.
//!
//! Capture is a person's: an app that offers it declares turning a
//! device on for a person alone, and an agent reads the state the app
//! projects. The system's permission is the platform's (on macOS the
//! prompt belongs to the process that launched the app); a refusal is
//! the state `Denied`, never a failure of the app.

use std::fmt;
use std::sync::{Arc, Mutex};

use web_time::Instant;

use super::menu::locked;

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
mod ffmpeg;
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
pub use ffmpeg::{FFMPEG_ENV, FfmpegCapture, Input};

/// A camera or a microphone.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DeviceKind {
    Camera,
    Microphone,
}

impl DeviceKind {
    pub const ALL: [DeviceKind; 2] = [DeviceKind::Camera, DeviceKind::Microphone];

    /// "camera", "microphone".
    pub fn noun(self) -> &'static str {
        match self {
            DeviceKind::Camera => "camera",
            DeviceKind::Microphone => "microphone",
        }
    }
}

/// The device a capture opens: by its index in the platform's list, or
/// by its name when it has one (windows opens a device by name only).
/// The first device of each kind, index 0 and no name, is the default.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Device {
    pub kind: DeviceKind,
    pub name: String,
    pub index: u32,
}

impl Device {
    /// The first device of `kind`.
    pub fn first(kind: DeviceKind) -> Device {
        Device {
            kind,
            name: String::new(),
            index: 0,
        }
    }

    /// The device at `index` of the platform's list.
    pub fn at(kind: DeviceKind, index: u32) -> Device {
        Device {
            kind,
            name: String::new(),
            index,
        }
    }

    /// The device named `name`.
    pub fn named(kind: DeviceKind, name: impl Into<String>) -> Device {
        Device {
            kind,
            name: name.into(),
            index: 0,
        }
    }
}

/// What a capture asks for: the camera's frame size and rate (frames are
/// scaled and cropped to the size, whatever the device gives), the
/// microphone's sample rate, and which device of each kind.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaptureSettings {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub rate: u32,
    pub camera: Device,
    pub microphone: Device,
}

impl Default for CaptureSettings {
    /// 640x360 at 15 frames a second, 16 kHz mono, the first device of
    /// each kind.
    fn default() -> CaptureSettings {
        CaptureSettings {
            width: 640,
            height: 360,
            fps: 15,
            rate: 16_000,
            camera: Device::first(DeviceKind::Camera),
            microphone: Device::first(DeviceKind::Microphone),
        }
    }
}

impl CaptureSettings {
    /// The device of `kind` these settings open.
    pub fn device(&self, kind: DeviceKind) -> &Device {
        match kind {
            DeviceKind::Camera => &self.camera,
            DeviceKind::Microphone => &self.microphone,
        }
    }

    /// The bytes of one RGBA frame.
    pub fn frame_bytes(&self) -> usize {
        self.width as usize * self.height as usize * 4
    }

    /// The samples of one chunk: a twentieth of a second.
    pub fn chunk_samples(&self) -> usize {
        (self.rate as usize / 20).max(1)
    }

    /// The samples the ring holds at most: two seconds.
    pub fn ring_samples(&self) -> usize {
        self.rate as usize * 2
    }
}

/// One camera frame: `width x height` pixels of tightly packed RGBA, and
/// when it arrived.
#[derive(Clone, PartialEq)]
pub struct Frame {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    pub at: Instant,
}

impl fmt::Debug for Frame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Frame")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("rgba_bytes", &self.rgba.len())
            .finish()
    }
}

/// A block of microphone samples, mono, in `[-1, 1]`, at `rate` a second,
/// and when it arrived.
#[derive(Clone, PartialEq)]
pub struct AudioChunk {
    pub samples: Vec<f32>,
    pub rate: u32,
    pub at: Instant,
}

impl fmt::Debug for AudioChunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AudioChunk")
            .field("samples", &self.samples.len())
            .field("rate", &self.rate)
            .finish()
    }
}

impl AudioChunk {
    /// The level of the block: the root of the mean square, 0 for none.
    pub fn rms(&self) -> f32 {
        rms(&self.samples)
    }
}

/// The root of the mean square of `samples`, 0 for none.
pub fn rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    (samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32).sqrt()
}

/// Where a device is: off; asked, with nothing yet; handing frames or
/// samples; refused by the system's permission; not there; or failed, in
/// the words of whatever failed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CaptureState {
    Off,
    Starting,
    On,
    Denied,
    NoDevice,
    Failed(String),
}

impl CaptureState {
    /// Asked or running: what a person turned on and has not turned off.
    pub fn is_live(&self) -> bool {
        matches!(self, CaptureState::Starting | CaptureState::On)
    }
}

/// Why a capture did not start.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CaptureError {
    /// This target has no capture (wasm, android, ios).
    Unsupported,
    /// The device cannot be named for this platform's input (windows
    /// opens a device by name).
    NoDevice(String),
    /// The program could not be started.
    Io(String),
}

impl fmt::Display for CaptureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CaptureError::Unsupported => {
                f.write_str("capture: no camera or microphone on this target")
            }
            CaptureError::NoDevice(why) => write!(f, "capture: {why}"),
            CaptureError::Io(why) => write!(f, "capture: {why}"),
        }
    }
}

impl std::error::Error for CaptureError {}

/// The system's cameras and microphones. `start` asks for a device and
/// returns at once; `state` says what it came to; `latest_frame` takes
/// the newest frame (the one before it is gone), `take_audio` the chunks
/// that arrived since the last take; `stop` releases the device and
/// drops what it had handed and nobody took. No call blocks: the app
/// calls from its loop.
pub trait Capture {
    /// Starts the device of `kind` with `settings`; a device already live
    /// keeps going.
    fn start(&mut self, kind: DeviceKind, settings: &CaptureSettings) -> Result<(), CaptureError>;
    /// Stops the device of `kind` and releases it; `Off` after.
    fn stop(&mut self, kind: DeviceKind);
    fn state(&self, kind: DeviceKind) -> CaptureState;
    /// The newest camera frame not taken yet.
    fn latest_frame(&mut self) -> Option<Frame>;
    /// The microphone's chunks not taken yet, oldest first.
    fn take_audio(&mut self) -> Vec<AudioChunk>;
}

// -- the fake ---------------------------------------------------------------

/// The capture of a test: frames and samples made on demand, no process.
/// A started camera is `On` at once and hands a new frame at every
/// `latest_frame`: frame `n` is filled with [`FakeCapture::pixel`]`(n)`.
/// A started microphone hands one chunk of a 440 Hz sine of amplitude
/// one eighth at every `take_audio`. What a start should come to instead
/// (`Denied`, `NoDevice`, `Failed`) is lined up per kind.
#[derive(Debug, Default)]
pub struct FakeCapture {
    states: [Option<CaptureState>; 2],
    lined_up: [Option<CaptureState>; 2],
    settings: Option<CaptureSettings>,
    frames: u64,
    samples: u64,
    starts: [u32; 2],
    stops: [u32; 2],
}

fn slot(kind: DeviceKind) -> usize {
    match kind {
        DeviceKind::Camera => 0,
        DeviceKind::Microphone => 1,
    }
}

impl FakeCapture {
    pub fn new() -> FakeCapture {
        FakeCapture::default()
    }

    /// The next start of `kind` comes to `state` (`Denied`, `NoDevice`,
    /// `Failed`) instead of `On`.
    pub fn will_come_to(mut self, kind: DeviceKind, state: CaptureState) -> FakeCapture {
        self.lined_up[slot(kind)] = Some(state);
        self
    }

    /// The color of the fake's frame `n`: every pixel of it.
    pub fn pixel(n: u64) -> [u8; 4] {
        let v = (n % 200) as u8;
        [40 + v, 200 - v, 120, 255]
    }

    /// The level of the fake's chunks: a sine of amplitude one eighth.
    pub const RMS: f32 = 0.125 * std::f32::consts::FRAC_1_SQRT_2;

    /// How many times `kind` was started and stopped.
    pub fn starts(&self, kind: DeviceKind) -> u32 {
        self.starts[slot(kind)]
    }

    pub fn stops(&self, kind: DeviceKind) -> u32 {
        self.stops[slot(kind)]
    }

    /// Frames handed so far.
    pub fn frames(&self) -> u64 {
        self.frames
    }
}

impl Capture for FakeCapture {
    fn start(&mut self, kind: DeviceKind, settings: &CaptureSettings) -> Result<(), CaptureError> {
        let i = slot(kind);
        if self.states[i].as_ref().is_some_and(CaptureState::is_live) {
            return Ok(());
        }
        self.starts[i] += 1;
        self.settings = Some(settings.clone());
        self.states[i] = Some(self.lined_up[i].take().unwrap_or(CaptureState::On));
        Ok(())
    }

    fn stop(&mut self, kind: DeviceKind) {
        let i = slot(kind);
        if self.states[i].take().is_some() {
            self.stops[i] += 1;
        }
    }

    fn state(&self, kind: DeviceKind) -> CaptureState {
        self.states[slot(kind)].clone().unwrap_or(CaptureState::Off)
    }

    fn latest_frame(&mut self) -> Option<Frame> {
        if self.state(DeviceKind::Camera) != CaptureState::On {
            return None;
        }
        let settings = self.settings.clone().unwrap_or_default();
        let pixel = FakeCapture::pixel(self.frames);
        self.frames += 1;
        Some(Frame {
            width: settings.width,
            height: settings.height,
            rgba: pixel.repeat(settings.width as usize * settings.height as usize),
            at: Instant::now(),
        })
    }

    fn take_audio(&mut self) -> Vec<AudioChunk> {
        if self.state(DeviceKind::Microphone) != CaptureState::On {
            return Vec::new();
        }
        let settings = self.settings.clone().unwrap_or_default();
        let n = settings.chunk_samples();
        let rate = settings.rate as f32;
        let start = self.samples;
        self.samples += n as u64;
        let samples = (0..n as u64)
            .map(|k| {
                let t = (start + k) as f32 / rate;
                0.125 * (std::f32::consts::TAU * 440.0 * t).sin()
            })
            .collect();
        vec![AudioChunk {
            samples,
            rate: settings.rate,
            at: Instant::now(),
        }]
    }
}

// -- no capture at all ------------------------------------------------------

/// The capture of a target without one: wasm (the browser's camera is
/// another capability) and mobile until its bring-up. Every start is
/// `Unsupported`; nothing is ever on.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoCapture;

impl Capture for NoCapture {
    fn start(
        &mut self,
        _kind: DeviceKind,
        _settings: &CaptureSettings,
    ) -> Result<(), CaptureError> {
        Err(CaptureError::Unsupported)
    }

    fn stop(&mut self, _kind: DeviceKind) {}

    fn state(&self, _kind: DeviceKind) -> CaptureState {
        CaptureState::Off
    }

    fn latest_frame(&mut self) -> Option<Frame> {
        None
    }

    fn take_audio(&mut self) -> Vec<AudioChunk> {
        Vec::new()
    }
}

/// A capture shared by handle: the app holds one `Arc`, a test the
/// other. A poisoned lock still answers: the capture holds no invariant a
/// panic elsewhere could break.
impl<C: Capture> Capture for Arc<Mutex<C>> {
    fn start(&mut self, kind: DeviceKind, settings: &CaptureSettings) -> Result<(), CaptureError> {
        locked(self).start(kind, settings)
    }

    fn stop(&mut self, kind: DeviceKind) {
        locked(self).stop(kind)
    }

    fn state(&self, kind: DeviceKind) -> CaptureState {
        locked(self).state(kind)
    }

    fn latest_frame(&mut self) -> Option<Frame> {
        locked(self).latest_frame()
    }

    fn take_audio(&mut self) -> Vec<AudioChunk> {
        locked(self).take_audio()
    }
}

impl<C: Capture + ?Sized> Capture for Box<C> {
    fn start(&mut self, kind: DeviceKind, settings: &CaptureSettings) -> Result<(), CaptureError> {
        (**self).start(kind, settings)
    }

    fn stop(&mut self, kind: DeviceKind) {
        (**self).stop(kind)
    }

    fn state(&self, kind: DeviceKind) -> CaptureState {
        (**self).state(kind)
    }

    fn latest_frame(&mut self) -> Option<Frame> {
        (**self).latest_frame()
    }

    fn take_audio(&mut self) -> Vec<AudioChunk> {
        (**self).take_audio()
    }
}

/// The capture of the current target: ffmpeg as a child on the desktop
/// (`PLEV_FFMPEG` or the one on the PATH), none elsewhere. Building it
/// runs nothing and opens nothing; the first `start` does.
pub fn default_capture() -> Box<dyn Capture + Send> {
    #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
    {
        Box::new(FfmpegCapture::new())
    }
    #[cfg(any(target_arch = "wasm32", target_os = "android", target_os = "ios"))]
    {
        Box::new(NoCapture)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_settings_default_to_a_small_camera_and_a_voice_rate() {
        let s = CaptureSettings::default();
        assert_eq!((s.width, s.height, s.fps, s.rate), (640, 360, 15, 16_000));
        assert_eq!(s.frame_bytes(), 921_600);
        assert_eq!(s.chunk_samples(), 800);
        assert_eq!(s.ring_samples(), 32_000);
        assert_eq!(
            s.device(DeviceKind::Camera),
            &Device::first(DeviceKind::Camera)
        );
        assert_eq!(
            s.device(DeviceKind::Microphone),
            &Device::at(DeviceKind::Microphone, 0)
        );
        assert_eq!(Device::named(DeviceKind::Camera, "Front").name, "Front");
        assert_eq!(DeviceKind::Microphone.noun(), "microphone");
    }

    #[test]
    fn the_fake_hands_a_new_frame_and_a_sine_while_on_and_nothing_off() {
        let mut fake = FakeCapture::new();
        let settings = CaptureSettings {
            width: 4,
            height: 2,
            ..CaptureSettings::default()
        };
        assert_eq!(fake.state(DeviceKind::Camera), CaptureState::Off);
        assert!(fake.latest_frame().is_none());
        assert!(fake.take_audio().is_empty());

        fake.start(DeviceKind::Camera, &settings).unwrap();
        fake.start(DeviceKind::Camera, &settings).unwrap();
        assert_eq!(
            fake.starts(DeviceKind::Camera),
            1,
            "a live camera keeps going"
        );
        assert_eq!(fake.state(DeviceKind::Camera), CaptureState::On);
        let a = fake.latest_frame().unwrap();
        let b = fake.latest_frame().unwrap();
        assert_eq!((a.width, a.height, a.rgba.len()), (4, 2, 32));
        assert_eq!(&a.rgba[..4], FakeCapture::pixel(0));
        assert_eq!(&b.rgba[..4], FakeCapture::pixel(1));
        assert_ne!(a.rgba, b.rgba);
        assert!(fake.take_audio().is_empty(), "the microphone is off");

        fake.start(DeviceKind::Microphone, &settings).unwrap();
        let chunks = fake.take_audio();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].samples.len(), 800);
        assert!(
            (chunks[0].rms() - FakeCapture::RMS).abs() < 0.002,
            "{}",
            chunks[0].rms()
        );

        fake.stop(DeviceKind::Camera);
        fake.stop(DeviceKind::Camera);
        assert_eq!(fake.stops(DeviceKind::Camera), 1);
        assert!(fake.latest_frame().is_none());
        assert_eq!(fake.frames(), 2);
        assert!(format!("{a:?}").contains("rgba_bytes: 32"));
    }

    #[test]
    fn the_fake_comes_to_what_was_lined_up() {
        let mut fake = FakeCapture::new()
            .will_come_to(DeviceKind::Camera, CaptureState::Denied)
            .will_come_to(DeviceKind::Microphone, CaptureState::NoDevice);
        let settings = CaptureSettings::default();
        fake.start(DeviceKind::Camera, &settings).unwrap();
        fake.start(DeviceKind::Microphone, &settings).unwrap();
        assert_eq!(fake.state(DeviceKind::Camera), CaptureState::Denied);
        assert_eq!(fake.state(DeviceKind::Microphone), CaptureState::NoDevice);
        assert!(fake.latest_frame().is_none());
        // a refused device is not live: asking again starts it again
        fake.start(DeviceKind::Camera, &settings).unwrap();
        assert_eq!(fake.state(DeviceKind::Camera), CaptureState::On, "spent");
        assert_eq!(fake.starts(DeviceKind::Camera), 2);
    }

    #[test]
    fn the_level_is_the_root_of_the_mean_square() {
        assert_eq!(rms(&[]), 0.0);
        assert!((rms(&[0.5, -0.5]) - 0.5).abs() < 1e-6);
        let chunk = AudioChunk {
            samples: vec![0.25; 8],
            rate: 8,
            at: Instant::now(),
        };
        assert!((chunk.rms() - 0.25).abs() < 1e-6);
        assert!(format!("{chunk:?}").contains("samples: 8"));
    }

    #[test]
    fn no_capture_is_unsupported_and_never_on() {
        let mut none = NoCapture;
        assert_eq!(
            none.start(DeviceKind::Camera, &CaptureSettings::default()),
            Err(CaptureError::Unsupported)
        );
        none.stop(DeviceKind::Camera);
        assert_eq!(none.state(DeviceKind::Microphone), CaptureState::Off);
        assert!(none.latest_frame().is_none());
        assert!(none.take_audio().is_empty());
        assert_eq!(
            CaptureError::Unsupported.to_string(),
            "capture: no camera or microphone on this target"
        );
        // shared, boxed, built and never started: nothing runs
        let shared = Arc::new(Mutex::new(FakeCapture::new()));
        let mut app: Box<dyn Capture + Send> = Box::new(shared.clone());
        app.start(DeviceKind::Camera, &CaptureSettings::default())
            .unwrap();
        assert_eq!(locked(&shared).starts(DeviceKind::Camera), 1);
        let _capture = default_capture();
    }
}
