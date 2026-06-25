//! Showcase application shell: the winit `ApplicationHandler` that owns the
//! window, GPU state, compositor and `ShowcaseView`, plus the per-platform
//! entry points (desktop `run`, web `run_web`, android `android_main`, iOS
//! `showcase_ios_main`). The view stays winit-free; this is the only module
//! that touches the event loop.
//!
//! Frames render on demand; animations (springs, fades) keep requesting
//! frames only while something is actually moving. On wasm GPU init is
//! async: `resumed` spawns `GpuContext::new` and the result comes back
//! through a `UserEvent::GpuReady` on the event loop proxy. On desktop and
//! mobile GPU init blocks inside `resumed` instead.

use std::sync::Arc;

use crate::view::ShowcaseView;
use crate::{keys, renderer};
use engine::animation::FrameClock;
use engine::compositor::Compositor;
use engine::gpu::GpuContext;
use engine::text::TextSystem;
use engine::ui::widgets::WidgetEvent;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
#[cfg(target_arch = "wasm32")]
use winit::event_loop::EventLoopProxy;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowAttributes, WindowId};

/// Sent by the async GPU-init task on wasm; never constructed natively
/// (desktop blocks on `GpuContext::new` inside `resumed` instead).
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
enum UserEvent {
    GpuReady {
        gpu: GpuContext,
        text_system: TextSystem,
        effects: engine::effects::EffectProcessor,
        texture_pool: engine::gpu::texture_pool::TexturePool,
    },
}

// Ready is ~2464 B vs 0 for Uninitialized; one instance lives for the
// whole process, so boxing would only add indirection on the render path.
#[allow(clippy::large_enum_variant)]
enum GpuState {
    Uninitialized,
    Ready {
        gpu: GpuContext,
        text_system: TextSystem,
        effects: engine::effects::EffectProcessor,
        texture_pool: engine::gpu::texture_pool::TexturePool,
    },
}

pub struct App {
    window: Option<Arc<Window>>,
    state: GpuState,
    compositor: Compositor,
    view: ShowcaseView,
    clock: FrameClock,
    cursor: (f32, f32),
    scale_factor: f64,
    #[cfg(target_arch = "wasm32")]
    proxy: Option<EventLoopProxy<UserEvent>>,
}

impl App {
    pub fn new() -> Self {
        let mut view = ShowcaseView::new(1200.0, 800.0);
        // Optional launch state for demos/snapshots: section, then theme.
        // (Empty iterator on wasm — no process args in the browser.)
        let mut args = std::env::args().skip(1);
        if let Some(section) = args.next() {
            view.jump_to_section(&section);
        }
        if let Some(theme) = args.next() {
            view.apply_theme(&theme);
        }
        Self {
            window: None,
            state: GpuState::Uninitialized,
            compositor: Compositor::new(),
            view,
            clock: FrameClock::new(),
            cursor: (0.0, 0.0),
            scale_factor: 1.0,
            #[cfg(target_arch = "wasm32")]
            proxy: None,
        }
    }

    fn invalidate(&mut self) {
        self.compositor.invalidate();
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }

    /// Sync surface, projection, and view layout to the window's current
    /// inner size. Called once GPU state becomes `Ready` (and harmless to
    /// call again — `gpu.resize` clamps to at least 1x1).
    fn configure_viewport(&mut self) {
        let Some(window) = self.window.clone() else {
            return;
        };
        let size = window.inner_size();
        let sf = self.scale_factor as f32;
        let (lw, lh) = (size.width as f32 / sf, size.height as f32 / sf);
        if let GpuState::Ready { gpu, .. } = &mut self.state {
            gpu.resize(size.width, size.height);
            gpu.set_projection(lw, lh);
        }
        self.view.resize(lw, lh, sf);
        self.invalidate();
    }
}

