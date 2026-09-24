//! A surface changes form. Before `transform` the only way from one
//! verified form to another was to close the surface and open the other
//! one: the ids came back, the app's state came back through the
//! bindings, and every widget was new, so what lived in a widget (the
//! focus, the caret) was gone.

// shared with the other surface tests: this binary uses the fixtures, the
// page and the app.
#[allow(dead_code)]
mod support;

use comps::form::EditKey;
use comps::{Rect, WidgetEvent};
use engine::compositor::Compositor;
use engine::theme::Theme;
use sem::{SemId, StateRef, Value};
use support::{App, COLUMNS, SHOWCASE, TODO, id, snapshot, verified, verified_local};
use surface::{Surface, SurfaceHost};

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

    assert!(host.close(&panel).is_some());
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
