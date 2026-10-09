//! The GPU device is lost and the next frame draws the same (task-76).
//!
//! `Device::destroy` is the loss a test can force: wgpu reports it as
//! `DeviceLostReason::Destroyed` on the next poll and, from then on,
//! every resource of that device is dead. The frame path must notice,
//! rebuild everything on a new device and draw the scene again, with the
//! pixels a frame before the loss had; a loss in the middle of a sequence
//! must not leave a stale frame; and a machine whose GPU does not come
//! back keeps the app alive, with one error in the log.
//!
//! Through `engine::headless`, the window's frame path into a texture. The
//! tests skip without a GPU adapter, like their neighbours.

use std::sync::{Mutex, OnceLock};

use engine::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use engine::drawing::gpu::{ImageHandle, RenderConfig, load_image_rgba};
use engine::drawing::headless::{Frame, HeadlessError, Renderer, Viewport};
use engine::drawing::text::TextStyle;
use engine::styling::theme::hoff::PAGE_BG;
use engine::wgpu;

const IMAGE_RGBA: [u8; 4] = [20, 200, 40, 255];
/// Inside the image, in physical pixels (drawn at logical 180,10 40x40, scale 2).
const IMAGE_AT: (u32, u32) = (400, 60);
/// Inside the flat rect that changes color per frame (logical 10,10 20x20).
const RECT_AT: (u32, u32) = (30, 30);

fn clear() -> [f32; 4] {
    PAGE_BG.to_linear_array()
}

fn renderer() -> Option<Renderer> {
    match Renderer::new(Viewport::new(240.0, 140.0, 2.0), RenderConfig::default()) {
        Ok(r) => Some(r),
        Err(HeadlessError::NoAdapter) => {
            eprintln!("no GPU adapter; skipping");
            None
        }
        Err(e) => panic!("headless renderer: {e}"),
    }
}

fn image() -> ImageHandle {
    load_image_rgba(8, 8, IMAGE_RGBA.repeat(8 * 8)).expect("an 8x8 image packs")
}

/// One pure color per frame, so a frame's rect names the frame.
fn rect_color(frame: u8) -> [f32; 4] {
    match frame % 4 {
        0 => [1.0, 0.0, 0.0, 1.0],
        1 => [0.0, 1.0, 0.0, 1.0],
        2 => [0.0, 0.0, 1.0, 1.0],
        _ => [1.0, 1.0, 0.0, 1.0],
    }
}

fn rect_pixel(frame: u8) -> [u8; 4] {
    let c = rect_color(frame);
    [c[0] as u8 * 255, c[1] as u8 * 255, c[2] as u8 * 255, 255]
}

/// Text, a rounded rect with a border and an image on `retained`, one of
/// each resource kind that lives on the device; the frame's flat rect on
/// the default layer.
fn scene(c: &mut Compositor, retained: LayerId, image: ImageHandle, frame: u8) {
    c.begin_frame();
    c.draw_rect(10.0, 10.0, 20.0, 20.0, rect_color(frame));
    c.push_to_layer(
        retained,
        SceneNode::RoundedRect {
            x: 40.0,
            y: 40.0,
            w: 120.0,
            h: 60.0,
            color: [0.2, 0.4, 0.9, 1.0],
            corner_radius: 12.0,
            border_width: 1.0,
            border_color: [1.0, 1.0, 1.0, 0.3],
        },
    );
    c.push_to_layer(
        retained,
        SceneNode::Text {
            key: TextNodeKey::from_style(
                "device loss",
                &TextStyle::new(16.0).with_line_height(22.4),
                None,
            ),
            x: 50.0,
            y: 60.0,
            color: [1.0, 1.0, 1.0, 1.0],
        },
    );
    c.push_to_layer(
        retained,
        SceneNode::Image {
            x: 180.0,
            y: 10.0,
            w: 40.0,
            h: 40.0,
            image,
            corner_radius: 0.0,
        },
    );
}

/// Pixels that are not the page background.
fn ink(frame: &Frame) -> usize {
    frame
        .rgba
        .chunks_exact(4)
        .filter(|p| p[..3] != [48, 48, 48])
        .count()
}

fn assert_same(got: &Frame, want: &Frame, what: &str) {
    assert_eq!(
        (got.width, got.height),
        (want.width, want.height),
        "{what}: size"
    );
    let differing = got
        .rgba
        .chunks_exact(4)
        .zip(want.rgba.chunks_exact(4))
        .enumerate()
        .filter(|(_, (a, b))| a != b)
        .map(|(i, _)| (i as u32 % got.width, i as u32 / got.width))
        .collect::<Vec<_>>();
    assert!(
        differing.is_empty(),
        "{what}: {} pixels differ, the first at {:?}",
        differing.len(),
        differing[0]
    );
}

fn render(r: &mut Renderer, c: &mut Compositor, what: &str) -> Frame {
    match r.render(c, clear()) {
        Ok(f) => f,
        Err(e) => panic!("{what}: {e}"),
    }
}

