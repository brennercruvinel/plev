//! prs_card: renders the OUTPUT of the prs transpiler on a plev
//! window. The card function included below is the golden fixture the
//! parser emits from the react+sass corpus (crates/prs/fixtures/react)
//! verbatim; this binary proves the transpiled code is real plev code
//! that lays out, measures text and draws.
//!
//! Run: `cargo run --release --example prs_card`

use std::sync::Arc;

use plev::builder::{Element, Justify, div};
use plev::compositor::Compositor;
use plev::gpu::{GpuContext, RenderConfig};
use plev::input::InputState;
use plev::text::TextSystem;
use plev::view::ViewContext;
use plev::window::{encode_composite_pass, encode_layer_passes, resolve_layer_text};
use plev::winit::application::ApplicationHandler;
use plev::winit::event::WindowEvent;
use plev::winit::event_loop::{ActiveEventLoop, EventLoop};
use plev::winit::window::{Window, WindowAttributes, WindowId};

mod card {
    // the parser's emitted output, byte-identical to the golden test.
    include!("../../crates/prs/fixtures/react/expected.rs");
}

// Linear clear values (sRGB graphite #171A1F linearized).
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

struct CardApp {
    window: Option<Arc<Window>>,
    state: AppState,
    compositor: Compositor,
    input: InputState,
}

/// The preview slot the site fills with an image; a quiet glass block
/// stands in, so the card's own chrome is what is being judged.
fn preview() -> Element {
    let theme = plev::theme::Theme::hoff();
    div()
        .w(336.0)
        .h(336.0)
        .rounded(theme.radius.lg)
        .bg(plev::theme::hoff::n2(0.08))
        .border(1.0)
        .border_color(theme.glass.edge_soft)
}

fn scene(w: f32, h: f32) -> Element {
    div()
        .w(w)
        .h(h)
        .col()
        .align_center()
        .justify(Justify::Center)
        .child(card::hoff_research_card(
            preview(),
            "Pesquisa HOFF",
            "Este card foi transpilado do react+sass do corpus pelo prs: \
             layout, vidro, tipografia e tokens emitidos como codigo plev.",
        ))
}

impl CardApp {
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
        self.compositor.begin_frame();
        let mut cx = ViewContext::new(w, h).with_theme(plev::theme::Theme::hoff());
        plev::builder::render_element_to_compositor(
            &scene(w, h),
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
                    label: Some("prs_card_encoder"),
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

impl ApplicationHandler for CardApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = WindowAttributes::default()
            .with_title("prs: card react+sass transpilado para plev".to_string())
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
    let event_loop = EventLoop::new().unwrap();
    let mut app = CardApp {
        window: None,
        state: AppState::Uninitialized,
        compositor: Compositor::new(),
        input: InputState::new(),
    };
    event_loop.run_app(&mut app).unwrap();
}
