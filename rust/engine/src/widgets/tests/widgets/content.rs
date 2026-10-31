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
    let row_h =
        theme.control.height(crate::styling::theme::ControlSize::Lg) + theme.spacing.xs * 2.0;
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

// Collapsible and Accordion (task-104d)

use accord::{Role, SemId, SnapshotBuilder};

fn core() -> Theme {
    Theme::hoff_core()
}

/// Lets a motion land: 2 s of frames.
fn settle(c: &mut Collapsible) {
    for _ in 0..120 {
        c.wake(1.0 / 60.0);
    }
}

#[test]
fn disclosure_tokens_derived_from_the_core_palette_are_the_kit() {
    let kit = DisclosureTokens::hoff_core();
    let mut derived = DisclosureTokens::derive(&core());
    // Two kit values sit outside the core palette: the tile's hover
    // border, which the derived theme puts between the line and the dim
    // text, and the add button's white glyph, which it draws in the page
    // tone. Every other value is the kit's.
    let between = derived.item.tile_hover.border.0;
    let (line, dim) = (core().colors.divider.0, core().colors.text_dim.0);
    assert!((0..3).all(|i| between[i] < line[i] && between[i] > dim[i]));
    assert_eq!(derived.item.add_glyph_color, core().colors.bg);
    derived.item.tile_hover.border = kit.item.tile_hover.border;
    derived.item.add_glyph_color = kit.item.add_glyph_color;
    // The rows and the table derive the look before the kit, which the
    // test below holds; only the core draws them in the kit's language.
    assert_ne!(derived.table, kit.table);
    assert_ne!(derived.tree, kit.tree);
    derived.list = kit.list;
    derived.table = kit.table;
    derived.tree = kit.tree;
    assert_ne!((derived.panel, derived.header), (kit.panel, kit.header));
    derived.panel = kit.panel;
    derived.header = kit.header;
    assert_eq!(derived, kit);
    assert_eq!(core().disclosure, kit);
    // The two kit files, spot-checked: the Accordion's `px-4 py-3`, its
    // `p-4 pt-0` body; the Dropdown's `p-0.75 pr-3 rounded-xl`; 500 ms.
    assert_eq!(kit.accordion.pad, [12.0, 16.0, 12.0, 16.0]);
    assert_eq!(kit.accordion.content_pad, [0.0, 16.0, 16.0, 16.0]);
    assert_eq!(kit.collapsible.pad, [3.0, 12.0, 3.0, 3.0]);
    assert_eq!(kit.collapsible.radius, 12.0);
    assert_eq!(kit.motion.reveal, 0.5);
}

