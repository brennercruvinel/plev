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
    assert_eq!(core.overlay, OverlayTokens::hoff_core());
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
    let core = Theme::hoff_core().overlay.tooltip;
    assert_eq!(core.fill, crate::styling::color::Color::hex(0x121212));
    assert_eq!((core.pad_x, core.pad_y, core.radius), (6.0, 4.0, 6.0));
    assert_eq!((core.text.font_size, core.text.line_height), (11.0, 16.0));
    assert_eq!(core.clearance.gap, 18.0, "mb-4.5");
    let hoff = Theme::hoff();
    let t = &hoff.overlay.tooltip;
    assert_eq!(t.fill, hoff.glass.tooltip);
    assert_eq!(t.text, hoff.typography.caption_r());
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
