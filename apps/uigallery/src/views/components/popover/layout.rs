//! The popover page laid out: the groups in the showcase mold, each
//! trigger in its card and where its panel goes, and what the pointer and
//! the frame do with them.

use engine::drawing::compositor::{Compositor, LayerId, SceneNode};
use engine::styling::theme::Theme;
use engine::widgets::overlay::OverlayManager;
use engine::widgets::overlay::popover::render_panel;
use engine::widgets::prelude::{ButtonSize, EventResult, Rect, WidgetEvent};

use super::super::showcase::{self, CARD_H, GroupLayout};
use super::{Card, PRESETS, PopoverPage, Spot};

impl PopoverPage {
    /// Space inside a card's body around what it holds.
    fn pad(theme: &Theme) -> f32 {
        theme.spacing.md
    }

    /// The band at a card's top that its chip takes.
    fn chip_band(theme: &Theme) -> f32 {
        theme.spacing.xxl + theme.spacing.sm
    }

    /// The panel's size: the kit's small width, the presets stacked.
    fn panel_size(theme: &Theme) -> (f32, f32) {
        let t = &theme.overlay.popover;
        let row = ButtonSize::Sm.height(theme);
        (t.widths[0], row * PRESETS.len() as f32 + t.pad * 2.0)
    }

    /// A forced card's body: the card less its chip band and its padding,
    /// the container its popover receives.
    pub(super) fn body(card: Rect, theme: &Theme) -> Rect {
        let pad = Self::pad(theme);
        let top = Self::chip_band(theme);
        Rect::new(
            card.x + pad,
            card.y + top,
            (card.w - pad * 2.0).max(0.0),
            (card.h - top - pad).max(0.0),
        )
    }

    /// How tall a group's cards are: the showcase's for the live card; a
    /// forced card holds its chip band, its trigger, the gap and the open
    /// panel, whatever is taller.
    fn card_h(&self, g: usize, theme: &Theme) -> f32 {
        let group = &self.groups[g];
        if !group.pinned {
            return CARD_H;
        }
        let trigger_h = group
            .cards
            .iter()
            .map(|c| c.trigger.preferred_size(theme).1)
            .fold(0.0, f32::max);
        let gap = theme.overlay.popover.clearance.gap;
        let body = trigger_h + gap + Self::panel_size(theme).1;
        CARD_H.max(Self::chip_band(theme) + body + Self::pad(theme))
    }

    /// The groups in the showcase mold, the rows of a forced group
    /// restacked at its cards' height (the showcase lays every card out
    /// `CARD_H` tall).
    pub(super) fn layout(&self, content: Rect, theme: &Theme) -> Vec<GroupLayout> {
        let spans = self.groups.iter().map(|g| (g.columns, g.cards.len()));
        let mut layout = showcase::layout(spans, content, theme);
        let gap = theme.spacing.sm;
        let mut down = 0.0;
        for (g, at) in layout.iter_mut().enumerate() {
            let h = self.card_h(g, theme);
            at.title_y += down;
            let first = at.cards.first().map_or(0.0, |r| r.y);
            let mut rows = 0.0_f32;
            for card in &mut at.cards {
                let row = ((card.y - first) / (CARD_H + gap)).round();
                card.y = first + down + row * (h + gap);
                card.h = h;
                rows = rows.max(row + 1.0);
            }
            down += rows * (h - CARD_H);
            at.rule_y += down;
        }
        layout
    }

    /// The trigger's rect in its card: centered in the live card, at its
    /// spot in a forced card's body.
    fn trigger_rect(card: &Card, rect: Rect, theme: &Theme) -> Rect {
        let (w, h) = card.trigger.preferred_size(theme);
        let body = Self::body(rect, theme);
        let (x, y) = match card.spot {
            Spot::Center => return showcase::centered((w, h), rect),
            Spot::TopStart => (body.x, body.y),
            Spot::TopEnd => (body.right() - w, body.y),
            Spot::BottomCenter => (body.x + (body.w - w) / 2.0, body.bottom() - h),
        };
        // On whole pixels, never past the body's edges.
        let x = x
            .round()
            .clamp(body.x.ceil(), (body.right() - w).floor().max(body.x.ceil()));
        let y = y.round().clamp(
            body.y.ceil(),
            (body.bottom() - h).floor().max(body.y.ceil()),
        );
        Rect::new(x, y, w, h)
    }

