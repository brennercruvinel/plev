//! What the frames show while a surface changes form, and what they do
//! not wait for. A node born is projected, bound and hit at once and
//! fades in; a node that left is out of the projection at once, an act on
//! it is refused as on any node that does not exist, and its picture
//! fades out where it was, bound to nothing. The transition asks for
//! frames until it settles, then for none; with reduced motion the change
//! is instant; a dismissal fades the surface out and lets it go.

// shared with the other surface tests: this binary uses the fixtures, the
// page and the app.
#[allow(dead_code)]
mod support;

use accord::{Actor, Rejection, Request, SemId, SnapshotBuilder, Stage, StateRef, Value};
use engine::drawing::compositor::{Compositor, LayerId, SceneNode};
use engine::styling::theme::Theme;
use engine::widgets::form::EditKey;
use engine::widgets::{Rect, Wake, WidgetEvent};
use plevir::{Capabilities, GenerationPolicy, verify_json};
use screen::{Dispatch, Surface, SurfaceHost};
use support::{
    App, MIXER_LEFT, MIXER_RIGHT, SHOWCASE, TODO, TURN, id, mixer, page, settle, snapshot,
    verified, verified_local,
};

const AREA: Rect = Rect {
    x: 40.0,
    y: 30.0,
    w: 900.0,
    h: 600.0,
};

fn frame(surface: &mut Surface, theme: &Theme) -> Compositor {
    let mut c = Compositor::new();
    surface.render(&mut c, AREA, theme);
    c
}

/// The alpha of the text run `s` in the frame, if it is drawn.
fn ink(c: &Compositor, s: &str) -> Option<f32> {
    let nodes = c.layer(LayerId::DEFAULT).map_or(&[][..], |l| l.nodes());
    nodes.iter().find_map(|n| match n {
        SceneNode::Text { key, color, .. } if key.text == s => Some(color[3]),
        _ => None,
    })
}

fn center(c: &Compositor, at: &SemId) -> (f32, f32) {
    let [x, y, w, h] = c
        .binding(at)
        .unwrap_or_else(|| panic!("{at} is bound"))
        .rect;
    (x + w / 2.0, y + h / 2.0)
}

fn click(surface: &mut Surface, app: &mut App, (x, y): (f32, f32), theme: &Theme) {
    for event in [
        WidgetEvent::MouseMove { x, y },
        WidgetEvent::MouseDown { x, y },
        WidgetEvent::MouseUp { x, y },
    ] {
        surface.handle_event(&event, theme, app);
    }
}

/// The mixer in its left form, drawn once and at rest.
fn mixer_left(theme: &Theme) -> (App, Surface, Compositor) {
    let app = App::over(&mixer());
    let now = app.snapshot();
    let mut surface = Surface::materialize(verified_local(MIXER_LEFT, &now), &now);
    let c = frame(&mut surface, theme);
    assert_eq!(surface.wake(TURN), Wake::idle(), "at rest");
    (app, surface, c)
}

fn to_right(surface: &mut Surface, app: &App) {
    let now = app.snapshot();
    surface.transform(verified_local(MIXER_RIGHT, &now), &now);
}

