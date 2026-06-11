//! GPU render pipeline for the SDF shapes demo.

use crate::shapes::BG;
use crate::{App, GpuState};

pub(crate) fn render(app: &mut App) {
    let tick = app.clock.tick();
    app.pulse.tick(tick.dt);
    if !app.pulse.is_animating() {
        let cur = app.pulse.get();
        app.pulse.set_target(if cur > 0.5 { 0.0 } else { 1.0 });
    }
    app.frame += 1;

    let (vw, vh) = match &app.gpu_state {
        GpuState::Ready { gpu, .. } => (
            gpu.surface_config.width as f32,
            gpu.surface_config.height as f32,
        ),
        _ => return,
    };

    build_scene(app, vw, vh);
    submit_gpu(app, vw, vh);
}

fn build_scene(app: &mut App, vw: f32, vh: f32) {
    use crate::shapes::{DIVIDER, TEXT, TEXT_DIM, body_style, title_style};
    use plev::compositor::{SceneNode, TextNodeKey};

    app.compositor.begin_frame();

    // Header
    app.compositor.push(SceneNode::Rect {
        x: 0.0,
        y: 0.0,
        w: vw,
        h: 70.0,
        color: [0.07, 0.07, 0.11, 1.0],
    });
    app.compositor.push(SceneNode::Rect {
        x: 0.0,
        y: 69.0,
        w: vw,
        h: 1.0,
        color: DIVIDER,
    });
    app.compositor.draw_text(
        TextNodeKey::from_style("SDF SHAPES", &title_style(28.0, 34.0), Some(vw - 64.0)),
        32.0,
        12.0,
        TEXT,
    );
    app.compositor.draw_text(
        TextNodeKey::from_style(
            "RoundedRect + PathBuilder + Composition",
            &body_style(12.0, 16.0),
            Some(vw - 64.0),
        ),
        32.0,
        46.0,
        TEXT_DIM,
    );

    let margin = 24.0;
    let gap = 16.0;
    let cols = 4.0;
    let card_w = (vw - margin * 2.0 - gap * (cols - 1.0)) / cols;
    let card_h = ((vh - 70.0 - 32.0 - margin * 2.0 - gap) / 2.0)
        .min(280.0)
        .max(180.0);
    let y0 = 70.0 + margin;
    let y1 = y0 + card_h + gap;

    let p = app.pulse.get();
    let t = app.frame as f32 * 0.02;

    crate::cards_row1::draw_row1(&mut app.compositor, margin, gap, card_w, card_h, y0, t);
    crate::cards_row2::draw_row2(&mut app.compositor, margin, gap, card_w, card_h, y1, p, t);

    // Footer
    app.compositor.push(SceneNode::Rect {
        x: 0.0,
        y: vh - 28.0,
        w: vw,
        h: 28.0,
        color: [0.06, 0.06, 0.10, 1.0],
    });
    app.compositor.push(SceneNode::Rect {
        x: 0.0,
        y: vh - 28.0,
        w: vw,
        h: 1.0,
        color: DIVIDER,
    });
    app.compositor.draw_text(
        TextNodeKey::from_style(
            "RoundedRect SDF + PathBuilder (Lyon) + pseudo-shadows | All cross-platform",
            &body_style(10.0, 14.0),
            Some(vw - 64.0),
        ),
        32.0,
        vh - 20.0,
        TEXT_DIM,
    );
}

fn submit_gpu(app: &mut App, _vw: f32, _vh: f32) {
    let GpuState::Ready {
        ref mut gpu,
        ref mut text_system,
        ref mut pool,
    } = app.gpu_state
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
    app.compositor.resolve(&plev::compositor::ResolveResources {
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
        let layer_info: Vec<_> = app
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
            if let Some(layer) = app.compositor.layer_mut(layer_id) {
                layer.set_text_data(&gpu.device, &gpu.queue, vertices, indices);
            }
        }
    }
    text_system.finish_frame();
    let mut encoder = gpu
        .device
        .create_command_encoder(&plev::wgpu::CommandEncoderDescriptor {
            label: Some("sdf_encoder"),
        });
    let dirty_ids: Vec<_> = app
        .compositor
        .layers()
        .iter()
        .filter(|l| l.visible && l.is_dirty())
        .map(|l| l.id)
        .collect();
    for layer_id in &dirty_ids {
        let layer = app.compositor.layer(*layer_id).unwrap();
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
        app.compositor.mark_layer_clean(*id);
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
        for layer in app.compositor.layers() {
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
