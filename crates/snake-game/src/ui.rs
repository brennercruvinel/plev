//! Scene building for the Snake example.

use plev::compositor::{Compositor, RoundedRectParams, TextNodeKey};
use web_time::Instant;

use crate::state::*;

impl SnakeGame {
    pub(crate) fn build_scene(&mut self, compositor: &mut Compositor, w: f32, h: f32) {
        self.frame += 1;
        self.fps_count += 1;
        let now = Instant::now();
        let fps_elapsed = now.duration_since(self.fps_time).as_secs_f32();
        if fps_elapsed >= 1.0 {
            self.fps_display = self.fps_count as f32 / fps_elapsed;
            self.fps_count = 0;
            self.fps_time = now;
        }

        self.update_flash();

        // Compute grid geometry
        let grid_area_w = w - GRID_PAD * 2.0;
        let grid_area_h = h - HEADER_H - FOOTER_H - GRID_PAD * 2.0;
        let cell_w = (grid_area_w / GRID_W as f32).floor();
        let cell_h = (grid_area_h / GRID_H as f32).floor();
        let cell = cell_w.min(cell_h);
        let total_w = cell * GRID_W as f32;
        let total_h = cell * GRID_H as f32;
        let ox = ((w - total_w) / 2.0).floor();
        let oy = HEADER_H + ((h - HEADER_H - FOOTER_H - total_h) / 2.0).floor();

        // Header
        self.build_header(compositor, w, h);

        // Grid
        self.build_grid(compositor, ox, oy, total_w, total_h, cell);

        // Game over overlay
        if self.game_over {
            self.build_game_over(compositor, ox, oy, total_w, total_h);
        }

        // Footer
        self.build_footer(compositor, w, h);
    }

    fn build_header(&self, compositor: &mut Compositor, w: f32, _h: f32) {
        compositor.draw_rect(0.0, 0.0, w, HEADER_H, [0.06, 0.06, 0.11, 1.0]);
        compositor.draw_rect(0.0, HEADER_H - 1.0, w, 1.0, DIVIDER);

        let title = if self.ai_mode { "SNAKE (AI)" } else { "SNAKE" };
        compositor.draw_text(
            TextNodeKey::new(title, 22.0, 28.0, None),
            24.0,
            10.0,
            TEXT_COLOR,
        );

        let score_text = format!("Score: {}   High: {}", self.score, self.high_score);
        compositor.draw_text(
            TextNodeKey::new(&score_text, 13.0, 18.0, None),
            24.0,
            34.0,
            TEXT_DIM,
        );

        // Mode badge
        let mode_label = if self.ai_mode { "AI" } else { "MANUAL" };
        let badge_w = if self.ai_mode { 32.0 } else { 64.0 };
        compositor.draw_rounded_rect(RoundedRectParams {
            x: w - 24.0 - badge_w,
            y: 14.0,
            w: badge_w,
            h: 22.0,
            color: ACCENT,
            corner_radius: 4.0,
            border_width: 0.0,
            border_color: [0.0; 4],
        });
        compositor.draw_text(
            TextNodeKey::new(mode_label, 11.0, 14.0, Some(badge_w - 8.0)),
            w - 24.0 - badge_w + 4.0,
            18.0,
            TEXT_COLOR,
        );
    }

    fn build_grid(
        &self,
        compositor: &mut Compositor,
        ox: f32,
        oy: f32,
        total_w: f32,
        total_h: f32,
        cell: f32,
    ) {
        // Grid background + border
        compositor.draw_rect(ox - 1.0, oy - 1.0, total_w + 2.0, total_h + 2.0, DIVIDER);
        compositor.draw_rect(ox, oy, total_w, total_h, GRID_BG);

        // Grid lines
        for gx in 0..=GRID_W {
            let x = ox + gx as f32 * cell;
            compositor.draw_rect(x, oy, 1.0, total_h, GRID_LINE);
        }
        for gy in 0..=GRID_H {
            let y = oy + gy as f32 * cell;
            compositor.draw_rect(ox, y, total_w, 1.0, GRID_LINE);
        }

        // Flash overlay
        if self.food_flash > 0.01 {
            compositor.draw_rect(
                ox,
                oy,
                total_w,
                total_h,
                [1.0, 0.95, 0.5, self.food_flash * 0.08],
            );
        }

        let snake_len = self.body.len();
        let gap = (cell * 0.08).max(1.0);
        let radius = (cell * 0.2).max(2.0);

        // Draw cells
        for cy in 0..GRID_H {
            for cx in 0..GRID_W {
                let cell_type = self.cell_at((cx, cy));
                let px = ox + cx as f32 * cell + gap;
                let py = oy + cy as f32 * cell + gap;
                let sz = cell - gap * 2.0;

                match cell_type {
                    Cell::Head => {
                        self.draw_head(compositor, px, py, sz, radius);
                    }
                    Cell::Snake => {
                        let fade = self
                            .body
                            .iter()
                            .position(|&p| p == (cx, cy))
                            .map(|i| i as f32 / snake_len.max(1) as f32)
                            .unwrap_or(0.5);
                        let color = lerp_color(SNAKE_BODY, SNAKE_TAIL, fade);
                        compositor.draw_rounded_rect(RoundedRectParams {
                            x: px,
                            y: py,
                            w: sz,
                            h: sz,
                            color,
                            corner_radius: radius,
                            border_width: 0.0,
                            border_color: [0.0; 4],
                        });
                    }
                    Cell::Food => {
                        let glow_sz = sz + 6.0;
                        compositor.draw_rounded_rect(RoundedRectParams {
                            x: px - 3.0,
                            y: py - 3.0,
                            w: glow_sz,
                            h: glow_sz,
                            color: FOOD_GLOW,
                            corner_radius: radius * 2.0,
                            border_width: 0.0,
                            border_color: [0.0; 4],
                        });
                        let pulse = ((self.frame as f32 * 0.08).sin() * 0.5 + 0.5) * 0.15;
                        let food_c = [FOOD_COLOR[0], FOOD_COLOR[1] - pulse, FOOD_COLOR[2], 1.0];
                        compositor.draw_rounded_rect(RoundedRectParams {
                            x: px + 1.0,
                            y: py + 1.0,
                            w: sz - 2.0,
                            h: sz - 2.0,
                            color: food_c,
                            corner_radius: radius * 1.2,
                            border_width: 0.0,
                            border_color: [0.0; 4],
                        });
                    }
                    Cell::Wall => {
                        compositor.draw_rect(px, py, sz, sz, WALL_COLOR);
                    }
                    Cell::Empty => {}
                }
            }
        }
    }

