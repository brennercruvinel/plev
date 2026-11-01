//! overlay widgets (task-104, front B): the popover's placement over
//! continuous ranges of anchor and container, its entry in the overlay
//! stack, its dismissal, its projection and its tokens.

use super::*;
use crate::widgets::overlay::popover::{Align, Popover, PopoverTrigger, Side, place, render_panel};
use crate::widgets::overlay::tokens::{Clearance, OverlayTokens};
use crate::widgets::overlay::{OverlayKind, OverlayManager};
use accord::{Role, SemId, Snapshot, SnapshotBuilder, Value};

/// The kit's clearance: gap 12, offset 8, no edge.
const KIT: Clearance = Clearance {
    gap: 12.0,
    offset: 8.0,
    edge: 0.0,
};

/// A container 400 x 300 at (20, 40).
const BOX: Rect = Rect {
    x: 20.0,
    y: 40.0,
    w: 400.0,
    h: 300.0,
};

const SIDES: [Side; 4] = [Side::Top, Side::Bottom, Side::Start, Side::End];
const ALIGNS: [Align; 3] = [Align::Start, Align::Center, Align::End];

/// `inner` lies inside `outer`, to a hundredth of a pixel.
fn within(inner: Rect, outer: Rect) -> bool {
    const E: f32 = 0.01;
    inner.x >= outer.x - E
        && inner.y >= outer.y - E
        && inner.right() <= outer.right() + E
        && inner.bottom() <= outer.bottom() + E
}

fn overlaps(a: Rect, b: Rect) -> bool {
    a.x < b.right() && b.x < a.right() && a.y < b.bottom() && b.y < a.bottom()
}

// Placement

/// Every anchor position across the container, every 8 px on both
/// axes, every side and alignment: the panel stays inside, and wherever
/// some side has room for it, it keeps the gap and never covers the
/// anchor.
#[test]
fn popover_placement_stays_inside_and_clear_of_the_anchor_across_the_container() {
    let size = (160.0, 90.0);
    let mut x = BOX.x;
    while x <= BOX.right() - 40.0 {
        let mut y = BOX.y;
        while y <= BOX.bottom() - 24.0 {
            let anchor = Rect::new(x, y, 40.0, 24.0);
            for side in SIDES {
                for align in ALIGNS {
                    let p = place(anchor, size, BOX, side, align, KIT);
                    let at = format!("{side:?} {align:?} at ({x}, {y})");
                    assert!(within(p.rect, BOX), "{at}: {:?} leaves the box", p.rect);
                    if !p.capped {
                        assert!(!overlaps(p.rect, anchor), "{at}: covers the anchor");
                    }
                    assert_eq!(p.flipped, p.side != side, "{at}");
                }
            }
            y += 8.0;
        }
        x += 8.0;
    }
}

#[test]
fn popover_opens_below_its_end_skidded_by_the_offset_when_there_is_room() {
    let anchor = Rect::new(200.0, 60.0, 88.0, 40.0);
    let p = place(anchor, (204.0, 120.0), BOX, Side::Bottom, Align::End, KIT);
    assert_eq!(p.side, Side::Bottom);
    assert!(!p.flipped && !p.capped);
    assert_eq!(p.rect.y, anchor.bottom() + KIT.gap);
    // floating-ui's crossAxis: positive moves toward the end.
    assert_eq!(p.rect.right(), anchor.right() + KIT.offset);
    assert_eq!(p.shift, 0.0);
}

#[test]
fn popover_flips_above_when_below_cannot_hold_it_and_above_can() {
    let anchor = Rect::new(100.0, 260.0, 88.0, 40.0);
    let p = place(anchor, (204.0, 120.0), BOX, Side::Bottom, Align::Start, KIT);
    assert!(p.flipped);
    assert_eq!(p.side, Side::Top);
    assert_eq!(p.rect.bottom(), anchor.y - KIT.gap);
}

#[test]
fn popover_takes_the_roomier_side_and_caps_when_neither_holds_it() {
    let anchor = Rect::new(100.0, 170.0, 88.0, 40.0);
    let p = place(anchor, (204.0, 400.0), BOX, Side::Bottom, Align::Start, KIT);
    assert!(p.capped);
    // Above: 170 - 12 - 40 = 118; below: 340 - 210 - 12 = 118. A tie
    // keeps the side asked.
    assert_eq!(p.side, Side::Bottom);
    assert_eq!(p.rect.h, 118.0);
    assert!(within(p.rect, BOX));
}

