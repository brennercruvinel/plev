//! Frame presentation: resolves the compositor, shapes/uploads text for
//! dirty layers, encodes the per-layer and composite render passes, and
//! presents the surface. Pure GPU plumbing — no app or editor state.

use plev::compositor::Compositor;
use plev::gpu::GpuContext;
use plev::text::TextSystem;

/// Encode and present one frame of the already-built scene in `compositor`.
/// `background` is the editor theme's sRGB background color.
pub fn present_frame(
    gpu: &mut GpuContext,
    text_system: &mut TextSystem,
    compositor: &mut Compositor,
    background: [f32; 4],
) {
    let Some(surface) = gpu.surface.as_ref() else {
        return;
    };
    let output = match surface.get_current_texture() {
        Ok(t) => t,
        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
            gpu.resize(gpu.surface_config.width, gpu.surface_config.height);
            return;
        }
        Err(_) => return,
    };
    let surface_view = gpu.surface_render_view(&output);

    compositor.resolve(&plev::compositor::ResolveResources {
        device: &gpu.device,
        queue: &gpu.queue,
        format: gpu.surface_format(),
        width: gpu.surface_config.width,
        height: gpu.surface_config.height,
        msaa_samples: gpu.config.msaa_samples,
        composite_bgl: &gpu.composite_bind_group_layout,
        opacity_bgl: &gpu.opacity_bind_group_layout,
        sampler: &gpu.composite_sampler,
    });

    // Shape and upload text for dirty layers.
    text_system.begin_frame();
    let layer_info: Vec<_> = compositor
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
        if let Some(layer) = compositor.layer_mut(layer_id) {
            layer.set_text_data(&gpu.device, &gpu.queue, vertices, indices);
        }
    }
    text_system.finish_frame();

    let mut encoder = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("editor_demo_frame"),
        });

    let dirty_layer_ids: Vec<_> = compositor
        .layers()
        .iter()
        .filter(|l| l.visible && l.is_dirty())
        .map(|l| l.id)
        .collect();

    for layer_id in &dirty_layer_ids {
        let layer = compositor.layer(*layer_id).unwrap();
        let Some((view, resolve_target)) = layer.render_attachment() else {
            continue;
        };
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("layer_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
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
    for id in &dirty_layer_ids {
        compositor.mark_layer_clean(*id);
    }

    // Composite all layers to the surface.
    {
        let [r, g, b, a] = background;
        // Linear clear value (the sRGB surface re-encodes on write).
        let lin = |c: f32| -> f64 {
            let c = c as f64;
            if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        };
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("composite_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &surface_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: lin(r),
                        g: lin(g),
                        b: lin(b),
                        a: a as f64,
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
