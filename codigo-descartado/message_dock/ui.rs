//! Scene building, event processing, and animation updates for the dock.

use plev::compositor::{Compositor, TextNodeKey};
use plev::input::{InputEvent, InputState, PressState};

use crate::state::*;

impl AnimatedDock {
    pub(crate) fn process_events(&mut self, input: &mut InputState) {
        for event in input.drain_events() {
            match event {
                InputEvent::Click(click) => {
                    if click.state == PressState::Pressed {
                        for i in 1..NUM_CHARS {
                            if let Some(id) = self.char_view_ids[i] {
                                if click.view_id == id {
                                    if self.selected == Some(i) {
                                        self.selected = None;
                                    } else {
                                        self.selected = Some(i);
                                    }
                                }
                            }
                        }
                        if let Some(send_id) = self.send_btn_id {
                            if click.view_id == send_id && self.selected.is_some() {
                                self.sent_flash_timer = 1.0;
                                self.selected = None;
                            }
                        }
                    }
                }
                InputEvent::Hover(hover) => {
                    let mut found = false;
                    for i in 0..NUM_CHARS {
                        if let Some(id) = self.char_view_ids[i] {
                            if hover.view_id == id {
                                self.hovered_char = if hover.entered { Some(i) } else { None };
                                found = true;
                            }
                        }
                    }
                    if !found {
                        if let Some(send_id) = self.send_btn_id {
                            if hover.view_id == send_id {
                                self.send_hovered = hover.entered;
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    pub(crate) fn update_animations(&mut self) {
        let is_expanded = self.selected.is_some();
        let pad = 10.0;
        let asz = self.avatar_size;
        let gap = self.avatar_gap;
        let chars_start = pad + asz + 16.0;

        let target_w = if is_expanded { self.expanded_width } else { self.collapsed_width };
        self.dock_width = smooth(self.dock_width, target_w, 0.10);

        let target_bg = if let Some(sel) = self.selected {
            let a = CHARACTERS[sel].accent_color;
            lerp_color(DOCK_BG, [a[0], a[1], a[2], 0.95], 0.5)
        } else {
            DOCK_BG
        };
        self.dock_bg_color = smooth_color(self.dock_bg_color, target_bg, 0.10);

        for i in 0..NUM_CHARS {
            let is_hovered = self.hovered_char == Some(i) && !is_expanded;
            let is_selected = self.selected == Some(i);

            let target_x = if is_expanded && is_selected {
                pad
            } else if i == 0 {
                pad
            } else {
                chars_start + (i - 1) as f32 * (asz + gap)
            };
            self.char_x[i] = smooth(self.char_x[i], target_x, 0.12);

            let target_y = if is_hovered { -8.0 } else { 0.0 };
            self.char_y_offsets[i] = smooth(self.char_y_offsets[i], target_y, 0.15);

            let target_o = if is_expanded && !is_selected { 0.0 }
                else if is_expanded && i == 0 { 0.0 }
                else { 1.0 };
            self.char_opacities[i] = smooth_n(self.char_opacities[i], target_o, 0.12);
        }

        let target_input = if is_expanded { 1.0 } else { 0.0 };
        self.input_opacity = smooth_n(self.input_opacity, target_input, 0.10);
        self.send_btn_opacity = smooth_n(self.send_btn_opacity, target_input, 0.10);

        let target_sep = if is_expanded { 0.0 } else { 1.0 };
        self.separator_opacity = smooth_n(self.separator_opacity, target_sep, 0.15);

        if self.sent_flash_timer > 0.01 {
            self.sent_flash_timer *= 0.93;
            if self.sent_flash_timer < 0.01 { self.sent_flash_timer = 0.0; }
        }
    }

    pub(crate) fn build_scene(
        &mut self,
        comp: &mut Compositor,
        input: &mut InputState,
        vw: f32,
        vh: f32,
        frame: u64,
    ) {
        self.update_animations();

        let dock_x = px((vw - self.dock_width) / 2.0);
        let dock_y = px(vh - 80.0 - self.dock_height);
        let dw = px(self.dock_width);
        let dh = self.dock_height;
        let asz = self.avatar_size;
        let pad = 10.0;

        // Dock shadow + border + background
        draw_rect(comp, dock_x - 6.0, dock_y - 2.0, dw + 12.0, dh + 12.0, [0.0, 0.0, 0.0, 0.20]);
        draw_rect(comp, dock_x - 3.0, dock_y, dw + 6.0, dh + 6.0, [0.0, 0.0, 0.0, 0.12]);
        draw_rect(comp, dock_x - 1.0, dock_y - 1.0, dw + 2.0, dh + 2.0, DOCK_BORDER);

        let bg = if self.sent_flash_timer > 0.01 {
            lerp_color(self.dock_bg_color, SENT_FLASH, self.sent_flash_timer * 0.4)
        } else {
            self.dock_bg_color
        };
        draw_rect(comp, dock_x, dock_y, dw, dh, bg);

        let is_expanded = self.selected.is_some();

        // Characters
        self.build_characters(comp, input, dock_x, dock_y, dw, dh, asz, bg);

        // Separators
        if self.separator_opacity > 0.01 {
            let sx = px(dock_x + pad + asz + 6.0);
            draw_rect(comp, sx, px(dock_y + dh * 0.2), 1.0, px(dh * 0.6),
                [0.35, 0.35, 0.45, 0.4 * self.separator_opacity]);
            let sx2 = px(dock_x + dw - pad - asz - 10.0);
            draw_rect(comp, sx2, px(dock_y + dh * 0.2), 1.0, px(dh * 0.6),
                [0.35, 0.35, 0.45, 0.4 * self.separator_opacity]);
        }

        // Expanded: placeholder text with blinking cursor
        if self.input_opacity > 0.03 {
            let name = self.selected.map(|i| CHARACTERS[i].name).unwrap_or("...");
            let dots = match ((frame / 20) % 4) as usize {
                0 => ".", 1 => "..", 2 => "...", _ => "",
            };
            let placeholder = format!("Message {}{}", name, dots);
            let text_x = px(dock_x + pad + asz + 16.0);
            let text_y = px(dock_y + dh * 0.32);
            let max_w = (dw - pad * 2.0 - asz - 60.0).max(10.0);
            comp.draw_text(
                TextNodeKey::new(&placeholder, 14.0, 20.0, Some(max_w)), text_x, text_y,
                [TEXT_PLACEHOLDER[0], TEXT_PLACEHOLDER[1], TEXT_PLACEHOLDER[2],
                 TEXT_PLACEHOLDER[3] * self.input_opacity],
            );
            if (frame / 30) % 2 == 0 && self.input_opacity > 0.5 {
                draw_rect(comp, text_x, text_y - 1.0, 2.0, 16.0,
                    [TEXT_PRIMARY[0], TEXT_PRIMARY[1], TEXT_PRIMARY[2], 0.7 * self.input_opacity]);
            }
        }

        // Send button / Menu bars
        self.build_send_button(comp, input, dock_x, dock_y, dw, dh, asz, pad, is_expanded);

        // Sent feedback
        if self.sent_flash_timer > 0.03 {
            comp.draw_text(
                TextNodeKey::new("Sent!", 15.0, 20.0, None),
                px((vw - 40.0) / 2.0), px(dock_y - 32.0),
                [SENT_FLASH[0], SENT_FLASH[1], SENT_FLASH[2], self.sent_flash_timer],
            );
        }

        // Title & decoration
        self.build_chrome(comp, vw, vh, frame);
    }

    fn build_characters(
        &mut self, comp: &mut Compositor, input: &mut InputState,
        dock_x: f32, dock_y: f32, _dw: f32, dh: f32, asz: f32, bg: [f32; 4],
    ) {
        for i in 0..NUM_CHARS {
            let opacity = self.char_opacities[i];
            if opacity < 0.01 { self.char_view_ids[i] = None; continue; }
            let ch = &CHARACTERS[i];
            let cx = px(dock_x + self.char_x[i]);
            let cy = px(dock_y + (dh - asz) / 2.0 + self.char_y_offsets[i]);
            draw_rect(comp, cx, cy, asz, asz,
                [ch.bg_color[0], ch.bg_color[1], ch.bg_color[2], ch.bg_color[3] * opacity]);
            comp.draw_text(
                TextNodeKey::new(ch.initial, 18.0, 22.0, None),
                px(cx + asz * 0.28), px(cy + asz * 0.22), [1.0, 1.0, 1.0, opacity],
            );
            if ch.online && opacity > 0.3 {
                let dot = 8.0;
                let dx = px(cx + asz - dot + 1.0);
                let dy = px(cy + asz - dot + 1.0);
                draw_rect(comp, dx - 2.0, dy - 2.0, dot + 4.0, dot + 4.0,
                    [bg[0], bg[1], bg[2], opacity]);
                draw_rect(comp, dx, dy, dot, dot,
                    [ONLINE_GREEN[0], ONLINE_GREEN[1], ONLINE_GREEN[2], opacity]);
            }
            let id = input.next_view_id();
            input.register_hit_region(id, cx, cy - 4.0, asz, asz + 8.0, false);
            self.char_view_ids[i] = Some(id);
        }
    }

    fn build_send_button(
        &mut self, comp: &mut Compositor, input: &mut InputState,
        dock_x: f32, dock_y: f32, dw: f32, dh: f32, asz: f32, pad: f32, is_expanded: bool,
    ) {
        let btn_x = px(dock_x + dw - pad - asz);
        let btn_y = px(dock_y + (dh - asz) / 2.0);
        if is_expanded && self.send_btn_opacity > 0.03 {
            let o = self.send_btn_opacity;
            let btn_color = if self.send_hovered { SEND_BTN_HOVER } else { SEND_BTN };
            draw_rect(comp, btn_x, btn_y, asz, asz,
                [btn_color[0], btn_color[1], btn_color[2], btn_color[3] * o]);
            comp.draw_text(
                TextNodeKey::new(">", 18.0, 22.0, None),
                px(btn_x + asz * 0.28), px(btn_y + asz * 0.22), [1.0, 1.0, 1.0, o],
            );
            let id = input.next_view_id();
            input.register_hit_region(id, btn_x, btn_y, asz, asz, false);
            self.send_btn_id = Some(id);
        } else if !is_expanded {
            let bar_w = 18.0;
            let bar_h = 2.0;
            let bar_gap = 5.0;
            let ix = px(btn_x + (asz - bar_w) / 2.0);
            let start_y = px(btn_y + (asz - bar_h * 3.0 - bar_gap * 2.0) / 2.0);
            let menu_opacity = 1.0 - self.send_btn_opacity;
            for j in 0..3 {
                draw_rect(comp, ix, px(start_y + j as f32 * (bar_h + bar_gap)),
                    bar_w, bar_h, [0.55, 0.55, 0.65, 0.8 * menu_opacity]);
            }
            self.send_btn_id = None;
        }
    }

    fn build_chrome(&self, comp: &mut Compositor, vw: f32, vh: f32, frame: u64) {
        comp.draw_text(
            TextNodeKey::new("plev ENGINE", 36.0, 44.0, Some(vw - 80.0)),
            40.0, 40.0, TEXT_PRIMARY,
        );
        comp.draw_text(
            TextNodeKey::new("GPU-first compositing  //  wgpu 28  //  6 platforms", 14.0, 20.0, Some(vw - 80.0)),
            40.0, 88.0, TEXT_DIM,
        );
        draw_rect(comp, 40.0, 118.0, vw - 80.0, 1.0, [0.20, 0.20, 0.28, 0.5]);

        // Feature cards
        let card_y = 148.0;
        let card_h = (vh - 320.0).clamp(120.0, 200.0);
        let card_w = ((vw - 120.0) / 3.0).max(100.0);
        let features: [(&str, &str, [f32; 4]); 3] = [
            ("Animated Dock", "Frame-based lerp interpolation\nSmooth exponential easing\nPixel-snapped rendering", [0.30, 0.55, 1.0, 1.0]),
            ("Hit Testing", "Per-view hover detection\nClick events with ViewId\nLayer-aware regions", [0.20, 0.80, 0.45, 1.0]),
            ("GPU Rendering", "wgpu compositor pipeline\nDirty tracking per layer\nPremultiplied alpha blend", [0.65, 0.40, 0.90, 1.0]),
        ];
        for (j, (title, desc, accent)) in features.iter().enumerate() {
            let cx = px(40.0 + j as f32 * (card_w + 20.0));
            let cy = card_y;
            draw_rect(comp, cx, cy, card_w, card_h, [0.10, 0.10, 0.16, 1.0]);
            draw_rect(comp, cx, cy, card_w, 3.0, *accent);
            comp.draw_text(
                TextNodeKey::new(title, 16.0, 22.0, Some(card_w - 24.0)),
                cx + 12.0, cy + 16.0, *accent,
            );
            comp.draw_text(
                TextNodeKey::new(desc, 12.0, 18.0, Some(card_w - 24.0)),
                cx + 12.0, cy + 46.0, TEXT_DIM,
            );
        }

        comp.draw_text(
            TextNodeKey::new("Hover and click the characters in the dock below", 13.0, 18.0, Some(vw - 80.0)),
            40.0, px(vh - 152.0), [0.45, 0.45, 0.55, 0.8],
        );

        let frame_text = format!("Frame {}", frame);
        comp.draw_text(
            TextNodeKey::new(&frame_text, 11.0, 16.0, None),
            vw - 100.0, vh - 20.0, [0.35, 0.35, 0.45, 1.0],
        );
    }
}
