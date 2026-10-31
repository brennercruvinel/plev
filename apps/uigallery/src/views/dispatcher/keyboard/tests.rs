//! The keyboard over the context menu (task-75 phase 2c): the menu the
//! pointer opens is walked with the arrows (they wrap, and skip the
//! separator and the disabled item), Home and End, and Tab stays in it;
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
    let area = view.page_rect();
    assert!(view.handle_right_click(area.x + 20.0, area.y + 20.0));
    assert_eq!(focused(&view).as_deref(), Some("context_menu[1]"));
    // The entries: 1, 2, 3, a separator, 4 (disabled), 5.
    let walk = [
        (NavKey::Down, "context_menu[2]", 1),
        (NavKey::Down, "context_menu[3]", 2),
        (NavKey::Down, "context_menu[5]", 5),
        (NavKey::Down, "context_menu[1]", 0),
        (NavKey::Up, "context_menu[5]", 5),
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
    assert_eq!(focused(&view).as_deref(), Some("context_menu[5]"));
    assert!(view.handle_edit_key(EditKey::Tab));
    assert_eq!(
        focused(&view).as_deref(),
        Some("context_menu[1]"),
        "Tab stays in the menu"
    );
    assert!(view.handle_nav(NavKey::Down));
    assert!(view.handle_enter(), "picks Copy path");
    assert!(!view.overlay_holds_pointer(), "picked, it leaves");
    let said = view.semantics().is_some_and(|s| {
        s.nodes()
            .values()
            .any(|n| n.label == "Context menu: Copy path")
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

/// A calendar (task-104 front E): left and right move a day, up and down
/// a week, each choosing the day it lands on; at the month's edge up and
/// down stay.
#[test]
fn the_arrows_choose_along_a_calendar_a_day_and_a_week() {
    let card = "calendar_page.group[0].card[0]";
    let at = |day: &str| format!("{card}[2026-10-{day}]");
    let mut view = ShowcaseView::new(1280.0, 800.0);
    view.jump_to_page("Calendar");
    assert!(view.assist_focus(&accord::SemId::parse(&at("14")).unwrap()));
    let selected = |view: &ShowcaseView| {
        view.semantics().and_then(|s| {
            let id = accord::SemId::parse(card).ok()?;
            s.get(&id)?.state.get("selected").cloned()
        })
    };
    let press = |view: &mut ShowcaseView, key: &str| match key {
        "left" => view.handle_edit_key(EditKey::Left),
        "right" => view.handle_edit_key(EditKey::Right),
        "up" => view.handle_nav(NavKey::Up),
        _ => view.handle_nav(NavKey::Down),
    };
    let walk = [
        ("right", "15"),
        ("down", "22"),
        ("up", "15"),
        ("up", "08"),
        ("up", "01"),
        ("left", "31"),
    ];
    for (key, to) in walk {
        assert!(press(&mut view, key), "{key}");
        assert_eq!(focused(&view), Some(at(to)), "{key}");
        assert_eq!(
            selected(&view),
            Some(accord::Value::Text(format!("2026-10-{to}"))),
            "{key}: chosen"
        );
    }
    assert!(!press(&mut view, "down"), "the month's edge");
    assert_eq!(focused(&view), Some(at("31")));
}
