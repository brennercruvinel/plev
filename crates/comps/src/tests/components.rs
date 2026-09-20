//! The components that used to live in the apps (avatar, badge,
//! separator, panel header, nav link, sidebar, panel, stat, code block,
//! skeleton, breadcrumb, table) and the app shell: geometry from tokens,
//! content-driven sizes, the pointer contract, and the breakpoint
//! behavior (rail, drawer, dropped columns, collapsed crumbs).

use engine::compositor::{Compositor, LayerId, SceneNode};
use engine::text::TextMeasurer;
use engine::theme::{Breakpoint, ControlSize, Elevation, Intent, SidebarMode, Theme};

use crate::core::{EventResult, Rect, WidgetEvent};
use crate::prelude::*;

fn hoff() -> Theme {
    Theme::hoff()
}

fn nodes(render: impl FnOnce(&mut Compositor)) -> Vec<SceneNode> {
    let mut c = Compositor::new();
    c.begin_frame();
    render(&mut c);
    c.layer(LayerId::DEFAULT).unwrap().nodes().to_vec()
}

fn texts(nodes: &[SceneNode]) -> Vec<String> {
    nodes
        .iter()
        .filter_map(|n| match n {
            SceneNode::Text { key, .. } => Some(key.text.clone()),
            _ => None,
        })
        .collect()
}

fn down(x: f32, y: f32) -> WidgetEvent {
    WidgetEvent::MouseDown { x, y }
}

fn up(x: f32, y: f32) -> WidgetEvent {
    WidgetEvent::MouseUp { x, y }
}

// ---------------------------------------------------------------------------
// Avatar
// ---------------------------------------------------------------------------

#[test]
fn avatar_sizes_follow_the_control_ladder() {
    let t = hoff();
    assert_eq!(
        Avatar::new("ann").preferred_size(&t).0,
        t.control.height(ControlSize::Md)
    );
    assert_eq!(
        Avatar::new("ann").size(AvatarSize::Sm).preferred_size(&t).0,
        t.control.height(ControlSize::Xs)
    );
    assert!(Avatar::new("ann").size(AvatarSize::Lg).preferred_size(&t).0 > 60.0);
}

#[test]
fn avatar_without_photo_draws_the_uppercased_initial() {
    let t = hoff();
    let a = Avatar::new("brenner").online(true);
    let n = nodes(|c| a.render(c, Rect::new(0.0, 0.0, 44.0, 44.0), &t));
    assert_eq!(texts(&n), vec!["B".to_string()]);
    // Disc (glass pill = 2 nodes) + initial + online dot gradient.
    assert!(
        n.iter()
            .any(|n| matches!(n, SceneNode::GradientRect { .. }))
    );
    assert!(Avatar::new("").initial().is_empty());
}

// ---------------------------------------------------------------------------
// Badge
// ---------------------------------------------------------------------------

#[test]
fn badge_count_is_a_disc_that_widens_for_long_numbers() {
    let t = hoff();
    let (w1, h1) = Badge::count("3").preferred_size(&t);
    let (w2, h2) = Badge::count("1200").preferred_size(&t);
    assert_eq!(w1, h1, "a single digit is a circle");
    assert_eq!(h1, h2);
    assert!(w2 > w1, "wide numbers stretch into a pill");
}

#[test]
fn badge_tag_fits_the_measured_label_plus_padding() {
    let t = hoff();
    let b = Badge::tag("MODIFIED");
    let (w, _) = b.preferred_size(&t);
    let (tw, _) = TextMeasurer::measure_styled("MODIFIED", &b.text_style(&t), None);
    assert!((w - (tw + t.spacing.sm * 2.0).ceil()).abs() < 1e-3);
    let n = nodes(|c| b.render(c, Rect::new(0.0, 0.0, 0.0, 0.0), &t));
    assert!(matches!(n[0], SceneNode::RoundedRect { w: rw, .. } if rw == w));
}

#[test]
fn badge_count_takes_the_intent_color() {
    let t = hoff();
    let n = nodes(|c| Badge::count("9").render(c, Rect::default(), &t));
    assert!(matches!(n[0], SceneNode::RoundedRect { color, .. } if color == t.colors.danger.0));
    let n = nodes(|c| {
        Badge::count("9")
            .intent(Intent::Constructive)
            .render(c, Rect::default(), &t)
    });
    assert!(matches!(n[0], SceneNode::RoundedRect { color, .. } if color == t.colors.success.0));
}

