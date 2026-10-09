//! What an intent costs until the surface is verified and until its first
//! frame, with the deterministic proposer, headless, debug as the gate
//! builds it: the rules alone (the intent, the todo's registry and
//! snapshot and the surface shown, to a `VerifiedIr`); the whole request
//! through the page (observed, dispatched through the todo's dispatcher,
//! proposed, verified, the surface transformed in place); that and the
//! page's next frame built (the scene); and that frame drawn through the
//! window's frame path into an offscreen target and read back (the
//! pixels). The GPU context exists already, as in a running app.
//!
//! A measurement, not a check. Run by hand:
//! `cargo test -p uigallery --lib surface_page::tests::measure -- --ignored --nocapture`.

use std::time::{Duration, Instant};

use engine::drawing::compositor::Compositor;
use engine::drawing::gpu::RenderConfig;
use engine::drawing::headless::{Renderer, Viewport};
use offers::{Ask, Rules, propose_verified};
use plevir::{Capabilities, GenerationPolicy};

use crate::views::ShowcaseView;

const ROUNDS: usize = 20;
const INTENT: &str = "only what is active and the add field";
const DESKTOP: Viewport = Viewport {
    width: 1200.0,
    height: 800.0,
    scale: 1.0,
};

/// The gallery on the Surface page, one frame drawn, the intent written.
fn staged() -> (ShowcaseView, Compositor) {
    let mut view = ShowcaseView::new(DESKTOP.width, DESKTOP.height);
    view.jump_to_page("Surface");
    view.resize(DESKTOP.width, DESKTOP.height, DESKTOP.scale);
    let mut c = Compositor::new();
    view.render(&mut c);
    view.panes.surface.proposing.intent.set_text(INTENT);
    (view, c)
}

fn us(d: Duration) -> u128 {
    d.as_micros()
}

#[test]
#[ignore = "a measurement, not a check"]
fn intent_to_verified_ir_and_to_first_frame() {
    let mut renderer = Renderer::new(DESKTOP, RenderConfig::default()).ok();
    let mut best = [Duration::MAX; 4];
    for round in 0..ROUNDS {
        let (mut view, mut c) = staged();
        // the rules alone
        let page = &view.panes.surface;
        let app = view.panes.todo.snapshot().expect("the todo projects");
        let caps = Capabilities::from_snapshot(&app);
        let policy = GenerationPolicy::default();
        let current = page.id.as_ref().and_then(|id| page.host.get(id));
        let ask = Ask {
            intent: INTENT,
            caps: &caps,
            app: &app,
            current: current.map(screen::Surface::verified),
            policy: &policy,
        };
        let start = Instant::now();
        let verified = propose_verified(&mut Rules, &ask);
        let rules = start.elapsed();
        assert!(verified.is_ok());

        // the whole path, as a person's propose
        let start = Instant::now();
        let todo = &mut view.panes.todo;
        assert!(view.panes.surface.propose(todo).is_some());
        let request = start.elapsed();
        view.render(&mut c);
        let scene = start.elapsed();
        let pixels = renderer.as_mut().map(|r| {
            let clear = view.theme.colors.bg.to_linear_array();
            r.render(&mut c, clear).expect("the frame draws");
            start.elapsed()
        });
        let times = [rules, request, scene, pixels.unwrap_or(Duration::MAX)];
        if round == 0 {
            println!(
                "first round: rules {} us, request {} us, scene {} us, pixels {} us",
                us(times[0]),
                us(times[1]),
                us(times[2]),
                pixels.map_or("(no GPU)".to_owned(), |p| us(p).to_string())
            );
        }
        for (b, t) in best.iter_mut().zip(times) {
            *b = (*b).min(t);
        }
    }
    println!(
        "best of {ROUNDS}: rules {} us, request {} us, scene {} us, pixels {} us",
        us(best[0]),
        us(best[1]),
        us(best[2]),
        us(best[3])
    );
}
