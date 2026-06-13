//! Scene building and input handling for the Todo App example.

use plev::compositor::{SceneNode, TextNodeKey};
use plev::winit::event::ElementState;
use plev::winit::keyboard::{Key, NamedKey};

use crate::state::*;

// ---------------------------------------------------------------------------
// Hover & click handling
// ---------------------------------------------------------------------------

impl TodoApp {
    pub(crate) fn update_hover(&mut self) {
        let (mx, my) = self.cursor_pos;
        self.hover_item_id = None;
        self.hover_delete_id = None;

        let visible_ids: Vec<_> = self.visible_items().iter().map(|i| i.id).collect();
        for (vi, id) in visible_ids.iter().enumerate() {
            let (ix, iy, iw, ih) = self.item_rect(vi);
            if mx >= ix && mx <= ix + iw && my >= iy && my <= iy + ih {
                self.hover_item_id = Some(*id);
                let (dx, dy, dw, dh) = self.delete_rect(vi);
                if mx >= dx && mx <= dx + dw && my >= dy && my <= dy + dh {
                    self.hover_delete_id = Some(*id);
                }
            }
        }
    }

    pub(crate) fn handle_click(&mut self) {
        let (mx, my) = self.cursor_pos;

        // Check input area click
        let (ix, iy, iw, ih) = self.input_rect();
        if mx >= ix && mx <= ix + iw && my >= iy && my <= iy + ih {
            let local_x = mx - ix - 8.0;
            self.input.handle_click(local_x.max(0.0));
            return;
        }

        // Check item clicks
        let visible: Vec<_> = self.visible_items().iter().map(|i| i.id).collect();
        for (vi, &item_id) in visible.iter().enumerate() {
            let (dx, dy, dw, dh) = self.delete_rect(vi);
            if mx >= dx && mx <= dx + dw && my >= dy && my <= dy + dh {
                self.remove_todo(item_id);
                return;
            }
            let (cx_r, cy_r, cw, ch) = self.checkbox_rect(vi);
            if mx >= cx_r && mx <= cx_r + cw && my >= cy_r && my <= cy_r + ch {
                self.toggle_todo(item_id);
                return;
            }
            let (rx, ry, rw, rh) = self.item_rect(vi);
            if mx >= rx && mx <= rx + rw && my >= ry && my <= ry + rh {
                self.toggle_todo(item_id);
                return;
            }
        }

        // Check filter buttons
        for (f, fx, fy, fw, fh) in self.filter_rects() {
            if mx >= fx && mx <= fx + fw && my >= fy && my <= fy + fh {
                self.filter = f;
                return;
            }
        }
    }

    pub(crate) fn handle_key_event(&mut self, event: &plev::winit::event::KeyEvent) {
        if event.state != ElementState::Pressed {
            return;
        }

        match &event.logical_key {
            Key::Named(NamedKey::Enter) => {
                self.add_todo();
            }
            Key::Named(NamedKey::Escape) => {
                self.input.buffer.set_text("");
                self.input.reset_blink();
            }
            Key::Named(NamedKey::Backspace) => {
                self.input.handle_backspace();
            }
            Key::Named(NamedKey::Delete) => {
                self.input.handle_delete();
            }
            Key::Named(NamedKey::ArrowLeft) => {
                self.input.handle_left();
            }
            Key::Named(NamedKey::ArrowRight) => {
                self.input.handle_right();
            }
            Key::Named(NamedKey::Home) => {
                self.input.handle_home();
            }
            Key::Named(NamedKey::End) => {
                self.input.handle_end();
            }
            Key::Character(c) => {
                for ch in c.chars() {
                    if !ch.is_control() {
                        self.input.handle_char(ch);
                    }
                }
            }
            _ => {}
        }
    }

