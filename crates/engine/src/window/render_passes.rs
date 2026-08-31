use crate::compositor::{
    ClipRect, DrawCommand, DrawKind, LayerEffect, clip_to_scissor, intersect_scissors,
};

/// Scissor for one draw command: the command's clip (intersection of the
/// PushClip stack, in scene coordinates scaled to physical pixels by
/// `clip_scale`) clamped to the viewport and intersected with the layer's
/// own clip rect. `None` means the visible area is empty (skip the draw).
fn command_scissor(
    clip: Option<ClipRect>,
    base_scissor: (u32, u32, u32, u32),
    clip_scale: (f32, f32),
    vw: u32,
    vh: u32,
) -> Option<(u32, u32, u32, u32)> {
    match clip {
        None => Some(base_scissor),
        Some(clip) => {
            let clip = [
                clip[0] * clip_scale.0,
                clip[1] * clip_scale.1,
                clip[2] * clip_scale.0,
                clip[3] * clip_scale.1,
            ];
            clip_to_scissor(clip, vw, vh).and_then(|s| intersect_scissors(s, base_scissor))
        }
    }
}

/// Whether a layer's text must be re-resolved this frame.
///
/// Normally only layers whose scene changed need it. A raster-scale change
/// is the exception: it resets the glyph cache and hands the whole atlas
/// back to the allocator, so the vertices of an untouched layer point at
/// texels that the next glyphs will be packed over. Skipping those layers
/// is what made a window dragged between displays of different DPI come
/// back with scrambled text.
pub(crate) fn must_resolve_text(layer_dirty: bool, raster_scale_changed: bool) -> bool {
    layer_dirty || raster_scale_changed
}

#[cfg(test)]
mod resolve_gate_tests {
    use super::must_resolve_text;

    #[test]
    fn clean_layers_are_skipped_normally() {
        assert!(!must_resolve_text(false, false));
        assert!(must_resolve_text(true, false));
    }

    #[test]
    fn a_raster_scale_change_forces_even_clean_layers() {
        assert!(
            must_resolve_text(false, true),
            "a clean layer keeps vertices into an atlas that the scale change \
             just repacked; it must be re-resolved"
        );
        assert!(must_resolve_text(true, true));
    }
}

/// Resolve text for every dirty layer, one `resolve_for_layer` call per
/// clip group so clipped text (scrolled lists, panels) scissors with its
/// container. Shared by the built-in render loop and standalone apps.
///
/// Also syncs the glyph raster scale with the active projection: scenes
/// laid out in logical coordinates on a HiDPI surface need glyph bitmaps
/// at physical resolution, or text renders blurry (bitmaps stretched by
/// the projection). Derived here so every app gets it without plumbing.
pub fn resolve_layer_text(
    compositor: &mut crate::compositor::Compositor,
    gpu: &crate::gpu::GpuContext,
    text_system: &mut crate::text::TextSystem,
) {
    let (scale, _) = gpu.clip_scale();
    // A scale change resets the glyph cache and repacks the atlas, so every
    // layer's text vertices are stale — including layers whose scene did not
    // change and would otherwise be skipped below.
    let scale_changed = text_system.set_raster_scale(scale);

    let layer_info: Vec<_> = compositor
        .layers()
        .iter()
        .map(|l| (l.id, l.is_dirty(), l.text_node_groups()))
        .collect();

    for (layer_id, dirty, groups) in layer_info {
        if !must_resolve_text(dirty, scale_changed) {
            continue;
        }
        let resolved: Vec<_> = groups
            .into_iter()
            .map(|(nodes, clip)| {
                let (vertices, indices) = text_system.resolve_for_layer(
                    &gpu.device,
                    &gpu.queue,
                    &gpu.text_bind_group_layout,
                    &nodes,
                );
                (vertices, indices, clip)
            })
            .collect();
        let (vertices, indices, ranges) = crate::compositor::merge_text_groups(resolved);
        if let Some(layer) = compositor.layer_mut(layer_id) {
            layer.set_text_data_with_ranges(&gpu.device, &gpu.queue, vertices, indices, ranges);
        }
    }
}

