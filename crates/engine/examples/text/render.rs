// Rendering: scene construction and GPU submission for the text demo.

use engine::compositor::TextNodeKey;

use crate::palette::*;
use crate::state::{State, TextDemoApp};

impl TextDemoApp {
    pub fn render(&mut self) {
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

        let surface_view = gpu.surface_render_view(&output);

        let w = gpu.surface_config.width as f32;
        let h = gpu.surface_config.height as f32;

        let compositor = &mut self.compositor;
        compositor.begin_frame();
        text_system.begin_frame();

        let margin = 32.0;
        let header_h = 70.0;
        let footer_h = 32.0;

        // Header
        compositor.draw_rect(0.0, 0.0, w, header_h, HEADER_BG);
        compositor.draw_text(
            TextNodeKey::from_style(
                "TEXT SYSTEM",
                &title_style(24.0, 30.0),
                Some(w - margin * 2.0),
            ),
            margin,
            16.0,
            TEXT,
        );
        compositor.draw_text(
            TextNodeKey::from_style(
                "cosmic-text + HarfBuzz shaping + Glyph atlas",
                &body_style(12.0, 16.0),
                Some(w - margin * 2.0),
            ),
            margin,
            48.0,
            TEXT_DIM,
        );
        compositor.draw_rect(0.0, header_h - 1.0, w, 1.0, DIVIDER);

        let content_y = header_h + 20.0;
        let content_w = w - margin * 2.0;
        let content_h = h - content_y - footer_h - 20.0;

        Self::build_typography_card(compositor, margin, content_y, content_w, content_h);
        Self::build_unicode_card(compositor, margin, content_y, content_w, content_h);
        Self::build_wrapping_card(compositor, margin, content_y, content_w, content_h);

        // Footer
        let footer_y = h - footer_h;
        compositor.draw_rect(0.0, footer_y - 1.0, w, 1.0, DIVIDER);
        compositor.draw_rect(0.0, footer_y, w, footer_h, FOOTER_BG);
        compositor.draw_text(
            TextNodeKey::from_style(
                "Glyph atlas: R8Unorm 512x512->4096  |  etagere packing  |  LRU eviction",
                &body_style(11.0, 14.0),
                Some(w - margin * 2.0),
            ),
            margin,
            footer_y + 9.0,
            TEXT_DIM,
        );

        crate::gpu::gpu_submit(compositor, gpu, text_system, &surface_view);
        output.present();
    }

    fn build_typography_card(
        compositor: &mut engine::compositor::Compositor,
        margin: f32,
        content_y: f32,
        content_w: f32,
        content_h: f32,
    ) {
        let left_w = content_w * 0.55 - 8.0;
        let left_x = margin;

        card(compositor, left_x, content_y, left_w, content_h, CYAN);

        compositor.draw_text(
            TextNodeKey::from_style(
                "TYPOGRAPHY SCALE",
                &card_title_style(11.0, 14.0),
                Some(left_w - 32.0),
            ),
            left_x + 16.0,
            content_y + 16.0,
            CYAN,
        );
        compositor.draw_rect(left_x + 16.0, content_y + 36.0, left_w - 32.0, 1.0, DIVIDER);

        let sx = left_x + 16.0;
        let sw = left_w - 32.0;
        let mut ty = content_y + 48.0;

        // (label, text, size, line height, advance, weight, color): the
        // scale demo carries semantic weights so the hierarchy is visible.
        type Sample = (&'static str, &'static str, f32, f32, f32, u16, [f32; 4]);
        let samples: &[Sample] = &[
            (
                "36px  Display",
                "The quick brown fox",
                36.0,
                44.0,
                52.0,
                700,
                TEXT,
            ),
            (
                "24px  Heading",
                "Jumps over the lazy dog",
                24.0,
                30.0,
                38.0,
                600,
                TEXT,
            ),
            (
                "18px  Body",
                "Body text for paragraphs and long-form content.",
                18.0,
                24.0,
                32.0,
                400,
                TEXT_MID,
            ),
            (
                "14px  Small",
                "Secondary information, labels, metadata fields.",
                14.0,
                20.0,
                28.0,
                400,
                TEXT_MID,
            ),
            (
                "11px  Caption",
                "Fine print, timestamps, auxiliary details and footnotes.",
                11.0,
                14.0,
                0.0,
                400,
                TEXT_MID,
            ),
        ];

        for (label, text, size, lh, advance, weight, color) in samples {
            compositor.draw_text(
                TextNodeKey::from_style(label, &label_style(11.0, 14.0), Some(sw)),
                sx,
                ty,
                TEXT_DIM,
            );
            ty += 16.0;
            let sample_style = body_style(*size, *lh).with_weight(*weight);
            compositor.draw_text(
                TextNodeKey::from_style(text, &sample_style, Some(sw)),
                sx,
                ty,
                *color,
            );
            ty += advance;
        }
    }