    pub(crate) fn render(&mut self) {
        let tick = self.clock.tick();
        let dt = tick.dt;

        self.input.tick(dt);
        for item in &mut self.items {
            item.tick(dt);
        }

        let (vw, vh) = match &self.state {
            GpuState::Ready { gpu, .. } => (
                gpu.surface_config.width as f32,
                gpu.surface_config.height as f32,
            ),
            _ => return,
        };

        let cx = self.content_x();
        let cw = self.content_width();
        self.compositor.begin_frame();

        // --- Header ---
        self.compositor.push(SceneNode::Rect {
            x: 0.0,
            y: 0.0,
            w: vw,
            h: HEADER_H,
            color: HEADER_BG,
        });
        self.compositor.push(SceneNode::Rect {
            x: 0.0,
            y: HEADER_H - 1.0,
            w: vw,
            h: 1.0,
            color: DIVIDER,
        });
        self.compositor.draw_text(
            TextNodeKey::from_style("plev todos", &style(28.0, 36.0), Some(cw)),
            cx,
            16.0,
            ACCENT,
        );
        self.compositor.draw_text(
            TextNodeKey::from_style("Proof of Life Demo", &style(12.0, 16.0), Some(cw)),
            cx,
            50.0,
            TEXT_DIM,
        );

        // --- Input field ---
        self.compositor.push(SceneNode::Rect {
            x: cx,
            y: HEADER_H + 14.0,
            w: cw,
            h: 2.0,
            color: ACCENT,
        });
        let input_nodes = self.input.build_scene(cx, HEADER_H + 16.0, cw);
        for node in input_nodes {
            self.compositor.push(node);
        }

        // --- Item list ---
        self.build_item_list(cx, cw, vw, vh);

        // --- GPU rendering ---
        self.submit_gpu_frame(vw, vh);
    }

    fn build_item_list(&mut self, cx: f32, cw: f32, vw: f32, vh: f32) {
        let visible = self.visible_items();
        let visible_data: Vec<_> = visible
            .iter()
            .map(|i| (i.id, i.text.clone(), i.completed, i.effective_opacity()))
            .collect();

        for (vi, (id, text, completed, opacity)) in visible_data.iter().enumerate() {
            let opacity = *opacity;
            let (ix, iy, iw, ih) = self.item_rect(vi);

            let bg = if self.hover_item_id == Some(*id) {
                SURFACE_HOVER
            } else {
                SURFACE
            };
            let bg_with_opacity = [bg[0], bg[1], bg[2], bg[3] * opacity];
            self.compositor.push(SceneNode::Rect {
                x: ix,
                y: iy,
                w: iw,
                h: ih,
                color: bg_with_opacity,
            });

            // Checkbox
            let (cbx, cby, cbw, cbh) = self.checkbox_rect(vi);
            if *completed {
                self.compositor.push(SceneNode::Rect {
                    x: cbx,
                    y: cby,
                    w: cbw,
                    h: cbh,
                    color: [
                        CHECKBOX_FILL[0],
                        CHECKBOX_FILL[1],
                        CHECKBOX_FILL[2],
                        CHECKBOX_FILL[3] * opacity,
                    ],
                });
                self.compositor.push(SceneNode::Rect {
                    x: cbx + 4.0,
                    y: cby + cbh / 2.0,
                    w: 5.0,
                    h: 2.0,
                    color: [TEXT[0], TEXT[1], TEXT[2], opacity],
                });
                self.compositor.push(SceneNode::Rect {
                    x: cbx + 8.0,
                    y: cby + 4.0,
                    w: 2.0,
                    h: cbh / 2.0 + 2.0,
                    color: [TEXT[0], TEXT[1], TEXT[2], opacity],
                });
            } else {
                let bc = [
                    CHECKBOX_BORDER[0],
                    CHECKBOX_BORDER[1],
                    CHECKBOX_BORDER[2],
                    opacity,
                ];
                self.compositor.push(SceneNode::Rect {
                    x: cbx,
                    y: cby,
                    w: cbw,
                    h: 1.5,
                    color: bc,
                });
                self.compositor.push(SceneNode::Rect {
                    x: cbx,
                    y: cby + cbh - 1.5,
                    w: cbw,
                    h: 1.5,
                    color: bc,
                });
                self.compositor.push(SceneNode::Rect {
                    x: cbx,
                    y: cby,
                    w: 1.5,
                    h: cbh,
                    color: bc,
                });
                self.compositor.push(SceneNode::Rect {
                    x: cbx + cbw - 1.5,
                    y: cby,
                    w: 1.5,
                    h: cbh,
                    color: bc,
                });
            }

            // Todo text
            let text_color = if *completed {
                [
                    TEXT_COMPLETED[0],
                    TEXT_COMPLETED[1],
                    TEXT_COMPLETED[2],
                    TEXT_COMPLETED[3] * opacity,
                ]
            } else {
                [TEXT[0], TEXT[1], TEXT[2], TEXT[3] * opacity]
            };
            let text_x = cbx + cbw + 12.0;
            let text_w = iw - (text_x - ix) - DELETE_SIZE - 24.0;
            self.compositor.draw_text(
                TextNodeKey::from_style(text, &style(15.0, 20.0), Some(text_w)),
                text_x,
                iy + (ih - 15.0) / 2.0,
                text_color,
            );

            // Strikethrough
            if *completed {
                let strike_w = text.len().min(40) as f32 * 9.0;
                self.compositor.push(SceneNode::Rect {
                    x: text_x,
                    y: iy + ih / 2.0,
                    w: strike_w.min(text_w),
                    h: 1.0,
                    color: [
                        TEXT_COMPLETED[0],
                        TEXT_COMPLETED[1],
                        TEXT_COMPLETED[2],
                        0.5 * opacity,
                    ],
                });
            }

            // Delete button
            let (dx, dy, dw, dh) = self.delete_rect(vi);
            let del_color = if self.hover_delete_id == Some(*id) {
                RED_HOVER
            } else {
                RED
            };
            let del_color = [
                del_color[0],
                del_color[1],
                del_color[2],
                del_color[3] * opacity,
            ];
            let cx_del = dx + dw / 2.0;
            let cy_del = dy + dh / 2.0;
            self.compositor.push(SceneNode::Rect {
                x: cx_del - 6.0,
                y: cy_del - 1.0,
                w: 12.0,
                h: 2.0,
                color: del_color,
            });
            self.compositor.push(SceneNode::Rect {
                x: cx_del - 1.0,
                y: cy_del - 6.0,
                w: 2.0,
                h: 12.0,
                color: del_color,
            });
        }

        // --- Empty state ---
        if visible_data.is_empty() {
            let ey = HEADER_H + 16.0 + INPUT_H + 40.0;
            let msg = match self.filter {
                Filter::All => "No todos yet. Type something and press Enter!",
                Filter::Active => "No active todos. Everything is done!",
                Filter::Completed => "No completed todos yet.",
            };
            self.compositor.draw_text(
                TextNodeKey::from_style(msg, &style(14.0, 20.0), Some(cw)),
                cx + 12.0,
                ey,
                TEXT_DIM,
            );
        }

        // --- Footer ---
        self.build_footer(cx, cw, vw, vh);
    }

