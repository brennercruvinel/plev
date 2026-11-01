//! The offset a focus asks for: below, above, already in view, longer
//! than the window, nested, under a fixed band; the glide's motion and
//! the pointer's exemption.

use super::*;
use crate::compose::animation::set_reduced_motion;

/// The window a page shows: 100 px down the window, 400 px tall.
const VIEW: Span = Span {
    start: 100.0,
    len: 400.0,
};
const MARGIN: f32 = 20.0;
const NONE: Cover = Cover {
    start: 0.0,
    end: 0.0,
};

fn page(offset: f32) -> ScrollState {
    let mut s = ScrollState::new();
    s.set_viewport(VIEW.len);
    s.set_content(2000.0);
    s.scroll_to(offset);
    s
}

fn sid(text: &str) -> SemId {
    SemId::parse(text).unwrap_or_else(|e| panic!("{text}: {e}"))
}

#[test]
fn a_control_below_the_fold_scrolls_up_to_the_bottom_edge_and_its_margin() {
    // 40 px tall at 600: its end (640) must sit at 500 - 20
    let target = Span::new(600.0, 40.0);
    assert_eq!(nearest(target, VIEW, MARGIN, NONE), 160.0);
    let mut s = page(0.0);
    assert_eq!(s.reveal(target, VIEW, MARGIN, NONE), 160.0);
    assert_eq!(s.offset(), 160.0);
}

#[test]
fn a_control_above_the_window_scrolls_back_to_the_top_edge_and_its_margin() {
    let target = Span::new(40.0, 40.0);
    assert_eq!(nearest(target, VIEW, MARGIN, NONE), -80.0);
    let mut s = page(300.0);
    assert_eq!(s.reveal(target, VIEW, MARGIN, NONE), -80.0);
    assert_eq!(s.offset(), 220.0);
}

#[test]
fn a_control_already_in_view_does_not_move_the_page() {
    for start in [120.0, 300.0, 440.0] {
        let target = Span::new(start, 40.0);
        assert_eq!(nearest(target, VIEW, MARGIN, NONE), 0.0, "{start}");
        let mut s = page(50.0);
        assert_eq!(s.reveal(target, VIEW, MARGIN, NONE), 0.0);
        assert_eq!(s.offset(), 50.0);
    }
}

#[test]
fn the_scroll_stops_at_the_content_s_ends() {
    // asks for 160 more, has 30 left
    let mut s = page(1570.0);
    let moved = s.reveal(Span::new(600.0, 40.0), VIEW, MARGIN, NONE);
    assert_eq!(moved, 30.0);
    assert_eq!(s.offset(), s.max_offset());
    let mut s = page(10.0);
    assert_eq!(s.reveal(Span::new(0.0, 40.0), VIEW, MARGIN, NONE), -10.0);
    assert_eq!(s.offset(), 0.0);
}

#[test]
fn a_control_longer_than_the_window_shows_its_start_unless_it_fills_the_window() {
    let tall = Span::new(700.0, 900.0);
    assert_eq!(nearest(tall, VIEW, MARGIN, NONE), 580.0);
    let covering = Span::new(50.0, 900.0);
    assert_eq!(nearest(covering, VIEW, MARGIN, NONE), 0.0);
}

#[test]
fn a_fixed_band_over_the_top_keeps_the_control_clear_of_it() {
    // a 60 px header fixed over the window's top
    let band = Cover {
        start: 60.0,
        end: 0.0,
    };
    // in view without the band, under it with it
    let target = Span::new(130.0, 40.0);
    assert_eq!(nearest(target, VIEW, MARGIN, NONE), 0.0);
    assert_eq!(nearest(target, VIEW, MARGIN, band), -50.0);
    // and a bar fixed over the bottom
    let bar = Cover {
        start: 0.0,
        end: 80.0,
    };
    assert_eq!(nearest(Span::new(400.0, 40.0), VIEW, MARGIN, bar), 40.0);
}

#[test]
fn a_window_too_short_for_the_margin_drops_it_and_still_shows_the_control() {
    let narrow = Span::new(100.0, 50.0);
    assert_eq!(nearest(Span::new(300.0, 40.0), narrow, MARGIN, NONE), 190.0);
    // too short even for the band: the band goes too
    let band = Cover {
        start: 60.0,
        end: 0.0,
    };
    assert_eq!(nearest(Span::new(300.0, 40.0), narrow, MARGIN, band), 190.0);
}

#[test]
fn nested_containers_scroll_the_inner_one_first_then_the_outer_one() {
    // an inner panel 200 px tall at 300 (inside the page's window), its
    // content 1000; the control at 900 in window space, under both folds
    let mut inner = ScrollState::new();
    inner.set_viewport(200.0);
    inner.set_content(1000.0);
    let mut outer = page(0.0);
    let inner_view = Span::new(300.0, 200.0);
    let target = Span::new(900.0, 40.0);
    let total = reveal_nested(
        target,
        &mut [
            Scroller {
                scroll: &mut inner,
                view: inner_view,
                cover: NONE,
            },
            Scroller {
                scroll: &mut outer,
                view: VIEW,
                cover: NONE,
            },
        ],
        MARGIN,
    );
    // the panel brings it to its own bottom edge (900 + 40 - 480)...
    assert_eq!(inner.offset(), 460.0);
    // ...which is inside the page's window already: the page stays
    assert_eq!(outer.offset(), 0.0);
    assert_eq!(total, 460.0);

    // the same panel below the page's fold: the page follows
    let mut inner = ScrollState::new();
    inner.set_viewport(200.0);
    inner.set_content(1000.0);
    let mut outer = page(0.0);
    reveal_nested(
        Span::new(1200.0, 40.0),
        &mut [
            Scroller {
                scroll: &mut inner,
                view: Span::new(700.0, 200.0),
                cover: NONE,
            },
            Scroller {
                scroll: &mut outer,
                view: VIEW,
                cover: NONE,
            },
        ],
        MARGIN,
    );
    assert_eq!(inner.offset(), 360.0, "the panel shows it at its bottom");
    // it sits at 840..880 once the panel scrolled: the page brings 880 to 480
    assert_eq!(
        outer.offset(),
        400.0,
        "then the page shows the panel's part"
    );
}

