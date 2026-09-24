//! What the workspace asks of the loop, with an injected clock: a focused
//! commit field only at its flips, a toast only while it moves and at the
//! instant it leaves.

use super::*;
use comps::prelude::Wake;

/// One turn of a 60 Hz display.
const TURN: f32 = 1.0 / 60.0;

/// How long the window sat idle before the event: the event's frame
/// reports it as its `since_last`.
const IDLE: f32 = 5.0;

/// The ide shell over `turns` display turns: turn 0 is the frame of the
/// event (the key that focused, the refusal that notified), after an idle
/// window; afterwards a turn is drawn when the last ask was continuous or
/// its deadline has come due, and the drawn turn's tick gets the time
/// since the last drawn turn (`since_last`). Returns each drawn turn with
/// its ask.
fn paced(w: &mut WorkspaceView, turns: usize) -> Vec<(usize, Wake)> {
    let mut drawn = vec![(0, w.wake(IDLE))];
    for turn in 1..turns {
        let (last, ask) = *drawn.last().expect("turn 0 is drawn");
        let since = (turn - last) as f32 * TURN;
        if ask.next.is_some_and(|d| since >= d.as_secs_f32()) {
            drawn.push((turn, w.wake(since)));
        }
    }
    drawn
}

#[test]
fn a_focused_commit_field_asks_for_a_frame_only_at_each_flip() {
    let (mut w, _) = ws();
    w.commit_form.show();
    // six seconds at 60 Hz with nothing typed
    let drawn = paced(&mut w, 360);
    let flips = &drawn[1..];
    assert!(
        flips.iter().all(|(_, ask)| ask.now && !ask.is_continuous()),
        "every frame after the key's is a flip: {drawn:?}"
    );
    // the flips at 0.53 s, 1.06 s ... 5.83 s from the key: eleven, where
    // the bool drew all 360 turns
    assert_eq!(flips.len(), 11, "{drawn:?}");
    let first = flips[0].0 as f32 * TURN;
    assert!((first - 0.53).abs() <= TURN, "first flip at {first} s");
}

#[test]
fn a_hidden_commit_form_asks_for_nothing() {
    let (mut w, _) = ws();
    w.commit_form.show();
    w.commit_form.hide();
    assert_eq!(w.wake(TURN), Wake::idle());
    assert_eq!(w.wake(10.0), Wake::idle());
}

#[test]
fn a_toast_asks_for_the_instant_it_leaves_then_nothing() {
    let (mut w, _) = ws();
    w.notify("Refused: the commit message is empty", Intent::Neutral);
    let drawn = paced(&mut w, 360);
    assert!(w.toasts.is_empty(), "gone within six seconds");
    // its last frame draws it gone and asks for nothing more
    assert_eq!(drawn.last().map(|(_, ask)| ask.next), Some(None));

    // springing in: every turn; at rest: one deadline, the dismissal
    let rest = drawn
        .iter()
        .position(|(_, ask)| !ask.is_continuous())
        .expect("the entry settles");
    let (rest_turn, rest_ask) = drawn[rest];
    let left = rest_ask.next.expect("a deadline while it rests");
    let due = rest_turn as f32 * TURN + left.as_secs_f32();
    assert!((due - w.toasts.duration).abs() < TURN, "leaves at {due} s");
    let (leave_turn, leave_ask) = drawn[rest + 1];
    assert!(leave_ask.is_continuous(), "the exit spring moves");
    assert!(
        (leave_turn - rest_turn - 1) as f32 * TURN > 2.0,
        "nothing drawn while it rests: {drawn:?}"
    );
}

#[test]
fn a_toast_and_a_focused_field_ask_for_the_earliest_of_the_two() {
    let (mut w, _) = ws();
    w.commit_form.show();
    w.notify("Refused: the commit message is empty", Intent::Neutral);
    let drawn = paced(&mut w, 360);
    assert!(w.toasts.is_empty(), "gone within six seconds");
    // the toast springs in, then rests: every frame until it leaves is a
    // caret flip, since the flip is always the earlier deadline
    let rest = drawn
        .iter()
        .position(|(_, ask)| !ask.is_continuous())
        .expect("the entry settles");
    let leave = rest
        + drawn[rest..]
            .iter()
            .position(|(_, ask)| ask.is_continuous())
            .expect("the toast leaves");
    let resting = &drawn[rest + 1..leave];
    assert!(resting.len() >= 3, "{drawn:?}");
    assert!(resting.iter().all(|(_, ask)| ask.now), "{drawn:?}");
    assert!(
        drawn[rest..leave]
            .iter()
            .all(|(_, ask)| ask.next.is_some_and(|d| d.as_secs_f32() <= 0.53))
    );
    // once it is gone only the caret's flips remain
    let (_, last) = *drawn.last().expect("drawn");
    let next = last.next.expect("the caret still blinks").as_secs_f32();
    assert!(next > 0.0 && next <= 0.53, "{next}");
}
