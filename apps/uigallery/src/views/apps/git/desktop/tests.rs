//! What the Git page asks of the loop, with an injected clock, and when
//! the git worker's events draw: only with the page on screen.

use super::*;
use crate::views::ShowcaseView;
use accord::SemId;

/// One turn of a 60 Hz display.
const TURN: f32 = 1.0 / 60.0;

/// How long the window sat idle before the event. The shell catches up
/// (a tick of this length) before it applies the event.
const IDLE: f32 = 5.0;

/// How long after the event its frame is drawn.
const LATENCY: f32 = 0.004;

/// The shell over `turns` display turns after an event it applied once
/// caught up: turn 0 is the event's frame, a few milliseconds after it;
/// afterwards a turn is drawn when the last ask was continuous or its
/// deadline has come due, and the drawn turn's tick gets the time since
/// the last drawn turn (`since_last`).
fn paced(turns: usize, mut wake: impl FnMut(f32) -> Wake) -> Vec<(usize, Wake)> {
    let mut drawn = vec![(0, wake(LATENCY))];
    for turn in 1..turns {
        let (last, ask) = *drawn.last().expect("turn 0 is drawn");
        let since = (turn - last) as f32 * TURN;
        if ask.next.is_some_and(|d| since >= d.as_secs_f32()) {
            drawn.push((turn, wake(since)));
        }
    }
    drawn
}

/// The toasts the workbench projects under `notices`.
fn notices(page: &GitPage) -> usize {
    let snap = page.bench.snapshot().expect("snapshot");
    let id: SemId = "notices".parse().expect("id");
    snap.children(&id).count()
}

#[test]
fn a_focused_commit_field_asks_for_a_frame_only_at_each_flip() {
    let mut page = GitPage::new();
    page.wake(IDLE);
    page.bench.workspace_mut().commit_form.show();
    let drawn = paced(360, |dt| page.wake(dt));
    let flips = &drawn[1..];
    assert!(
        flips.iter().all(|(_, ask)| ask.now && !ask.is_continuous()),
        "{drawn:?}"
    );
    assert_eq!(flips.len(), 11, "one frame per flip in six seconds");
}

#[test]
fn a_refusal_toast_draws_while_it_moves_and_at_the_instant_it_leaves() {
    let mut page = GitPage::new();
    page.bench.workspace_mut().commit_form.show();
    page.wake(IDLE);
    // Enter with no message: the dispatcher refuses and says so
    assert!(page.handle_enter());
    assert_eq!(notices(&page), 1, "the refusal shows as a toast");
    // Escape closes the form: only the toast is left
    assert!(page.handle_escape());
    let drawn = paced(360, |dt| page.wake(dt));
    assert_eq!(notices(&page), 0, "gone within six seconds");
    assert_eq!(drawn.last().map(|(_, ask)| ask.next), Some(None));
    let rest = drawn
        .iter()
        .position(|(_, ask)| !ask.is_continuous())
        .expect("the entry settles");
    let (rest_turn, rest_ask) = drawn[rest];
    let left = rest_ask.next.expect("a deadline while it rests");
    let due = rest_turn as f32 * TURN + left.as_secs_f32();
    assert!((due - 4.0).abs() < TURN, "leaves at {due} s");
    let (leave_turn, leave_ask) = drawn[rest + 1];
    assert!(leave_ask.is_continuous(), "the exit spring moves");
    assert!(
        (leave_turn - rest_turn - 1) as f32 * TURN > 2.0,
        "{drawn:?}"
    );
}

#[test]
fn worker_events_wait_for_the_host_to_apply_them() {
    let mut page = GitPage::new();
    page.sender
        .send(GitEvent::Status(Ok(Vec::new())))
        .expect("send");
    assert_eq!(page.wake(TURN), Wake::idle(), "a tick applies nothing");
    assert!(page.apply_events(), "the host applies them: a change");
    assert!(!page.apply_events(), "drained");
}

/// Ticks the gallery until nothing on it moves.
fn settle(view: &mut ShowcaseView) -> Wake {
    let mut wake = view.wake(TURN);
    for _ in 0..600 {
        if !wake.is_continuous() {
            break;
        }
        wake = view.wake(TURN);
    }
    wake
}

#[test]
fn the_gallery_asks_for_the_git_caret_only_while_the_page_shows() {
    let mut view = ShowcaseView::new(1200.0, 800.0);
    view.jump_to_page("Git");
    view.panes.git.bench.workspace_mut().commit_form.show();
    let wake = settle(&mut view);
    let next = wake.next.expect("the caret's next flip").as_secs_f32();
    assert!(next > 0.0 && next <= 0.53, "{next}");
    view.jump_to_page("Button");
    assert_eq!(settle(&mut view), Wake::idle(), "hidden: nothing asked");
}

#[test]
fn worker_events_land_while_hidden_and_draw_only_on_the_git_page() {
    let mut view = ShowcaseView::new(1200.0, 800.0);
    view.jump_to_page("Button");
    let event = || GitEvent::Status(Ok(Vec::new()));
    view.panes.git.sender.send(event()).expect("send");
    assert!(!view.apply_git(), "hidden: no frame");
    assert!(
        view.panes.git.events.try_recv().is_err(),
        "applied all the same"
    );
    view.jump_to_page("Git");
    view.panes.git.sender.send(event()).expect("send");
    assert!(view.apply_git(), "on screen: a frame");
    assert!(!view.apply_git(), "nothing left to apply");
}

/// The workspace draws in the gallery's theme: core's light palette and
/// face in its light slot, hoff's dark one in its dark slot.
#[test]
fn the_workspace_follows_the_gallery_s_theme() {
    use idestudio::interfaces::panels::layout::ThemeMode;
    let mut page = GitPage::new();
    let core = Theme::hoff_core();
    page.set_theme(&core);
    let ws = page.bench.workspace();
    assert_eq!(ws.theme_mode, ThemeMode::Light);
    assert_eq!(ws.theme().colors.bg, core.colors.bg);
    assert_eq!(ws.theme().typography.family, core.typography.family);
    let hoff = Theme::hoff();
    page.set_theme(&hoff);
    let ws = page.bench.workspace();
    assert_eq!(ws.theme_mode, ThemeMode::Dark);
    assert_eq!(ws.theme().colors.bg, hoff.colors.bg);
}
