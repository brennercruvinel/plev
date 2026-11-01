use accord::{Role, SemId, SnapshotBuilder, Value};

use super::{DockMode, SidebarDock};
use crate::compose::animation::set_reduced_motion;
use crate::styling::theme::Theme;
use crate::widgets::nav::tokens::DockRecipe;
use crate::widgets::widget::{Rect, WidgetEvent};

const WIDE: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1280.0,
    h: 800.0,
};

fn at(x: f32, y: f32) -> WidgetEvent {
    WidgetEvent::MouseMove { x, y }
}

/// Runs the dock's clock for `secs`, a frame at a time, in a window `w`
/// wide.
fn run_in(dock: &mut SidebarDock, theme: &Theme, w: f32, secs: f32) {
    let dt = 1.0 / 60.0;
    let mut t = 0.0;
    while t < secs {
        dock.wake(dt, theme, w);
        t += dt;
    }
}

fn run(dock: &mut SidebarDock, theme: &Theme, secs: f32) {
    run_in(dock, theme, WIDE.w, secs);
}

/// A dock with an id, motion on (a test that wants it reduced says so).
fn dock() -> SidebarDock {
    set_reduced_motion(false);
    SidebarDock::new().sem(SemId::name("nav").unwrap())
}

/// The kit has no dock: its recipe is the one rule written in the kit's
/// terms, and both read the core theme's scales the same.
#[test]
fn the_core_recipe_is_the_rule_in_the_core_scales() {
    let core = Theme::hoff_core();
    let nav = &core.navigation;
    let rule = DockRecipe::derive(&core, nav.sidebars.width, &nav.link);
    assert_eq!(nav.dock, rule);
    assert_eq!(nav.dock.width, nav.sidebars.width);
    assert_eq!(nav.dock.zone, core.spacing.sm);
    assert_eq!(nav.dock.slide, core.duration.slow);
    assert_eq!(nav.dock.content_min_w, core.size.card_min_w);
    assert_eq!(nav.dock.button, nav.link.icon_box());
    for theme in [Theme::hoff(), Theme::dark(), Theme::light()] {
        let nav = &theme.navigation;
        assert_eq!(
            nav.dock,
            DockRecipe::derive(&theme, nav.sidebars.width, &nav.link)
        );
    }
}

#[test]
fn it_peeks_by_default_out_of_the_window_and_the_page_has_it_all() {
    let theme = Theme::hoff_core();
    let d = dock();
    let l = d.layout(&theme, WIDE);
    assert_eq!(l.mode, DockMode::Peek);
    assert_eq!(l.panel, None);
    assert_eq!(l.page_x, 0.0);
    let zone = l.zone.expect("a peeking panel has a zone");
    assert_eq!(zone.w, theme.navigation.dock.zone);
    assert!(!d.is_expanded());
}

#[test]
fn a_pointer_at_the_edge_slides_it_in_after_the_pause_and_out_after_it_leaves() {
    let theme = Theme::hoff_core();
    let r = theme.navigation.dock;
    let mut d = dock();
    let res = d.pointer(&at(1.0, 400.0), &theme, WIDE);
    assert!(
        !res.handled,
        "the zone is not the panel: the page still sees it"
    );
    assert!(d.is_expanded(), "on its way in");
    assert_eq!(d.shown(), 0.0, "nothing before the pause runs out");
    run(&mut d, &theme, r.enter_delay * 0.5);
    assert_eq!(d.shown(), 0.0);
    run(&mut d, &theme, r.enter_delay + r.slide + 0.05);
    assert_eq!(d.shown(), 1.0);
    let l = d.layout(&theme, WIDE);
    assert_eq!(l.panel, Some(l.rest));
    assert_eq!(l.page_x, 0.0, "it floats over the page");
    // over the panel the events are the panel's
    assert!(d.pointer(&at(r.width / 2.0, 400.0), &theme, WIDE).handled);
    // away, it stays for the pause, then slides out
    d.pointer(&at(r.width + 200.0, 400.0), &theme, WIDE);
    run(&mut d, &theme, r.leave_delay * 0.5);
    assert_eq!(d.shown(), 1.0, "a pointer that overshoots can come back");
    run(&mut d, &theme, r.leave_delay + r.slide + 0.05);
    assert_eq!(d.shown(), 0.0);
    assert!(!d.is_expanded());
}