#[test]
fn popover_shifts_along_the_cross_axis_at_the_container_s_edges() {
    let start = Rect::new(BOX.x, 60.0, 40.0, 24.0);
    let p = place(start, (204.0, 80.0), BOX, Side::Bottom, Align::End, KIT);
    assert_eq!(p.rect.x, BOX.x, "pushed back in at the start");
    assert!(p.shift > 0.0);
    let end = Rect::new(BOX.right() - 40.0, 60.0, 40.0, 24.0);
    let p = place(end, (204.0, 80.0), BOX, Side::Bottom, Align::Start, KIT);
    assert_eq!(p.rect.right(), BOX.right(), "pushed back in at the end");
    assert!(p.shift < 0.0);
}

#[test]
fn popover_in_a_container_narrower_than_it_takes_the_container_s_width() {
    let narrow = Rect::new(0.0, 0.0, 150.0, 600.0);
    let anchor = Rect::new(10.0, 10.0, 88.0, 40.0);
    let p = place(
        anchor,
        (384.0, 200.0),
        narrow,
        Side::Bottom,
        Align::End,
        KIT,
    );
    assert!(p.capped);
    assert_eq!((p.rect.x, p.rect.w), (0.0, 150.0));
}

#[test]
fn popover_keeps_the_edge_clearance_when_the_theme_asks_for_one() {
    let clear = Clearance { edge: 8.0, ..KIT };
    let anchor = Rect::new(BOX.x, 60.0, 40.0, 24.0);
    let p = place(anchor, (204.0, 80.0), BOX, Side::Bottom, Align::End, clear);
    assert_eq!(p.rect.x, BOX.x + 8.0);
    assert!(within(p.rect, BOX.inset(8.0)));
}

#[test]
fn popover_by_an_anchor_outside_the_container_or_of_no_size_stays_inside() {
    for anchor in [
        Rect::new(-200.0, -200.0, 40.0, 24.0),
        Rect::new(900.0, 900.0, 40.0, 24.0),
        Rect::new(120.0, 120.0, 0.0, 0.0),
    ] {
        for side in SIDES {
            let p = place(anchor, (160.0, 90.0), BOX, side, Align::Center, KIT);
            assert!(within(p.rect, BOX), "{anchor:?} {side:?}: {:?}", p.rect);
        }
    }
}

#[test]
fn popover_on_the_logical_sides_opens_beside_the_anchor() {
    let anchor = Rect::new(150.0, 150.0, 40.0, 24.0);
    let end = place(anchor, (100.0, 60.0), BOX, Side::End, Align::Start, KIT);
    assert_eq!(end.rect.x, anchor.right() + KIT.gap);
    let start = place(anchor, (100.0, 60.0), BOX, Side::Start, Align::Start, KIT);
    assert_eq!(start.rect.right(), anchor.x - KIT.gap);
}

// The stack

fn motion() -> crate::styling::theme::MotionPhysics {
    Theme::hoff_core().intent_motion(Intent::Neutral)
}

/// Ticks the stack until nothing moves.
fn settle(mgr: &mut OverlayManager) {
    for _ in 0..600 {
        if !mgr.tick(1.0 / 60.0) {
            return;
        }
    }
    panic!("the stack never settled");
}

#[test]
fn popover_takes_an_entry_on_the_stack_while_open_and_leaves_others_alone() {
    let mut mgr = OverlayManager::new();
    let other = mgr.push(
        OverlayKind::Tooltip { text: "hi".into() },
        0.0,
        0.0,
        10.0,
        10.0,
    );
    let mut pop = Popover::new();
    let panel = Rect::new(10.0, 10.0, 200.0, 100.0);
    assert!(
        !pop.sync(&mut mgr, panel, &motion()),
        "closed: nothing to do"
    );
    assert!(pop.open());
    assert!(pop.sync(&mut mgr, panel, &motion()));
    assert_eq!(mgr.len(), 2);
    assert!(!pop.sync(&mut mgr, panel, &motion()), "in line already");
    assert!(pop.close());
    assert!(pop.sync(&mut mgr, panel, &motion()));
    assert_eq!(mgr.len(), 2, "it fades out on the stack");
    settle(&mut mgr);
    assert_eq!(mgr.stack.iter().map(|o| o.id).collect::<Vec<_>>(), [other]);
    assert!(pop.sync(&mut mgr, panel, &motion()), "its entry is gone");
    assert!(!pop.sync(&mut mgr, panel, &motion()));
}

