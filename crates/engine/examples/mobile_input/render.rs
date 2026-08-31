// Rendering: scene construction and GPU submission for the mobile input demo.

use crate::palette::BG;
use crate::state::{MobileInputApp, State};

impl MobileInputApp {
    pub fn render(&mut self) {
        // Early-exit and grab surface dimensions before building UI.
        let (w, h) = match &self.state {
            State::Ready { gpu, .. } => (
                gpu.surface_config.width as f32,
                gpu.surface_config.height as f32,
            ),
            _ => return,
        };

        // Handle IME committed text and begin frame for compositor.
        if !self.ime_state.committed_text.is_empty() {
            self.input_text.push_str(&self.ime_state.committed_text);
        }
        self.ime_state.begin_frame();

        self.compositor.begin_frame();

        // Build UI scene nodes (only touches compositor, safe_area, ime_state, etc.).
        self.build_safe_area_bars(w, h);
        self.build_content(w, h);

        // Now destructure state for GPU submission.
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

        text_system.begin_frame();

        // Resolve and GPU submit
        self.compositor
            .resolve(&engine::compositor::ResolveResources {
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

        // Resolve text per clip group so the ranges patched into the
        // draw sequence line up 1:1 with its Text commands.
        engine::window::resolve_layer_text(&mut self.compositor, gpu, text_system);
        text_system.finish_frame();

        let mut encoder =
            gpu.device
                .create_command_encoder(&engine::wgpu::CommandEncoderDescriptor {
                    label: Some("mobile_input_encoder"),
                });

        let dirty_layer_ids: Vec<_> = self
            .compositor
            .layers()
            .iter()
            .filter(|l| l.visible && l.is_dirty())
            .map(|l| l.id)
            .collect();

        for layer_id in &dirty_layer_ids {
            let layer = self.compositor.layer(*layer_id).unwrap();
            let Some(msaa_v) = layer.msaa_view() else {
                continue;
            };
            let resolve_v = layer.texture_view();
            let mut pass = encoder.begin_render_pass(&engine::wgpu::RenderPassDescriptor {
                label: Some("layer_pass"),
                color_attachments: &[Some(engine::wgpu::RenderPassColorAttachment {
                    view: msaa_v,
                    resolve_target: resolve_v,
                    ops: engine::wgpu::Operations {
                        load: engine::wgpu::LoadOp::Clear(engine::wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        }),
                        store: engine::wgpu::StoreOp::Store,
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
                pass.set_index_buffer(ib.slice(..), engine::wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..count, 0, 0..1);
            }
            if let Some((vb, ib, count)) = layer.text_buffers() {
                pass.set_pipeline(&gpu.text_pipeline);
                pass.set_bind_group(0, &gpu.projection_bind_group, &[]);
                pass.set_bind_group(1, &text_system.atlas_bind_group, &[]);
                pass.set_vertex_buffer(0, vb.slice(..));
                pass.set_index_buffer(ib.slice(..), engine::wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..count, 0, 0..1);
            }
        }

        for id in &dirty_layer_ids {
            self.compositor.mark_layer_clean(*id);
        }

        {
            let mut pass = encoder.begin_render_pass(&engine::wgpu::RenderPassDescriptor {
                label: Some("composite_pass"),
                color_attachments: &[Some(engine::wgpu::RenderPassColorAttachment {
                    view: &surface_view,
                    resolve_target: None,
                    ops: engine::wgpu::Operations {
                        load: engine::wgpu::LoadOp::Clear({
                            let [lr, lg, lb, la] =
                                engine::color::Color::rgb(BG[0], BG[1], BG[2]).to_linear_array();
                            engine::wgpu::Color {
                                r: lr as f64,
                                g: lg as f64,
                                b: lb as f64,
                                a: la as f64,
                            }
                        }),
                        store: engine::wgpu::StoreOp::Store,
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
                if let (Some(composite_bg), Some(opacity_bg)) =
                    (layer.composite_bind_group(), layer.opacity_bind_group())
                {
                    pass.set_bind_group(0, composite_bg, &[]);
                    pass.set_bind_group(1, opacity_bg, &[]);
                    pass.draw(0..3, 0..1);
                }
            }
        }

        gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}