/// The right form has a hint the left one has not, and no reset: the
/// hint is a node of the page from the change on, drawn fading in and
/// bound; the reset is out of the page at once, its picture fading out
/// where it was, bound to nothing and hit by nothing, and an agent's act
/// on it, observed before the change, is refused as not found.
#[test]
fn a_node_born_is_there_at_once_and_a_node_gone_is_gone_at_once_while_its_picture_fades() {
    let theme = Theme::hoff_core();
    let (mut app, mut surface, before) = mixer_left(&theme);
    let (hint, reset) = (id("mix_panel.hint"), id("mix_panel.reset"));
    let label = ink(&before, "reset").expect("the reset is drawn");
    let at = center(&before, &reset);
    let seen = page(&app.snapshot(), &surface);
    assert!(seen.get(&reset).is_some());

    to_right(&mut surface, &app);
    let now = page(&app.snapshot(), &surface);
    assert!(now.get(&hint).is_some(), "born: projected at once");
    assert!(now.get(&reset).is_none(), "gone: out of the page at once");

    let c = frame(&mut surface, &theme);
    let fading = ink(&c, "drag to set the volume").expect("the hint is drawn");
    assert!(fading < 0.5, "the hint fades in: {fading}");
    assert!(c.binding(&hint).is_some(), "and is bound already");
    let leaving = ink(&c, "reset").expect("the reset's picture is drawn");
    assert!(leaving <= label, "{leaving} of {label}");
    assert!(c.binding(&reset).is_none(), "bound to nothing");
    assert_ne!(c.hit(at.0, at.1), Some(&reset));
    click(&mut surface, &mut app, at, &theme);
    assert!(
        app.executed.is_empty(),
        "a click on the picture asks nothing"
    );

    // an agent acts on what it observed before the change
    let agent = Actor::Agent {
        name: "claude".into(),
    };
    let req = Request::observe(&seen, agent, &reset, "activate", None).expect("observed");
    let routed = surface.route(req);
    assert_eq!(routed.target, reset, "no node of the surface: as it is");
    let receipt = app.dispatch(routed).expect("a receipt");
    let Some((Stage::Rejected(Rejection::NotFound { target }), _)) = receipt.stages.last() else {
        panic!("not found: {:?}", receipt.stages);
    };
    assert_eq!(target, &reset);
    assert!(app.executed.is_empty());

    let (c, _) = settle(&mut surface, AREA, &theme);
    assert_eq!(ink(&c, "reset"), None, "its picture is gone");
    let shown = ink(&c, "drag to set the volume").expect("the hint");
    assert!(shown > fading, "{shown} after {fading}");
}

/// Render on demand, counted as task-82 counted: two seconds of turns
/// after the change, a frame drawn only on a turn that asks for one. The
/// transition asks from the change until it settles; then no turn asks.
#[test]
fn a_transition_asks_for_frames_until_it_settles_and_then_for_none() {
    let theme = Theme::hoff_core();
    let (app, mut surface, _) = mixer_left(&theme);
    to_right(&mut surface, &app);
    let mut drawn = Vec::new();
    for turn in 0..120 {
        let wake = surface.wake(TURN);
        if wake.now {
            frame(&mut surface, &theme);
            drawn.push(turn);
        }
    }
    let slide = (theme.duration.slow / TURN) as usize;
    let n = drawn.len();
    println!("the mixer's change of form: {n} of 120 turns drawn");
    assert!(n > slide && n < 60, "{n} frames: {drawn:?}");
    // one run from the change on, no gap, and nothing after it settled
    assert_eq!(drawn, (0..n).collect::<Vec<_>>());
    for _ in 0..120 {
        assert_eq!(surface.wake(TURN), Wake::idle());
    }
}

/// With reduced motion the change is instant: one frame draws the new
/// form where it goes, the born node whole, nothing of the gone one, and
/// no turn after it asks for another.
#[test]
fn reduced_motion_makes_the_change_instant() {
    let theme = Theme::hoff_core();
    let (app, mut surface, before) = mixer_left(&theme);
    surface.set_reduced_motion(true);
    let slider = id("mix_panel.volume");
    let left = before.binding(&slider).expect("bound").rect;
    to_right(&mut surface, &app);
    assert!(surface.wake(TURN).now, "the change asks for its frame");
    let c = frame(&mut surface, &theme);
    let right = c.binding(&slider).expect("bound").rect;
    let panel = c.binding(&id("mix_panel.right")).expect("bound").rect;
    assert!(right[0] >= panel[0] && right[0] + right[2] <= panel[0] + panel[2] + 0.5);
    assert_ne!(left, right);
    assert_eq!(ink(&c, "reset"), None);
    let hint = ink(&c, "drag to set the volume").expect("the hint");
    let (settled, _) = settle(&mut surface, AREA, &theme);
    assert_eq!(
        ink(&settled, "drag to set the volume"),
        Some(hint),
        "whole at once"
    );
    for _ in 0..120 {
        assert_eq!(surface.wake(TURN), Wake::idle());
    }
}

