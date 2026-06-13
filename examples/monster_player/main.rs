//! monster player: plays a .monster file with `monster::MonsterPlayer` on a plev
//! window. This binary contains zero lottie knowledge: it decodes our
//! container, unpacks the path assets, and pushes the lowered scene.
//!
//! Run: `cargo run --release --example monster_player -- file.monster`
//! Space pauses, left/right arrows scrub by half a second; the
//! animation loops.

use std::sync::Arc;

use monster::asset_path::unpack;
use monster::{AssetKind, LoweredAsset, MonsterPlayer};
use plev::animation::FrameClock;
use plev::compositor::{Compositor, QuadVertex};
use plev::gpu::{GpuContext, RenderConfig};
use plev::path::TessellatedPath;
use plev::text::TextSystem;
use plev::window::{encode_composite_pass, encode_layer_passes, resolve_layer_text};
use plev::winit::application::ApplicationHandler;
use plev::winit::event::{ElementState, WindowEvent};
use plev::winit::event_loop::{ActiveEventLoop, EventLoop};
use plev::winit::keyboard::{Key, NamedKey};
use plev::winit::window::{Window, WindowAttributes, WindowId};

// Linear clear values (sRGB graphite #171A1F linearized): the sRGB
// surface re-encodes on write, raw sRGB here would render ~2.5x light.
const BG: [f64; 3] = [0.0090, 0.0105, 0.0137];
const WINDOW: f32 = 640.0;

#[allow(clippy::large_enum_variant)]
enum AppState {
    Uninitialized,
    Ready {
        gpu: GpuContext,
        text_system: TextSystem,
        effects: plev::effects::EffectProcessor,
        texture_pool: plev::gpu::texture_pool::TexturePool,
    },
}

struct MonsterApp {
    window: Option<Arc<Window>>,
    state: AppState,
    compositor: Compositor,
    clock: FrameClock,
    player: MonsterPlayer,
    title: String,
}

impl MonsterApp {
    fn render(&mut self) {
        let AppState::Ready { .. } = self.state else {
            return;
        };
        let tick = self.clock.tick();
        self.player.tick(&tick);
        if !self.player.is_playing() {
            self.player.play(); // reached the end: loop from the start
        }

        self.compositor.begin_frame();
        for node in self.player.scene() {
            self.compositor.push(node);
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
                    label: Some("monster_player_encoder"),
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

impl ApplicationHandler for MonsterApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = WindowAttributes::default()
            .with_title(self.title.clone())
            .with_inner_size(plev::winit::dpi::LogicalSize::new(WINDOW, WINDOW));
        let window = Arc::new(event_loop.create_window(attrs).unwrap());
        self.window = Some(window.clone());
        let gpu = pollster::block_on(GpuContext::new_with_config(window, RenderConfig::default()));
        let text_system = TextSystem::new(&gpu.device, &gpu.text_bind_group_layout);
        let effects = plev::effects::EffectProcessor::new(&gpu.device, gpu.surface_format());
        self.state = AppState::Ready {
            gpu,
            text_system,
            effects,
            texture_pool: plev::gpu::texture_pool::TexturePool::new(),
        };
        self.player.play();
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
                let t = self.player.current_time();
                match event.logical_key {
                    Key::Named(NamedKey::Space) => {
                        if self.player.is_playing() {
                            self.player.pause();
                        } else {
                            self.player.play();
                        }
                    }
                    Key::Named(NamedKey::ArrowLeft) => self.player.scrub(t - 0.5),
                    Key::Named(NamedKey::ArrowRight) => self.player.scrub(t + 0.5),
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

/// Fit the stage into the window: scale and center every path asset
/// once at load. Assets are static in v0, so this is the whole layout.
fn fit(path: TessellatedPath, scale: f32, dx: f32, dy: f32) -> TessellatedPath {
    let vertices: Vec<QuadVertex> = path
        .vertices
        .iter()
        .map(|v| QuadVertex {
            position: [v.position[0] * scale + dx, v.position[1] * scale + dy],
            color: v.color,
        })
        .collect();
    TessellatedPath {
        vertices,
        indices: path.indices,
        // the payload hash is content-stable; mixing the uniform fit
        // keeps distinct assets distinct after scaling.
        hash: path.hash ^ u64::from(scale.to_bits()),
    }
}

fn main() {
    env_logger::init();
    let Some(path) = std::env::args().nth(1) else {
        eprintln!("usage: monster_player <file.monster>");
        std::process::exit(2);
    };
    let bytes = match std::fs::read(&path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("cannot read {path}: {e}");
            std::process::exit(1);
        }
    };
    let doc = match monster::decode(&bytes) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("cannot decode {path}: {e}");
            std::process::exit(1);
        }
    };
    let (sw, sh) = monster::stage_size(&doc.descs).unwrap_or((WINDOW, WINDOW));
    let scale = (WINDOW / sw).min(WINDOW / sh) * 0.95;
    let (dx, dy) = ((WINDOW - sw * scale) / 2.0, (WINDOW - sh * scale) / 2.0);
    let assets: Vec<LoweredAsset> = doc
        .assets
        .iter()
        .map(|a| match (a.kind, unpack(&a.data)) {
            (AssetKind::Path, Some(p)) => LoweredAsset::Path(fit(p, scale, dx, dy)),
            _ => LoweredAsset::Path(TessellatedPath {
                vertices: Vec::new(),
                indices: Vec::new(),
                hash: 0,
            }),
        })
        .collect();
    let mut player = match MonsterPlayer::new(doc.timeline) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("invalid timeline in {path}: {e}");
            std::process::exit(1);
        }
    };
    player.set_assets(assets);

    let kb = bytes.len() as f64 / 1024.0;
    let file = std::path::Path::new(&path)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.clone());
    let title = format!(
        "{file} - {kb:.0} KB - monster v0 ({:.1}s, {} keyframes)",
        player.duration_s(),
        player.timeline().keyframes.len()
    );
    log::info!("{title}");
    let event_loop = EventLoop::new().unwrap();
    let mut app = MonsterApp {
        window: None,
        state: AppState::Uninitialized,
        compositor: Compositor::new(),
        clock: FrameClock::new(),
        player,
        title,
    };
    event_loop.run_app(&mut app).unwrap();
}
