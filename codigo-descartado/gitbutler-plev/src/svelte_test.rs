//! svelte-test — Visual test of Plev Ui builder rendering GitButler-style components
//!
//! Run: cargo run --bin svelte-test

use std::sync::Arc;

use plev::compositor::Compositor;
use plev::gpu::GpuContext;
use plev::text::TextSystem;
use plev::texture_pool::TexturePool;
use plev::ui::{Accent, Ui, UiTheme};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowAttributes, WindowId};

enum GpuState {
    Uninitialized,
    Ready { gpu: GpuContext, text_system: TextSystem, _pool: TexturePool },
}

struct App {
    window: Option<Arc<Window>>,
    state: GpuState,
    compositor: Compositor,
}

impl App {
    fn new() -> Self {
        Self {
            window: None,
            state: GpuState::Uninitialized,
            compositor: Compositor::new(),
        }
    }

    fn render(&mut self) {
        let GpuState::Ready { gpu, text_system, .. } = &mut self.state else { return };
        let vw = gpu.surface_config.width as f32;
        let vh = gpu.surface_config.height as f32;
        let theme = UiTheme::dark();

        self.compositor.begin_frame();

        let mut ui = Ui::new(theme.clone());

        // Root layout
        let root = ui.vstack(|ui| {
            // Title bar
            let hdr = ui.hstack(|ui| {
                let t = ui.text("svelte2phi"); ui.modify(t).bold().size(16.0);
                let b = ui.badge("Ui Builder"); ui.modify(b).soft(Accent::Pop);
                ui.spacer();
                let t = ui.text("3 components"); ui.modify(t).size(12.0).color(theme.text[1]);
            });
            ui.modify(hdr).pad(16.0, 20.0).height(48.0).bg(theme.bg[1]);

            ui.separator();

            // Content
            let content = ui.vstack(|ui| {
                // -- BADGES --
                section_label(ui, "Badge");
                let row = ui.hstack(|ui| {
                    ui.badge("12");
                    let n = ui.badge("NEW"); ui.modify(n).accent(Accent::Pop);
                    let n = ui.badge("3 files"); ui.modify(n).accent(Accent::Safe);
                    let n = ui.badge("WIP"); ui.modify(n).soft(Accent::Warn);
                    let n = ui.badge("!"); ui.modify(n).accent(Accent::Danger);
                    let n = ui.badge("AI"); ui.modify(n).soft(Accent::Purple);
                });
                ui.modify(row).pad(0.0, 20.0).gap(8.0).height(32.0).align_center();

                // -- BUTTONS --
                section_label(ui, "Button");
                let row = ui.hstack(|ui| {
                    ui.button("Cancel");
                    let n = ui.button("Commit"); ui.modify(n).accent(Accent::Pop);
                    let n = ui.button("Delete"); ui.modify(n).outline(Accent::Danger);
                    let n = ui.button("Merge"); ui.modify(n).ghost(Accent::Safe);
                    let n = ui.button("AI Gen"); ui.modify(n).accent(Accent::Purple);
                    let n = ui.button("Force Push"); ui.modify(n).outline(Accent::Warn);
                });
                ui.modify(row).pad(0.0, 20.0).gap(8.0).height(40.0).align_center();

                // -- CARDS --
                section_label(ui, "CardGroupItem");
                let cards = ui.vstack(|ui| {
                    let c = ui.vstack(|ui| {
                        let t = ui.text("Project Settings"); ui.modify(t).bold().size(15.0);
                        let t = ui.text("Configure your project preferences."); ui.modify(t).size(12.0).color(theme.text[1]);
                    });
                    ui.modify(c).pad(16.0, 16.0).gap(6.0).bg(theme.bg[0]).corner(6.0).border(1.0, theme.border);

                    let c = ui.vstack(|ui| {
                        let h = ui.hstack(|ui| {
                            let t = ui.text("Git Configuration"); ui.modify(t).bold().size(15.0);
                            ui.spacer();
                            let b = ui.badge("Advanced"); ui.modify(b).soft(Accent::Purple);
                        });
                        ui.modify(h).align_center();
                        let t = ui.text("Manage branches, remotes, and hooks."); ui.modify(t).size(12.0).color(theme.text[1]);
                        let btns = ui.hstack(|ui| {
                            let n = ui.button("Edit"); ui.modify(n).accent(Accent::Pop);
                            let n = ui.button("Reset"); ui.modify(n).outline(Accent::Danger);
                        });
                        ui.modify(btns).gap(8.0);
                    });
                    ui.modify(c).pad(16.0, 16.0).gap(12.0).bg(theme.bg[0]).corner(6.0).border(1.0, theme.border);
                });
                ui.modify(cards).pad(0.0, 20.0).gap(12.0);

                // -- FILE LIST --
                section_label(ui, "Composed: File List");
                let fl = ui.vstack(|ui| {
                    file_row(ui, "M", "src/compositor.rs", Accent::Warn);
                    file_row(ui, "A", "src/ui.rs", Accent::Safe);
                    file_row(ui, "D", "src/old_layout.rs", Accent::Danger);
                    file_row(ui, "M", "Cargo.toml", Accent::Warn);
                    file_row(ui, "?", "system_prompt.md", Accent::Gray);
                });
                ui.modify(fl).pad(0.0, 20.0).bg(theme.bg[0]).corner(8.0).border(1.0, theme.border);
            });
            ui.modify(content).pad(20.0, 0.0).gap(20.0).flex(1.0);
        });
        ui.modify(root).bg(theme.bg[0]);

        ui.render(&mut self.compositor, vw, vh);

        // --- Standard Plev render pipeline ---
        let surface = match gpu.surface.as_ref() { Some(s) => s, None => return };
        let output = match surface.get_current_texture() {
            Ok(t) => t, Err(_) => return,
        };
        let surface_view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        self.compositor.resolve(&plev::compositor::ResolveResources {
            device: &gpu.device,
            queue: &gpu.queue,
            format: gpu.surface_format(),
            width: gpu.surface_config.width,
            height: gpu.surface_config.height,
            composite_bgl: &gpu.composite_bind_group_layout,
            opacity_bgl: &gpu.opacity_bind_group_layout,
            sampler: &gpu.composite_sampler,
        });

        text_system.begin_frame();
        {
            let layer_info: Vec<_> = self.compositor.layers().iter()
                .map(|l| (l.id, l.is_dirty(), l.text_nodes())).collect();
            for (lid, dirty, tnodes) in layer_info {
                if !dirty { continue; }
                let (v, i) = text_system.resolve_for_layer(&gpu.device, &gpu.queue, &gpu.text_bind_group_layout, &tnodes);
                if let Some(layer) = self.compositor.layer_mut(lid) {
                    layer.set_text_data(&gpu.device, &gpu.queue, v, i);
                }
            }
        }
        text_system.finish_frame();

        let mut enc = gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("frame") });
        let dirty_ids: Vec<_> = self.compositor.layers().iter()
            .filter(|l| l.visible && l.is_dirty()).map(|l| l.id).collect();

        for lid in &dirty_ids {
            let layer = self.compositor.layer(*lid).unwrap();
            let Some(msaa_v) = layer.msaa_view() else { continue };
            let resolve_v = layer.texture_view();
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("layer"), color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: msaa_v, resolve_target: resolve_v,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }), store: wgpu::StoreOp::Store },
                    depth_slice: None,
                })], depth_stencil_attachment: None, timestamp_writes: None, occlusion_query_set: None, multiview_mask: None,
            });
            if let Some((vb, ib, c)) = layer.quad_buffers() {
                pass.set_pipeline(&gpu.quad_pipeline);
                pass.set_bind_group(0, &gpu.projection_bind_group, &[]);
                pass.set_vertex_buffer(0, vb.slice(..));
                pass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..c, 0, 0..1);
            }
            if let Some((vb, ib, c)) = layer.sdf_rect_buffers() {
                pass.set_pipeline(&gpu.rect_sdf_pipeline);
                pass.set_bind_group(0, &gpu.projection_bind_group, &[]);
                pass.set_vertex_buffer(0, vb.slice(..));
                pass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..c, 0, 0..1);
            }
            if let Some((vb, ib, c)) = layer.text_buffers() {
                pass.set_pipeline(&gpu.text_pipeline);
                pass.set_bind_group(0, &gpu.projection_bind_group, &[]);
                pass.set_bind_group(1, &text_system.atlas_bind_group, &[]);
                pass.set_vertex_buffer(0, vb.slice(..));
                pass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..c, 0, 0..1);
            }
        }
        for id in &dirty_ids { self.compositor.mark_layer_clean(*id); }

        {
            let bg = theme.bg[0];
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("composite"), color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &surface_view, resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color { r: bg[0] as f64, g: bg[1] as f64, b: bg[2] as f64, a: 1.0 }), store: wgpu::StoreOp::Store },
                    depth_slice: None,
                })], depth_stencil_attachment: None, timestamp_writes: None, occlusion_query_set: None, multiview_mask: None,
            });
            pass.set_pipeline(&gpu.composite_pipeline);
            for layer in self.compositor.layers() {
                if !layer.visible { continue; }
                if let (Some(bg), Some(obg)) = (layer.composite_bind_group(), layer.opacity_bind_group()) {
                    pass.set_bind_group(0, bg, &[]);
                    pass.set_bind_group(1, obg, &[]);
                    pass.draw(0..3, 0..1);
                }
            }
        }
        gpu.queue.submit(std::iter::once(enc.finish()));
        output.present();
    }
}

