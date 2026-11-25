//! criterion 1 of task-64: the session of the commit scenario (recorded in
//! task-63) opened in the inspector; scrub to before the commit, click the
//! button, and the panel shows the state the agent observed, the request,
//! the refusal with "removed: B", then the acceptance and the sha.

use accord::Millis;
use inspector::InspectorView;

use crate::support::{Fallible, click_control, draw, id, open, row_after, rows, texts};

const SHA: &str = "bf680c840eb2109e90b7ffd9e44e2305a83dbdd2";

/// a press on the scrub where it stands for `at` ms from the start, and
/// its release.
fn scrub_to(view: &mut InspectorView, at: Millis) {
    let theme = view.theme().clone();
    let g = view.strip().geometry(view.layout().strip, &theme);
    let knob = theme.control.slider_knob;
    let ratio = at as f32 / view.strip().span as f32;
    let x = g.scrub.x + knob / 2.0 + (g.scrub.w - knob) * ratio;
    let y = g.scrub.y + g.scrub.h / 2.0;
    view.pointer_move(x, y);
    view.pointer_down(x, y);
    view.pointer_up(x, y);
}

#[test]
fn the_commit_scenario_in_the_inspector() -> Fallible {
    let mut view = open("a2ppa", 1280.0, 800.0)?;
    let submit = id("commit.submit");

    // the commit: request 9, asked at +110.643s; the form (and its
    // button) is gone from the frame once it completes, at +110.688s.
    let commit = view
        .requests()
        .iter()
        .find(|l| l.id.to_string() == "9")
        .ok_or("request 9 in the list")?;
    let (commit_at, commit_action) = (commit.at, commit.action.to_string());
    assert_eq!(commit_action, "submit");
    assert!(
        view.control_rect(&submit).is_none(),
        "no button at the start"
    );

    // scrub to before the commit, after the agent's last look (+103.290s).
    scrub_to(&mut view, 105_000);
    assert!(
        (103_290..commit_at).contains(&view.at()),
        "before the commit: +{}ms",
        view.at()
    );
    let before = view.at();

    // the button is on the frame there: click it.
    click_control(&mut view, &submit)?;
    assert_eq!(view.picked(), Some(&submit));
    assert_eq!(view.at(), before, "a click does not move the instant");

    let rows = rows(&view);
    let text = rows.join("\n");
    // the control at the instant.
    let control = row_after(&rows, 0, &["control commit.submit at +10"]).ok_or(text.clone())?;
    // the stale submit: the state the agent observed, the request, the
    // refusal with the delta.
    let request = row_after(
        &rows,
        control,
        &["request 7 (origin 0): submit on commit.submit by agent:claude-a2ppa"],
    )
    .ok_or(text.clone())?;
    let initiated =
        row_after(&rows, request, &["initiated by", "agent:claude-a2ppa"]).ok_or(text.clone())?;
    let observed = row_after(&rows, initiated, &["observed", "the subtree commit.form"])
        .ok_or(text.clone())?;
    let seen = row_after(
        &rows,
        observed,
        &["changes:staged = {src/login.rs, src/session.rs, tests/login_test.rs}"],
    )
    .ok_or(text.clone())?;
    let refused =
        row_after(&rows, seen, &["verdict", "rejected at +70.679s: stale"]).ok_or(text.clone())?;
    let removed = row_after(
        &rows,
        refused,
        &["what changed", "changes:staged: removed src/session.rs"],
    )
    .ok_or(text.clone())?;
    let acted = row_after(&rows, removed, &["acted", "human, request 6"]).ok_or(text.clone())?;
    // then the acceptance and the sha, after the instant.
    let after = row_after(
        &rows,
        acted,
        &["after +10", "as the whole session tells it"],
    )
    .ok_or(text.clone())?;
    let accepted = row_after(
        &rows,
        after,
        &["request 9 (origin 0): submit on commit.submit by agent:claude-a2ppa"],
    )
    .ok_or(text.clone())?;
    let verdict =
        row_after(&rows, accepted, &["verdict", "accepted at +110.643s"]).ok_or(text.clone())?;
    let sha = row_after(&rows, verdict, &["result", SHA]).ok_or(text.clone())?;
    assert!(sha > verdict && verdict > accepted && accepted > after);

    // what the panel reads is what the window draws: scrolled through,
    // row by row, the panel shows the refusal's delta, then the sha.
    let panel = view.layout().panel;
    let (x, y) = panel.center();
    let step = engine::widgets::content::CodeBlock::row_h(view.theme());
    let mut drawn = texts(&draw(&mut view));
    while view.scroll(x, y, step) {
        drawn.extend(texts(&draw(&mut view)));
    }
    // a row wider than the panel wraps, and the panel draws every character
    // in order over the rows it takes. since the code face is JetBrains Mono
    // (task-104g) the delta's row is 470 px in a column of 454, and the sha's
    // row is cut inside the sha: the drawn text is read without its breaks.
    let drawn: String = drawn.iter().flat_map(|t| t.split_whitespace()).collect();
    let delta = drawn
        .find("changes:staged:removedsrc/session.rs")
        .ok_or("the refusal's delta is drawn")?;
    let sha = drawn.find(SHA).ok_or("the sha is drawn")?;
    assert!(delta < sha, "the refusal before the commit");
    Ok(())
}