// ---------------------------------------------------------------------------
// Separator
// ---------------------------------------------------------------------------

#[test]
fn separator_is_one_rim_thick_and_centered() {
    let t = hoff();
    let b = Rect::new(10.0, 10.0, 100.0, 9.0);
    let h = Separator::Horizontal.line_rect(b, &t);
    assert_eq!(h.h, t.control.edge_width);
    assert_eq!(h.w, 100.0);
    assert_eq!(h.y, 10.0 + (9.0 - t.control.edge_width) / 2.0);
    let v = Separator::Vertical.line_rect(b, &t);
    assert_eq!(v.w, t.control.edge_width);
    assert_eq!(v.h, 9.0);
}

// ---------------------------------------------------------------------------
// Panel
// ---------------------------------------------------------------------------

#[test]
fn panel_content_rect_insets_padding_and_label_band() {
    let t = hoff();
    let b = Rect::new(0.0, 0.0, 300.0, 200.0);
    let plain = Panel::new().content_rect(b, &t);
    assert_eq!(plain, b.inset(Panel::pad(&t)));
    let labeled = Panel::new().label("STATS").content_rect(b, &t);
    assert!(labeled.y > plain.y);
    assert!(labeled.h < plain.h);
}

#[test]
fn raised_panels_cast_the_overlay_stack_and_flat_ones_do_not() {
    let t = hoff();
    let b = Rect::new(0.0, 0.0, 300.0, 200.0);
    let shadows = |p: Panel| {
        nodes(|c| p.render(c, b, &t))
            .iter()
            .filter(|n| matches!(n, SceneNode::Shadow { inset: false, .. }))
            .count()
    };
    assert_eq!(shadows(Panel::new()), 0);
    assert_eq!(shadows(Panel::new().elevation(Elevation::Overlay)), 4);
}

// ---------------------------------------------------------------------------
// Stat
// ---------------------------------------------------------------------------

#[test]
fn stat_height_is_two_ramp_lines_plus_the_gap() {
    let t = hoff();
    let s = Stat::new("38 412", "chunks");
    let (_, h) = s.preferred_size(&t);
    assert_eq!(
        h,
        t.typography.title().line_height + t.spacing.xs + t.typography.caption_r().line_height
    );
    let hero = Stat::new("1.3", "GB").size(StatSize::Lg);
    assert!(hero.preferred_size(&t).1 > h);
}

#[test]
fn stat_with_unit_and_delta_draws_every_part() {
    let t = hoff();
    let s = Stat::new("1.3", "on disk")
        .unit("GB")
        .delta("+12%", Intent::Constructive);
    let n = nodes(|c| s.render(c, Rect::new(0.0, 0.0, 200.0, 60.0), &t));
    let tx = texts(&n);
    assert_eq!(tx, vec!["1.3", "GB", "+12%", "on disk"]);
    // The delta badge is the only rounded rect.
    assert_eq!(
        n.iter()
            .filter(|n| matches!(n, SceneNode::RoundedRect { .. }))
            .count(),
        1
    );
}

// ---------------------------------------------------------------------------
// Code block
// ---------------------------------------------------------------------------

#[test]
fn code_block_height_wraps_against_its_width() {
    let t = hoff();
    let cb = CodeBlock::new("cargo run -p showcase -- cards hoff && cargo test --workspace");
    let wide = cb.height_for(800.0, &t);
    let narrow = cb.height_for(200.0, &t);
    assert!(narrow > wide, "narrow blocks wrap onto more lines");
    let captioned = cb.clone().caption("shell").height_for(800.0, &t);
    assert!(captioned > wide);
}

// ---------------------------------------------------------------------------
// Skeleton
// ---------------------------------------------------------------------------

