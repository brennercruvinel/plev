//! The shell's pacing with an injected clock: a change from outside the
//! frame lands after the loop caught up (`App::advance`, what `catch_up`
//! runs with the real elapsed time), so it counts its time from the
//! change, however long the window sat idle before it.

use engine::styling::theme::Intent;
use engine::widgets::prelude::Wake;

use super::App;
use crate::views::ShowcaseView;

/// One turn of a 60 Hz display.
const TURN: f32 = 1.0 / 60.0;

/// How long the window sat idle before the event.
const IDLE: f32 = 10.0;

/// How long after the event its frame is drawn.
const LATENCY: f32 = 0.004;

/// The gallery's shell, settled: nothing it starts with still moves.
fn app(page: &str) -> App {
    let mut app = App::with_view(ShowcaseView::new(1200.0, 800.0));
    app.view.jump_to_page(page);
    for _ in 0..600 {
        if !app.frame_wake(TURN).is_continuous() {
            break;
        }
    }
    app
}

/// The frames over `turns` display turns after an event: turn 0 is the
/// event's frame; afterwards a turn is drawn when the last ask was
/// continuous or its deadline has come due, and its tick gets the time
/// since the last drawn turn.
fn paced(app: &mut App, turns: usize) -> Vec<(usize, Wake)> {
    let mut drawn = vec![(0, app.frame_wake(LATENCY))];
    for turn in 1..turns {
        let (last, ask) = *drawn.last().expect("turn 0 is drawn");
        let since = (turn - last) as f32 * TURN;
        if ask.next.is_some_and(|d| since >= d.as_secs_f32()) {
            drawn.push((turn, app.frame_wake(since)));
        }
    }
    drawn
}

#[test]
fn a_toast_pushed_after_an_idle_gap_lives_its_whole_duration() {
    let mut app = app("Button");
    app.advance(IDLE);
    let theme = app.view.theme.clone();
    app.view.toasts.push("saved", Intent::Constructive, &theme);
    let drawn = paced(&mut app, 360);
    let toast = |app: &App| app.view.toasts.visible().next().map(|t| t.is_closing());
    // springs in, rests on the deadline, leaves 4 s after the push
    let rest = drawn
        .iter()
        .position(|(_, ask)| !ask.is_continuous())
        .expect("the entry settles");
    let (rest_turn, rest_ask) = drawn[rest];
    let left = rest_ask.next.expect("a deadline while it rests");
    let due = rest_turn as f32 * TURN + LATENCY + left.as_secs_f32();
    assert!((due - 4.0).abs() < TURN, "leaves {due} s after the push");
    assert_eq!(toast(&app), None, "gone within six seconds");
}

#[test]
fn a_toast_pushed_after_an_idle_gap_springs_in_from_the_start() {
    let mut app = app("Button");
    app.advance(IDLE);
    let theme = app.view.theme.clone();
    app.view.toasts.push("saved", Intent::Constructive, &theme);
    let ask = app.frame_wake(LATENCY);
    assert!(ask.is_continuous(), "springing in");
    let shown = app.view.toasts.visible().next().expect("on screen");
    assert!(!shown.is_closing());
    assert!(shown.progress() < 0.1, "{}", shown.progress());
}

#[test]
fn a_caret_focused_after_an_idle_gap_starts_solid_and_flips_on_time() {
    let mut app = app("Git");
    app.advance(IDLE);
    // `c` opens the commit form, focused
    assert!(app.view.handle_key("c"));
    let drawn = paced(&mut app, 360);
    let (_, first) = drawn[0];
    assert!(
        !first.now || first.is_continuous(),
        "no flip on the key's frame"
    );
    // a whole half-period from the key, none of the idle before it
    let next = first.next.expect("a flip is pending").as_secs_f32();
    assert!(
        (0.53 - LATENCY - 1e-3..0.53 + 1e-3).contains(&next),
        "{next}"
    );
    let flips: Vec<usize> = drawn[1..]
        .iter()
        .filter(|(_, ask)| ask.now)
        .map(|(turn, _)| *turn)
        .collect();
    // 0.53 s, 1.06 s ... after the key: the first on turn 32
    assert_eq!(flips.len(), 11, "{drawn:?}");
    assert_eq!(flips[0], 32);
}

#[test]
fn a_catch_up_that_changes_nothing_asks_for_no_frame() {
    let mut app = app("Button");
    app.advance(IDLE);
    assert!(!app.caught_up);
    assert_eq!(app.frame_wake(TURN), Wake::idle());
}
