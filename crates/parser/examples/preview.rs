//! Live preview: point at any react component (tsx + sass + vars) and a
//! plev window opens with the transpiled tree, interpreted at runtime
//! straight from the resolver's IR. Nothing is generated on disk and
//! nothing is hand-written per component: what you see is the parse.
//!
//! Run: `cargo run -p parser --example preview -- <index.tsx> <module.sass> <vars.sass>`

use std::sync::Arc;

use parser::ir::{Arg, ParserNode, Tag, TextValue};
use plev::builder::{Element, Justify, div, text};
use plev::color::Color;
use plev::compositor::Compositor;
use plev::gpu::{GpuContext, RenderConfig};
use plev::input::InputState;
use plev::text::TextSystem;
use plev::theme::Theme;
use plev::view::ViewContext;
use plev::window::{encode_composite_pass, encode_layer_passes, resolve_layer_text};
use plev::winit::application::ApplicationHandler;
use plev::winit::event::WindowEvent;
use plev::winit::event_loop::{ActiveEventLoop, EventLoop};
use plev::winit::window::{Window, WindowAttributes, WindowId};

const BG: [f64; 3] = [0.0090, 0.0105, 0.0137];

/// Evaluate one emitted token expression against the live theme. The
/// token vocabulary is the closed set css_map produces; anything new
/// falls back loudly (magenta) instead of silently.
fn color_of(theme: &Theme, token: &str) -> Color {
    let call =
        |s: &str| -> Option<f32> { s.split_once('(')?.1.strip_suffix(')')?.trim().parse().ok() };
    match token {
        "theme.colors.text" => theme.colors.text,
        "theme.colors.text_mid" => theme.colors.text_mid,
        "theme.colors.text_dim" => theme.colors.text_dim,
        "theme.glass.inset_highlight" => theme.glass.inset_highlight,
        "theme.glass.edge_soft" => theme.glass.edge_soft,
        "theme.glass.edge" => theme.glass.edge,
        "theme.glass.button" => theme.glass.button,
        "plev::theme::hoff::CARD_OVERLAY" => plev::theme::hoff::CARD_OVERLAY,
        t if t.starts_with("plev::theme::hoff::n2(") => {
            plev::theme::hoff::n2(call(t).unwrap_or(1.0))
        }
        t if t.starts_with("plev::theme::hoff::n3(") => {
            plev::theme::hoff::n3(call(t).unwrap_or(1.0))
        }
        t if t.starts_with("plev::color::Color::rgba(") => {
            let inner = t.split_once('(').map(|(_, r)| r.trim_end_matches(')'));
            let v: Vec<f32> = inner
                .unwrap_or("")
                .split(',')
                .filter_map(|x| x.trim().parse().ok())
                .collect();
            match v.as_slice() {
                [r, g, b, a] => Color::rgba(*r, *g, *b, *a),
                _ => Color::rgba(1.0, 0.0, 1.0, 1.0),
            }
        }
        other => {
            eprintln!("preview: token desconhecido {other}, magenta");
            Color::rgba(1.0, 0.0, 1.0, 1.0)
        }
    }
}

fn f32_of(theme: &Theme, arg: &Arg) -> f32 {
    match arg {
        Arg::F32(v) => *v,
        Arg::Int(i) => *i as f32,
        Arg::Token(t) => match t.as_str() {
            "theme.radius.xl" => theme.radius.xl,
            "theme.effects.blur_sigma" => theme.effects.blur_sigma,
            other => {
                eprintln!("preview: token escalar desconhecido {other}, 0");
                0.0
            }
        },
        Arg::Pair(_) => 0.0,
    }
}

/// Apply one resolved prop to the element: the runtime twin of how
/// emit.rs prints `.name(args)`.
fn apply(el: Element, theme: &Theme, name: &str, args: &[Arg]) -> Element {
    let f = |i: usize| args.get(i).map_or(0.0, |a| f32_of(theme, a));
    let c = |i: usize| match args.get(i) {
        Some(Arg::Token(t)) => color_of(theme, t),
        _ => Color::rgba(1.0, 0.0, 1.0, 1.0),
    };
    match name {
        "w" => el.w(f(0)),
        "h" => el.h(f(0)),
        "p" => el.p(f(0)),
        "pt" => el.pt(f(0)),
        "pb" => el.pb(f(0)),
        "pl" => el.pl(f(0)),
        "pr" => el.pr(f(0)),
        "px" => el.px(f(0)),
        "py" => el.py(f(0)),
        "bg" => el.bg(c(0)),
        "rounded" => el.rounded(f(0)),
        "border" => el.border(f(0)),
        "border_color" => el.border_color(c(0)),
        "clip_children" => el.clip_children(),
        "backdrop_blur" => el.backdrop_blur(f(0)),
        "font_size" => el.font_size(f(0)),
        "line_height" => el.line_height(f(0)),
        "font_weight" => el.font_weight(f(0) as u16),
        "text_color" => el.text_color(c(0)),
        "row" => el.row(),
        "align_center" => el.align_center(),
        "justify" => el.justify(Justify::Center),
        "grow" => el.grow(if args.is_empty() { 1.0 } else { f(0) }),
        "shrink" => el.shrink(if args.is_empty() { 0.0 } else { f(0) }),
        "shadow_inset" => match args {
            [blur, Arg::Pair(off), Arg::Token(t)] => {
                el.shadow_inset(f32_of(theme, blur), *off, color_of(theme, t))
            }
            _ => el,
        },
        "shadow_drop" => match args {
            [blur, Arg::Pair(off), Arg::Token(t)] => {
                el.shadow_drop(f32_of(theme, blur), off[1], color_of(theme, t))
            }
            _ => el,
        },
        other => {
            eprintln!("preview: prop {other} sem interprete, ignorada");
            el
        }
    }
}