#[test]
fn skeleton_lines_end_short_and_breathe() {
    let t = hoff();
    let mut sk = Skeleton::new(SkeletonShape::Lines).lines(3);
    let b = Rect::new(0.0, 0.0, 200.0, 100.0);
    let n = nodes(|c| sk.render(c, b, &t));
    let widths: Vec<f32> = n
        .iter()
        .filter_map(|n| match n {
            SceneNode::RoundedRect { w, .. } => Some(*w),
            _ => None,
        })
        .collect();
    assert_eq!(widths.len(), 3);
    assert!(widths[2] < widths[0], "the last line runs short");
    let a = match n[0] {
        SceneNode::RoundedRect { color, .. } => color[3],
        _ => unreachable!(),
    };
    assert!(sk.tick(t.duration.slow));
    let n2 = nodes(|c| sk.render(c, b, &t));
    let a2 = match n2[0] {
        SceneNode::RoundedRect { color, .. } => color[3],
        _ => unreachable!(),
    };
    assert_ne!(a, a2, "the wash breathes over time");
    assert_eq!(
        sk.preferred_height(&t),
        Some(3.0 * Skeleton::line_pitch(&t))
    );
    assert!(
        Skeleton::new(SkeletonShape::Block)
            .preferred_height(&t)
            .is_none()
    );
}

// ---------------------------------------------------------------------------
// Nav link
// ---------------------------------------------------------------------------

#[test]
fn nav_link_is_one_lg_control_and_clicks_on_release() {
    let t = hoff();
    let mut link = NavLink::new("Cards").icon("clipboard").hint("1");
    let b = Rect::new(0.0, 0.0, 200.0, NavLink::height(&t));
    assert_eq!(b.h, t.control.height(ControlSize::Lg));
    assert!(link.preferred_width(&t) > NavLink::icon_slot(&t));
    assert!(!link.handle_event(&down(10.0, 10.0), b).clicked);
    assert!(link.handle_event(&up(10.0, 10.0), b).clicked);
}

#[test]
fn nav_link_draws_only_the_icon_in_a_rail() {
    let t = hoff();
    let link = NavLink::new("Cards").icon("clipboard").hint("1");
    let full = nodes(|c| link.render(c, Rect::new(0.0, 0.0, 220.0, 48.0), &t));
    let rail = nodes(|c| link.render(c, Rect::new(0.0, 0.0, 48.0, 48.0), &t));
    assert_eq!(texts(&full), vec!["1", "Cards"]);
    assert!(texts(&rail).is_empty(), "no label or hint in the rail");
}

#[test]
fn nav_link_tones_climb_rest_hover_active() {
    let t = hoff();
    let b = Rect::new(0.0, 0.0, 220.0, 48.0);
    let label_color = |link: &NavLink| {
        nodes(|c| link.render(c, b, &t))
            .iter()
            .find_map(|n| match n {
                SceneNode::Text { key, color, .. } if key.text == "Cards" => Some(*color),
                _ => None,
            })
            .unwrap()
    };
    let rest = NavLink::new("Cards");
    assert_eq!(label_color(&rest), t.glass.text_faint.0);
    let mut hovered = NavLink::new("Cards");
    hovered.handle_event(&WidgetEvent::MouseMove { x: 5.0, y: 5.0 }, b);
    assert_eq!(label_color(&hovered), t.glass.text_default.0);
    assert_eq!(
        label_color(&NavLink::new("Cards").active(true)),
        t.glass.text_active.0
    );
}

// ---------------------------------------------------------------------------
// Sidebar
// ---------------------------------------------------------------------------

fn sidebar() -> Sidebar {
    Sidebar::new(vec![
        NavLink::new("Cards").icon("clipboard"),
        NavLink::new("Buttons").icon("square"),
        NavLink::new("Forms").icon("settings"),
    ])
    .brand("plev", "HOFF DESIGN SYSTEM")
    .footer(vec!["Esc  close".into()])
}

#[test]
fn sidebar_mode_and_width_follow_the_breakpoint() {
    let t = hoff();
    let sb = sidebar();
    assert_eq!(sb.mode(&t, 400.0), SidebarMode::Drawer);
    assert_eq!(sb.mode(&t, 800.0), SidebarMode::Rail);
    assert_eq!(sb.mode(&t, 1400.0), SidebarMode::Full);
    assert_eq!(sb.page_width(&t, 400.0), 0.0);
    assert_eq!(sb.page_width(&t, 800.0), t.size.sidebar_rail_w);
    assert_eq!(sb.page_width(&t, 1400.0), t.size.sidebar_w);
    // A pinned rail ignores the breakpoint.
    let rail = sidebar().fixed_mode(SidebarMode::Rail);
    assert_eq!(rail.mode(&t, 1400.0), SidebarMode::Rail);
    assert_eq!(rail.page_width(&t, 400.0), t.size.sidebar_rail_w);
}

