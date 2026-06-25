//! Lottie player: renders a lottie json file with the `lot` crate on a
//! plev window. Loops at the file's native frame rate.
//!
//! Run: `cargo run --example lottie_player -- path/to/animation.json`
//! (default: ref/lottie/SNAKE). Space pauses, left/right arrows scrub
//! by half a second.

use std::sync::Arc;

use engine::animation::FrameClock;
use engine::compositor::Compositor;
use engine::gpu::{GpuContext, RenderConfig};
use engine::text::TextSystem;
use engine::window::{encode_composite_pass, encode_layer_passes, resolve_layer_text};
use engine::winit::application::ApplicationHandler;
use engine::winit::event::{ElementState, WindowEvent};
use engine::winit::event_loop::{ActiveEventLoop, EventLoop};
use engine::winit::keyboard::{Key, NamedKey};
use engine::winit::window::{Window, WindowAttributes, WindowId};
use lot::{Mat, Player};

const DEFAULT_FILE: &str = "ref/lottie/SNAKE/fd5e87b4-1189-11ee-9745-e700d1385b38.json";

// Linear clear values (sRGB graphite #171A1F linearized): the sRGB surface
// re-encodes on write, so raw sRGB here would render ~2.5x too light.
const BG: [f64; 3] = [0.0090, 0.0105, 0.0137];

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

struct LottieApp {
    window: Option<Arc<Window>>,
    state: AppState,
    compositor: Compositor,
    clock: FrameClock,
    player: Player,
    title: String,
    frame: f64,
    paused: bool,
}

impl LottieApp {
    fn new(player: Player, title: String) -> Self {
        let frame = player.anim.ip;
        Self {
            window: None,
            state: AppState::Uninitialized,
            compositor: Compositor::new(),
            clock: FrameClock::new(),
            player,
            title,
            frame,
            paused: false,
        }
    }

    fn wrap_frame(&mut self) {
        let (ip, op) = (self.player.anim.ip, self.player.anim.op);
        let dur = (op - ip).max(1.0);
        self.frame = ip + (self.frame - ip).rem_euclid(dur);
    }

    fn scrub(&mut self, seconds: f64) {
        self.frame += seconds * self.player.anim.fr;
        self.wrap_frame();
    }

    fn render(&mut self) {
        let AppState::Ready { .. } = self.state else {
            return;
        };
        let tick = self.clock.tick();
        if !self.paused {
            self.frame += f64::from(tick.dt) * self.player.anim.fr;
            self.wrap_frame();
        }

        let (vw, vh) = match &self.state {
            AppState::Ready { gpu, .. } => (
                gpu.surface_config.width as f64,
                gpu.surface_config.height as f64,
            ),
            AppState::Uninitialized => return,
        };
        let (aw, ah) = (self.player.anim.w, self.player.anim.h);
        let s = (vw / aw).min(vh / ah) * 0.95;
        let root = Mat {
            a: s,
            b: 0.0,
            c: 0.0,
            d: s,
            e: (vw - aw * s) / 2.0,
            f: (vh - ah * s) / 2.0,
        };

        self.compositor.begin_frame();
        for path in self.player.render(self.frame, root) {
            self.compositor.draw_path(path);
        }

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

        let mut encoder =
            gpu.device
                .create_command_encoder(&engine::wgpu::CommandEncoderDescriptor {
                    label: Some("lottie_player_encoder"),
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

impl ApplicationHandler for LottieApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = WindowAttributes::default()
            .with_title(self.title.clone())
            .with_inner_size(engine::winit::dpi::LogicalSize::new(640, 640));
        let window = Arc::new(event_loop.create_window(attrs).unwrap());
        self.window = Some(window.clone());
        let gpu = pollster::block_on(GpuContext::new_with_config(window, RenderConfig::default()));
        let text_system = TextSystem::new(&gpu.device, &gpu.text_bind_group_layout);
        let effects = engine::effects::EffectProcessor::new(&gpu.device, gpu.surface_format());
        self.state = AppState::Ready {
            gpu,
            text_system,
            effects,
            texture_pool: engine::gpu::texture_pool::TexturePool::new(),
        };
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
                if let Some(ref w) = self.window {
                    w.request_redraw();
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state != ElementState::Pressed {
                    return;
                }
                match event.logical_key {
                    Key::Named(NamedKey::Space) => self.paused = !self.paused,
                    Key::Named(NamedKey::ArrowLeft) => self.scrub(-0.5),
                    Key::Named(NamedKey::ArrowRight) => self.scrub(0.5),
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

fn main() {
    env_logger::init();
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_FILE.to_string());
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("cannot read {path}: {e}");
            std::process::exit(1);
        }
    };
    let anim = match lot::Animation::from_json(&text) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("cannot parse {path}: {e}");
            std::process::exit(1);
        }
    };
    log::info!(
        "lottie {}x{} frames {}..{} @ {} fps, {} layers",
        anim.w,
        anim.h,
        anim.ip,
        anim.op,
        anim.fr,
        anim.layers.len()
    );
    let title = std::path::Path::new(&path)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.clone());
    let event_loop = EventLoop::new().unwrap();
    let mut app = LottieApp::new(Player::new(anim), title);
    event_loop.run_app(&mut app).unwrap();
}
