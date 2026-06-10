//! GPU rendering and event loop for the Snake example.

use std::sync::Arc;

use plev::compositor::Compositor;
use plev::gpu::GpuContext;
use plev::text::TextSystem;
use plev::winit::application::ApplicationHandler;
use plev::winit::event::{ElementState, WindowEvent};
use plev::winit::event_loop::ActiveEventLoop;
use plev::winit::keyboard::{Key, NamedKey};
use plev::winit::window::{Window, WindowAttributes, WindowId};
use web_time::Instant;

use crate::state::{BG, SnakeGame, TICK_INTERVAL};

pub(crate) enum AppState {
    Uninitialized,
    Ready {
        gpu: GpuContext,
        text_system: TextSystem,
    },
}

pub(crate) struct SnakeApp {
    window: Option<Arc<Window>>,
    state: AppState,
    compositor: Compositor,
    game: SnakeGame,
}

impl SnakeApp {
    pub(crate) fn new() -> Self {
        Self {
            window: None,
            state: AppState::Uninitialized,
            compositor: Compositor::new(),
            game: SnakeGame::new(),
        }
    }

    fn render(&mut self) {
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

        let surface_view = gpu.surface_render_view(&output);
        let w = gpu.surface_config.width as f32;
        let h = gpu.surface_config.height as f32;

        // Game tick
        let now = Instant::now();
        if now.duration_since(self.game.last_tick).as_secs_f64() >= TICK_INTERVAL {
            self.game.tick();
            self.game.last_tick = now;
            if self.game.game_over && self.game.ai_mode {
                self.game.restart();
            }
        }

        self.compositor.begin_frame();
        text_system.begin_frame();

        self.game.build_scene(&mut self.compositor, w, h);

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

        {
            let layer_info: Vec<_> = self
                .compositor
                .layers()
                .iter()
                .map(|l| (l.id, l.is_dirty(), l.text_nodes()))
                .collect();
            for (layer_id, dirty, text_nodes) in layer_info {
                if !dirty {
                    continue;
                }
                let (vertices, indices) = text_system.resolve_for_layer(
                    &gpu.device,
                    &gpu.queue,
                    &gpu.text_bind_group_layout,
                    &text_nodes,
                );
                if let Some(layer) = self.compositor.layer_mut(layer_id) {
                    layer.set_text_data(&gpu.device, &gpu.queue, vertices, indices);
                }
            }
        }
        text_system.finish_frame();

        let mut encoder =
            gpu.device
                .create_command_encoder(&plev::wgpu::CommandEncoderDescriptor {
                    label: Some("snake_encoder"),
                });

        let dirty_ids: Vec<_> = self
            .compositor
            .layers()
            .iter()
            .filter(|l| l.visible && l.is_dirty())
            .map(|l| l.id)
            .collect();

        for layer_id in &dirty_ids {
            let layer = self.compositor.layer(*layer_id).unwrap();
            let Some((view, resolve_target)) = layer.render_attachment() else {
                continue;
            };

            let mut pass = encoder.begin_render_pass(&plev::wgpu::RenderPassDescriptor {
                label: Some("layer_pass"),
                color_attachments: &[Some(plev::wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target,
                    ops: plev::wgpu::Operations {
                        load: plev::wgpu::LoadOp::Clear(plev::wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        }),
                        store: plev::wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            if let Some((vb, ib, count)) = layer.quad_buffers() {
                pass.set_pipeline(&gpu.quad_pipeline);
                pass.set_bind_group(0, &gpu.projection_bind_group, &[]);
                pass.set_vertex_buffer(0, vb.slice(..));
                pass.set_index_buffer(ib.slice(..), plev::wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..count, 0, 0..1);
            }

            if let Some((vb, ib, count)) = layer.sdf_rect_buffers() {
                pass.set_pipeline(&gpu.rect_sdf_pipeline);
                pass.set_bind_group(0, &gpu.projection_bind_group, &[]);
                pass.set_vertex_buffer(0, vb.slice(..));
                pass.set_index_buffer(ib.slice(..), plev::wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..count, 0, 0..1);
            }

            if let Some((vb, ib, count)) = layer.text_buffers() {
                pass.set_pipeline(&gpu.text_pipeline);
                pass.set_bind_group(0, &gpu.projection_bind_group, &[]);
                pass.set_bind_group(1, &text_system.atlas_bind_group, &[]);
                pass.set_vertex_buffer(0, vb.slice(..));
                pass.set_index_buffer(ib.slice(..), plev::wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..count, 0, 0..1);
            }
        }

        for id in &dirty_ids {
            self.compositor.mark_layer_clean(*id);
        }

        {
            let mut pass = encoder.begin_render_pass(&plev::wgpu::RenderPassDescriptor {
                label: Some("composite_pass"),
                color_attachments: &[Some(plev::wgpu::RenderPassColorAttachment {
                    view: &surface_view,
                    resolve_target: None,
                    ops: plev::wgpu::Operations {
                        load: plev::wgpu::LoadOp::Clear(plev::wgpu::Color {
                            r: BG[0] as f64,
                            g: BG[1] as f64,
                            b: BG[2] as f64,
                            a: 1.0,
                        }),
                        store: plev::wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            pass.set_pipeline(&gpu.composite_pipeline);
            for layer in self.compositor.layers() {
                if !layer.visible {
                    continue;
                }
                if let (Some(cbg), Some(obg)) =
                    (layer.composite_bind_group(), layer.opacity_bind_group())
                {
                    pass.set_bind_group(0, cbg, &[]);
                    pass.set_bind_group(1, obg, &[]);
                    pass.draw(0..3, 0..1);
                }
            }
        }

        gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}

impl ApplicationHandler for SnakeApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = WindowAttributes::default()
            .with_title("plev -- Snake")
            .with_inner_size(plev::winit::dpi::LogicalSize::new(900, 700));
        let window = Arc::new(event_loop.create_window(attrs).unwrap());
        self.window = Some(window.clone());
        let gpu = pollster::block_on(GpuContext::new(window));
        let text_system = TextSystem::new(&gpu.device, &gpu.text_bind_group_layout);
        self.state = AppState::Ready { gpu, text_system };
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
            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
                match event.logical_key {
                    Key::Named(NamedKey::ArrowUp) => self.game.try_set_dir((0, -1)),
                    Key::Named(NamedKey::ArrowDown) => self.game.try_set_dir((0, 1)),
                    Key::Named(NamedKey::ArrowLeft) => self.game.try_set_dir((-1, 0)),
                    Key::Named(NamedKey::ArrowRight) => self.game.try_set_dir((1, 0)),
                    Key::Named(NamedKey::Space) => {
                        self.game.restart();
                    }
                    Key::Character(ref c) if c == "r" || c == "R" => {
                        self.game.restart();
                    }
                    Key::Character(ref c) if c == "a" || c == "A" => {
                        self.game.ai_mode = !self.game.ai_mode;
                    }
                    _ => {}
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
