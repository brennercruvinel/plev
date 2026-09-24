//! The capture probe (task-76 item 10), before any capture code: what a
//! camera and a microphone would hand the engine through ffmpeg's pipe,
//! and what it costs to put a frame on screen.
//!
//! No device is opened here, ever: ffmpeg reads `-f lavfi` sources only
//! (`testsrc` for the camera, `sine` for the microphone), paced by `-re`
//! as a device paces itself, and writes raw frames (`-f rawvideo -pix_fmt
//! rgba`) and raw samples (`-f f32le -ac 1`) to its stdout. Measured:
//! the bytes of a frame, the frames per second that arrive, the latency
//! of the first frame and of the first samples, the size of a pipe read,
//! the level of a block. Then a captured frame into the image atlas
//! (`load_image_rgba`) and drawn through the window's frame path without
//! a window (`engine::headless`), timed; and what the atlas does with a
//! stream of frames: every new frame is a new handle and a new upload
//! kept whole, since the store is grow-only (`gpu/image.rs`), so the
//! stream grows the memory and ends in `TooLarge`.
//!
//! The probe skips without ffmpeg (`PLEV_FFMPEG` or the PATH) and the GPU
//! test skips without an adapter. `--nocapture` prints the measures; the
//! run to `TooLarge` is `--ignored` (it holds a quarter of a gigabyte).

use std::io::Read;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use engine::compositor::Compositor;
use engine::gpu::image::{ImageError, ImageStore};
use engine::gpu::{RenderConfig, load_image_rgba};
use engine::headless::{HeadlessError, Renderer, Viewport};

const WIDTH: u32 = 640;
const HEIGHT: u32 = 360;
const FPS: u32 = 15;
const RATE: u32 = 16_000;
const FRAME_BYTES: usize = (WIDTH * HEIGHT * 4) as usize;

/// `PLEV_FFMPEG`, or `ffmpeg` in a directory of the PATH.
fn ffmpeg() -> Option<PathBuf> {
    if let Some(named) = std::env::var_os("PLEV_FFMPEG") {
        return Some(PathBuf::from(named));
    }
    let dirs = std::env::var_os("PATH").unwrap_or_default();
    let found = std::env::split_paths(&dirs)
        .map(|dir| dir.join("ffmpeg"))
        .find(|p| p.is_file());
    if found.is_none() {
        eprintln!("no ffmpeg; skipping");
    }
    found
}

/// ffmpeg reading one lavfi source at its own pace, raw to the pipe.
fn spawn(program: &PathBuf, source: &str, output: &[&str]) -> Child {
    Command::new(program)
        .args([
            "-hide_banner",
            "-nostdin",
            "-v",
            "error",
            "-re",
            "-f",
            "lavfi",
        ])
        .args(["-i", source])
        .args(output)
        .arg("pipe:1")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("ffmpeg starts")
}

fn kill(mut child: Child) {
    let _ = child.kill();
    let _ = child.wait();
}

/// Frames as they arrive: the first one's latency from the spawn, then
/// the rate over the frames after the first half second (`-re` lets the
/// first half second through at once, as `-readrate_initial_burst`
/// says).
#[test]
fn a_camera_frame_is_a_fixed_block_of_rgba_at_the_devices_pace() {
    let Some(program) = ffmpeg() else {
        return;
    };
    let source = format!("testsrc=size={WIDTH}x{HEIGHT}:rate={FPS}");
    let started = Instant::now();
    let mut child = spawn(&program, &source, &["-f", "rawvideo", "-pix_fmt", "rgba"]);
    let mut out = child.stdout.take().expect("a pipe");
    let mut frame = vec![0u8; FRAME_BYTES];
    let mut stamps = Vec::new();
    while started.elapsed() < Duration::from_millis(2500) {
        out.read_exact(&mut frame).expect("a whole frame");
        stamps.push(started.elapsed());
    }
    kill(child);
    let first = stamps[0];
    let steady: Vec<_> = stamps.iter().filter(|t| t.as_millis() >= 700).collect();
    let span = steady[steady.len() - 1].as_secs_f64() - steady[0].as_secs_f64();
    let fps = (steady.len() - 1) as f64 / span;
    eprintln!(
        "camera {WIDTH}x{HEIGHT} at {FPS}: {FRAME_BYTES} bytes a frame, first frame {:.1} ms, \
         {} frames in 2.5 s, {fps:.2} fps after the burst",
        first.as_secs_f64() * 1000.0,
        stamps.len()
    );
    assert_eq!(frame.len(), 921_600);
    assert!(first < Duration::from_secs(2), "{first:?}");
    assert!((fps - FPS as f64).abs() < 1.5, "{fps}");
    // testsrc's top-left is its first color bar, opaque
    assert_eq!(frame[3], 255);
}

