//! The Log view: the lines code said about the frames kept
//! (`engine::message!`), the latest at the bottom: the frame each was
//! said in, its moment on the zones' clock, its thread and its text. The
//! lines of the frame shown are in the text color, the others dimmer.
//! The same lines show in the Zones view as marks on the time axis.

use super::paint::Painter;
use crate::runtime::perf::{FrameReport, Recorder};
use crate::widgets::prelude::Rect;

/// The view in `body`.
pub(super) fn draw(
    p: &mut Painter<'_>,
    body: Rect,
    recorder: &Recorder,
    shown: Option<&FrameReport>,
) {
    let (caption, strong) = (p.caption(), p.strong());
    let (gap, row) = (p.gap(), p.row());
    let said: Vec<_> = recorder
        .history()
        .flat_map(|frame| frame.messages.iter().map(move |m| (frame.index, m)))
        .collect();
    let dropped: u64 = recorder.history().map(|frame| frame.marks_dropped).sum();
    if said.is_empty() {
        let none = "Nothing was said in the frames kept. Code says a line with engine::message!.";
        p.text(none, body.x, body.y, body.w, p.mid(), &caption);
        return;
    }
    let frame_w = p.width("frame 000000", &caption) + gap;
    let at_w = p.width("0000.000 s", &caption) + gap * 2.0;
    let thread_w = said
        .iter()
        .map(|(_, m)| p.width(&m.thread, &caption))
        .fold(p.width("thread", &caption), f32::max)
        .min(body.w / 4.0)
        + gap * 2.0;
    let text_x = body.x + frame_w + at_w + thread_w;
    let mut y = body.y;
    p.text("frame", body.x, y, frame_w, p.mid(), &strong);
    p.text_right(
        "at",
        body.x + frame_w + at_w - gap * 2.0,
        y,
        p.mid(),
        &strong,
    );
    p.text(
        "thread",
        body.x + frame_w + at_w,
        y,
        thread_w,
        p.mid(),
        &strong,
    );
    p.text(
        "message",
        text_x,
        y,
        body.right() - text_x,
        p.mid(),
        &strong,
    );
    y += row;

    let mut room = ((body.bottom() - y) / row).floor() as usize;
    if dropped > 0 {
        room = room.saturating_sub(1);
    }
    let shown = shown.map(|frame| frame.index);
    for (frame, message) in &said[said.len().saturating_sub(room)..] {
        let color = if shown == Some(*frame) {
            p.ink()
        } else {
            p.mid()
        };
        let at = moment(message.at_ns);
        p.text(&frame.to_string(), body.x, y, frame_w, color, &caption);
        p.text_right(&at, body.x + frame_w + at_w - gap * 2.0, y, color, &caption);
        p.text(
            &message.thread,
            body.x + frame_w + at_w,
            y,
            thread_w - gap,
            color,
            &caption,
        );
        p.text(
            &message.text,
            text_x,
            y,
            body.right() - text_x,
            color,
            &caption,
        );
        y += row;
    }
    if dropped > 0 {
        let lost = format!("{dropped} plots and messages over a frame's limit were not kept");
        p.text(&lost, body.x, y, body.w, p.theme.colors.danger.0, &caption);
    }
}

/// A moment on the zones' clock: milliseconds in its first second,
/// seconds after.
fn moment(at_ns: u64) -> String {
    if at_ns < 1_000_000_000 {
        format!("{:.2} ms", at_ns as f64 / 1e6)
    } else {
        format!("{:.3} s", at_ns as f64 / 1e9)
    }
}
