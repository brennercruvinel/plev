//! Visual capabilities demo: analytic drop shadows at several blur radii,
//! 2-stop linear gradients, images from the atlas, and a clipped panel
//! whose content is larger than the panel (scissored scrolling).
//!
//! Also demonstrates the perf instrumentation: the HUD overlay starts on
//! (toggle with the `p` key) and a compact perf line logs every 120 frames
//! (`RUST_LOG=info`).
//!
//! Run: `cargo run --example visual`

mod scene;

use std::sync::Arc;

use engine::animation::FrameClock;
use engine::compositor::{Compositor, Layer};
use engine::gpu::{GpuContext, RenderConfig};
use engine::perf::{MemoryStats, PerfHud, PerfMonitor, process_rss_bytes};
use engine::text::TextSystem;
use engine::window::{encode_composite_pass, encode_layer_passes, resolve_layer_text};
use engine::winit::application::ApplicationHandler;
use engine::winit::event::{ElementState, WindowEvent};
use engine::winit::event_loop::{ActiveEventLoop, EventLoop};
use engine::winit::keyboard::Key;
use engine::winit::window::{Window, WindowAttributes, WindowId};
use web_time::Instant;

// Linear clear values (sRGB #171A1F linearized): the sRGB surface re-encodes on
// write, so feeding raw sRGB here would show the bg ~2.5× too light.
const BG: [f64; 3] = [0.0090, 0.0105, 0.0137];

// Ready is ~2464 B vs 0 for Uninitialized; one instance lives for the
// whole process, so boxing would only add indirection on the render path.
#[allow(clippy::large_enum_variant)]
enum AppState {
    Uninitialized,
    Ready {
        gpu: GpuContext,
        text_system: TextSystem,
        effects: engine::effects::EffectProcessor,
        texture_pool: engine::gpu::texture_pool::TexturePool,
    },
}

struct DemoApp {
    window: Option<Arc<Window>>,
    state: AppState,
    compositor: Compositor,
    started: Instant,
    logo: Option<engine::gpu::ImageHandle>,
    pattern: Option<engine::gpu::ImageHandle>,
    frame_clock: FrameClock,
    perf: PerfMonitor,
    hud: PerfHud,
}

impl DemoApp {
    fn new() -> Self {
        Self {
            window: None,
            state: AppState::Uninitialized,
            compositor: Compositor::new(),
            started: Instant::now(),
            logo: None,
            pattern: None,
            frame_clock: FrameClock::new(),
            perf: PerfMonitor::new(),
            hud: PerfHud::new(),
        }
    }