/// Frame A, the device destroyed, frame B: the same pixels, text, rect
/// and image included, and the context is not lost any more.
#[test]
fn the_frame_after_a_forced_loss_matches_the_frame_before() {
    let Some(mut r) = renderer() else {
        return;
    };
    let image = image();
    let mut c = Compositor::new();
    let retained = c.create_layer(5);

    scene(&mut c, retained, image, 0);
    let a = render(&mut r, &mut c, "frame A");
    // what the loss has to bring back: glyphs, geometry and the image
    assert!(ink(&a) > 1000, "frame A drew {} pixels", ink(&a));
    assert_eq!(a.pixel(IMAGE_AT.0, IMAGE_AT.1), Some(IMAGE_RGBA));
    assert_eq!(a.pixel(RECT_AT.0, RECT_AT.1), Some(rect_pixel(0)));

    r.gpu().device.destroy();
    assert!(r.gpu().poll_device_loss(), "the poll delivers the loss");
    let lost = r.gpu().lost_device().expect("the reason is kept");
    assert_eq!(lost.reason, wgpu::DeviceLostReason::Destroyed);

    scene(&mut c, retained, image, 0);
    let b = render(&mut r, &mut c, "frame B");
    assert!(!r.gpu().device_lost(), "recovered in place");
    assert_eq!(r.gpu().lost_device(), None);
    assert_same(&b, &a, "frame B against frame A");

    // and the renderer goes on: a third frame, still the same
    scene(&mut c, retained, image, 0);
    let c3 = render(&mut r, &mut c, "frame C");
    assert_same(&c3, &a, "frame C against frame A");
}

/// A sequence where the default layer changes every frame and another
/// layer is retained (drawn once, never redrawn while clean). The device
/// dies between frames 1 and 2 with no poll in between: frame 2 must
/// notice on its own, show its own rect and the retained layer's content,
/// and match a renderer that never lost its device.
#[test]
fn a_loss_in_the_middle_of_a_sequence_leaves_no_stale_frame() {
    let Some(mut r) = renderer() else {
        return;
    };
    let Some(mut control) = renderer() else {
        return;
    };
    let image = image();
    let mut c = Compositor::new();
    let mut cc = Compositor::new();
    let retained = c.create_layer(5);
    let retained_c = cc.create_layer(5);

    for frame in 0..4u8 {
        if frame == 2 {
            r.gpu().device.destroy();
        }
        scene(&mut c, retained, image, frame);
        scene(&mut cc, retained_c, image, frame);
        let got = render(&mut r, &mut c, &format!("frame {frame}"));
        let want = render(&mut control, &mut cc, &format!("control frame {frame}"));
        assert_eq!(
            got.pixel(RECT_AT.0, RECT_AT.1),
            Some(rect_pixel(frame)),
            "frame {frame} shows its own rect, not the previous frame's"
        );
        assert_eq!(
            got.pixel(IMAGE_AT.0, IMAGE_AT.1),
            Some(IMAGE_RGBA),
            "frame {frame} shows the retained image"
        );
        assert_same(&got, &want, &format!("frame {frame} against the control"));
    }
    assert!(!r.gpu().device_lost());
}

/// The log lines the frame path emits, to count the error of a failed
/// recovery: once per loss, not once per frame.
struct Recorder(Mutex<Vec<(log::Level, String)>>);

impl log::Log for Recorder {
    fn enabled(&self, m: &log::Metadata) -> bool {
        m.level() <= log::Level::Info
    }
    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        if let Ok(mut lines) = self.0.lock() {
            lines.push((record.level(), record.args().to_string()));
        }
    }
    fn flush(&self) {}
}

fn recorder() -> &'static Recorder {
    static RECORDER: OnceLock<&'static Recorder> = OnceLock::new();
    RECORDER.get_or_init(|| {
        let r: &'static Recorder = Box::leak(Box::new(Recorder(Mutex::new(Vec::new()))));
        if log::set_logger(r).is_ok() {
            log::set_max_level(log::LevelFilter::Info);
        }
        r
    })
}

fn recorded(level: log::Level, contains: &str) -> Vec<String> {
    recorder()
        .0
        .lock()
        .map(|lines| {
            lines
                .iter()
                .filter(|(l, line)| *l == level && line.contains(contains))
                .map(|(_, line)| line.clone())
                .collect()
        })
        .unwrap_or_default()
}

/// No adapter answers the recovery (the GPU is gone): frames are skipped
/// with no panic, `read` says the device is lost, one error is logged for
/// the two skipped frames; then the adapter is back and the next frame is
/// the frame before the loss again.
#[test]
fn a_recovery_without_an_adapter_leaves_the_renderer_alive_and_says_so_once() {
    recorder();
    let Some(mut r) = renderer() else {
        return;
    };
    let image = image();
    let mut c = Compositor::new();
    let retained = c.create_layer(5);
    scene(&mut c, retained, image, 0);
    let a = render(&mut r, &mut c, "frame A");

    r.gpu_mut().set_recovery_backends(wgpu::Backends::empty());
    r.gpu().device.destroy();

    for frame in 0..2 {
        scene(&mut c, retained, image, 0);
        assert_eq!(
            r.draw(&mut c, clear()),
            0,
            "skipped frame {frame} draws nothing"
        );
        assert!(
            r.gpu().device_lost(),
            "still lost after skipped frame {frame}"
        );
    }
    match r.read() {
        Err(HeadlessError::DeviceLost(lost)) => {
            assert_eq!(lost.reason, wgpu::DeviceLostReason::Destroyed)
        }
        Err(e) => panic!("expected DeviceLost, got {e}"),
        Ok(_) => panic!("a dead target must not read back"),
    }
    let errors = recorded(log::Level::Error, "not recovered");
    assert_eq!(errors.len(), 1, "one error per loss, got {errors:?}");

    // the GPU is back
    r.gpu_mut().set_recovery_backends(wgpu::Backends::PRIMARY);
    scene(&mut c, retained, image, 0);
    let b = render(&mut r, &mut c, "frame after the adapter is back");
    assert!(!r.gpu().device_lost());
    assert_same(&b, &a, "the frame after the recovery against frame A");
    assert_eq!(recorded(log::Level::Error, "not recovered").len(), 1);
}
