//! The Field page: its layout over a continuous range of widths, the
//! pointer and the keys on a field, and what a request runs.

use accord::{Actor, Snapshot};

use super::*;

fn themes() -> [Theme; 2] {
    [Theme::hoff_core(), Theme::hoff()]
}

fn content() -> Rect {
    Rect::new(288.0, 80.0, 900.0, 2400.0)
}

fn overlap(a: Rect, b: Rect) -> bool {
    a.x < b.right() && b.x < a.right() && a.y < b.bottom() && b.y < a.bottom()
}

fn snapshot(p: &FieldPage) -> Snapshot {
    let root = p.root().expect("the root parses").clone();
    let mut b = SnapshotBuilder::new();
    b.node(&root, Role::Region, "Field");
    p.project(&mut b, &root);
    b.finish().expect("the page projects")
}

fn ask(p: &FieldPage, target: &SemId, action: &str, param: Option<Value>) -> Request {
    let actor = Actor::Test {
        name: "field".into(),
    };
    match Request::observe(&snapshot(p), actor, target, action, param) {
        Ok(req) => req,
        Err(e) => panic!("{target} {action}: {e}"),
    }
}

fn field(p: &FieldPage, (g, c): (usize, usize)) -> &Field {
    &p.groups[g].cards[c].field
}

/// The center of a field's box, laid out in `content()`.
fn input_center(p: &FieldPage, at: (usize, usize), theme: &Theme) -> (f32, f32) {
    let rect = p.field_rect(at, content(), theme).expect("a card");
    field(p, at).layout(rect, theme).input.center()
}

#[test]
fn every_field_has_its_id() {
    let p = FieldPage::new();
    assert!(p.root.is_some());
    assert!(p.fields().all(|f| f.sem_id().is_some()));
}

/// The width sweep: from 280 to 1600 px every 8 px, in the core and the
/// HOFF themes, with the page's long, translated and right-to-left
/// content. Every card stays inside the content and clear of the other
/// cards, every chip inside its card and over its field, every field
/// inside its card with each of its parts inside the field, and the
/// groups follow each other down the page.
#[test]
fn every_card_and_field_stays_in_place_from_280_to_1600() {
    let p = FieldPage::new();
    for theme in themes() {
        for w in (280..=1600).step_by(8).map(|w| w as f32) {
            let content = Rect::new(0.0, 0.0, w, 800.0);
            let layout = p.layout(content, &theme);
            let mut prev = content.y;
            for (group, laid) in p.groups.iter().zip(&layout) {
                assert!(laid.title_y >= prev, "{w}: {} rises", group.title);
                for (i, ((card, rect), spec)) in laid.cards.iter().zip(&group.cards).enumerate() {
                    let what = format!("{w}: {} / {}", group.title, spec.title);
                    assert!(card.x >= -0.5 && card.right() <= w + 0.5, "{what}: card");
                    assert!(
                        rect.x >= card.x && rect.right() <= card.right() + 0.5,
                        "{what}: field leaves its card's width"
                    );
                    assert!(
                        rect.y >= card.y && rect.bottom() <= card.bottom() + 0.5,
                        "{what}: field leaves its card's height"
                    );
                    let (chip, _) = vitrine::chip(*card, spec.title, &theme);
                    assert!(chip.right() <= card.right(), "{what}: chip");
                    assert!(chip.bottom() <= rect.y, "{what}: chip over the field");
                    let l = spec.field.layout(*rect, &theme);
                    assert!((l.bottom() - rect.bottom()).abs() < 0.5, "{what}: height");
                    let parts = [l.label, l.link, l.message, l.icon, l.check, l.reveal];
                    for r in parts.into_iter().flatten().chain([l.input]) {
                        assert!(
                            r.x >= rect.x - 0.5 && r.right() <= rect.right() + 0.5,
                            "{what}: {r:?} leaves the field"
                        );
                    }
                    for (other, _) in &laid.cards[i + 1..] {
                        assert!(!overlap(*card, *other), "{what}: cards overlap");
                    }
                }
                assert!(laid.rule_y > laid.title_y);
                prev = laid.rule_y;
            }
            assert!(p.content_height(content, &theme) > 0.0);
        }
    }
}

/// Wide, a group's cards share a row; narrow, they stack in one column.
#[test]
fn cards_share_a_row_wide_and_stack_narrow() {
    let p = FieldPage::new();
    let theme = Theme::hoff_core();
    let wide = p.layout(Rect::new(0.0, 0.0, 1200.0, 800.0), &theme);
    let email = &wide[0].cards;
    assert_eq!((email[0].0.y, email[2].0.y), (email[1].0.y, email[1].0.y));
    let narrow = p.layout(Rect::new(0.0, 0.0, 360.0, 800.0), &theme);
    for laid in &narrow {
        for (card, _) in &laid.cards {
            assert_eq!(card.x, 0.0);
        }
    }
}

