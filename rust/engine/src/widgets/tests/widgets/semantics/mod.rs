//! the semantic side of the interactive widgets: the optional SemId binds
//! the rect the widget draws and hit-tests with, and each projection
//! helper declares role, label, state and actions from the widget the app
//! owns. one file per comps category; the helpers below are shared.

mod buttons;
mod content;
mod feedback;
mod form;
mod nav;
mod roving;

use super::*;
use accord::{
    Actor, Change, ParamSpec, Rejection, Request, Role, SemId, Snapshot, SnapshotBuilder, Value,
    ValueKind, Verdict, check,
};

fn id(text: &str) -> SemId {
    SemId::parse(text).unwrap()
}

fn snap(project: impl FnOnce(&mut SnapshotBuilder)) -> Snapshot {
    let mut b = SnapshotBuilder::new();
    project(&mut b);
    b.finish().unwrap()
}

fn test_actor() -> Actor {
    Actor::Test {
        name: "comps".to_owned(),
    }
}

fn rect_of(r: Rect) -> [f32; 4] {
    [r.x, r.y, r.w, r.h]
}

/// `who` is bound to `rect` on `layer`, and `hit` at the center of `rect`
/// answers `who`.
#[track_caller]
fn assert_bound(c: &Compositor, who: &SemId, rect: Rect, layer: LayerId) {
    let bound = c
        .binding(who)
        .unwrap_or_else(|| panic!("{who} is not bound"));
    assert_eq!((bound.rect, bound.layer), (rect_of(rect), layer), "{who}");
    let (x, y) = rect.center();
    assert_eq!(c.hit(x, y), Some(who), "hit at the center of {who}");
}

/// The names of `who`'s actions, in declaration order.
fn actions(s: &Snapshot, who: &SemId) -> Vec<String> {
    s.get(who)
        .map(|n| {
            n.actions
                .iter()
                .map(|a| a.name.as_str().to_owned())
                .collect()
        })
        .unwrap_or_default()
}

/// The reads of `who`'s `action`, as text.
fn reads(s: &Snapshot, who: &SemId, action: &str) -> Vec<String> {
    s.get(who)
        .and_then(|n| n.actions.iter().find(|a| a.name.as_str() == action))
        .map(|a| a.reads.iter().map(ToString::to_string).collect())
        .unwrap_or_default()
}

/// `req` comes back stale against `now` with one change, from `before`
/// to `after`.
#[track_caller]
fn assert_stale(req: &Request, now: &Snapshot, before: Value, after: Value) {
    let Verdict::Rejected(Rejection::Stale { changes, .. }) = check(req, now) else {
        panic!("expected stale, got {:?}", check(req, now));
    };
    match &changes[..] {
        [
            Change::StateChanged {
                before: b,
                after: a,
                ..
            },
        ] => {
            assert_eq!((b.as_ref(), a.as_ref()), (Some(&before), Some(&after)));
        }
        other => panic!("expected one state change, got {other:?}"),
    }
}

/// Nothing projected moved between `seen` and `now` (hover, press, focus,
/// animation), so `req`, made on `seen`, still holds on `now`.
#[track_caller]
fn assert_visual_only(req: &Request, seen: &Snapshot, now: &Snapshot) {
    assert_eq!(
        now.revision(),
        seen.revision(),
        "{:?}",
        Snapshot::diff(seen, now)
    );
    assert_eq!(check(req, now), Verdict::Accepted);
}

// Checkbox

#[test]
fn checkbox_projects_checkbox_role_checked_state_and_toggle() {
    let agree = id("form.agree");
    let form = id("form");
    let cb = Checkbox::new(true).label("I agree");
    let s = snap(|b| {
        b.node(&form, Role::Group, "form");
        cb.project(b, &agree, "I agree", Some(&form));
    });
    let node = s.get(&agree).unwrap();
    assert_eq!(node.role, Role::Checkbox);
    assert_eq!(node.label, "I agree");
    assert_eq!(node.parent.as_ref(), Some(&form));
    assert!(node.enabled && node.focusable);
    assert_eq!(node.state.get("checked"), Some(&Value::Bool(true)));
    let toggle = &node.actions[0];
    assert_eq!(toggle.name.as_str(), "toggle");
    assert_eq!(toggle.reads.len(), 1);
    assert_eq!(toggle.reads[0].to_string(), "form.agree:checked");
    // The source is the app's projection site, not the helper.
    let source = node.source.as_ref().unwrap();
    assert!(
        source.file.ends_with("mod.rs") && source.file.contains("semantics"),
        "{}",
        source.file
    );
}