/// ParserNode -> Element, recursively. Params render as their own name,
/// slots as a quiet glass block: the component's chrome is the subject.
fn build(node: &ParserNode, theme: &Theme) -> Element {
    let mut el = match &node.tag {
        Tag::Div => div(),
        Tag::Text(TextValue::Literal(s)) => text(s),
        Tag::Text(TextValue::Param(p)) => text(&format!("\u{ab}{p}\u{bb}")),
        Tag::Slot(name) => {
            eprintln!("preview: slot {name} vira bloco neutro");
            div()
                .min_h(64.0)
                .rounded(12.0)
                .bg(plev::theme::hoff::n2(0.06))
        }
    };
    for prop in &node.props {
        el = apply(el, theme, prop.name, &prop.args);
    }
    for child in &node.children {
        el = el.child(build(child, theme));
    }
    el
}

enum AppState {
    Uninitialized,
    Ready {
        gpu: GpuContext,
        text_system: TextSystem,
        effects: plev::effects::EffectProcessor,
        texture_pool: plev::texture_pool::TexturePool,
    },
}

struct PreviewApp {
    window: Option<Arc<Window>>,
    state: AppState,
    compositor: Compositor,
    input: InputState,
    root: ParserNode,
    title: String,
}

impl PreviewApp {
    fn render(&mut self) {
        let AppState::Ready {
            ref mut gpu,
            ref mut text_system,
            ref effects,
            ref mut texture_pool,
        } = self.state
        else {
            return;
        };
        let sf = self.window.as_ref().map_or(1.0, |w| w.scale_factor()) as f32;
        let (w, h) = (
            gpu.surface_config.width as f32 / sf,
            gpu.surface_config.height as f32 / sf,
        );
        let theme = Theme::hoff();
        let scene = div()
            .w(w)
            .h(h)
            .col()
            .align_center()
            .justify(Justify::Center)
            .child(build(&self.root, &theme));
        self.compositor.begin_frame();
        let mut cx = ViewContext::new(w, h).with_theme(theme);
        plev::builder::render_element_to_compositor(
            &scene,
            &mut self.compositor,
            &mut self.input,
            &mut cx,
        );

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
                    label: Some("parser_preview_encoder"),
                });
        let dirty: Vec<_> = self
            .compositor
            .layers()
            .iter()
            .filter(|l| l.visible && l.is_dirty())
            .map(|l| l.id)
            .collect();
        let clear = plev::wgpu::Color {
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
            clear,
            &dirty,
            &mut encoder,
        );
        for id in &dirty {
            self.compositor.mark_layer_clean(*id);
        }
        encode_composite_pass(
            &self.compositor,
            clear,
            gpu,
            &surface_view,
            &[],
            &mut encoder,
        );
        gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}

impl ApplicationHandler for PreviewApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = WindowAttributes::default()
            .with_title(self.title.clone())
            .with_inner_size(plev::winit::dpi::LogicalSize::new(560.0, 760.0));
        let window = Arc::new(event_loop.create_window(attrs).unwrap());
        self.window = Some(window.clone());
        let gpu = pollster::block_on(GpuContext::new_with_config(window, RenderConfig::default()));
        let text_system = TextSystem::new(&gpu.device, &gpu.text_bind_group_layout);
        let effects = plev::effects::EffectProcessor::new(&gpu.device, gpu.surface_format());
        self.state = AppState::Ready {
            gpu,
            text_system,
            effects,
            texture_pool: plev::texture_pool::TexturePool::new(),
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
            _ => {}
        }
    }
}

fn main() {
    env_logger::init();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [tsx, sass, vars] = args.as_slice() else {
        eprintln!("uso: preview <index.tsx> <module.sass> <vars.sass>");
        std::process::exit(2);
    };
    let read = |p: &String| {
        std::fs::read_to_string(p).unwrap_or_else(|e| {
            eprintln!("nao li {p}: {e}");
            std::process::exit(1);
        })
    };
    let name = |p: &String| {
        std::path::Path::new(p)
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| p.clone())
    };
    let component = match parser::tsx::parse_tsx(&read(tsx)) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("tsx: {e}");
            std::process::exit(1);
        }
    };
    let rules = parser::sass::parse_sass(&read(sass), &read(vars));
    let res = parser::resolve_react::resolve_react(&component, &rules, &name(tsx), &name(sass));
    let title = format!(
        "parser preview: {} - {} mapeadas, {} droplist",
        res.fn_name,
        res.mapped,
        res.dropped.len()
    );
    eprintln!("{title}");
    let event_loop = EventLoop::new().unwrap();
    let mut app = PreviewApp {
        window: None,
        state: AppState::Uninitialized,
        compositor: Compositor::new(),
        input: InputState::new(),
        root: res.root.clone(),
        title,
    };
    event_loop.run_app(&mut app).unwrap();
}