#[test]
fn sidebar_click_activates_the_link_and_closes_a_drawer() {
    let t = hoff();
    let mut sb = sidebar();
    let (vw, vh) = (1400.0, 900.0);
    let rect = sb.rect(&t, vw, vh).unwrap();
    let rects = sb.link_rects(rect, &t, SidebarMode::Full);
    let (x, y) = rects[2].center();
    sb.handle_event(&down(x, y), &t, vw, vh);
    let (r, nav) = sb.handle_event(&up(x, y), &t, vw, vh);
    assert!(r.clicked);
    assert_eq!(nav, Some(2));
    assert_eq!(sb.active, 2);
    assert!(sb.links[2].active && !sb.links[0].active);

    // Drawer: closed draws nothing; open, a link click closes it.
    let (vw, vh) = (400.0, 800.0);
    assert!(sb.rect(&t, vw, vh).is_none());
    sb.drawer_open = true;
    let rect = sb.rect(&t, vw, vh).unwrap();
    let (x, y) = sb.link_rects(rect, &t, SidebarMode::Drawer)[0].center();
    sb.handle_event(&down(x, y), &t, vw, vh);
    let (_, nav) = sb.handle_event(&up(x, y), &t, vw, vh);
    assert_eq!(nav, Some(0));
    assert!(!sb.drawer_open);
}

#[test]
fn sidebar_links_scroll_in_a_short_window_and_clip_to_the_band() {
    let t = hoff();
    let links: Vec<NavLink> = (0..30).map(|i| NavLink::new(format!("Item {i}"))).collect();
    let mut sb = Sidebar::new(links).brand("plev", "x");
    let (vw, vh) = (1400.0, 400.0);
    let rect = sb.rect(&t, vw, vh).unwrap();
    let band = sb.band(rect, &t, SidebarMode::Full);
    let (bx, by) = band.center();
    let before = sb.link_rects(rect, &t, SidebarMode::Full)[0].y;
    let (r, _) = sb.handle_event(
        &WidgetEvent::Scroll {
            x: bx,
            y: by,
            delta: 200.0,
        },
        &t,
        vw,
        vh,
    );
    assert!(r.changed);
    let after = sb.link_rects(rect, &t, SidebarMode::Full)[0].y;
    assert!(after < before, "the band scrolled");
    let mut c = Compositor::new();
    c.begin_frame();
    sb.render(&mut c, LayerId::DEFAULT, &t, vw, vh);
    let n = c.layer(LayerId::DEFAULT).unwrap().nodes().to_vec();
    assert!(n.iter().any(|n| matches!(n, SceneNode::PushClip { .. })));
}

#[test]
fn sidebar_rail_draws_no_brand_or_footer_text() {
    let t = hoff();
    let mut sb = sidebar();
    let mut c = Compositor::new();
    c.begin_frame();
    sb.render(&mut c, LayerId::DEFAULT, &t, 800.0, 600.0);
    let n = c.layer(LayerId::DEFAULT).unwrap().nodes().to_vec();
    assert!(texts(&n).is_empty(), "rail: icons only");
    let mut c = Compositor::new();
    c.begin_frame();
    sb.render(&mut c, LayerId::DEFAULT, &t, 1400.0, 600.0);
    let n = c.layer(LayerId::DEFAULT).unwrap().nodes().to_vec();
    assert!(texts(&n).contains(&"plev".to_string()));
}

// ---------------------------------------------------------------------------
// Panel header
// ---------------------------------------------------------------------------

#[test]
fn panel_header_height_grows_with_the_blurb_and_reserves_the_badge() {
    let t = hoff();
    let plain = PanelHeader::new("Cards");
    assert_eq!(plain.height(&t), t.control.height(ControlSize::Xl));
    let blurbed = PanelHeader::new("Cards").blurb("the deck");
    assert!(blurbed.height(&t) > plain.height(&t));
    let b = Rect::new(0.0, 0.0, 600.0, 60.0);
    let with_badge = PanelHeader::new("Changes").badge(Badge::tag("12"));
    assert!(with_badge.trailing_rect(b, &t).w < plain.trailing_rect(b, &t).w);
    let n = nodes(|c| with_badge.render(c, b, &t));
    assert_eq!(texts(&n), vec!["12", "Changes"]);
}

// ---------------------------------------------------------------------------
// Breadcrumb
// ---------------------------------------------------------------------------

