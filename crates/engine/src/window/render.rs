use super::state::GpuState;
use crate::compositor::Layer;

impl super::App {
    /// The engine `App`'s frame. It keeps its own tail instead of calling
    /// `window::draw_frame`: it composites the per-layer effects (blur and
    /// shadow, `apply_layer_effects`) between the layer passes and the
    /// composite pass, and `draw_frame` composites with none. Going
    /// through `draw_frame` needs the effects inside it, a change to the
    /// frame path every app and the headless renderer share.
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
            ref mut effect_processor,
            ref mut texture_pool,
        } = self.state
        else {
            return;
        };

        #[cfg(hot_reload)]
        if gpu.poll_shader_reload(effect_processor) {
            self.compositor.mark_all_dirty();
        }

        // A lost device: recover in place and draw on the new one, or skip
        // the frame (as `present_frame` does).
        if gpu.poll_device_loss()
            && !crate::gpu::recover_or_skip(
                &mut self.compositor,
                gpu,
                text_system,
                effect_processor,
                texture_pool,
                "render",
            )
        {
            return;
        }

        let surface = match gpu.surface.as_ref() {
            Some(s) => s,
            None => return,
        };

        let output = match surface.get_current_texture() {
            Ok(t) => t,
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                // a lost device loses its surface too: the next requested
                // frame recovers before acquiring
                if gpu.poll_device_loss() {
                    self.compositor.invalidate();
                } else {
                    gpu.resize(gpu.surface_config.width, gpu.surface_config.height);
                }
                return;
            }
            Err(wgpu::SurfaceError::Timeout) => {
                log::warn!("Surface timeout");
                return;
            }
            Err(e) => {
                log::error!("Surface error: {e:?}");
                return;
            }
        };

        let surface_view = gpu.surface_render_view(&output);

        text_system.begin_frame();
        self.ime_state.begin_frame();

        // Perf HUD: engine-drawn overlay on its own high-z layer, fed with
        // the previous frame's snapshot (this frame's numbers land after
        // submit). Drawn before resolve so it enters this frame's passes.
        if gpu.config.perf_hud {
            let viewport_w = gpu
                .logical_size
                .map(|(w, _)| w)
                .unwrap_or(gpu.surface_config.width as f32);
            let snapshot = self.perf.snapshot();
            self.perf_hud
                .draw(&mut self.compositor, &snapshot, viewport_w);
        } else {
            self.perf_hud.clear(&mut self.compositor);
        }

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
        super::render_passes::resolve_layer_text(&mut self.compositor, gpu, text_system);

        text_system.finish_frame();

        // The accessibility tree of this frame, while a screen reader
        // listens. This `App` projects no meaning (no `sem::Snapshot`), so
        // the tree is the window alone and nothing the reader asks maps.
        #[cfg(all(feature = "accessibility", not(target_arch = "wasm32")))]
        if let (Some(bridge), Some(window)) = (self.a11y.as_mut(), self.window.as_ref()) {
            bridge.update(window, &self.compositor, || None);
            for assist in bridge.take() {
                log::debug!("accessibility: nothing handles {assist:?}");
            }
        }

        // Upload any images loaded while building the scene
        gpu.prepare_images();

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

        // Window clear color: composite base AND backdrop-blur base (glass
        // over the bare background frosts the same color the user sees).
        let clear_color = {
            // wgpu clear values are linear; the sRGB surface re-encodes on
            // write. Linearize the sRGB theme color so the bg shows its true
            // tone instead of a washed-out ~2.5× lighter gray.
            let bg = self.theme.colors.bg.to_linear_array();
            #[cfg(hot_reload)]
            let bg = crate::hot_reload::literals::linear(bg);
            wgpu::Color {
                r: bg[0] as f64,
                g: bg[1] as f64,
                b: bg[2] as f64,
                a: 1.0,
            }
        };

        let layer_draws = super::render_passes::encode_layer_passes(
            &self.compositor,
            gpu,
            text_system,
            effect_processor,
            texture_pool,
            clear_color,
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
        let composite_draws = super::render_passes::encode_composite_pass(
            &self.compositor,
            clear_color,
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

        // Feed the perf monitor from the existing per-frame sources.
        self.perf
            .record_frame(self.animation_tick, self.compositor.stats());
        self.perf.record_memory(crate::perf::MemoryStats {
            glyph_atlas_bytes: text_system.atlas_memory_bytes(),
            texture_pool_bytes: texture_pool.memory_bytes(),
            layer_bytes: self.compositor.gpu_memory_bytes(),
            process_rss_bytes: crate::perf::process_rss_bytes(),
        });
        if gpu.config.perf_log
            && gpu.config.perf_log_interval > 0
            && self.perf.frames() % u64::from(gpu.config.perf_log_interval) == 0
        {
            log::info!("{}", self.perf.snapshot().log_line());
        }
    }
}
