//! Render pipeline for the counter demo: scene building + GPU submission.

use plev::compositor::{SceneNode, TextNodeKey};
use plev::view::ViewContext;

use crate::lifecycle::*;
use crate::{AppState, CounterApp};

pub fn render(app: &mut CounterApp) {
    let AppState::Ready {
        ref mut gpu,
        ref mut text_system,
    } = app.state
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

    app.compositor.begin_frame();
    text_system.begin_frame();

    // -- Header --
    app.compositor.push(SceneNode::Rect {
        x: 0.0,
        y: 0.0,
        w,
        h: HEADER_H,
        color: HEADER_BG,
    });
    app.compositor.draw_text(
        TextNodeKey::new("COMPONENT COUNTER", 24.0, 31.2, Some(w - MARGIN * 2.0)),
        MARGIN,
        16.0,
        TEXT,
    );
    app.compositor.draw_text(
        TextNodeKey::new(
            "State persists between frames via Lifecycle trait",
            12.0,
            15.6,
            Some(w - MARGIN * 2.0),
        ),
        MARGIN,
        48.0,
        TEXT_DIM,
    );
    app.compositor.push(SceneNode::Rect {
        x: 0.0,
        y: HEADER_H - 1.0,
        w,
        h: 1.0,
        color: DIVIDER,
    });

    // -- Counter component (left card) --
    let mut cx = ViewContext::new(w, h);
    let nodes = app.counter.render(&mut cx);
    for node in nodes {
        app.compositor.push(node);
    }

    // -- Info card (right side) --
    build_info_card(&mut app.compositor, w);

    // -- Footer --
    build_footer(&mut app.compositor, w, h);

    // -- Resolve & render --
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
            label: Some("counter_encoder"),
        });

    let dirty_layer_ids: Vec<_> = app
        .compositor
        .layers()
        .iter()
        .filter(|l| l.visible && l.is_dirty())
        .map(|l| l.id)
        .collect();

    for layer_id in &dirty_layer_ids {
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

        if let Some((vb, ib, count)) = layer.text_buffers() {
            pass.set_pipeline(&gpu.text_pipeline);
            pass.set_bind_group(0, &gpu.projection_bind_group, &[]);
            pass.set_bind_group(1, &text_system.atlas_bind_group, &[]);
            pass.set_vertex_buffer(0, vb.slice(..));
            pass.set_index_buffer(ib.slice(..), plev::wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..count, 0, 0..1);
        }
    }

    for id in &dirty_layer_ids {
        app.compositor.mark_layer_clean(*id);
    }

    // Composite pass
    {
        let mut pass = encoder.begin_render_pass(&plev::wgpu::RenderPassDescriptor {
            label: Some("composite_pass"),
            color_attachments: &[Some(plev::wgpu::RenderPassColorAttachment {
                view: &view,
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
            if let (Some(cbg), Some(obg)) =
                (layer.composite_bind_group(), layer.opacity_bind_group())
            {
                pass.set_bind_group(0, cbg, &[]);
                pass.set_bind_group(1, obg, &[]);
                pass.draw(0..3, 0..1);
            }
        }
    }

    gpu.queue.submit(std::iter::once(encoder.finish()));
    output.present();
}

// ---------------------------------------------------------------------------
// Scene-building helpers
// ---------------------------------------------------------------------------

fn build_info_card(comp: &mut plev::compositor::Compositor, w: f32) {
    let content_w = w - MARGIN * 2.0;
    let left_card_w = content_w * 0.55;
    let gap = 16.0;
    let info_x = MARGIN + left_card_w + gap;
    let info_w = content_w - left_card_w - gap;
    let info_y = HEADER_H + MARGIN;
    let info_h = 180.0;

    comp.push(SceneNode::Rect {
        x: info_x,
        y: info_y,
        w: info_w,
        h: info_h,
        color: SURFACE,
    });
    comp.push(SceneNode::Rect {
        x: info_x,
        y: info_y,
        w: info_w,
        h: ACCENT_BAR_H,
        color: CYAN,
    });
    comp.draw_text(
        TextNodeKey::new("LIFECYCLE", 11.0, 14.3, Some(info_w - 32.0)),
        info_x + 16.0,
        info_y + 16.0,
        CYAN,
    );

    let labels = [
        "on_mount()    -> first frame",
        "on_update()   -> every frame",
        "on_unmount()  -> cleanup",
    ];
    for (i, label) in labels.iter().enumerate() {
        let ly = info_y + 46.0 + (i as f32) * 24.0;
        comp.draw_text(
            TextNodeKey::new(label, 11.0, 14.3, Some(info_w - 32.0)),
            info_x + 16.0,
            ly,
            TEXT_DIM,
        );
    }

    comp.push(SceneNode::Rect {
        x: info_x + 16.0,
        y: info_y + 124.0,
        w: info_w - 32.0,
        h: 1.0,
        color: DIVIDER,
    });
    comp.draw_text(
        TextNodeKey::new(
            "Component<T> caches SceneNodes",
            10.0,
            13.0,
            Some(info_w - 32.0),
        ),
        info_x + 16.0,
        info_y + 140.0,
        TEXT_DIM,
    );
}

fn build_footer(comp: &mut plev::compositor::Compositor, w: f32, h: f32) {
    let footer_y = h - FOOTER_H;
    comp.push(SceneNode::Rect {
        x: 0.0,
        y: footer_y,
        w,
        h: 1.0,
        color: DIVIDER,
    });
    comp.push(SceneNode::Rect {
        x: 0.0,
        y: footer_y + 1.0,
        w,
        h: FOOTER_H - 1.0,
        color: FOOTER_BG,
    });
    comp.draw_text(
        TextNodeKey::new(
            "Component<Counter>  |  Lifecycle trait  |  Cached SceneNodes",
            11.0,
            14.3,
            Some(w - MARGIN * 2.0),
        ),
        MARGIN,
        footer_y + 9.0,
        TEXT_DIM,
    );
}
