use crate::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::text::{TextMeasurer, TextStyle};
use crate::theme::{Intent, Theme};
use crate::ui::icons;

use super::{EventResult, Rect, WidgetEvent, intent_fill, with_alpha};

const FONT: f32 = 13.0;
const ITEM_H: f32 = 28.0;
const SEP_H: f32 = 9.0;
const PAD_X: f32 = 10.0;
const PAD_Y: f32 = 5.0;
const ICON: f32 = 14.0;
const MIN_W: f32 = 160.0;

/// One row of a [`ContextMenu`].
#[derive(Clone, Debug)]
pub enum MenuEntry {
    Item {
        /// Opaque id reported on click.
        id: u64,
        label: String,
        /// Optional leading icon ([`crate::ui::icons`] name).
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

    /// Menu size from real text measurement.
    pub fn size(&self) -> (f32, f32) {
        let style = TextStyle::new(FONT);
        let mut w: f32 = MIN_W;
        let mut h = PAD_Y * 2.0;
        for entry in &self.entries {
            match entry {
                MenuEntry::Item { label, icon, .. } => {
                    let (tw, _) = TextMeasurer::measure_styled(label, &style, None);
                    let icon_w = if icon.is_some() { ICON + 8.0 } else { 0.0 };
                    w = w.max(tw + icon_w + PAD_X * 2.0 + 12.0);
                    h += ITEM_H;
                }
                MenuEntry::Separator => h += SEP_H,
            }
        }
        (w.ceil(), h)
    }

    /// Row rects (separators included, in entry order) for a menu whose
    /// top-left is at (x, y).
    fn entry_rects(&self, x: f32, y: f32) -> Vec<Rect> {
        let (w, _) = self.size();
        let mut rects = Vec::with_capacity(self.entries.len());
        let mut cy = y + PAD_Y;
        for entry in &self.entries {
            let h = match entry {
                MenuEntry::Item { .. } => ITEM_H,
                MenuEntry::Separator => SEP_H,
            };
            rects.push(Rect::new(x, cy, w, h));
            cy += h;
        }
        rects
    }

    fn item_at(&self, px: f32, py: f32, x: f32, y: f32) -> Option<usize> {
        self.entry_rects(x, y)
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

    /// Handle an event for a menu anchored at `(x, y)`.
    /// On item activation, `EventResult::clicked` is set and
    /// [`last_clicked`](ContextMenu::last_clicked) holds the item id.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        x: f32,
        y: f32,
    ) -> (EventResult, Option<u64>) {
        match *event {
            WidgetEvent::MouseMove { x: px, y: py } => {
                let hit = self.item_at(px, py, x, y).filter(|&i| {
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
                if let Some(i) = self.item_at(px, py, x, y) {
                    if let MenuEntry::Item { id, disabled, .. } = &self.entries[i] {
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
        let (w, h) = self.size();

        // Path-based surface: row icons are paths and must stack on top.
        let radius = theme.radius.md + 2.0;
        compositor.push_to_layer(
            layer,
            super::path_rounded_rect(x, y, w, h, radius, with_alpha(theme.colors.bg_panel, 0.99)),
        );
        compositor.push_to_layer(
            layer,
            super::path_rounded_rect_stroke(
                x,
                y,
                w,
                h,
                radius,
                with_alpha(theme.colors.divider, 1.0),
                1.0,
            ),
        );

        let line_height = FONT * 1.3;
        for (i, (entry, rect)) in self.entries.iter().zip(self.entry_rects(x, y)).enumerate() {
            match entry {
                MenuEntry::Separator => {
                    compositor.push_to_layer(
                        layer,
                        SceneNode::Rect {
                            x: rect.x + 6.0,
                            y: rect.y + rect.h / 2.0,
                            w: rect.w - 12.0,
                            h: 1.0,
                            color: with_alpha(theme.colors.divider, 1.0),
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
                    let alpha = if *disabled { 0.45 } else { 1.0 };
                    if self.hovered == Some(i) {
                        compositor.push_to_layer(
                            layer,
                            super::path_rounded_rect(
                                rect.x + 4.0,
                                rect.y + 1.0,
                                rect.w - 8.0,
                                rect.h - 2.0,
                                theme.radius.md,
                                with_alpha(theme.colors.bg_hover, 1.0),
                            ),
                        );
                    }
                    let fg = match intent {
                        Intent::Neutral => with_alpha(theme.colors.text, alpha),
                        other => {
                            let c = intent_fill(theme, *other);
                            [c[0], c[1], c[2], alpha]
                        }
                    };
                    let mut tx = rect.x + PAD_X;
                    if let Some(name) = icon {
                        if let Some(node) =
                            icons::icon_at(name, ICON, fg, tx, rect.y + (rect.h - ICON) / 2.0)
                        {
                            compositor.push_to_layer(layer, node);
                        }
                        tx += ICON + 8.0;
                    }
                    compositor.push_to_layer(
                        layer,
                        SceneNode::Text {
                            key: TextNodeKey::new(label, FONT, line_height, None),
                            x: tx,
                            y: rect.y + (rect.h - line_height) / 2.0,
                            color: fg,
                        },
                    );
                }
            }
        }
    }
}
