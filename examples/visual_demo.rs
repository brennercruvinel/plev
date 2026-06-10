//! Visual capabilities demo: analytic drop shadows at several blur radii,
//! 2-stop linear gradients, images from the atlas, and a clipped panel
//! whose content is larger than the panel (scissored scrolling).
//!
//! Run: `cargo run --example visual_demo`

use std::sync::Arc;

use plev::compositor::{
    Compositor, GradientRectParams, RoundedRectParams, ShadowParams, TextNodeKey,
};
use plev::gpu::GpuContext;
use plev::text::TextSystem;
use plev::window::{encode_composite_pass, encode_layer_passes, resolve_layer_text};
use plev::winit::application::ApplicationHandler;
use plev::winit::event::WindowEvent;
use plev::winit::event_loop::{ActiveEventLoop, EventLoop};
use plev::winit::window::{Window, WindowAttributes, WindowId};
use web_time::Instant;

const BG: [f64; 3] = [0.09, 0.10, 0.12];
const TEXT: [f32; 4] = [0.92, 0.93, 0.95, 1.0];
const MUTED: [f32; 4] = [0.55, 0.58, 0.64, 1.0];
const CARD: [f32; 4] = [0.16, 0.17, 0.21, 1.0];

enum AppState {
    Uninitialized,
    Ready {
        gpu: GpuContext,
        text_system: TextSystem,
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

    fn label(&mut self, text: &str, size: f32, x: f32, y: f32, color: [f32; 4]) {
        self.compositor
            .draw_text(TextNodeKey::new(text, size, size * 1.3, None), x, y, color);
    }

    fn build_scene(&mut self, _w: f32, _h: f32) {
        let elapsed = self.started.elapsed().as_secs_f32();

        self.label("plev renderer -- visual capabilities", 22.0, 32.0, 24.0, TEXT);

        // -- Analytic shadows at several blur radii --------------------------
        self.label("analytic shadows (blur 4 / 12 / 24 / 48)", 14.0, 32.0, 68.0, MUTED);
        for (i, blur) in [4.0f32, 12.0, 24.0, 48.0].into_iter().enumerate() {
            let x = 32.0 + i as f32 * 180.0;
            let y = 100.0;
            self.compositor.draw_shadow(ShadowParams {
                x,
                y,
                w: 150.0,
                h: 90.0,
                corner_radius: 12.0,
                blur_radius: blur,
                offset: [0.0, 6.0],
                color: [0.0, 0.0, 0.0, 0.55],
            });
            self.compositor.draw_rounded_rect(RoundedRectParams {
                x,
                y,
                w: 150.0,
                h: 90.0,
                color: CARD,
                corner_radius: 12.0,
                border_width: 1.0,
                border_color: [1.0, 1.0, 1.0, 0.08],
            });
            self.label(&format!("blur {blur}"), 13.0, x + 16.0, y + 36.0, TEXT);
        }

        // -- Linear gradients -------------------------------------------------
        self.label("linear gradients (0 / 45 / 90 / 135 deg)", 14.0, 32.0, 230.0, MUTED);
        let stops = [
            ([0.98, 0.45, 0.25, 1.0], [0.85, 0.20, 0.55, 1.0], 0.0),
            ([0.20, 0.55, 0.95, 1.0], [0.25, 0.90, 0.75, 1.0], 45.0),
            ([0.55, 0.30, 0.95, 1.0], [0.95, 0.40, 0.45, 1.0], 90.0),
            ([0.95, 0.75, 0.25, 1.0], [0.90, 0.30, 0.30, 1.0], 135.0),
        ];
        for (i, (from, to, angle)) in stops.into_iter().enumerate() {
            let x = 32.0 + i as f32 * 180.0;
            self.compositor.draw_gradient_rect(GradientRectParams {
                x,
                y: 262.0,
                w: 150.0,
                h: 70.0,
                color: from,
                color2: to,
                angle_deg: angle,
                corner_radius: 10.0,
                border_width: 0.0,
                border_color: [0.0; 4],
            });
        }

        // -- Images from the atlas -------------------------------------------
        self.label("image atlas (png decode + procedural rgba)", 14.0, 32.0, 356.0, MUTED);
        if let Some(logo) = self.logo {
            // Keep the aspect ratio at a fixed display height.
            let h = 96.0;
            let w = h * logo.width as f32 / logo.height as f32;
            self.compositor.draw_image(32.0, 388.0, w, h, logo, 8.0);
        }
        if let Some(pattern) = self.pattern {
            self.compositor.draw_image(160.0, 388.0, 96.0, 96.0, pattern, 48.0);
            self.compositor.draw_image(272.0, 388.0, 96.0, 96.0, pattern, 12.0);
        }

        // -- Clipped panel with oversized content ------------------------------
        let (px, py, pw, ph) = (420.0, 388.0, 300.0, 180.0);
        self.label("clip stack (content larger than panel)", 14.0, px, 356.0, MUTED);
        self.compositor.draw_rounded_rect(RoundedRectParams {
            x: px,
            y: py,
            w: pw,
            h: ph,
            color: [0.13, 0.14, 0.17, 1.0],
            corner_radius: 8.0,
            border_width: 1.0,
            border_color: [1.0, 1.0, 1.0, 0.10],
        });

        // Oscillating scroll offset shows rows being cut at the edges.
        let scroll = ((elapsed * 0.7).sin() * 0.5 + 0.5) * 220.0;
        self.compositor.push_clip(px, py, pw, ph);
        for row in 0..14 {
            let y = py + 8.0 + row as f32 * 28.0 - scroll;
            // Wider than the panel on purpose: the right edge is clipped too.
            self.compositor.draw_rounded_rect(RoundedRectParams {
                x: px + 8.0,
                y,
                w: pw + 60.0,
                h: 22.0,
                color: if row % 2 == 0 {
                    [0.20, 0.22, 0.27, 1.0]
                } else {
                    [0.17, 0.18, 0.22, 1.0]
                },
                corner_radius: 4.0,
                border_width: 0.0,
                border_color: [0.0; 4],
            });
            self.label(
                &format!("row {row} -- clipped to the panel bounds"),
                12.0,
                px + 16.0,
                y + 4.0,
                TEXT,
            );
        }
        self.compositor.pop_clip();
    }

    fn render(&mut self) {
        let AppState::Ready { .. } = self.state else {
            return;
        };

        // Build the scene first (immutable borrow of gpu happens below).
        self.compositor.begin_frame();
        let (w, h) = match &self.state {
            AppState::Ready { gpu, .. } => (
                gpu.surface_config.width as f32,
                gpu.surface_config.height as f32,
            ),
            AppState::Uninitialized => return,
        };
        self.build_scene(w, h);

        let AppState::Ready {
            ref mut gpu,
            ref mut text_system,
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
        let surface_view = output
            .texture
            .create_view(&plev::wgpu::TextureViewDescriptor::default());

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

        let mut encoder = gpu
            .device
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

        encode_layer_passes(&self.compositor, gpu, text_system, &dirty_ids, &mut encoder);
        for id in &dirty_ids {
            self.compositor.mark_layer_clean(*id);
        }

        encode_composite_pass(
            &self.compositor,
            plev::wgpu::Color {
                r: BG[0],
                g: BG[1],
                b: BG[2],
                a: 1.0,
            },
            gpu,
            &surface_view,
            &[],
            &mut encoder,
        );

        gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}

/// Procedural test image: radial color wheel with an alpha falloff.
fn procedural_pattern(size: u32) -> Vec<u8> {
    let mut pixels = Vec::with_capacity((size * size * 4) as usize);
    let center = size as f32 / 2.0;
    for y in 0..size {
        for x in 0..size {
            let (dx, dy) = (x as f32 - center, y as f32 - center);
            let dist = (dx * dx + dy * dy).sqrt() / center;
            let angle = dy.atan2(dx);
            let r = (angle.sin() * 0.5 + 0.5) * 255.0;
            let g = ((angle + 2.1).sin() * 0.5 + 0.5) * 255.0;
            let b = ((angle + 4.2).sin() * 0.5 + 0.5) * 255.0;
            let a = ((1.0 - dist).clamp(0.0, 1.0) * 255.0 * 1.5).min(255.0);
            pixels.extend_from_slice(&[r as u8, g as u8, b as u8, a as u8]);
        }
    }
    pixels
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
        self.state = AppState::Ready { gpu, text_system };

        // Load images once: a real PNG through the decode path and a
        // procedural RGBA pattern.
        self.logo = plev::gpu::load_image_bytes(include_bytes!("../assets/logo-φ.png"))
            .inspect_err(|e| log::warn!("logo load failed: {e}"))
            .ok();
        self.pattern = plev::gpu::load_image_rgba(64, 64, procedural_pattern(64))
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
