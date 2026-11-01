//! The controls drawn under the minimum target press in it (WCAG 2.5.8,
//! `widgets::hit`), with a pointer (24) and with a finger (44), and look
//! the same: the split's divider, the scrollbar, the switch and the
//! checkbox.

use crate::control::input::scroll::ScrollState;
use crate::drawing::compositor::Compositor;
use crate::styling::theme::Theme;
use crate::widgets::feedback::Scrollbar;
use crate::widgets::form::{Checkbox, Switch};
use crate::widgets::hit::set_touch_input;
use crate::widgets::nav::{SplitDirection, SplitPane};
use crate::widgets::widget::{Rect, WidgetEvent};

fn both(f: impl Fn(f32)) {
    let theme = Theme::default();
    for (touch, min) in [
        (false, theme.control.min_target),
        (true, theme.control.min_touch_target),
    ] {
        set_touch_input(touch);
        f(min);
    }
    set_touch_input(false);
}

#[test]
fn the_split_divider_grabs_across_the_minimum() {
    let theme = Theme::default();
    let bounds = Rect::new(0.0, 0.0, 800.0, 400.0);
    both(|min| {
        for dir in [SplitDirection::Horizontal, SplitDirection::Vertical] {
            let pane = SplitPane::new(dir, 0.5, &theme);
            let g = pane.grab_rect(bounds);
            let across = match dir {
                SplitDirection::Horizontal => g.w,
                SplitDirection::Vertical => g.h,
            };
            assert!(across >= min - 0.01, "{dir:?}: {across} < {min}");
            assert!(theme.control.divider_line < min, "the line stays thin");
        }
    });
}

#[test]
fn the_scrollbar_presses_in_the_minimum_from_its_edge() {
    let theme = Theme::default();
    let bounds = Rect::new(0.0, 0.0, 300.0, 400.0);
    both(|min| {
        let mut scroll = ScrollState::new();
        scroll.set_viewport(400.0);
        scroll.set_content(2000.0);
        let mut bar = Scrollbar::new();
        let band = bar.press_rect(bounds, &theme);
        assert!(band.w >= min - 0.01, "{} < {min}", band.w);
        assert!(
            (band.right() - bounds.right()).abs() < 0.01,
            "from the edge"
        );
        assert!(bar.track_rect(bounds, &theme).w < min, "drawn thin");
        // shown (a scroll wakes it), a press at the band's inner edge
        // grabs the bar
        bar.notify_scroll();
        for _ in 0..60 {
            bar.tick(1.0 / 60.0);
        }
        let r = bar.handle_event(
            &WidgetEvent::MouseDown {
                x: band.x + 0.5,
                y: 10.0,
            },
            bounds,
            &mut scroll,
            &theme,
        );
        assert!(
            r.handled || bar.is_dragging(),
            "the band's edge presses the bar"
        );
    });
}

#[test]
fn a_switch_and_a_checkbox_press_and_bind_in_the_minimum() {
    let theme = Theme::default();
    both(|min| {
        let small = Rect::new(100.0, 100.0, 18.0, 18.0);
        let mut c = Compositor::new();
        c.begin_frame();
        let id = accord::SemId::parse("t.check").unwrap();
        let mut check = Checkbox::new(false).sem(id.clone());
        check.render(&mut c, small, &theme);
        let b = c.binding(&id).unwrap().rect;
        assert!(b[2] >= min - 0.01 && b[3] >= min - 0.01, "{b:?}");
        // a press just outside the box, inside the minimum, toggles it
        let (x, y) = (small.x - (min - small.w) / 2.0 + 0.5, small.y + 9.0);
        check.handle_event(&WidgetEvent::MouseDown { x, y }, small);
        check.handle_event(&WidgetEvent::MouseUp { x, y }, small);
        assert!(check.checked, "pressed at {x}");

        let track = Rect::new(0.0, 0.0, 40.0, 22.0);
        let mut c = Compositor::new();
        c.begin_frame();
        let sid = accord::SemId::parse("t.switch").unwrap();
        let mut switch = Switch::new(false).sem(sid.clone());
        switch.render(&mut c, track, &theme);
        let b = c.binding(&sid).unwrap().rect;
        assert!(b[3] >= min - 0.01, "{b:?}");
        let y = track.y - (min - track.h) / 2.0 + 0.5;
        switch.handle_event(&WidgetEvent::MouseDown { x: 20.0, y }, track);
        switch.handle_event(&WidgetEvent::MouseUp { x: 20.0, y }, track);
        assert!(switch.on, "pressed above the track");
    });
}

/// The view controller's 16 px arrows press in the least target around
/// them and bind it; the arrows draw where they did.
#[test]
fn the_view_controller_s_arrows_press_in_the_minimum() {
    use crate::widgets::nav::ViewController;
    let theme = Theme::default();
    let bounds = Rect::new(40.0, 40.0, 400.0, 200.0);
    both(|min| {
        for upright in [false, true] {
            let v = ViewController::new(upright);
            let (back, forward) = v.arrow_rects(bounds, &theme);
            let (pb, pf) = v.press_rects(bounds, &theme);
            for (drawn, press) in [(back, pb), (forward, pf)] {
                assert!(drawn.w < min, "drawn small");
                assert!(press.w >= min - 0.01 && press.h >= min - 0.01);
                assert_eq!(press.center(), drawn.center(), "around the arrow");
            }
            assert!(pb.right() <= pf.x || pb.bottom() <= pf.y || pf.bottom() <= pb.y);
        }
    });
}
