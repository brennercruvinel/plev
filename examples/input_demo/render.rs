// Rendering: scene construction and GPU submission for the input demo.

use plev::compositor::{Compositor, TextNodeKey};
use plev::text::TextMeasurer;

use crate::palette::*;
use crate::state::{InputDemoApp, State};

impl InputDemoApp {
    pub fn render(&mut self) {
        self.input_state.begin_frame();
        self.process_events();
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

        let view = gpu.surface_render_view(&output);

        let w = gpu.surface_config.width as f32;
        let h = gpu.surface_config.height as f32;

        text_system.begin_frame();

        let margin = 32.0;
        let header_h = 70.0;
        let footer_h = 32.0;
        let content_y = header_h + 20.0;
        let content_w = w - margin * 2.0;
        let card_h = h - content_y - footer_h - 20.0;
        let card_gap = 16.0;

        let compositor = &mut self.compositor;

        // Header
        compositor.draw_rect(0.0, 0.0, w, header_h, HEADER_BG);
        compositor.draw_text(
            TextNodeKey::from_style(
                "INPUT SYSTEM",
                &title_style(24.0, 30.0),
                Some(w - margin * 2.0),
            ),
            margin,
            16.0,
            TEXT,
        );
        compositor.draw_text(
            TextNodeKey::from_style(
                "Hit regions, hover states, event queue",
                &body_style(12.0, 16.0),
                Some(w - margin * 2.0),
            ),
            margin,
            48.0,
            TEXT_DIM,
        );
        compositor.draw_rect(0.0, header_h - 1.0, w, 1.0, DIVIDER);

        let btn_x;
        let btn_y;
        let btn_w;
        let btn_h;
        {
            let left_w = (content_w - card_gap) * 0.55;
            let left_x = margin;
            btn_w = 180.0;
            btn_h = 44.0;
            btn_x = left_x + (left_w - btn_w) / 2.0;
            btn_y = content_y + 52.0 + 24.0;
            Self::build_button_card(
                compositor,
                left_x,
                content_y,
                left_w,
                card_h,
                btn_x,
                btn_y,
                btn_w,
                btn_h,
                self.button_hovered,
                self.click_count,
            );
        }

        {
            let left_w = (content_w - card_gap) * 0.55;
            let right_x = margin + left_w + card_gap;
            let right_w = content_w - left_w - card_gap;
            Self::build_events_card(compositor, right_x, content_y, right_w, card_h);
        }

        // Footer
        let footer_y = h - footer_h;
        compositor.draw_rect(0.0, footer_y - 1.0, w, 1.0, DIVIDER);
        compositor.draw_rect(0.0, footer_y, w, footer_h, FOOTER_BG);
        compositor.draw_text(
            TextNodeKey::from_style(
                "InputState  |  ViewId hit testing  |  Gesture recognizer",
                &body_style(11.0, 14.0),
                Some(w - margin * 2.0),
            ),
            margin,
            footer_y + 9.0,
            TEXT_DIM,
        );

        // Register hit regions
        let btn_id = self.input_state.next_view_id();
        self.input_state
            .register_hit_region(btn_id, btn_x, btn_y, btn_w, btn_h, true);
        self.button_view_id = Some(btn_id);

        // GPU resolve and submit
        crate::gpu::gpu_resolve_and_submit(compositor, gpu, text_system, &view);
        output.present();
    }