#[test]
fn checkbox_toggle_goes_stale_when_the_widget_state_moves() {
    let agree = id("agree");
    let mut cb = Checkbox::new(false);
    let seen = snap(|b| {
        cb.project(b, &agree, "agree", None);
    });
    let req = Request::observe(&seen, test_actor(), &agree, "toggle", None).unwrap();

    // Hover is not projected: it never invalidates the request.
    cb.handle_event(&move_to(20.0, 20.0), B);
    assert!(cb.is_hovered());
    let now = snap(|b| {
        cb.project(b, &agree, "agree", None);
    });
    assert_eq!(now.revision(), seen.revision());
    assert_eq!(check(&req, &now), Verdict::Accepted);

    // The widget the app owns was toggled meanwhile: the request is stale.
    click(|e| cb.handle_event(e, B));
    let now = snap(|b| {
        cb.project(b, &agree, "agree", None);
    });
    let Verdict::Rejected(Rejection::Stale { changes, .. }) = check(&req, &now) else {
        panic!("expected stale");
    };
    assert!(matches!(
        &changes[..],
        [Change::StateChanged {
            before: Some(Value::Bool(false)),
            after: Some(Value::Bool(true)),
            ..
        }]
    ));
}

#[test]
fn disabled_checkbox_projects_disabled_and_refuses_toggle() {
    let agree = id("agree");
    let cb = Checkbox::new(false).disabled(true);
    let s = snap(|b| {
        cb.project(b, &agree, "agree", None);
    });
    assert!(!s.get(&agree).unwrap().enabled);
    let req = Request::observe(&s, test_actor(), &agree, "toggle", None).unwrap();
    assert!(matches!(
        check(&req, &s),
        Verdict::Rejected(Rejection::Disabled { .. })
    ));
}

// TextField

#[test]
fn text_field_projects_editable_text_input_with_set_value() {
    let name = id("form.name");
    let field = TextField::new("your name").with_text("brenner");
    let s = snap(|b| {
        field.project(b, &name, "name", None);
    });
    let node = s.get(&name).unwrap();
    assert_eq!(node.role, Role::TextInput);
    assert_eq!(node.label, "name");
    assert_eq!(node.state.get("text"), Some(&Value::from("brenner")));
    assert!(node.constraints.editable);
    let set = &node.actions[0];
    assert_eq!(set.name.as_str(), "set_value");
    assert_eq!(
        set.param,
        Some(ParamSpec {
            kind: ValueKind::Text,
            constrained: true
        })
    );
    assert_eq!(reads(&s, &name, "set_value"), ["form.name:text"]);

    let ok = Request::observe(&s, test_actor(), &name, "set_value", Some("ann".into())).unwrap();
    assert_eq!(check(&ok, &s), Verdict::Accepted);
    let wrong = Request::observe(
        &s,
        test_actor(),
        &name,
        "set_value",
        Some(Value::Bool(true)),
    )
    .unwrap();
    assert!(matches!(
        check(&wrong, &s),
        Verdict::Rejected(Rejection::InvalidParam { .. })
    ));
}

#[test]
fn text_field_projection_leaves_the_app_room_for_its_own_actions() {
    let name = id("form.name");
    let field = TextField::new("name").with_text("x");
    let s = snap(|b| {
        field
            .project(b, &name, "name", None)
            .action("submit")
            .reads("form.name:text");
    });
    let actions: Vec<&str> = s
        .get(&name)
        .unwrap()
        .actions
        .iter()
        .map(|a| a.name.as_str())
        .collect();
    assert_eq!(actions, ["set_value", "submit"]);
}

