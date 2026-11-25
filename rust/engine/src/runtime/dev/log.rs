//! The Log view: the lines code said (`engine::message!`) and the process
//! logged (the `log` crate's macros, through `perf::marks::Tee`), the
//! latest at the bottom: its level, its moment on the zones' clock, the
//! frame it was said in, its thread and its text. A warning and an error
//! stand on a tinted line, as a console shows them.
//!
//! The lines outlive their frames (`Recorder::log`), so the view is not
//! empty a few seconds after something was said. Over them, a count a
//! level that also filters to it, and a clear that hides what was said so
//! far. The wheel scrolls back through the lines.

use super::Act;
use super::paint::{ICON, Painter, alpha};
use crate::runtime::perf::{FrameReport, Level, LogLine, Recorder};
use crate::widgets::prelude::Rect;

/// Lines a notch of the wheel moves, at least.
const NOTCH_PX: f32 = 20.0;

pub(super) struct LogView {
    /// The level shown; every line while `None`.
    level: Option<Level>,
    /// Lines scrolled back from the latest.
    back: usize,
    /// Lines said before this one of everything said are hidden.
    cleared: u64,
    /// Lines ever said, as of the last draw.
    total: u64,
    /// Lines the last draw had to show, at the level chosen.
    pub(super) listed: usize,
}

impl LogView {
    pub fn new() -> Self {
        Self {
            level: None,
            back: 0,
            cleared: 0,
            total: 0,
            listed: 0,
        }
    }

    /// Show the lines of `level` only, or every line.
    pub fn filter(&mut self, level: Option<Level>) {
        self.level = level;
        self.back = 0;
    }

    /// Hide every line said so far.
    pub fn clear(&mut self) {
        self.cleared = self.total;
        self.back = 0;
    }

    /// The wheel: a positive `delta` goes on to the later lines. `true`
    /// when the view moved.
    pub fn scroll(&mut self, delta: f32) -> bool {
        let lines = (delta.abs() / NOTCH_PX).ceil().max(1.0) as usize;
        let back = if delta > 0.0 {
            self.back.saturating_sub(lines)
        } else {
            self.back.saturating_add(lines)
        };
        let moved = back != self.back;
        self.back = back;
        moved
    }

    /// The view in `body`.
    pub fn draw(
        &mut self,
        p: &mut Painter<'_>,
        body: Rect,
        recorder: &Recorder,
        shown: Option<&FrameReport>,
    ) {
        self.total = recorder.log_total();
        let first = self.total - recorder.log().len() as u64;
        let said: Vec<&LogLine> = recorder
            .log()
            .enumerate()
            .filter(|(i, _)| first + *i as u64 >= self.cleared)
            .map(|(_, line)| line)
            .collect();
        let y = self.bar(p, body, &said, recorder.log_lost());
        let area = Rect::new(body.x, y, body.w, body.bottom() - y);
        let lines: Vec<&LogLine> = said
            .iter()
            .copied()
            .filter(|line| self.level.is_none_or(|level| line.message.level == level))
            .collect();
        self.listed = lines.len();
        if lines.is_empty() {
            let why = if said.is_empty() {
                "Nothing said yet. Code says a line with engine::message!, log::warn! or log::error!."
            } else {
                "No line at this level."
            };
            p.empty(area, "terminal", why);
            return;
        }
        self.lines(p, area, &lines, shown.map(|frame| frame.index));
    }

