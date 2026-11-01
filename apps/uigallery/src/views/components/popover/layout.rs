//! The popover page laid out: the groups in the showcase mold, each
//! trigger in its card and where its panel goes, and what the pointer and
//! the frame do with them.

use engine::drawing::compositor::{Compositor, LayerId, SceneNode};
use engine::drawing::text::TextMeasurer;
use engine::styling::theme::{Intent, Theme};
use engine::widgets::content::kbd::Kbd;
use engine::widgets::overlay::OverlayManager;
use engine::widgets::overlay::popover::{Line, render_line, render_panel};
use engine::widgets::prelude::{EventResult, Rect, WidgetEvent};

use super::super::showcase::{self, GroupLayout};
use super::{Card, PRESETS, PopoverPage, Spot};

impl PopoverPage {
    /// The panel's size: the kit's small width, or what its widest line
    /// takes where that is wider (a longer language, a larger text
    /// size), the presets stacked.
    fn panel_size(theme: &Theme) -> (f32, f32) {
        let t = &theme.overlay.popover;
        let l = &t.line;
        let style = theme.typography.run(l.label);
        let widest = PRESETS
            .iter()
            .map(|(label, _, keys)| {
                let (text_w, _) = TextMeasurer::measure_styled(label, &style, None);
                let (keys_w, _) = Kbd::new(*keys).preferred_size(theme);
                text_w + keys_w.max(l.key_min_w)
            })
            .fold(0.0, f32::max);
        let line_w = widest + l.slot + l.slot_gap + theme.control.inline_gap + l.pad_x * 2.0;
        (
            t.widths[0].max((line_w + t.pad * 2.0).ceil()),
            l.height * PRESETS.len() as f32 + t.pad * 2.0,
        )
    }

    /// A forced card's body, the container its popover receives.
    pub(super) fn body(card: Rect, theme: &Theme) -> Rect {
        showcase::body(card, theme)
    }

    /// How tall a group's cards are: the showcase's for the live card; a
    /// forced card holds its chip band, its trigger, the gap and the open
    /// panel, whatever is taller.
    fn card_h(&self, g: usize, theme: &Theme) -> f32 {
        let group = &self.groups[g];
        if !group.pinned {
            // the live stage: its panel floats over the page
            return showcase::card_min_h(theme);
        }
        let trigger_h = group
            .cards
            .iter()
            .map(|c| c.trigger.preferred_size(theme).1)
            .fold(0.0, f32::max);
        let gap = theme.overlay.popover.clearance.gap;
        let body = trigger_h + gap + Self::panel_size(theme).1;
        // The body's margins: what `showcase::body` takes off a card.
        let frame = showcase::body_height(0.0, theme);
        frame + body
    }

    /// The groups in the showcase mold, each at its cards' height.
    pub(super) fn layout(&self, content: Rect, theme: &Theme) -> Vec<GroupLayout> {
        let groups = self.groups.iter().enumerate();
        let spans = groups.map(|(g, gr)| (gr.columns, gr.cards.len(), self.card_h(g, theme)));
        showcase::layout_sized(spans.map(|(span, n, h)| (span, vec![h; n])), content, theme)
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
        let row = theme.overlay.popover.line.height;
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
        let motion = theme.intent_motion(Intent::Neutral);
        let mut changed = false;
        for (g, c, _, panel) in places {
            changed |= self.groups[g].cards[c].popover.sync(mgr, panel, &motion);
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
    /// panels in their cards; the live panel, while it shows, on
    /// `anchored`, whose opacity is its fade on `mgr`.
    pub fn render(
        &mut self,
        c: &mut Compositor,
        anchored: LayerId,
        content: Rect,
        viewport: Rect,
        theme: &Theme,
        mgr: &OverlayManager,
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
        let zoom_now = self.zoom;
        let places = self.places(content, viewport, theme);
        for (g, ci, trigger, panel) in places {
            let pinned = self.groups[g].pinned;
            let card = &mut self.groups[g].cards[ci];
            card.trigger
                .render(c, trigger, pinned || card.popover.is_open(), theme);
            let layer = if pinned {
                LayerId::DEFAULT
            } else {
                let Some(shown) = card.popover.shown(mgr) else {
                    continue;
                };
                c.set_layer_opacity(anchored, shown);
                anchored
            };
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
            let rows = card.presets.iter().zip(Self::preset_rects(panel, theme));
            for ((preset, rect), (label, zoom, keys)) in rows.zip(PRESETS) {
                // The preset keeps its press, its focus and its binding;
                // the kit's line is what it draws.
                if let Some(id) = preset.sem_id() {
                    c.bind_to_layer(layer, id, [rect.x, rect.y, rect.w, rect.h]);
                }
                let line = Line {
                    label,
                    keys: Some(keys),
                    checked: zoom.shows(zoom_now),
                    lit: preset.is_hovered(),
                    focused: preset.is_focused(),
                };
                render_line(c, layer, rect, line, theme);
            }
            c.push_to_layer(layer, SceneNode::PopClip);
        }
    }
}
