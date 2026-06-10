use crate::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::theme::Theme;
use crate::ui::icons;

use super::{EventResult, Rect, WidgetEvent, with_alpha};

const FONT: f32 = 13.0;
const OPTION_H: f32 = 28.0;
const PAD_X: f32 = 10.0;
const PAD_Y: f32 = 5.0;
const CHEVRON: f32 = 14.0;
const DROPDOWN_GAP: f32 = 4.0;
const MAX_DROPDOWN_H: f32 = 280.0;

/// Dropdown select. The closed control renders in normal flow; while
/// open, render the dropdown via [`render_dropdown`](Select::render_dropdown)
/// onto an overlay layer (it draws below the control bounds).
#[derive(Clone, Debug)]
pub struct Select {
    pub options: Vec<String>,
    pub selected: usize,
    pub disabled: bool,
    open: bool,
    hovered: bool,
    hovered_option: Option<usize>,
}

impl Select {
    pub fn new(options: impl IntoIterator<Item = impl Into<String>>, selected: usize) -> Self {
        let options: Vec<String> = options.into_iter().map(Into::into).collect();
        let selected = selected.min(options.len().saturating_sub(1));
        Self {
            options,
            selected,
            disabled: false,
            open: false,
            hovered: false,
            hovered_option: None,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn close(&mut self) {
        self.open = false;
        self.hovered_option = None;
    }

    pub fn selected_label(&self) -> Option<&str> {
        self.options.get(self.selected).map(String::as_str)
    }

    /// Dropdown rect below the control.
    pub fn dropdown_rect(&self, bounds: Rect) -> Rect {
        let h = (PAD_Y * 2.0 + self.options.len() as f32 * OPTION_H).min(MAX_DROPDOWN_H);
        Rect::new(bounds.x, bounds.y + bounds.h + DROPDOWN_GAP, bounds.w, h)
    }

    fn option_at(&self, x: f32, y: f32, bounds: Rect) -> Option<usize> {
        let dd = self.dropdown_rect(bounds);
        if !dd.contains(x, y) {
            return None;
        }
        let i = ((y - dd.y - PAD_Y) / OPTION_H).floor();
        if i < 0.0 {
            return None;
        }
        let i = i as usize;
        (i < self.options.len()).then_some(i)
    }

    pub fn handle_event(&mut self, event: &WidgetEvent, bounds: Rect) -> EventResult {
        if self.disabled {
            if self.hovered || self.open {
                self.hovered = false;
                self.close();
                return EventResult::changed();
            }
            return EventResult::IGNORED;
        }
        match *event {
            WidgetEvent::MouseMove { x, y } => {
                let mut result = EventResult::IGNORED;
                let inside = bounds.contains(x, y);
                if inside != self.hovered {
                    self.hovered = inside;
                    result = EventResult::changed();
                }
                if self.open {
                    let hit = self.option_at(x, y, bounds);
                    if hit != self.hovered_option {
                        self.hovered_option = hit;
                        result = result.merge(EventResult::changed());
                    }
                }
                result
            }
            WidgetEvent::MouseDown { x, y } => {
                if self.open {
                    if let Some(i) = self.option_at(x, y, bounds) {
                        let changed = i != self.selected;
                        self.selected = i;
                        self.close();
                        return if changed {
                            EventResult::clicked()
                        } else {
                            EventResult::changed()
                        };
                    }
                    // Click anywhere else closes the dropdown. Re-clicking
                    // the control itself just toggles closed.
                    self.close();
                    return EventResult::changed();
                }
                if bounds.contains(x, y) {
                    self.open = true;
                    EventResult::changed()
                } else {
                    EventResult::IGNORED
                }
            }
            _ => EventResult::IGNORED,
        }
    }

    /// Render the closed control (selected value + chevron).
    pub fn render(&self, compositor: &mut Compositor, bounds: Rect, theme: &Theme) {
        let alpha = if self.disabled { 0.5 } else { 1.0 };
        let border = if self.open {
            theme.colors.border_active
        } else if self.hovered {
            theme.colors.border_active
        } else {
            theme.colors.divider
        };
        let bg = if self.hovered && !self.open {
            with_alpha(theme.colors.bg_hover, alpha)
        } else {
            with_alpha(theme.colors.surface, alpha)
        };

        compositor.push(SceneNode::RoundedRect {
            x: bounds.x,
            y: bounds.y,
            w: bounds.w,
            h: bounds.h,
            color: bg,
            corner_radius: theme.radius.md + 2.0,
            border_width: 1.0,
            border_color: with_alpha(border, alpha),
        });

        if let Some(label) = self.selected_label() {
            let line_height = FONT * 1.3;
            compositor.push(SceneNode::Text {
                key: TextNodeKey::new(label, FONT, line_height, None),
                x: bounds.x + PAD_X,
                y: bounds.y + (bounds.h - line_height) / 2.0,
                color: with_alpha(theme.colors.text, alpha),
            });
        }

        let chevron = if self.open {
            "chevron-up"
        } else {
            "chevron-down"
        };
        if let Some(node) = icons::icon_at(
            chevron,
            CHEVRON,
            with_alpha(theme.colors.text_dim, alpha),
            bounds.x + bounds.w - PAD_X - CHEVRON,
            bounds.y + (bounds.h - CHEVRON) / 2.0,
        ) {
            compositor.push(node);
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
        let dd = self.dropdown_rect(bounds);

        compositor.push_to_layer(
            layer,
            SceneNode::RoundedRect {
                x: dd.x,
                y: dd.y,
                w: dd.w,
                h: dd.h,
                color: with_alpha(theme.colors.bg_panel, 0.99),
                corner_radius: theme.radius.md + 2.0,
                border_width: 1.0,
                border_color: with_alpha(theme.colors.divider, 1.0),
            },
        );

        let line_height = FONT * 1.3;
        for (i, option) in self.options.iter().enumerate() {
            let oy = dd.y + PAD_Y + i as f32 * OPTION_H;
            if oy + OPTION_H > dd.y + dd.h {
                break;
            }
            let is_selected = i == self.selected;
            let is_hovered = self.hovered_option == Some(i);

            if is_hovered {
                compositor.push_to_layer(
                    layer,
                    SceneNode::RoundedRect {
                        x: dd.x + 4.0,
                        y: oy + 1.0,
                        w: dd.w - 8.0,
                        h: OPTION_H - 2.0,
                        color: with_alpha(theme.colors.bg_hover, 1.0),
                        corner_radius: theme.radius.md,
                        border_width: 0.0,
                        border_color: [0.0; 4],
                    },
                );
            }
            compositor.push_to_layer(
                layer,
                SceneNode::Text {
                    key: TextNodeKey::new(option, FONT, line_height, None)
                        .with_weight(if is_selected { 600 } else { 400 }),
                    x: dd.x + PAD_X,
                    y: oy + (OPTION_H - line_height) / 2.0,
                    color: with_alpha(theme.colors.text, 1.0),
                },
            );
            if is_selected
                && let Some(node) = icons::icon_at(
                    "check",
                    12.0,
                    with_alpha(theme.colors.accent, 1.0),
                    dd.x + dd.w - PAD_X - 12.0,
                    oy + (OPTION_H - 12.0) / 2.0,
                )
            {
                compositor.push_to_layer(layer, node);
            }
        }
    }
}
