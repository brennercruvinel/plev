//! monster player: plays a .monster file with `monster::MonsterPlayer` on a plev
//! window. This binary contains zero lottie knowledge: it decodes our
//! container, unpacks the path assets, and pushes the lowered scene.
//!
//! Run: `cargo run --release --example monster_player -- file.monster`
//! Space pauses, left/right arrows scrub by half a second; the
//! animation loops.
//!
//! Without a window: `monster_player file.monster --frame <t> --out f.png`
//! draws the frame at `t` seconds through `engine::headless` (the window's
//! frame path into an offscreen texture) at 640x640 and exits.
//!
//! `monster_player file.monster --frame <t> --capture f.png` is the check
//! that the two are one path: it opens the window at 640x640 physical
//! pixels, draws the frame at `t` into the surface with the same
//! `draw_frame`, reads the surface texture back before presenting, writes
//! it and exits. Both PNGs hash the same.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use engine::compose::animation::FrameClock;
use engine::drawing::compositor::{Compositor, QuadVertex};
use engine::drawing::gpu::{GpuContext, RenderConfig};
use engine::drawing::headless::{Renderer, Viewport, read_texture};
use engine::drawing::path::TessellatedPath;
use engine::drawing::text::TextSystem;
use engine::runtime::window::{draw_frame, present_frame};
use engine::winit::application::ApplicationHandler;
use engine::winit::event::{ElementState, WindowEvent};
use engine::winit::event_loop::{ActiveEventLoop, EventLoop};
use engine::winit::keyboard::{Key, NamedKey};
use engine::winit::window::{Window, WindowAttributes, WindowId};
use motion::encoding::asset_path::unpack;
use motion::{AssetKind, LoweredAsset, MonsterPlayer};

// Linear clear values (sRGB graphite #171A1F linearized): the sRGB
// surface re-encodes on write, raw sRGB here would render ~2.5x light.
const BG: [f32; 4] = [0.0090, 0.0105, 0.0137, 1.0];
const WINDOW: f32 = 640.0;

const USAGE: &str = "usage: monster_player <file.monster> \
                     [--frame <seconds> (--out <file.png> | --capture <file.png>)]";

#[allow(clippy::large_enum_variant)]
enum AppState {
    Uninitialized,
    Ready {
        gpu: GpuContext,
        text_system: TextSystem,
        effects: engine::drawing::effects::EffectProcessor,
        texture_pool: engine::drawing::gpu::texture_pool::TexturePool,
    },
}

/// What the binary was asked to do.
enum Mode {
    Play,
    /// Draw the frame at `t` without a window.
    Frame {
        t: f32,
        out: PathBuf,
    },
    /// Draw the frame at `t` into the window's surface and read it back.
    Capture {
        t: f32,
        out: PathBuf,
    },
}

struct MonsterApp {
    window: Option<Arc<Window>>,
    state: AppState,
    compositor: Compositor,
    clock: FrameClock,
    player: MonsterPlayer,
    title: String,
    /// `--capture`: the frame time and the PNG to write, then exit.
    capture: Option<(f32, PathBuf)>,
    captured: bool,
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
            ref mut effects,
            ref mut texture_pool,
        } = self.state
        else {
            return;
        };
        present_frame(
            &mut self.compositor,
            gpu,
            text_system,
            effects,
            texture_pool,
            BG,
            "monster_player",
        );
    }

    /// `--capture`: the frame at `t`, drawn into the surface texture by
    /// the same `draw_frame` the headless renderer runs, read back before
    /// the present.
    fn capture(&mut self, t: f32, out: &Path) -> Result<(u32, u32), String> {
        let AppState::Ready {
            ref mut gpu,
            ref mut text_system,
            ref mut effects,
            ref mut texture_pool,
        } = self.state
        else {
            return Err("the GPU is not ready".into());
        };
        self.compositor.begin_frame();
        for node in self.player.scene_at(t) {
            self.compositor.push(node);
        }
        let surface = gpu.surface.as_ref().ok_or("the window has no surface")?;
        let output = match surface.get_current_texture() {
            engine::wgpu::CurrentSurfaceTexture::Success(t)
            | engine::wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
            other => return Err(format!("surface texture: {other:?}")),
        };
        let view = gpu.surface_render_view(&output);
        draw_frame(
            &mut self.compositor,
            gpu,
            text_system,
            effects,
            texture_pool,
            &view,
            BG,
            "monster_capture",
        );
        let frame = read_texture(gpu, &output.texture).map_err(|e| e.to_string())?;
        output.present();
        frame.write_png(out).map_err(|e| e.to_string())?;
        Ok((frame.width, frame.height))
    }
}