/// Reduced motion asked for in the middle of a transition ends it: the
/// next frame is the settled one.
#[test]
fn reduced_motion_asked_mid_transition_ends_it() {
    let theme = Theme::hoff_core();
    let (app, mut surface, _) = mixer_left(&theme);
    to_right(&mut surface, &app);
    frame(&mut surface, &theme);
    assert!(surface.wake(TURN).is_continuous());
    surface.set_reduced_motion(true);
    let c = frame(&mut surface, &theme);
    assert_eq!(ink(&c, "reset"), None);
    assert_eq!(surface.wake(TURN), Wake::idle());
}

/// A form of the todo without its field: the field leaves.
const COUNTS_ONLY: &str = r#"{"plev_ir": 1, "surface": {"id": "todo_panel", "title": "quick todo",
    "root": {"sem": "todo_panel.card", "component": "card", "label": "todo",
        "props": {"padding": "lg", "elevation": "raised"},
        "children": [{"sem": "todo_panel.active", "component": "stat", "label": "active",
            "bind": {"state": {"value": "todo.counter:active"}}}]}}}"#;

/// A node that comes back before its picture is gone is the same
/// instance: the field that left mid-edit returns with its caret where it
/// was (it gave the keyboard up when it left).
#[test]
fn a_node_back_before_its_picture_is_gone_is_the_same_instance() {
    let theme = Theme::hoff_core();
    let mut app = App::over(&snapshot(SHOWCASE));
    let now = app.snapshot();
    let mut surface = Surface::materialize(verified(TODO, &now), &now);
    let field = id("todo_panel.input");
    let c = frame(&mut surface, &theme);
    click(&mut surface, &mut app, center(&c, &field), &theme);
    for ch in "buy milk".chars() {
        assert!(surface.handle_text(&ch.to_string(), &mut app));
    }
    for _ in 0..3 {
        assert!(surface.handle_edit_key(EditKey::Left, &mut app));
    }

    let caps = Capabilities::from_snapshot(&app.snapshot());
    let counts = verify_json(COUNTS_ONLY, &caps, &GenerationPolicy::default()).expect("verifies");
    surface.transform(counts, &app.snapshot());
    assert!(!surface.holds(&field));
    assert_eq!(surface.focused(), None, "the keyboard left with it");
    frame(&mut surface, &theme);
    frame(&mut surface, &theme);
    assert!(
        surface.wake(TURN).is_continuous(),
        "its picture still fades"
    );

    let now = app.snapshot();
    surface.transform(verified(TODO, &now), &now);
    assert!(surface.focus(&field));
    assert!(surface.handle_text("Z", &mut app));
    let text = app
        .snapshot()
        .value(&StateRef::parse("todo.input:text").unwrap())
        .cloned();
    assert_eq!(text, Some(Value::Text("buy mZilk".into())), "its caret");
}

/// The page's projection, as the host adds its surfaces to it.
fn projected(host: &SurfaceHost, app: &App) -> accord::Snapshot {
    let mut b = SnapshotBuilder::new();
    for node in app.snapshot().nodes().values() {
        support::copy(&mut b, node);
    }
    host.project(&mut b);
    b.finish().expect("the page projects")
}

fn closing(host: &mut SurfaceHost, theme: &Theme) -> Compositor {
    let mut c = Compositor::new();
    host.draw_closing(&mut c, theme);
    c
}

