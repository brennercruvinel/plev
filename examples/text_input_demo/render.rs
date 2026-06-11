// Rendering: scene graph construction and GPU submission.

use plev::compositor::{SceneNode, TextNodeKey};

use crate::palette::*;
use crate::state::{GpuState, TextInputApp};

impl TextInputApp {
    pub fn render(&mut self) {
        let tick = self.clock.tick();

        for input in &mut self.inputs {
            input.tick(tick.dt);
        }

        let (vw, vh) = match &self.state {
            GpuState::Ready { gpu, .. } => (
                gpu.surface_config.width as f32,
                gpu.surface_config.height as f32,
            ),
            _ => return,
        };

        self.compositor.begin_frame();

        // Header
        self.compositor.push(SceneNode::Rect {
            x: 0.0,
            y: 0.0,
            w: vw,
            h: 70.0,
            color: HEADER_BG,
        });
        self.compositor.push(SceneNode::Rect {
            x: 0.0,
            y: 69.0,
            w: vw,
            h: 1.0,
            color: DIVIDER,
        });
        self.compositor.draw_text(
            TextNodeKey::new("TEXT INPUT DEMO", 24.0, 30.0, Some(vw - 64.0)),
            32.0,
            16.0,
            TEXT,
        );
        self.compositor.draw_text(
            TextNodeKey::new(
                "Tab to cycle, Escape to unfocus, click to position cursor",
                12.0,
                16.0,
                Some(vw - 64.0),
            ),
            32.0,
            46.0,
            TEXT_DIM,
        );

        let input_w = (vw - 64.0).min(500.0);
        let input_x = (vw - input_w) / 2.0;
        let start_y = 106.0;
        let card_h = 62.0;
        let spacing = card_h + 16.0;

        let labels = ["Name", "Email", "Notes"];

        for (i, input) in self.inputs.iter().enumerate() {
            let card_y = start_y + i as f32 * spacing;
            let field_y = card_y + 26.0;

            self.compositor.push(SceneNode::Rect {
                x: input_x,
                y: card_y,
                w: input_w,
                h: 2.0,
                color: ACCENT_DIM,
            });
            self.compositor.push(SceneNode::Rect {
                x: input_x,
                y: card_y + 2.0,
                w: input_w,
                h: card_h - 2.0,
                color: SURFACE,
            });

            self.compositor.draw_text(
                TextNodeKey::new(labels[i], 11.0, 15.0, Some(input_w - 20.0)),
                input_x + 10.0,
                card_y + 8.0,
                LABEL_COLOR,
            );

            let nodes = input.build_scene(input_x + 8.0, field_y, input_w - 16.0);
            for node in nodes {
                self.compositor.push(node);
            }
        }

        self.build_preview(input_x, start_y, input_w, spacing, &labels);
        self.build_footer(vw, vh);
        self.gpu_submit();
    }

    fn build_preview(
        &mut self,
        input_x: f32,
        start_y: f32,
        input_w: f32,
        spacing: f32,
        labels: &[&str; 3],
    ) {
        let preview_x = input_x;
        let preview_y = start_y + self.inputs.len() as f32 * spacing + 8.0;
        let preview_w = input_w;

        self.compositor.push(SceneNode::Rect {
            x: preview_x,
            y: preview_y,
            w: preview_w,
            h: 2.0,
            color: ACCENT,
        });
        self.compositor.push(SceneNode::Rect {
            x: preview_x,
            y: preview_y + 2.0,
            w: preview_w,
            h: 140.0,
            color: SURFACE,
        });

        self.compositor.draw_text(
            TextNodeKey::new("LIVE PREVIEW", 13.0, 17.0, Some(preview_w - 20.0)),
            preview_x + 10.0,
            preview_y + 10.0,
            ACCENT,
        );

        for (i, (input, label)) in self.inputs.iter().zip(labels.iter()).enumerate() {
            let text = if input.buffer.is_empty() {
                format!("{}: (empty)", label)
            } else {
                format!("{}: {}", label, input.buffer.text())
            };
            let color = if input.buffer.is_empty() {
                TEXT_DIM
            } else {
                TEXT
            };
            self.compositor.draw_text(
                TextNodeKey::new(&text, 14.0, 20.0, Some(preview_w - 20.0)),
                preview_x + 10.0,
                preview_y + 36.0 + i as f32 * 28.0,
                color,
            );
        }

        let focus_text = match self.focus_index {
            Some(i) => format!("Focus: {} (press Tab to cycle)", labels[i]),
            None => "Focus: none (click a field or press Tab)".to_string(),
        };
        self.compositor.draw_text(
            TextNodeKey::new(&focus_text, 11.0, 15.0, Some(preview_w - 20.0)),
            preview_x + 10.0,
            preview_y + 120.0,
            TEXT_DIM,
        );
    }