impl ApplicationHandler for MonsterApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = WindowAttributes::default().with_title(self.title.clone());
        // A capture compares against a 640x640 headless frame, so it asks
        // for physical pixels whatever the display scale.
        let attrs = if self.capture.is_some() {
            attrs
                .with_inner_size(engine::winit::dpi::PhysicalSize::new(WINDOW, WINDOW))
                .with_resizable(false)
        } else {
            attrs.with_inner_size(engine::winit::dpi::LogicalSize::new(WINDOW, WINDOW))
        };
        let window = match event_loop.create_window(attrs) {
            Ok(w) => Arc::new(w),
            Err(e) => {
                eprintln!("cannot open a window: {e}");
                event_loop.exit();
                return;
            }
        };
        self.window = Some(window.clone());
        let mut gpu =
            pollster::block_on(GpuContext::new_with_config(window, RenderConfig::default()));
        if self.capture.is_some()
            && let Some(surface) = gpu.surface.as_ref()
        {
            // the surface texture is read back once, so it must be a copy
            // source; the drawing is unchanged
            gpu.surface_config.usage |= engine::wgpu::TextureUsages::COPY_SRC;
            surface.configure(&gpu.device, &gpu.surface_config);
        }
        let text_system = TextSystem::new(&gpu.device, &gpu.text_bind_group_layout);
        let effects =
            engine::drawing::effects::EffectProcessor::new(&gpu.device, gpu.surface_format());
        self.state = AppState::Ready {
            gpu,
            text_system,
            effects,
            texture_pool: engine::drawing::gpu::texture_pool::TexturePool::new(),
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
                if let Some((t, out)) = self.capture.clone() {
                    // redraws can arrive before the loop exits; one capture
                    if !self.captured {
                        self.captured = true;
                        match self.capture(t, &out) {
                            Ok((w, h)) => {
                                println!("wrote {} ({w}x{h}, window surface)", out.display())
                            }
                            Err(e) => eprintln!("capture failed: {e}"),
                        }
                        event_loop.exit();
                    }
                    return;
                }
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

/// `<file> [--frame <t> (--out <png> | --capture <png>)]`.
fn parse(args: &[String]) -> Result<(String, Mode), String> {
    let mut it = args.iter();
    let path = it.next().ok_or("missing <file.monster>")?.clone();
    let (mut t, mut out, mut capture) = (None, None, None);
    while let Some(flag) = it.next() {
        let value = it.next().ok_or_else(|| format!("{flag} needs a value"))?;
        match flag.as_str() {
            "--frame" => match value.parse::<f32>() {
                Ok(v) if v.is_finite() && v >= 0.0 => t = Some(v),
                _ => return Err(format!("--frame takes seconds >= 0, got '{value}'")),
            },
            "--out" => out = Some(PathBuf::from(value)),
            "--capture" => capture = Some(PathBuf::from(value)),
            other => return Err(format!("unknown argument {other}")),
        }
    }
    let mode = match (t, out, capture) {
        (None, None, None) => Mode::Play,
        (Some(t), Some(out), None) => Mode::Frame { t, out },
        (Some(t), None, Some(out)) => Mode::Capture { t, out },
        (Some(_), None, None) => return Err("--frame needs --out or --capture".into()),
        (None, _, _) => return Err("--out and --capture need --frame <t>".into()),
        (Some(_), Some(_), Some(_)) => return Err("--out and --capture exclude each other".into()),
    };
    Ok((path, mode))
}

/// `--frame --out`: the frame at `t`, no window.
fn render_headless(player: &mut MonsterPlayer, t: f32, out: &Path) -> Result<(), String> {
    let viewport = Viewport::new(WINDOW, WINDOW, 1.0);
    let mut renderer =
        Renderer::new(viewport, RenderConfig::default()).map_err(|e| e.to_string())?;
    let mut compositor = Compositor::new();
    compositor.begin_frame();
    for node in player.scene_at(t) {
        compositor.push(node);
    }
    let frame = renderer
        .render(&mut compositor, BG)
        .map_err(|e| e.to_string())?;
    frame.write_png(out).map_err(|e| e.to_string())?;
    println!(
        "wrote {} ({}x{}, t = {t}s, headless)",
        out.display(),
        frame.width,
        frame.height
    );
    Ok(())
}

fn main() {
    env_logger::init();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (path, mode) = match parse(&args) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{e}\n{USAGE}");
            std::process::exit(2);
        }
    };
    let bytes = match std::fs::read(&path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("cannot read {path}: {e}");
            std::process::exit(1);
        }
    };
    // a session .monster (magic MONS) is a recording of an app, not an
    // animation; its viewer is the session inspector.
    if bytes.starts_with(b"MONS") {
        eprintln!(
            "{path} is a recorded session (magic MONS), not an animation (MON0); \
             monster_player plays animations only. sessions open in the session \
             inspector (task-64), or read them with rec::Session::open"
        );
        std::process::exit(1);
    }
    let doc = match motion::decode(&bytes) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("cannot decode {path}: {e}");
            std::process::exit(1);
        }
    };
    let (sw, sh) = motion::stage_size(&doc.descs).unwrap_or((WINDOW, WINDOW));
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

    let capture = match mode {
        Mode::Frame { t, out } => {
            if let Err(e) = render_headless(&mut player, t, &out) {
                eprintln!("cannot render {path} at {t}s: {e}");
                std::process::exit(1);
            }
            return;
        }
        Mode::Capture { t, out } => Some((t, out)),
        Mode::Play => None,
    };

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
    let event_loop = match EventLoop::new() {
        Ok(l) => l,
        Err(e) => {
            eprintln!("cannot start the event loop: {e}");
            std::process::exit(1);
        }
    };
    let mut app = MonsterApp {
        window: None,
        state: AppState::Uninitialized,
        compositor: Compositor::new(),
        clock: FrameClock::new(),
        player,
        title,
        capture,
        captured: false,
    };
    if let Err(e) = event_loop.run_app(&mut app) {
        eprintln!("event loop: {e}");
        std::process::exit(1);
    }
}
