//! Scene building for the App tab. State and events live in the parent
//! module; everything here only reads it and pushes nodes. Every text run
//! is measured and drawn through one shared TextStyle.

use engine::compositor::{Compositor, SceneNode, TextNodeKey};
use engine::text::TextMeasurer;
use engine::theme::Theme;
use engine::ui::icons;
use engine::ui::widgets::{Rect, glass_pill, rounded_rect, rounded_rect_stroke};
use showcase::model::todo::Filter;

use super::AppSection;
use super::layout::*;
use crate::view::{panel, text, with_alpha};

impl AppSection {
    pub fn render(&mut self, c: &mut Compositor, content: Rect, theme: &Theme) {
        self.sync_rows();
        let l = compute(content, &self.counter_text());
        self.sync_scroll(&l);
        panel(c, l.panel, theme);
        self.render_input(c, &l, theme);
        self.render_rows(c, &l, theme);
        self.render_footer(c, &l, theme);
    }

    /// HOFF glass chrome around the engine TextInput: the component owns
    /// editing, blink, selection and cursor mapping; only its square field
    /// chrome (background and focus-border rects, the nodes spanning the
    /// full field width/height) is replaced by the rounded glass below.
    fn render_input(&mut self, c: &mut Compositor, l: &Layout, theme: &Theme) {
        let g = &theme.glass;
        let r = l.input;
        c.push(rounded_rect(r.x, r.y, r.w, r.h, theme.radius.md, g.field.0));
        let border = if self.input.focused {
            g.field_focus_border
        } else {
            g.edge_soft
        };
        c.push(rounded_rect_stroke(
            r.x,
            r.y,
            r.w,
            r.h,
            theme.radius.md,
            border.0,
            1.0,
        ));
        self.input.text_color = theme.colors.text.0;
        self.input.placeholder_color = g.text_placeholder.0;
        self.input.cursor_color = theme.colors.accent.0;
        self.input.selection_color = with_alpha(theme.colors.accent.0, 0.35);
        for node in self.input.build_scene(r.x, r.y, r.w) {
            let keep = match node {
                SceneNode::Rect { w, h, .. } => w < r.w && h < r.h,
                _ => true,
            };
            if keep {
                c.push(node);
            }
        }
    }

    fn render_rows(&self, c: &mut Compositor, l: &Layout, theme: &Theme) {
        let g = &theme.glass;
        if self.rows.is_empty() {
            let msg = match self.model.filter() {
                Filter::All => "Nothing here. Click the field above, type, press Enter.",
                Filter::Active => "No active todos. Enjoy it.",
                Filter::Completed => "Nothing done yet.",
            };
            text(
                c,
                msg,
                13.0,
                400,
                l.list.x + 2.0,
                l.list.y + 10.0,
                g.text_placeholder.0,
            );
            return;
        }
        c.push(SceneNode::PushClip {
            x: l.list.x,
            y: l.list.y,
            w: l.list.w,
            h: l.list.h,
        });
        let offset = self.scroll.offset();
        let style = item_style();
        for (i, item) in self.model.visible_items().iter().enumerate() {
            let row = row_rect(l.list, i, offset);
            if row.y + ROW_H < l.list.y || row.y > l.list.y + l.list.h {
                continue;
            }
            let hovered = self.hover_row == Some(item.id());
            if hovered {
                c.push(rounded_rect(
                    row.x,
                    row.y + 2.0,
                    row.w,
                    row.h - 4.0,
                    theme.radius.sm,
                    g.surface.0,
                ));
            }
            self.rows[i].1.render(c, checkbox_rect(row), theme);

            // The label fades in with the enter tween and dims as the
            // strike crosses it; the strike width is measured with the
            // same style the label is drawn with, never estimated.
            let enter = item.enter_progress().clamp(0.0, 1.0);
            let strike = item.strike_progress().clamp(0.0, 1.0);
            let (lx, lw) = label_span(row);
            let base = theme.colors.text_mid.0;
            let color = with_alpha(base, base[3] * enter * (1.0 - 0.45 * strike));
            c.push(SceneNode::Text {
                key: TextNodeKey::from_style(item.text(), &style, Some(lw)),
                x: lx,
                y: row.y + TextMeasurer::vertical_center(&style, ROW_H),
                color,
            });
            if strike > 0.0 {
                let tw = TextMeasurer::measure_styled(item.text(), &style, Some(lw)).0;
                c.push(SceneNode::Rect {
                    x: lx,
                    y: row.y + (ROW_H - STRIKE_H) / 2.0,
                    w: tw.min(lw) * strike,
                    h: STRIKE_H,
                    color,
                });
            }

            // Delete: quiet x, lifted by row hover, destructive when aimed.
            let del = delete_rect(row);
            let hot = self.hover_delete == Some(item.id());
            if hot {
                c.push(rounded_rect(
                    del.x,
                    del.y,
                    del.w,
                    del.h,
                    del.w / 2.0,
                    g.surface_hover.0,
                ));
            }
            let tint = if hot {
                theme.colors.danger
            } else if hovered {
                g.text_faint
            } else {
                g.text_placeholder
            };
            let (ix, iy) = (del.x + (del.w - 14.0) / 2.0, del.y + (del.h - 14.0) / 2.0);
            if let Some(node) = icons::icon_at("x", 14.0, tint.0, ix, iy) {
                c.push(node);
            }
        }
        c.push(SceneNode::PopClip);
        if self.scroll.is_scrollable() {
            let th = (l.list.h * self.scroll.thumb_ratio()).max(24.0);
            let ty = l.list.y + (l.list.h - th) * self.scroll.thumb_position();
            c.push(rounded_rect(
                l.list.x + l.list.w - 4.0,
                ty,
                4.0,
                th,
                2.0,
                g.text_placeholder.0,
            ));
        }
    }

    fn render_footer(&self, c: &mut Compositor, l: &Layout, theme: &Theme) {
        let g = &theme.glass;
        c.push(SceneNode::Rect {
            x: l.panel.x + PAD,
            y: l.divider_y,
            w: l.panel.w - PAD * 2.0,
            h: 1.0,
            color: g.edge_soft.0,
        });
        c.push(SceneNode::Text {
            key: TextNodeKey::from_style(&self.counter_text(), &counter_style(), None),
            x: l.counter.0,
            y: l.counter.1,
            color: g.text_faint.0,
        });
        let pstyle = pill_style();
        for (i, (f, pr)) in Filter::ALL.iter().zip(l.pills).enumerate() {
            let active = self.model.filter() == *f;
            let hovered = self.hover_pill == Some(i);
            let fill = if active {
                g.surface_active
            } else if hovered {
                g.button_hover
            } else {
                g.button
            };
            let edge = if active { g.edge } else { g.edge_soft };
            for node in glass_pill(pr, PILL_H / 2.0, edge.0, 1.0, fill.0) {
                c.push(node);
            }
            let tw = TextMeasurer::measure_styled(f.label(), &pstyle, None).0;
            let tint = if active {
                theme.colors.text
            } else if hovered {
                theme.colors.text_mid
            } else {
                g.text_faint
            };
            c.push(SceneNode::Text {
                key: TextNodeKey::from_style(f.label(), &pstyle, None),
                x: pr.x + (pr.w - tw) / 2.0,
                y: pr.y + TextMeasurer::vertical_center(&pstyle, PILL_H),
                color: tint.0,
            });
        }
    }
}
