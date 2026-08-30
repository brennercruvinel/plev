// UI scene construction helpers for the mobile input demo.

use engine::compositor::TextNodeKey;

use crate::palette::*;
use crate::state::MobileInputApp;

impl MobileInputApp {
    pub(crate) fn build_safe_area_bars(&mut self, w: f32, h: f32) {
        let sa = &self.safe_area;
        if sa.top > 0.0 {
            self.compositor
                .draw_rect(0.0, 0.0, w, sa.top, [0.8, 0.2, 0.2, 0.3]);
        }
        if sa.bottom > 0.0 {
            self.compositor
                .draw_rect(0.0, h - sa.bottom, w, sa.bottom, [0.2, 0.8, 0.2, 0.3]);
        }
        if sa.left > 0.0 {
            self.compositor.draw_rect(
                0.0,
                sa.top,
                sa.left,
                h - sa.top - sa.bottom,
                [0.2, 0.2, 0.8, 0.3],
            );
        }
        if sa.right > 0.0 {
            self.compositor.draw_rect(
                w - sa.right,
                sa.top,
                sa.right,
                h - sa.top - sa.bottom,
                [0.8, 0.8, 0.2, 0.3],
            );
        }
    }

    pub(crate) fn build_content(&mut self, w: f32, h: f32) {
        // Copy safe_area values to locals to avoid borrowing self immutably.
        let sa_top = self.safe_area.top;
        let sa_bottom = self.safe_area.bottom;
        let sa_left = self.safe_area.left;
        let sa_right = self.safe_area.right;

        let cx = sa_left;
        let content_w = w - sa_left - sa_right;
        let pad = 24.0;

        // Header
        let header_y = sa_top;
        let header_h = 70.0;
        self.compositor
            .draw_rect(cx, header_y, content_w, header_h, HEADER_BG);
        self.compositor.draw_text(
            TextNodeKey::from_style("MOBILE INPUT", &title_style(24.0, 30.0), None),
            cx + pad,
            header_y + 14.0,
            TEXT,
        );
        self.compositor.draw_text(
            TextNodeKey::from_style("Safe areas, IME, Lifecycle", &body_style(12.0, 16.0), None),
            cx + pad,
            header_y + 46.0,
            TEXT_DIM,
        );
        self.compositor
            .draw_rect(cx, header_y + header_h - 1.0, content_w, 1.0, DIVIDER);

        let card_x = cx + pad;
        let card_w = content_w - pad * 2.0;

        self.build_status_card(card_x, card_w, header_y + header_h + 20.0);

        let card2_y = header_y + header_h + 20.0 + 90.0 + 16.0;
        self.build_text_input_card(card_x, card_w, card2_y);

        // Instructions
        let help_y = card2_y + 110.0 + 20.0;
        self.compositor.draw_text(
            TextNodeKey::from_style(
                "On mobile: tap the text field to open keyboard.\n\
                 Safe area insets shown as colored bars at screen edges.\n\
                 On desktop: insets are zero, IME works with physical keyboard.",
                &body_style(12.0, 17.0),
                Some(card_w),
            ),
            card_x,
            help_y,
            TEXT_DIM,
        );

        // Footer
        let footer_h = 32.0;
        let footer_y = h - sa_bottom - footer_h;
        self.compositor
            .draw_rect(cx, footer_y, content_w, footer_h, FOOTER_BG);
        self.compositor
            .draw_rect(cx, footer_y, content_w, 1.0, DIVIDER);
        self.compositor.draw_text(
            TextNodeKey::from_style(
                "Safe area insets  |  IME composing  |  Lifecycle transitions",
                &body_style(11.0, 14.0),
                None,
            ),
            cx + pad,
            footer_y + 9.0,
            TEXT_DIM,
        );
    }

