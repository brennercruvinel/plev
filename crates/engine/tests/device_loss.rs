//! task-76, phase 1: the probe.
//!
//! What a destroyed GPU device does to the headless frame path today,
//! before the engine knows about device loss. Ignored by default: it
//! prints, it does not assert. Run it with
//! `cargo test -p engine --test device_loss -- --ignored --nocapture`.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Arc, Mutex};

use engine::compositor::{Compositor, RoundedRectParams, SceneNode, TextNodeKey};
use engine::gpu::{ImageHandle, RenderConfig, load_image_rgba};
use engine::headless::{HeadlessError, Renderer, Viewport};
use engine::text::TextStyle;
use engine::theme::hoff::PAGE_BG;
use engine::wgpu;

const IMAGE_RGBA: [u8; 4] = [20, 200, 40, 255];

fn clear() -> [f32; 4] {
    PAGE_BG.to_linear_array()
}

fn image() -> ImageHandle {
    let pixels = IMAGE_RGBA.repeat(8 * 8);
    load_image_rgba(8, 8, pixels).expect("an 8x8 image packs")
}

/// Text, a rounded rect with a border, a flat rect and an image: one of
/// each resource kind that lives on the device.
fn scene(c: &mut Compositor, image: ImageHandle) {
    c.begin_frame();
    c.draw_rect(10.0, 10.0, 20.0, 20.0, [1.0, 0.0, 0.0, 1.0]);
    c.draw_rounded_rect(RoundedRectParams {
        x: 40.0,
        y: 40.0,
        w: 120.0,
        h: 60.0,
        color: [0.2, 0.4, 0.9, 1.0],
        corner_radius: 12.0,
        border_width: 1.0,
        border_color: [1.0, 1.0, 1.0, 0.3],
    });
    c.push(SceneNode::Text {
        key: TextNodeKey::from_style(
            "device loss",
            &TextStyle::new(16.0).with_line_height(22.4),
            None,
        ),
        x: 50.0,
        y: 60.0,
        color: [1.0, 1.0, 1.0, 1.0],
    });
    c.draw_image(180.0, 10.0, 40.0, 40.0, image, 0.0);
}

fn panic_text(p: &Box<dyn std::any::Any + Send>) -> String {
    p.downcast_ref::<String>()
        .cloned()
        .or_else(|| p.downcast_ref::<&str>().map(|s| s.to_string()))
        .unwrap_or_else(|| "non-string panic".to_string())
}

#[test]
#[ignore = "task-76 probe: prints what a destroyed device does today"]
fn probe_destroy_then_draw() {
    let mut r = match Renderer::new(Viewport::new(240.0, 140.0, 2.0), RenderConfig::default()) {
        Ok(r) => r,
        Err(HeadlessError::NoAdapter) => {
            eprintln!("no GPU adapter; skipping");
            return;
        }
        Err(e) => panic!("{e}"),
    };
    eprintln!("adapter backend features: {:?}", r.gpu().device.features());
    let image = image();

    let seen: Arc<Mutex<Vec<(wgpu::DeviceLostReason, String)>>> = Arc::default();
    {
        let seen = seen.clone();
        r.gpu().device.set_device_lost_callback(move |reason, msg| {
            seen.lock().unwrap().push((reason, msg));
        });
    }

    let mut c = Compositor::new();
    scene(&mut c, image);
    let a = r.render(&mut c, clear()).expect("frame A");
    let ink = a.rgba.chunks_exact(4).filter(|p| p[..3] != [48, 48, 48]).count();
    eprintln!("frame A: {}x{}, {ink} non-background pixels", a.width, a.height);
    eprintln!("frame A image pixel (400,60) = {:?}", a.pixel(400, 60));

    r.gpu().device.destroy();
    eprintln!("after destroy, before any poll: callback = {:?}", seen.lock().unwrap());
    let p1 = r.gpu().device.poll(wgpu::PollType::Poll);
    eprintln!("poll(Poll) = {p1:?}; callback = {:?}", seen.lock().unwrap());
    let p2 = r.gpu().device.poll(wgpu::PollType::wait_indefinitely());
    eprintln!("poll(wait) = {p2:?}; callback = {:?}", seen.lock().unwrap());

    scene(&mut c, image);
    let drawn = catch_unwind(AssertUnwindSafe(|| r.draw(&mut c, clear())));
    match &drawn {
        Ok(draws) => eprintln!("draw after destroy: returned {draws} draws, no panic"),
        Err(p) => eprintln!("draw after destroy: PANIC: {}", panic_text(p)),
    }
    let read = catch_unwind(AssertUnwindSafe(|| r.read()));
    match read {
        Ok(Ok(f)) => eprintln!(
            "read after destroy: ok, equal to A: {}, image pixel = {:?}",
            f.rgba == a.rgba,
            f.pixel(400, 60)
        ),
        Ok(Err(e)) => eprintln!("read after destroy: error: {e}"),
        Err(p) => eprintln!("read after destroy: PANIC: {}", panic_text(&p)),
    }

    // A second frame on a fresh renderer still works (the machine's GPU is
    // fine, only that device is gone).
    let mut fresh = Renderer::new(Viewport::new(240.0, 140.0, 2.0), RenderConfig::default())
        .expect("a fresh renderer");
    let mut c2 = Compositor::new();
    scene(&mut c2, image);
    let b = fresh.render(&mut c2, clear()).expect("frame on a fresh device");
    eprintln!(
        "fresh renderer, same scene: equal to A: {}, image pixel = {:?}",
        b.rgba == a.rgba,
        b.pixel(400, 60)
    );

    // Can a recovery be made to fail on purpose? An instance with no
    // backends must answer "no adapter" rather than panic.
    let empty = catch_unwind(|| {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::empty(),
            ..Default::default()
        });
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
            .map(|a| a.get_info().name)
            .map_err(|e| e.to_string())
    });
    match empty {
        Ok(r) => eprintln!("Backends::empty(): request_adapter = {r:?}"),
        Err(p) => eprintln!("Backends::empty(): PANIC: {}", panic_text(&p)),
    }
}