#[test]
fn breadcrumb_collapses_leading_crumbs_when_narrow() {
    let t = hoff();
    let bc = Breadcrumb::new(["crates", "engine", "src", "theme", "scales.rs"]);
    assert_eq!(
        bc.visible(2000.0, &t),
        vec![Some(0), Some(1), Some(2), Some(3), Some(4)]
    );
    let narrow = bc.visible(220.0, &t);
    assert!(narrow.contains(&None), "an ellipsis stands in");
    assert_eq!(
        *narrow.last().unwrap(),
        Some(4),
        "the current place always shows"
    );
    assert_eq!(narrow[0], Some(0), "the root stays while it fits");
}

#[test]
fn breadcrumb_clicks_a_parent_crumb_but_not_the_last() {
    let t = hoff();
    let mut bc = Breadcrumb::new(["home", "docs", "adr"]);
    let b = Rect::new(0.0, 0.0, 600.0, 24.0);
    let rects = bc.crumb_rects(b, &t);
    let (x, y) = rects[1].1.center();
    bc.handle_event(&down(x, y), b, &t);
    let (r, hit) = bc.handle_event(&up(x, y), b, &t);
    assert!(r.clicked);
    assert_eq!(hit, Some(1));
    let (x, y) = rects[2].1.center();
    let (r, hit) = bc.handle_event(&down(x, y), b, &t);
    assert!(
        !r.handled && hit.is_none(),
        "the last crumb is where you are"
    );
}

// ---------------------------------------------------------------------------
// Table
// ---------------------------------------------------------------------------

fn table(t: &Theme) -> Table {
    let mut tb = Table::new(
        vec![
            Column::new("name", t).weight(2.0).min_w(120.0),
            Column::new("size", t)
                .align(Align::End)
                .min_w(80.0)
                .priority(1),
            Column::new("hash", t).min_w(160.0).priority(0),
        ],
        t,
    );
    tb.set_row_count(1_000);
    tb
}

#[test]
fn table_columns_share_free_width_by_weight_and_drop_by_priority() {
    let t = hoff();
    let tb = table(&t);
    let wide = tb.column_layout(1000.0, &t);
    assert_eq!(wide.visible, vec![0, 1, 2]);
    let name_w = wide.spans[0].1;
    let size_w = wide.spans[1].1;
    assert!(name_w > size_w, "weight 2 takes more of the free width");
    let total: f32 = wide.spans.iter().map(|s| s.1).sum::<f32>() + t.spacing.sm * 2.0;
    assert!((total - 1000.0).abs() < 0.5);

    let narrow = tb.column_layout(260.0, &t);
    assert_eq!(narrow.visible, vec![0, 1], "hash (priority 0) drops first");
    let tiny = tb.column_layout(100.0, &t);
    assert_eq!(tiny.visible, vec![0], "the name column never drops");
}

#[test]
fn table_renders_header_and_virtual_rows_only() {
    let t = hoff();
    let mut tb = table(&t);
    let b = Rect::new(0.0, 0.0, 800.0, 300.0);
    let mut c = Compositor::new();
    c.begin_frame();
    let mut cells = 0usize;
    tb.render_with(&mut c, b, &t, |c, row, cell, col| {
        cells += 1;
        Table::draw_cell(
            c,
            LayerId::DEFAULT,
            cell,
            &format!("r{row}c{col}"),
            Align::Start,
            t.colors.text.0,
            &t,
        );
    });
    // Fewer than the 1000 rows times 3 columns were drawn.
    assert!(cells > 3 && cells < 60, "drew {cells} cells");
    let n = c.layer(LayerId::DEFAULT).unwrap().nodes().to_vec();
    assert!(texts(&n).contains(&"name".to_string()));
    assert!(texts(&n).contains(&"r0c0".to_string()));
    // A click on a row selects it.
    let rows = Table::rows_rect(b, &t);
    let r = tb.handle_event(&down(rows.x + 10.0, rows.y + 10.0), b, &t);
    assert!(r.clicked);
    assert_eq!(tb.selected(), Some(0));
}

// ---------------------------------------------------------------------------
// App shell
// ---------------------------------------------------------------------------

fn shell(w: f32, h: f32) -> AppShell {
    AppShell::new(sidebar(), PanelHeader::new("Cards").blurb("the deck"), w, h)
}