    fn build_status_card(&mut self, card_x: f32, card_w: f32, card1_y: f32) {
        let card1_h = 90.0;
        self.compositor
            .draw_rect(card_x, card1_y, card_w, card1_h, SURFACE);
        self.compositor
            .draw_rect(card_x, card1_y, card_w, 2.0, ACCENT);

        self.compositor.draw_text(
            TextNodeKey::from_style("STATUS", &card_title_style(11.0, 14.0), None),
            card_x + 16.0,
            card1_y + 12.0,
            ACCENT,
        );

        let lifecycle_str = format!("Lifecycle:  {}", self.lifecycle.state());
        self.compositor.draw_text(
            TextNodeKey::from_style(&lifecycle_str, &code_style(13.0, 18.0), Some(card_w - 32.0)),
            card_x + 16.0,
            card1_y + 32.0,
            TEXT,
        );

        let scale_str = format!("Scale:  {:.1}x", self.scale_factor);
        self.compositor.draw_text(
            TextNodeKey::from_style(&scale_str, &code_style(13.0, 18.0), None),
            card_x + 16.0,
            card1_y + 52.0,
            TEXT,
        );

        let kb_label = if self.ime_state.keyboard_visible() {
            "visible"
        } else {
            "hidden"
        };
        let kb_str = format!("Keyboard:  {kb_label}");
        self.compositor.draw_text(
            TextNodeKey::from_style(&kb_str, &code_style(13.0, 18.0), None),
            card_x + card_w * 0.4,
            card1_y + 52.0,
            TEXT,
        );

        let sa = &self.safe_area;
        let insets_str = format!(
            "Insets:  T{:.0}  B{:.0}  L{:.0}  R{:.0}",
            sa.top, sa.bottom, sa.left, sa.right,
        );
        self.compositor.draw_text(
            TextNodeKey::from_style(&insets_str, &code_style(13.0, 18.0), None),
            card_x + 16.0,
            card1_y + 72.0,
            TEXT_DIM,
        );
    }

    fn build_text_input_card(&mut self, card_x: f32, card_w: f32, card2_y: f32) {
        let card2_h = 110.0;
        self.compositor
            .draw_rect(card_x, card2_y, card_w, card2_h, SURFACE);
        self.compositor
            .draw_rect(card_x, card2_y, card_w, 2.0, CYAN);

        self.compositor.draw_text(
            TextNodeKey::from_style("TEXT INPUT", &card_title_style(11.0, 14.0), None),
            card_x + 16.0,
            card2_y + 12.0,
            CYAN,
        );

        let field_x = card_x + 16.0;
        let field_y = card2_y + 34.0;
        let field_w = card_w - 32.0;
        let field_h = 40.0;
        self.compositor
            .draw_rect(field_x, field_y, field_w, field_h, [0.06, 0.06, 0.10, 1.0]);
        self.compositor
            .draw_rect(field_x, field_y, field_w, 1.0, DIVIDER);
        self.compositor
            .draw_rect(field_x, field_y + field_h - 1.0, field_w, 1.0, DIVIDER);
        self.compositor
            .draw_rect(field_x, field_y, 1.0, field_h, DIVIDER);
        self.compositor
            .draw_rect(field_x + field_w - 1.0, field_y, 1.0, field_h, DIVIDER);

        let display_text = if self.input_text.is_empty() && self.ime_state.preedit_text.is_empty() {
            "Type here (IME input)...".to_string()
        } else {
            let mut t = self.input_text.clone();
            if !self.ime_state.preedit_text.is_empty() {
                t.push_str(&self.ime_state.preedit_text);
            }
            t
        };

        let text_color = if self.input_text.is_empty() && self.ime_state.preedit_text.is_empty() {
            TEXT_DIM
        } else {
            TEXT
        };

        self.compositor.draw_text(
            TextNodeKey::from_style(&display_text, &body_style(14.0, 20.0), Some(field_w - 16.0)),
            field_x + 8.0,
            field_y + 10.0,
            text_color,
        );

        self.compositor.draw_text(
            TextNodeKey::from_style(
                "IME preedit composing is shown inline",
                &body_style(11.0, 14.0),
                None,
            ),
            card_x + 16.0,
            card2_y + 84.0,
            TEXT_DIM,
        );
    }
}
