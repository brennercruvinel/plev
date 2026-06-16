mod components;
mod scene3d;
mod theme;

use std::sync::Arc;

use plev::gpu::GpuContext;
use plev::text::TextSystem;
use plev::wgpu;
use plev::winit::application::ApplicationHandler;
use plev::winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use plev::winit::event_loop::{ActiveEventLoop, EventLoop};
use plev::winit::window::{Window, WindowAttributes, WindowId};

use scene3d::{ClinicScene, OrbitCamera, Pipeline3D, Uniforms3D};

#[allow(clippy::large_enum_variant)]
enum State {
    Uninitialized,
    Ready {
        gpu: GpuContext,
        #[allow(dead_code)]
        text_system: TextSystem,
        pipeline3d: Pipeline3D,
        scene: ClinicScene,
    },
}

struct App {
    window: Option<Arc<Window>>,
    state: State,
    camera: OrbitCamera,
    mouse_pressed: bool,
    shift_held: bool,
    last_mouse: Option<(f64, f64)>,
}

impl App {
    fn new() -> Self {
        Self {
            window: None,
            state: State::Uninitialized,
            camera: OrbitCamera::new([9.0, 0.0, 7.0]), // ~center of clinic
            mouse_pressed: false,
            shift_held: false,
            last_mouse: None,
        }
    }

    fn render(&mut self) {
        self.camera.update();

        let State::Ready { ref mut gpu, ref pipeline3d, ref scene, .. } = self.state else { return };
        let Some(surface) = gpu.surface.as_ref() else { return };

        let output = match surface.get_current_texture() {
            Ok(t) => t,
            Err(_) => { gpu.resize(gpu.surface_config.width, gpu.surface_config.height); return; }
        };

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let w = gpu.surface_config.width as f32;
        let h = gpu.surface_config.height as f32;
        let aspect = w / h;

        let cam_pos = self.camera.position();
        use std::sync::atomic::{AtomicU64, Ordering};
        static FRAME: AtomicU64 = AtomicU64::new(0);
        let f = FRAME.fetch_add(1, Ordering::Relaxed);
        let vp = self.camera.view_proj_flat(aspect);
        if f < 2 {
            let p = [10.0_f32, 0.1, 10.0, 1.0];
            let clip = [
                vp[0]*p[0] + vp[4]*p[1] + vp[8]*p[2] + vp[12]*p[3],
                vp[1]*p[0] + vp[5]*p[1] + vp[9]*p[2] + vp[13]*p[3],
                vp[2]*p[0] + vp[6]*p[1] + vp[10]*p[2] + vp[14]*p[3],
                vp[3]*p[0] + vp[7]*p[1] + vp[11]*p[2] + vp[15]*p[3],
            ];
            let ndc = [clip[0]/clip[3], clip[1]/clip[3], clip[2]/clip[3]];
            eprintln!("[frame {}] ndc:[{:.3},{:.3},{:.3}] w:{:.2}", f, ndc[0], ndc[1], ndc[2], clip[3]);
        }
        let uniforms = Uniforms3D {
            view_proj: vp,
            camera_pos: [cam_pos[0], cam_pos[1], cam_pos[2], 1.0],
            light_dir: [0.5, 0.8, 0.5, 0.0],
            ambient: [0.6, 0.6, 0.6, 1.0],
            fog_color: [0.024, 0.024, 0.024, 0.001],
        };
        pipeline3d.update_uniforms(&gpu.queue, &uniforms);

        let mut encoder = gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("3d") });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("3d_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.024, g: 0.024, b: 0.024, a: 1.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &pipeline3d.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            // 1. Opaque geometry (floors)
            pass.set_pipeline(&pipeline3d.mesh_pipe);
            pass.set_bind_group(0, &pipeline3d.bind_group, &[]);
            pass.set_vertex_buffer(0, scene.opaque_vb.slice(..));
            pass.set_index_buffer(scene.opaque_ib.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..scene.opaque_count, 0, 0..1);

            // 2. Lines (grid + wireframes)
            pass.set_pipeline(&pipeline3d.line_pipe);
            pass.set_bind_group(0, &pipeline3d.bind_group, &[]);
            pass.set_vertex_buffer(0, scene.line_vb.slice(..));
            pass.draw(0..scene.line_count, 0..1);

            // 3. Transparent geometry (walls + furniture)
            pass.set_pipeline(&pipeline3d.transparent_pipe);
            pass.set_bind_group(0, &pipeline3d.bind_group, &[]);
            pass.set_vertex_buffer(0, scene.transparent_vb.slice(..));
            pass.set_index_buffer(scene.transparent_ib.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..scene.transparent_count, 0, 0..1);
        }

        gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() { return; }
        let attrs = WindowAttributes::default()
            .with_title("plev-clinic — Vista 3D")
            .with_inner_size(plev::winit::dpi::LogicalSize::new(1280, 800));
        let window = Arc::new(event_loop.create_window(attrs).unwrap());
        self.window = Some(window.clone());

        let gpu = pollster::block_on(GpuContext::new(window));
        let text_system = TextSystem::new(&gpu.device, &gpu.text_bind_group_layout);
        let fmt = gpu.surface_format();
        let w = gpu.surface_config.width;
        let h = gpu.surface_config.height;
        let pipeline3d = Pipeline3D::new(&gpu.device, fmt, w, h);
        let scene = ClinicScene::build(&gpu.device, &gpu.queue);

        self.camera = OrbitCamera::new(scene.center);

        self.state = State::Ready { gpu, text_system, pipeline3d, scene };
        if let Some(ref w) = self.window { w.request_redraw(); }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let State::Ready { ref mut gpu, ref mut pipeline3d, .. } = self.state {
                    gpu.resize(size.width, size.height);
                    pipeline3d.resize(&gpu.device, size.width, size.height);
                }
                if let Some(ref w) = self.window { w.request_redraw(); }
            }
            WindowEvent::RedrawRequested => {
                self.render();
                if let Some(ref w) = self.window { w.request_redraw(); }
            }
            WindowEvent::MouseInput { state, button: MouseButton::Left, .. } => {
                self.mouse_pressed = state == ElementState::Pressed;
                if state == ElementState::Released { self.last_mouse = None; }
            }
            WindowEvent::ModifiersChanged(mods) => {
                self.shift_held = mods.state().shift_key();
            }
            WindowEvent::CursorMoved { position, .. } => {
                let (x, y) = (position.x, position.y);
                if self.mouse_pressed
                    && let Some((lx, ly)) = self.last_mouse
                {
                    let dx = (x - lx) as f32;
                    let dy = (y - ly) as f32;
                    if self.shift_held {
                        self.camera.pan(-dx * 0.02, dy * 0.02);
                    } else {
                        self.camera.rotate(-dx * 0.005, -dy * 0.005);
                    }
                }
                self.last_mouse = Some((x, y));
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let scroll = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(p) => p.y as f32 * 0.01,
                };
                self.camera.zoom(-scroll * 2.0);
            }
            _ => {}
        }
    }
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}