#[test]
fn popover_whose_entry_another_hand_popped_reads_closed() {
    let mut mgr = OverlayManager::new();
    let mut pop = Popover::new();
    pop.open();
    pop.sync(&mut mgr, Rect::new(0.0, 0.0, 100.0, 60.0), &motion());
    // Escape on the stack, from outside the popover: popped at once, or
    // its exit begun.
    assert!(mgr.pop().is_some());
    assert!(pop.sync(&mut mgr, Rect::default(), &motion()));
    assert!(!pop.is_open());
    assert!(mgr.is_empty());
    pop.open();
    pop.sync(&mut mgr, Rect::default(), &motion());
    assert!(mgr.pop_animated().is_some());
    assert!(pop.sync(&mut mgr, Rect::default(), &motion()));
    assert!(!pop.is_open());
}

/// The panel fades in and out with its entry (the kit's opacity
/// transition); with reduced motion, it is in on the first frame.
#[test]
fn popover_fades_in_and_out_and_cuts_with_reduced_motion() {
    let mut mgr = OverlayManager::new();
    let mut pop = Popover::new();
    let panel = Rect::new(0.0, 0.0, 100.0, 60.0);
    assert_eq!(pop.shown(&mgr), None);
    pop.open();
    assert_eq!(pop.shown(&mgr), Some(0.0), "asked, not yet on the stack");
    pop.sync(&mut mgr, panel, &motion());
    mgr.tick(1.0 / 60.0);
    let first = pop.shown(&mgr).unwrap();
    assert!(first > 0.0 && first < 1.0, "mid fade: {first}");
    settle(&mut mgr);
    assert_eq!(pop.shown(&mgr), Some(1.0));
    pop.close();
    pop.sync(&mut mgr, panel, &motion());
    mgr.tick(1.0 / 60.0);
    let leaving = pop.shown(&mgr).unwrap();
    assert!(leaving < 1.0, "fading out: {leaving}");
    settle(&mut mgr);
    pop.sync(&mut mgr, panel, &motion());
    assert_eq!(pop.shown(&mgr), None);

    crate::compose::animation::set_reduced_motion(true);
    pop.open();
    pop.sync(&mut mgr, panel, &motion());
    mgr.tick(1.0 / 60.0);
    let reduced = pop.shown(&mgr);
    crate::compose::animation::set_reduced_motion(false);
    assert_eq!(reduced, Some(1.0), "a cut");
}

#[test]
fn popover_closes_on_a_press_outside_and_stays_on_one_inside() {
    let trigger = Rect::new(10.0, 10.0, 88.0, 40.0);
    let panel = Rect::new(10.0, 62.0, 204.0, 100.0);
    let mut pop = Popover::new();
    assert!(!pop.dismiss(&down(300.0, 300.0), trigger, panel), "closed");
    pop.open();
    assert!(
        !pop.dismiss(&down(50.0, 100.0), trigger, panel),
        "inside the panel"
    );
    assert!(
        !pop.dismiss(&down(20.0, 20.0), trigger, panel),
        "on the trigger"
    );
    assert!(
        !pop.dismiss(&move_to(300.0, 300.0), trigger, panel),
        "a move"
    );
    assert!(pop.dismiss(&down(300.0, 300.0), trigger, panel));
    assert!(!pop.is_open());
}

// Projection

fn sid(text: &str) -> SemId {
    SemId::parse(text).unwrap()
}

fn projected(pop: &Popover, trigger: &PopoverTrigger) -> Snapshot {
    let (root, id) = (sid("page"), sid("page.zoom"));
    let mut b = SnapshotBuilder::new();
    b.node(&root, Role::Region, "Popover");
    trigger.project(&mut b, &id, Some(&root));
    if let Some(mut panel) = pop.project(&mut b, &id, "Zoom", Some(&root)) {
        panel.description("presets");
    }
    b.finish().unwrap()
}

