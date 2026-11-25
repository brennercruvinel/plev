//! A surface changes form. Before `transform` the only way from one
//! verified form to another was to close the surface and open the other
//! one: the ids came back, the app's state came back through the
//! bindings, and every widget was new, so what lived in a widget (the
//! focus, the caret) was gone. `transform` reconciles by `SemId`: an id
//! that stays under the same widget keeps it, and what lives in it.

// shared with the other surface tests: this binary uses the fixtures, the
// page and the app.
#[allow(dead_code)]
use crate::support;

use accord::{Role, SemId, Snapshot, StateRef, Value};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;
use engine::widgets::form::EditKey;
use engine::widgets::{Rect, WidgetEvent};
use plevir::{Capabilities, GenerationPolicy, verify_json};
use screen::{Surface, SurfaceError, SurfaceHost};
use support::{
    App, COLUMNS, MIXER_LEFT, MIXER_RIGHT, SHOWCASE, TODO, id, mixer, page, settle, snapshot,
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

fn bound(c: &Compositor, at: &SemId) -> [f32; 4] {
    c.binding(at)
        .unwrap_or_else(|| panic!("{at} is bound"))
        .rect
}

fn click(surface: &mut Surface, app: &mut App, [x, y, w, h]: [f32; 4], theme: &Theme) {
    let (x, y) = (x + w / 2.0, y + h / 2.0);
    for event in [
        WidgetEvent::MouseMove { x, y },
        WidgetEvent::MouseDown { x, y },
        WidgetEvent::MouseUp { x, y },
    ] {
        surface.handle_event(&event, theme, app);
    }
}

fn text(app: &App) -> Option<Value> {
    app.snapshot()
        .value(&StateRef::parse("todo.input:text").unwrap())
        .cloned()
}

fn typed(s: &str) -> Option<Value> {
    Some(Value::Text(s.to_owned()))
}

/// The probe of phase 2, on the base: a person types in the compact
/// todo, the caret three places back; then the surface is closed and
/// the two-column form, verified against the app as it is, is opened
/// with the same ids. The field is somewhere else, its text is back (the
/// app's), and its focus and caret are gone (they were the widget's).
#[test]
fn closing_and_reopening_makes_every_widget_new() {
    let theme = Theme::hoff_core();
    let mut app = App::over(&snapshot(SHOWCASE));
    let now = app.snapshot();
    let mut host = SurfaceHost::new();
    let panel = id("todo_panel");
    let field = id("todo_panel.input");
    host.open(verified(TODO, &now), &now).expect("opens");

    let surface = host.get_mut(&panel).expect("open");
    let c = frame(surface, &theme);
    let compact = bound(&c, &field);
    click(surface, &mut app, compact, &theme);
    for ch in "buy milk".chars() {
        assert!(surface.handle_text(&ch.to_string(), &mut app));
    }
    for _ in 0..3 {
        assert!(surface.handle_edit_key(EditKey::Left, &mut app));
    }
    assert!(surface.handle_text("X", &mut app));
    // the caret is the widget's: the key lands three places back
    assert_eq!(text(&app), typed("buy mXilk"));
    assert_eq!(surface.focused(), Some(&field));

    assert!(host.close(&panel));
    let now = app.snapshot();
    host.open(verified_local(COLUMNS, &now), &now)
        .expect("the other form opens");
    let surface = host.get_mut(&panel).expect("open again");
    let c = frame(surface, &theme);
    let columns = bound(&c, &field);
    assert_ne!(compact, columns, "the same id, somewhere else");
    // the text is the app's: it comes back through the binding
    let shown = StateRef::parse("todo.input:text").unwrap();
    assert_eq!(surface.shown().value(&shown), typed("buy mXilk").as_ref());
    // the focus was the widget's: gone with it
    assert_eq!(surface.focused(), None);
    assert!(
        !surface.handle_text("Y", &mut app),
        "no field has the keyboard"
    );
    // and the caret: focused again, a key lands at the end
    assert!(surface.focus(&field));
    assert!(surface.handle_text("Y", &mut app));
    assert_eq!(text(&app), typed("buy mXilkY"));
}

/// The same person, the same keys, the other form by `transform`: the
/// field is somewhere else and it is the same widget, focused, its caret
/// three places back; the new form is what the page projects at once.
#[test]
fn transforming_keeps_the_widget_and_the_state_of_every_id_that_stays() {
    let theme = Theme::hoff_core();
    let mut app = App::over(&snapshot(SHOWCASE));
    let now = app.snapshot();
    let mut host = SurfaceHost::new();
    let field = id("todo_panel.input");
    host.open(verified(TODO, &now), &now).expect("opens");
    let surface = host.get_mut(&id("todo_panel")).expect("open");
    let c = frame(surface, &theme);
    let compact = bound(&c, &field);
    click(surface, &mut app, compact, &theme);
    for ch in "buy milk".chars() {
        assert!(surface.handle_text(&ch.to_string(), &mut app));
    }
    for _ in 0..3 {
        assert!(surface.handle_edit_key(EditKey::Left, &mut app));
    }

    let now = app.snapshot();
    let surface = host
        .transform(verified_local(COLUMNS, &now), &now)
        .expect("the open surface takes the other form");
    // the same widget: it has the keyboard, and the key lands where its
    // caret was, three places back
    assert_eq!(surface.focused(), Some(&field));
    assert!(surface.handle_text("X", &mut app));
    assert_eq!(text(&app), typed("buy mXilk"));
    let (c, _) = settle(surface, AREA, &theme);
    assert_ne!(bound(&c, &field), compact, "the field is elsewhere");
    // the page projects the new form at once: what was born, and not
    // what left
    let seen = page(&app.snapshot(), surface);
    assert!(seen.get(&id("todo_panel.side")).is_some());
    assert!(seen.get(&id("todo_panel.counts")).is_none());
    assert_eq!(seen.focused(), Some(&field));

    // and back again: still the same widget
    let now = app.snapshot();
    let surface = host.transform(verified(TODO, &now), &now).expect("back");
    assert!(surface.handle_text("Y", &mut app));
    assert_eq!(text(&app), typed("buy mXYilk"));
    let (c, _) = settle(surface, AREA, &theme);
    assert_eq!(bound(&c, &field), compact, "the compact form again");
}

/// The value of a node at `id` in the page `seen`.
fn value(seen: &Snapshot, at: &str) -> Option<Value> {
    seen.value(&StateRef::parse(at).unwrap()).cloned()
}

/// A node whose id stays under another component gets that component's
/// widget; nothing of the old widget passes, and what it shows is the
/// app's, read again through its binding. The mute of the mixer is a
/// switch in one form and a checkbox in the other.
#[test]
fn another_component_under_the_same_id_is_a_new_widget_over_the_same_state() {
    let theme = Theme::hoff_core();
    let mut app = App::over(&mixer());
    let now = app.snapshot();
    let mut surface = Surface::materialize(verified_local(MIXER_LEFT, &now), &now);
    let mute = id("mix_panel.mute");
    let c = frame(&mut surface, &theme);
    click(&mut surface, &mut app, bound(&c, &mute), &theme);
    assert_eq!(
        value(&app.snapshot(), "mixer.mute:on"),
        Some(Value::Bool(true))
    );
    let seen = page(&app.snapshot(), &surface);
    assert_eq!(seen.get(&mute).map(|n| n.role), Some(Role::Toggle));

    surface.transform(
        verified_local(MIXER_RIGHT, &app.snapshot()),
        &app.snapshot(),
    );
    let seen = page(&app.snapshot(), &surface);
    let node = seen.get(&mute).expect("the id stays");
    assert_eq!(node.role, Role::Checkbox);
    assert_eq!(
        value(&seen, "mix_panel.mute:checked"),
        Some(Value::Bool(true)),
        "the app's state, read by the new widget"
    );
    let c = frame(&mut surface, &theme);
    click(&mut surface, &mut app, bound(&c, &mute), &theme);
    let ran = app.executed.last().expect("the checkbox toggled");
    assert_eq!(
        (ran.target.as_str(), ran.action.as_str()),
        ("mixer.mute", "toggle")
    );
    assert_eq!(
        value(&app.snapshot(), "mixer.mute:on"),
        Some(Value::Bool(false))
    );
}

/// A form for a surface that is not open is refused, and so is one whose
/// node would take the id of a node another open surface holds; the
/// surface keeps the form it had.
#[test]
fn a_transformation_is_refused_for_a_surface_not_open_or_an_id_another_holds() {
    let todo = snapshot(SHOWCASE);
    let mut host = SurfaceHost::new();
    let refused = host.transform(verified_local(COLUMNS, &todo), &todo).err();
    assert_eq!(refused, Some(SurfaceError::NotOpen(id("todo_panel"))));

    host.open(verified(TODO, &todo), &todo)
        .expect("the todo opens");
    let mixer = mixer();
    host.open(verified_local(MIXER_LEFT, &mixer), &mixer)
        .expect("the mixer opens beside it");
    // the mixer's card named as the todo's card
    let taken = r#"{"plev_ir": 1, "surface": {"id": "mix_panel", "title": "mixer",
        "root": {"sem": "todo_panel.card", "component": "card", "label": "mixer"}}}"#;
    let caps = Capabilities::from_snapshot(&mixer);
    let ir = verify_json(taken, &caps, &GenerationPolicy::default()).expect("it verifies");
    let refused = host.transform(ir, &mixer).err();
    assert_eq!(
        refused,
        Some(SurfaceError::Collision(
            id("todo_panel.card"),
            id("todo_panel")
        ))
    );
    let kept = host.get(&id("mix_panel")).expect("still open");
    assert!(kept.holds(&id("mix_panel.volume")), "the form it had");
}