#[test]
fn text_field_set_value_goes_stale_when_the_text_changed_and_not_when_the_caret_moved() {
    let name = id("name");
    let mut field = TextField::new("name").with_text("ann");
    let seen = snap(|b| {
        field.project(b, &name, "name", None);
    });
    let req =
        Request::observe(&seen, test_actor(), &name, "set_value", Some("bob".into())).unwrap();

    // Focus and the caret are editing, not state the value reads.
    field.focus();
    assert!(field.edit(EditKey::Home));
    let now = snap(|b| {
        field.project(b, &name, "name", None);
    });
    assert_visual_only(&req, &seen, &now);

    // The person types after the actor looked: its value would overwrite
    // what they typed, so it goes stale.
    assert!(field.insert("x"));
    assert_eq!(field.text(), "xann");
    let now = snap(|b| {
        field.project(b, &name, "name", None);
    });
    assert_stale(&req, &now, Value::from("ann"), Value::from("xann"));
}

#[test]
fn disabled_text_field_refuses_set_value() {
    let name = id("name");
    let field = TextField::new("name").disabled(true);
    let s = snap(|b| {
        field.project(b, &name, "name", None);
    });
    let req = Request::observe(&s, test_actor(), &name, "set_value", Some("a".into())).unwrap();
    assert!(matches!(
        check(&req, &s),
        Verdict::Rejected(Rejection::Disabled { .. })
    ));
}

// Button

#[test]
fn button_projects_its_label_enabled_and_activate() {
    let save = id("form.save");
    let on = Button::new("Save");
    let off = Button::new("Delete").disabled(true);
    let delete = id("form.delete");
    let s = snap(|b| {
        on.project(b, &save, None);
        off.project(b, &delete, None);
    });
    let node = s.get(&save).unwrap();
    assert_eq!(node.role, Role::Button);
    assert_eq!(node.label, "Save");
    assert!(node.enabled);
    assert_eq!(node.actions[0].name.as_str(), "activate");
    assert!(node.actions[0].reads.is_empty());
    assert!(!s.get(&delete).unwrap().enabled);
}

// Binding

#[test]
fn a_widget_with_a_sem_id_binds_the_bounds_it_draws_and_one_without_binds_nothing() {
    let theme = hoff();
    let mut c = Compositor::new();
    let layer = c.create_layer(10);
    c.begin_frame();
    Checkbox::new(false).render(&mut c, B, &theme);
    TextField::new("x").render(&mut c, B, &theme);
    Button::new("Go").render(&mut c, B, &theme);
    assert_eq!(c.bindings().count(), 0, "no SemId, no binding");

    let at = Rect::new(10.0, 60.0, 100.0, 30.0);
    let far = Rect::new(10.0, 110.0, 100.0, 30.0);
    Checkbox::new(false).sem(id("cb")).render(&mut c, B, &theme);
    TextField::new("x")
        .sem(id("field"))
        .render_to_layer(&mut c, layer, at, &theme);
    Button::new("Go")
        .sem(id("go"))
        .render_to_layer(&mut c, LayerId::DEFAULT, far, &theme);

    let cb = c.binding(&id("cb")).unwrap();
    assert_eq!((cb.rect, cb.layer), (rect_of(B), LayerId::DEFAULT));
    let field = c.binding(&id("field")).unwrap();
    assert_eq!((field.rect, field.layer), (rect_of(at), layer));
    let go = c.binding(&id("go")).unwrap();
    assert_eq!((go.rect, go.layer), (rect_of(far), LayerId::DEFAULT));
    let (x, y) = at.center();
    assert_eq!(c.hit(x, y), Some(&id("field")));
}

#[test]
fn the_sem_id_is_kept_by_the_widget() {
    assert_eq!(Button::new("Go").sem_id(), None);
    assert_eq!(
        Checkbox::new(true).sem(id("a.b")).sem_id(),
        Some(&id("a.b"))
    );
    assert_eq!(
        TextField::new("x").sem(id("c[1]")).clone().sem_id(),
        Some(&id("c[1]"))
    );
}
