//! prime number creatures, a windowed particle sandbox on the plev engine.
//!
//! N particles each carry a prime; a 250x250 coherence matrix decides which
//! attract, repel, and breathe together, so compatible primes condense into
//! emergent clusters threaded by cyan bond links, trailing through a dark
//! ocean. faithful to the "Entropic Life XVI" reference: steering physics,
//! Kuramoto phase sync, motion trails, and glow. the simulation core is pure
//! and tested (the lib target); this bin runs it and draws it through plev's
//! layer encoder. left mouse paints new particles.
//!
//! Run: `cargo run -p prime` (desktop). Web: `trunk serve` from
//! crates/prime (WebGPU only, Chrome/Edge 113+).

mod scene;

use std::sync::Arc;

use engine::color::Color;
use engine::compositor::{Compositor, ResolveResources};
use engine::effects::EffectProcessor;
use engine::gpu::GpuContext;
use engine::gpu::texture_pool::TexturePool;
use engine::text::TextSystem;
use engine::window::{encode_composite_pass, encode_layer_passes, resolve_layer_text};
use prime::sim::{Simulation, params};
use web_time::Instant;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, WindowEvent};
#[cfg(target_arch = "wasm32")]
use winit::event_loop::EventLoopProxy;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

/// the near-black ocean behind the swarm, rgba(5,5,8). linearized once at the
/// clear: the sRGB surface re-encodes on write.
const BACKGROUND: Color = Color::hex(0x050508);

/// fixed seed: the run is reproducible.
const SEED: u64 = 0xCAFE_F00D;

/// async GPU-init result, only sent on wasm. desktop blocks in `resumed`.
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
enum UserEvent {
    GpuReady {
        gpu: GpuContext,
        text_system: TextSystem,
        effects: EffectProcessor,
        texture_pool: TexturePool,
    },
}

// `Ready` carries the whole GPU context; one instance lives for the process,
// so boxing would only add indirection on the render path.
#[allow(clippy::large_enum_variant)]
enum GpuState {
    Uninitialized,
    Ready {
        gpu: GpuContext,
        text_system: TextSystem,
        effects: EffectProcessor,
        texture_pool: TexturePool,
    },
}

struct App {
    window: Option<Arc<Window>>,
    state: GpuState,
    compositor: Compositor,
    sim: Option<Simulation>,
    scale_factor: f64,
    last_tick: Instant,
    accum: f32,
    cursor: (f32, f32),
    mouse_down: bool,
    #[cfg(target_arch = "wasm32")]
    proxy: Option<EventLoopProxy<UserEvent>>,
}

impl App {
    fn new() -> Self {
        Self {
            window: None,
            state: GpuState::Uninitialized,
            compositor: Compositor::new(),
            sim: None,
            scale_factor: 1.0,
            last_tick: Instant::now(),
            accum: 0.0,
            cursor: (0.0, 0.0),
            mouse_down: false,
            #[cfg(target_arch = "wasm32")]
            proxy: None,
        }
    }

    /// logical (css-pixel) size of the window, the space the sim lives in.
    fn logical_size(&self) -> (f32, f32) {
        let Some(window) = &self.window else {
            return (1.0, 1.0);
        };
        let size = window.inner_size();
        let sf = self.scale_factor as f32;
        (
            (size.width as f32 / sf).max(1.0),
            (size.height as f32 / sf).max(1.0),
        )
    }

    /// sync the surface, the projection (logical pixels), and the sim bounds to
    /// the window. builds the sim the first time the GPU is ready.
    fn configure_viewport(&mut self) {
        let Some(window) = self.window.clone() else {
            return;
        };
        let phys = window.inner_size();
        let (lw, lh) = self.logical_size();
        if let GpuState::Ready { gpu, .. } = &mut self.state {
            gpu.resize(phys.width, phys.height);
            gpu.set_projection(lw, lh);
        }
        match &mut self.sim {
            Some(sim) => sim.resize(lw, lh),
            None => {
                self.sim = Some(Simulation::new(lw, lh, SEED));
                self.last_tick = Instant::now();
            }
        }
        window.request_redraw();
    }