#[test]
fn shell_layout_follows_the_breakpoint() {
    let t = hoff();
    let full = shell(1400.0, 900.0).layout(&t);
    assert_eq!(full.breakpoint, Breakpoint::Expanded);
    assert_eq!(full.sidebar.unwrap().w, t.size.sidebar_w);
    assert!(full.menu_button.is_none());
    assert!(full.content.x >= t.size.sidebar_w + t.layout.gutter(Breakpoint::Expanded));

    let rail = shell(800.0, 600.0).layout(&t);
    assert_eq!(rail.sidebar_mode, SidebarMode::Rail);
    assert_eq!(rail.sidebar.unwrap().w, t.size.sidebar_rail_w);

    let phone = shell(390.0, 844.0).layout(&t);
    assert_eq!(phone.sidebar_mode, SidebarMode::Drawer);
    assert!(phone.sidebar.is_none());
    assert!(phone.menu_button.is_some());
    assert_eq!(phone.content.x, t.layout.gutter(Breakpoint::Compact));
    assert!(phone.content.w > 0.0 && phone.content.h > 0.0);
}

#[test]
fn shell_safe_area_insets_the_content() {
    let t = hoff();
    let mut s = shell(390.0, 844.0);
    let before = s.layout(&t);
    s.set_safe_area(SafeArea {
        top: 47.0,
        bottom: 34.0,
        ..Default::default()
    });
    let after = s.layout(&t);
    assert_eq!(after.header.y, before.header.y + 47.0);
    assert_eq!(after.content.h, before.content.h - 47.0 - 34.0);
}

#[test]
fn shell_menu_button_opens_the_drawer_and_the_drawer_is_exclusive() {
    let t = hoff();
    let mut s = shell(390.0, 844.0);
    let mb = s.layout(&t).menu_button.unwrap();
    let (x, y) = mb.center();
    s.handle_event(&down(x, y), &t);
    let (r, _) = s.handle_event(&up(x, y), &t);
    assert!(r.clicked);
    assert!(s.sidebar.drawer_open);
    // With the drawer open every event is handled by the chrome.
    let (r, _) = s.handle_event(&WidgetEvent::MouseMove { x: 380.0, y: 700.0 }, &t);
    assert!(r.handled);
    // A press outside the sheet closes it.
    let (r, _) = s.handle_event(&down(380.0, 700.0), &t);
    assert!(r.changed && !s.sidebar.drawer_open);
}

#[test]
fn shell_routes_sidebar_navigation() {
    let t = hoff();
    let mut s = shell(1400.0, 900.0);
    let rect = s.sidebar.rect(&t, 1400.0, 900.0).unwrap();
    let (x, y) = s.sidebar.link_rects(rect, &t, SidebarMode::Full)[1].center();
    s.handle_event(&down(x, y), &t);
    let (r, nav) = s.handle_event(&up(x, y), &t);
    assert!(r.clicked);
    assert_eq!(nav, Some(1));
    // Content-area events pass through untouched.
    let content = s.layout(&t).content;
    let (cx, cy) = content.center();
    let (r, nav) = s.handle_event(&down(cx, cy), &t);
    assert_eq!(r, EventResult::IGNORED);
    assert!(nav.is_none());
    let mut c = Compositor::new();
    c.begin_frame();
    s.render(&mut c, LayerId::DEFAULT, LayerId::DEFAULT, &t);
    let n = c.layer(LayerId::DEFAULT).unwrap().nodes().to_vec();
    assert!(texts(&n).contains(&"Cards".to_string()));
}

#[test]
fn sidebar_footer_links_pin_to_the_foot_and_share_the_active_index() {
    let t = hoff();
    let mut sb = sidebar()
        .footer_links(vec![NavLink::new("Settings").icon("settings")])
        .fixed_mode(SidebarMode::Rail);
    let (vw, vh) = (1400.0, 700.0);
    let rect = sb.rect(&t, vw, vh).unwrap();
    let foot = sb.footer_link_rects(rect, &t)[0];
    assert!(foot.y + foot.h <= rect.y + rect.h);
    assert!(foot.y > sb.band(rect, &t, SidebarMode::Rail).y);
    let (x, y) = foot.center();
    sb.handle_event(&down(x, y), &t, vw, vh);
    let (r, nav) = sb.handle_event(&up(x, y), &t, vw, vh);
    assert!(r.clicked);
    assert_eq!(nav, Some(3), "footer links follow the band links");
    assert!(sb.footer_links[0].active && !sb.links[0].active);
}
