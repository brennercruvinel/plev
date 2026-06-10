use super::workspace::ThemeMode;
use crate::components::button::{ButtonKind, ButtonSize, draw as draw_button};
use crate::theme::Theme;
use plev::compositor::{Compositor, SceneNode, TextNodeKey};

pub const HEADER_H: f32 = 48.0;

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

    pub fn render(
        &mut self,
        compositor: &mut Compositor,
        theme: &Theme,
        theme_mode: ThemeMode,
        vw: f32,
        sidebar_w: f32,
    ) {
        let x = sidebar_w;
        let w = vw - sidebar_w;

        // Bar background
        compositor.push(SceneNode::Rect {
            x,
            y: 0.0,
            w,
            h: HEADER_H,
            color: theme.bg_2.to_array(),
        });
        // Bottom border
        compositor.push(SceneNode::Rect {
            x,
            y: HEADER_H - 1.0,
            w,
            h: 1.0,
            color: theme.border.to_array(),
        });

        // App name
        compositor.push(SceneNode::Text {
            key: TextNodeKey::new("basicIDE", 14.0, 18.0, None).with_weight(700),
            x: x + 16.0,
            y: (HEADER_H - 18.0) / 2.0,
            color: theme.text_1.to_array(),
        });

        // Plev badge (rounded rect)
        let badge_x = x + 104.0;
        let badge_y = (HEADER_H - 20.0) / 2.0;
        compositor.push(SceneNode::RoundedRect {
            x: badge_x,
            y: badge_y,
            w: 44.0,
            h: 20.0,
            color: [theme.pop.0[0], theme.pop.0[1], theme.pop.0[2], 0.15],
            corner_radius: 4.0,
            border_width: 0.0,
            border_color: [0.0; 4],
        });
        compositor.push(SceneNode::Text {
            key: TextNodeKey::new("plev", 11.0, 14.0, None).with_weight(600),
            x: badge_x + 10.0,
            y: (HEADER_H - 14.0) / 2.0,
            color: theme.pop.to_array(),
        });

        // Repo name (center)
        compositor.push(SceneNode::Text {
            key: TextNodeKey::new("plev/experiment", 13.0, 18.0, None).with_weight(400),
            x: x + w / 2.0 - 60.0,
            y: (HEADER_H - 18.0) / 2.0,
            color: theme.text_2.to_array(),
        });

        // Theme toggle button (right side)
        let mode_label = match theme_mode {
            ThemeMode::Dark => "Light",
            ThemeMode::Light => "Dark",
        };
        let btn_x = x + w - 80.0;
        let btn_y = (HEADER_H - 28.0) / 2.0;
        self.theme_btn_rect = draw_button(
            compositor,
            theme,
            btn_x,
            btn_y,
            mode_label,
            ButtonKind::Ghost,
            ButtonSize::Sm,
            false,
            false,
        );
    }
}
