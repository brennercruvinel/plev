//! GPU rendering and event loop for the MessageDock example.

use std::sync::Arc;

use plev::compositor::Compositor;
use plev::gpu::GpuContext;
use plev::input::InputState;
use plev::text::TextSystem;
use plev::winit::application::ApplicationHandler;
use plev::winit::event::WindowEvent;
use plev::winit::event_loop::ActiveEventLoop;
use plev::winit::window::{Window, WindowAttributes, WindowId};

use crate::state::{AnimatedDock, BG};

pub(crate) enum GpuState {
    Uninitialized,
    Ready {
        gpu: GpuContext,
        text_system: TextSystem,
    },
}

pub(crate) struct App {
    window: Option<Arc<Window>>,
    state: GpuState,
    compositor: Compositor,
    input_state: InputState,
    dock: AnimatedDock,
    frame: u64,
}

impl App {
    pub(crate) fn new() -> Self {
        Self {
            window: None,
            state: GpuState::Uninitialized,
            compositor: Compositor::new(),
            input_state: InputState::new(),
            dock: AnimatedDock::new(),
            frame: 0,
        }
    }

    fn render(&mut self) {
        self.frame += 1;
        self.input_state.begin_frame();
        self.dock.process_events(&mut self.input_state);
        self.compositor.begin_frame();

        let GpuState::Ready {
            ref mut gpu,
            ref mut text_system,
        } = self.state
        else {
            return;
        };
        let surface = match gpu.surface.as_ref() {
            Some(s) => s,
            None => return,
        };
        let output = match surface.get_current_texture() {
            Ok(t) => t,
            Err(_) => {
                gpu.resize(gpu.surface_config.width, gpu.surface_config.height);
                return;
            }
        };
        let view = gpu.surface_render_view(&output);
        let w = gpu.surface_config.width as f32;
        let h = gpu.surface_config.height as f32;

        text_system.begin_frame();
        self.dock.build_scene(
            &mut self.compositor,
            &mut self.input_state,
            w,
            h,
            self.frame,
        );

        self.compositor
            .resolve(&plev::compositor::ResolveResources {
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
                    label: Some("dock_encoder"),
                });
        let dirty_layer_ids: Vec<_> = self
            .compositor
            .layers()
            .iter()
            .filter(|l| l.visible && l.is_dirty())
            .map(|l| l.id)
            .collect();

        for layer_id in &dirty_layer_ids {
            let layer = self.compositor.layer(*layer_id).unwrap();
            let Some(msaa_v) = layer.msaa_view() else {
                continue;
            };
            let resolve_v = layer.texture_view();
            let mut pass = encoder.begin_render_pass(&plev::wgpu::RenderPassDescriptor {
                label: Some("layer_pass"),
                color_attachments: &[Some(plev::wgpu::RenderPassColorAttachment {
                    view: msaa_v,
                    resolve_target: resolve_v,
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
            if let Some((vb, ib, count)) = layer.text_buffers() {
                pass.set_pipeline(&gpu.text_pipeline);
                pass.set_bind_group(0, &gpu.projection_bind_group, &[]);
                pass.set_bind_group(1, &text_system.atlas_bind_group, &[]);
                pass.set_vertex_buffer(0, vb.slice(..));
                pass.set_index_buffer(ib.slice(..), plev::wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..count, 0, 0..1);
            }
        }
        for id in &dirty_layer_ids {
            self.compositor.mark_layer_clean(*id);
        }

        {
            let mut pass = encoder.begin_render_pass(&plev::wgpu::RenderPassDescriptor {
                label: Some("composite_pass"),
                color_attachments: &[Some(plev::wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: plev::wgpu::Operations {
                        load: plev::wgpu::LoadOp::Clear({
                            let [lr, lg, lb, la] =
                                plev::color::Color::rgb(BG[0], BG[1], BG[2]).to_linear_array();
                            plev::wgpu::Color {
                                r: lr as f64,
                                g: lg as f64,
                                b: lb as f64,
                                a: la as f64,
                            }
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

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = WindowAttributes::default()
            .with_title("plev -- MessageDock Demo")
            .with_inner_size(plev::winit::dpi::LogicalSize::new(960, 640));
        let window = Arc::new(event_loop.create_window(attrs).unwrap());
        self.window = Some(window.clone());
        let gpu = pollster::block_on(GpuContext::new(window));
        let text_system = TextSystem::new(&gpu.device, &gpu.text_bind_group_layout);
        self.state = GpuState::Ready { gpu, text_system };
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let GpuState::Ready { ref mut gpu, .. } = self.state {
                    gpu.resize(size.width, size.height);
                }
                if let Some(ref w) = self.window {
                    w.request_redraw();
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.input_state
                    .handle_cursor_moved(position.x as f32, position.y as f32);
                if let Some(ref w) = self.window {
                    w.request_redraw();
                }
            }
            WindowEvent::CursorLeft { .. } => {
                self.input_state.handle_cursor_left();
                if let Some(ref w) = self.window {
                    w.request_redraw();
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                self.input_state.handle_mouse_input(button, state);
                if let Some(ref w) = self.window {
                    w.request_redraw();
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                self.input_state.handle_keyboard_input(&event);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.input_state.handle_mouse_wheel(delta);
            }
            WindowEvent::ModifiersChanged(ref mods) => {
                self.input_state.handle_modifiers_changed(mods);
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
