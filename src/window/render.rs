use super::state::GpuState;
use crate::compositor::Layer;

impl super::App {
    pub(crate) fn render(&mut self) {
        // Tick frame clock
        self.animation_tick = self.frame_clock.tick();

        // Bail out early if the GPU isn't ready yet
        if !matches!(self.state, GpuState::Ready { .. }) {
            return;
        }

        // begin_frame -- clear previous scene BEFORE building new one
        self.compositor.begin_frame();

        // Update frame signal
        let counter_value = self.frame_read.get();
        self.frame_write.set(counter_value + 1);

        // Now borrow GPU state for rendering
        let GpuState::Ready {
            ref mut gpu,
            ref mut text_system,
            ref effect_processor,
            ref mut texture_pool,
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
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                gpu.resize(gpu.surface_config.width, gpu.surface_config.height);
                return;
            }
            Err(wgpu::SurfaceError::Timeout) => {
                log::warn!("Surface timeout");
                return;
            }
            Err(e) => {
                log::error!("Surface error: {:?}", e);
                return;
            }
        };

        let surface_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        text_system.begin_frame();
        self.ime_state.begin_frame();

        // Resolve compositor
        self.compositor
            .resolve(&crate::compositor::ResolveResources {
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

        // Resolve text for each dirty layer, one resolve per clip group so
        // clipped text (scrolled lists, panels) scissors with its container.
        {
            let layer_info: Vec<_> = self
                .compositor
                .layers()
                .iter()
                .map(|l| (l.id, l.is_dirty(), l.text_node_groups()))
                .collect();

            for (layer_id, dirty, groups) in layer_info {
                if !dirty {
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
                let (vertices, indices, ranges) =
                    crate::compositor::merge_text_groups(resolved);
                if let Some(layer) = self.compositor.layer_mut(layer_id) {
                    layer.set_text_data_with_ranges(
                        &gpu.device,
                        &gpu.queue,
                        vertices,
                        indices,
                        ranges,
                    );
                }
            }
        }

        text_system.finish_frame();

        // Update accessibility tree if active
        #[cfg(feature = "accessibility")]
        if self.a11y_state.is_active() {
            self.a11y_state.begin_frame();
            // Push accessible nodes from hit regions
            for region in self.input_state.hit_regions() {
                let role = if region.focusable {
                    accesskit::Role::Button
                } else {
                    accesskit::Role::GenericContainer
                };
                self.a11y_state.push_node(
                    region.view_id,
                    role,
                    None,
                    [region.x, region.y, region.w, region.h],
                    region.focusable,
                    None,
                );
            }
            self.a11y_state
                .update_focus_graph(self.input_state.hit_regions());
            let update = self
                .a11y_state
                .build_tree_update(self.input_state.focused_view());
            if let Some(ref mut adapter) = self.a11y_adapter {
                adapter.update_if_active(|| update);
            }
        }

        // Encode render passes
        let encode_start = web_time::Instant::now();
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render_encoder"),
            });

        // Per-layer render passes (to offscreen textures)
        let dirty_layer_ids: Vec<_> = self
            .compositor
            .layers()
            .iter()
            .filter(|l| l.visible && l.is_dirty())
            .map(|l| l.id)
            .collect();

        let layer_draws = Self::encode_layer_passes(
            &self.compositor,
            gpu,
            text_system,
            &dirty_layer_ids,
            &mut encoder,
        );
        let effect_results = Self::apply_layer_effects(
            &self.compositor,
            gpu,
            effect_processor,
            texture_pool,
            &mut encoder,
        );

        // Mark rendered layers clean
        for id in &dirty_layer_ids {
            self.compositor.mark_layer_clean(*id);
        }

        // Composite pass: draw all visible layers to surface
        let composite_draws = Self::encode_composite_pass(
            &self.compositor,
            &self.theme,
            gpu,
            &surface_view,
            &effect_results,
            &mut encoder,
        );

        gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        let glyphs: u32 = self
            .compositor
            .layers()
            .iter()
            .map(Layer::glyph_count)
            .sum();
        self.compositor.record_encode_stats(
            layer_draws + composite_draws,
            glyphs,
            encode_start.elapsed().as_micros() as u64,
        );
    }
}
