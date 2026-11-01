//! Button page tests: the cards' rows at narrow and wide widths, the
//! pointer at the centered widget, the Icon Button page's groups and the
//! closing Tooltip card.

use super::super::showcase::{card_min_h, card_min_w};
use super::*;

/// Narrow content: rows drop to one column instead of squeezing the
/// cards under the theme's least card width.
#[test]
fn cards_stack_in_narrow_content() {
    let section = ButtonPage::new();
    let content = Rect::new(0.0, 0.0, 500.0, 900.0);
    let layout = section.layout(content, &Theme::hoff_core());
    for group in &layout {
        for r in &group.cards {
            assert_eq!(r.x, content.x);
            assert!(r.w >= card_min_w(&Theme::hoff_core()));
        }
    }
}

/// Wide content: two-column groups sit side by side, three-column
/// groups in threes, and the groups follow each other down the page.
#[test]
fn cards_fill_rows_in_wide_content() {
    let section = ButtonPage::new();
    let content = Rect::new(0.0, 0.0, 1200.0, 900.0);
    let layout = section.layout(content, &Theme::hoff_core());
    let first = &layout[0].cards;
    assert_eq!(first.len(), 2);
    assert_eq!(first[0].y, first[1].y);
    assert!(first[1].x > first[0].x);
    let toolbar = &layout[11].cards;
    assert_eq!(toolbar.len(), 3);
    assert_eq!(toolbar[0].y, toolbar[2].y);
    let mut prev = 0.0;
    for group in &layout {
        assert!(group.title_y > prev);
        prev = group.rule_y;
    }
    let theme = Theme::hoff_core();
    assert!(section.content_height(content, &theme) > 14.0 * card_min_h(&theme));
}

/// Every card's widget gets pointer events at its centered rect.
#[test]
fn hover_reaches_the_centered_widget() {
    let mut section = ButtonPage::new();
    let content = Rect::new(0.0, 0.0, 1200.0, 900.0);
    let theme = Theme::hoff_core();
    let card = section.layout(content, &theme)[0].cards[0];
    let center = (card.x + card.w / 2.0, card.y + card.h / 2.0);
    let r = section.handle_event(
        &WidgetEvent::MouseMove {
            x: center.0,
            y: center.1,
        },
        content,
        &theme,
    );
    assert!(r.changed);
}

/// The Icon Button page is the Button page's icon-only groups: the
/// three icon buttons live and disabled, the toolbar tool at rest,
/// disabled and active, each widget centered in its card.
#[test]
fn the_icon_page_is_the_icon_groups_of_the_buttons_page() {
    let page = ButtonPage::icons();
    let titles: Vec<&str> = page.groups.iter().map(|g| g.title).collect();
    assert_eq!(
        titles,
        [
            "Ghost icon button",
            "Large icon button",
            "Small icon button",
            "Toolbar button"
        ]
    );
    let toolbar = &page.groups[3];
    let states: Vec<&str> = toolbar.cards.iter().map(|c| c.title).collect();
    assert_eq!(states, ["Default, Hover", "Disabled", "Active"]);
    let theme = Theme::hoff_core();
    let content = Rect::new(0.0, 0.0, 1000.0, 800.0);
    for (group, at) in page.groups.iter().zip(page.layout(content, &theme)) {
        for (card, rect) in group.cards.iter().zip(&at.cards) {
            let r = ButtonPage::widget_rect(&card.spec, *rect, &theme);
            let (cy, wy) = (rect.y + rect.h / 2.0, r.y + r.h / 2.0);
            assert!((cy - wy).abs() <= 0.5, "{}: centered", group.title);
        }
    }
    let root = page.root.as_ref().map(|r| r.as_str().to_string());
    assert_eq!(root.as_deref(), Some("icon_button_page"));
}

/// The ButtonsPage ends with its Tooltip: one card, "Text", the
/// bubble "Copy prompt" at its own size, projected as a label (not a
/// button: it has no `activate`).
#[test]
fn the_page_ends_with_the_kit_s_tooltip() {
    let mut page = ButtonPage::new();
    let theme = Theme::hoff_core();
    let last = page.groups.last().expect("groups");
    assert_eq!(last.title, "Tooltip");
    let card = &last.cards[0];
    assert_eq!(card.title, "Text");
    let Spec::Tooltip(tip, Some(id)) = &card.spec else {
        panic!("the tooltip card holds the bubble with its id");
    };
    assert_eq!(card.spec.preferred_size(&theme), tip.size(&theme));
    let id = id.clone();
    let root = page.root.clone().expect("the root parses");
    let mut b = SnapshotBuilder::new();
    b.node(&root, Role::Region, "Button");
    page.project(&mut b, &root);
    let snap = b.finish().expect("a valid tree");
    let node = snap.get(&id).expect("the bubble is projected");
    assert_eq!(
        (node.role, node.label.as_str()),
        (Role::Label, "Copy prompt")
    );
    let req = Request {
        actor: accord::Actor::Test {
            name: "button".into(),
        },
        target: id,
        action: accord::ActionName::parse("activate").expect("an action name"),
        param: None,
        observed: accord::Observation {
            revision: snap.revision(),
            values: Vec::new(),
        },
        ask: None,
    };
    assert!(page.run(&req).is_err(), "a tooltip does not press");
}
