//! Top bar — HOFF column head proportions (68px, 12px padding):
//! app name in the `title` mixin (20/1.2/500) at rgba($n2,.76), a "plev"
//! glass tag, repo · branch centered in base-2r at rgba($n2,.4), and the
//! theme toggle as a 36px glass pill on the right.

use super::workspace::ThemeMode;
use crate::components::badge::{self, BadgeKind};
use crate::components::button::{ButtonKind, ButtonSize, draw as draw_button, width_for};
use crate::components::hoff;
use crate::theme::Theme;
use plev::compositor::{Compositor, SceneNode, TextNodeKey};
use plev::text::TextStyle;

pub const HEADER_H: f32 = 68.0;
const PAD_X: f32 = 12.0;
const TITLE_SIZE: f32 = 20.0;
const TITLE_LINE_H: f32 = 20.0 * 1.2;

pub struct Header {
    theme_btn_rect: (f32, f32, f32, f32),
}

impl Header {
    pub fn new() -> Self {
        Self {
            theme_btn_rect: (0.0, 0.0, 0.0, 0.0),
        }
    }

    /// Hit-test for the theme toggle button.
    pub fn hit_test_theme_btn(&self, cx: f32, cy: f32) -> bool {
        let (bx, by, bw, bh) = self.theme_btn_rect;
        cx >= bx && cx <= bx + bw && cy >= by && cy <= by + bh
    }

    // One over the limit; a bag struct for two labels + two widths would
    // just be repacked here (card.rs trade-off).
    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &mut self,
        compositor: &mut Compositor,
        theme: &Theme,
        theme_mode: ThemeMode,
        vw: f32,
        sidebar_w: f32,
        repo_label: &str,
        branch_label: &str,
    ) {
        let x = sidebar_w;
        let w = vw - sidebar_w;

        // Bar surface — same glass as the sidebar, with a hairline edge below.
        compositor.push(SceneNode::Rect {
            x: 0.0,
            y: 0.0,
            w: vw,
            h: HEADER_H,
            color: theme.bg_sidebar.to_array(),
        });
        compositor.push(SceneNode::Rect {
            x: 0.0,
            y: HEADER_H - 1.0,
            w: vw,
            h: 1.0,
            color: theme.edge.to_array(),
        });

        // App name — title (20/500) at .76. One style measures AND draws,
        // so the tag placed after the name never overlaps it.
        let title_style = TextStyle::new(TITLE_SIZE)
            .with_line_height(TITLE_LINE_H)
            .with_weight(500);
        compositor.push(SceneNode::Text {
            key: TextNodeKey::from_style("basicIDE", &title_style, None),
            x: x + PAD_X,
            y: (HEADER_H - TITLE_LINE_H) / 2.0,
            color: theme.text_active.to_array(),
        });

        // "plev" glass tag next to the name.
        let name_w = hoff::measure_text("basicIDE", &title_style);
        badge::draw(
            compositor,
            theme,
            x + PAD_X + name_w + 12.0,
            (HEADER_H - 22.0) / 2.0,
            "plev",
            BadgeKind::Tag,
        );

        // Repo name + current branch (center) — base-2r at .4.
        let center = if branch_label.is_empty() {
            repo_label.to_string()
        } else {
            format!("{repo_label} \u{00B7} {branch_label}")
        };
        let center_style = TextStyle::new(14.0).with_line_height(14.0 * 1.4);
        let center_w = hoff::measure_text(&center, &center_style);
        compositor.push(SceneNode::Text {
            key: TextNodeKey::from_style(&center, &center_style, None),
            x: x + (w - center_w) / 2.0,
            y: (HEADER_H - 14.0 * 1.4) / 2.0,
            color: theme.text_muted.to_array(),
        });

        // Theme toggle — 36px glass pill on the right.
        let mode_label = match theme_mode {
            ThemeMode::Dark => "Light",
            ThemeMode::Light => "Dark",
        };
        // Same real measurement draw_button uses (Sm pads 16).
        let btn_w = width_for(mode_label, ButtonSize::Sm);
        let btn_x = vw - PAD_X - btn_w;
        let btn_y = (HEADER_H - 36.0) / 2.0;
        self.theme_btn_rect = draw_button(
            compositor,
            theme,
            btn_x,
            btn_y,
            mode_label,
            ButtonKind::Glass,
            ButtonSize::Sm,
            false,
            false,
        );
    }
}