    /// Every card's trigger and, open or not, where its panel goes: inside
    /// its card's body for a forced group, inside the visible page for the
    /// live one.
    pub(super) fn places(
        &self,
        content: Rect,
        viewport: Rect,
        theme: &Theme,
    ) -> Vec<(usize, usize, Rect, Rect)> {
        let size = Self::panel_size(theme);
        let layout = self.layout(content, theme);
        let mut out = Vec::new();
        for (g, (group, at)) in self.groups.iter().zip(&layout).enumerate() {
            for (c, (card, rect)) in group.cards.iter().zip(&at.cards).enumerate() {
                let trigger = Self::trigger_rect(card, *rect, theme);
                let bounds = if group.pinned {
                    Self::body(*rect, theme)
                } else {
                    viewport
                };
                let panel = card.popover.placement(trigger, size, bounds, theme).rect;
                out.push((g, c, trigger, panel));
            }
        }
        out
    }

    /// The presets' rects in a panel at `panel`.
    fn preset_rects(panel: Rect, theme: &Theme) -> impl Iterator<Item = Rect> {
        let inner = panel.inset(theme.overlay.popover.pad);
        let row = ButtonSize::Sm.height(theme);
        (0..PRESETS.len()).map(move |i| Rect::new(inner.x, inner.y + i as f32 * row, inner.w, row))
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        showcase::content_height(&self.layout(content, theme), content)
    }

    /// The stack brought in line with the popovers (the page's wake).
    /// `true` when that changed something.
    pub fn sync(
        &mut self,
        mgr: &mut OverlayManager,
        content: Rect,
        viewport: Rect,
        theme: &Theme,
    ) -> bool {
        let places = self.places(content, viewport, theme);
        let mut changed = false;
        for (g, c, _, panel) in places {
            changed |= self.groups[g].cards[c].popover.sync(mgr, panel);
        }
        changed
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        viewport: Rect,
        theme: &Theme,
    ) -> EventResult {
        let mut result = EventResult::IGNORED;
        if matches!(event, WidgetEvent::MouseDown { .. }) && self.blur_focus() {
            result = EventResult::changed();
        }
        let places = self.places(content, viewport, theme);
        // The open panel floats over the page: what lands in it is the
        // panel's, never a trigger's beneath.
        let (px, py) = event.pos();
        let over_panel = places.iter().any(|&(g, c, _, panel)| {
            self.groups[g].cards[c].popover.is_open() && panel.contains(px, py)
        });
        let mut pressed = None;
        let mut picked = None;
        for &(g, c, trigger, panel) in &places {
            // A forced card shows a state; it does not operate.
            if self.groups[g].pinned {
                continue;
            }
            let card = &mut self.groups[g].cards[c];
            if card.popover.is_open() {
                for (i, (preset, rect)) in card
                    .presets
                    .iter_mut()
                    .zip(Self::preset_rects(panel, theme))
                    .enumerate()
                {
                    if rect.bottom() > panel.bottom() {
                        continue;
                    }
                    let r = preset.handle_event(event, rect);
                    if r.clicked {
                        picked = Some((g, c, i));
                    }
                    result = result.merge(r);
                }
                if card.popover.dismiss(event, trigger, panel) {
                    result = result.merge(EventResult::changed());
                }
            } else if over_panel {
                continue;
            }
            let r = card.trigger.handle_event(event, trigger);
            if r.clicked {
                pressed = Some((g, c));
            }
            result = result.merge(r);
        }
        if let Some((g, c, i)) = picked {
            self.pick(g, c, i);
        } else if let Some((g, c)) = pressed {
            self.press(g, c);
        }
        result
    }

    /// The groups' titles, cards and rules, the triggers, the forced
    /// panels in their cards; the live panel, when open, on `overlay`.
    pub fn render(
        &mut self,
        c: &mut Compositor,
        overlay: LayerId,
        content: Rect,
        viewport: Rect,
        theme: &Theme,
    ) {
        let layout = self.layout(content, theme);
        let last = layout.len().saturating_sub(1);
        for (g, (group, at)) in self.groups.iter().zip(&layout).enumerate() {
            showcase::title(c, group.title, content, at.title_y, theme);
            for (card, rect) in group.cards.iter().zip(&at.cards) {
                showcase::card(c, *rect, card.title, theme);
            }
            if g != last {
                showcase::rule(c, content, at.rule_y, theme);
            }
        }
        let places = self.places(content, viewport, theme);
        for (g, ci, trigger, panel) in places {
            let pinned = self.groups[g].pinned;
            let card = &mut self.groups[g].cards[ci];
            let open = pinned || card.popover.is_open();
            card.trigger.render(c, trigger, open, theme);
            if !open {
                continue;
            }
            let layer = if pinned { LayerId::DEFAULT } else { overlay };
            render_panel(c, layer, panel, theme);
            c.push_to_layer(
                layer,
                SceneNode::PushClip {
                    x: panel.x,
                    y: panel.y,
                    w: panel.w,
                    h: panel.h,
                },
            );
            for (preset, rect) in card.presets.iter().zip(Self::preset_rects(panel, theme)) {
                preset.render_to_layer(c, layer, rect, theme);
            }
            c.push_to_layer(layer, SceneNode::PopClip);
        }
    }
}
