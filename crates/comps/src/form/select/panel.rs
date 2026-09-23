//! The open options panel of a [`Select`]: where it sits, which option
//! rows it shows as they scroll, the wheel and the arrow keys that move
//! them, and its drawing.
//!
//! The rows scroll in the strip of the panel inside the menu padding and
//! are clipped to it. [`Window`] is the one computation of which rows
//! show and where: the draw, the binding and the widget's own hit test
//! all read it, and each row is bound with the rect it is drawn at, under
//! the same clip, so `Compositor::hit` and a click agree at every point
//! of the panel.

use std::ops::Range;

use engine::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use engine::input::scroll::ScrollState;
use engine::text::TextMeasurer;
use engine::theme::Theme;

use super::{Metrics, Select, label_style};
use crate::core::{EventResult, Rect};
use crate::recipe::{glass_pill, menu_shadow};

/// Handled with nothing visible changed: the event stops at the panel.
const CONSUMED: EventResult = EventResult {
    handled: true,
    changed: false,
    clicked: false,
};

/// The option rows the open panel shows: the strip they scroll in, how
/// far they are scrolled, and the rows that reach into the strip.
struct Window {
    view: Rect,
    offset: f32,
    row_h: f32,
    rows: Range<usize>,
}

impl Window {
    /// Where option row `i` is drawn: the panel's full width, one option
    /// tall, scrolled. The strip clips whatever of it reaches past.
    fn row(&self, i: usize) -> Rect {
        Rect::new(
            self.view.x,
            self.view.y + i as f32 * self.row_h - self.offset,
            self.view.w,
            self.row_h,
        )
    }

    /// The shown row under `(x, y)`, found the way `Compositor::hit` finds
    /// a bound one: inside the clip and the row, edges included, the later
    /// row on top where two rows share an edge.
    fn at(&self, x: f32, y: f32) -> Option<usize> {
        if !self.view.contains(x, y) {
            return None;
        }
        self.rows
            .clone()
            .rev()
            .find(|&i| self.row(i).contains(x, y))
    }
}

impl Select {
    /// The height of the strip the rows scroll in: every option, never
    /// more than `size.dropdown_max_h` leaves inside the menu padding.
    fn view_h(&self, m: &Metrics) -> f32 {
        let fit = (m.max_h - m.pad_y * 2.0).max(0.0);
        (self.options.len() as f32 * m.option_h).min(fit)
    }

    /// The panel below the control `bounds`: the strip and the padding.
    fn panel(&self, bounds: Rect, m: &Metrics) -> Rect {
        Rect::new(
            bounds.x,
            bounds.y + bounds.h + m.gap,
            bounds.w,
            self.view_h(m) + m.pad_y * 2.0,
        )
    }

    /// Dropdown rect below the control.
    pub fn dropdown_rect(&self, bounds: Rect, theme: &Theme) -> Rect {
        self.panel(bounds, &Metrics::of(theme))
    }

    /// The scroll as the panel shows it: synced to the options and the
    /// theme and, right after opening, moved so the selected option is
    /// the first row, or as near as the end of the list allows. Pure, so
    /// the render shows the scroll the next event stores
    /// ([`Select::settle`]).
    fn scroll_for(&self, m: &Metrics) -> ScrollState {
        let mut scroll = self.scroll.clone();
        scroll.set_content(self.options.len() as f32 * m.option_h);
        scroll.set_viewport(self.view_h(m));
        if self.reveal {
            scroll.scroll_to(self.selected as f32 * m.option_h);
        }
        scroll
    }

    /// Stores the scroll the panel shows, before an event reads or moves
    /// it.
    pub(super) fn settle(&mut self, m: &Metrics) {
        self.scroll = self.scroll_for(m);
        self.reveal = false;
    }

    /// The rows the panel `dd` shows. The draw, the binding and the hit
    /// test read this and nothing else.
    fn window(&self, dd: Rect, m: &Metrics) -> Window {
        let view = Rect::new(dd.x, dd.y + m.pad_y, dd.w, self.view_h(m));
        let offset = self.scroll_for(m).offset();
        let n = self.options.len();
        let rows = if view.h > 0.0 && m.option_h > 0.0 {
            let first = (offset / m.option_h).floor() as usize;
            let end = ((offset + view.h) / m.option_h).ceil() as usize;
            first.min(n)..end.min(n)
        } else {
            0..0
        };
        Window {
            view,
            offset,
            row_h: m.option_h,
            rows,
        }
    }

