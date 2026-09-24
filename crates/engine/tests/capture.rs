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
//! the level of a block. Then captured frames into one stream image
//! (`stream_image_rgba`, `replace_image_rgba`: one slot whose pixels
//! change in place) and drawn through the window's frame path without a
//! window (`engine::headless`), timed, in every context over the store.
//! Loaded as fixed images instead, every frame would be a new handle and
//! a new upload kept whole, since the store is grow-only
//! (`gpu/image.rs`): the stream would grow the memory and end in
//! `TooLarge`, which the ignored test measures.
//!
//! The probe skips without ffmpeg (`PLEV_FFMPEG` or the PATH) and the GPU
//! tests skip without an adapter. `--nocapture` prints the measures; the
//! run to `TooLarge` is `--ignored` (it holds a quarter of a gigabyte).

use std::io::Read;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use engine::compositor::Compositor;
use engine::gpu::image::{ImageError, ImageStore};
use engine::gpu::{ImageHandle, RenderConfig, replace_image_rgba, stream_image_rgba};
use engine::headless::{Frame, HeadlessError, Renderer, Viewport};

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

/// Frames, as the pipe gives them, into one stream image and on screen:
/// the cost of a replacement and of a frame drawn and read back, the
/// same place in the atlas for all of them under a new id each, and the
/// drawn frame the captured one.
#[test]
fn captured_frames_replace_one_stream_image_and_draw_as_captured() {
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

    let Some(mut renderer) = renderer(WIDTH as f32, HEIGHT as f32) else {
        return;
    };
    let slot = stream_image_rgba(WIDTH, HEIGHT).expect("the slot packs");
    let mut handle = slot;
    let mut ids = Vec::new();
    let mut replaces = Vec::new();
    let mut draws = Vec::new();
    let mut last = None;
    for pixels in &frames {
        let t = Instant::now();
        handle = replace_image_rgba(&handle, pixels).expect("the frame replaces");
        replaces.push(t.elapsed());
        ids.push(handle.id);
        assert_eq!(
            (handle.atlas_x, handle.atlas_y),
            (slot.atlas_x, slot.atlas_y)
        );
        let t = Instant::now();
        let frame = draw(&mut renderer, handle, WIDTH as f32, HEIGHT as f32);
        draws.push(t.elapsed());
        last = Some((frame, pixels));
    }
    let ms = |d: &Duration| d.as_secs_f64() * 1000.0;
    eprintln!(
        "replace_image_rgba {WIDTH}x{HEIGHT}: {:.2} to {:.2} ms; \
         draw and read back: {:.1} ms first (device warm-up), {:.1} to {:.1} ms after; ids {:?}",
        replaces.iter().map(ms).fold(f64::MAX, f64::min),
        replaces.iter().map(ms).fold(0.0, f64::max),
        ms(&draws[0]),
        draws[1..].iter().map(ms).fold(f64::MAX, f64::min),
        draws[1..].iter().map(ms).fold(0.0, f64::max),
        ids
    );
    let mut unique = ids.clone();
    unique.dedup();
    assert_eq!(unique.len(), ids.len(), "a new id per frame: {ids:?}");
    // the drawn frame is the last captured one, where the bars are flat
    let (frame, pixels) = last.expect("a frame");
    let (x, y) = (40u32, 40u32);
    let at = ((y * WIDTH + x) * 4) as usize;
    assert!(close(frame.pixel(x, y), &pixels[at..at + 4]));
}

/// A stream image is one slot drawn by every context over the one store
/// (a window each, the headless one): two renderers draw its first
/// frame, the frame is replaced and both draw the second at the same
/// place; a renderer that starts after, and one whose device was lost
/// and recovered, draw the latest, never the first.
#[test]
fn a_stream_image_draws_its_latest_frame_in_every_context() {
    let (Some(mut a), Some(mut b)) = (renderer(16.0, 16.0), renderer(16.0, 16.0)) else {
        return;
    };
    let red = [200u8, 30, 30, 255];
    let green = [30u8, 200, 30, 255];
    let blue = [30u8, 30, 200, 255];
    let slot = stream_image_rgba(8, 8).expect("a slot");
    let first = replace_image_rgba(&slot, &red.repeat(64)).expect("red");
    assert!(close(draw(&mut a, first, 8.0, 8.0).pixel(4, 4), &red));
    assert!(close(draw(&mut b, first, 8.0, 8.0).pixel(4, 4), &red));

    let second = replace_image_rgba(&first, &green.repeat(64)).expect("green");
    assert_eq!(
        (second.atlas_x, second.atlas_y),
        (slot.atlas_x, slot.atlas_y)
    );
    assert!(close(draw(&mut a, second, 8.0, 8.0).pixel(4, 4), &green));
    assert!(close(draw(&mut b, second, 8.0, 8.0).pixel(4, 4), &green));

    // a new context, and a context whose device is lost and recovered
    // (its atlas refilled from the log): both write the latest frame
    let third = replace_image_rgba(&second, &blue.repeat(64)).expect("blue");
    let Some(mut late) = renderer(16.0, 16.0) else {
        return;
    };
    assert!(
        close(draw(&mut late, third, 8.0, 8.0).pixel(4, 4), &blue),
        "a new atlas writes the latest frame"
    );
    a.gpu().device.destroy();
    assert!(
        close(draw(&mut a, third, 8.0, 8.0).pixel(4, 4), &blue),
        "a recovered atlas writes the latest frame"
    );
    assert!(!a.gpu().device_lost(), "recovered in place");
    assert!(close(draw(&mut b, third, 8.0, 8.0).pixel(4, 4), &blue));
}

fn renderer(width: f32, height: f32) -> Option<Renderer> {
    match Renderer::new(Viewport::new(width, height, 1.0), RenderConfig::default()) {
        Ok(r) => Some(r),
        Err(HeadlessError::NoAdapter) => {
            eprintln!("no GPU adapter; skipping");
            None
        }
        Err(e) => panic!("{e}"),
    }
}

/// `image` drawn at the origin at `w x h`, on black, read back.
fn draw(renderer: &mut Renderer, image: ImageHandle, w: f32, h: f32) -> Frame {
    let mut c = Compositor::new();
    c.begin_frame();
    c.draw_image(0.0, 0.0, w, h, image, 0.0);
    renderer
        .render(&mut c, [0.0, 0.0, 0.0, 1.0])
        .expect("the frame reads")
}

/// A drawn pixel against a source one, within the rounding of the sRGB
/// texture's round trip.
fn close(drawn: Option<[u8; 4]>, source: &[u8]) -> bool {
    let Some(drawn) = drawn else {
        return false;
    };
    let ok = (0..3).all(|k| (drawn[k] as i32 - source[k] as i32).abs() <= 2);
    if !ok {
        eprintln!("drawn {drawn:?}, source {source:?}");
    }
    ok
}

/// Why the stream image exists: frames loaded as fixed images into a
/// store of their own, as a live camera would without it. Each frame is a
/// new upload with its pixels kept, the atlas doubles, until the store
/// refuses. Prints the count, the bytes held and the seconds of a 15 fps
/// camera that is (the probe: 244 frames, 225 MB, 16.3 s).
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
