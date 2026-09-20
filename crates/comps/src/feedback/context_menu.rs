//! HOFF actions dropdown: a solid popover body at the pill radius with
//! the floating shadow and the inset key-light; `Md`-tall items at the
//! item radius, base-2sm labels (text-default at rest, text-active on
//! hover), optional leading icon, separators. Width fits the widest
//! label and never drops under `size.menu_w`.

use crate::icons;
use engine::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use engine::text::{TextMeasurer, TextStyle};
use engine::theme::{ControlSize, IconSize, Intent, Theme};

use crate::core::{EventResult, Rect, WidgetEvent, intent_fill, with_alpha};
use crate::recipe::{inset_keylight, menu_shadow, rounded_rect, rounded_rect_stroke};

/// The per-theme geometry the size, the hit test and the render share.
struct Metrics {
    item_h: f32,
    /// A separator row: one hairline with `xs` above and below.
    sep_h: f32,
    pad: f32,
    radius: f32,
    item_radius: f32,
    icon: f32,
    gap: f32,
    min_w: f32,
}

impl Metrics {
    fn of(theme: &Theme) -> Self {
        Self {
            item_h: theme.control.height(ControlSize::Md),
            sep_h: theme.spacing.xs * 2.0 + theme.control.edge_width,
            pad: theme.control.menu_pad,
            radius: theme.shape.pill,
            item_radius: theme.shape.item,
            icon: theme.control.icon(IconSize::Sm),
            gap: theme.control.inline_gap,
            min_w: theme.size.menu_w,
        }
    }
}

/// One row of a [`ContextMenu`].
#[derive(Clone, Debug)]
pub enum MenuEntry {
    Item {
        /// Opaque id reported on click.
        id: u64,
        label: String,
        /// Optional leading icon ([`crate::icons`] name).
        icon: Option<&'static str>,
        disabled: bool,
        /// Colors the label (Destructive = red row, etc.).
        intent: Intent,
    },
    Separator,
}

impl MenuEntry {
    pub fn item(id: u64, label: impl Into<String>) -> Self {
        Self::Item {
            id,
            label: label.into(),
            icon: None,
            disabled: false,
            intent: Intent::Neutral,
        }
    }

    pub fn icon(mut self, name: &'static str) -> Self {
        if let Self::Item { icon, .. } = &mut self {
            *icon = Some(name);
        }
        self
    }

    pub fn disabled(mut self, value: bool) -> Self {
        if let Self::Item { disabled, .. } = &mut self {
            *disabled = value;
        }
        self
    }

    pub fn intent(mut self, value: Intent) -> Self {
        if let Self::Item { intent, .. } = &mut self {
            *intent = value;
        }
        self
    }
}

/// Context menu rendered at an arbitrary screen position (overlay layer).
/// Width fits the widest label; rows hover; disabled rows are inert.
#[derive(Clone, Debug)]
pub struct ContextMenu {
    pub entries: Vec<MenuEntry>,
    hovered: Option<usize>,
}

impl ContextMenu {
    pub fn new(entries: Vec<MenuEntry>) -> Self {
        Self {
            entries,
            hovered: None,
        }
    }

    pub fn hovered(&self) -> Option<usize> {
        self.hovered
    }

    /// Item label style: base-2sm, the same for measuring and rendering.
    fn label_style(theme: &Theme) -> TextStyle {
        theme.typography.base_2sm()
    }

    /// Menu size from real text measurement: the widest row (icon slot,
    /// label, item padding on both sides, menu padding on both sides),
    /// never narrower than the menu width token.
    pub fn size(&self, theme: &Theme) -> (f32, f32) {
        let m = Metrics::of(theme);
        let style = Self::label_style(theme);
        let mut w: f32 = m.min_w;
        let mut h = m.pad * 2.0;
        for entry in &self.entries {
            match entry {
                MenuEntry::Item { label, icon, .. } => {
                    let (tw, _) = TextMeasurer::measure_styled(label, &style, None);
                    let icon_w = if icon.is_some() { m.icon + m.gap } else { 0.0 };
                    w = w.max(tw + icon_w + m.pad * 4.0 + m.gap);
                    h += m.item_h;
                }
                MenuEntry::Separator => h += m.sep_h,
            }
        }
        (w.ceil(), h)
    }

    /// Row rects (separators included, in entry order) for a menu whose
    /// top-left is at (x, y).
    pub fn entry_rects(&self, x: f32, y: f32, theme: &Theme) -> Vec<Rect> {
        let m = Metrics::of(theme);
        let (w, _) = self.size(theme);
        let mut rects = Vec::with_capacity(self.entries.len());
        let mut cy = y + m.pad;
        for entry in &self.entries {
            let h = match entry {
                MenuEntry::Item { .. } => m.item_h,
                MenuEntry::Separator => m.sep_h,
            };
            rects.push(Rect::new(x, cy, w, h));
            cy += h;
        }
        rects
    }