/// Samples as they arrive: the latency of the first bytes, the size of a
/// read, the rate after the burst, and the level of a block (lavfi's sine
/// has an amplitude of one eighth: an RMS of 0.0884).
#[test]
fn a_microphone_is_a_stream_of_mono_f32_samples() {
    let Some(program) = ffmpeg() else {
        return;
    };
    let source = format!("sine=frequency=440:sample_rate={RATE}");
    let rate = RATE.to_string();
    let started = Instant::now();
    let mut child = spawn(
        &program,
        &source,
        &["-f", "f32le", "-ac", "1", "-ar", &rate],
    );
    let mut out = child.stdout.take().expect("a pipe");
    let mut chunk = [0u8; 65_536];
    let mut first = None;
    let mut reads = Vec::new();
    let mut bytes = Vec::new();
    let mut late = (0usize, None::<Duration>, Duration::ZERO);
    while started.elapsed() < Duration::from_millis(2500) {
        let n = out.read(&mut chunk).expect("the pipe reads");
        assert!(n > 0, "the pipe ended");
        let now = started.elapsed();
        first.get_or_insert(now);
        reads.push(n);
        bytes.extend_from_slice(&chunk[..n]);
        if now.as_millis() >= 700 {
            late.0 += n;
            late.1.get_or_insert(now);
            late.2 = now;
        }
    }
    kill(child);
    let samples: Vec<f32> = bytes
        .chunks_exact(4)
        .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .collect();
    let block = &samples[..(RATE / 10) as usize];
    let rms = (block.iter().map(|s| s * s).sum::<f32>() / block.len() as f32).sqrt();
    let span = (late.2 - late.1.unwrap_or_default()).as_secs_f64();
    let steady = late.0 as f64 / 4.0 / span;
    let mut sorted = reads.clone();
    sorted.sort_unstable();
    eprintln!(
        "microphone {RATE} Hz mono f32: first bytes {:.1} ms, {} reads of {} to {} bytes \
         (median {}), about {steady:.0} samples/s after the burst, RMS of 100 ms {rms:.4}",
        first.unwrap_or_default().as_secs_f64() * 1000.0,
        reads.len(),
        sorted[0],
        sorted[sorted.len() - 1],
        sorted[sorted.len() / 2]
    );
    assert!((rms - 0.0884).abs() < 0.002, "{rms}");
    assert!(
        (steady - RATE as f64).abs() < RATE as f64 * 0.15,
        "{steady}"
    );
}

