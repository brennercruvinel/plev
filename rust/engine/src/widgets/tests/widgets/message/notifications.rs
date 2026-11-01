//! The notifications dropdown of the kit (task-104 front C, wave 3): its
//! tokens, its filters, its list that scrolls past the kit's height, over
//! every width it is given.

use super::*;
use crate::widgets::message::notifications::{Notifications, kit_notifications};
use crate::widgets::message::tokens::NotificationsTokens;
use accord::Value;

fn themes() -> [Theme; 4] {
    [core(), Theme::hoff(), Theme::dark(), Theme::light()]
}

fn dropdown() -> Notifications {
    Notifications::new(kit_notifications()).sem(SemId::parse("n").unwrap())
}

/// The core is the kit verbatim; the others keep its geometry over their
/// own text.
#[test]
fn the_core_is_the_kit_and_the_others_derive_it() {
    let kit = NotificationsTokens::hoff_core();
    assert_eq!(core().message.notifications, kit);
    assert_eq!((kit.w, kit.max_h, kit.row_w), (384.0, 800.0, 386.0));
    for theme in themes() {
        let t = &theme.message.notifications;
        assert_eq!(t.w, kit.w);
        if *t != kit {
            assert_eq!(t.title.color, theme.colors.text);
        }
    }
}

/// Over 200 to 700 px: as wide as the kit's or the column, never taller
/// than the kit's; the title and the filters in the header, apart; every
/// row as wide as the list.
#[test]
fn the_dropdown_holds_over_every_width() {
    for theme in themes() {
        let d = dropdown();
        for w in (200..=700).step_by(4) {
            let b = Rect::new(10.0, 20.0, w as f32, 0.0);
            let l = d.layout(b, &theme);
            let t = &theme.message.notifications;
            assert_eq!(l.card.w, (w as f32).min(t.w));
            assert!(l.card.h <= t.max_h);
            assert!(l.title.right() <= l.filters[0].x + 0.5, "{w}");
            assert!(l.filters[0].right() <= l.filters[1].x + 0.5, "{w}");
            assert!(l.filters[1].right() <= l.card.right(), "{w}");
            assert!(l.list.y >= l.rule.bottom() && l.list.bottom() <= l.card.bottom());
            for (_, r) in d.rows(l.list, &theme) {
                assert_eq!((r.x, r.w), (l.list.x, l.list.w));
            }
        }
    }
}

/// The kit's eight outgrow the card: the wheel scrolls the list to its
/// end and no further, and a row is revealed at the bottom.
#[test]
fn the_list_scrolls_past_the_card() {
    let theme = core();
    let mut d = dropdown();
    let b = Rect::new(0.0, 0.0, 384.0, 0.0);
    let l = d.layout(b, &theme);
    assert_eq!(l.card.h, theme.message.notifications.max_h, "capped");
    assert!(d.content_h(l.list.w, &theme) > l.list.h);
    let (x, y) = l.list.center();
    assert!(d.handle_event(&wheel(x, y, 120.0), b, &theme).changed);
    for _ in 0..100 {
        d.handle_event(&wheel(x, y, 120.0), b, &theme);
    }
    let last = d.rows(l.list, &theme).last().unwrap().1;
    assert!((last.bottom() - l.list.bottom()).abs() < 0.5, "at its end");
    let mut d = dropdown();
    assert!(d.reveal(7, b, &theme));
    let last = d.rows(l.list, &theme).last().unwrap().1;
    assert!(last.bottom() <= l.list.bottom() + 0.5);
}

/// The Unread filter shows the unread, keyed `0..count`; a click on a
/// filter switches it; a row read under it leaves the list.
#[test]
fn the_filter_shows_the_unread_and_keys_them_in_order() {
    let theme = core();
    let mut d = dropdown();
    let b = Rect::new(0.0, 0.0, 384.0, 0.0);
    let l = d.layout(b, &theme);
    let (x, y) = l.filters[1].center();
    d.handle_event(&down(x, y), b, &theme);
    d.handle_event(&up(x, y), b, &theme);
    assert_eq!(d.filter(), 1);
    assert_eq!(d.shown().len(), 5);
    let project = |d: &Notifications| {
        let mut sb = SnapshotBuilder::new();
        d.project(&mut sb, d.sem_id().unwrap(), None);
        sb.finish().expect("the projection validates")
    };
    let seen = project(&d);
    let list = SemId::parse("n.list").unwrap();
    assert_eq!(
        seen.get(&list).unwrap().state.get("count"),
        Some(&Value::Int(5))
    );
    for i in 0..5 {
        assert!(seen.get(&list.key(i.to_string())).is_some(), "row {i}");
    }
    let row = d.rows(l.list, &theme)[0].1;
    let (x, y) = row.center();
    d.handle_event(&down(x, y), b, &theme);
    d.handle_event(&up(x, y), b, &theme);
    assert_eq!(d.shown().len(), 4, "read, it left the list");
    let seen = project(&d);
    assert_eq!(
        seen.get(&list).unwrap().state.get("count"),
        Some(&Value::Int(4))
    );
    assert!(seen.get(&list.key("3")).is_some() && seen.get(&list.key("4")).is_none());
}

/// With nothing to show, the list says so (the empty state, as tall as it
/// needs, inside the dropdown); under Unread with every row read, it says
/// they are all read.
#[test]
fn an_empty_list_shows_its_empty_state() {
    for theme in themes() {
        let mut n = Notifications::new(Vec::new());
        let (w, h) = n.size(400.0, &theme);
        let mut c = Compositor::new();
        c.begin_frame();
        n.render(&mut c, Rect::new(0.0, 0.0, w, h), &theme);
        let nodes = c.layer(LayerId::DEFAULT).unwrap().nodes().to_vec();
        let said = n.empty_state().title;
        let list = n.layout(Rect::new(0.0, 0.0, w, h), &theme).list;
        let drawn = nodes.iter().any(|node| {
            matches!(node, SceneNode::Text { key, y, .. }
            if key.text == said && *y >= list.y && *y < list.bottom())
        });
        assert!(drawn, "'{said}' in the list");
        let all = n.empty_state().title;
        assert!(n.set_filter(1));
        assert_ne!(n.empty_state().title, all, "unread says another thing");
    }
}
