// Rendering: scene construction for the touch demo.

use plev::compositor::{Compositor, TextNodeKey};

use crate::palette::*;
use crate::state::{State, TouchDemoApp};

impl TouchDemoApp {
    pub fn render(&mut self) {
        self.compositor.begin_frame();

        let State::Ready {
            ref mut gpu,
            ref mut text_system,
        } = self.state
        else {
            return;
        };

        let surface = match gpu.surface.as_ref() {
            Some(s) => s,
            None => return,
        };

        let output = match surface.get_current_texture() {
            Ok(t) => t,
            Err(_) => {
                gpu.resize(gpu.surface_config.width, gpu.surface_config.height);
                return;
            }
        };

        let view = output
            .texture
            .create_view(&plev::wgpu::TextureViewDescriptor::default());

        let w = gpu.surface_config.width as f32;
        let h = gpu.surface_config.height as f32;

        text_system.begin_frame();

        let margin = 32.0;
        let header_h = 70.0;
        let footer_h = 32.0;

        let compositor = &mut self.compositor;

        // Header
        compositor.draw_rect(0.0, 0.0, w, header_h, HEADER_BG);
        compositor.draw_rect(0.0, header_h - 1.0, w, 1.0, DIVIDER);
        compositor.draw_text(
            TextNodeKey::new("TOUCH & GESTURE DEMO", 24.0, 30.0, Some(w - margin * 2.0)),
            margin, 16.0, TEXT,
        );
        compositor.draw_text(
            TextNodeKey::new(
                "6-state gesture recognizer per touch ID",
                12.0, 16.0, Some(w - margin * 2.0),
            ),
            margin, 48.0, TEXT_DIM,
        );

        // Content area
        let content_y = header_h + 16.0;
        let content_w = w - margin * 2.0;
        let card_gap = 16.0;

        // Left column: interactive area card (60%)
        let left_w = (content_w - card_gap) * 0.60;
        let left_x = margin;
        let left_card_h = h - content_y - footer_h - 16.0;

        card(compositor, left_x, content_y, left_w, left_card_h, ACCENT);

        compositor.draw_text(
            TextNodeKey::new("INTERACTIVE AREA", 11.0, 14.0, Some(left_w - 32.0)),
            left_x + 16.0, content_y + 14.0, ACCENT,
        );
        compositor.draw_rect(
            left_x + 16.0, content_y + 34.0, left_w - 32.0, 1.0, DIVIDER,
        );

        Self::build_interactive_area(
            compositor,
            left_x, content_y, left_w, left_card_h,
            self.rect_x, self.rect_y, self.rect_w, self.rect_h,
            self.rect_scale, self.rect_color, &self.status_text,
        );
        Self::build_gesture_panel(compositor, margin, content_y, content_w, left_w, card_gap, left_card_h);

        // Footer
        let footer_y = h - footer_h;
        compositor.draw_rect(0.0, footer_y - 1.0, w, 1.0, DIVIDER);
        compositor.draw_rect(0.0, footer_y, w, footer_h, FOOTER_BG);
        compositor.draw_text(
            TextNodeKey::new(
                "TouchInputState  |  GestureEvent  |  Phase state machine",
                11.0, 15.0, Some(w - margin * 2.0),
            ),
            margin, footer_y + 9.0, TEXT_DIM,
        );

        // GPU resolve + submit
        crate::gpu::submit(compositor, gpu, text_system, &view, output);
    }