/// Bind the pipeline + buffers + bind groups for one draw kind. Returns
/// `false` (nothing bound) when the layer has no buffers for that kind or
/// a required atlas is missing.
fn bind_draw_kind(
    pass: &mut wgpu::RenderPass<'_>,
    kind: DrawKind,
    layer: &crate::compositor::Layer,
    gpu: &crate::gpu::GpuContext,
    text_system: &crate::text::TextSystem,
) -> bool {
    let buffers = match kind {
        DrawKind::Quad => layer.quad_buffers(),
        DrawKind::Shadow => layer.shadow_buffers(),
        DrawKind::SdfRect => layer.sdf_rect_buffers(),
        DrawKind::Image => layer.image_buffers(),
        DrawKind::Text => layer.text_buffers(),
    };
    let Some((vb, ib, _)) = buffers else {
        return false;
    };
    match kind {
        DrawKind::Quad => pass.set_pipeline(&gpu.quad_pipeline),
        DrawKind::Shadow => pass.set_pipeline(&gpu.shadow_analytic_pipeline),
        DrawKind::SdfRect => pass.set_pipeline(&gpu.rect_sdf_pipeline),
        DrawKind::Image => {
            let Some(image_bg) = gpu.image_atlas.bind_group() else {
                return false;
            };
            pass.set_pipeline(&gpu.image_pipeline);
            pass.set_bind_group(1, image_bg, &[]);
        }
        DrawKind::Text => {
            pass.set_pipeline(&gpu.text_pipeline);
            pass.set_bind_group(1, &text_system.atlas_bind_group, &[]);
        }
    }
    pass.set_bind_group(0, &gpu.projection_bind_group, &[]);
    pass.set_vertex_buffer(0, vb.slice(..));
    pass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);
    true
}

/// Begin (or resume) the render pass targeting a layer's attachment.
fn begin_layer_pass<'e>(
    encoder: &'e mut wgpu::CommandEncoder,
    view: &wgpu::TextureView,
    resolve_target: Option<&wgpu::TextureView>,
    load: wgpu::LoadOp<wgpu::Color>,
) -> wgpu::RenderPass<'e> {
    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("layer_pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view,
            resolve_target,
            ops: wgpu::Operations {
                load,
                store: wgpu::StoreOp::Store,
            },
            depth_slice: None,
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    })
}

/// Resolve the backdrop for one `BackdropBlur` command: composite the
/// background color, every visible layer below `layer_id`, and the
/// current layer's partial content (everything its pass drew so far) into
/// a pooled texture, then Gaussian-blur it with the 2-pass effect
/// processor. Returns the bind group sampling the blurred result plus the
/// number of draw calls issued.
///
/// Cost: one full-surface composite + blur per backdrop node. Scenes are
/// expected to carry only a few backdrops (cards, bars), so this version
/// favors correctness over batching; the pooled textures make the steady
/// state allocation-free.
#[allow(clippy::too_many_arguments)]
fn resolve_blurred_backdrop(
    compositor: &crate::compositor::Compositor,
    layer_id: crate::compositor::LayerId,
    gpu: &crate::gpu::GpuContext,
    effects: &crate::effects::EffectProcessor,
    texture_pool: &mut crate::gpu::texture_pool::TexturePool,
    background: wgpu::Color,
    sigma: f32,
    encoder: &mut wgpu::CommandEncoder,
) -> (wgpu::BindGroup, u32) {
    let vw = gpu.surface_config.width;
    let vh = gpu.surface_config.height;
    let format = gpu.surface_config.format;
    let mut draw_calls = 0u32;

    // 1) Compose "everything below this point": clear color, lower layers
    // in z-order, then this layer's own partial content (its pass was
    // suspended, so its texture holds what was drawn so far).
    let compose = texture_pool.acquire(&gpu.device, vw, vh, format);
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("backdrop_compose_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: compose.view(),
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(background),
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
        for l in compositor.layers() {
            let is_current = l.id == layer_id;
            if !is_current && !l.visible {
                continue;
            }
            if let (Some(bg), Some(opacity_bg)) = (l.composite_bind_group(), l.opacity_bind_group())
            {
                pass.set_bind_group(0, bg, &[]);
                pass.set_bind_group(1, opacity_bg, &[]);
                pass.draw(0..3, 0..1);
                draw_calls += 1;
            }
            if is_current {
                break;
            }
        }
    }

    // 2) Two-pass Gaussian blur of the composed backdrop.
    let compose_view = compose.view().clone();
    let blurred = effects.apply_blur(
        &mut crate::effects::EffectContext {
            device: &gpu.device,
            queue: &gpu.queue,
            encoder,
            pool: texture_pool,
            source_view: &compose_view,
            width: vw,
            height: vh,
        },
        sigma,
    );
    draw_calls += 2;
    texture_pool.release(compose);

    let backdrop_bg = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("backdrop_blur_bg"),
        layout: &gpu.composite_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(blurred.view()),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&gpu.composite_sampler),
            },
        ],
    });
    // Releasing before the sampling draw is recorded is safe: the bind
    // group keeps the texture alive, and any reuse of the pooled texture
    // is recorded -- and therefore executes -- after this read.
    texture_pool.release(blurred);

    (backdrop_bg, draw_calls)
}