/// `close` takes the surface out of the page at once and fades its
/// picture out where it was drawn, bound to nothing; then the host lets
/// everything go and asks for no frame.
#[test]
fn a_dismissal_fades_the_surface_out_and_then_lets_it_go() {
    let theme = Theme::hoff_core();
    let app = App::over(&mixer());
    let now = app.snapshot();
    let mut host = SurfaceHost::new();
    let panel = id("mix_panel");
    host.open(verified_local(MIXER_LEFT, &now), &now)
        .expect("opens");
    frame(host.get_mut(&panel).expect("open"), &theme);
    assert!(projected(&host, &app).get(&panel).is_some());

    assert!(host.close(&panel));
    assert!(!host.close(&panel), "closed already");
    assert!(host.is_empty() && host.get(&panel).is_none());
    let page = projected(&host, &app);
    assert!(page.get(&panel).is_none() && page.get(&id("mix_panel.volume")).is_none());
    let req = Request::observe(&page, Actor::Human, "mixer.volume", "reset", None)
        .expect("the app's own node");
    assert_eq!(host.route(req.clone()), req, "nothing of it to route to");

    let c = closing(&mut host, &theme);
    assert!(ink(&c, "reset").is_some(), "its picture");
    assert_eq!(c.bindings().count(), 0, "bound to nothing");
    let mut frames = 0;
    loop {
        let wake = host.wake(TURN);
        if wake.now {
            closing(&mut host, &theme);
            frames += 1;
        }
        if !wake.is_continuous() {
            break;
        }
        assert!(frames < 600);
    }
    assert!(frames > 1, "{frames}");
    let c = closing(&mut host, &theme);
    assert!(
        c.layer(LayerId::DEFAULT)
            .is_none_or(|l| l.nodes().is_empty())
    );
    assert_eq!(host.wake(TURN), Wake::idle());

    // with reduced motion a dismissal lets go at once
    host.set_reduced_motion(true);
    host.open(verified_local(MIXER_LEFT, &now), &now)
        .expect("opens");
    frame(host.get_mut(&panel).expect("open"), &theme);
    assert!(host.close(&panel));
    let c = closing(&mut host, &theme);
    assert!(
        c.layer(LayerId::DEFAULT)
            .is_none_or(|l| l.nodes().is_empty())
    );
    assert_eq!(host.wake(TURN), Wake::idle());
}

/// The thread's reduced motion (task-75: the system's, which the bridge
/// sets on the loop's thread) makes a change instant too, with nothing
/// told to the surface: the first frame after the change draws the new
/// form, the born node whole and nothing of the gone one, and no turn
/// after it asks for another; a dismissal lets go at once. A thread
/// starts with motion (`engine::compose::animation::set_reduced_motion`).
#[test]
fn the_thread_s_reduced_motion_makes_the_change_instant_from_its_first_frame() {
    assert!(
        !engine::compose::animation::reduced_motion(),
        "a thread starts with motion"
    );
    engine::compose::animation::set_reduced_motion(true);
    let theme = Theme::hoff_core();
    let (app, mut surface, _) = mixer_left(&theme);
    to_right(&mut surface, &app);
    assert!(surface.wake(TURN).now, "the change asks for its frame");
    let c = frame(&mut surface, &theme);
    assert_eq!(ink(&c, "reset"), None, "nothing of the gone node");
    let hint = ink(&c, "drag to set the volume").expect("the hint");
    assert_eq!(hint, 1.0, "the born node whole on the first frame");
    for _ in 0..120 {
        assert_eq!(surface.wake(TURN), Wake::idle());
    }
    let now = app.snapshot();
    let mut host = SurfaceHost::new();
    let panel = id("mix_panel");
    host.open(verified_local(MIXER_LEFT, &now), &now)
        .expect("opens");
    frame(host.get_mut(&panel).expect("open"), &theme);
    assert!(host.close(&panel));
    assert_eq!(host.wake(TURN), Wake::idle(), "let go at once");
    engine::compose::animation::set_reduced_motion(false);
}
