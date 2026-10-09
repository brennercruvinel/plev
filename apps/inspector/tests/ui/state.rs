//! the view's model: the instant, the control picked, what the panel
//! says. every answer the panel shows is the model's, called, not worked
//! out again.

use engine::drawing::text::TextMeasurer;
use engine::widgets::content::{CodeBlock, CodeLine, LineKind};
use inspector::{Key, Tone, wrap};
use replay::{Inspector, prose};

use crate::support::{Fallible, click, click_control, draw, id, open, texts};

#[test]
fn the_instant_moves_by_the_scrub_and_by_event() -> Fallible {
    let mut view = open("a2ppa", 1280.0, 800.0)?;
    let (start, end) = view.span();
    assert_eq!(view.t(), start, "it opens at the start");
    assert_eq!(view.at(), 0);
    assert_eq!(view.strip().span, end - start);

    // the keys step from one recorded instant to the next, and back.
    let instants = view.strip().instants.clone();
    assert!(instants.len() > 10, "{instants:?}");
    assert!(view.key(Key::Next));
    assert_eq!(view.at(), instants[1]);
    assert!(view.key(Key::Next));
    assert_eq!(view.at(), instants[2]);
    assert!(view.key(Key::Previous));
    assert_eq!(view.at(), instants[1]);
    assert!(view.key(Key::Last));
    assert_eq!(view.t(), end);
    assert!(!view.key(Key::Next), "nothing after the end");
    assert!(view.key(Key::First));
    assert_eq!(view.t(), start);

    // a seek outside the session stays inside it.
    view.seek(end + 10_000);
    assert_eq!(view.t(), end);
    view.seek(0);
    assert_eq!(view.t(), start);

    // a press on the lanes moves the instant to the time under it.
    let theme = view.theme().clone();
    let g = view.strip().geometry(view.layout().strip, &theme);
    let (x, y) = (g.track.x + g.track.w / 2.0, g.track.y + g.track.h / 2.0);
    click(&mut view, x, y);
    let half = view.strip().span / 2;
    assert!(
        view.at().abs_diff(half) <= view.strip().span / 100,
        "{}",
        view.at()
    );
    Ok(())
}

#[test]
fn the_frame_is_the_one_on_screen_at_the_instant() -> Fallible {
    let mut view = open("a2ppa", 1280.0, 800.0)?;
    let session = record::Session::open(crate::support::fixture("a2ppa"))?;
    for t in session.frame_times() {
        view.seek(t);
        assert_eq!(view.frame_time(), session.frame_at(t));
        assert_eq!(Some(view.bound().to_vec()), session.bindings_at(t));
        let fit = view.fit().ok_or("a frame")?;
        assert_eq!(fit.extent, (1280.0, 800.0), "the window the ide drew");
        let frame = fit.frame();
        let into = view.layout().frame;
        assert!(frame.w <= into.w + 0.01 && frame.h <= into.h + 0.01);
        for b in view.bound() {
            let r = fit.rect(b.rect);
            assert!(r.x >= frame.x - 0.01 && r.right() <= frame.right() + 0.01);
            assert!(r.y >= frame.y - 0.01 && r.bottom() <= frame.bottom() + 0.01);
        }
    }
    // the header says the instant and the frame.
    view.seek(session.frame_times()[5]);
    let drawn = texts(&draw(&mut view));
    let stamp = prose::stamp(view.at());
    assert!(
        drawn.iter().any(|t| t.contains(&format!("{stamp} of "))),
        "{stamp} in {drawn:?}"
    );
    Ok(())
}

#[test]
fn the_pointer_names_the_control_under_it() -> Fallible {
    let mut view = open("a2ppa", 1280.0, 800.0)?;
    let session = record::Session::open(crate::support::fixture("a2ppa"))?;
    let t = session.frame_times()[5];
    view.seek(t);
    let fit = view.fit().ok_or("a frame")?;
    for b in view.bound().to_vec() {
        let (x, y) = fit.rect(b.rect).center();
        view.pointer_move(x, y);
        let (fx, fy) = fit.to_frame(x, y).ok_or("inside")?;
        assert_eq!(view.hovered(), session.hit_at(t, fx, fy).as_ref());
        let under = view
            .hovered()
            .ok_or("a control under the pointer")?
            .to_string();
        assert!(texts(&draw(&mut view)).contains(&under), "its id is shown");
    }
    // off the frame, nothing is under the pointer.
    let panel = view.layout().panel;
    view.pointer_move(panel.x + 1.0, panel.y + 1.0);
    assert_eq!(view.hovered(), None);
    Ok(())
}