#[test]
fn a_pointer_that_comes_back_within_the_pause_keeps_it() {
    let theme = Theme::hoff_core();
    let r = theme.navigation.dock;
    let mut d = dock();
    d.pointer(&at(1.0, 400.0), &theme, WIDE);
    run(&mut d, &theme, r.enter_delay + r.slide + 0.05);
    d.pointer(&at(r.width + 10.0, 400.0), &theme, WIDE);
    run(&mut d, &theme, r.leave_delay * 0.5);
    d.pointer(&at(r.width - 10.0, 400.0), &theme, WIDE);
    run(&mut d, &theme, r.leave_delay * 2.0);
    assert_eq!(d.shown(), 1.0);
}

#[test]
fn with_reduced_motion_it_appears_and_vanishes_without_sliding() {
    let theme = Theme::hoff_core();
    let r = theme.navigation.dock;
    let mut d = dock();
    set_reduced_motion(true);
    d.pointer(&at(1.0, 400.0), &theme, WIDE);
    run(&mut d, &theme, r.enter_delay + 1.0 / 60.0);
    assert_eq!(d.shown(), 1.0, "a cut, not a slide");
    d.pointer(&at(r.width + 10.0, 400.0), &theme, WIDE);
    run(&mut d, &theme, r.leave_delay + 1.0 / 60.0);
    assert_eq!(d.shown(), 0.0);
    set_reduced_motion(false);
}

#[test]
fn the_pin_holds_it_beside_the_page_and_lets_it_go() {
    let theme = Theme::hoff_core();
    let r = theme.navigation.dock;
    let mut d = dock();
    assert!(d.toggle_pin(&theme, WIDE.w));
    let l = d.layout(&theme, WIDE);
    assert_eq!(l.mode, DockMode::Pinned);
    assert_eq!(l.page_x, r.width, "the page starts past it");
    assert_eq!(l.panel, Some(l.rest));
    assert!(!l.floats);
    // the pointer on the pin, let go: it stays while the pointer is on it
    d.pointer(&at(r.width - 10.0, 20.0), &theme, WIDE);
    d.toggle_pin(&theme, WIDE.w);
    let l = d.layout(&theme, WIDE);
    assert_eq!(l.mode, DockMode::Peek);
    assert_eq!(l.page_x, 0.0);
    assert_eq!(l.shown, 1.0);
    run(&mut d, &theme, r.leave_delay + r.slide + 0.05);
    assert_eq!(d.shown(), 1.0, "the pointer is still on it");
    assert!(d.is_expanded());
}

#[test]
fn a_pointer_that_only_grazes_the_edge_reveals_nothing() {
    let theme = Theme::hoff_core();
    let r = theme.navigation.dock;
    let mut d = dock();
    d.pointer(&at(1.0, 400.0), &theme, WIDE);
    run(&mut d, &theme, r.enter_delay * 0.5);
    // on past the panel before the pause ran out
    d.pointer(&at(r.width + 40.0, 400.0), &theme, WIDE);
    run(&mut d, &theme, r.enter_delay + r.slide + 0.05);
    assert_eq!(d.shown(), 0.0);
    assert!(!d.is_expanded());
}

#[test]
fn the_focus_walking_in_shows_it_at_once_and_escape_hides_it() {
    let theme = Theme::hoff_core();
    let mut d = dock();
    set_reduced_motion(true);
    d.set_focus_inside(true, &theme, WIDE.w);
    assert!(d.is_expanded());
    assert_eq!(d.shown(), 1.0);
    assert!(d.escape(&theme, WIDE.w));
    assert!(!d.is_expanded());
    set_reduced_motion(false);
}

#[test]
fn a_window_too_narrow_for_the_page_beside_it_makes_it_a_drawer() {
    let theme = Theme::hoff_core();
    let r = theme.navigation.dock;
    let narrow = Rect::new(0.0, 0.0, 360.0, 700.0);
    let mut d = dock().pinned(true);
    let l = d.layout(&theme, narrow);
    assert_eq!(l.mode, DockMode::Drawer, "pinned or not");
    assert!(l.menu && !l.modal && l.panel.is_none());
    assert_eq!(l.page_x, 0.0);
    assert!(d.open_drawer(&theme, narrow.w));
    run_in(&mut d, &theme, narrow.w, r.slide + 0.05);
    let l = d.layout(&theme, narrow);
    assert!(l.modal);
    assert!(l.rest.w <= narrow.w);
    // the page is inert; a press off the panel closes it
    assert!(d.pointer(&at(350.0, 300.0), &theme, narrow).handled);
    let res = d.pointer(
        &WidgetEvent::MouseDown { x: 350.0, y: 300.0 },
        &theme,
        narrow,
    );
    assert!(res.clicked && !d.is_drawer_open());
}

