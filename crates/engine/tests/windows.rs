//! Several windows in one engine (task-76 item 2).
//!
//! The probe first: two GPU contexts with two compositors, each drawing
//! its own scene in the same loop, to see what a second window shares
//! with the first and what it does not. Through `engine::headless`, the
//! window's frame path into a texture, so a machine without a window can
//! prove it; the tests skip without a GPU adapter, like their neighbours.

use engine::compositor::{Compositor, SceneNode, TextNodeKey};
use engine::gpu::{ImageHandle, RenderConfig, load_image_rgba};
use engine::headless::{Frame, HeadlessError, Renderer, Viewport};
use engine::text::TextStyle;
use engine::theme::hoff::PAGE_BG;

const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
const BLUE: [f32; 4] = [0.0, 0.0, 1.0, 1.0];
const IMAGE_RGBA: [u8; 4] = [20, 200, 40, 255];

/// Where each scene puts its things, logical pixels at scale 1.
const RECT: (f32, f32) = (10.0, 10.0);
const IMAGE: (f32, f32) = (100.0, 10.0);
const TEXT: (f32, f32) = (10.0, 60.0);

fn clear() -> [f32; 4] {
    PAGE_BG.to_linear_array()
}

fn bg() -> [u8; 4] {
    let [r, g, b, _] = PAGE_BG.to_array();
    [
        (r * 255.0).round() as u8,
        (g * 255.0).round() as u8,
        (b * 255.0).round() as u8,
        255,
    ]
}

fn renderer() -> Option<Renderer> {
    match Renderer::new(Viewport::new(200.0, 100.0, 1.0), RenderConfig::default()) {
        Ok(r) => Some(r),
        Err(HeadlessError::NoAdapter) => {
            eprintln!("no GPU adapter; skipping");
            None
        }
        Err(e) => panic!("headless renderer: {e}"),
    }
}

/// A flat rect, a line of text and, when asked, the shared image.
fn scene(c: &mut Compositor, color: [f32; 4], text: &str, image: Option<ImageHandle>) {
    c.begin_frame();
    c.draw_rect(RECT.0, RECT.1, 40.0, 40.0, color);
    c.push(SceneNode::Text {
        key: TextNodeKey::from_style(text, &TextStyle::new(20.0), None),
        x: TEXT.0,
        y: TEXT.1,
        color: [1.0, 1.0, 1.0, 1.0],
    });
    if let Some(image) = image {
        c.push(SceneNode::Image {
            x: IMAGE.0,
            y: IMAGE.1,
            w: 40.0,
            h: 40.0,
            image,
            corner_radius: 0.0,
        });
    }
}

fn px(frame: &Frame, (x, y): (f32, f32)) -> [u8; 4] {
    frame
        .pixel(x as u32 + 5, y as u32 + 5)
        .expect("inside the frame")
}

/// Pixels in the text's row band that are not the background.
fn text_pixels(frame: &Frame) -> usize {
    let mut n = 0;
    for y in (TEXT.1 as u32)..(TEXT.1 as u32 + 24) {
        for x in (TEXT.0 as u32)..(TEXT.0 as u32 + 120) {
            if frame.pixel(x, y) != Some(bg()) {
                n += 1;
            }
        }
    }
    n
}

/// The probe: two contexts, two compositors, two scenes in one loop.
/// Each shows only what was pushed into it (the glyph atlas, the layer
/// textures, the pipelines are per context); the image store is the one
/// global log, so a handle loaded once draws in either context without
/// a second load.
#[test]
fn two_contexts_draw_two_scenes_in_the_same_loop_and_share_only_the_image_log() {
    let (Some(mut a), Some(mut b)) = (renderer(), renderer()) else {
        return;
    };
    let mut ca = Compositor::new();
    let mut cb = Compositor::new();
    let image = load_image_rgba(8, 8, IMAGE_RGBA.repeat(64)).expect("an 8x8 image packs");

    for round in 0..3 {
        scene(&mut ca, RED, "one", Some(image));
        let draws_a = a.draw(&mut ca, clear());
        scene(&mut cb, BLUE, "two", None);
        let draws_b = b.draw(&mut cb, clear());
        assert!(draws_a > 0 && draws_b > 0, "round {round}: {draws_a} {draws_b}");
    }

    let fa = a.read().expect("frame a");
    let fb = b.read().expect("frame b");
    assert_eq!(px(&fa, RECT), [255, 0, 0, 255], "a: its own rect");
    assert_eq!(px(&fb, RECT), [0, 0, 255, 255], "b: its own rect");
    assert_eq!(px(&fa, IMAGE), IMAGE_RGBA, "a: the image it pushed");
    assert_eq!(px(&fb, IMAGE), bg(), "b: no image was pushed into it");
    assert!(text_pixels(&fa) > 20, "a: text drawn from its own atlas");
    assert!(text_pixels(&fb) > 20, "b: text drawn from its own atlas");
    assert_ne!(
        fa.rgba, fb.rgba,
        "two scenes, two frames: nothing was drawn into the wrong context"
    );

    // the image loaded before b existed draws in b on its next frame:
    // the store is a log every atlas replays, never drained by a's upload
    scene(&mut cb, BLUE, "two", Some(image));
    b.draw(&mut cb, clear());
    let fb = b.read().expect("frame b");
    assert_eq!(px(&fb, IMAGE), IMAGE_RGBA, "b: the shared image, from the log");
    assert_eq!(px(&fb, RECT), [0, 0, 255, 255]);
}
