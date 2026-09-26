use super::host::{Gpu, WindowHost};
use crate::compositor::Layer;
use crate::theme::Theme;

/// Where a frame goes: a window's surface texture, presented at the end,
/// or a headless host's target, read back by a test.
enum Target {
    Surface(wgpu::SurfaceTexture, wgpu::TextureView),
    Offscreen(wgpu::TextureView),
}

impl WindowHost {
    /// The engine `App`'s frame of this window. It keeps its own tail
    /// instead of calling `window::draw_frame`: it composites the
    /// per-layer effects (blur and shadow, `apply_layer_effects`) between
    /// the layer passes and the composite pass, and `draw_frame`
    /// composites with none. Going through `draw_frame` needs the effects
    /// inside it, a change to the frame path every app and the headless
    /// renderer share.
    ///
    /// `theme` gives the window's clear color (the app's, shared by every
    /// window). Nothing happens without a device.
    pub(crate) fn render(&mut self, theme: &Theme) {
        // Tick frame clock (and take the pending request as served)
        self.begin_frame();

        // Bail out early if the GPU isn't ready yet
        if self.gpu.is_none() {
            return;
        }

        // begin_frame -- clear previous scene BEFORE building new one
        self.compositor.begin_frame();

        // A lost device: recover in place and draw on the new one, or skip
        // the frame (as `present_frame` does).
        if self.device_lost() && !self.recover_or_skip("render") {
            return;
        }

        #[cfg(hot_reload)]
        if let Some(gpu) = self.gpu.as_mut()
            && gpu.gpu.poll_shader_reload(&mut gpu.effects)
        {
            self.compositor.mark_all_dirty();
        }

        let Some(target) = self.acquire() else {
            return;
        };
        let Self {
            gpu:
                Some(Gpu {
                    gpu,
                    text_system,
                    effects,
                    texture_pool,
                }),
            compositor,
            perf,
            perf_hud,
            tick,
            ..
        } = self
        else {
            return;
        };
        let view = match &target {
            Target::Surface(_, view) | Target::Offscreen(view) => view,
        };

        text_system.begin_frame();

        // Perf HUD: engine-drawn overlay on its own high-z layer, fed with
        // the previous frame's snapshot (this frame's numbers land after
        // submit). Drawn before resolve so it enters this frame's passes.
        if gpu.config.perf_hud {
            let viewport_w = gpu
                .logical_size
                .map(|(w, _)| w)
                .unwrap_or(gpu.surface_config.width as f32);
            let snapshot = perf.snapshot();
            perf_hud.draw(compositor, &snapshot, viewport_w);
        } else {
            perf_hud.clear(compositor);
        }

        // Resolve compositor
        compositor.resolve(&crate::compositor::ResolveResources {
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
        super::render_passes::resolve_layer_text(compositor, gpu, text_system);

        text_system.finish_frame();

        // The accessibility tree of this frame, while a screen reader
        // listens. This `App` projects no meaning (no `sem::Snapshot`), so
        // the tree is the window alone and nothing the reader asks maps.
        #[cfg(all(feature = "accessibility", not(target_arch = "wasm32")))]
        if let (Some(bridge), Some(window)) = (self.a11y.as_mut(), self.window.as_ref()) {
            bridge.update(window, compositor, || None);
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
        let dirty_layer_ids: Vec<_> = compositor
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
            // tone instead of a washed-out ~2.5x lighter gray.
            let bg = theme.colors.bg.to_linear_array();
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
            compositor,
            gpu,
            text_system,
            effects,
            texture_pool,
            clear_color,
            &dirty_layer_ids,
            &mut encoder,
        );
        let effect_results =
            Self::apply_layer_effects(compositor, gpu, effects, texture_pool, &mut encoder);

        // Mark rendered layers clean
        for id in &dirty_layer_ids {
            compositor.mark_layer_clean(*id);
        }
        let composite_draws = super::render_passes::encode_composite_pass(
            compositor,
            clear_color,
            gpu,
            view,
            &effect_results,
            &mut encoder,
        );

        gpu.queue.submit(std::iter::once(encoder.finish()));
        if let Target::Surface(output, _) = target {
            output.present();
        }
        self.frames += 1;

        let glyphs: u32 = compositor.layers().iter().map(Layer::glyph_count).sum();
        compositor.record_encode_stats(
            layer_draws + composite_draws,
            glyphs,
            encode_start.elapsed().as_micros() as u64,
        );

        // Feed the perf monitor from the existing per-frame sources.
        perf.record_frame(*tick, compositor.stats());
        perf.record_memory(crate::perf::MemoryStats {
            glyph_atlas_bytes: text_system.atlas_memory_bytes(),
            texture_pool_bytes: texture_pool.memory_bytes(),
            layer_bytes: compositor.gpu_memory_bytes(),
            process_rss_bytes: crate::perf::process_rss_bytes(),
        });
        if gpu.config.perf_log
            && gpu.config.perf_log_interval > 0
            && perf.frames() % u64::from(gpu.config.perf_log_interval) == 0
        {
            log::info!("{}", perf.snapshot().log_line());
        }
    }

    /// The frame's target: the surface's next texture for a window (a lost
    /// or outdated swapchain is reconfigured, or, on a lost device, a
    /// frame asked for that recovers first), the target texture for a
    /// headless host. `None` skips the frame.
    fn acquire(&mut self) -> Option<Target> {
        let gpu = &mut self.gpu.as_mut()?.gpu;
        let Some(surface) = gpu.surface.as_ref() else {
            let target = self.target.as_ref()?;
            return Some(Target::Offscreen(gpu.texture_render_view(target)));
        };
        match surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(output)
            | wgpu::CurrentSurfaceTexture::Suboptimal(output) => {
                let view = gpu.surface_render_view(&output);
                Some(Target::Surface(output, view))
            }
            wgpu::CurrentSurfaceTexture::Lost | wgpu::CurrentSurfaceTexture::Outdated => {
                // a lost device loses its surface too: the next requested
                // frame recovers before acquiring
                if gpu.poll_device_loss() {
                    self.compositor.invalidate();
                } else {
                    gpu.resize(gpu.surface_config.width, gpu.surface_config.height);
                }
                None
            }
            wgpu::CurrentSurfaceTexture::Timeout => {
                log::warn!("Surface timeout");
                None
            }
            // a covered window: its `WindowEvent::Occluded(false)` draws
            // the frame when it shows again
            wgpu::CurrentSurfaceTexture::Occluded => None,
            wgpu::CurrentSurfaceTexture::Validation => {
                log::error!("Surface validation error");
                None
            }
        }
    }
}