#[test]
fn a_finger_makes_it_a_drawer_on_a_wide_window() {
    let theme = Theme::hoff_core();
    let mut d = dock();
    d.set_touch(true, &theme, WIDE.w);
    assert_eq!(d.layout(&theme, WIDE).mode, DockMode::Drawer);
    d.set_touch(false, &theme, WIDE.w);
    assert_eq!(d.layout(&theme, WIDE).mode, DockMode::Peek);
}

/// Every width from 280 to 1600: a drawer exactly where the panel and
/// the page's narrowest content do not fit side by side; pinned, the
/// page keeps at least that content; the panel never leaves the window.
#[test]
fn the_mode_follows_the_content_rule_across_widths() {
    for theme in [Theme::hoff_core(), Theme::hoff(), Theme::dark()] {
        let r = theme.navigation.dock;
        let d = dock().pinned(true);
        let mut drawer_seen = false;
        let mut pinned_seen = false;
        for w in (280..=1600).step_by(8) {
            let w = w as f32;
            let window = Rect::new(0.0, 0.0, w, 800.0);
            let l = d.layout(&theme, window);
            assert!(l.rest.w <= w, "{w}: the panel fits the window");
            if r.fits(w, &theme) {
                pinned_seen = true;
                assert_eq!(l.mode, DockMode::Pinned, "{w}");
                let page = w - l.page_x;
                let gutter = theme.layout.gutter(theme.layout.breakpoint(page));
                assert!(page - gutter * 2.0 >= r.content_min_w, "{w}");
            } else {
                drawer_seen = true;
                assert_eq!(l.mode, DockMode::Drawer, "{w}");
                assert_eq!(l.page_x, 0.0, "{w}");
            }
        }
        assert!(drawer_seen && pinned_seen, "the sweep crosses the rule");
    }
}

#[test]
fn it_projects_a_landmark_its_pin_and_in_the_drawer_its_menu() {
    let theme = Theme::hoff_core();
    let mut d = dock();
    d.toggle_pin(&theme, WIDE.w);
    let mut b = SnapshotBuilder::new();
    d.project(&mut b, "Gallery", None, &theme, WIDE.w);
    let snap = b.finish().unwrap();
    let nav = snap.get(&SemId::name("nav").unwrap()).unwrap();
    assert_eq!(nav.role, Role::Navigation, "a navigation landmark");
    assert_eq!(nav.state.get("mode"), Some(&Value::Text("pinned".into())));
    assert_eq!(nav.state.get("expanded"), Some(&Value::Bool(true)));
    let pin = snap.get(&d.pin_id().unwrap()).unwrap();
    assert_eq!(pin.role, Role::Toggle, "a toggle with its state");
    assert_eq!(pin.state.get("on"), Some(&Value::Bool(true)));
    assert!(snap.get(&d.menu_id().unwrap()).is_none());

    let mut b = SnapshotBuilder::new();
    d.project(&mut b, "Gallery", None, &theme, 360.0);
    let snap = b.finish().unwrap();
    let menu = snap.get(&d.menu_id().unwrap()).unwrap();
    assert_eq!(menu.state.get("expanded"), Some(&Value::Bool(false)));
    let nav = snap.get(&SemId::name("nav").unwrap()).unwrap();
    assert_eq!(nav.state.get("mode"), Some(&Value::Text("drawer".into())));
    assert_eq!(
        nav.state.get("expanded"),
        Some(&Value::Bool(false)),
        "a closed drawer, pinned or not"
    );
}

