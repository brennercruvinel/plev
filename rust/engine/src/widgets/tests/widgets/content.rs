//! content widgets: card, tree, virtual list, empty state.

use super::*;

// Card

fn card_nodes(card: &Card, theme: &Theme) -> Vec<SceneNode> {
    let mut c = Compositor::new();
    c.begin_frame();
    let (w, h) = card.preferred_size(&hoff());
    card.render(&mut c, Rect::new(0.0, 0.0, w, h), theme);
    c.layer(LayerId::DEFAULT).unwrap().nodes().to_vec()
}

fn sample_cards() -> Vec<Card> {
    vec![
        Card::new(CardVariant::Stat {
            value: "1,632".into(),
            label: "Clicks".into(),
            delta: Some(("+12.4%".into(), true)),
        }),
        Card::new(CardVariant::Profile {
            name: "Artur".into(),
            username: "@artur".into(),
            bio: "Designs dark glass.".into(),
            action: "Follow".into(),
            online: true,
            avatar: None,
        }),
        Card::new(CardVariant::Media {
            title: "4K Video Streaming".into(),
            caption: "Buffer-free playback".into(),
            badge: Some("4K".into()),
            image: None,
        }),
        Card::new(CardVariant::List {
            title: "Expense Tracker".into(),
            rows: vec![
                CardListRow::new("Starter", "$88.00"),
                CardListRow::new("Pro", "$128.00").active(true),
                CardListRow::new("Sync", "70%").progress(0.7),
            ],
        }),
        Card::new(CardVariant::Chart {
            value: "$408.36".into(),
            label: "Last month".into(),
            groups: vec![(0.4, 0.6), (0.5, 0.85), (0.3, 0.5), (0.6, 0.76)],
            highlight: 1,
        }),
        Card::new(CardVariant::Cta {
            title: "Detailed Analytics".into(),
            body: "Track every click with the comparative toolkit.".into(),
            button: "Discover".into(),
        }),
    ]
}

#[test]
fn card_default_width_is_hoff_368() {
    for card in sample_cards() {
        assert_eq!(card.preferred_size(&hoff()).0, hoff().size.card_w);
        assert!(card.preferred_size(&hoff()).1 > 0.0);
    }
}

#[test]
fn card_deck_shell_is_the_discreet_post_lift() {
    let theme = Theme::hoff();
    let card = &sample_cards()[0]; // Stat: deck shell.
    let nodes = card_nodes(card, &theme);

    // No frost, no drop shadow: the live post card measures
    // `backdrop-filter:none` and `box-shadow:none`. The deck never paints a
    // backdrop blur or an outset (non-inset) shadow.
    assert!(
        !nodes
            .iter()
            .any(|n| matches!(n, SceneNode::BackdropBlur { .. })),
        "deck shell does not frost (content cards aren't real glass)"
    );
    assert!(
        !nodes
            .iter()
            .any(|n| matches!(n, SceneNode::Shadow { inset: false, .. })),
        "deck shell casts no drop shadow"
    );
    // Edge-light underlay first: a soft white gradient fading downward.
    assert!(
        matches!(nodes[0], SceneNode::GradientRect { color, color2, angle_deg, .. }
            if color[3] > 0.0 && color2[3] == 0.0 && angle_deg == 180.0),
        "edge-light underlay fades to transparent"
    );
    // Surface: the .02 white post-card lift at radius 20 (inset 1.0 → 19.0),
    // translucent so the #303030 page reads through as ~#343434.
    match nodes[1] {
        SceneNode::RoundedRect {
            color,
            corner_radius,
            ..
        } => {
            // White (#f8f8f8) at .02 — a whisper lighter than the page.
            assert!((color[0] - 248.0 / 255.0).abs() < 1e-5);
            assert!((color[3] - 0.02).abs() < 1e-5);
            assert_eq!(corner_radius, theme.radius.lg - 1.0);
        }
        ref other => panic!("expected surface RoundedRect, got {other:?}"),
    }
    // Inset key-light closes the stack.
    assert!(
        nodes
            .iter()
            .any(|n| matches!(n, SceneNode::Shadow { inset: true, color, .. }
                if (color[3] - 0.06).abs() < 1e-5)),
        "deck shell carries the inset key-light glint"
    );
}