/// Every theme but the core draws a list's rows and a table as before the
/// kit, token for token (the urnadbapp's lists and `screen`'s tables keep
/// their pixels): the glass washes under a row, square, the nav radius on
/// its ring; the tabs glass header in the nav radius, caption titles in
/// the faint glass text, the divider under it, `md` padding, body cells.
#[test]
fn rows_and_tables_keep_their_look_outside_the_core() {
    use crate::styling::theme::ControlSize;
    use crate::widgets::nav::PanelHeader;
    for theme in [Theme::hoff(), Theme::dark(), Theme::light()] {
        let (list, table) = (theme.disclosure.list, theme.disclosure.table);
        let g = &theme.glass;
        assert_eq!(list.hover.fill, g.surface_hover);
        assert_eq!(list.selected.fill, g.surface_active);
        assert_eq!((list.radius, list.border, list.rule), (0.0, 0.0, 0.0));
        assert_eq!(list.ring_radius, theme.shape.nav);
        assert_eq!(table.rows, list);
        assert_eq!(table.frame, 0.0);
        assert_eq!(table.header_fill, g.tabs);
        assert_eq!(table.header_radius, theme.shape.nav);
        assert_eq!(table.header_text, g.text_faint);
        assert_eq!(
            (table.rule, table.rule_color),
            (theme.control.edge_width, theme.colors.divider)
        );
        assert_eq!(Table::cell_pad(&theme), theme.spacing.md);
        let tree = theme.disclosure.tree;
        let body = theme.typography.base_2r();
        let gap = theme.spacing.xs;
        assert_eq!(tree.row_h, (body.line_height + gap * 2.0).round());
        assert_eq!(
            (tree.indent, tree.pad_x),
            (theme.spacing.lg, theme.spacing.sm)
        );
        assert_eq!((tree.chevron_gap, tree.label_gap), (gap, gap * 2.0));
        assert_eq!(tree.seam, [gap / 2.0, theme.control.edge_width]);
        assert_eq!(
            (tree.hover.fill, tree.selected.fill),
            (g.surface_hover, g.surface_active)
        );
        assert_eq!(
            (tree.label, tree.label_selected),
            (theme.colors.text_mid, theme.colors.text)
        );
        assert!(tree.tile.is_none() && tree.border == 0.0);
        assert_eq!(Panel::pad(&theme), theme.spacing.lg);
        assert_eq!(Panel::label_style(&theme), theme.typography.caption_sm());
        assert_eq!(theme.disclosure.panel.radius, theme.shape.card);
        assert_eq!(theme.disclosure.panel.label, g.text_faint);
        let xl = theme.control.height(ControlSize::Xl);
        let head = PanelHeader::new("Changes");
        assert_eq!(head.height(&theme), xl);
        let blurb = theme.typography.base_2r().line_height;
        let with = PanelHeader::new("Changes").blurb("a blurb");
        assert_eq!(with.height(&theme), xl + blurb + theme.spacing.xs);
        assert_eq!(PanelHeader::title_style(&theme), theme.typography.title());
        assert_eq!(PanelHeader::pad(&theme), theme.spacing.md);
        let h = theme.disclosure.header;
        assert_eq!((h.title, h.blurb), (g.text_default, theme.colors.text_dim));
        assert_eq!((h.badge_gap, h.rule), (theme.spacing.sm, 0.0));
        assert_eq!(Table::cell_style(&theme), theme.typography.base_2r());
        let mut c = Compositor::new();
        c.begin_frame();
        let mut list_widget = VirtualList::new(24.0);
        list_widget.set_item_count(4);
        list_widget.selected = Some(1);
        list_widget.render_with(
            &mut c,
            Rect::new(0.0, 0.0, 200.0, 96.0),
            &theme,
            |_, _, _, _, _| {},
        );
        let nodes = c.layer(LayerId::DEFAULT).unwrap().nodes().to_vec();
        assert!(
            nodes
                .iter()
                .any(|n| matches!(n, SceneNode::Rect { y, h, color, .. }
            if *y == 24.0 && *h == 24.0 && *color == g.surface_active.0))
        );
    }
}

/// The core draws a table as the kit's spec sheet: a frame, no band, a
/// rule over every row but the first, the lead cell in ink.
#[test]
fn a_core_table_is_the_kit_s_spec_sheet() {
    let theme = core();
    let table = theme.disclosure.table;
    assert_eq!(
        (table.frame, table.frame_color),
        (1.0, theme.colors.divider)
    );
    assert_eq!(table.header_fill.0[3], 0.0);
    assert_eq!(table.cell_pad, 20.0);
    assert_eq!(table.cell, theme.colors.text);
    let mut t = Table::new(vec![Column::new("Style", &theme)], &theme);
    t.set_row_count(3);
    let (row_h, head) = (Table::row_h(&theme), Table::header_h(&theme));
    let mut c = Compositor::new();
    c.begin_frame();
    t.render_with(
        &mut c,
        Rect::new(0.0, 0.0, 300.0, 600.0),
        &theme,
        |_, _, _, _| {},
    );
    let nodes = c.layer(LayerId::DEFAULT).unwrap().nodes().to_vec();
    let rules = nodes
        .iter()
        .filter(|n| {
            matches!(n, SceneNode::Rect { h, color, .. }
            if *h == 1.0 && *color == theme.colors.divider.0)
        })
        .count();
    assert_eq!(rules, 3, "under the header and over rows 1 and 2");
    let framed = nodes.iter().any(|n| {
        matches!(n, SceneNode::RoundedRect { h, border_width, .. }
        if *border_width == 1.0 && (*h - (head + 3.0 * row_h)).abs() < 1e-3)
    });
    assert!(framed, "the frame holds the rows there are");
}