impl ApplicationHandler<UserEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        #[allow(unused_mut)] // mut needed for cfg(not(wasm)) inner_size
        let mut attrs = WindowAttributes::default().with_title("plev — design system showcase");

        // Desktop gets a fixed initial size. On the web the canvas size is
        // owned by CSS (100vw/100vh in index.html): setting an inner size
        // here would pin inline styles on the canvas and stop it from
        // tracking the browser window.
        #[cfg(not(target_arch = "wasm32"))]
        {
            attrs = attrs.with_inner_size(winit::dpi::LogicalSize::new(1200u32, 800u32));
        }

        let window = Arc::new(event_loop.create_window(attrs).unwrap());

        #[cfg(target_arch = "wasm32")]
        if let Err(e) = engine::window::setup_wasm_canvas(&window) {
            log::error!("WASM canvas setup failed: {e}");
        }

        self.window = Some(window.clone());
        self.scale_factor = window.scale_factor();

        #[cfg(not(target_arch = "wasm32"))]
        {
            let gpu = pollster::block_on(GpuContext::new(window));
            let text_system = TextSystem::new(&gpu.device, &gpu.text_bind_group_layout);
            let effects = engine::effects::EffectProcessor::new(&gpu.device, gpu.surface_format());
            self.state = GpuState::Ready {
                gpu,
                text_system,
                effects,
                texture_pool: engine::gpu::texture_pool::TexturePool::new(),
            };
            self.configure_viewport();
        }

        // Browsers forbid blocking on async GPU setup inside the event
        // loop: spawn it and hand the result back via the proxy.
        #[cfg(target_arch = "wasm32")]
        if let Some(proxy) = self.proxy.take() {
            wasm_bindgen_futures::spawn_local(async move {
                let gpu = GpuContext::new(window).await;
                let text_system = TextSystem::new(&gpu.device, &gpu.text_bind_group_layout);
                let effects =
                    engine::effects::EffectProcessor::new(&gpu.device, gpu.surface_format());
                let _ = proxy.send_event(UserEvent::GpuReady {
                    gpu,
                    text_system,
                    effects,
                    texture_pool: engine::gpu::texture_pool::TexturePool::new(),
                });
            });
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::GpuReady {
                gpu,
                text_system,
                effects,
                texture_pool,
            } => {
                log::info!("GPU context ready (async)");
                self.state = GpuState::Ready {
                    gpu,
                    text_system,
                    effects,
                    texture_pool,
                };
                // The canvas may have been resized while the adapter and
                // device were being requested; re-sync everything.
                self.configure_viewport();
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::KeyboardInput {
                event: key_event, ..
            } => {
                if key_event.state != ElementState::Pressed {
                    return;
                }
                match &key_event.logical_key {
                    Key::Named(NamedKey::Escape) => {
                        if !self.view.close_top_overlay() {
                            event_loop.exit();
                        } else {
                            self.invalidate();
                        }
                    }
                    // Space and the editing keys bridge through keys.rs
                    // (the view stays winit-free).
                    Key::Named(named) => {
                        if keys::handle_named(&mut self.view, named) {
                            self.invalidate();
                        }
                    }
                    Key::Character(c) if self.view.handle_key(c.as_str()) => {
                        self.invalidate();
                    }
                    _ => {}
                }
            }

            WindowEvent::Resized(size) => {
                let sf = self.scale_factor as f32;
                let (lw, lh) = (size.width as f32 / sf, size.height as f32 / sf);
                if let GpuState::Ready { gpu, .. } = &mut self.state {
                    gpu.resize(size.width, size.height);
                    gpu.set_projection(lw, lh);
                }
                self.view.resize(lw, lh, sf);
                self.invalidate();
            }

            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                // A Resized follows on most platforms, but invalidating here
                // is cheap and guarantees a frame on DPI changes.
                self.scale_factor = scale_factor;
                self.invalidate();
            }

            WindowEvent::CursorMoved { position, .. } => {
                let sf = self.scale_factor as f32;
                let (x, y) = (position.x as f32 / sf, position.y as f32 / sf);
                self.cursor = (x, y);
                if self.view.handle_event(&WidgetEvent::MouseMove { x, y }) {
                    self.invalidate();
                }
            }

            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => {
                let (x, y) = self.cursor;
                let ev = match state {
                    ElementState::Pressed => WidgetEvent::MouseDown { x, y },
                    ElementState::Released => WidgetEvent::MouseUp { x, y },
                };
                if self.view.handle_event(&ev) {
                    self.invalidate();
                }
            }

            WindowEvent::MouseInput {
                button: MouseButton::Right,
                state: ElementState::Pressed,
                ..
            } => {
                let (x, y) = self.cursor;
                if self.view.handle_right_click(x, y) {
                    self.invalidate();
                }
            }

            WindowEvent::MouseWheel { delta, .. } => {
                let (x, y) = self.cursor;
                let delta = match delta {
                    MouseScrollDelta::LineDelta(_, dy) => -dy * 24.0,
                    MouseScrollDelta::PixelDelta(pos) => -pos.y as f32,
                };
                if self.view.handle_event(&WidgetEvent::Scroll { x, y, delta }) {
                    self.invalidate();
                }
            }

            WindowEvent::RedrawRequested => {
                let GpuState::Ready {
                    gpu,
                    text_system,
                    effects,
                    texture_pool,
                } = &mut self.state
                else {
                    return;
                };
                let tick = self.clock.tick();
                let animating = self.view.tick(tick.dt);
                renderer::render_frame(
                    gpu,
                    text_system,
                    effects,
                    texture_pool,
                    &mut self.compositor,
                    &mut self.view,
                );
                if animating {
                    // Springs still moving: keep the frames coming.
                    self.compositor.invalidate();
                    if let Some(w) = &self.window {
                        w.request_redraw();
                    }
                }
            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {}
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

/// Desktop entry: build the winit event loop and run the showcase.
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    let event_loop = EventLoop::<UserEvent>::with_user_event().build().unwrap();
    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}

/// Browser entry: GPU init is async, so hand the proxy to the app and let
/// `spawn_app` keep the loop alive inside the browser's own event loop
/// (`run_app` would throw to escape `main`).
#[cfg(target_arch = "wasm32")]
pub fn run_web() {
    use winit::platform::web::EventLoopExtWebSys;

    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Info).expect("failed to init console_log");

    let event_loop = EventLoop::<UserEvent>::with_user_event().build().unwrap();
    let mut app = App::new();
    app.proxy = Some(event_loop.create_proxy());
    event_loop.spawn_app(app);
}

/// Android entry: GameActivity calls `android_main` on the native side. The
/// `android-game-activity` winit backend (enabled via the
/// `plev/android-game-activity` feature) provides the JNI glue; we own the
/// event loop here.
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub fn android_main(android_app: winit::platform::android::activity::AndroidApp) {
    use winit::platform::android::EventLoopBuilderExtAndroid;

    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Info)
            .with_tag("plev-showcase"),
    );

    let event_loop = EventLoop::<UserEvent>::with_user_event()
        .with_android_app(android_app)
        .build()
        .expect("android event loop");
    let mut app = App::new();
    if let Err(e) = event_loop.run_app(&mut app) {
        log::error!("event loop error: {e:?}");
    }
}

/// iOS entry: the thin Objective-C `main` in the Xcode app calls this; winit
/// drives `UIApplicationMain` from `run_app` and never returns.
#[cfg(target_os = "ios")]
#[unsafe(no_mangle)]
pub extern "C" fn showcase_ios_main() {
    let event_loop = EventLoop::<UserEvent>::with_user_event()
        .build()
        .expect("ios event loop");
    let mut app = App::new();
    let _ = event_loop.run_app(&mut app);
}