#[test]
fn card_profile_uses_post_surface_and_hover() {
    let theme = Theme::hoff();
    let mut card = sample_cards().remove(1);
    let (w, h) = card.preferred_size(&hoff());
    let bounds = Rect::new(0.0, 0.0, w, h);

    let nodes = card_nodes(&card, &theme);
    // Same discreet post shell as the deck: no frost, no drop shadow, a .02
    // white surface (edge-light underlay first, surface second).
    assert!(!matches!(nodes[0], SceneNode::Shadow { inset: false, .. }));
    assert!(
        !nodes
            .iter()
            .any(|n| matches!(n, SceneNode::BackdropBlur { .. })),
        "post card does not frost its backdrop"
    );
    assert!(matches!(nodes[1], SceneNode::RoundedRect { color, .. }
        if (color[3] - 0.02).abs() < 1e-5));

    card.handle_event(&move_to(10.0, 10.0), bounds);
    assert!(card.is_hovered());
    let nodes = card_nodes(&card, &theme);
    assert!(
        matches!(nodes[1], SceneNode::RoundedRect { color, .. }
            if (color[3] - 0.05).abs() < 1e-5),
        "hover raises the surface to .05"
    );
}

#[test]
fn card_chart_highlight_is_gradient_with_cap() {
    let theme = Theme::hoff();
    let card = &sample_cards()[4];
    let nodes = card_nodes(card, &theme);
    let wash = theme.glass.wash_hover_alpha;

    let gradients: Vec<_> = nodes
        .iter()
        .filter(|n| {
            matches!(n, SceneNode::GradientRect { angle_deg, color, .. }
                if *angle_deg == 180.0 && (color[3] - wash).abs() < 1e-5)
        })
        .collect();
    assert_eq!(gradients.len(), 1, "exactly one highlighted bar");

    // The floating cap above the highlighted bar: a strong-rim-thick
    // slab in the tertiary text tone.
    let cap_h = theme.control.edge_width_strong;
    let cap_color = theme.colors.text_dim.0;
    assert!(nodes.iter().any(|n| matches!(n,
        SceneNode::RoundedRect { h, color, .. } if *h == cap_h && *color == cap_color)));

    // 4 groups x 2 bars: 7 plain bars + 1 gradient highlight.
    let bar_radius = theme.shape.micro;
    let plain_bars = nodes
        .iter()
        .filter(|n| {
            matches!(n, SceneNode::RoundedRect { corner_radius, h, .. }
                if *corner_radius == bar_radius && *h > cap_h)
        })
        .count();
    assert_eq!(plain_bars, 7);
}

#[test]
fn card_list_marks_active_row_with_strong_border() {
    let theme = Theme::hoff();
    let card = &sample_cards()[3];
    let nodes = card_nodes(card, &theme);
    let row_h = theme.control.height(crate::styling::theme::ControlSize::Lg) + theme.spacing.xs * 2.0;
    let active = theme.colors.border_active.0;
    let soft = theme.glass.edge_soft.0;

    let strong_rows = nodes
        .iter()
        .filter(|n| {
            matches!(n, SceneNode::RoundedRect { h, border_color, .. }
                if *h == row_h && *border_color == active)
        })
        .count();
    let soft_rows = nodes
        .iter()
        .filter(|n| {
            matches!(n, SceneNode::RoundedRect { h, border_color, .. }
                if *h == row_h && *border_color == soft)
        })
        .count();
    assert_eq!(strong_rows, 1, "one active row");
    assert_eq!(soft_rows, 2, "two resting rows");

    // The progress row carries the 90deg gradient fill at the fill height.
    let fill_h = theme.control.progress_fill;
    assert!(nodes.iter().any(|n| matches!(n,
        SceneNode::GradientRect { angle_deg, h, .. } if *angle_deg == 90.0 && *h == fill_h)));
}