#[test]
fn a_trigger_is_as_tall_as_the_kit_draws_it() {
    let theme = core();
    let accordion = |first| {
        Collapsible::new("3D Objects")
            .look(DisclosureLook::Accordion)
            .first(first)
    };
    // py-3 around the size-6 chevron box; the divider above the others.
    assert_eq!(accordion(true).trigger_height(240.0, &theme), 48.0);
    assert_eq!(accordion(false).trigger_height(240.0, &theme), 49.0);
    // border, p-0.75 around the size-8 tile.
    let dropdown = Collapsible::new("Explore").icon("house");
    assert_eq!(dropdown.trigger_height(240.0, &theme), 40.0);
    // A long title wraps, and the trigger grows.
    let long = Collapsible::new("Materials imported from the shared library of the studio")
        .look(DisclosureLook::Accordion);
    assert!(long.trigger_height(160.0, &theme) > long.trigger_height(1200.0, &theme));
}

#[test]
fn a_collapsible_starts_at_rest_and_opens_to_its_content() {
    let theme = core();
    let mut c = Collapsible::new("Assets").open(true);
    assert_eq!(c.wake(1.0 / 60.0), Wake::idle());
    let trigger = c.trigger_height(240.0, &theme);
    let [top, _, bottom, _] = theme.disclosure.collapsible.content_pad;
    assert_eq!(
        c.height(240.0, 100.0, &theme),
        trigger + top + 100.0 + bottom
    );
    assert!(c.toggle());
    settle(&mut c);
    assert_eq!(c.height(240.0, 100.0, &theme), trigger);
}

#[test]
fn opening_moves_the_height_and_lands() {
    let theme = core();
    let mut c = Collapsible::new("Assets");
    let closed = c.height(240.0, 100.0, &theme);
    assert!(c.toggle() && c.is_open());
    assert_eq!(c.wake(0.1), Wake::continuous());
    let mid = c.height(240.0, 100.0, &theme);
    assert!(mid > closed && c.revealed() < 1.0, "it moves: {mid}");
    for _ in 0..60 {
        c.wake(1.0 / 60.0);
    }
    assert!(!c.is_animating());
    assert_eq!(c.revealed(), 1.0);
    assert_eq!(
        c.wake(1.0 / 60.0),
        Wake::idle(),
        "at rest it asks for nothing"
    );
}

#[test]
fn with_reduced_motion_a_collapsible_opens_at_once() {
    let theme = core();
    let mut c = Collapsible::new("Assets");
    crate::compose::animation::set_reduced_motion(true);
    let toggled = c.toggle();
    let wake = c.wake(1.0 / 60.0);
    crate::compose::animation::set_reduced_motion(false);
    assert!(toggled);
    assert_eq!(c.revealed(), 1.0);
    assert_eq!(wake, Wake::idle());
    assert!(c.height(240.0, 100.0, &theme) > c.trigger_height(240.0, &theme));
}

#[test]
fn a_click_on_the_trigger_flips_it_and_a_drag_away_does_not() {
    let theme = core();
    let mut c = Collapsible::new("Assets");
    let bounds = Rect::new(0.0, 0.0, 240.0, 200.0);
    let (x, y) = c.trigger_rect(bounds, &theme).center();
    assert!(c.handle_event(&move_to(x, y), bounds, &theme).changed);
    c.handle_event(&down(x, y), bounds, &theme);
    assert!(c.handle_event(&up(x, y), bounds, &theme).clicked);
    assert!(c.is_open());
    c.handle_event(&down(x, y), bounds, &theme);
    let r = c.handle_event(&up(x, y + 500.0), bounds, &theme);
    assert!(!r.clicked && c.is_open(), "released outside: nothing");
}

#[test]
fn a_disabled_collapsible_neither_flips_nor_projects_enabled() {
    let theme = core();
    let mut c = Collapsible::new("Assets").disabled(true);
    assert!(!c.toggle() && !c.set_open(true));
    let bounds = Rect::new(0.0, 0.0, 240.0, 200.0);
    let (x, y) = c.trigger_rect(bounds, &theme).center();
    c.handle_event(&down(x, y), bounds, &theme);
    assert_eq!(
        c.handle_event(&up(x, y), bounds, &theme),
        EventResult::IGNORED
    );
    assert!(!c.is_open());
    let id = SemId::parse("d.assets").unwrap();
    let mut b = SnapshotBuilder::new();
    c.project(&mut b, &id, None);
    assert!(!b.finish().unwrap().get(&id).unwrap().enabled);
}

