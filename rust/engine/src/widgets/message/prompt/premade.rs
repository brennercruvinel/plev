//! The premade prompts over the prompt input (`PromptInput/
//! PremadePrompt`): the suggestions, one per row, wrapped to the panel's
//! width; under them, on the well, the dark `Surprise me` and the
//! categories, flowing onto another line where the panel is too narrow
//! for them side by side (the kit scrolls them sideways).
//!
//! A suggestion picked (a click, a reader's or an agent's `activate`)
//! goes into the prompt; `Surprise me` puts the next one there; a
//! category presses and nothing else happens, as in the kit.

use crate::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::drawing::text::TextMeasurer;
use crate::styling::theme::Theme;
use crate::widgets::buttons::{Button, ButtonSize, ButtonVariant};
use crate::widgets::recipe::{focus_ring_inside, rounded_rect, rounded_rect_stroke, shadow_drops};
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

/// What a press on the panel asked for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Hit {
    Prompt(usize),
    Surprise,
    Category(usize),
}

/// Where the panel's parts land.
#[derive(Clone, Debug, PartialEq)]
pub struct PremadeLayout {
    pub panel: Rect,
    pub rows: Vec<Rect>,
    pub footer: Rect,
    pub surprise: Rect,
    pub categories: Vec<Rect>,
}

#[derive(Clone, Debug)]
pub(super) struct Premade {
    pub prompts: Vec<String>,
    pub hovered: Option<usize>,
    pub pressed: Option<usize>,
    pub focused: Option<usize>,
    pub surprise: Button,
    pub categories: Vec<Button>,
    /// The suggestion `Surprise me` puts next.
    pub next: usize,
}

impl Premade {
    /// The kit's (`PremadePrompt/index.tsx:3-8, 34`).
    pub fn kit() -> Self {
        let prompts = [
            "Create a 3D model of a sleek, futuristic car with neon accents.",
            "Design a 3D medieval sword with intricate engravings on the blade.",
            "Generate a 3D low-poly tree with a vibrant, autumn color palette.",
            "Build a 3D cozy wooden cabin with a detailed, textured roof.",
        ];
        let categories = ["Characters", "Objects", "Backgrounds", "Characters", "Cute"];
        Self {
            prompts: prompts.map(str::to_owned).to_vec(),
            hovered: None,
            pressed: None,
            focused: None,
            // :23 `Button isPrimary isSmall`, its arrows
            surprise: Button::new("Surprise me")
                .variant(ButtonVariant::Primary)
                .size(ButtonSize::Sm)
                .icon("repeat"),
            // :37 `px-5 border border-[#E2E2E2] rounded-xl text-[0.75rem]`:
            // the outline button at its small step
            categories: categories
                .map(|c| {
                    Button::new(c)
                        .variant(ButtonVariant::Outline)
                        .size(ButtonSize::Sm)
                })
                .to_vec(),
            next: 0,
        }
    }

    /// The panel over a box `frame`: as wide, its rows wrapped, its footer
    /// flowed, its bottom the gap above the box.
    pub fn layout(&self, frame: Rect, theme: &Theme) -> PremadeLayout {
        let t = &theme.message.prompt.premade;
        let style = t.row.style(&theme.typography);
        let inner_x = frame.x + t.border_width + t.list_pad;
        let inner_w = (frame.w - (t.border_width + t.list_pad) * 2.0).max(0.0);
        let text_w = (inner_w - t.row_pad * 2.0).max(0.0);
        let mut y = t.border_width + t.list_pad;
        let mut rows = Vec::with_capacity(self.prompts.len());
        for (i, p) in self.prompts.iter().enumerate() {
            if i > 0 {
                y += t.row_gap;
            }
            let (_, h) = TextMeasurer::measure_styled(p, &style, Some(text_w));
            let r = Rect::new(inner_x, y, inner_w, h + t.row_pad * 2.0);
            y = r.bottom();
            rows.push(r);
        }
        y += t.list_pad;
        let footer_top = y;
        let left = frame.x + t.border_width + t.footer_pad;
        let right = frame.right() - t.border_width - t.footer_pad;
        let sizes: Vec<(f32, f32)> = std::iter::once(&self.surprise)
            .chain(&self.categories)
            .map(|b| b.preferred_size(theme))
            .collect();
        let (mut at, mut line_y, mut line_h) = (left, footer_top + t.footer_pad, 0.0_f32);
        let mut placed = Vec::with_capacity(sizes.len());
        for (w, h) in sizes {
            if at > left && at + w > right {
                at = left;
                line_y += line_h + t.footer_gap;
                line_h = 0.0;
            }
            placed.push(Rect::new(at, line_y, w.min(right - left), h));
            at += w + t.footer_gap;
            line_h = line_h.max(h);
        }
        let footer_h = line_y + line_h + t.footer_pad + t.border_width - footer_top;
        let h = footer_top + footer_h;
        let top = frame.y - t.gap - h;
        let shift = |r: Rect| Rect::new(r.x, r.y + top, r.w, r.h);
        let mut buttons = placed.into_iter().map(shift);
        PremadeLayout {
            panel: Rect::new(frame.x, top, frame.w, h),
            rows: rows.into_iter().map(shift).collect(),
            footer: Rect::new(frame.x, top + footer_top, frame.w, footer_h),
            surprise: buttons.next().unwrap_or_default(),
            categories: buttons.collect(),
        }
    }

