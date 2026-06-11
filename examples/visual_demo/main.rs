//! Visual capabilities demo: analytic drop shadows at several blur radii,
//! 2-stop linear gradients, images from the atlas, and a clipped panel
//! whose content is larger than the panel (scissored scrolling).
//!
//! Run: `cargo run --example visual_demo`

mod scene;

use std::sync::Arc;

use plev::compositor::Compositor;
use plev::gpu::GpuContext;
use plev::text::TextSystem;
use plev::window::{encode_composite_pass, encode_layer_passes, resolve_layer_text};
use plev::winit::application::ApplicationHandler;
use plev::winit::event::WindowEvent;
use plev::winit::event_loop::{ActiveEventLoop, EventLoop};
use plev::winit::window::{Window, WindowAttributes, WindowId};
use web_time::Instant;

// Linear clear values (sRGB #171A1F linearized): the sRGB surface re-encodes on
// write, so feeding raw sRGB here would show the bg ~2.5× too light.
const BG: [f64; 3] = [0.0090, 0.0105, 0.0137];

enum AppState {
    Uninitialized,
    Ready {
        gpu: GpuContext,
        text_system: TextSystem,
        effects: plev::effects::EffectProcessor,
        texture_pool: plev::texture_pool::TexturePool,
    },
}

struct DemoApp {
    window: Option<Arc<Window>>,
    state: AppState,
    compositor: Compositor,
    started: Instant,
    logo: Option<plev::gpu::ImageHandle>,
    pattern: Option<plev::gpu::ImageHandle>,
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
        }
    }

    fn render(&mut self) {
        let AppState::Ready { .. } = self.state else {
            return;
        };

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

        self.compositor
            .resolve(&plev::compositor::ResolveResources {
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

        let mut encoder =
            gpu.device
                .create_command_encoder(&plev::wgpu::CommandEncoderDescriptor {
                    label: Some("visual_demo_encoder"),
                });

        let dirty_ids: Vec<_> = self
            .compositor
            .layers()
            .iter()
            .filter(|l| l.visible && l.is_dirty())
            .map(|l| l.id)
            .collect();

        let clear_color = plev::wgpu::Color {
            r: BG[0],
            g: BG[1],
            b: BG[2],
            a: 1.0,
        };
        encode_layer_passes(
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

        encode_composite_pass(
            &self.compositor,
            clear_color,
            gpu,
            &surface_view,
            &[],
            &mut encoder,
        );

        gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}

impl ApplicationHandler for DemoApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = WindowAttributes::default()
            .with_title("plev -- visual demo")
            .with_inner_size(plev::winit::dpi::LogicalSize::new(780, 620));
        let window = Arc::new(event_loop.create_window(attrs).unwrap());
        self.window = Some(window.clone());
        let gpu = pollster::block_on(GpuContext::new(window));
        let text_system = TextSystem::new(&gpu.device, &gpu.text_bind_group_layout);
        let effects = plev::effects::EffectProcessor::new(&gpu.device, gpu.surface_format());
        self.state = AppState::Ready {
            gpu,
            text_system,
            effects,
            texture_pool: plev::texture_pool::TexturePool::new(),
        };

        // Load images once: a real PNG through the decode path and a
        // procedural RGBA pattern.
        self.logo = plev::gpu::load_image_bytes(include_bytes!("../../assets/logo-phi.png"))
            .inspect_err(|e| log::warn!("logo load failed: {e}"))
            .ok();
        self.pattern = plev::gpu::load_image_rgba(64, 64, scene::procedural_pattern(64))
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