#[test]
fn popover_projects_its_trigger_expanded_and_its_panel_while_open() {
    let trigger = PopoverTrigger::new("100%");
    let mut pop = Popover::new();
    let id = sid("page.zoom");
    let panel = Popover::panel_id(&id).unwrap();

    let s = projected(&pop, &trigger);
    let node = s.get(&id).unwrap();
    assert_eq!((node.role, node.label.as_str()), (Role::Button, "100%"));
    assert_eq!(node.state.get("expanded"), Some(&Value::Bool(false)));
    let names: Vec<_> = node.actions.iter().map(|a| a.name.as_str()).collect();
    assert_eq!(names, ["activate"], "the press opens and closes it");
    assert!(s.get(&panel).is_none(), "no panel while closed");

    pop.open();
    let s = projected(&pop, &trigger);
    assert_eq!(
        s.get(&id).unwrap().state.get("expanded"),
        Some(&Value::Bool(true))
    );
    let node = s.get(&panel).unwrap();
    assert_eq!((node.role, node.label.as_str()), (Role::Dialog, "Zoom"));
    assert_eq!(node.parent.as_ref(), Some(&sid("page")));
    assert_eq!(node.description.as_deref(), Some("presets"));
}

#[test]
fn popover_without_its_trigger_declared_projects_nothing() {
    let mut pop = Popover::new();
    pop.open();
    let mut b = SnapshotBuilder::new();
    assert!(pop.project(&mut b, &sid("missing"), "Zoom", None).is_none());
    assert!(b.finish().unwrap().nodes().is_empty());
}

// Trigger and panel

#[test]
fn popover_trigger_is_as_wide_as_its_value_and_at_least_the_kit_s_minimum() {
    let theme = Theme::hoff_core();
    let t = &theme.overlay.popover.trigger;
    let (w, h) = PopoverTrigger::new("%").preferred_size(&theme);
    assert_eq!((w, h), (t.min_w, t.height));
    let long = "a value much longer than the kit's zoom percentage";
    let (w, _) = PopoverTrigger::new(long).preferred_size(&theme);
    assert!(w > t.min_w);
}

#[test]
fn popover_trigger_binds_the_bounds_it_hit_tests_with_and_clicks_on_release() {
    let theme = Theme::hoff_core();
    let id = sid("zoom");
    let mut trigger = PopoverTrigger::new("100%").sem(id.clone());
    let mut c = Compositor::new();
    c.begin_frame();
    trigger.render(&mut c, LayerId::DEFAULT, B, false, &theme);
    let bound = c.binding(&id).expect("bound");
    assert_eq!(bound.rect, [B.x, B.y, B.w, B.h]);
    let (x, y) = B.center();
    trigger.handle_event(&down(x, y), B);
    assert!(trigger.handle_event(&up(x, y), B).clicked);
}

/// Every built-in theme draws the trigger, open and closed, and the
/// panel, at a phone and a desktop width.
#[test]
fn popover_renders_under_every_theme() {
    for name in ["core", "hoff", "hoff-light", "plev", "catppuccin"] {
        let theme = Theme::named(name).expect(name);
        for vw in [360.0, 1280.0] {
            let bounds = Rect::new(0.0, 0.0, vw, 800.0);
            let anchor = Rect::new(vw - 100.0, 20.0, 88.0, 40.0);
            let p = Popover::new().placement(anchor, (204.0, 160.0), bounds, &theme);
            let mut c = Compositor::new();
            c.begin_frame();
            let trigger = PopoverTrigger::new("100%");
            trigger.render(&mut c, LayerId::DEFAULT, anchor, false, &theme);
            trigger.render(&mut c, LayerId::DEFAULT, anchor, true, &theme);
            render_panel(&mut c, LayerId::DEFAULT, p.rect, &theme);
            assert!(within(p.rect, bounds), "{name} at {vw}");
        }
    }
}

// Tokens

