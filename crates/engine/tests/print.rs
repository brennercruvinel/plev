//! Printing as the frame on paper (task-76 item 9): the probe. A text run
//! drawn through the window's frame path without a window
//! (`engine::headless`) at the screen's scale (2.0) and at a paper scale
//! (300 dpi over 72 points per inch, 4.1667), timed piece by piece: the
//! device, the draw and the readback, the png. The glyphs are rasterized
//! at the physical scale (the atlas of task-66), so the ink of the same
//! run is twice as wide at twice the scale, not a blown-up bitmap. Skips
//! without a GPU adapter, like every headless test.

use std::time::Instant;

use engine::compositor::{Compositor, SceneNode, TextNodeKey};
use engine::gpu::RenderConfig;
use engine::headless::{HeadlessError, Renderer, Viewport};
use engine::text::TextStyle;
use engine::text::probe::Rendered;

/// One line of the Todo page, at the body size.
const TEXT: &str = "Absorb the plev demos into the showcase";
/// The logical viewport of the specimen: wide enough for the run at the
/// body size, one line tall.
const LOGICAL: (f32, f32) = (600.0, 80.0);
/// The scale of a Retina screen, and of paper: 300 dpi in points.
const SCREEN: f32 = 2.0;
const PAPER: f32 = 300.0 / 72.0;
/// Background, linear (the probe's own: the sRGB target encodes it to
/// about 108, well under the ink threshold).
const CLEAR: [f32; 4] = [0.15, 0.15, 0.15, 1.0];

fn style() -> TextStyle {
    TextStyle::new(15.0).with_line_height(21.0).with_weight(400)
}

/// The run on a layer of its own, white on the clear.
fn scene() -> Compositor {
    let mut c = Compositor::new();
    let layer = c.create_layer(1);
    c.begin_frame();
    c.push_to_layer(
        layer,
        SceneNode::Text {
            key: TextNodeKey::from_style(TEXT, &style(), None),
            x: 20.0,
            y: 20.0,
            color: [1.0, 1.0, 1.0, 1.0],
        },
    );
    c
}

/// What one scale cost and gave.
struct Measured {
    physical: (u32, u32),
    device_ms: f64,
    draw_read_ms: f64,
    png_ms: f64,
    png_bytes: usize,
    /// The ink of the run in physical pixels, first to last column.
    ink: u32,
}

fn measure(scale: f32) -> Option<Measured> {
    let viewport = Viewport::new(LOGICAL.0, LOGICAL.1, scale);
    let t = Instant::now();
    let mut renderer = match Renderer::new(viewport, RenderConfig::default()) {
        Ok(r) => r,
        Err(HeadlessError::NoAdapter) => {
            eprintln!("no GPU adapter; skipping");
            return None;
        }
        Err(e) => panic!("{e}"),
    };
    let device_ms = t.elapsed().as_secs_f64() * 1000.0;
    let mut c = scene();
    let t = Instant::now();
    let frame = renderer.render(&mut c, CLEAR).expect("the frame reads");
    let draw_read_ms = t.elapsed().as_secs_f64() * 1000.0;
    let t = Instant::now();
    let png = frame.encode_png().expect("the frame encodes");
    let png_ms = t.elapsed().as_secs_f64() * 1000.0;
    let rendered = Rendered {
        width: frame.width,
        height: frame.height,
        rgba: frame.rgba,
    };
    let (first, last) = rendered
        .ink_extent(0, rendered.height)
        .expect("the run left ink");
    Some(Measured {
        physical: (frame.width, frame.height),
        device_ms,
        draw_read_ms,
        png_ms,
        png_bytes: png.len(),
        ink: last + 1 - first,
    })
}

/// The same run at the screen's scale and at paper: the physical size
/// follows the scale, and so does the ink, within the rounding of glyph
/// edges. The timings are the probe's measurements (`--nocapture`).
#[test]
fn the_same_run_on_paper_is_as_many_times_wider_as_the_scale() {
    let Some(screen) = measure(SCREEN) else {
        return;
    };
    let Some(paper) = measure(PAPER).or_else(|| measure(PAPER)) else {
        return;
    };
    for (name, m) in [("screen 2.0", &screen), ("paper 4.1667", &paper)] {
        eprintln!(
            "{name}: {}x{} px, device {:.1} ms, draw+read {:.1} ms, png {:.1} ms ({} bytes), ink {} px",
            m.physical.0, m.physical.1, m.device_ms, m.draw_read_ms, m.png_ms, m.png_bytes, m.ink
        );
    }
    assert_eq!(screen.physical, (1200, 160));
    assert_eq!(paper.physical, (2500, 333));
    let ratio = paper.ink as f32 / screen.ink as f32;
    let expected = PAPER / SCREEN;
    assert!(
        (ratio - expected).abs() < expected * 0.03,
        "the ink grew {ratio:.3}x for a scale {expected:.3}x: the glyphs are not rasterized at the physical scale"
    );
}