    /// The counts a level, each a filter, and the clear; the y under them.
    fn bar(&self, p: &mut Painter<'_>, body: Rect, said: &[&LogLine], lost: u64) -> f32 {
        let (caption, num) = (p.caption(), p.num());
        let gap = p.gap();
        let h = p.row() + 4.0;
        let text_y = body.y + (h - p.line()) / 2.0;
        let mut x = body.x;
        let mut chip = |p: &mut Painter<'_>, level: Option<Level>, label: &str, n: usize| {
            let icon = level.map(Painter::level_icon);
            let lead = if icon.is_some() {
                ICON + gap / 2.0
            } else {
                0.0
            };
            let count = n.to_string();
            let w = gap + lead + p.width(label, &caption) + gap / 2.0 + p.width(&count, &num) + gap;
            let at = Rect::new(x, body.y, w, h);
            let hot = p.region(at, Act::Level(level));
            let on = self.level == level;
            if on {
                p.chip(at, alpha(p.ink(), 0.08));
            } else if hot {
                p.chip(at, alpha(p.ink(), 0.04));
            }
            let mut text_x = at.x + gap;
            if let (Some(icon), Some(level)) = (icon, level) {
                let color = if n > 0 {
                    p.level_color(level)
                } else {
                    p.faint()
                };
                p.icon(icon, text_x, body.y + (h - ICON) / 2.0, ICON, color);
                text_x += ICON + gap / 2.0;
            }
            let ink = if on || hot { p.ink() } else { p.mid() };
            p.text(label, text_x, text_y, w, ink, &caption);
            text_x += p.width(label, &caption) + gap / 2.0;
            p.text(&count, text_x, text_y, w, p.faint(), &num);
            x += w + gap / 2.0;
        };
        chip(p, None, "All", said.len());
        for level in Level::ALL {
            let n = said.iter().filter(|l| l.message.level == level).count();
            let label = match level {
                Level::Error => "Errors",
                Level::Warn => "Warnings",
                Level::Info => "Info",
            };
            chip(p, Some(level), label, n);
        }
        let clear = Rect::new(body.right() - h, body.y, h, h);
        let hot = p.region(clear, Act::ClearLog);
        if hot {
            p.chip(clear, alpha(p.ink(), 0.07));
        }
        let ink = if hot { p.ink() } else { p.mid() };
        p.icon_in("trash", clear, ICON, ink);
        if lost > 0 {
            let note = format!("{lost} older lines not kept");
            p.text_right(&note, clear.x - gap, text_y, p.faint(), &num);
        }
        body.y + h + gap / 2.0
    }

    /// The lines that fit in `area`, the latest at the bottom.
    fn lines(&mut self, p: &mut Painter<'_>, area: Rect, lines: &[&LogLine], shown: Option<u64>) {
        let (caption, strong, num) = (p.caption(), p.strong(), p.num());
        let gap = p.gap();
        let row = p.row() + 2.0;
        let icon_w = ICON + gap;
        let at_w = p.width("0000.000 s", &num) + gap * 1.5;
        let frame_w = p.width("frame 000000", &num) + gap * 1.5;
        let thread_w = lines
            .iter()
            .map(|line| p.width(&line.message.thread, &caption))
            .fold(p.width("thread", &caption), f32::max)
            .min(area.w / 5.0)
            + gap * 1.5;
        let at_x = area.x + gap / 2.0 + icon_w;
        let frame_x = at_x + at_w;
        let thread_x = frame_x + frame_w;
        let text_x = thread_x + thread_w;

        let head = Rect::new(area.x, area.y, area.w, row);
        p.table_head(head);
        let head_y = head.y + (row - p.line()) / 2.0;
        p.text_right("time", frame_x - gap * 1.5, head_y, p.mid(), &strong);
        p.text("frame", frame_x, head_y, frame_w, p.mid(), &strong);
        p.text("thread", thread_x, head_y, thread_w, p.mid(), &strong);
        p.text(
            "message",
            text_x,
            head_y,
            area.right() - text_x,
            p.mid(),
            &strong,
        );

        let top = head.bottom();
        let room = ((area.bottom() - top) / row).floor() as usize;
        self.back = self.back.min(lines.len().saturating_sub(room));
        let end = lines.len() - self.back;
        let begin = end.saturating_sub(room);
        let mut y = top;
        for line in &lines[begin..end] {
            let message = &line.message;
            let place = Rect::new(area.x, y, area.w, row);
            let color = p.level_color(message.level);
            if message.level != Level::Info {
                p.fill(place, alpha(color, 0.07));
            }
            let text_y = y + (row - p.line()) / 2.0;
            let here = line.frame.is_some() && line.frame == shown;
            let ink = if message.level != Level::Info || here {
                p.ink()
            } else {
                p.mid()
            };
            p.icon(
                Painter::level_icon(message.level),
                area.x + gap / 2.0,
                y + (row - ICON) / 2.0,
                ICON,
                color,
            );
            let at = moment(message.at_ns);
            p.text_right(&at, frame_x - gap * 1.5, text_y, p.faint(), &num);
            let frame = line
                .frame
                .map_or_else(|| "before".to_owned(), |index| format!("frame {index}"));
            p.text(&frame, frame_x, text_y, frame_w - gap, p.faint(), &num);
            p.text(
                &message.thread,
                thread_x,
                text_y,
                thread_w - gap,
                p.faint(),
                &caption,
            );
            p.text(
                &message.text,
                text_x,
                text_y,
                area.right() - text_x - gap / 2.0,
                ink,
                &caption,
            );
            y += row;
            p.fill(
                Rect::new(area.x, y - 1.0, area.w, 1.0),
                alpha(p.ink(), 0.04),
            );
        }
        if self.back > 0 {
            let note = format!("{} later lines below", self.back);
            let w = p.width(&note, &num) + gap * 2.0;
            let at = Rect::new(area.right() - w, area.bottom() - row, w, row);
            p.chip(at, p.theme.colors.bg_panel.0);
            p.text_in(&note, at, p.mid(), &num);
        }
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
