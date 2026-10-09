//! What a `VerifiedIr` costs until its first frame, headless, for each
//! fixture of task-68: materializing it over the app's snapshot (the
//! repeats unrolled, one widget per node, the bindings read), laying it
//! out and building its scene (the frame on the CPU), and drawing that
//! scene by the window's frame path into an offscreen target, read back
//! (the pixels). The GPU context and its text system exist already, as
//! in a running app; the glyphs are new to the atlas, as on a first frame.
//!
//! A measurement, not a check. Run by hand, debug as the gate builds it:
//! `cargo test -p surface --test measure -- --ignored --nocapture`. With
//! `SURFACE_FRAMES=<dir>` it also writes each fixture's first frame there
//! as a png, to look at.

// shared with the other surface tests: this binary uses the fixtures.
#[allow(dead_code)]
mod support;

use std::time::{Duration, Instant};

use engine::drawing::compositor::Compositor;
use engine::drawing::gpu::RenderConfig;
use engine::drawing::headless::{Renderer, Viewport};
use engine::styling::theme::Theme;
use engine::widgets::Rect;
use screen::Surface;
use support::{CASES, snapshot, verified};

const ROUNDS: usize = 20;

#[test]
#[ignore = "a measurement, not a check"]
fn verified_ir_to_first_frame() {
    let viewport = Viewport::new(800.0, 600.0, 2.0);
    let Ok(mut renderer) = Renderer::new(viewport, RenderConfig::default()) else {
        println!("no GPU adapter: nothing measured");
        return;
    };
    let theme = Theme::hoff_core();
    let area = Rect::new(0.0, 0.0, viewport.width, viewport.height);
    let clear = theme.colors.bg.to_linear_array();
    for (ir_name, snap_name) in CASES {
        let app = snapshot(snap_name);
        let ir = verified(ir_name, &app);
        let (mut scene, mut pixels) = (Duration::MAX, Duration::MAX);
        let mut nodes = 0;
        for round in 0..ROUNDS {
            let ir = ir.clone();
            let start = Instant::now();
            let mut surface = Surface::materialize(ir, &app);
            let mut c = Compositor::new();
            surface.render(&mut c, area, &theme);
            let built = start.elapsed();
            let frame = renderer.render(&mut c, clear);
            let drawn = start.elapsed();
            let Ok(frame) = frame else {
                panic!("{ir_name}: the frame did not draw");
            };
            if let (0, Some(dir)) = (round, std::env::var_os("SURFACE_FRAMES")) {
                let png = std::path::Path::new(&dir).join(ir_name.replace(".ir.json", ".png"));
                frame.write_png(&png).expect("the png writes");
            }
            nodes = surface.nodes().count();
            // the first round is the first frame: glyphs new to the atlas
            if round == 0 {
                println!(
                    "{ir_name}: {nodes} nodes, first round: scene {} us, pixels {} us",
                    built.as_micros(),
                    drawn.as_micros()
                );
            }
            scene = scene.min(built);
            pixels = pixels.min(drawn);
        }
        println!(
            "{ir_name}: {nodes} nodes, best of {ROUNDS}: scene {} us, pixels {} us",
            scene.as_micros(),
            pixels.as_micros()
        );
    }
}