/// A frame, as the pipe gives it, into the atlas and on screen: the
/// cost of the load and of a frame drawn and read back; and ten frames
/// in a row, each a new handle.
#[test]
fn a_captured_frame_drawn_costs_a_load_and_a_frame_and_each_is_a_new_handle() {
    let Some(program) = ffmpeg() else {
        return;
    };
    let source = format!("testsrc=size={WIDTH}x{HEIGHT}:rate={FPS}");
    let mut child = Command::new(&program)
        .args(["-hide_banner", "-nostdin", "-v", "error", "-f", "lavfi"])
        .args(["-i", &source])
        .args([
            "-frames:v",
            "11",
            "-f",
            "rawvideo",
            "-pix_fmt",
            "rgba",
            "pipe:1",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("ffmpeg starts");
    let mut out = child.stdout.take().expect("a pipe");
    let frames: Vec<Vec<u8>> = (0..11)
        .map(|_| {
            let mut f = vec![0u8; FRAME_BYTES];
            out.read_exact(&mut f).expect("a whole frame");
            f
        })
        .collect();
    kill(child);

    let viewport = Viewport::new(WIDTH as f32, HEIGHT as f32, 1.0);
    let mut renderer = match Renderer::new(viewport, RenderConfig::default()) {
        Ok(r) => r,
        Err(HeadlessError::NoAdapter) => {
            eprintln!("no GPU adapter; skipping");
            return;
        }
        Err(e) => panic!("{e}"),
    };
    let mut ids = Vec::new();
    let mut loads = Vec::new();
    let mut draws = Vec::new();
    let mut last = None;
    for pixels in &frames {
        let t = Instant::now();
        let handle = load_image_rgba(WIDTH, HEIGHT, pixels.clone()).expect("the frame packs");
        loads.push(t.elapsed());
        ids.push(handle.id);
        let mut c = Compositor::new();
        c.begin_frame();
        c.draw_image(0.0, 0.0, WIDTH as f32, HEIGHT as f32, handle, 0.0);
        let t = Instant::now();
        let frame = renderer
            .render(&mut c, [0.0, 0.0, 0.0, 1.0])
            .expect("reads");
        draws.push(t.elapsed());
        last = Some((frame, pixels));
    }
    let ms = |d: &Duration| d.as_secs_f64() * 1000.0;
    eprintln!(
        "load_image_rgba {WIDTH}x{HEIGHT}: {:.2} ms first, {:.2} to {:.2} ms after; \
         draw and read back: {:.1} ms first (device warm-up), {:.1} to {:.1} ms after; handles {:?}",
        ms(&loads[0]),
        loads[1..].iter().map(ms).fold(f64::MAX, f64::min),
        loads[1..].iter().map(ms).fold(0.0, f64::max),
        ms(&draws[0]),
        draws[1..].iter().map(ms).fold(f64::MAX, f64::min),
        draws[1..].iter().map(ms).fold(0.0, f64::max),
        ids
    );
    // testsrc moves every frame: no two frames share a handle
    let mut unique = ids.clone();
    unique.dedup();
    assert_eq!(unique.len(), ids.len(), "{ids:?}");
    // the drawn frame is the captured one where the bars are flat
    let (frame, pixels) = last.expect("a frame");
    let (x, y) = (40u32, 40u32);
    let at = ((y * WIDTH + x) * 4) as usize;
    let drawn = frame.pixel(x, y).expect("inside");
    for k in 0..3 {
        assert!(
            (drawn[k] as i32 - pixels[at + k] as i32).abs() <= 2,
            "{drawn:?} vs {:?}",
            &pixels[at..at + 4]
        );
    }
}

/// A stream of frames into a store of its own, as a live camera would
/// load them: each frame a new upload with its pixels kept, the atlas
/// doubling, until the store refuses. Prints the count, the bytes held
/// and the seconds of a 15 fps camera that is.
#[test]
#[ignore = "holds a quarter of a gigabyte; the probe's number, run once"]
fn a_stream_of_frames_fills_the_grow_only_atlas() {
    let mut store = ImageStore::new();
    let mut sizes = vec![(0usize, store.atlas_size())];
    let mut n = 0usize;
    let refused = loop {
        // a frame that differs from every other one in its first pixel
        let mut pixels = vec![128u8; FRAME_BYTES];
        pixels[..8].copy_from_slice(&(n as u64).to_le_bytes());
        match store.load_rgba(WIDTH, HEIGHT, pixels) {
            Ok(_) => n += 1,
            Err(e) => break e,
        }
        if store.atlas_size() != sizes[sizes.len() - 1].1 {
            sizes.push((n, store.atlas_size()));
        }
    };
    let held = n * FRAME_BYTES;
    eprintln!(
        "{n} frames of {WIDTH}x{HEIGHT} before {refused}; {} uploads, {:.0} MB held by the log; \
         atlas sizes by frame count {sizes:?}; {:.1} s of a {FPS} fps camera",
        store.upload_count(),
        held as f64 / 1_000_000.0,
        n as f64 / FPS as f64
    );
    assert!(matches!(refused, ImageError::TooLarge { .. }));
    assert_eq!(store.upload_count(), n);
}
