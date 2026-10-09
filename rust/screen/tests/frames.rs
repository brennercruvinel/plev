//! The frames of a surface (task-69 phase 4b): an `image` bound to a
//! frame ref the app projects draws the picture the app holds for it,
//! asked through `Frames`; without one it draws its shell and label, and
//! `sources` says what the app should decode. The review of the silences
//! materializes over the app's real projection with a frame per cut; the
//! comparison transforms it in place, keeping the instance of every cut
//! that continues (a press begun on one completes after), and a
//! dismissal fades the pictures out with the rest.

// shared with the other surface tests: this binary uses the fixtures, the
// page and the app.
#[allow(dead_code)]
mod support;

use std::collections::BTreeMap;

use engine::widgets::{Rect, WidgetEvent};
use engine::drawing::compositor::{Compositor, LayerId, SceneNode};
use engine::drawing::gpu::image::{ImageHandle, load_image_rgba};
use engine::styling::theme::Theme;
use accord::{Role, StateRef, Value};
use support::{
    App, COMPARE_SILENCES, REVIEW_SILENCES, SILENCES, TURN, id, page, snapshot, verified_local,
};
use screen::{Frames, NoFrames, Surface, SurfaceHost};

const AREA: Rect = Rect {
    x: 40.0,
    y: 30.0,
    w: 900.0,
    h: 600.0,
};

/// The frames a test app holds: a wash per frame ref, made in memory.
struct Pictures(BTreeMap<String, ImageHandle>);

impl Pictures {
    /// A 32 by 18 wash per source, its gray following its place in the
    /// list, so two sources are two handles.
    fn of(sources: &[&str]) -> Pictures {
        let mut out = BTreeMap::new();
        for (i, s) in sources.iter().enumerate() {
            let v = u8::try_from(60 + i * 40).unwrap_or(200);
            let pixels = std::iter::repeat_n([v, v, v, 255], 32 * 18)
                .flatten()
                .collect();
            let handle = load_image_rgba(32, 18, pixels).expect("a wash loads");
            out.insert((*s).to_owned(), handle);
        }
        Pictures(out)
    }
}

impl Frames for Pictures {
    fn frame(&self, source: &str) -> Option<ImageHandle> {
        self.0.get(source).copied()
    }
}

/// The frame refs the seed projects: the video at the instant shown,
/// and the middle of each cut.
const VIDEO: &str = "silences.video@0";
const CUT_1: &str = "silences.video@4.017052";
const CUT_2: &str = "silences.video@9.009342";