    fn build_footer(&mut self, vw: f32, vh: f32) {
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
            TextNodeKey::new(
                "TextBuffer  |  Cursor blink 530ms  |  IME bridge  |  Selection",
                11.0,
                15.0,
                Some(vw - 64.0),
            ),
            32.0,
            vh - 22.0,
            TEXT_DIM,
        );
    }

    fn gpu_submit(&mut self) {
        let GpuState::Ready {
            ref mut gpu,
            ref mut text_system,
            ref mut pool,
        } = self.state
        else {
            return;
        };

        let _ = pool;

        let surface = match gpu.surface.as_ref() {
            Some(s) => s,
            None => return,
        };

        let output = match surface.get_current_texture() {
            Ok(t) => t,
            Err(plev::wgpu::SurfaceError::Lost | plev::wgpu::SurfaceError::Outdated) => {
                gpu.resize(gpu.surface_config.width, gpu.surface_config.height);
                return;
            }
            Err(_) => return,
        };

        let surface_view = gpu.surface_render_view(&output);
        text_system.begin_frame();

        self.compositor
            .resolve(&plev::compositor::ResolveResources {
                msaa_samples: gpu.config.msaa_samples,
                device: &gpu.device,
                queue: &gpu.queue,
                format: gpu.surface_format(),
                width: gpu.surface_config.width,
                height: gpu.surface_config.height,
                composite_bgl: &gpu.composite_bind_group_layout,
                opacity_bgl: &gpu.opacity_bind_group_layout,
                sampler: &gpu.composite_sampler,
            });

        {
            let layer_info: Vec<_> = self
                .compositor
                .layers()
                .iter()
                .map(|l| (l.id, l.is_dirty(), l.text_nodes()))
                .collect();
            for (layer_id, dirty, text_nodes) in layer_info {
                if !dirty {
                    continue;
                }
                let (vertices, indices) = text_system.resolve_for_layer(
                    &gpu.device,
                    &gpu.queue,
                    &gpu.text_bind_group_layout,
                    &text_nodes,
                );
                if let Some(layer) = self.compositor.layer_mut(layer_id) {
                    layer.set_text_data(&gpu.device, &gpu.queue, vertices, indices);
                }
            }
        }
        text_system.finish_frame();

        let mut encoder =
            gpu.device
                .create_command_encoder(&plev::wgpu::CommandEncoderDescriptor {
                    label: Some("text_input_encoder"),
                });

        let dirty_ids: Vec<_> = self
            .compositor
            .layers()
            .iter()
            .filter(|l| l.visible && l.is_dirty())
            .map(|l| l.id)
            .collect();

        for layer_id in &dirty_ids {
            let layer = self.compositor.layer(*layer_id).unwrap();
            let Some(msaa_v) = layer.msaa_view() else {
                continue;
            };
            let resolve_v = layer.texture_view();
            let mut pass = encoder.begin_render_pass(&plev::wgpu::RenderPassDescriptor {
                label: Some("layer_pass"),
                color_attachments: &[Some(plev::wgpu::RenderPassColorAttachment {
                    view: msaa_v,
                    resolve_target: resolve_v,
                    ops: plev::wgpu::Operations {
                        load: plev::wgpu::LoadOp::Clear(plev::wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        }),
                        store: plev::wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            if let Some((vb, ib, count)) = layer.quad_buffers() {
                pass.set_pipeline(&gpu.quad_pipeline);
                pass.set_bind_group(0, &gpu.projection_bind_group, &[]);
                pass.set_vertex_buffer(0, vb.slice(..));
                pass.set_index_buffer(ib.slice(..), plev::wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..count, 0, 0..1);
            }
            if let Some((vb, ib, count)) = layer.text_buffers() {
                pass.set_pipeline(&gpu.text_pipeline);
                pass.set_bind_group(0, &gpu.projection_bind_group, &[]);
                pass.set_bind_group(1, &text_system.atlas_bind_group, &[]);
                pass.set_vertex_buffer(0, vb.slice(..));
                pass.set_index_buffer(ib.slice(..), plev::wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..count, 0, 0..1);
            }
        }

        for id in &dirty_ids {
            self.compositor.mark_layer_clean(*id);
        }

        {
            let mut pass = encoder.begin_render_pass(&plev::wgpu::RenderPassDescriptor {
                label: Some("composite_pass"),
                color_attachments: &[Some(plev::wgpu::RenderPassColorAttachment {
                    view: &surface_view,
                    resolve_target: None,
                    ops: plev::wgpu::Operations {
                        load: plev::wgpu::LoadOp::Clear({
                            let [lr, lg, lb, la] =
                                plev::color::Color::rgb(BG[0], BG[1], BG[2]).to_linear_array();
                            plev::wgpu::Color {
                                r: lr as f64,
                                g: lg as f64,
                                b: lb as f64,
                                a: la as f64,
                            }
                        }),
                        store: plev::wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&gpu.composite_pipeline);
            for layer in self.compositor.layers() {
                if !layer.visible {
                    continue;
                }
                if let (Some(bg), Some(opacity_bg)) =
                    (layer.composite_bind_group(), layer.opacity_bind_group())
                {
                    pass.set_bind_group(0, bg, &[]);
                    pass.set_bind_group(1, opacity_bg, &[]);
                    pass.draw(0..3, 0..1);
                }
            }
        }

        gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}
