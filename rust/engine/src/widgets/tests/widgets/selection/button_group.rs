//! Selection tests: the button group.

use super::*;

fn tool(name: &str) -> GroupItem {
    GroupItem::Toggle(Toggle::icon_only("eye", name))
}

fn action(label: &str) -> GroupItem {
    GroupItem::Action {
        button: IconButton::new("x").variant(IconButtonVariant::Toolbar),
        label: label.into(),
    }
}

/// The kit's toolbar: three tools, a separator, two actions.
fn toolbar() -> ButtonGroup {
    ButtonGroup::new([
        tool("Select"),
        tool("Move"),
        tool("Crop"),
        GroupItem::Separator,
        action("Undo"),
        action("Redo"),
    ])
}

/// One row in the kit's measure: 40px items, 8px gap and padding, a 1px
/// separator, a 56px shell.
#[test]
fn button_group_lays_one_row_out_as_the_kit() {
    let theme = Theme::hoff_core();
    let g = toolbar();
    let (w, h) = g.preferred_size(&theme);
    assert_eq!(h, 56.0);
    assert_eq!(w, 8.0 * 2.0 + 40.0 * 5.0 + 1.0 + 8.0 * 5.0);
    let rects = g.item_rects(Rect::new(0.0, 0.0, w, h), &theme);
    let sep = rects[3].unwrap();
    assert_eq!((sep.y, sep.h), (0.0, 56.0), "the hairline spans the shell");
    assert!(rects.iter().flatten().all(|r| r.right() <= w - 8.0 + 1e-3));
}

/// Narrower than its row, the group wraps: never past its width, no
/// overlap, a separator never starts a row, and the height grows.
#[test]
fn button_group_wraps_when_the_width_runs_out() {
    let theme = Theme::hoff_core();
    let g = toolbar();
    let one_row = g.preferred_size(&theme);
    for max_w in [g.min_width(&theme), 120.0, 160.0, 200.0, one_row.0 - 1.0] {
        let (w, h) = g.size_within(max_w, &theme);
        assert!(w <= max_w.max(g.min_width(&theme)) + 0.5, "{max_w}: {w}");
        assert!(h > one_row.1, "{max_w}: wraps");
        let bounds = Rect::new(0.0, 0.0, w, h);
        let rects: Vec<Rect> = g.item_rects(bounds, &theme).into_iter().flatten().collect();
        for (i, a) in rects.iter().enumerate() {
            assert!(a.x >= 0.0 && a.right() <= w + 0.5 && a.bottom() <= h + 0.5);
            for b in &rects[i + 1..] {
                let apart =
                    a.right() <= b.x || b.right() <= a.x || a.bottom() <= b.y || b.bottom() <= a.y;
                assert!(apart, "{max_w}: {a:?} {b:?}");
            }
        }
    }
}

/// An exclusive group keeps one tool on: a click on another moves it, a
/// click on the one on keeps it; a free group flips each.
#[test]
fn button_group_exclusive_keeps_one_tool_on() {
    let theme = Theme::hoff_core();
    let mut g = toolbar().exclusive(true);
    let (w, h) = g.preferred_size(&theme);
    let bounds = Rect::new(0.0, 0.0, w, h);
    let on = |g: &ButtonGroup| -> Vec<bool> {
        g.items
            .iter()
            .filter_map(|i| match i {
                GroupItem::Toggle(t) => Some(t.on),
                _ => None,
            })
            .collect()
    };
    let rects = g.item_rects(bounds, &theme);
    let tap = |g: &mut ButtonGroup, i: usize| {
        let (x, y) = rects[i].unwrap().center();
        g.handle_event(&down(x, y), bounds, &theme);
        g.handle_event(&up(x, y), bounds, &theme)
    };
    assert!(tap(&mut g, 1).clicked);
    assert_eq!(on(&g), [false, true, false]);
    tap(&mut g, 1);
    assert_eq!(on(&g), [false, true, false], "the one on stays on");
    assert!(g.press(2));
    assert_eq!(on(&g), [false, false, true]);
    assert!(!g.press(4), "an action holds no state");

    let mut free = toolbar();
    assert!(free.press(0) && free.press(1));
    assert_eq!(on(&free), [true, true, false]);
}

#[test]
fn button_group_projects_a_toolbar_of_its_items() {
    let tb = id("m.tools");
    let mut g = toolbar();
    for (i, item) in g.items.iter_mut().enumerate() {
        let at = tb.child("item").unwrap().key(i.to_string());
        match item {
            GroupItem::Toggle(t) => *t = t.clone().sem(at),
            GroupItem::Action { button, .. } => *button = button.clone().sem(at),
            GroupItem::Separator => {}
        }
    }
    let s = snap(|b| {
        g.project(b, &tb, "tools", None);
    });
    assert_eq!(s.get(&tb).unwrap().role, Role::Toolbar);
    let item = |i: usize| s.get(&tb.child("item").unwrap().key(i.to_string()));
    assert_eq!(
        item(0).map(|n| (n.role, n.label.as_str())),
        Some((Role::Toggle, "Select"))
    );
    assert_eq!(
        item(4).map(|n| (n.role, n.label.as_str())),
        Some((Role::Button, "Undo"))
    );
    assert!(item(3).is_none(), "a separator is not a control");
    assert_eq!(item(0).unwrap().parent.as_ref(), Some(&tb));
}