#[test]
fn card_click_reports_activation() {
    let mut card = sample_cards().remove(5);
    let (w, h) = card.preferred_size(&hoff());
    let bounds = Rect::new(0.0, 0.0, w, h);
    let r = click(|e| card.handle_event(e, bounds));
    assert!(r.clicked);
}

#[test]
fn card_renders_under_every_builtin_theme() {
    for theme in [Theme::hoff(), Theme::dark(), Theme::light()] {
        for card in sample_cards() {
            let nodes = card_nodes(&card, &theme);
            assert!(nodes.len() > 3, "variant emits a scene");
        }
    }
}

// Tree

fn sample_tree() -> Tree {
    Tree::new(vec![
        TreeNode::branch(
            1,
            "src",
            vec![
                TreeNode::leaf(2, "main.rs"),
                TreeNode::branch(3, "ui", vec![TreeNode::leaf(4, "mod.rs")]),
            ],
        )
        .expanded(true),
        TreeNode::leaf(5, "Cargo.toml"),
    ])
}

#[test]
fn tree_flattens_only_expanded_branches() {
    let tree = sample_tree();
    let rows = tree.visible_rows();
    // src (expanded) -> main.rs, ui (collapsed); Cargo.toml.
    let ids: Vec<u64> = rows.iter().map(|r| r.id).collect();
    assert_eq!(ids, vec![1, 2, 3, 5]);
    assert_eq!(rows[1].depth, 1);
    assert_eq!(rows[3].depth, 0);
}

#[test]
fn tree_click_branch_toggles_expansion() {
    let mut tree = sample_tree();
    let bounds = Rect::new(0.0, 0.0, 300.0, 400.0);
    // Row index 2 is the collapsed "ui" branch.
    let y = 2.0 * tree.row_height(&hoff()) + tree.row_height(&hoff()) / 2.0;
    let r = tree.handle_event(&down(50.0, y), bounds, &hoff());
    assert!(r.clicked);
    let ids: Vec<u64> = tree.visible_rows().iter().map(|r| r.id).collect();
    assert_eq!(ids, vec![1, 2, 3, 4, 5], "ui expanded, mod.rs visible");

    // Collapse the root: only roots remain.
    let r = tree.handle_event(&down(50.0, tree.row_height(&hoff()) / 2.0), bounds, &hoff());
    assert!(r.clicked);
    let ids: Vec<u64> = tree.visible_rows().iter().map(|r| r.id).collect();
    assert_eq!(ids, vec![1, 5]);
}

#[test]
fn tree_click_leaf_selects() {
    let mut tree = sample_tree();
    let bounds = Rect::new(0.0, 0.0, 300.0, 400.0);
    let y = 1.0 * tree.row_height(&hoff()) + tree.row_height(&hoff()) / 2.0;
    let r = tree.handle_event(&down(50.0, y), bounds, &hoff());
    assert!(r.clicked);
    assert_eq!(tree.selected, Some(2));
}

// VirtualList

#[test]
fn virtual_list_visible_range_at_top() {
    let mut list = VirtualList::new(24.0);
    list.set_item_count(10_000);
    list.set_viewport(Rect::new(0.0, 0.0, 300.0, 240.0));
    let range = list.visible_range();
    assert_eq!(range.start, 0);
    // 10 rows fit; +1 partial +2 overscan.
    assert!(range.end >= 11 && range.end <= 13, "end={}", range.end);
}

#[test]
fn virtual_list_visible_range_mid_scroll() {
    let mut list = VirtualList::new(24.0);
    list.set_item_count(10_000);
    list.set_viewport(Rect::new(0.0, 0.0, 300.0, 240.0));
    list.scroll.scroll_to(2400.0); // exactly 100 rows down
    let range = list.visible_range();
    assert!(
        range.start <= 100 && range.start >= 98,
        "start={}",
        range.start
    );
    assert!(range.contains(&100));
    assert!(range.contains(&109));
    assert!(range.end <= 113, "end={}", range.end);
}

