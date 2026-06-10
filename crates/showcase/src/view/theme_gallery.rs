//! Theme section: the HOFF token strip, palette cards, typography ramp.

use plev::compositor::{Compositor, SceneNode, TextNodeKey};
use plev::theme::Theme;
use plev::ui::icons;
use plev::ui::widgets::{EventResult, Rect, WidgetEvent};

use super::{group_label, text};

pub const THEMES: &[&str] = &[
    "hoff",
    "dark",
    "light",
    "catppuccin",
    "dracula",
    "tokyo-night",
    "rose-pine",
    "nord",
    "gruvbox",
    "github-dark",
    "one-dark",
    "kanagawa",
    "moonlight",
];

/// Resolve a showcase theme name to a `Theme`.
pub fn resolve(name: &str) -> Option<Theme> {
    match name {
        "dark" => Some(Theme::dark()),
        "light" => Some(Theme::light()),
        other => Theme::named(other),
    }
}

const CARD_W: f32 = 168.0;
const CARD_H: f32 = 84.0;
const GAP: f32 = 12.0;
const LABEL_H: f32 = 24.0;
/// Height of the HOFF token strip above the palette grid.
const TOKENS_H: f32 = 110.0;

pub struct ThemeSection {
    hovered: Option<usize>,
}

impl ThemeSection {
    pub fn new() -> Self {
        Self { hovered: None }
    }

    fn card_rects(&self, content: Rect) -> Vec<Rect> {
        let cols = ((content.w + GAP) / (CARD_W + GAP)).floor().max(1.0) as usize;
        THEMES
            .iter()
            .enumerate()
            .map(|(i, _)| {
                let col = i % cols;
                let row = i / cols;
                Rect::new(
                    content.x + col as f32 * (CARD_W + GAP),
                    content.y + TOKENS_H + LABEL_H + row as f32 * (CARD_H + GAP),
                    CARD_W,
                    CARD_H,
                )
            })
            .collect()
    }

    /// The HOFF signature strip: the canonical white-alpha ramp and the
    /// four chromatic accents, drawn with the current theme's glass.
    fn render_hoff_tokens(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        use plev::theme::hoff;
        group_label(c, "HOFF ALPHAS — #F8F8F8", content.x, content.y, theme);
        let alphas: [(f32, &str); 7] = [
            (0.02, ".02"),
            (0.05, ".05"),
            (0.10, ".10"),
            (0.25, ".25"),
            (0.40, ".40"),
            (0.70, ".70"),
            (0.95, ".95"),
        ];
        let sw = 56.0;
        let y = content.y + LABEL_H;
        for (i, (a, label)) in alphas.iter().enumerate() {
            let x = content.x + i as f32 * (sw + 8.0);
            c.push(SceneNode::RoundedRect {
                x,
                y,
                w: sw,
                h: 36.0,
                color: [hoff::N2.0[0], hoff::N2.0[1], hoff::N2.0[2], *a],
                corner_radius: theme.radius.sm,
                border_width: 1.0,
                border_color: theme.glass.edge_soft.0,
            });
            text(
                c,
                label,
                12.0,
                600,
                x + 16.0,
                y + 40.0,
                theme.glass.text_faint.0,
            );
        }

        // Accents: the only chromatic tokens in the system.
        let accents = [
            (hoff::RED, "red"),
            (hoff::GREEN, "green"),
            (hoff::GREEN_LIGHT, "repost"),
            (hoff::ORANGE, "heart"),
        ];
        let ax = content.x + alphas.len() as f32 * (sw + 8.0) + 24.0;
        for (i, (color, label)) in accents.iter().enumerate() {
            let x = ax + i as f32 * (sw + 8.0);
            c.push(SceneNode::RoundedRect {
                x: x + sw / 2.0 - 9.0,
                y: y + 9.0,
                w: 18.0,
                h: 18.0,
                color: color.0,
                corner_radius: 9.0,
                border_width: 0.0,
                border_color: [0.0; 4],
            });
            text(
                c,
                label,
                12.0,
                400,
                x + 12.0,
                y + 40.0,
                theme.glass.text_faint.0,
            );
        }
    }