fn section_label(ui: &mut Ui, title: &str) {
    let theme = ui.theme().clone();
    let h = ui.hstack(|ui| {
        let r = ui.rect(); ui.modify(r).width(3.0).height(16.0).bg(theme.accent_bg[1]).corner(1.5);
        let t = ui.text(title); ui.modify(t).semibold().size(13.0);
    });
    ui.modify(h).pad(0.0, 20.0).gap(8.0).height(24.0).align_center();
}

fn file_row(ui: &mut Ui, status: &str, path: &str, accent: Accent) {
    let theme = ui.theme().clone();
    let h = ui.hstack(|ui| {
        let b = ui.badge(status); ui.modify(b).accent(accent);
        let t = ui.text(path); ui.modify(t).size(13.0).color(theme.text[0]);
    });
    ui.modify(h).pad(8.0, 12.0).gap(10.0).height(36.0).align_center();
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attrs = WindowAttributes::default()
            .with_title("svelte2plev — Ui Builder Test")
            .with_inner_size(winit::dpi::LogicalSize::new(800u32, 700u32));
        let window = Arc::new(event_loop.create_window(attrs).unwrap());
        self.window = Some(window.clone());
        let gpu = pollster::block_on(GpuContext::new(window));
        let ts = TextSystem::new(&gpu.device, &gpu.text_bind_group_layout);
        let pool = TexturePool::new();
        self.state = GpuState::Ready { gpu, text_system: ts, _pool: pool };
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed {
                    if let Key::Named(NamedKey::Escape) = event.logical_key { event_loop.exit(); }
                }
            }
            WindowEvent::Resized(s) => {
                if let GpuState::Ready { gpu, .. } = &mut self.state { gpu.resize(s.width, s.height); }
            }
            WindowEvent::RedrawRequested => {
                self.render();
                if let Some(w) = &self.window { w.request_redraw(); }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        if let Some(w) = &self.window { w.request_redraw(); }
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    let event_loop = EventLoop::new().unwrap();
    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}