    /// advance the sim on a fixed-step accumulator, then draw the field through
    /// the engine's layer encoder.
    fn render(&mut self) {
        let GpuState::Ready {
            gpu,
            text_system,
            effects,
            texture_pool,
        } = &mut self.state
        else {
            return;
        };
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

        // brush once per frame, then step the physics in fixed steps.
        if let Some(sim) = &mut self.sim {
            if self.mouse_down {
                sim.brush(self.cursor.0, self.cursor.1);
            }
            let now = Instant::now();
            let mut dt = now.duration_since(self.last_tick).as_secs_f32();
            self.last_tick = now;
            if dt > 0.1 {
                dt = 0.1;
            }
            self.accum += dt;
            let mut steps = 0;
            while self.accum >= params::FIXED_DT && steps < 5 {
                sim.step(params::FIXED_DT);
                self.accum -= params::FIXED_DT;
                steps += 1;
            }
            if steps == 5 {
                self.accum = 0.0;
            }
        }

        self.compositor.begin_frame();
        if let Some(sim) = &self.sim {
            scene::field_scene(&mut self.compositor, sim);
        }

        self.compositor.resolve(&ResolveResources {
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

        text_system.begin_frame();
        resolve_layer_text(&mut self.compositor, gpu, text_system);
        text_system.finish_frame();
        gpu.prepare_images();

        let [cr, cg, cb, ca] = BACKGROUND.to_linear_array();
        let clear_color = wgpu::Color {
            r: cr as f64,
            g: cg as f64,
            b: cb as f64,
            a: ca as f64,
        };

        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("prime_frame"),
            });

        let dirty_ids: Vec<_> = self
            .compositor
            .layers()
            .iter()
            .filter(|l| l.visible && l.is_dirty())
            .map(|l| l.id)
            .collect();

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

impl ApplicationHandler<UserEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        #[allow(unused_mut)] // mut needed only for cfg(not(wasm)) inner_size
        let mut attrs = WindowAttributes::default().with_title("prime number creatures");
        #[cfg(not(target_arch = "wasm32"))]
        {
            attrs = attrs.with_inner_size(winit::dpi::LogicalSize::new(1100u32, 760u32));
        }

        let window = Arc::new(event_loop.create_window(attrs).unwrap());

        #[cfg(target_arch = "wasm32")]
        if let Err(e) = engine::window::setup_wasm_canvas(&window) {
            log::error!("wasm canvas setup failed: {e}");
        }

        self.window = Some(window.clone());
        self.scale_factor = window.scale_factor();

        #[cfg(not(target_arch = "wasm32"))]
        {
            let gpu = pollster::block_on(GpuContext::new(window));
            let text_system = TextSystem::new(&gpu.device, &gpu.text_bind_group_layout);
            let effects = EffectProcessor::new(&gpu.device, gpu.surface_format());
            self.state = GpuState::Ready {
                gpu,
                text_system,
                effects,
                texture_pool: TexturePool::new(),
            };
            self.configure_viewport();
        }

        #[cfg(target_arch = "wasm32")]
        if let Some(proxy) = self.proxy.take() {
            wasm_bindgen_futures::spawn_local(async move {
                let gpu = GpuContext::new(window).await;
                let text_system = TextSystem::new(&gpu.device, &gpu.text_bind_group_layout);
                let effects = EffectProcessor::new(&gpu.device, gpu.surface_format());
                let _ = proxy.send_event(UserEvent::GpuReady {
                    gpu,
                    text_system,
                    effects,
                    texture_pool: TexturePool::new(),
                });
            });
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: UserEvent) {
        let UserEvent::GpuReady {
            gpu,
            text_system,
            effects,
            texture_pool,
        } = event;
        self.state = GpuState::Ready {
            gpu,
            text_system,
            effects,
            texture_pool,
        };
        self.configure_viewport();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::Resized(_) => self.configure_viewport(),

            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.scale_factor = scale_factor;
                self.configure_viewport();
            }

            WindowEvent::CursorMoved { position, .. } => {
                let sf = self.scale_factor as f32;
                self.cursor = (position.x as f32 / sf, position.y as f32 / sf);
            }

            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => {
                self.mouse_down = state == ElementState::Pressed;
            }

            WindowEvent::RedrawRequested => {
                self.render();
                // the swarm animates every frame: keep requesting frames.
                if let Some(w) = &self.window {
                    w.request_redraw();
                }
            }

            _ => {}
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    let event_loop = EventLoop::<UserEvent>::with_user_event().build().unwrap();
    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use winit::platform::web::EventLoopExtWebSys;

    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Info).expect("failed to init console_log");

    let event_loop = EventLoop::<UserEvent>::with_user_event().build().unwrap();
    let mut app = App::new();
    app.proxy = Some(event_loop.create_proxy());
    event_loop.spawn_app(app);
}
