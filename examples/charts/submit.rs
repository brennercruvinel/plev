// GPU submission for the charts demo: resolve dirty layers, encode
// per-layer passes, composite to the srgb surface. Kept apart from
// render.rs so chart drawing and gpu plumbing each own one file.

use crate::palette::BG;
use crate::state::{App, GpuState};

impl App {
    pub(crate) fn gpu_submit(&mut self) {
        let GpuState::Ready {
            ref mut gpu,
            ref mut text_system,
            ref mut pool,
        } = self.gpu_state
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
                    label: Some("charts_encoder"),
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
            if let Some((vb, ib, count)) = layer.sdf_rect_buffers() {
                pass.set_pipeline(&gpu.rect_sdf_pipeline);
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
                if let (Some(bg), Some(obg)) =
                    (layer.composite_bind_group(), layer.opacity_bind_group())
                {
                    pass.set_bind_group(0, bg, &[]);
                    pass.set_bind_group(1, obg, &[]);
                    pass.draw(0..3, 0..1);
                }
            }
        }
        gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}
