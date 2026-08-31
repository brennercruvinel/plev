//! GPU rendering and event loop for the Todo App example.

use std::sync::Arc;

use engine::gpu::GpuContext;
use engine::gpu::texture_pool::TexturePool;
use engine::text::TextSystem;
use engine::winit::application::ApplicationHandler;
use engine::winit::event::{ElementState, WindowEvent};
use engine::winit::event_loop::ActiveEventLoop;
use engine::winit::window::{WindowAttributes, WindowId};

use crate::state::{BG, GpuState, TodoApp};

impl TodoApp {
    pub(crate) fn submit_gpu_frame(&mut self, _vw: f32, _vh: f32) {
        let GpuState::Ready {
            ref mut gpu,
            ref mut text_system,
            ref mut pool,
        } = self.state
        else {
            return;
        };

        let _ = pool;

        let surface = match gpu.surface.as_ref() {
            Some(s) => s,
            None => return,
        };

        let output = match surface.get_current_texture() {
            Ok(t) => t,
            Err(engine::wgpu::SurfaceError::Lost | engine::wgpu::SurfaceError::Outdated) => {
                gpu.resize(gpu.surface_config.width, gpu.surface_config.height);
                return;
            }
            Err(_) => return,
        };

        let surface_view = gpu.surface_render_view(&output);
        text_system.begin_frame();

        self.compositor
            .resolve(&engine::compositor::ResolveResources {
                msaa_samples: gpu.config.msaa_samples,
                device: &gpu.device,
                queue: &gpu.queue,
                format: gpu.surface_format(),
                width: gpu.surface_config.width,
                height: gpu.surface_config.height,
                composite_bgl: &gpu.composite_bind_group_layout,
                opacity_bgl: &gpu.opacity_bind_group_layout,
                sampler: &gpu.composite_sampler,
            });

        // Resolve text per clip group so the ranges patched into the
        // draw sequence line up 1:1 with its Text commands.
        engine::window::resolve_layer_text(&mut self.compositor, gpu, text_system);
        text_system.finish_frame();

        let mut encoder =
            gpu.device
                .create_command_encoder(&engine::wgpu::CommandEncoderDescriptor {
                    label: Some("todo_encoder"),
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
            let Some(msaa_v) = layer.msaa_view() else {
                continue;
            };
            let resolve_v = layer.texture_view();
            let mut pass = encoder.begin_render_pass(&engine::wgpu::RenderPassDescriptor {
                label: Some("layer_pass"),
                color_attachments: &[Some(engine::wgpu::RenderPassColorAttachment {
                    view: msaa_v,
                    resolve_target: resolve_v,
                    ops: engine::wgpu::Operations {
                        load: engine::wgpu::LoadOp::Clear(engine::wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        }),
                        store: engine::wgpu::StoreOp::Store,
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
                pass.set_index_buffer(ib.slice(..), engine::wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..count, 0, 0..1);
            }
            if let Some((vb, ib, count)) = layer.text_buffers() {
                pass.set_pipeline(&gpu.text_pipeline);
                pass.set_bind_group(0, &gpu.projection_bind_group, &[]);
                pass.set_bind_group(1, &text_system.atlas_bind_group, &[]);
                pass.set_vertex_buffer(0, vb.slice(..));
                pass.set_index_buffer(ib.slice(..), engine::wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..count, 0, 0..1);
            }
        }

        for id in &dirty_ids {
            self.compositor.mark_layer_clean(*id);
        }

        {
            let mut pass = encoder.begin_render_pass(&engine::wgpu::RenderPassDescriptor {
                label: Some("composite_pass"),
                color_attachments: &[Some(engine::wgpu::RenderPassColorAttachment {
                    view: &surface_view,
                    resolve_target: None,
                    ops: engine::wgpu::Operations {
                        load: engine::wgpu::LoadOp::Clear({
                            let [lr, lg, lb, la] =
                                engine::color::Color::rgb(BG[0], BG[1], BG[2]).to_linear_array();
                            engine::wgpu::Color {
                                r: lr as f64,
                                g: lg as f64,
                                b: lb as f64,
                                a: la as f64,
                            }
                        }),
                        store: engine::wgpu::StoreOp::Store,
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
                if let (Some(bg), Some(opacity_bg)) =
                    (layer.composite_bind_group(), layer.opacity_bind_group())
                {
                    pass.set_bind_group(0, bg, &[]);
                    pass.set_bind_group(1, opacity_bg, &[]);
                    pass.draw(0..3, 0..1);
                }
            }
        }

        gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}

impl ApplicationHandler for TodoApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = WindowAttributes::default()
            .with_title("plev -- Todo App")
            .with_inner_size(engine::winit::dpi::LogicalSize::new(700, 600));
        let window = Arc::new(event_loop.create_window(attrs).unwrap());
        self.window = Some(window.clone());

        let gpu = pollster::block_on(GpuContext::new(window));
        let text_system = TextSystem::new(&gpu.device, &gpu.text_bind_group_layout);
        let pool = TexturePool::new();
        self.state = GpuState::Ready {
            gpu,
            text_system,
            pool,
        };
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _wid: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let GpuState::Ready { ref mut gpu, .. } = self.state {
                    gpu.resize(size.width, size.height);
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                self.handle_key_event(&event);
                if let Some(ref w) = self.window {
                    w.request_redraw();
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_pos = (position.x as f32, position.y as f32);
                self.update_hover();
                if let Some(ref w) = self.window {
                    w.request_redraw();
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: engine::winit::event::MouseButton::Left,
                ..
            } => {
                self.handle_click();
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
