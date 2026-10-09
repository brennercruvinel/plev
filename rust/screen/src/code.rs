//! a `code` on the comps `CodeBlock` (task-97): its `lines` (a list, or
//! one text split at its line breaks) are the block's rows, each with the
//! kind its `kinds` names at the same place (`plevir::LineKind` by name; a
//! name it does not know, a place `kinds` does not reach, or no `kinds`
//! at all is a context row), and its label is the caption.
//!
//! the block is as tall as its rows; `size.max` bounds it, and then the
//! rows scroll inside it: the wheel over it moves them, the scrollbar the
//! fixed diff panel uses shows while they move and can be dragged. new
//! lines (another diff) start at the top. the offset is the widget's,
//! never projected: it changes what shows, not what anyone may do.

use accord::{Snapshot, Value};
use engine::control::input::scroll::ScrollState;
use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;
use engine::widgets::content::{CodeBlock, CodeLine};
use engine::widgets::feedback::Scrollbar;
use engine::widgets::{EventResult, Rect, Wake, WidgetEvent};
use plevir::LineKind;

use crate::expand::Node;
use crate::token;
use crate::widget::{list_of, text_of};

/// the rows `node` shows over `shown`.
pub(crate) fn rows(node: &Node, shown: &Snapshot) -> Vec<CodeLine> {
    let lines: Vec<String> = match node.value("lines", shown) {
        Some(Value::Text(text)) => text.lines().map(str::to_owned).collect(),
        Some(Value::List(_) | Value::Set(_)) => list_of(node.value("lines", shown)),
        Some(other) => vec![text_of(other)],
        None => Vec::new(),
    };
    let kinds = list_of(node.value("kinds", shown));
    lines
        .into_iter()
        .enumerate()
        .map(|(at, text)| {
            let kind = kinds
                .get(at)
                .and_then(|name| LineKind::parse(name))
                .unwrap_or_default();
            CodeLine::new(text, token::line_kind(kind))
        })
        .collect()
}

/// the block `node` draws over `shown`, scrolled to the top.
pub(crate) fn block(node: &Node, shown: &Snapshot) -> CodeBlock {
    let mut block = CodeBlock::lines(rows(node, shown));
    block.caption.clone_from(&node.label);
    block
}

/// the retained widget of a `code`: the block, and where its rows are
/// scrolled to.
#[derive(Debug)]
pub(crate) struct Code {
    block: CodeBlock,
    scroll: ScrollState,
    bar: Scrollbar,
}

impl Code {
    pub fn new() -> Code {
        Code {
            block: CodeBlock::lines(Vec::new()),
            scroll: ScrollState::new(),
            bar: Scrollbar::new(),
        }
    }

    /// the rows and the caption `node` shows over `shown`; new rows start
    /// at the top.
    pub fn sync(&mut self, node: &Node, shown: &Snapshot) {
        let rows = rows(node, shown);
        if rows != self.block.rows {
            self.block.rows = rows;
            self.scroll.scroll_to(0.0);
        }
        if self.block.caption != node.label {
            self.block.caption.clone_from(&node.label);
        }
    }

    /// the height the block takes with every row showing.
    pub fn height(&self, theme: &Theme) -> f32 {
        self.block.height_for(0.0, theme)
    }

    /// the wheel over the block drawn at `rect` moves its rows; the
    /// pointer drags its scrollbar.
    pub fn handle(&mut self, event: &WidgetEvent, rect: Rect, theme: &Theme) -> EventResult {
        let area = self.block.rows_rect(rect, theme);
        if let WidgetEvent::Scroll { x, y, delta } = *event {
            if !rect.contains(x, y) || !self.scroll.is_scrollable() {
                return EventResult::IGNORED;
            }
            let old = self.scroll.offset();
            self.scroll.scroll_by(delta);
            self.bar.notify_scroll();
            return if self.scroll.offset() != old {
                EventResult::changed()
            } else {
                EventResult {
                    handled: true,
                    ..EventResult::IGNORED
                }
            };
        }
        self.bar.handle_event(event, area, &mut self.scroll, theme)
    }

    /// the scrollbar fading in and out.
    pub fn wake(&mut self, dt: f32) -> Wake {
        Wake::moving(self.bar.tick(dt))
    }

    /// the block at `rect`, its rows at the offset, the scrollbar over
    /// them.
    pub fn draw(&mut self, c: &mut Compositor, rect: Rect, theme: &Theme) {
        let area = self.block.rows_rect(rect, theme);
        self.scroll.set_viewport(area.h);
        self.scroll.set_content(self.block.content_h(theme));
        self.block.scroll = self.scroll.offset();
        self.block.render(c, rect, theme);
        self.bar.render(c, area, &self.scroll, theme);
    }
}