#[test]
fn popover_tokens_are_the_kit_s_on_core_and_the_theme_s_elsewhere() {
    let core = Theme::hoff_core();
    assert_eq!(*core.overlay, OverlayTokens::hoff_core());
    let t = &core.overlay.popover;
    assert_eq!((t.radius, t.radius_menu), (20.0, 16.0));
    assert_eq!(t.clearance, KIT);
    assert_eq!(t.widths, [204.0, 384.0]);
    assert_eq!(t.panel.fill[0], crate::styling::color::Color::hex(0xFCFCFC));

    let hoff = Theme::hoff();
    let d = &hoff.overlay.popover;
    assert_eq!(
        d.panel.fill[0], hoff.glass.popover,
        "the context menu's surface"
    );
    assert_eq!(d.radius, hoff.shape.dropdown);
    assert_eq!(d.clearance, KIT, "the kit's geometry");
}

// Tooltip

/// The tooltip is the kit's bubble on core, and keeps every other theme's
/// own (its glass, hint shadow and caption) as it was.
#[test]
fn tooltip_tokens_are_the_kit_s_on_core_and_the_theme_s_elsewhere() {
    let theme = Theme::hoff_core();
    let core = &theme.overlay.tooltip;
    assert_eq!(core.fill, crate::styling::color::Color::hex(0x121212));
    assert_eq!((core.pad_x, core.pad_y, core.radius), (6.0, 4.0, 6.0));
    let text = theme.typography.run(core.text);
    assert_eq!((text.font_size, text.line_height), (11.0, 16.0));
    assert_eq!(core.clearance.gap, 18.0, "mb-4.5");
    let hoff = Theme::hoff();
    let t = &hoff.overlay.tooltip;
    assert_eq!(t.fill, hoff.glass.tooltip);
    assert_eq!(t.text, core.text, "the role is the component's");
    assert_eq!(t.radius, hoff.shape.tooltip);
}

/// The tooltip answers the container it is given, never the window:
/// above its anchor when there is room, below when not, inside the
/// container at every anchor position across it.
#[test]
fn tooltip_stays_inside_its_container_across_it() {
    let theme = Theme::hoff_core();
    let mut x = BOX.x;
    while x <= BOX.right() - 40.0 {
        let mut y = BOX.y;
        while y <= BOX.bottom() - 24.0 {
            let anchor = Rect::new(x, y, 40.0, 24.0);
            let mut t = Tooltip::new("Copy prompt");
            t.set_hover(true, anchor);
            let r = t.placement_in(&theme, BOX);
            assert!(within(r, BOX), "at ({x}, {y}): {r:?}");
            assert!(!overlaps(r, anchor), "at ({x}, {y}): covers its anchor");
            y += 8.0;
        }
        x += 8.0;
    }
    let mut t = Tooltip::new("Copy prompt");
    t.set_hover(true, Rect::new(200.0, 200.0, 40.0, 24.0));
    let above = t.placement_in(&theme, BOX);
    assert_eq!(above.bottom(), 200.0 - 18.0, "mb-4.5 above");
    t.set_hover(true, Rect::new(200.0, BOX.y + 4.0, 40.0, 24.0));
    assert!(
        t.placement_in(&theme, BOX).y >= BOX.y + 28.0,
        "flipped below"
    );
}

// Sheet and drawer

use crate::widgets::feedback::ModalAction;
use crate::widgets::overlay::drawer::{Drawer, DrawerAction};
use crate::widgets::overlay::sheet::{Edge, Sheet, SheetRow, edge_rect, slide};

fn sheet(edge: Edge) -> Sheet {
    Sheet::new("Settings", "Changes apply when you save.", "Save", "Cancel")
        .edge(edge)
        .row(SheetRow::new("Display name", "Sophie Bennett").hint("Shown on comments"))
        .row(SheetRow::new("Language", "English"))
}

fn drawer() -> Drawer {
    Drawer::new("Research")
        .group("Foundations", &[("Colors", "layers"), ("Icons", "house")])
        .group("Components", &[("Buttons", "square")])
        .current(Some(1))
}

