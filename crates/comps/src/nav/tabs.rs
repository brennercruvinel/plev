//! HOFF segmented tabs: a graphite strip (`glass.tabs`, the tabs radius)
//! whose equal-width segments share one base-2sm style for measurement
//! and drawing; the active segment is a raised, edge-lit glass block.
//! Keyboard focus ([`Tabs::set_focused`]) rings the whole strip with the
//! accent ring.

use engine::compositor::{Compositor, SceneNode, TextNodeKey};
use engine::text::TextMeasurer;
use engine::theme::{ControlSize, Theme};

use crate::core::{EventResult, Rect, WidgetEvent};
use crate::recipe::{focus_ring, glass_pill, shadow_node};

/// Segmented tab strip. Tabs share the width equally (CSS `flex: 1`);
/// give the widget [`Tabs::height`] (one `Md` control) for the canonical
/// `Xs` segments inside the tabs padding.
#[derive(Clone, Debug)]
pub struct Tabs {
    pub labels: Vec<String>,
    pub active: usize,
    hovered: Option<usize>,
    focused: bool,
}

impl Tabs {
    pub fn new(labels: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            labels: labels.into_iter().map(Into::into).collect(),
            active: 0,
            hovered: None,
            focused: false,
        }
    }

    pub fn hovered(&self) -> Option<usize> {
        self.hovered
    }

    /// Keyboard focus, driven by the owning view (plev has no global
    /// focus chain). The ring wraps the whole strip.
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Canonical strip height: the `Md` control (segments are `Xs` inside
    /// the tabs padding).
    pub fn height(theme: &Theme) -> f32 {
        theme.control.height(ControlSize::Md)
    }

    /// Hit rects for each tab within `bounds`: equal-width segments
    /// inside the tabs container padding.
    pub fn item_rects(&self, bounds: Rect, theme: &Theme) -> Vec<Rect> {
        let pad = theme.control.tabs_pad;
        let n = self.labels.len().max(1) as f32;
        let w = (bounds.w - pad * 2.0) / n;
        let h = bounds.h - pad * 2.0;
        (0..self.labels.len())
            .map(|i| Rect::new(bounds.x + pad + i as f32 * w, bounds.y + pad, w, h))
            .collect()
    }

    fn tab_at(&self, x: f32, y: f32, bounds: Rect, theme: &Theme) -> Option<usize> {
        self.item_rects(bounds, theme)
            .iter()
            .position(|r| r.contains(x, y))
    }

    /// Segments are laid out from the theme's tabs padding, so the event
    /// path takes the same `theme` the render path draws with.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        theme: &Theme,
    ) -> EventResult {
        match *event {
            WidgetEvent::MouseMove { x, y } => {
                let hit = self.tab_at(x, y, bounds, theme);
                if hit != self.hovered {
                    self.hovered = hit;
                    EventResult::changed()
                } else {
                    EventResult::IGNORED
                }
            }
            WidgetEvent::MouseDown { x, y } => {
                if let Some(i) = self.tab_at(x, y, bounds, theme) {
                    if i != self.active {
                        self.active = i;
                        EventResult::clicked()
                    } else {
                        EventResult {
                            handled: true,
                            ..EventResult::IGNORED
                        }
                    }
                } else {
                    EventResult::IGNORED
                }
            }
            _ => EventResult::IGNORED,
        }
    }

    pub fn render(&self, compositor: &mut Compositor, bounds: Rect, theme: &Theme) {
        let glass = &theme.glass;

        let radius = theme.shape.tabs.min(bounds.h / 2.0);
        if self.focused {
            compositor.push(focus_ring(bounds, radius, theme));
        }

        compositor.push(SceneNode::RoundedRect {
            x: bounds.x,
            y: bounds.y,
            w: bounds.w,
            h: bounds.h,
            color: glass.tabs.0,
            corner_radius: radius,
            border_width: 0.0,
            border_color: [0.0; 4],
        });

        // Labels: base-2sm (Tabs.module.sass), one style for measure+render.
        let style = theme.typography.base_2sm();
        let rects = self.item_rects(bounds, theme);

        // Active block: raised shadow + edge-light + surface-hover fill at
        // the block radius.
        if let Some(rect) = rects.get(self.active) {
            let block_radius = theme.shape.block.min(rect.h / 2.0);
            if let Some(shadow) = shadow_node(*rect, block_radius, &theme.shadows.raised) {
                compositor.push(shadow);
            }
            for node in glass_pill(
                *rect,
                block_radius,
                glass.edge.0,
                theme.control.edge_width_strong,
                glass.surface_hover.0,
            ) {
                compositor.push(node);
            }
        }
        for (i, (label, rect)) in self.labels.iter().zip(&rects).enumerate() {
            let is_active = i == self.active;
            let is_hovered = self.hovered == Some(i);

            // base-2sm $text-secondary -> $text-primary on hover/active.
            let color = if is_active || is_hovered {
                theme.colors.text
            } else {
                theme.colors.text_mid
            };
            let (text_w, _) = TextMeasurer::measure_styled(label, &style, None);
            compositor.push(SceneNode::Text {
                key: TextNodeKey::from_style(label, &style, None),
                x: rect.x + (rect.w - text_w) / 2.0,
                y: rect.y + TextMeasurer::vertical_center(&style, rect.h),
                color: color.0,
            });
        }
    }
}