    fn render(&mut self) {
        let AppState::Ready { .. } = self.state else {
            return;
        };
        let tick = self.frame_clock.tick();

        // Build the scene first (immutable borrow of gpu happens below).
        self.compositor.begin_frame();
        scene::build_scene(
            &mut self.compositor,
            self.logo,
            self.pattern,
            self.started.elapsed().as_secs_f32(),
        );

        let AppState::Ready {
            ref mut gpu,
            ref mut text_system,
            ref effects,
            ref mut texture_pool,
        } = self.state
        else {
            return;
        };
        let Some(surface) = gpu.surface.as_ref() else {
            return;
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

        // Perf HUD overlay (toggled with `p`): drawn by the compositor on
        // its own layer, after the app scene and before resolve. Shows the
        // previous frame's snapshot.
        if gpu.config.perf_hud {
            let snapshot = self.perf.snapshot();
            self.hud.draw(
                &mut self.compositor,
                &snapshot,
                gpu.surface_config.width as f32,
            );
        } else {
            self.hud.clear(&mut self.compositor);
        }

        self.compositor
            .resolve(&engine::compositor::ResolveResources {
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

        resolve_layer_text(&mut self.compositor, gpu, text_system);
        text_system.finish_frame();
        gpu.prepare_images();

        let encode_start = Instant::now();
        let mut encoder =
            gpu.device
                .create_command_encoder(&engine::wgpu::CommandEncoderDescriptor {
                    label: Some("visual_encoder"),
                });

        let dirty_ids: Vec<_> = self
            .compositor
            .layers()
            .iter()
            .filter(|l| l.visible && l.is_dirty())
            .map(|l| l.id)
            .collect();

        let clear_color = engine::wgpu::Color {
            r: BG[0],
            g: BG[1],
            b: BG[2],
            a: 1.0,
        };
        let layer_draws = encode_layer_passes(
            &self.compositor,
            gpu,
            text_system,
            effects,
            texture_pool,
            clear_color,
            &dirty_ids,
            &mut encoder,
        );
        for id in &dirty_ids {
            self.compositor.mark_layer_clean(*id);
        }

        let composite_draws = encode_composite_pass(
            &self.compositor,
            clear_color,
            gpu,
            &surface_view,
            &[],
            &mut encoder,
        );

        gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        // Feed the perf monitor from the engine's own counters.
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
        self.perf.record_frame(tick, self.compositor.stats());
        self.perf.record_memory(MemoryStats {
            glyph_atlas_bytes: text_system.atlas_memory_bytes(),
            texture_pool_bytes: texture_pool.memory_bytes(),
            layer_bytes: self.compositor.gpu_memory_bytes(),
            process_rss_bytes: process_rss_bytes(),
        });
        if gpu.config.perf_log
            && gpu.config.perf_log_interval > 0
            && self.perf.frames() % u64::from(gpu.config.perf_log_interval) == 0
        {
            log::info!("{}", self.perf.snapshot().log_line());
        }
    }
}

impl ApplicationHandler for DemoApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = WindowAttributes::default()
            .with_title("plev -- visual demo")
            .with_inner_size(engine::winit::dpi::LogicalSize::new(780, 620));
        let window = Arc::new(event_loop.create_window(attrs).unwrap());
        self.window = Some(window.clone());
        // Perf instrumentation on for the demo (defaults are off): HUD
        // visible at start, compact log line every 120 frames.
        let config = RenderConfig {
            perf_hud: true,
            perf_log: true,
            ..RenderConfig::default()
        };
        let gpu = pollster::block_on(GpuContext::new_with_config(window, config));
        let text_system = TextSystem::new(&gpu.device, &gpu.text_bind_group_layout);
        let effects = engine::effects::EffectProcessor::new(&gpu.device, gpu.surface_format());
        self.state = AppState::Ready {
            gpu,
            text_system,
            effects,
            texture_pool: engine::gpu::texture_pool::TexturePool::new(),
        };

        // Load images once: a real PNG through the decode path and a
        // procedural RGBA pattern.
        self.logo = engine::gpu::load_image_bytes(include_bytes!("../../assets/logo-plev.png"))
            .inspect_err(|e| log::warn!("logo load failed: {e}"))
            .ok();
        self.pattern = engine::gpu::load_image_rgba(64, 64, scene::procedural_pattern(64))
            .inspect_err(|e| log::warn!("pattern load failed: {e}"))
            .ok();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let AppState::Ready { ref mut gpu, .. } = self.state {
                    gpu.resize(size.width, size.height);
                }
                if let Some(ref w) = self.window {
                    w.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                self.render();
                // Continuous redraw: the clipped panel scroll is animated.
                if let Some(ref w) = self.window {
                    w.request_redraw();
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                // `p` toggles the perf HUD overlay.
                if event.state == ElementState::Pressed
                    && matches!(event.logical_key.as_ref(), Key::Character("p"))
                    && let AppState::Ready { ref mut gpu, .. } = self.state
                {
                    gpu.config.perf_hud = !gpu.config.perf_hud;
                }
            }
            _ => {}
        }
    }
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let mut app = DemoApp::new();
    event_loop.run_app(&mut app).unwrap();
}