    // Demo helper; the args are the already-derived card/button geometry
    // and a bag struct would just be repacked at the single call site.
    #[allow(clippy::too_many_arguments)]
    fn build_button_card(
        compositor: &mut Compositor,
        left_x: f32,
        content_y: f32,
        left_w: f32,
        card_h: f32,
        btn_x: f32,
        btn_y: f32,
        btn_w: f32,
        btn_h: f32,
        button_hovered: bool,
        click_count: u32,
    ) {
        card(compositor, left_x, content_y, left_w, card_h, ACCENT);

        compositor.draw_text(
            TextNodeKey::from_style(
                "INTERACTIVE BUTTON",
                &card_title_style(11.0, 14.0),
                Some(left_w - 32.0),
            ),
            left_x + 16.0,
            content_y + 16.0,
            ACCENT,
        );
        compositor.draw_rect(left_x + 16.0, content_y + 36.0, left_w - 32.0, 1.0, DIVIDER);

        let (btn_bg, btn_label_color) = if button_hovered {
            (BTN_HOVER, TEXT)
        } else {
            (BTN_NORMAL, TEXT_MID)
        };

        compositor.draw_rect(
            btn_x - 1.0,
            btn_y - 1.0,
            btn_w + 2.0,
            btn_h + 2.0,
            BTN_BORDER,
        );
        compositor.draw_rect(btn_x, btn_y, btn_w, btn_h, btn_bg);
        // Center the label from its measured size: the same style measures
        // and draws, so the text cannot drift off-center.
        let btn_style = label_style(16.0, 20.0);
        let (btn_tw, btn_th) = TextMeasurer::measure_styled("Click me!", &btn_style, None);
        compositor.draw_text(
            TextNodeKey::from_style("Click me!", &btn_style, Some(btn_w - 20.0)),
            btn_x + (btn_w - btn_tw) / 2.0,
            btn_y + (btn_h - btn_th) / 2.0,
            btn_label_color,
        );

        let counter_y = btn_y + btn_h + 24.0;
        let counter_w = left_w - 48.0;
        let counter_x = left_x + 24.0;
        compositor.draw_rect(
            counter_x,
            counter_y,
            counter_w,
            48.0,
            [0.08, 0.08, 0.13, 1.0],
        );
        compositor.draw_text(
            TextNodeKey::from_style(
                "CLICK COUNT",
                &label_style(11.0, 14.0),
                Some(counter_w - 24.0),
            ),
            counter_x + 12.0,
            counter_y + 8.0,
            TEXT_DIM,
        );
        let count_str = format!("{}", click_count);
        compositor.draw_text(
            TextNodeKey::from_style(
                &count_str,
                &code_style(20.0, 26.0).with_weight(700),
                Some(counter_w - 24.0),
            ),
            counter_x + 12.0,
            counter_y + 24.0,
            ACCENT,
        );

        let hint_text = if button_hovered {
            "State: HOVERED"
        } else {
            "State: idle"
        };
        let hint_color = if button_hovered { ACCENT } else { TEXT_DIM };
        compositor.draw_text(
            TextNodeKey::from_style(hint_text, &body_style(11.0, 14.0), Some(left_w - 32.0)),
            left_x + 16.0,
            counter_y + 60.0,
            hint_color,
        );
    }

    fn build_events_card(
        compositor: &mut Compositor,
        right_x: f32,
        content_y: f32,
        right_w: f32,
        card_h: f32,
    ) {
        card(compositor, right_x, content_y, right_w, card_h, CYAN);

        compositor.draw_text(
            TextNodeKey::from_style(
                "EVENT TYPES",
                &card_title_style(11.0, 14.0),
                Some(right_w - 32.0),
            ),
            right_x + 16.0,
            content_y + 16.0,
            CYAN,
        );
        compositor.draw_rect(
            right_x + 16.0,
            content_y + 36.0,
            right_w - 32.0,
            1.0,
            DIVIDER,
        );

        let ev_x = right_x + 16.0;
        let ev_max_w = right_w - 32.0;
        let mut ey = content_y + 52.0;

        let events_list = [
            ("Click", "MouseInput -> PressState -> ViewId lookup"),
            ("Hover", "CursorMoved -> enter/leave detection per region"),
            ("Scroll", "MouseWheel delta -> dispatched to hovered view"),
            ("Keyboard", "KeyboardInput -> key + state + modifiers"),
        ];

        for (name, desc) in &events_list {
            compositor.draw_rect(ev_x, ey + 4.0, 4.0, 4.0, CYAN);
            compositor.draw_text(
                TextNodeKey::from_style(name, &label_style(14.0, 20.0), Some(ev_max_w - 16.0)),
                ev_x + 12.0,
                ey,
                TEXT,
            );
            ey += 20.0;
            compositor.draw_text(
                TextNodeKey::from_style(desc, &code_style(11.0, 14.0), Some(ev_max_w - 16.0)),
                ev_x + 12.0,
                ey,
                TEXT_DIM,
            );
            ey += 24.0;
        }

        compositor.draw_rect(right_x + 16.0, ey + 8.0, right_w - 32.0, 1.0, DIVIDER);
        compositor.draw_text(
            TextNodeKey::from_style(
                "Hit regions: register_hit_region(id, x, y, w, h)",
                &code_style(11.0, 14.0),
                Some(ev_max_w),
            ),
            ev_x,
            ey + 18.0,
            ACCENT_DIM,
        );
    }
}
