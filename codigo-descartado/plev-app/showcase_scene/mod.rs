//! Showcase scene builder -- used by window.rs for the main app view.
//! Pure scene construction, no GPU or platform dependencies.

pub(crate) mod card_types;
mod cards;
mod cards_row3;
pub(crate) mod helpers;
mod state;

#[cfg(test)]
mod tests;

pub use helpers::clear_color;

use crate::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::input::{InputState, ViewId};
use crate::theme::Theme;

use card_types::*;
use cards::*;
use cards_row3::*;
use helpers::palette;

pub struct ShowcaseState {
    pub bg_layer: LayerId,
    pub overlay_layer: LayerId,
    pub click_count: u32,
    pub btn_hovered: bool,
    pub btn_view_id: Option<ViewId>,
    pub frame: u64,
    pub(crate) fps_last_time: web_time::Instant,
    pub(crate) fps_frame_count: u32,
    pub(crate) fps_display: f32,
}

impl ShowcaseState {
    pub fn new(compositor: &mut Compositor) -> Self {
        let bg_layer = compositor.create_layer(-1);
        let overlay_layer = compositor.create_layer(1);
        compositor.set_layer_opacity(overlay_layer, 0.85);
        Self {
            bg_layer,
            overlay_layer,
            click_count: 0,
            btn_hovered: false,
            btn_view_id: None,
            frame: 0,
            fps_last_time: web_time::Instant::now(),
            fps_frame_count: 0,
            fps_display: 0.0,
        }
    }

    pub fn build_scene(
        &mut self,
        compositor: &mut Compositor,
        input_state: &mut InputState,
        theme: &Theme,
        w: f32,
        h: f32,
        counter_value: u64,
    ) {
        #[allow(non_snake_case, unused_variables)]
        let (BG, SURFACE, ACCENT, ACCENT_DIM, GREEN, RED, YELLOW, CYAN, TEXT, TEXT_DIM, TEXT_MID, DIVIDER) = palette(theme);

        self.tick_fps();

        let margin = 32.0;
        let card_w = (w - margin * 2.0 - 16.0 * 3.0) / 4.0;
        let card_h = (h - 130.0 - 32.0 - 16.0 * 2.0 - 20.0) / 3.0;
        let card_h = card_h.clamp(130.0, 190.0);
        let row1_y = 130.0;
        let row2_y = row1_y + card_h + 16.0;
        let row3_y = row2_y + card_h + 16.0;

        // === BACKGROUND LAYER -- dot grid ===
        {
            let step = 40.0;
            let mut gx = margin;
            while gx < w - margin {
                let mut gy = 120.0;
                while gy < h - 30.0 {
                    compositor.push_to_layer(self.bg_layer, SceneNode::Rect {
                        x: gx, y: gy, w: 1.0, h: 1.0,
                        color: [0.15, 0.15, 0.22, 0.5],
                    });
                    gy += step;
                }
                gx += step;
            }
        }

        // === HEADER ===
        compositor.push(SceneNode::Rect { x: 0.0, y: 0.0, w, h: 110.0, color: [0.08, 0.08, 0.14, 1.0] });
        compositor.push(SceneNode::Rect { x: 0.0, y: 109.0, w, h: 1.0, color: DIVIDER });
        compositor.draw_text(TextNodeKey::new("\u{03A6} ENGINE", 36.0, 44.0, Some(w - margin * 2.0)), margin, 20.0, TEXT);
        compositor.draw_text(
            TextNodeKey::new("GPU-first compositing engine in Rust   \u{2022}   wgpu 28   \u{2022}   6 platforms", 13.0, 18.0, Some(w - margin * 2.0)),
            margin, 68.0, TEXT_DIM,
        );
        compositor.push(SceneNode::Rect { x: w - margin - 72.0, y: 26.0, w: 72.0, h: 24.0, color: ACCENT_DIM });
        compositor.draw_text(TextNodeKey::new("v0.3.0", 12.0, 16.0, Some(60.0)), w - margin - 66.0, 30.0, TEXT);

        // Helper to build a CardLayout for a grid position.
        let lay = |col: f32, row_y: f32| -> card_types::CardLayout {
            card_types::CardLayout {
                cx: margin + col * (card_w + 16.0), cy: row_y,
                card_w, card_h, surface: SURFACE, accent_dim: ACCENT_DIM,
            }
        };

        let colors = card_types::CardColors {
            accent: ACCENT, green: GREEN, red: RED, yellow: YELLOW,
            cyan: CYAN, text: TEXT, text_dim: TEXT_DIM, text_mid: TEXT_MID,
        };

        // === ROW 1 ===
        card_quad_rendering(compositor, lay(0.0, row1_y), &colors);
        card_text_system(compositor, lay(1.0, row1_y), &colors);
        card_layer_system(compositor, lay(2.0, row1_y), GREEN, TEXT_DIM);
        card_effects(compositor, lay(3.0, row1_y), TEXT_DIM, TEXT_MID);

        // === ROW 2 ===
        card_builder_api(compositor, lay(0.0, row2_y), &colors);
        card_input_system(self, compositor, input_state, lay(1.0, row2_y), ACCENT, GREEN, TEXT, TEXT_DIM, TEXT_MID);
        card_signals(compositor, lay(2.0, row2_y), ORANGE, TEXT_DIM, counter_value);
        card_platforms(compositor, lay(3.0, row2_y), ACCENT, GREEN, YELLOW, RED, CYAN, TEXT_DIM);

        // === ROW 3 ===
        card_dispatch(compositor, lay(0.0, row3_y), ACCENT, GREEN, RED, TEXT_DIM);
        card_overlays(compositor, lay(1.0, row3_y), YELLOW, TEXT_DIM);
        card_animation(compositor, lay(2.0, row3_y), TEXT_DIM, self.frame);
        card_vector_paths(compositor, lay(3.0, row3_y), GREEN, YELLOW, CYAN, TEXT_DIM);

        // === OVERLAY -- animated accent glow ===
        let glow_phase = (self.frame as f32 / 120.0).sin() * 0.5 + 0.5;
        let glow_x = margin + glow_phase * (w - margin * 2.0 - 200.0);
        compositor.push_to_layer(self.overlay_layer, SceneNode::Rect {
            x: glow_x, y: 106.0, w: 200.0, h: 3.0,
            color: [ACCENT[0], ACCENT[1], ACCENT[2], 0.4 + glow_phase * 0.4],
        });

        // === FOOTER ===
        compositor.push(SceneNode::Rect { x: 0.0, y: h - 32.0, w, h: 32.0, color: [0.07, 0.07, 0.12, 1.0] });
        compositor.push(SceneNode::Rect { x: 0.0, y: h - 32.0, w, h: 1.0, color: DIVIDER });
        compositor.draw_text(
            TextNodeKey::new("Rust 2024 \u{2022} wgpu 28 \u{2022} cosmic-text 0.18 \u{2022} winit 0.30 \u{2022} Taffy 0.9", 11.0, 14.0, Some(w - margin * 2.0)),
            margin, h - 22.0, TEXT_DIM,
        );
        let fps_text = if self.fps_display > 0.0 {
            format!("{:.0} FPS | F{}", self.fps_display, self.frame)
        } else {
            format!("F{}", self.frame)
        };
        compositor.draw_text(TextNodeKey::new(&fps_text, 11.0, 14.0, Some(160.0)), w - margin - 120.0, h - 22.0, TEXT_DIM);
    }
}
