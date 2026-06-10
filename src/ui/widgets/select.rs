use crate::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::theme::Theme;
use crate::ui::icons;

use super::{EventResult, Rect, WidgetEvent, glass_pill, menu_shadow, with_alpha};

/// HOFF select: 44px pill control (radius 22, base-2m label), options
/// panel radius 20 / pad 8, 44px options (radius 12) with an 8px dot
/// marking the active one.
const FONT: f32 = 14.0;
const OPTION_H: f32 = 44.0;
const PAD_X: f32 = 16.0;
const PAD_Y: f32 = 8.0;
const CHEVRON: f32 = 16.0;
const DROPDOWN_GAP: f32 = 4.0;
const MAX_DROPDOWN_H: f32 = 320.0;
const DOT: f32 = 8.0;

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
        let glass = &theme.glass;

        // Glass pill: rgba($n2,.05), hover .10, focus border rgba($n2,.25).
        let bg = if self.hovered || self.open {
            with_alpha(glass.surface_active, glass.surface_active.0[3] * alpha)
        } else {
            with_alpha(glass.field, glass.field.0[3] * alpha)
        };
        let edge = if self.open {
            with_alpha(
                glass.field_focus_border,
                glass.field_focus_border.0[3] * alpha,
            )
        } else {
            with_alpha(glass.edge_soft, glass.edge_soft.0[3] * alpha)
        };

        // Glass field; the chevron icon pushed later stacks on top (the
        // compositor preserves push order across primitive types).
        let radius = theme.radius.xl.min(bounds.h / 2.0);
        compositor.push(super::rounded_rect(
            bounds.x, bounds.y, bounds.w, bounds.h, radius, bg,
        ));
        compositor.push(super::rounded_rect_stroke(
            bounds.x, bounds.y, bounds.w, bounds.h, radius, edge, 1.5,
        ));

        if let Some(label) = self.selected_label() {
            let line_height = FONT * 1.4;
            // base-2m at rgba($n2,.76).
            let text = theme.colors.text;
            compositor.push(SceneNode::Text {
                key: TextNodeKey::new(label, FONT, line_height, None).with_weight(500),
                x: bounds.x + PAD_X,
                y: bounds.y + (bounds.h - line_height) / 2.0,
                color: with_alpha(text, text.0[3] * 0.8 * alpha),
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
            with_alpha(glass.text_faint, glass.text_faint.0[3] * alpha),
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
        let glass = &theme.glass;
        let text = theme.colors.text;

        // Floating panel: solid popover body, edge-light, deep shadow.
        // No path icons inside (the active marker is an SDF dot), so the
        // whole panel can use the gradient edge-light recipe.
        let radius = theme.radius.lg;
        compositor.push_to_layer(layer, menu_shadow(dd, radius));
        for node in glass_pill(dd, radius, glass.edge_soft.0, 1.5, glass.popover.0) {
            compositor.push_to_layer(layer, node);
        }

        let line_height = FONT * 1.4;
        for (i, option) in self.options.iter().enumerate() {
            let oy = dd.y + PAD_Y + i as f32 * OPTION_H;
            if oy + OPTION_H > dd.y + dd.h {
                break;
            }
            let is_selected = i == self.selected;
            let is_hovered = self.hovered_option == Some(i);

            if is_hovered || is_selected {
                // Option: radius 12, active bg rgba($n2,.10).
                compositor.push_to_layer(
                    layer,
                    SceneNode::RoundedRect {
                        x: dd.x + PAD_Y,
                        y: oy,
                        w: dd.w - PAD_Y * 2.0,
                        h: OPTION_H,
                        color: if is_selected {
                            glass.surface_active.0
                        } else {
                            glass.surface_hover.0
                        },
                        corner_radius: theme.radius.md.min(OPTION_H / 2.0),
                        border_width: 0.0,
                        border_color: [0.0; 4],
                    },
                );
            }
            // base-2m: rgba($n2,.56) -> .76 hovered/selected.
            let label_alpha = if is_hovered || is_selected { 0.8 } else { 0.59 };
            compositor.push_to_layer(
                layer,
                SceneNode::Text {
                    key: TextNodeKey::new(option, FONT, line_height, None).with_weight(500),
                    x: dd.x + PAD_X,
                    y: oy + (OPTION_H - line_height) / 2.0,
                    color: with_alpha(text, text.0[3] * label_alpha),
                },
            );
            if is_selected {
                // 8px dot on the right marks the active option.
                compositor.push_to_layer(
                    layer,
                    SceneNode::RoundedRect {
                        x: dd.x + dd.w - PAD_X - DOT,
                        y: oy + (OPTION_H - DOT) / 2.0,
                        w: DOT,
                        h: DOT,
                        color: text.0,
                        corner_radius: DOT / 2.0,
                        border_width: 0.0,
                        border_color: [0.0; 4],
                    },
                );
            }
        }
    }
}