#[test]
fn virtual_list_visible_range_clamps_at_bottom() {
    let mut list = VirtualList::new(24.0);
    list.set_item_count(100);
    list.set_viewport(Rect::new(0.0, 0.0, 300.0, 240.0));
    list.scroll.scroll_to(f32::MAX);
    let range = list.visible_range();
    assert_eq!(range.end, 100);
    assert!(range.start < 100);
}

#[test]
fn virtual_list_renders_only_visible_rows() {
    let mut list = VirtualList::new(24.0);
    list.set_item_count(10_000);
    let bounds = Rect::new(0.0, 0.0, 300.0, 240.0);
    let theme = Theme::dark();
    let mut c = Compositor::new();
    let mut rendered = Vec::new();
    list.render_with(&mut c, bounds, &theme, |_, index, _, _, _| {
        rendered.push(index);
    });
    assert!(!rendered.is_empty());
    assert!(rendered.len() < 20, "rendered {} of 10000", rendered.len());
}

#[test]
fn virtual_list_scroll_event_moves_and_wakes_scrollbar() {
    let mut list = VirtualList::new(24.0);
    list.set_item_count(1_000);
    let bounds = Rect::new(0.0, 0.0, 300.0, 240.0);
    let r = list.handle_event(
        &WidgetEvent::Scroll {
            x: 100.0,
            y: 100.0,
            delta: 48.0,
        },
        bounds,
        &hoff(),
    );
    assert!(r.changed);
    assert_eq!(list.scroll.offset(), 48.0);
    assert!(list.scrollbar.opacity() > 0.0 || list.scrollbar.is_animating());
    assert_eq!(list.visible_range().start, 0);
}

#[test]
fn virtual_list_click_selects_item() {
    let mut list = VirtualList::new(24.0);
    list.set_item_count(1_000);
    let bounds = Rect::new(0.0, 0.0, 300.0, 240.0);
    list.handle_event(&down(100.0, 50.0), bounds, &hoff());
    assert_eq!(list.selected, Some(2)); // y=50 / 24 = row 2
}

#[test]
fn virtual_list_empty_has_empty_range() {
    let mut list = VirtualList::new(24.0);
    list.set_viewport(Rect::new(0.0, 0.0, 300.0, 240.0));
    assert_eq!(list.visible_range(), 0..0);
}

// EmptyState

#[test]
fn empty_state_centers_its_stack_and_routes_cta_clicks() {
    let mut es = EmptyState::new("No database", "Open a .nest file to explore it.")
        .icon("folder-open")
        .cta(Button::new("Open"));
    let bounds = Rect::new(0.0, 0.0, 800.0, 600.0);
    // The CTA is centered horizontally; find it by probing the center.
    let cta = es.cta.as_ref().unwrap().preferred_size(&hoff());
    let cx = bounds.x + (bounds.w - cta.0) / 2.0 + 2.0;
    // Probe vertically: scan for the cta's y band.
    let mut fired = false;
    for y in (0..600).step_by(4) {
        let r = click(|e| es.handle_event(e, bounds, &hoff()));
        let _ = (cx, y);
        if r.clicked {
            fired = true;
            break;
        }
        let r1 = es.handle_event(&down(cx, y as f32), bounds, &hoff());
        let r2 = es.handle_event(&up(cx, y as f32), bounds, &hoff());
        if r1.merge(r2).clicked {
            fired = true;
            break;
        }
    }
    assert!(fired, "the CTA is clickable somewhere in the stack");
}

#[test]
fn empty_state_without_cta_is_inert() {
    let mut es = EmptyState::new("Title", "Message");
    let r = click(|e| es.handle_event(e, B, &hoff()));
    assert_eq!(r, EventResult::IGNORED);
}

#[test]
fn empty_state_renders_narrow_and_wide_without_gpu() {
    let theme = Theme::hoff();
    for w in [320.0, 1600.0] {
        let mut c = Compositor::new();
        EmptyState::new(
            "A fairly long empty-state title",
            "A long message that must wrap against the available width rather than run edge to edge in the panel it is given.",
        )
        .icon("file")
        .cta(Button::new("Do the thing"))
        .render(&mut c, Rect::new(0.0, 0.0, w, 600.0), &theme);
    }
}