#[test]
fn a_click_picks_and_the_panel_is_the_models() -> Fallible {
    let mut view = open("a2ppa", 1280.0, 800.0)?;
    let session = record::Session::open(crate::support::fixture("a2ppa"))?;
    let inspector = Inspector::new(&session);
    let message = id("commit.message");
    view.seek(session.frame_times()[6]);
    click_control(&mut view, &message)?;
    assert_eq!(view.picked(), Some(&message));

    let panel = view.panel().ok_or("a panel")?;
    let control = inspector.control(&message, view.t())?;
    assert_eq!(panel.at, control, "the model's answer at the instant");
    let written = prose::control(&control);
    let written: Vec<&str> = written.lines().collect();
    let rows: Vec<&str> = view.rows().iter().map(|r| r.text.as_str()).collect();
    assert_eq!(rows.get(..written.len()), Some(written.as_slice()));

    // another instant: the same control, answered at it.
    view.key(Key::Last);
    let panel = view.panel().ok_or("a panel")?;
    assert_eq!(panel.at, inspector.control(&message, view.t())?);
    assert!(panel.later.is_empty(), "nothing comes after the end");

    // escape drops the pick; so does a click on the frame over no control.
    assert!(view.key(Key::Close));
    assert_eq!(view.picked(), None);
    assert!(view.rows().is_empty());
    view.seek(session.frame_times()[6]);
    click_control(&mut view, &message)?;
    let fit = view.fit().ok_or("a frame")?;
    let frame = fit.frame();
    let bare = (0..40)
        .flat_map(|i| (0..40).map(move |j| (i, j)))
        .map(|(i, j)| {
            (
                frame.x + frame.w * (i as f32 + 0.5) / 40.0,
                frame.y + frame.h * (j as f32 + 0.5) / 40.0,
            )
        })
        .find(|&(x, y)| {
            fit.to_frame(x, y)
                .is_some_and(|(fx, fy)| session.hit_at(view.t(), fx, fy).is_none())
        })
        .ok_or("a point over no control")?;
    click(&mut view, bare.0, bare.1);
    assert_eq!(view.picked(), None);
    Ok(())
}

#[test]
fn a_request_of_the_list_goes_to_its_instant_and_its_control() -> Fallible {
    let mut view = open("a2ppa", 1280.0, 800.0)?;
    let lines = view.requests().to_vec();
    assert_eq!(lines.len(), 9);
    for (i, line) in lines.iter().enumerate() {
        assert!(view.go_to_request(i));
        assert_eq!(view.t(), line.t);
        assert_eq!(view.picked(), Some(&line.target));
        let rows: Vec<&str> = view.rows().iter().map(|r| r.text.as_str()).collect();
        let head = format!(
            "request {} (origin 0): {} on {}",
            line.id, line.action, line.target
        );
        assert!(rows.iter().any(|r| r.contains(&head)), "{head} in {rows:?}");
    }
    assert!(!view.go_to_request(lines.len()));
    Ok(())
}

#[test]
fn the_strip_puts_every_event_on_its_actors_lane() -> Fallible {
    let view = open("a2ppa", 1280.0, 800.0)?;
    let session = record::Session::open(crate::support::fixture("a2ppa"))?;
    let timeline = Inspector::new(&session).timeline();
    let strip = view.strip();
    assert_eq!(strip.marks.len(), timeline.entries.len(), "every event");
    assert_eq!(
        strip.lanes.len(),
        timeline.actors.len() + 1,
        "and the state"
    );
    assert!(strip.lanes[0].starts_with("agent:claude-a2ppa: 7 observed, 8 asked"));
    assert!(strip.lanes[1].starts_with("human: 1 asked, 1 input"));
    let tones = |tone| strip.marks.iter().filter(|m| m.tone == tone).count();
    assert_eq!(tones(Tone::Observed), 7);
    assert_eq!(tones(Tone::Refused), 1, "the stale submit");
    assert_eq!(tones(Tone::Done), 8);
    let chips: Vec<&str> = strip.chips.iter().map(|c| c.label.as_str()).collect();
    for count in &timeline.kinds {
        assert!(chips.contains(&format!("{} {}", count.kind, count.count).as_str()));
    }
    Ok(())
}

#[test]
fn a_long_row_wraps_and_keeps_every_character() {
    let theme = engine::widgets::Theme::default();
    let style = CodeBlock::text_style(&theme);
    let rows = vec![
        CodeLine::new(
            "    result        completed: commit = \"bf680c840eb2109e90b7ffd9e44e2305a83dbdd2\", files = [src/login.rs, src/session.rs, tests/login_test.rs]",
            LineKind::Context,
        ),
        CodeLine::new("control commit.submit at +95.084s", LineKind::Header),
    ];
    for width in [120.0, 220.0, 400.0] {
        let wrapped = wrap(&rows, width, &style);
        assert!(wrapped.len() > rows.len(), "{width}: {wrapped:?}");
        for w in &wrapped {
            assert!(
                TextMeasurer::measure_styled(&w.text, &style, None).0 <= width,
                "{width}: {:?}",
                w.text
            );
        }
        let squeeze = |s: &str| s.chars().filter(|c| !c.is_whitespace()).collect::<String>();
        let before: String = rows.iter().map(|r| squeeze(&r.text)).collect();
        let after: String = wrapped.iter().map(|r| squeeze(&r.text)).collect();
        assert_eq!(before, after, "{width}");
        assert_eq!(wrapped.last().map(|r| r.kind), Some(LineKind::Header));
    }
}