    fn draw_head(&self, compositor: &mut Compositor, px: f32, py: f32, sz: f32, radius: f32) {
        compositor.draw_rounded_rect(RoundedRectParams {
            x: px,
            y: py,
            w: sz,
            h: sz,
            color: SNAKE_HEAD,
            corner_radius: radius * 1.5,
            border_width: 0.0,
            border_color: [0.0; 4],
        });
        // Eyes
        let eye_r = (sz * 0.12).max(1.5);
        let cx_f = px + sz / 2.0;
        let cy_f = py + sz / 2.0;
        let (ex1, ey1, ex2, ey2) = match self.dir {
            (1, 0) => (
                cx_f + sz * 0.15,
                cy_f - sz * 0.15,
                cx_f + sz * 0.15,
                cy_f + sz * 0.15,
            ),
            (-1, 0) => (
                cx_f - sz * 0.15,
                cy_f - sz * 0.15,
                cx_f - sz * 0.15,
                cy_f + sz * 0.15,
            ),
            (0, -1) => (
                cx_f - sz * 0.15,
                cy_f - sz * 0.15,
                cx_f + sz * 0.15,
                cy_f - sz * 0.15,
            ),
            _ => (
                cx_f - sz * 0.15,
                cy_f + sz * 0.15,
                cx_f + sz * 0.15,
                cy_f + sz * 0.15,
            ),
        };
        compositor.draw_rounded_rect(RoundedRectParams {
            x: ex1 - eye_r,
            y: ey1 - eye_r,
            w: eye_r * 2.0,
            h: eye_r * 2.0,
            color: [0.05, 0.05, 0.10, 1.0],
            corner_radius: eye_r,
            border_width: 0.0,
            border_color: [0.0; 4],
        });
        compositor.draw_rounded_rect(RoundedRectParams {
            x: ex2 - eye_r,
            y: ey2 - eye_r,
            w: eye_r * 2.0,
            h: eye_r * 2.0,
            color: [0.05, 0.05, 0.10, 1.0],
            corner_radius: eye_r,
            border_width: 0.0,
            border_color: [0.0; 4],
        });
    }

    fn build_game_over(
        &self,
        compositor: &mut Compositor,
        ox: f32,
        oy: f32,
        total_w: f32,
        total_h: f32,
    ) {
        compositor.draw_rect(ox, oy, total_w, total_h, [0.0, 0.0, 0.0, 0.6]);
        compositor.draw_text(
            TextNodeKey::new("GAME OVER", 32.0, 40.0, Some(total_w)),
            ox + total_w / 2.0 - 100.0,
            oy + total_h / 2.0 - 30.0,
            [1.0, 0.3, 0.25, 1.0],
        );
        let restart_text = format!("Score: {} | Press SPACE or R to restart", self.score);
        compositor.draw_text(
            TextNodeKey::new(&restart_text, 14.0, 18.0, Some(total_w)),
            ox + total_w / 2.0 - 140.0,
            oy + total_h / 2.0 + 14.0,
            TEXT_DIM,
        );
    }

    fn build_footer(&self, compositor: &mut Compositor, w: f32, h: f32) {
        compositor.draw_rect(0.0, h - FOOTER_H, w, FOOTER_H, [0.05, 0.05, 0.09, 1.0]);
        compositor.draw_rect(0.0, h - FOOTER_H, w, 1.0, DIVIDER);
        compositor.draw_text(
            TextNodeKey::new(
                "plev Engine | Arrow keys: move | A: toggle AI | Space/R: restart",
                10.0,
                13.0,
                Some(w - 200.0),
            ),
            24.0,
            h - FOOTER_H + 8.0,
            TEXT_DIM,
        );
        let fps = format!("{:.0} FPS | F{}", self.fps_display, self.frame);
        compositor.draw_text(
            TextNodeKey::new(&fps, 10.0, 13.0, Some(120.0)),
            w - 140.0,
            h - FOOTER_H + 8.0,
            TEXT_DIM,
        );
    }
}