    pub(super) fn option_at(&self, x: f32, y: f32, bounds: Rect, m: &Metrics) -> Option<usize> {
        self.window(self.panel(bounds, m), m).at(x, y)
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
        m: &Metrics,
    ) -> EventResult {
        if !self.panel(bounds, m).contains(x, y) {
            return EventResult::IGNORED;
        }
        if !delta.is_finite() {
            // The clamp keeps a NaN offset, so the rows would never come back.
            return CONSUMED;
        }
        let before = (self.scroll.offset(), self.highlighted);
        self.scroll.scroll_by(delta);
        self.highlighted = self.option_at(x, y, bounds, m);
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
        let m = Metrics::of(theme);
        self.settle(&m);
        let from = self.highlighted.unwrap_or(self.selected).min(last);
        let to = if down {
            (from + 1).min(last)
        } else {
            from.saturating_sub(1)
        };
        let before = (self.scroll.offset(), self.highlighted);
        self.highlighted = Some(to);
        self.scroll_into_view(to, &m);
        if (self.scroll.offset(), self.highlighted) != before {
            EventResult::changed()
        } else {
            CONSUMED
        }
    }

    /// Scrolls the least that shows option `i` whole inside the strip.
    fn scroll_into_view(&mut self, i: usize, m: &Metrics) {
        let top = i as f32 * m.option_h;
        let bottom = top + m.option_h;
        let view_h = self.view_h(m);
        if top < self.scroll.offset() {
            self.scroll.scroll_to(top);
        } else if bottom > self.scroll.offset() + view_h {
            self.scroll.scroll_to(bottom - view_h);
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
        let m = Metrics::of(theme);
        let dd = self.panel(bounds, &m);
        let glass = &theme.glass;

        // Floating panel: solid popover body, edge-light, deep shadow.
        // No path icons inside (the active marker is an SDF dot), so the
        // whole panel can use the gradient edge-light recipe.
        let radius = theme.shape.card;
        compositor.push_to_layer(layer, menu_shadow(dd, radius, theme));
        for node in glass_pill(
            dd,
            radius,
            glass.edge_soft.0,
            theme.control.edge_width_strong,
            glass.popover.0,
        ) {
            compositor.push_to_layer(layer, node);
        }

        let style = label_style(theme);
        let window = self.window(dd, &m);
        let view = window.view;
        // The rows are clipped to the strip and bound under that clip, so
        // `hit` finds only the part of a row that shows.
        compositor.push_to_layer(
            layer,
            SceneNode::PushClip {
                x: view.x,
                y: view.y,
                w: view.w,
                h: view.h,
            },
        );
        let rows = window.rows.clone();
        for (i, option) in self
            .options
            .iter()
            .enumerate()
            .take(rows.end)
            .skip(rows.start)
        {
            let row = window.row(i);
            let oy = row.y;
            let is_selected = i == self.selected;
            let is_hovered = self.highlighted == Some(i);
            if let Some(id) = &self.sem {
                compositor.bind_to_layer(layer, &id.key(i.to_string()), [row.x, oy, row.w, row.h]);
            }

            if is_hovered || is_selected {
                // Option: radius 12, active bg rgba($n2,.10).
                compositor.push_to_layer(
                    layer,
                    SceneNode::RoundedRect {
                        x: dd.x + m.pad_y,
                        y: oy,
                        w: dd.w - m.pad_y * 2.0,
                        h: m.option_h,
                        color: if is_selected {
                            glass.surface_active.0
                        } else {
                            glass.surface_hover.0
                        },
                        corner_radius: theme.shape.nav.min(m.option_h / 2.0),
                        border_width: 0.0,
                        border_color: [0.0; 4],
                    },
                );
            }
            // base-2m: text-default at rest, text-active hovered/selected.
            let label = if is_hovered || is_selected {
                glass.text_active
            } else {
                glass.text_default
            };
            compositor.push_to_layer(
                layer,
                SceneNode::Text {
                    key: TextNodeKey::from_style(option, &style, None),
                    x: dd.x + m.pad_x,
                    y: oy + TextMeasurer::vertical_center(&style, m.option_h),
                    color: label.0,
                },
            );
            if is_selected {
                // A dot on the right marks the active option.
                compositor.push_to_layer(
                    layer,
                    SceneNode::RoundedRect {
                        x: dd.x + dd.w - m.pad_x - m.dot,
                        y: oy + (m.option_h - m.dot) / 2.0,
                        w: m.dot,
                        h: m.dot,
                        color: theme.colors.text.0,
                        corner_radius: m.dot / 2.0,
                        border_width: 0.0,
                        border_color: [0.0; 4],
                    },
                );
            }
        }
        compositor.push_to_layer(layer, SceneNode::PopClip);
    }
}