fn images(c: &Compositor) -> Vec<[f32; 4]> {
    c.layer(LayerId::DEFAULT)
        .map(|l| {
            l.nodes()
                .iter()
                .filter_map(|n| match n {
                    SceneNode::Image { x, y, w, h, .. } => Some([*x, *y, *w, *h]),
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default()
}

fn bound(c: &Compositor, at: &str) -> [f32; 4] {
    c.binding(&id(at))
        .unwrap_or_else(|| panic!("{at} is bound"))
        .rect
}

fn inside(r: [f32; 4], outer: [f32; 4]) -> bool {
    r[0] >= outer[0] - 0.5
        && r[1] >= outer[1] - 0.5
        && r[0] + r[2] <= outer[0] + outer[2] + 0.5
        && r[1] + r[3] <= outer[1] + outer[3] + 0.5
}

fn frame(surface: &mut Surface, theme: &Theme, frames: &dyn Frames) -> Compositor {
    let mut c = Compositor::new();
    surface.render_with(&mut c, AREA, theme, frames);
    c
}

/// The acceptance: the review materializes over the silences app's real
/// projection; its images name the frames the app should decode; with
/// the app holding them, each image draws its picture inside the rect it
/// is bound to, one per cut, fitted and never scaled up; without them,
/// the shells and labels, and nothing else changes.
#[test]
fn the_review_draws_a_frame_per_cut_from_the_app_through_the_trait() {
    let theme = Theme::hoff_core();
    let app = snapshot(SILENCES);
    let mut surface = Surface::materialize(verified_local(REVIEW_SILENCES, &app), &app);
    assert_eq!(surface.sources(), [VIDEO, CUT_1, CUT_2]);

    // nothing decoded yet: the shells, no picture
    let c = frame(&mut surface, &theme, &NoFrames);
    assert!(images(&c).is_empty());
    let video = bound(&c, "review.video");
    let thumb = bound(&c, "review.cut[1].frame");
    assert!(video[2] > thumb[2] * 3.0, "{video:?} {thumb:?}");
    assert_eq!(
        thumb[3],
        theme.control.height(engine::styling::theme::ControlSize::Xl)
    );

    // the app holds every frame: one picture per image, inside its rect
    let all = Pictures::of(&[VIDEO, CUT_1, CUT_2]);
    let c = frame(&mut surface, &theme, &all);
    let drawn = images(&c);
    assert_eq!(drawn.len(), 3, "{drawn:?}");
    for (at, picture) in ["review.video", "review.cut[1].frame", "review.cut[2].frame"]
        .into_iter()
        .zip(&drawn)
    {
        let rect = bound(&c, at);
        assert!(inside(*picture, rect), "{at}: {picture:?} in {rect:?}");
        // never scaled up: a 32 by 18 wash stays 32 by 18
        assert_eq!((picture[2], picture[3]), (32.0, 18.0), "{at}");
    }
    // the layout is the same with or without the pictures
    assert_eq!(bound(&c, "review.video"), video);
    assert_eq!(bound(&c, "review.cut[1].frame"), thumb);

    // only the cuts decoded: the video keeps its label
    let cuts = Pictures::of(&[CUT_1, CUT_2]);
    let c = frame(&mut surface, &theme, &cuts);
    assert_eq!(images(&c).len(), 2);
    let labels: Vec<&str> = c
        .layer(LayerId::DEFAULT)
        .map(|l| {
            l.nodes()
                .iter()
                .filter_map(|n| match n {
                    SceneNode::Text { key, .. } => Some(key.text.as_str()),
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default();
    assert!(
        labels.contains(&"the video at the instant shown"),
        "{labels:?}"
    );
    assert!(!labels.contains(&"the middle of the cut"), "{labels:?}");

    // the projection: the images are image nodes, the cuts instances
    let seen = page(&app, &surface);
    assert_eq!(
        seen.get(&id("review.video")).map(|n| n.role),
        Some(Role::Image)
    );
    let instance = seen.get(&id("review.cut[1]")).expect("the instance");
    assert_eq!(instance.label, "1");
    assert_eq!(
        seen.value(&StateRef::parse("review.cut[1].range:text").unwrap()),
        Some(&Value::Text("0:03.0 to 0:05.0".into()))
    );
    assert_eq!(surface.wake(TURN), engine::widgets::Wake::idle(), "at rest");
}

/// The acceptance of the transformation: the review becomes the
/// comparison in place. Every cut that continues keeps its instance: a
/// press begun on a cut's apply button in the review completes after the
/// change of form and applies that cut through the app's dispatcher; the
/// video's image keeps its id and slides, the result's view is born, the
/// detection row leaves as ghosts; and `close` fades everything out.
#[test]
fn the_comparison_keeps_the_instances_of_the_cuts_and_close_fades_it_out() {
    let theme = Theme::hoff_core();
    let mut app = App::over(&snapshot(SILENCES));
    let now = app.snapshot();
    let mut host = SurfaceHost::new();
    host.open(verified_local(REVIEW_SILENCES, &now), &now)
        .expect("the review opens");
    let review = id("review");
    let all = Pictures::of(&[VIDEO, CUT_1, CUT_2]);
    let surface = host.get_mut(&review).expect("open");
    let c = frame(surface, &theme, &all);
    let video_was = bound(&c, "review.video");
    let apply = bound(&c, "review.cut[1].apply");
    let (x, y) = (apply[0] + apply[2] / 2.0, apply[1] + apply[3] / 2.0);
    for event in [
        WidgetEvent::MouseMove { x, y },
        WidgetEvent::MouseDown { x, y },
    ] {
        surface.handle_event(&event, &theme, &mut app);
    }
    assert!(app.executed.is_empty(), "a press alone asks nothing");

    let now = app.snapshot();
    let surface = host
        .transform(verified_local(COMPARE_SILENCES, &now), &now)
        .expect("the review takes the comparison's form");
    // the new form at once: the result's view born, the detection gone
    let seen = page(&now, surface);
    assert!(seen.get(&id("review.result.video")).is_some());
    assert!(seen.get(&id("review.detect")).is_none());
    assert!(seen.get(&id("review.threshold")).is_none());
    assert_eq!(
        surface.sources(),
        [VIDEO, VIDEO, CUT_1, CUT_2]
            .iter()
            .fold(Vec::new(), |mut v, s| {
                if !v.contains(&(*s).to_owned()) {
                    v.push((*s).to_owned());
                }
                v
            }),
        "the result at the instant shown is the same frame as the original"
    );
    // the first frame of the new form draws the button where it was: the
    // release lands on the same widget, and the click completes
    let c = frame(surface, &theme, &all);
    assert_eq!(bound(&c, "review.cut[1].apply"), apply);
    surface.handle_event(&WidgetEvent::MouseUp { x, y }, &theme, &mut app);
    let ran = app.executed.last().expect("the click completed");
    assert_eq!(
        (ran.target.as_str(), ran.action.as_str()),
        ("silences.cut[1]", "apply_cut")
    );
    assert_eq!(
        app.snapshot()
            .value(&StateRef::parse("silences.cut[1]:decision").unwrap()),
        Some(&Value::Text("apply".into()))
    );
    // the ghosts of the detection row fade with the pictures drawn, the
    // video slides to its column, and it settles
    let (c, frames) = settle_with(surface, &theme, &all);
    assert!(frames > 1, "the change of form took {frames} frames");
    assert_ne!(bound(&c, "review.video"), video_was, "the video moved");
    assert!(c.binding(&id("review.detect")).is_none());
    assert_eq!(images(&c).len(), 4, "the original, the result, two cuts");

    // close: out of the projection at once, the pictures fade where they
    // were, then nothing
    assert!(host.close(&review));
    let mut b = accord::SnapshotBuilder::new();
    host.project(&mut b);
    assert!(
        b.finish()
            .expect("projects")
            .get(&id("review.video"))
            .is_none()
    );
    let mut c = Compositor::new();
    host.draw_closing_with(&mut c, &theme, &all);
    assert_eq!(images(&c).len(), 4, "the pictures fade out where they were");
    assert!(c.binding(&id("review.video")).is_none(), "bound to nothing");
    let mut turns = 0;
    while host.wake(TURN).is_continuous() {
        turns += 1;
        assert!(turns < 600, "the dismissal never settles");
    }
    let mut c = Compositor::new();
    host.draw_closing_with(&mut c, &theme, &all);
    assert!(images(&c).is_empty(), "gone");
    assert!(host.is_empty());
}

/// Draws `surface` and then, a turn apart, every frame it asks for while
/// its transitions run, the pictures of `frames`: the last frame, and
/// how many were drawn after the first.
fn settle_with(surface: &mut Surface, theme: &Theme, frames: &dyn Frames) -> (Compositor, usize) {
    let mut c = frame(surface, theme, frames);
    let mut drawn = 0;
    loop {
        let wake = surface.wake(TURN);
        if wake.now {
            c = frame(surface, theme, frames);
            drawn += 1;
        }
        if !wake.is_continuous() {
            return (c, drawn);
        }
        assert!(drawn < 600, "the transitions never settle");
    }
}

/// A surface without an image bound to a frame ref (the todo's) names
/// no source: the app has nothing to decode for it.
#[test]
fn only_a_bound_image_has_a_source() {
    let app = snapshot(support::SHOWCASE);
    let todo = Surface::materialize(support::verified(support::TODO, &app), &app);
    assert!(todo.sources().is_empty());
}