/// Encode one render pass per dirty layer, walking the layer's draw
/// sequence so primitive types interleave in scene push order (a path
/// icon over an SDF pill, text behind a later rect). Pipeline switches
/// happen exactly where the sequence changes kind; consecutive commands
/// of one kind reuse the bound pipeline.
///
/// `BackdropBlur` commands suspend the layer pass (resolving its partial
/// content), blur everything composited below via
/// [`resolve_blurred_backdrop`], then resume the pass and draw the
/// frosted quad; later commands paint on top. `background` is the window
/// clear color so glass over bare background frosts correctly.
///
/// Returns the number of draw calls issued.
#[allow(clippy::too_many_arguments)]
pub fn encode_layer_passes(
    compositor: &crate::compositor::Compositor,
    gpu: &crate::gpu::GpuContext,
    text_system: &crate::text::TextSystem,
    effects: &crate::effects::EffectProcessor,
    texture_pool: &mut crate::gpu::texture_pool::TexturePool,
    background: wgpu::Color,
    dirty_layer_ids: &[crate::compositor::LayerId],
    encoder: &mut wgpu::CommandEncoder,
) -> u32 {
    let vw = gpu.surface_config.width;
    let vh = gpu.surface_config.height;
    let clip_scale = gpu.clip_scale();
    let mut draw_calls = 0u32;
    for layer_id in dirty_layer_ids {
        let layer = compositor.layer(*layer_id).unwrap();
        let Some((view, resolve_target)) = layer.render_attachment() else {
            continue;
        };

        let mut pass = begin_layer_pass(
            encoder,
            view,
            resolve_target,
            wgpu::LoadOp::Clear(wgpu::Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.0,
            }),
        );

        let base_scissor = layer
            .clip_rect
            .and_then(|c| intersect_scissors(c, (0, 0, vw, vh)))
            .unwrap_or((0, 0, vw, vh));

        // The kind currently bound on the pass; `None` until the first
        // successful bind (or after a bind failed / the pass restarted).
        let mut bound: Option<DrawKind> = None;
        for cmd in layer.sequence() {
            match *cmd {
                DrawCommand::Geometry { kind, range } => {
                    if range.index_count == 0 {
                        continue;
                    }
                    let Some((sx, sy, sw, sh)) =
                        command_scissor(range.clip, base_scissor, clip_scale, vw, vh)
                    else {
                        continue;
                    };
                    if bound != Some(kind) {
                        if !bind_draw_kind(&mut pass, kind, layer, gpu, text_system) {
                            bound = None;
                            continue;
                        }
                        bound = Some(kind);
                    }
                    pass.set_scissor_rect(sx, sy, sw, sh);
                    pass.draw_indexed(
                        range.first_index..range.first_index + range.index_count,
                        0,
                        0..1,
                    );
                    draw_calls += 1;
                }
                DrawCommand::BackdropBlur {
                    first_index,
                    sigma,
                    clip,
                } => {
                    let Some((sx, sy, sw, sh)) =
                        command_scissor(clip, base_scissor, clip_scale, vw, vh)
                    else {
                        continue;
                    };
                    let Some((vb, ib, _)) = layer.backdrop_buffers() else {
                        continue;
                    };

                    // Suspend the layer pass: ending it resolves what was
                    // drawn so far into the layer texture (the MSAA
                    // attachment keeps its samples via StoreOp::Store).
                    drop(pass);

                    let (backdrop_bg, resolve_draws) = resolve_blurred_backdrop(
                        compositor,
                        *layer_id,
                        gpu,
                        effects,
                        texture_pool,
                        background,
                        sigma,
                        encoder,
                    );
                    draw_calls += resolve_draws;

                    // Resume on top of the existing content and draw the
                    // frosted rounded rect.
                    pass = begin_layer_pass(encoder, view, resolve_target, wgpu::LoadOp::Load);
                    pass.set_pipeline(&gpu.backdrop_pipeline);
                    pass.set_bind_group(0, &gpu.projection_bind_group, &[]);
                    pass.set_bind_group(1, &backdrop_bg, &[]);
                    pass.set_vertex_buffer(0, vb.slice(..));
                    pass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);
                    pass.set_scissor_rect(sx, sy, sw, sh);
                    pass.draw_indexed(first_index..first_index + 6, 0, 0..1);
                    draw_calls += 1;
                    bound = None;
                }
            }
        }
    }
    draw_calls
}

impl super::App {
    pub(super) fn apply_layer_effects(
        compositor: &crate::compositor::Compositor,
        gpu: &mut crate::gpu::GpuContext,
        effect_processor: &crate::effects::EffectProcessor,
        texture_pool: &mut crate::gpu::texture_pool::TexturePool,
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
            let mut current_view_owner: Option<crate::gpu::texture_pool::TextureHandle> = None;

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
}

/// Encode the composite pass drawing all visible layers to the surface.
/// Returns the number of draw calls issued.
pub fn encode_composite_pass(
    compositor: &crate::compositor::Compositor,
    clear_color: wgpu::Color,
    gpu: &crate::gpu::GpuContext,
    surface_view: &wgpu::TextureView,
    effect_results: &[(crate::compositor::LayerId, wgpu::BindGroup)],
    encoder: &mut wgpu::CommandEncoder,
) -> u32 {
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("composite_pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: surface_view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(clear_color),
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

    let mut draw_calls = 0u32;
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
            draw_calls += 1;
        }
    }
    draw_calls
}