    fn build_footer(&mut self, cx: f32, cw: f32, vw: f32, vh: f32) {
        let fy = self.footer_y();
        self.compositor.push(SceneNode::Rect {
            x: cx,
            y: fy - 4.0,
            w: cw,
            h: 1.0,
            color: DIVIDER,
        });

        let active = self.active_count();
        let counter_text = if active == 1 {
            "1 item left".to_string()
        } else {
            format!("{} items left", active)
        };
        self.compositor.draw_text(
            TextNodeKey::from_style(&counter_text, &style(13.0, 17.0), Some(200.0)),
            cx,
            fy + 4.0,
            TEXT_DIM,
        );

        for (f, fx, ffy, fw, fh) in self.filter_rects() {
            if f == self.filter {
                self.compositor.push(SceneNode::Rect {
                    x: fx,
                    y: ffy,
                    w: fw,
                    h: fh,
                    color: FILTER_ACTIVE_BG,
                });
            }
            let color = if f == self.filter { TEXT } else { TEXT_DIM };
            self.compositor.draw_text(
                TextNodeKey::from_style(f.label(), &style(12.0, 16.0), Some(fw - 10.0)),
                fx + 10.0,
                ffy + 6.0,
                color,
            );
        }

        // Bottom footer bar
        self.compositor.push(SceneNode::Rect {
            x: 0.0,
            y: vh - 32.0,
            w: vw,
            h: 1.0,
            color: DIVIDER,
        });
        self.compositor.push(SceneNode::Rect {
            x: 0.0,
            y: vh - 31.0,
            w: vw,
            h: 31.0,
            color: [0.07, 0.07, 0.12, 1.0],
        });
        self.compositor.draw_text(
            TextNodeKey::from_style(
                "TextInput + Tween<f32>  |  Filter state  |  Hit region per item",
                &style(11.0, 15.0),
                Some(vw - 64.0),
            ),
            32.0,
            vh - 22.0,
            TEXT_DIM,
        );
    }
}

fn style(size: f32, line_height: f32) -> plev::text::TextStyle {
    plev::text::TextStyle::new(size).with_line_height(line_height)
}