/// From 280 to 1600 wide and 240 to 1200 tall, every edge: the panel is
/// inside its container, held to its edge, and leaves a touch target of
/// scrim to close it by; a side panel is the token's extent when it fits.
#[test]
fn edge_panels_hold_their_edge_and_leave_a_touch_target_across_the_range() {
    for theme in [Theme::hoff_core(), Theme::hoff()] {
        let reserve = theme.control.min_touch_target;
        let mut w = 280.0;
        while w <= 1600.0 {
            let h = 240.0 + (w - 280.0) * 0.72;
            let bounds = Rect::new(10.0, 20.0, w, h);
            for edge in Edge::ALL {
                let p = sheet(edge).panel_rect(&theme, bounds);
                assert!(within(p, bounds), "{edge:?} at {w}");
                let free = if edge.is_side() { w - p.w } else { h - p.h };
                assert!(free >= reserve - 0.01, "{edge:?} at {w}: {free}");
                match edge {
                    Edge::Left => assert_eq!(p.x, bounds.x),
                    Edge::Right => assert!((p.right() - bounds.right()).abs() < 0.01),
                    Edge::Top => assert_eq!(p.y, bounds.y),
                    Edge::Bottom => assert!((p.bottom() - bounds.bottom()).abs() < 0.01),
                }
                let extent = theme.overlay.sheet.panel.extent;
                if edge.is_side() && w - reserve >= extent {
                    assert_eq!(p.w, extent, "{edge:?} at {w}");
                }
            }
            let d = drawer().panel_rect(&theme, bounds);
            assert!(within(d, bounds) && d.x == bounds.x, "drawer at {w}");
            w += 8.0;
        }
    }
}

/// A top or bottom sheet is as tall as its content when the container
/// holds it, and the container's width.
#[test]
fn top_and_bottom_sheets_are_as_tall_as_their_content() {
    let theme = Theme::hoff_core();
    let bounds = Rect::new(0.0, 0.0, 800.0, 900.0);
    for edge in [Edge::Top, Edge::Bottom] {
        let s = sheet(edge);
        let p = s.panel_rect(&theme, bounds);
        assert_eq!(p.w, bounds.w);
        assert!((p.h - s.content_h(&theme, bounds.w)).abs() < 0.01);
    }
    let r = edge_rect(Edge::Bottom, 10_000.0, bounds, &theme);
    assert_eq!(r.y, theme.control.min_touch_target, "capped");
}

/// The slide: out of the container at 0, settled at 1, toward its edge.
#[test]
fn edge_panels_slide_from_their_edge() {
    let r = Rect::new(100.0, 0.0, 300.0, 600.0);
    assert_eq!(slide(r, Edge::Right, 1.0), r);
    assert_eq!(slide(r, Edge::Right, 0.0).x, 400.0);
    assert_eq!(slide(r, Edge::Left, 0.0).right(), 100.0);
    assert_eq!(slide(r, Edge::Top, 0.5).y, -300.0);
    assert_eq!(slide(r, Edge::Bottom, 2.0), r, "an overshoot holds");
}

/// The close button, the cancel and a press on the scrim cancel, the
/// confirm confirms; a press inside the panel is swallowed.
#[test]
fn sheet_answers_by_its_buttons_and_cancels_on_the_scrim() {
    let theme = Theme::hoff_core();
    let bounds = Rect::new(0.0, 0.0, 1000.0, 700.0);
    let mut s = sheet(Edge::Right);
    let p = s.panel_rect(&theme, bounds);
    let (a, r) = s.handle_event(&down(10.0, 10.0), &theme, bounds);
    assert_eq!(a, ModalAction::Cancel, "the scrim");
    assert!(r.handled);
    let (a, r) = s.handle_event(&down(p.x + 30.0, p.y + 200.0), &theme, bounds);
    assert_eq!(a, ModalAction::None);
    assert!(r.handled, "it blocks");
    // The confirm is the footer's end; the close the head's.
    let m = &theme.overlay.modal;
    let (bx, by) = (p.right() - m.pad - 4.0, p.bottom() - m.pad - 4.0);
    s.handle_event(&down(bx, by), &theme, bounds);
    let (a, _) = s.handle_event(&up(bx, by), &theme, bounds);
    assert_eq!(a, ModalAction::Confirm);
    let t = &theme.overlay.sheet;
    let (cx, cy) = (
        p.right() - t.pad_y - 8.0,
        p.y + t.pad_y + theme.typography.run(t.title).line_height / 2.0,
    );
    s.handle_event(&down(cx, cy), &theme, bounds);
    let (a, _) = s.handle_event(&up(cx, cy), &theme, bounds);
    assert_eq!(a, ModalAction::Cancel, "the close button");
}