/// Whether `r` lies inside `outer`.
fn inside(r: [f32; 4], outer: [f32; 4]) -> bool {
    r[0] >= outer[0]
        && r[1] >= outer[1]
        && r[0] + r[2] <= outer[0] + outer[2] + 0.5
        && r[1] + r[3] <= outer[1] + outer[3] + 0.5
}

/// The x at the fraction `t` of a slider's travel drawn at `r`.
fn along(r: [f32; 4], t: f32, theme: &Theme) -> f32 {
    let thumb = theme.selection.slider.thumb_w;
    r[0] + thumb / 2.0 + t * (r[2] - thumb)
}

fn volume(app: &App) -> f64 {
    match value(&app.snapshot(), "mixer.volume:value") {
        Some(Value::Num(n)) => n,
        other => panic!("a number: {other:?}"),
    }
}

/// A slider in a drag stays in the drag when the surface changes form
/// and it changes panel: the pointer that moves on sets the value by the
/// rect the slider is drawn at, sliding from the old panel and then in
/// the new one, and the release ends the drag.
#[test]
fn a_drag_goes_on_after_the_slider_changes_panel() {
    let theme = Theme::hoff_core();
    let mut app = App::over(&mixer());
    let now = app.snapshot();
    let mut surface = Surface::materialize(verified_local(MIXER_LEFT, &now), &now);
    let slider = id("mix_panel.volume");
    let c = frame(&mut surface, &theme);
    let left = bound(&c, &slider);
    assert!(inside(left, bound(&c, &id("mix_panel.left"))));
    let y = left[1] + left[3] / 2.0;
    for event in [
        WidgetEvent::MouseMove {
            x: along(left, 0.25, &theme),
            y,
        },
        WidgetEvent::MouseDown {
            x: along(left, 0.25, &theme),
            y,
        },
        WidgetEvent::MouseMove {
            x: along(left, 0.5, &theme),
            y,
        },
    ] {
        surface.handle_event(&event, &theme, &mut app);
    }
    assert!((volume(&app) - 0.5).abs() < 1e-3, "{}", volume(&app));

    let now = app.snapshot();
    surface.transform(verified_local(MIXER_RIGHT, &now), &now);
    // the first frame of the new form draws it where it was, about to
    // slide: the pointer moves by what the person sees
    let c = frame(&mut surface, &theme);
    assert_eq!(bound(&c, &slider), left);
    let x = along(left, 0.75, &theme);
    surface.handle_event(&WidgetEvent::MouseMove { x, y }, &theme, &mut app);
    assert!((volume(&app) - 0.75).abs() < 1e-3, "{}", volume(&app));
    let (c, frames) = settle(&mut surface, AREA, &theme);
    assert!(frames > 1, "it slid over {frames} frames");
    let right = bound(&c, &slider);
    assert!(inside(right, bound(&c, &id("mix_panel.right"))));
    assert_ne!(left, right);
    // the button is still down: the drag goes on over the new rect
    let y = right[1] + right[3] / 2.0;
    let x = along(right, 0.9, &theme);
    surface.handle_event(&WidgetEvent::MouseMove { x, y }, &theme, &mut app);
    assert!((volume(&app) - 0.9).abs() < 1e-3, "{}", volume(&app));
    let set = app.executed.last().expect("set_value ran");
    assert_eq!(set.target, id("mixer.volume"));
    // released, the pointer moves the value no more
    surface.handle_event(&WidgetEvent::MouseUp { x, y }, &theme, &mut app);
    let x = along(right, 0.1, &theme);
    surface.handle_event(&WidgetEvent::MouseMove { x, y }, &theme, &mut app);
    assert!((volume(&app) - 0.9).abs() < 1e-3, "{}", volume(&app));
}