    /// Returns the picked theme name, if a card was clicked.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
    ) -> (EventResult, Option<&'static str>) {
        let rects = self.card_rects(content);
        match *event {
            WidgetEvent::MouseMove { x, y } => {
                let hit = rects.iter().position(|r| r.contains(x, y));
                if hit != self.hovered {
                    self.hovered = hit;
                    (EventResult::changed(), None)
                } else {
                    (EventResult::IGNORED, None)
                }
            }
            WidgetEvent::MouseDown { x, y } => {
                if let Some(i) = rects.iter().position(|r| r.contains(x, y)) {
                    (EventResult::clicked(), Some(THEMES[i]))
                } else {
                    (EventResult::IGNORED, None)
                }
            }
            _ => (EventResult::IGNORED, None),
        }
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme, current: &str) {
        self.render_hoff_tokens(c, content, theme);
        group_label(c, "PALETTES", content.x, content.y + TOKENS_H, theme);

        let rects = self.card_rects(content);
        let mut bottom = content.y;
        for (i, (name, rect)) in THEMES.iter().zip(&rects).enumerate() {
            let Some(t) = resolve(name) else { continue };
            let is_current = *name == current;
            let hovered = self.hovered == Some(i);

            // Card painted with the *target* theme's own colors. Path-based
            // because the current-theme check icon must stack on top.
            c.push(plev::ui::widgets::path_rounded_rect(
                rect.x,
                rect.y,
                rect.w,
                rect.h,
                theme.radius.lg,
                t.colors.bg_panel.0,
            ));
            c.push(plev::ui::widgets::path_rounded_rect_stroke(
                rect.x,
                rect.y,
                rect.w,
                rect.h,
                theme.radius.lg,
                if is_current {
                    theme.colors.accent.0
                } else if hovered {
                    theme.colors.border_active.0
                } else {
                    theme.colors.divider.0
                },
                if is_current { 1.5 } else { 1.0 },
            ));
            c.push(SceneNode::Text {
                key: TextNodeKey::new(name, 12.0, 12.0 * 1.3, None).with_weight(600),
                x: rect.x + 12.0,
                y: rect.y + 10.0,
                color: t.colors.text.0,
            });
            if is_current
                && let Some(node) = icons::icon_at(
                    "check",
                    12.0,
                    theme.colors.accent.0,
                    rect.x + rect.w - 22.0,
                    rect.y + 11.0,
                )
            {
                c.push(node);
            }

            // Swatch row: bg / accent / success / danger / text.
            let swatches = [
                t.colors.bg.0,
                t.colors.accent.0,
                t.colors.success.0,
                t.colors.danger.0,
                t.colors.text.0,
            ];
            for (si, color) in swatches.iter().enumerate() {
                c.push(SceneNode::RoundedRect {
                    x: rect.x + 12.0 + si as f32 * 24.0,
                    y: rect.y + rect.h - 30.0,
                    w: 18.0,
                    h: 18.0,
                    color: *color,
                    corner_radius: 5.0,
                    border_width: 1.0,
                    border_color: t.colors.divider.0,
                });
            }
            bottom = bottom.max(rect.y + rect.h);
        }

        // Typography ramp with the *current* theme.
        let mut y = bottom + 34.0;
        group_label(c, "TYPOGRAPHY", content.x, y, theme);
        y += LABEL_H;
        let ramp = [
            ("Display", theme.typography.display, 700u16),
            ("Title", theme.typography.title, 700),
            ("Title sm", theme.typography.title_sm, 600),
            ("Body", theme.typography.body, 400),
            ("Body sm", theme.typography.body_sm, 400),
            ("Caption", theme.typography.caption, 400),
        ];
        for (name, size, weight) in ramp {
            if y + size * 1.3 > content.y + content.h {
                break;
            }
            text(
                c,
                &format!("{name} — quick brown fox {size}px"),
                size,
                weight,
                content.x,
                y,
                theme.colors.text.0,
            );
            y += size * 1.3 + 6.0;
        }
    }
}
