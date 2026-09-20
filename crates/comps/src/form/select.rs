//! HOFF select: a glass pill control (base-2m value + chevron) that
//! opens a floating options panel on an overlay layer, the active option
//! marked by a dot. Keyboard focus ([`Select::set_focused`]) rings the
//! closed pill with the accent ring.
//!
//! Geometry from the theme: the control and each option are one `Md`
//! control tall, the pill is the tabs/select radius, the panel is the
//! card radius with the menu padding, options are the nav radius, the
//! panel never grows past `size.dropdown_max_h`.

use crate::icons;
use engine::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use engine::text::{TextMeasurer, TextStyle};
use engine::theme::{ControlSize, IconSize, Theme};

use crate::core::{EventResult, Rect, WidgetEvent, with_alpha};
use crate::recipe::{focus_ring, glass_pill, menu_shadow};

/// Label style for the control and its options: base-2m.
fn label_style(theme: &Theme) -> TextStyle {
    theme.typography.base_2m()
}

/// The per-theme geometry the control, the hit test and the panel share.
struct Metrics {
    option_h: f32,
    pad_x: f32,
    pad_y: f32,
    chevron: f32,
    gap: f32,
    max_h: f32,
    dot: f32,
}

impl Metrics {
    fn of(theme: &Theme) -> Self {
        Self {
            option_h: theme.control.height(ControlSize::Md),
            pad_x: theme.spacing.lg,
            pad_y: theme.control.menu_pad,
            chevron: theme.control.icon(IconSize::Sm),
            gap: theme.spacing.xs,
            max_h: theme.size.dropdown_max_h,
            dot: theme.spacing.sm,
        }
    }
}

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
    focused: bool,
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
            focused: false,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Keyboard focus, driven by the owning view (plev has no global focus
    /// chain). Disabled selects refuse focus.
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused && !self.disabled;
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    pub fn close(&mut self) {
        self.open = false;
        self.hovered_option = None;
    }

    pub fn selected_label(&self) -> Option<&str> {
        self.options.get(self.selected).map(String::as_str)
    }

    /// Dropdown rect below the control.
    pub fn dropdown_rect(&self, bounds: Rect, theme: &Theme) -> Rect {
        let m = Metrics::of(theme);
        let h = (m.pad_y * 2.0 + self.options.len() as f32 * m.option_h).min(m.max_h);
        Rect::new(bounds.x, bounds.y + bounds.h + m.gap, bounds.w, h)
    }

    fn option_at(&self, x: f32, y: f32, bounds: Rect, theme: &Theme) -> Option<usize> {
        let m = Metrics::of(theme);
        let dd = self.dropdown_rect(bounds, theme);
        if !dd.contains(x, y) {
            return None;
        }
        let i = ((y - dd.y - m.pad_y) / m.option_h).floor();
        if i < 0.0 {
            return None;
        }
        let i = i as usize;
        (i < self.options.len()).then_some(i)
    }

    /// The options panel is laid out from the theme, so the event path
    /// takes the same `theme` the render path draws with.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        theme: &Theme,
    ) -> EventResult {
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
                    let hit = self.option_at(x, y, bounds, theme);
                    if hit != self.hovered_option {
                        self.hovered_option = hit;
                        result = result.merge(EventResult::changed());
                    }
                }
                result
            }
            WidgetEvent::MouseDown { x, y } => {
                if self.open {
                    if let Some(i) = self.option_at(x, y, bounds, theme) {
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
        let glass = &theme.glass;
        let m = Metrics::of(theme);
        let alpha = if self.disabled {
            glass.disabled_alpha
        } else {
            1.0
        };

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
        let radius = theme.shape.tabs.min(bounds.h / 2.0);
        if self.focused {
            compositor.push(focus_ring(bounds, radius, theme));
        }
        compositor.push(crate::recipe::rounded_rect(
            bounds.x, bounds.y, bounds.w, bounds.h, radius, bg,
        ));
        compositor.push(crate::recipe::rounded_rect_stroke(
            bounds.x,
            bounds.y,
            bounds.w,
            bounds.h,
            radius,
            edge,
            theme.control.edge_width_strong,
        ));

        if let Some(label) = self.selected_label() {
            let style = label_style(theme);
            let text = glass.text_active;
            compositor.push(SceneNode::Text {
                key: TextNodeKey::from_style(label, &style, None),
                x: bounds.x + m.pad_x,
                y: bounds.y + TextMeasurer::vertical_center(&style, bounds.h),
                color: with_alpha(text, text.0[3] * alpha),
            });
        }

        let chevron = if self.open {
            "chevron-up"
        } else {
            "chevron-down"
        };
        if let Some(node) = icons::icon_at(
            chevron,
            m.chevron,
            with_alpha(glass.text_faint, glass.text_faint.0[3] * alpha),
            bounds.x + bounds.w - m.pad_x - m.chevron,
            bounds.y + (bounds.h - m.chevron) / 2.0,
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
        let m = Metrics::of(theme);
        let dd = self.dropdown_rect(bounds, theme);
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
        for (i, option) in self.options.iter().enumerate() {
            let oy = dd.y + m.pad_y + i as f32 * m.option_h;
            if oy + m.option_h > dd.y + dd.h {
                break;
            }
            let is_selected = i == self.selected;
            let is_hovered = self.hovered_option == Some(i);

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
    }
}
