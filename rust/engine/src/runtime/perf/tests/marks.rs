//! Plots and messages: the last value under a name, a message's text,
//! thread and moment, the frame's limits, and the off path.
//!
//! Marks are process wide, so every test holds `zones::test_lock`. The
//! engine says messages of its own (a layer made, the atlas grown), which
//! another test's thread can say while marks are on here: a test reads
//! the lines it said itself.

use std::sync::MutexGuard;

use super::super::marks::{self, MAX_MESSAGES_PER_FRAME, Message, PlotPoint};
use super::super::zones;

/// Marks on for one test, off again (and drained) when it ends.
struct On {
    _lock: MutexGuard<'static, ()>,
}

impl On {
    fn new() -> Self {
        let lock = zones::test_lock();
        marks::enable(false);
        marks::enable(true);
        let _ = marks::take();
        On { _lock: lock }
    }
}

impl Drop for On {
    fn drop(&mut self) {
        marks::enable(false);
    }
}

/// The lines of the take that start with `mark`.
fn said(mark: &str) -> Vec<Message> {
    let frame = marks::take();
    frame
        .messages
        .into_iter()
        .filter(|m| m.text.starts_with(mark))
        .collect()
}

#[test]
fn off_keeps_nothing_and_never_builds_the_text() {
    let _lock = zones::test_lock();
    marks::enable(false);
    let mut built = false;
    marks::message(|| {
        built = true;
        String::from("marks-off")
    });
    crate::plot!("marks-off", 1);
    assert!(!built, "the text was built with marks off");
    let frame = marks::take();
    assert!(frame.plots.is_empty());
    assert!(frame.messages.iter().all(|m| m.text != "marks-off"));
}

#[test]
fn a_plot_keeps_the_last_value_under_its_name() {
    let _on = On::new();
    crate::plot!("marks-nodes", 3);
    crate::plot!("marks-glyphs", 9u32);
    crate::plot!("marks-nodes", 5.5);
    let frame = marks::take();
    assert_eq!(
        frame.plots,
        vec![
            PlotPoint {
                name: "marks-nodes",
                value: 5.5
            },
            PlotPoint {
                name: "marks-glyphs",
                value: 9.0
            },
        ]
    );
    assert!(
        marks::take().plots.is_empty(),
        "a take starts the next empty"
    );
}

#[test]
fn a_message_keeps_its_text_its_thread_and_its_moment() {
    let _on = On::new();
    crate::message!("marks-say first {}", 1);
    crate::message!("marks-say second");
    let mine = said("marks-say");
    let texts: Vec<&str> = mine.iter().map(|m| m.text.as_str()).collect();
    assert_eq!(texts, ["marks-say first 1", "marks-say second"]);
    assert!(mine[0].at_ns <= mine[1].at_ns);
    let here = std::thread::current();
    assert_eq!(Some(mine[0].thread.as_str()), here.name());
}

#[test]
fn a_frame_keeps_at_most_its_limit_and_counts_the_rest() {
    let _on = On::new();
    for i in 0..MAX_MESSAGES_PER_FRAME + 3 {
        crate::message!("marks-many {i}");
    }
    let frame = marks::take();
    assert_eq!(frame.messages.len(), MAX_MESSAGES_PER_FRAME);
    assert!(frame.dropped >= 3, "dropped {}", frame.dropped);
}

#[test]
fn turning_off_drops_what_was_said() {
    let _on = On::new();
    crate::message!("marks-drop");
    crate::plot!("marks-drop", 1);
    marks::enable(false);
    let frame = marks::take();
    assert!(frame.plots.is_empty());
    assert!(said("marks-drop").is_empty());
}