/// The pin at the head's right: a click asks the flip `toggle_pin` runs;
/// it is drawn and bound where it is hit, and the drawer has none.
#[test]
fn a_click_on_the_pin_pins_it_and_the_drawer_has_no_pin() {
    use crate::drawing::compositor::Compositor;
    let theme = Theme::hoff_core();
    let r = theme.navigation.dock;
    let mut d = dock();
    let head = Rect::new(0.0, 0.0, r.width, 80.0);
    let pin = SidebarDock::pin_rect(head, &theme);
    assert!(pin.right() <= head.right() && pin.w == r.button);
    let (x, y) = pin.center();
    d.pin_event(&at(x, y), head, &theme, WIDE.w);
    d.pin_event(&WidgetEvent::MouseDown { x, y }, head, &theme, WIDE.w);
    let up = d.pin_event(&WidgetEvent::MouseUp { x, y }, head, &theme, WIDE.w);
    assert!(
        up.clicked && !d.is_pinned(),
        "the click asks; the app flips"
    );
    d.toggle_pin(&theme, WIDE.w);
    let mut c = Compositor::new();
    d.render_pin(&mut c, head, &theme, WIDE.w);
    let bound = c.binding(&d.pin_id().unwrap()).map(|b| b.rect);
    assert_eq!(bound, Some([pin.x, pin.y, pin.w, pin.h]));
    // narrow: a drawer, no pin to draw or to press
    let mut c = Compositor::new();
    d.render_pin(&mut c, head, &theme, 360.0);
    assert!(c.binding(&d.pin_id().unwrap()).is_none());
    d.pin_event(&WidgetEvent::MouseDown { x, y }, head, &theme, 360.0);
    let up = d.pin_event(&WidgetEvent::MouseUp { x, y }, head, &theme, 360.0);
    assert!(!up.clicked && d.is_pinned());
}

// The dock in the app shell (`AppShell::docked`).

mod shell {
    use super::*;
    use crate::widgets::nav::PanelHeader;
    use crate::widgets::shell::{AppShell, Route};

    fn docked(w: f32, h: f32) -> AppShell {
        AppShell::docked(dock(), PanelHeader::new("Cards"), w, h)
    }

    #[test]
    fn peeking_the_page_has_the_window_and_pinned_it_starts_past_the_panel() {
        let theme = Theme::hoff_core();
        let r = theme.navigation.dock;
        let mut s = docked(WIDE.w, WIDE.h);
        let l = s.layout(&theme);
        let gutter = theme.layout.gutter(l.breakpoint);
        assert_eq!(l.dock.map(|d| d.mode), Some(DockMode::Peek));
        assert!(l.sidebar.is_none() && l.menu_button.is_none());
        assert_eq!(l.content.x, gutter);
        s.dock.as_mut().unwrap().toggle_pin(&theme, WIDE.w);
        let l = s.layout(&theme);
        assert_eq!(l.sidebar.map(|p| p.w), Some(r.width));
        assert_eq!(l.content.x, r.width + gutter);
    }

    #[test]
    fn the_edge_is_the_pages_and_the_shown_panel_is_the_apps() {
        let theme = Theme::hoff_core();
        let r = theme.navigation.dock;
        let mut s = docked(WIDE.w, WIDE.h);
        assert_eq!(s.route(&at(1.0, 400.0), &theme), Route::Page);
        let dt = 1.0 / 60.0;
        let mut t = 0.0;
        while t < r.enter_delay + r.slide + 0.05 {
            s.wake(dt, &theme);
            t += dt;
        }
        let l = s.layout(&theme);
        assert_eq!(l.dock.and_then(|d| d.panel).map(|p| p.x), Some(0.0));
        assert_eq!(s.route(&at(r.width / 2.0, 400.0), &theme), Route::Panel);
        assert_eq!(s.route(&at(r.width + 100.0, 400.0), &theme), Route::Page);
    }

    #[test]
    fn narrow_the_menu_button_opens_the_drawer_and_a_press_off_it_closes_it() {
        let theme = Theme::hoff_core();
        let mut s = docked(360.0, 700.0);
        let l = s.layout(&theme);
        assert_eq!(l.dock.map(|d| d.mode), Some(DockMode::Drawer));
        let mb = l.menu_button.expect("a drawer has its menu button");
        let (x, y) = (mb.x + mb.w / 2.0, mb.y + mb.h / 2.0);
        s.route(&at(x, y), &theme);
        s.route(&WidgetEvent::MouseDown { x, y }, &theme);
        let up = s.route(&WidgetEvent::MouseUp { x, y }, &theme);
        assert!(matches!(up, Route::Shell(r) if r.clicked));
        assert!(s.dock.as_ref().unwrap().is_drawer_open());
        // the page is inert under it
        assert!(matches!(
            s.route(&at(350.0, 600.0), &theme),
            Route::Shell(_)
        ));
        let off = s.route(&WidgetEvent::MouseDown { x: 350.0, y: 600.0 }, &theme);
        assert!(matches!(off, Route::Shell(r) if r.clicked));
        assert!(!s.dock.as_ref().unwrap().is_drawer_open());
    }
}
