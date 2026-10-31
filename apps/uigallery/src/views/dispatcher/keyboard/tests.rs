//! The keyboard over the context menu (task-75 phase 2c): the menu the
//! pointer opens is walked with the arrows (they wrap, and skip the
//! separator; a disabled item, the text field's menu's, in
//! `tests/component_pages/field_menu.rs`), Home and End, and Tab stays in it;
//! the one highlight follows the focus; Enter picks the item through the
//! dispatcher, as a click does.

use crate::views::{ActiveOverlay, EditKey, NavKey, ShowcaseView};

fn focused(view: &ShowcaseView) -> Option<String> {
    view.semantics()
        .and_then(|s| s.focused().map(ToString::to_string))
}

/// The entry the menu highlights, as the pointer's hover does.
fn highlighted(view: &ShowcaseView) -> Option<usize> {
    match &view.active_overlay {
        Some(ActiveOverlay::Menu { widget, .. }) => widget.hovered(),
        _ => None,
    }
}

#[test]
fn the_arrows_walk_the_context_menu_and_enter_picks() {
    let mut view = ShowcaseView::new(1280.0, 800.0);
    view.jump_to_page("context menu");
    let area = view.panes.context_menu.area(view.page_rect(), &view.theme);
    assert!(view.handle_right_click(area.x + area.w / 2.0, area.y + area.h / 2.0));
    assert_eq!(focused(&view).as_deref(), Some("context_menu[1]"));
    // The kit's file menu: 1, 2, 3, a separator, 4, 5, 6.
    let walk = [
        (NavKey::Down, "context_menu[2]", 1),
        (NavKey::Down, "context_menu[3]", 2),
        (NavKey::Down, "context_menu[4]", 4),
        (NavKey::Down, "context_menu[5]", 5),
        (NavKey::Down, "context_menu[6]", 6),
        (NavKey::Down, "context_menu[1]", 0),
        (NavKey::Up, "context_menu[6]", 6),
    ];
    for (key, at, entry) in walk {
        assert!(view.handle_nav(key), "{key:?}");
        assert_eq!(focused(&view).as_deref(), Some(at), "{key:?}");
        assert_eq!(
            highlighted(&view),
            Some(entry),
            "{at}: the highlight follows"
        );
    }
    assert!(view.handle_edit_key(EditKey::Home));
    assert_eq!(focused(&view).as_deref(), Some("context_menu[1]"));
    assert!(view.handle_edit_key(EditKey::End));
    assert_eq!(focused(&view).as_deref(), Some("context_menu[6]"));
    assert!(view.handle_edit_key(EditKey::Tab));
    assert_eq!(
        focused(&view).as_deref(),
        Some("context_menu[1]"),
        "Tab stays in the menu"
    );
    assert!(view.handle_nav(NavKey::Down));
    assert!(view.handle_enter(), "picks Create new scene...");
    assert!(!view.overlay_holds_pointer(), "picked, it leaves");
    let said = view.semantics().is_some_and(|s| {
        s.nodes()
            .values()
            .any(|n| n.label == "Context menu: Create new scene...")
    });
    assert!(said, "the pick's toast");
}

/// A radio group (task-104 front E): the arrows move the focus along the
/// group's radios, wrapping, and choose each one they land on; Home and
/// End go to the ends. The same on a toggle group's segments.
#[test]
fn the_arrows_choose_along_a_radio_group() {
    let checked = |view: &ShowcaseView, id: &str| {
        view.semantics().and_then(|s| {
            let id = accord::SemId::parse(id).ok()?;
            s.get(&id)?.state.get("checked").cloned()
        })
    };
    for (page, group) in [
        ("Radio Group", "radio_group_page.group[0].card[0]"),
        ("Toggle Group", "toggle_group_page.group[0].card[0]"),
    ] {
        let mut view = ShowcaseView::new(1280.0, 800.0);
        view.jump_to_page(page);
        let at = |i: usize| format!("{group}[{i}]");
        let id = accord::SemId::parse(&at(1)).unwrap();
        assert!(view.assist_focus(&id), "{page}");
        let walk = [
            (EditKey::Right, 2),
            (EditKey::Right, 0),
            (EditKey::Left, 2),
            (EditKey::Home, 0),
            (EditKey::End, 2),
        ];
        for (key, to) in walk {
            assert!(view.handle_edit_key(key), "{page} {key:?}");
            assert_eq!(focused(&view), Some(at(to)), "{page} {key:?}");
            assert_eq!(
                checked(&view, &at(to)),
                Some(accord::Value::Bool(true)),
                "{page} {key:?}: chosen"
            );
        }
        assert!(view.handle_nav(NavKey::Up));
        assert_eq!(focused(&view), Some(at(1)), "{page}: up is back");
    }
}
