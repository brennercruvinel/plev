//! Sidebar page tests: every sidebar in its card, a reader's focus and
//! actions, the fields that take the keys, the color picker.

use super::*;

fn id(s: &str) -> SemId {
    SemId::parse(s).unwrap()
}

fn seen(page: &SidebarPage) -> accord::Snapshot {
    let root = id("sidebar_page");
    let mut b = SnapshotBuilder::new();
    b.node(&root, Role::Region, "Sidebar");
    page.project(&mut b, &root);
    b.finish().expect("the page projects a valid tree")
}

fn req(page: &SidebarPage, target: &str, action: &str) -> Request {
    Request {
        actor: accord::Actor::Test {
            name: "sidebar".into(),
        },
        target: id(target),
        action: accord::ActionName::parse(action).unwrap(),
        param: None,
        observed: accord::Observation {
            revision: seen(page).revision(),
            values: Vec::new(),
        },
        ask: None,
    }
}

/// Every sidebar fits its card, whole, on a wide page and a phone.
#[test]
fn every_sidebar_sits_inside_its_card() {
    for theme in [Theme::hoff_core(), Theme::hoff()] {
        let page = SidebarPage::new();
        for w in [1200.0, 330.0] {
            let content = Rect::new(0.0, 0.0, w, 800.0);
            for (_, _, card, at) in page.placed(content, &theme) {
                assert!(
                    at.x >= card.x && at.bottom() <= card.bottom(),
                    "{at:?} {card:?}"
                );
            }
        }
    }
}

/// A reader's focus rings the part and is projected back; activate
/// runs it.
#[test]
fn a_readers_focus_and_activate_reach_the_sidebars() {
    let mut page = SidebarPage::new();
    let new = id("sidebar_page.home.new");
    assert!(page.focus(&new));
    assert_eq!(page.focus.as_ref(), Some(&new));
    let r = req(&page, "sidebar_page.home.new", "activate");
    page.run(&r).unwrap();
    assert!(page.home.naming);
    let r = req(&page, "sidebar_page.home.row[1]", "toggle_expanded");
    page.run(&r).unwrap();
    let r = req(&page, "sidebar_page.guideline[1]", "activate");
    page.run(&r).unwrap();
    assert_eq!(page.guideline.current_link(), Some(1));
    assert!(page.blur());
    let r = req(&page, "sidebar_page.nothing", "activate");
    assert!(page.run(&r).is_err());
}

/// A reader's focus on a field gives it the keys; typing and
/// `set_value` reach the artboard's width, Escape leaves it.
#[test]
fn the_sidebars_fields_take_the_keys() {
    let mut page = SidebarPage::new();
    let width = id("sidebar_page.design.block[1].width");
    assert!(page.focus(&width));
    assert!(page.handle_edit_key(EditKey::Backspace));
    assert!(page.handle_text("5"));
    assert_eq!(page.fields()[1].text(), "805");
    let mut r = req(&page, "sidebar_page.scene.search", "set_value");
    r.param = Some(Value::Text("robot".into()));
    page.run(&r).unwrap();
    assert_eq!(page.scene.search.input.text(), "robot");
    assert!(page.handle_escape());
    assert!(!page.handle_text("x"));
    seen(&page);
}

/// The background is the ColorPicker: a reader sets its hex.
#[test]
fn the_background_is_the_color_picker() {
    let mut page = SidebarPage::new();
    let mut r = req(&page, "sidebar_page.design.block[4].color.hex", "set_value");
    r.param = Some(Value::Text("FF0000".into()));
    page.run(&r).unwrap();
    assert_eq!(page.design.picker_mut().unwrap().rgb(), [255, 0, 0]);
    seen(&page);
}