    fn build_interactive_area(
        compositor: &mut Compositor,
        left_x: f32, content_y: f32, left_w: f32, left_card_h: f32,
        rect_x: f32, rect_y: f32, rect_w: f32, rect_h: f32,
        rect_scale: f32, rect_color: [f32; 4], status_text: &str,
    ) {
        let inner_x = left_x + 16.0;
        let inner_y = content_y + 46.0;
        let inner_w = left_w - 32.0;
        let inner_h = left_card_h - 140.0;

        compositor.draw_rect(
            inner_x, inner_y, inner_w, inner_h, [0.08, 0.08, 0.12, 1.0],
        );

        let scaled_w = rect_w * rect_scale;
        let scaled_h = rect_h * rect_scale;
        compositor.draw_rect(
            inner_x + rect_x, inner_y + rect_y,
            scaled_w, scaled_h, rect_color,
        );

        compositor.draw_text(
            TextNodeKey::new("Touch me", 18.0, 24.0, Some(scaled_w - 20.0)),
            inner_x + rect_x + 10.0,
            inner_y + rect_y + scaled_h / 2.0 - 12.0,
            TEXT,
        );

        let status_y = inner_y + inner_h + 8.0;
        let status_h = 48.0;
        compositor.draw_rect(
            inner_x, status_y, inner_w, status_h, [0.08, 0.08, 0.13, 1.0],
        );
        compositor.draw_text(
            TextNodeKey::new("STATUS", 11.0, 14.0, Some(inner_w - 24.0)),
            inner_x + 12.0, status_y + 6.0, TEXT_DIM,
        );
        compositor.draw_text(
            TextNodeKey::new(status_text, 13.0, 17.0, Some(inner_w - 24.0)),
            inner_x + 12.0, status_y + 24.0, YELLOW,
        );

        let note_y = status_y + status_h + 8.0;
        compositor.draw_text(
            TextNodeKey::new(
                "macOS does not emit Touch events. Test on mobile or touchscreen.",
                10.0, 13.0, Some(inner_w),
            ),
            inner_x, note_y, TEXT_DIM,
        );
    }

    fn build_gesture_panel(
        compositor: &mut Compositor, margin: f32, content_y: f32, content_w: f32,
        left_w: f32, card_gap: f32, right_card_h: f32,
    ) {
        let right_x = margin + left_w + card_gap;
        let right_w = content_w - left_w - card_gap;

        card(compositor, right_x, content_y, right_w, right_card_h, PURPLE);

        compositor.draw_text(
            TextNodeKey::new("GESTURE TYPES", 11.0, 14.0, Some(right_w - 32.0)),
            right_x + 16.0, content_y + 14.0, PURPLE,
        );
        compositor.draw_rect(
            right_x + 16.0, content_y + 34.0, right_w - 32.0, 1.0, DIVIDER,
        );

        let ev_x = right_x + 16.0;
        let ev_max_w = right_w - 32.0;
        let mut ey = content_y + 50.0;

        let gesture_list: &[(&str, &str, [f32; 4])] = &[
            ("Tap",        "color -> green",   GREEN),
            ("Double-tap", "reset position",   ACCENT),
            ("Long-press", "color -> red",     RED),
            ("Drag",       "move rectangle",   YELLOW),
            ("Pinch",      "scale rectangle",  CYAN),
            ("Swipe",      "color -> purple",  PURPLE),
        ];

        for (name, effect, color) in gesture_list {
            compositor.draw_rect(ev_x, ey + 4.0, 4.0, 4.0, *color);
            compositor.draw_text(
                TextNodeKey::new(name, 14.0, 20.0, Some(ev_max_w - 16.0)),
                ev_x + 12.0, ey, TEXT,
            );
            ey += 20.0;
            compositor.draw_text(
                TextNodeKey::new(effect, 11.0, 14.0, Some(ev_max_w - 16.0)),
                ev_x + 12.0, ey, TEXT_DIM,
            );
            ey += 28.0;
        }

        compositor.draw_rect(
            right_x + 16.0, ey + 4.0, right_w - 32.0, 1.0, DIVIDER,
        );
        compositor.draw_text(
            TextNodeKey::new(
                "Phase: Started -> Changed -> Ended",
                11.0, 14.0, Some(ev_max_w),
            ),
            ev_x, ey + 14.0, ACCENT_DIM,
        );
    }
}