/// A link picked on release over the link pressed; the close button and
/// the scrim close.
#[test]
fn drawer_picks_a_link_and_closes_on_the_scrim() {
    let theme = Theme::hoff_core();
    let bounds = Rect::new(0.0, 0.0, 900.0, 700.0);
    let mut d = drawer();
    let p = d.panel_rect(&theme, bounds);
    let t = &theme.overlay.drawer;
    // The first link: under the head and the first group's label.
    let y = p.y
        + t.head_pad * 2.0
        + t.head_h
        + t.group_pad * 2.0
        + theme.typography.run(t.group_label).line_height
        + t.item_gap
        + 10.0;
    let x = p.x + t.body_pad + 40.0;
    assert!(
        d.handle_event(&move_to(x, y), &theme, bounds).1.changed,
        "hover"
    );
    d.handle_event(&down(x, y), &theme, bounds);
    assert_eq!(
        d.handle_event(&up(x, y), &theme, bounds).0,
        DrawerAction::Pick(0)
    );
    d.handle_event(&down(x, y), &theme, bounds);
    let away = y + 200.0;
    assert_eq!(
        d.handle_event(&up(x, away), &theme, bounds).0,
        DrawerAction::None
    );
    let (a, _) = d.handle_event(&down(p.right() + 20.0, 300.0), &theme, bounds);
    assert_eq!(a, DrawerAction::Close);
    assert!(d.content_h(&theme) < bounds.h);
}

/// The sheet is a dialog named by its title with its three buttons and
/// its rows; the drawer a dialog of groups of links, the current marked.
#[test]
fn sheet_and_drawer_project_their_dialog() {
    let id = sid("sheet");
    let s = sheet(Edge::Bottom).sem(id.clone());
    let mut b = SnapshotBuilder::new();
    s.project(&mut b, &id, None);
    let snap = b.finish().unwrap();
    let node = snap.get(&id).unwrap();
    assert_eq!((node.role, node.label.as_str()), (Role::Dialog, "Settings"));
    assert_eq!(node.state.get("edge"), Some(&Value::Text("bottom".into())));
    for key in ["close", "cancel", "confirm"] {
        assert_eq!(snap.get(&id.key(key)).unwrap().role, Role::Button);
    }
    let row = snap.get(&id.child("row").unwrap().key("0")).unwrap();
    assert_eq!(row.label, "Display name: Sophie Bennett");

    let id = sid("drawer");
    let d = drawer().sem(id.clone());
    let mut b = SnapshotBuilder::new();
    d.project(&mut b, &id, None);
    let snap = b.finish().unwrap();
    assert_eq!(snap.get(&id).unwrap().role, Role::Dialog);
    let link = snap.get(&id.key("1")).unwrap();
    assert_eq!((link.role, link.label.as_str()), (Role::Link, "Icons"));
    assert_eq!(link.state.get("current"), Some(&Value::Bool(true)));
    let group = id.child("group").unwrap().key("1");
    assert_eq!(
        snap.get(&id.key("2")).unwrap().parent.as_ref(),
        Some(&group)
    );
}

/// Core is the kit's drawer and settings panel; every other theme its
/// own surface with the kit's extents.
#[test]
fn edge_panel_tokens_are_the_kit_s_on_core_and_the_theme_s_elsewhere() {
    use crate::styling::color::Color;
    let theme = Theme::hoff_core();
    let core = &theme.overlay;
    assert_eq!(
        (core.drawer.panel.extent, core.drawer.panel.radius),
        (260.0, 0.0)
    );
    assert_eq!(core.drawer.panel.fill, Color::hex(0xFCFCFC));
    assert_eq!(core.drawer.panel.rule, Color::hex(0xECECEC));
    assert_eq!(
        (core.sheet.panel.extent, core.sheet.panel.radius),
        (384.0, 24.0)
    );
    assert_eq!(
        (
            theme.typography.run(core.sheet.title).font_size,
            core.sheet.row_min_h
        ),
        (18.0, 56.0)
    );
    let hoff = Theme::hoff();
    let d = &hoff.overlay;
    assert_eq!(d.sheet.panel.fill, hoff.glass.popover);
    assert_eq!(d.sheet.panel.scrim, hoff.glass.scrim);
    assert_eq!(d.drawer.panel.extent, 260.0, "the kit's extent");
}

mod menu;
mod tooltip;