    fn item_at(&self, px: f32, py: f32, x: f32, y: f32, theme: &Theme) -> Option<usize> {
        self.entry_rects(x, y, theme)
            .iter()
            .enumerate()
            .find(|(i, r)| matches!(self.entries[*i], MenuEntry::Item { .. }) && r.contains(px, py))
            .map(|(i, _)| i)
    }

    /// Returns the clicked item id, if any. `(x, y)` is the menu origin.
    pub fn clicked_id(&self, index: usize) -> Option<u64> {
        match self.entries.get(index) {
            Some(MenuEntry::Item { id, .. }) => Some(*id),
            _ => None,
        }
    }

    /// Handle an event for a menu anchored at `(x, y)`. On item
    /// activation, `EventResult::clicked` is set and the item id is
    /// returned. Rows are laid out from the theme, so the event path
    /// takes the same `theme` the render path draws with.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        x: f32,
        y: f32,
        theme: &Theme,
    ) -> (EventResult, Option<u64>) {
        match *event {
            WidgetEvent::MouseMove { x: px, y: py } => {
                let hit = self.item_at(px, py, x, y, theme).filter(|&i| {
                    !matches!(self.entries[i], MenuEntry::Item { disabled: true, .. })
                });
                if hit != self.hovered {
                    self.hovered = hit;
                    (EventResult::changed(), None)
                } else {
                    (EventResult::IGNORED, None)
                }
            }
            WidgetEvent::MouseDown { x: px, y: py } => {
                if let Some(i) = self.item_at(px, py, x, y, theme)
                    && let MenuEntry::Item { id, disabled, .. } = &self.entries[i]
                {
                    if *disabled {
                        // Swallow the click but do nothing.
                        return (
                            EventResult {
                                handled: true,
                                ..EventResult::IGNORED
                            },
                            None,
                        );
                    }
                    return (EventResult::clicked(), Some(*id));
                }
                (EventResult::IGNORED, None)
            }
            _ => (EventResult::IGNORED, None),
        }
    }

    pub fn render(
        &self,
        compositor: &mut Compositor,
        layer: LayerId,
        theme: &Theme,
        x: f32,
        y: f32,
    ) {
        let m = Metrics::of(theme);
        let (w, h) = self.size(theme);
        let glass = &theme.glass;
        let body = Rect::new(x, y, w, h);

        // Floating shadow, then the solid popover surface; row icons
        // pushed later stack on top (push order is preserved across
        // types).
        compositor.push_to_layer(layer, menu_shadow(body, m.radius, theme));
        compositor.push_to_layer(layer, rounded_rect(x, y, w, h, m.radius, glass.popover.0));
        compositor.push_to_layer(
            layer,
            rounded_rect_stroke(
                x,
                y,
                w,
                h,
                m.radius,
                glass.edge_soft.0,
                theme.control.edge_width,
            ),
        );
        inset_keylight(compositor, layer, body, m.radius, theme);

        let style = Self::label_style(theme);
        for (i, (entry, rect)) in self
            .entries
            .iter()
            .zip(self.entry_rects(x, y, theme))
            .enumerate()
        {
            match entry {
                MenuEntry::Separator => {
                    compositor.push_to_layer(
                        layer,
                        SceneNode::Rect {
                            x: rect.x + m.pad,
                            y: rect.y + (rect.h - theme.control.edge_width) / 2.0,
                            w: rect.w - m.pad * 2.0,
                            h: theme.control.edge_width,
                            color: glass.surface_active.0,
                        },
                    );
                }
                MenuEntry::Item {
                    label,
                    icon,
                    disabled,
                    intent,
                    ..
                } => {
                    let alpha = if *disabled { glass.disabled_alpha } else { 1.0 };
                    let hovered = self.hovered == Some(i);
                    if hovered {
                        compositor.push_to_layer(
                            layer,
                            rounded_rect(
                                rect.x + m.pad,
                                rect.y,
                                rect.w - m.pad * 2.0,
                                rect.h,
                                m.item_radius,
                                glass.surface_active.0,
                            ),
                        );
                    }
                    // base-2sm: text-default at rest, text-active on hover.
                    let fg = match intent {
                        Intent::Neutral => {
                            let c = if hovered {
                                glass.text_active
                            } else {
                                glass.text_default
                            };
                            with_alpha(c, c.0[3] * alpha)
                        }
                        other => {
                            let c = intent_fill(theme, *other);
                            [c[0], c[1], c[2], c[3] * alpha]
                        }
                    };
                    let mut tx = rect.x + m.pad * 2.0;
                    if let Some(name) = icon {
                        if let Some(node) =
                            icons::icon_at(name, m.icon, fg, tx, rect.y + (rect.h - m.icon) / 2.0)
                        {
                            compositor.push_to_layer(layer, node);
                        }
                        tx += m.icon + m.gap;
                    }
                    compositor.push_to_layer(
                        layer,
                        SceneNode::Text {
                            key: TextNodeKey::from_style(label, &style, None),
                            x: tx,
                            y: rect.y + TextMeasurer::vertical_center(&style, rect.h),
                            color: fg,
                        },
                    );
                }
            }
        }
    }
}
