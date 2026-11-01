//! The prompt input's draw: the premade panel over the box, the box with
//! its drops, border and key light, the line, the row of controls, the
//! voice mark; then, on `overlay`, what is open (the add-files menu, the
//! model listbox, the setting's options), so they draw over what is near.

use super::{FILES, PromptInput, PromptLayout};
use crate::drawing::compositor::{Compositor, LayerId};
use crate::styling::theme::Theme;
use crate::widgets::icons;
use crate::widgets::message::compose::LineLook;
use crate::widgets::message::select_ai::{line, paint_panel};
use crate::widgets::recipe::{
    focus_ring, focus_ring_inside, gradient_fill, rounded_rect, rounded_rect_stroke, shadow_drops,
    shadow_insets,
};
use crate::widgets::widget::{Rect, with_alpha};

impl PromptInput {
    pub fn render(&self, c: &mut Compositor, frame: Rect, overlay: LayerId, theme: &Theme) {
        let layer = LayerId::DEFAULT;
        let l = self.layout(frame, theme);
        let t = &theme.message.prompt;
        if let Some(id) = &self.sem {
            c.bind_to_layer(layer, id, [frame.x, frame.y, frame.w, frame.h]);
        }
        if let (Some(p), Some(pl)) = (&self.premade, &l.premade) {
            let ids: Vec<_> = (0..p.prompts.len()).map(|i| self.premade_id(i)).collect();
            p.render(c, layer, pl, &ids, theme);
        }
        if let (Some(v), Some(at)) = (&self.view, l.view) {
            v.render_to_layer(c, layer, at, theme);
            if let Some(forward) = self.view_focus {
                // the arrow where front G's controller places it
                let (back, ahead) = v.arrow_rects(at, theme);
                let r = if forward { ahead } else { back };
                c.push_to_layer(layer, focus_ring(r, r.w / 2.0, theme));
            }
        }
        shadow_drops(c, layer, frame, t.radius, &t.shadow);
        let border = if self.hovered || self.held_hover {
            t.border_hover
        } else {
            t.border
        };
        let f = frame;
        c.push_to_layer(layer, rounded_rect(f.x, f.y, f.w, f.h, t.radius, t.fill.0));
        c.push_to_layer(
            layer,
            rounded_rect_stroke(f.x, f.y, f.w, f.h, t.radius, border.0, t.border_width),
        );
        shadow_insets(c, layer, frame, t.radius, &t.shadow);
        if let Some(text) = l.text {
            let look = LineLook {
                alpha: 1.0,
                ..self.line_look(theme)
            };
            self.line.render(c, layer, text, &look, theme);
        }
        self.add.render_to_layer(c, layer, l.add, theme);
        self.setting.render(c, l.setting, theme);
        self.render_trigger(c, layer, &l, theme);
        self.voice_button.render_to_layer(c, layer, l.voice, theme);
        self.submit.render_to_layer(c, layer, l.submit, theme);
        if let Some(r) = l.mark {
            let v = &t.voice;
            c.push_to_layer(
                layer,
                rounded_rect_stroke(r.x, r.y, r.w, r.h, r.w / 2.0, v.ring_color.0, v.ring_width),
            );
            let (cx, cy) = r.center();
            let dot = Rect::new(cx - v.dot / 2.0, cy - v.dot / 2.0, v.dot, v.dot);
            c.push_to_layer(
                layer,
                gradient_fill(dot, v.dot / 2.0, v.dot_fill[0].0, v.dot_fill[1].0),
            );
        }
        // what is open, over the page
        if let Some((panel, rows)) = &l.files {
            self.render_files(c, overlay, *panel, rows, theme);
        }
        if let Some(panel) = l.models {
            self.model.render(c, overlay, panel, theme);
        }
        self.setting.render_dropdown(c, overlay, l.setting, theme);
    }

    /// The model menu's trigger: the chosen name and a chevron (up while
    /// open), on the hover wash while hovered or open.
    fn render_trigger(&self, c: &mut Compositor, layer: LayerId, l: &PromptLayout, theme: &Theme) {
        let t = &theme.message.prompt.trigger;
        let r = l.model;
        let alpha = if self.voice {
            theme.message.prompt.voice.dim
        } else {
            1.0
        };
        if let Some(id) = self.model.sem_id() {
            c.bind_to_layer(layer, id, [r.x, r.y, r.w, r.h]);
        }
        if !self.voice && (self.model_press.hovered || self.model_open) {
            c.push_to_layer(layer, rounded_rect(r.x, r.y, r.w, r.h, t.radius, t.hover.0));
        }
        if self.model_press.focused {
            c.push_to_layer(layer, focus_ring(r, t.radius, theme));
        }
        let x = r.x + t.pad_x;
        let room = (r.w - t.pad_x * 2.0 - t.gap - t.chevron).max(0.0);
        let y = r.y + (r.h - t.text.line_height(&theme.typography)) / 2.0;
        let run = crate::widgets::message::tokens::MessageText {
            color: crate::styling::color::Color(with_alpha(
                t.text.color,
                t.text.color.0[3] * alpha,
            )),
            ..t.text
        };
        let name = self.model.chosen_title().unwrap_or_default();
        line(c, layer, name, &run, x, y, room, theme);
        let chevron = if self.model_open {
            "chevron-up"
        } else {
            "chevron-down"
        };
        let color = with_alpha(t.chevron_color, t.chevron_color.0[3] * alpha);
        let cx = r.right() - t.pad_x - t.chevron;
        let cy = r.y + (r.h - t.chevron) / 2.0;
        if let Some(node) = icons::icon_at(chevron, t.chevron, color, cx, cy) {
            c.push_to_layer(layer, node);
        }
    }

    /// The add-files menu: the panel, a row per way to add, the one under
    /// the pointer or the keyboard washed, its icon darker.
    fn render_files(
        &self,
        c: &mut Compositor,
        layer: LayerId,
        panel: Rect,
        rows: &[Rect],
        theme: &Theme,
    ) {
        let p = &theme.message.prompt;
        let t = &p.files;
        paint_panel(c, layer, panel, &p.panel, &p.panel.shadow);
        for (i, ((icon, label), r)) in FILES.iter().zip(rows).enumerate() {
            if let Some(id) = self.file_id(i) {
                c.bind_to_layer(layer, &id, [r.x, r.y, r.w, r.h]);
            }
            let lit = self.files_hover == Some(i) || self.files_focus == Some(i);
            if lit {
                c.push_to_layer(layer, rounded_rect(r.x, r.y, r.w, r.h, t.radius, t.hover.0));
            }
            if self.files_focus == Some(i) {
                c.push_to_layer(layer, focus_ring_inside(*r, t.radius, theme));
            }
            let bx = r.x + t.pad_x;
            let by = r.y + (r.h - t.icon_box) / 2.0;
            let inset = (t.icon_box - t.icon) / 2.0;
            let color = if lit { t.icon_hover } else { t.icon_color };
            if let Some(node) = icons::icon_at(icon, t.icon, color.0, bx + inset, by + inset) {
                c.push_to_layer(layer, node);
            }
            let x = bx + t.icon_box + t.gap;
            let y = r.y + (r.h - t.text.line_height(&theme.typography)) / 2.0;
            line(
                c,
                layer,
                label,
                &t.text,
                x,
                y,
                (r.right() - t.pad_x - x).max(0.0),
                theme,
            );
        }
    }
}