#[test]
fn the_trigger_projects_expanded_and_the_region_only_while_open() {
    let id = SemId::parse("d.assets").unwrap();
    let project = |c: &Collapsible| {
        let mut b = SnapshotBuilder::new();
        c.project(&mut b, &id, None);
        c.project_panel(&mut b, &id, None);
        b.finish().unwrap()
    };
    let mut c = Collapsible::new("Assets").sem(id.clone());
    let closed = project(&c);
    let node = closed.get(&id).unwrap();
    assert_eq!(node.role, Role::Button);
    assert_eq!(
        node.state.get("expanded"),
        Some(&accord::Value::Bool(false))
    );
    assert!(
        node.actions
            .iter()
            .any(|a| a.name.as_str() == "toggle_expanded")
    );
    let panel = id.child("panel").unwrap();
    assert!(closed.get(&panel).is_none());
    c.toggle();
    let open = project(&c);
    assert_eq!(open.get(&panel).map(|n| n.role), Some(Role::Region));
}

/// The trigger binds where it draws; the content draws only while some
/// of it shows, clipped, and the region binds only while open.
#[test]
fn the_render_binds_the_trigger_and_clips_the_content() {
    let theme = core();
    let id = SemId::parse("d.assets").unwrap();
    let panel = id.child("panel").unwrap();
    let draw = |c: &Collapsible| {
        let mut comp = Compositor::new();
        comp.begin_frame();
        let h = c.height(240.0, 80.0, &theme);
        let mut drawn = false;
        c.render(
            &mut comp,
            Rect::new(0.0, 0.0, 240.0, h),
            80.0,
            &theme,
            |_, _| drawn = true,
        );
        (comp, drawn)
    };
    let mut c = Collapsible::new("Assets").sem(id.clone());
    let (comp, drawn) = draw(&c);
    assert!(!drawn && comp.binding(&panel).is_none());
    let trigger = c.trigger_rect(Rect::new(0.0, 0.0, 240.0, 0.0), &theme);
    assert_eq!(
        comp.binding(&id).unwrap().rect,
        [trigger.x, trigger.y, trigger.w, trigger.h]
    );
    c = c.open(true);
    let (comp, drawn) = draw(&c);
    assert!(drawn && comp.binding(&panel).is_some());
    let nodes = comp.layer(LayerId::DEFAULT).unwrap().nodes().to_vec();
    assert!(
        nodes
            .iter()
            .any(|n| matches!(n, SceneNode::PushClip { .. }))
    );
}

#[test]
fn a_single_accordion_keeps_one_open_and_a_multiple_one_many() {
    let theme = core();
    let make = |mode| {
        Accordion::new(mode)
            .item("3D Objects", "meshes")
            .item("Materials", "surfaces")
            .opened(0)
    };
    let mut single = make(AccordionMode::Single);
    assert!(single.toggle(1));
    assert!(single.is_open(1) && !single.is_open(0));
    let mut multiple = make(AccordionMode::Multiple);
    assert!(multiple.toggle(1));
    assert!(multiple.is_open(0) && multiple.is_open(1));
    assert!(!multiple.toggle(7), "no such section");
    // The sections stack, the first without a divider, in the height.
    let bounds = Rect::new(0.0, 0.0, 240.0, multiple.height(240.0, &theme));
    let rects = multiple.item_rects(bounds, &theme);
    assert_eq!(rects[1].y, rects[0].bottom());
    assert_eq!(rects[1].bottom(), bounds.bottom());
}

#[test]
fn a_click_in_a_single_accordion_closes_the_open_section() {
    let theme = core();
    let mut a = Accordion::new(AccordionMode::Single)
        .item("3D Objects", "meshes")
        .item("Materials", "surfaces")
        .opened(0);
    let bounds = Rect::new(0.0, 0.0, 240.0, a.height(240.0, &theme));
    let second = a.item_rects(bounds, &theme)[1];
    let (x, y) = a.items()[1].trigger_rect(second, &theme).center();
    a.handle_event(&down(x, y), bounds, &theme);
    assert!(a.handle_event(&up(x, y), bounds, &theme).clicked);
    assert!(a.is_open(1) && !a.is_open(0));
    assert_eq!(a.wake(1.0 / 60.0), Wake::continuous());
}