    /// The room the panel takes over a box `frame`: its height and the
    /// gap.
    pub fn room(&self, frame: Rect, theme: &Theme) -> f32 {
        frame.y - self.layout(frame, theme).panel.y
    }

    fn row_at(l: &PremadeLayout, x: f32, y: f32) -> Option<usize> {
        l.rows.iter().position(|r| r.contains(x, y))
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        l: &PremadeLayout,
    ) -> (EventResult, Option<Hit>) {
        let mut result = EventResult::IGNORED;
        let mut hit = None;
        match *event {
            WidgetEvent::MouseMove { x, y } => {
                let at = Self::row_at(l, x, y);
                if at != self.hovered {
                    self.hovered = at;
                    result = EventResult::changed();
                }
            }
            WidgetEvent::MouseDown { x, y } => {
                self.pressed = Self::row_at(l, x, y);
                if self.pressed.is_some() {
                    result = EventResult::changed();
                }
            }
            WidgetEvent::MouseUp { x, y } => {
                if let Some(down) = self.pressed.take() {
                    result = EventResult::changed();
                    if Self::row_at(l, x, y) == Some(down) {
                        hit = Some(Hit::Prompt(down));
                        result = EventResult::clicked();
                    }
                }
            }
            WidgetEvent::Scroll { .. } => {}
        }
        let r = self.surprise.handle_event(event, l.surprise);
        if r.clicked {
            hit = Some(Hit::Surprise);
        }
        result = result.merge(r);
        for (i, (b, at)) in self.categories.iter_mut().zip(&l.categories).enumerate() {
            let r = b.handle_event(event, *at);
            if r.clicked {
                hit = Some(Hit::Category(i));
            }
            result = result.merge(r);
        }
        (result, hit)
    }

    /// The suggestion `Surprise me` gives, and the next one after it.
    pub fn surprise_me(&mut self) -> Option<String> {
        let n = self.prompts.len();
        let p = self.prompts.get(self.next % n.max(1))?.clone();
        self.next = (self.next + 1) % n;
        Some(p)
    }

    pub fn render(
        &self,
        c: &mut Compositor,
        layer: LayerId,
        l: &PremadeLayout,
        ids: &[Option<accord::SemId>],
        theme: &Theme,
    ) {
        let t = &theme.message.prompt.premade;
        let p = l.panel;
        shadow_drops(c, layer, p, t.radius, &t.shadow);
        c.push_to_layer(layer, rounded_rect(p.x, p.y, p.w, p.h, t.radius, t.fill.0));
        // the well under the footer, its top a hairline (`border-t`), its
        // bottom corners the panel's: clipped to the panel's lower part
        let f = l.footer;
        c.push_to_layer(
            layer,
            SceneNode::PushClip {
                x: f.x,
                y: f.y,
                w: f.w,
                h: f.h,
            },
        );
        c.push_to_layer(
            layer,
            rounded_rect(p.x, p.y, p.w, p.h, t.radius, t.footer_fill.0),
        );
        c.push_to_layer(layer, SceneNode::PopClip);
        c.push_to_layer(
            layer,
            SceneNode::Rect {
                x: f.x,
                y: f.y,
                w: f.w,
                h: t.border_width,
                color: t.border.0,
            },
        );
        c.push_to_layer(
            layer,
            rounded_rect_stroke(p.x, p.y, p.w, p.h, t.radius, t.border.0, t.border_width),
        );
        let style = t.row.style(&theme.typography);
        for (i, (text, r)) in self.prompts.iter().zip(&l.rows).enumerate() {
            if let Some(Some(id)) = ids.get(i) {
                c.bind_to_layer(layer, id, [r.x, r.y, r.w, r.h]);
            }
            if self.hovered == Some(i) || self.focused == Some(i) {
                c.push_to_layer(
                    layer,
                    rounded_rect(r.x, r.y, r.w, r.h, t.row_radius, t.row_hover.0),
                );
                c.push_to_layer(
                    layer,
                    rounded_rect_stroke(
                        r.x,
                        r.y,
                        r.w,
                        r.h,
                        t.row_radius,
                        t.row_hover_border.0,
                        t.border_width,
                    ),
                );
            }
            if self.focused == Some(i) {
                c.push_to_layer(layer, focus_ring_inside(*r, t.row_radius, theme));
            }
            let w = (r.w - t.row_pad * 2.0).max(0.0);
            c.push_to_layer(
                layer,
                SceneNode::Text {
                    key: TextNodeKey::from_style(text, &style, Some(w)),
                    x: r.x + t.row_pad,
                    y: r.y + t.row_pad,
                    color: t.row.color.0,
                },
            );
        }
        self.surprise.render_to_layer(c, layer, l.surprise, theme);
        for (b, r) in self.categories.iter().zip(&l.categories) {
            b.render_to_layer(c, layer, *r, theme);
        }
    }
}