    fn build_unicode_card(
        compositor: &mut engine::compositor::Compositor,
        margin: f32,
        content_y: f32,
        content_w: f32,
        content_h: f32,
    ) {
        let right_x = margin + content_w * 0.55 + 8.0;
        let right_w = content_w * 0.45 - 8.0;
        let top_card_h = content_h * 0.48;

        card(compositor, right_x, content_y, right_w, top_card_h, GREEN);

        compositor.draw_text(
            TextNodeKey::from_style(
                "UNICODE",
                &card_title_style(11.0, 14.0),
                Some(right_w - 32.0),
            ),
            right_x + 16.0,
            content_y + 16.0,
            GREEN,
        );
        compositor.draw_rect(
            right_x + 16.0,
            content_y + 36.0,
            right_w - 32.0,
            1.0,
            DIVIDER,
        );

        let ux = right_x + 16.0;
        let uw = right_w - 32.0;
        let mut uy = content_y + 48.0;

        let unicode_samples: &[(&str, &str)] = &[
            (
                "Latin Extended",
                "Caf\u{00e9} na\u{00ef}ve r\u{00e9}sum\u{00e9} fa\u{00e7}ade \u{00fc}ber",
            ),
            (
                "CJK + Kana",
                "\u{6F22}\u{5B57}\u{3072}\u{3089}\u{304C}\u{306A}\u{30AB}\u{30BF}\u{30AB}\u{30CA}",
            ),
            (
                "Symbols + Math",
                "\u{2200}x \u{2208} \u{211d}: x\u{00b2} \u{2265} 0  |  \u{03b1}\u{03b2}\u{03b3}\u{03b4}  |  \u{2190}\u{2191}\u{2192}\u{2193}",
            ),
            ("Numbers + Punct.", "0123456789 !@#$%^&*(){}[]<>"),
        ];

        for (label, text) in unicode_samples {
            compositor.draw_text(
                TextNodeKey::from_style(label, &label_style(11.0, 14.0), Some(uw)),
                ux,
                uy,
                TEXT_DIM,
            );
            uy += 14.0;
            compositor.draw_text(
                TextNodeKey::from_style(text, &body_style(14.0, 20.0), Some(uw)),
                ux,
                uy,
                TEXT,
            );
            uy += 26.0;
        }
    }

    fn build_wrapping_card(
        compositor: &mut engine::compositor::Compositor,
        margin: f32,
        content_y: f32,
        content_w: f32,
        content_h: f32,
    ) {
        let right_x = margin + content_w * 0.55 + 8.0;
        let right_w = content_w * 0.45 - 8.0;
        let top_card_h = content_h * 0.48;
        let bottom_card_h = content_h - top_card_h - 16.0;
        let wrap_y = content_y + top_card_h + 16.0;

        card(compositor, right_x, wrap_y, right_w, bottom_card_h, PURPLE);

        compositor.draw_text(
            TextNodeKey::from_style(
                "TEXT WRAPPING",
                &card_title_style(11.0, 14.0),
                Some(right_w - 32.0),
            ),
            right_x + 16.0,
            wrap_y + 16.0,
            PURPLE,
        );
        compositor.draw_rect(right_x + 16.0, wrap_y + 36.0, right_w - 32.0, 1.0, DIVIDER);

        compositor.draw_text(
            TextNodeKey::from_style(
                "plev renders text via a glyph atlas built with etagere rectangle packing. \
                 Each unique (text, font_size) pair is shaped once by cosmic-text and cached \
                 in an FxHashMap. The atlas starts at 512x512 R8Unorm and grows up to 4096x4096 \
                 with LRU eviction for least-recently-used glyphs. This paragraph demonstrates \
                 automatic line wrapping within the card boundary.",
                &body_style(13.0, 18.0),
                Some(right_w - 32.0),
            ),
            right_x + 16.0,
            wrap_y + 48.0,
            TEXT_MID,
        );
    }
}
