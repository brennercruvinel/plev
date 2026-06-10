use crate::compositor::LayerEffect;

impl super::App {
    pub(super) fn encode_layer_passes(
        compositor: &crate::compositor::Compositor,
        gpu: &crate::gpu::GpuContext,
        text_system: &crate::text::TextSystem,
        dirty_layer_ids: &[crate::compositor::LayerId],
        encoder: &mut wgpu::CommandEncoder,
    ) {
        for layer_id in dirty_layer_ids {
            let layer = compositor.layer(*layer_id).unwrap();
            let Some(msaa_v) = layer.msaa_view() else {
                continue;
            };
            let resolve_v = layer.texture_view();

            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("layer_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: msaa_v,
                    resolve_target: resolve_v,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            if let Some((cx, cy, cw, ch)) = layer.clip_rect {
                pass.set_scissor_rect(cx, cy, cw, ch);
            }

            if let Some((vb, ib, count)) = layer.quad_buffers() {
                pass.set_pipeline(&gpu.quad_pipeline);
                pass.set_bind_group(0, &gpu.projection_bind_group, &[]);
                pass.set_vertex_buffer(0, vb.slice(..));
                pass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..count, 0, 0..1);
            }

            if let Some((vb, ib, count)) = layer.sdf_rect_buffers() {
                pass.set_pipeline(&gpu.rect_sdf_pipeline);
                pass.set_bind_group(0, &gpu.projection_bind_group, &[]);
                pass.set_vertex_buffer(0, vb.slice(..));
                pass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..count, 0, 0..1);
            }

            if let Some((vb, ib, count)) = layer.text_buffers() {
                pass.set_pipeline(&gpu.text_pipeline);
                pass.set_bind_group(0, &gpu.projection_bind_group, &[]);
                pass.set_bind_group(1, &text_system.atlas_bind_group, &[]);
                pass.set_vertex_buffer(0, vb.slice(..));
                pass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..count, 0, 0..1);
            }
        }
    }

    pub(super) fn apply_layer_effects(
        compositor: &crate::compositor::Compositor,
        gpu: &mut crate::gpu::GpuContext,
        effect_processor: &crate::effects::EffectProcessor,
        texture_pool: &mut crate::texture_pool::TexturePool,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Vec<(crate::compositor::LayerId, wgpu::BindGroup)> {
        let mut effect_results: Vec<(crate::compositor::LayerId, wgpu::BindGroup)> = Vec::new();
        let sw = gpu.surface_config.width;
        let sh = gpu.surface_config.height;

        let effect_layers: Vec<_> = compositor
            .layers()
            .iter()
            .filter(|l| l.visible && l.has_effects())
            .map(|l| (l.id, l.effects().to_vec(), l.texture_view().map(|_| l.id)))
            .collect();

        for (layer_id, effects, has_tv) in &effect_layers {
            if has_tv.is_none() {
                continue;
            }
            let layer = compositor.layer(*layer_id).unwrap();
            let source_view = layer.texture_view().unwrap();
            let mut current_view_owner: Option<crate::texture_pool::TextureHandle> = None;

            for effect in effects {
                let sv = current_view_owner
                    .as_ref()
                    .map(|h| h.view())
                    .unwrap_or(source_view);
                let handle = match effect {
                    LayerEffect::Blur { sigma } => effect_processor.apply_blur(
                        &mut crate::effects::EffectContext {
                            device: &gpu.device,
                            queue: &gpu.queue,
                            encoder,
                            pool: texture_pool,
                            source_view: sv,
                            width: sw,
                            height: sh,
                        },
                        *sigma,
                    ),
                    LayerEffect::Shadow { sigma, color } => effect_processor.apply_shadow(
                        &mut crate::effects::EffectContext {
                            device: &gpu.device,
                            queue: &gpu.queue,
                            encoder,
                            pool: texture_pool,
                            source_view: sv,
                            width: sw,
                            height: sh,
                        },
                        *sigma,
                        *color,
                    ),
                };
                if let Some(prev) = current_view_owner.take() {
                    texture_pool.release(prev);
                }
                current_view_owner = Some(handle);
            }

            if let Some(ref handle) = current_view_owner {
                let bg = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("effect_composite_bg"),
                    layout: &gpu.composite_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(handle.view()),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&gpu.composite_sampler),
                        },
                    ],
                });
                effect_results.push((*layer_id, bg));
            }
            if let Some(handle) = current_view_owner {
                texture_pool.release(handle);
            }
        }

        effect_results
    }

    pub(super) fn encode_composite_pass(
        compositor: &crate::compositor::Compositor,
        theme: &crate::theme::Theme,
        gpu: &crate::gpu::GpuContext,
        surface_view: &wgpu::TextureView,
        effect_results: &[(crate::compositor::LayerId, wgpu::BindGroup)],
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("composite_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: surface_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear({
                        let bg = theme.colors.bg.to_array();
                        wgpu::Color {
                            r: bg[0] as f64,
                            g: bg[1] as f64,
                            b: bg[2] as f64,
                            a: 1.0,
                        }
                    }),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        pass.set_pipeline(&gpu.composite_pipeline);

        for layer in compositor.layers() {
            if !layer.visible {
                continue;
            }
            let composite_bg = effect_results
                .iter()
                .find(|(id, _)| *id == layer.id)
                .map(|(_, bg)| bg);

            let orig_composite_bg = layer.composite_bind_group();
            let final_bg = composite_bg.or(orig_composite_bg);

            if let (Some(bg), Some(opacity_bg)) = (final_bg, layer.opacity_bind_group()) {
                pass.set_bind_group(0, bg, &[]);
                pass.set_bind_group(1, opacity_bg, &[]);
                pass.draw(0..3, 0..1);
            }
        }
    }
}
