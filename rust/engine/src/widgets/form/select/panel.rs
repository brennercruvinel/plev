//! The open options panel of a [`Select`]: which option rows it shows as
//! they scroll, the wheel and the arrow keys that move them. Where the
//! panel sits, its rows and its drawing are the options list's
//! (`super::list`), which a combobox opens too.

use crate::control::input::scroll::ScrollState;
use crate::drawing::compositor::{Compositor, LayerId};
use crate::styling::theme::Theme;

use super::Select;
use super::list::{self, Row, Window};
use crate::widgets::widget::{EventResult, Rect};

/// Handled with nothing visible changed: the event stops at the panel.
const CONSUMED: EventResult = EventResult {
    handled: true,
    changed: false,
    clicked: false,
};

impl Select {
    /// Dropdown rect below the control.
    pub fn dropdown_rect(&self, bounds: Rect, theme: &Theme) -> Rect {
        list::panel(bounds, self.options.len(), theme)
    }

    /// The scroll as the panel shows it: synced to the options and the
    /// theme and, right after opening, moved so the selected option is
    /// the first row, or as near as the end of the list allows. Pure, so
    /// the render shows the scroll the next event stores
    /// ([`Select::settle`]).
    fn scroll_for(&self, theme: &Theme) -> ScrollState {
        let n = self.options.len();
        let mut scroll = self.scroll.clone();
        list::sync(&mut scroll, n, theme);
        if self.reveal {
            scroll.scroll_to(self.selected as f32 * theme.form.select.panel.row_h);
        }
        if self.reveal_highlight
            && let Some(i) = self.highlighted
        {
            list::reveal(&mut scroll, i, n, theme);
        }
        scroll
    }

    /// Stores the scroll the panel shows, before an event reads or moves
    /// it.
    pub(super) fn settle(&mut self, theme: &Theme) {
        self.scroll = self.scroll_for(theme);
        self.reveal = false;
        self.reveal_highlight = false;
    }

    /// The rows the panel below `bounds` shows. The draw, the binding and
    /// the hit test read this and nothing else.
    fn window(&self, bounds: Rect, theme: &Theme) -> Window {
        let n = self.options.len();
        let dd = list::panel(bounds, n, theme);
        Window::new(dd, n, self.scroll_for(theme).offset(), theme)
    }

    pub(super) fn option_at(&self, x: f32, y: f32, bounds: Rect, theme: &Theme) -> Option<usize> {
        self.window(bounds, theme).at(x, y)
    }

    /// The wheel at `(x, y)` while the panel is open: over the panel it
    /// scrolls the rows, and the highlight follows the row that is now
    /// under the pointer; anywhere else it is not the panel's.
    pub(super) fn scroll_panel(
        &mut self,
        x: f32,
        y: f32,
        delta: f32,
        bounds: Rect,
        theme: &Theme,
    ) -> EventResult {
        if !self.dropdown_rect(bounds, theme).contains(x, y) {
            return EventResult::IGNORED;
        }
        if !delta.is_finite() {
            // The clamp keeps a NaN offset, so the rows would never come back.
            return CONSUMED;
        }
        let before = (self.scroll.offset(), self.highlighted);
        self.scroll.scroll_by(delta);
        self.highlighted = self.option_at(x, y, bounds, theme);
        if (self.scroll.offset(), self.highlighted) != before {
            EventResult::changed()
        } else {
            CONSUMED
        }
    }

    /// Moves the highlight one option down (the arrow down key), from the
    /// highlighted option or else the selected one, and scrolls the panel
    /// the least that shows it whole. The owning view calls it for the
    /// key, as it drives [`Select::set_focused`]. Only an open panel
    /// moves; at the last option the highlight stays.
    pub fn highlight_next(&mut self, theme: &Theme) -> EventResult {
        self.step_highlight(true, theme)
    }

    /// [`Select::highlight_next`] upward (the arrow up key); at the first
    /// option the highlight stays.
    pub fn highlight_prev(&mut self, theme: &Theme) -> EventResult {
        self.step_highlight(false, theme)
    }

    /// Selects the highlighted option and closes the panel, like a click
    /// on its row (the Enter key): `clicked` when the selection changed.
    /// An open panel with nothing highlighted just closes; a closed one
    /// ignores it.
    pub fn pick_highlighted(&mut self) -> EventResult {
        if !self.open {
            return EventResult::IGNORED;
        }
        let pick = self.highlighted.filter(|&i| i < self.options.len());
        let changed = pick.is_some_and(|i| i != self.selected);
        if let Some(i) = pick {
            self.selected = i;
        }
        self.close();
        if changed {
            EventResult::clicked()
        } else {
            EventResult::changed()
        }
    }

    fn step_highlight(&mut self, down: bool, theme: &Theme) -> EventResult {
        let Some(last) = self.options.len().checked_sub(1) else {
            return EventResult::IGNORED;
        };
        if !self.open || self.disabled {
            return EventResult::IGNORED;
        }
        self.settle(theme);
        let from = self.highlighted.unwrap_or(self.selected).min(last);
        let to = if down {
            (from + 1).min(last)
        } else {
            from.saturating_sub(1)
        };
        let before = (self.scroll.offset(), self.highlighted);
        self.highlighted = Some(to);
        list::reveal(&mut self.scroll, to, self.options.len(), theme);
        if (self.scroll.offset(), self.highlighted) != before {
            EventResult::changed()
        } else {
            CONSUMED
        }
    }

    /// Render the open dropdown list (call only when [`is_open`](Select::is_open)).
    pub fn render_dropdown(
        &self,
        compositor: &mut Compositor,
        layer: LayerId,
        bounds: Rect,
        theme: &Theme,
    ) {
        if !self.open {
            return;
        }
        let dd = self.dropdown_rect(bounds, theme);
        list::draw_body(compositor, layer, dd, theme);
        let window = self.window(bounds, theme);
        list::draw_rows(
            compositor,
            layer,
            dd,
            &window,
            |i| Row {
                label: &self.options[i],
                id: self.sem.as_ref().map(|id| id.key(i.to_string())),
                selected: i == self.selected,
                highlighted: self.highlighted == Some(i),
            },
            theme,
        );
    }
}