fn motion() -> MotionPhysics {
    MotionPhysics {
        mass: 1.0,
        stiffness: 170.0,
        damping: 26.0,
    }
}

#[test]
fn the_glide_moves_with_the_theme_s_motion_and_lands_on_the_offset() {
    let mut s = page(0.0);
    let mut glide = Glide::default();
    glide.to(&mut s, 300.0, &motion());
    assert!(glide.is_gliding());
    assert_eq!(s.offset(), 0.0, "nothing moves before the first tick");
    assert!(glide.tick(&mut s, 1.0 / 60.0));
    let first = s.offset();
    assert!(first > 0.0 && first < 300.0, "{first}: on its way");
    for _ in 0..600 {
        if !glide.tick(&mut s, 1.0 / 60.0) {
            break;
        }
    }
    assert!(!glide.is_gliding());
    assert_eq!(s.offset(), 300.0);
}

#[test]
fn a_glide_far_down_a_long_page_lands_on_its_offset() {
    // thousands of pixels down, where an f32 rounds coarser than the
    // spring's rest test: the glide lands, and soon (the spring steps its
    // displacement, `compose::animation::Spring`)
    let mut s = ScrollState::new();
    s.set_viewport(400.0);
    s.set_content(20_000.0);
    s.scroll_to(167.0);
    let mut glide = Glide::default();
    glide.to(&mut s, 2349.0, &motion());
    let mut frames = 0;
    while glide.tick(&mut s, 1.0 / 60.0) {
        frames += 1;
        assert!(frames < 120, "still gliding after two seconds");
    }
    assert_eq!(s.offset(), 2349.0);
}

#[test]
fn with_reduced_motion_the_glide_is_there_at_once() {
    set_reduced_motion(true);
    let mut s = page(0.0);
    let mut glide = Glide::default();
    glide.to(&mut s, 300.0, &motion());
    set_reduced_motion(false);
    assert_eq!(s.offset(), 300.0);
    assert!(!glide.is_gliding());
    assert!(!glide.tick(&mut s, 1.0 / 60.0));
}

#[test]
fn the_wheel_stops_the_glide_where_it_is() {
    let mut s = page(0.0);
    let mut glide = Glide::default();
    glide.to(&mut s, 300.0, &motion());
    glide.tick(&mut s, 1.0 / 60.0);
    let at = s.offset();
    glide.stop();
    assert!(!glide.tick(&mut s, 1.0 / 60.0));
    assert_eq!(s.offset(), at);
}

#[test]
fn a_focus_that_moves_asks_once_and_the_pointer_s_never_asks() {
    let mut f = FocusReveal::new();
    assert_eq!(f.moved(None), None);
    assert_eq!(f.moved(Some(&sid("page.a"))), Some(sid("page.a")));
    assert_eq!(f.moved(Some(&sid("page.a"))), None, "it stayed");
    f.pointer();
    assert_eq!(f.moved(Some(&sid("page.b"))), None, "a click moved it");
    assert_eq!(f.seen(), Some(&sid("page.b")));
    assert_eq!(f.moved(Some(&sid("page.c"))), Some(sid("page.c")), "Tab");
    f.pointer();
    assert_eq!(f.moved(Some(&sid("page.c"))), None);
    assert_eq!(
        f.moved(Some(&sid("page.d"))),
        Some(sid("page.d")),
        "a click that moved nothing leaves the next move to the keyboard"
    );
    assert_eq!(f.moved(None), None, "the focus went");
    f.reset();
    assert_eq!(f.seen(), None);
    assert_eq!(f.moved(Some(&sid("other.a"))), None, "a page shows as left");
    assert_eq!(f.moved(Some(&sid("other.b"))), Some(sid("other.b")));
}

/// The geometry is one axis: on x, a wide table's column off the right
/// edge of its strip scrolls left the least that shows it, a column off
/// the left edge back, and one in the strip stays (task-104 front I).
#[test]
fn the_same_rule_holds_across_a_wide_table() {
    // a strip 600 wide at x 200, its columns 1800 wide in all
    let strip = Span::new(200.0, 600.0);
    let mut columns = ScrollState::new();
    columns.set_viewport(strip.len);
    columns.set_content(1800.0);
    let column = |x: f32| Span::new(x, 160.0);
    assert_eq!(columns.reveal(column(900.0), strip, MARGIN, NONE), 280.0);
    assert_eq!(columns.offset(), 280.0);
    // the same column, now at 620 on screen, is in the strip
    assert_eq!(columns.reveal(column(620.0), strip, MARGIN, NONE), 0.0);
    // the first column, now at -80, comes back with its margin
    assert_eq!(columns.reveal(column(-80.0), strip, MARGIN, NONE), -280.0);
    assert_eq!(columns.offset(), 0.0);
}