/// Hover, a click that focuses and places the caret, typing, an editing
/// key, the caret's blink and Escape, on the live email field.
#[test]
fn a_click_focuses_a_field_the_keys_edit_it_and_escape_blurs() {
    let theme = Theme::hoff_core();
    let mut p = FieldPage::new();
    let (x, y) = input_center(&p, (0, 0), &theme);
    let moved = p.handle_event(&WidgetEvent::MouseMove { x, y }, content(), &theme);
    assert!(moved.changed, "hover");
    p.handle_event(&WidgetEvent::MouseDown { x, y }, content(), &theme);
    assert_eq!(p.focused(), Some((0, 0)));
    assert!(p.handle_text("ada@hoff.dev"));
    assert!(p.handle_edit_key(EditKey::Backspace));
    assert_eq!(field(&p, (0, 0)).input.text(), "ada@hoff.de");
    assert!(!p.handle_edit_key(EditKey::Tab), "Tab is the view's");
    assert!(p.wake(0.016).next.is_some(), "the caret's next flip");
    assert!(p.ime_area(content(), &theme).is_some());
    assert!(p.handle_escape());
    assert_eq!(p.focused(), None);
    assert!(!p.handle_text("x"), "blurred: not consumed");
}

/// A disabled field takes no caret, from a click or from the focus.
#[test]
fn a_disabled_field_takes_no_caret() {
    let theme = Theme::hoff_core();
    let mut p = FieldPage::new();
    let (x, y) = input_center(&p, (2, 0), &theme);
    p.handle_event(&WidgetEvent::MouseDown { x, y }, content(), &theme);
    assert_eq!(p.focused(), None);
    p.set_focused(Some((2, 0)));
    assert_eq!(p.focused(), None);
}

/// `set_value` types into a field, the eye's `toggle` reveals the
/// password as a click on the eye does, the link's `activate` changes
/// nothing, and what the page does not have is refused.
#[test]
fn a_request_runs_as_the_click_does() {
    let theme = Theme::hoff_core();
    let mut p = FieldPage::new();
    let email = field(&p, (0, 0)).sem_id().cloned().unwrap();
    let req = ask(&p, &email, "set_value", Some(Value::from("ada@hoff.dev")));
    assert_eq!(p.run(&req), Ok(After::Nothing));
    assert_eq!(field(&p, (0, 0)).input.text(), "ada@hoff.dev");

    let eye = field(&p, (1, 0)).reveal_id().unwrap();
    let req = ask(&p, &eye, "toggle", None);
    assert_eq!(p.run(&req), Ok(After::Nothing));
    assert!(field(&p, (1, 0)).is_revealed());
    let rect = p.field_rect((1, 0), content(), &theme).unwrap();
    let (x, y) = field(&p, (1, 0))
        .layout(rect, &theme)
        .reveal
        .unwrap()
        .center();
    p.handle_event(&WidgetEvent::MouseDown { x, y }, content(), &theme);
    assert!(!field(&p, (1, 0)).is_revealed(), "the click hides it again");

    let link = field(&p, (1, 0)).link_id().unwrap();
    let req = ask(&p, &link, "activate", None);
    assert_eq!(p.run(&req), Ok(After::Nothing));

    let req = ask(&p, &email, "set_value", Some(Value::from("x")));
    let mut other = FieldPage::new();
    other.groups.clear();
    assert!(other.run(&req).is_err(), "no such field");
}

/// The projection: a group per title, each field under it with its
/// label; the passwords secret, their eyes and links beside them; a
/// validated one has no eye; the reader's focus is the caret's field.
#[test]
fn the_page_projects_its_fields_and_the_focus() {
    let mut p = FieldPage::new();
    let snap = snapshot(&p);
    let pw = field(&p, (1, 0)).sem_id().cloned().unwrap();
    let node = snap.get(&pw).unwrap();
    assert_eq!(
        (node.role, node.label.as_str()),
        (Role::TextInput, "Password")
    );
    assert!(node.constraints.secret);
    assert!(snap.get(&field(&p, (1, 0)).reveal_id().unwrap()).is_some());
    assert!(
        snap.get(&field(&p, (1, 1)).reveal_id().unwrap()).is_none(),
        "validated: no eye"
    );
    let error = field(&p, (0, 2)).sem_id().cloned().unwrap();
    let node = snap.get(&error).unwrap();
    assert_eq!(
        node.description.as_deref(),
        Some("This email is already taken.")
    );
    assert!(
        !snap
            .get(&field(&p, (2, 0)).sem_id().cloned().unwrap())
            .unwrap()
            .enabled
    );

    assert!(p.focus(&pw));
    assert_eq!(p.focused(), Some((1, 0)));
    assert_eq!(snapshot(&p).focused(), Some(&pw));
    let eye = field(&p, (1, 0)).reveal_id().unwrap();
    assert!(p.focus(&eye), "the eye takes the focus from the caret");
    assert_eq!(p.focused(), None);
    assert_eq!(field(&p, (1, 0)).part_focus(), Some(FieldPart::Reveal));
    assert_eq!(snapshot(&p).focused(), Some(&eye));
    assert!(p.blur(), "Escape lets the eye go");
    assert_eq!(field(&p, (1, 0)).part_focus(), None);
    assert!(!p.blur());
    assert!(!p.focus(&SemId::parse("input_page.field[0]").unwrap()));
}
