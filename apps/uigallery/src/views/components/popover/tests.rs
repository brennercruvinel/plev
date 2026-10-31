//! The popover page: its ids, its placement over the width range, the
//! forced cards, Escape, the presets.

use super::*;

fn viewport(w: f32) -> Rect {
    Rect::new(0.0, 0.0, w, 800.0)
}

fn inside(inner: Rect, outer: Rect) -> bool {
    const E: f32 = 0.01;
    inner.x >= outer.x - E
        && inner.y >= outer.y - E
        && inner.right() <= outer.right() + E
        && inner.bottom() <= outer.bottom() + E
}

#[test]
fn the_page_s_ids_parse() {
    let page = PopoverPage::new();
    assert!(page.root.is_some());
    for group in &page.groups {
        for card in &group.cards {
            assert!(card.trigger.sem_id().is_some());
            // Only the live card's presets are bound: a forced card's are
            // labels.
            let bound = card.presets.iter().all(|p| p.sem_id().is_some());
            assert_eq!(bound, !group.pinned, "{}", card.title);
        }
    }
}

/// Every width from 280 to 1600 every 8 px, under core and hoff, every
/// card's panel: it stays inside its container (its card, or the visible
/// page for the live one), never covers its trigger, the cards never
/// overlap and never leave the content.
#[test]
fn every_panel_stays_in_its_container_across_the_width_range() {
    for theme in [Theme::hoff_core(), Theme::hoff()] {
        let page = PopoverPage::new();
        let mut w = 280.0;
        while w <= 1600.0 {
            let content = Rect::new(0.0, 0.0, w, 10_000.0);
            let layout = page.layout(content, &theme);
            let cards: Vec<Rect> = layout.iter().flat_map(|g| g.cards.clone()).collect();
            for (i, a) in cards.iter().enumerate() {
                assert!(inside(*a, content), "{w}: card {i} leaves the content");
                for b in &cards[i + 1..] {
                    let apart = a.right() <= b.x
                        || b.right() <= a.x
                        || a.bottom() <= b.y
                        || b.bottom() <= a.y;
                    assert!(apart, "{w}: cards overlap");
                }
            }
            for (g, c, trigger, panel) in page.places(content, viewport(w), &theme) {
                let container = if page.groups[g].pinned {
                    PopoverPage::body(layout[g].cards[c], &theme)
                } else {
                    viewport(w)
                };
                assert!(
                    inside(panel, container),
                    "{w}: panel ({g}, {c}) leaves its container"
                );
                let covers = panel.x < trigger.right()
                    && trigger.x < panel.right()
                    && panel.y < trigger.bottom()
                    && trigger.y < panel.bottom();
                assert!(!covers, "{w}: panel ({g}, {c}) covers its trigger");
                if page.groups[g].pinned {
                    assert!(
                        inside(trigger, container),
                        "{w}: trigger ({g}, {c}) leaves its card"
                    );
                }
            }
            w += 8.0;
        }
    }
}

#[test]
fn the_flip_card_opens_above_and_the_others_below() {
    let theme = Theme::hoff_core();
    let page = PopoverPage::new();
    for (g, c, trigger, panel) in page.places(
        Rect::new(0.0, 0.0, 1200.0, 4000.0),
        viewport(1200.0),
        &theme,
    ) {
        if g != 2 {
            continue;
        }
        if c == 1 {
            assert!(panel.bottom() <= trigger.y, "flipped above");
        } else {
            assert!(panel.y >= trigger.bottom(), "card {c} opens below");
        }
    }
}

#[test]
fn only_the_live_card_opens_and_escape_returns_the_focus() {
    let mut page = PopoverPage::new();
    assert_eq!(page.open_card(), None, "the page loads closed");
    page.press(1, 0);
    page.open(2, 1);
    assert_eq!(page.open_card(), None, "a forced card shows a state");
    page.set_focus(Some(At::Trigger(0, 0)));
    page.press(0, 0);
    assert_eq!(page.open_card(), Some((0, 0)));
    assert_eq!(
        page.focus,
        Some(At::Trigger(0, 0)),
        "opening keeps the focus"
    );
    page.set_focus(Some(At::Preset(0, 0, 2)));
    assert!(page.blur());
    assert_eq!(page.open_card(), None);
    assert_eq!(page.focus, Some(At::Trigger(0, 0)), "back on the trigger");
    assert!(page.blur(), "a second Escape lets the focus go");
    assert_eq!(page.focus, None);
    assert!(!page.blur());
}

#[test]
fn a_preset_sets_the_zoom_the_pill_shows_and_closes_the_panel() {
    let mut page = PopoverPage::new();
    page.press(0, 0);
    page.pick(0, 0, 0);
    assert_eq!(page.zoom, Some(125));
    assert_eq!(page.open_card(), None);
    page.pick(0, 0, 4);
    let Trigger::Kit(pill) = &page.groups[0].cards[0].trigger else {
        panic!("the first card holds the pill");
    };
    assert_eq!(pill.label, "Fit");
    for _ in 0..20 {
        page.pick(0, 0, 1);
    }
    assert_eq!(page.zoom, Some(ZOOM_RANGE.0), "zoom out stops at its limit");
}
