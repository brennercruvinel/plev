use std::sync::Arc;

use winit::event_loop::ActiveEventLoop;
use winit::window::WindowAttributes;

use super::AppEvent;
use super::state::GpuState;
use crate::lifecycle::AppState;

impl super::App {
    pub(crate) fn handle_resumed(&mut self, event_loop: &ActiveEventLoop) {
        match &self.state {
            GpuState::Uninitialized => {
                #[allow(unused_mut)]
                // mut needed for cfg(feature=accessibility), cfg(desktop), cfg(ios)
                let mut attrs = WindowAttributes::default().with_title("plev \u{2014} Showcase");

                #[cfg(feature = "accessibility")]
                {
                    attrs = attrs.with_visible(false);
                }

                #[cfg(not(any(target_os = "android", target_os = "ios", target_arch = "wasm32")))]
                {
                    attrs = attrs.with_inner_size(winit::dpi::LogicalSize::new(1024, 820));
                }

                #[cfg(target_os = "ios")]
                {
                    use winit::platform::ios::WindowAttributesExtIOS;
                    attrs = attrs.with_valid_orientations(
                        winit::platform::ios::ValidOrientations::LandscapeAndPortrait,
                    );
                }

                let window = Arc::new(
                    event_loop
                        .create_window(attrs)
                        .expect("plev: failed to create window"),
                );

                #[cfg(target_arch = "wasm32")]
                if let Err(e) = super::setup_wasm_canvas(&window) {
                    log::error!("WASM canvas setup failed: {e}");
                }

                self.window = Some(window.clone());

                #[cfg(feature = "accessibility")]
                {
                    self.a11y_adapter = Some(accesskit_winit::Adapter::with_direct_handlers(
                        event_loop,
                        &window,
                        super::PlevActivationHandler,
                        super::PlevActionHandler,
                        super::PlevDeactivationHandler,
                    ));
                    self.a11y_state.activate();
                    window.set_visible(true);
                }

                self.init_gpu(window.clone());
                self.lifecycle.transition_to(AppState::Active);
                window.request_redraw();
            }
            GpuState::Suspended { .. } => {
                #[allow(unused_mut)] // mut needed for cfg(desktop), cfg(ios)
                let mut attrs = WindowAttributes::default().with_title("plev \u{2014} Showcase");

                #[cfg(not(any(target_os = "android", target_os = "ios", target_arch = "wasm32")))]
                {
                    attrs = attrs.with_inner_size(winit::dpi::LogicalSize::new(1024, 820));
                }

                #[cfg(target_os = "ios")]
                {
                    use winit::platform::ios::WindowAttributesExtIOS;
                    attrs = attrs.with_valid_orientations(
                        winit::platform::ios::ValidOrientations::LandscapeAndPortrait,
                    );
                }

                let window = Arc::new(
                    event_loop
                        .create_window(attrs)
                        .expect("plev: failed to create window"),
                );
                self.window = Some(window.clone());

                let state = std::mem::replace(&mut self.state, GpuState::Uninitialized);
                #[allow(unused_mut)] // mut needed for cfg(not(wasm)) gpu.recreate_surface()
                if let GpuState::Suspended {
                    mut gpu,
                    text_system,
                    effect_processor,
                    texture_pool,
                } = state
                {
                    #[cfg(not(target_arch = "wasm32"))]
                    gpu.recreate_surface(window.clone());
                    self.state = GpuState::Ready {
                        gpu,
                        text_system,
                        effect_processor,
                        texture_pool,
                    };
                    log::info!("Resumed from suspended state");
                    window.request_redraw();
                }
                self.lifecycle.transition_to(AppState::Active);
            }
            _ => {}
        }
    }

    pub(crate) fn handle_suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.lifecycle.transition_to(AppState::Suspended);
        let state = std::mem::replace(&mut self.state, GpuState::Uninitialized);
        if let GpuState::Ready {
            mut gpu,
            text_system,
            effect_processor,
            texture_pool,
        } = state
        {
            gpu.drop_surface();
            self.state = GpuState::Suspended {
                gpu,
                text_system,
                effect_processor,
                texture_pool,
            };
            log::info!("App suspended \u{2014} surface dropped");
        }
        self.window = None;
    }

    pub(crate) fn handle_user_event(&mut self, _event_loop: &ActiveEventLoop, event: AppEvent) {
        match event {
            AppEvent::GpuReady {
                gpu,
                text_system,
                effect_processor,
                texture_pool,
            } => {
                log::info!("GPU context ready (async)");
                self.state = GpuState::Ready {
                    gpu,
                    text_system,
                    effect_processor,
                    texture_pool,
                };
                if let Some(ref window) = self.window {
                    window.request_redraw();
                }
            }
        }
    }

    pub(crate) fn handle_memory_warning(&mut self, _event_loop: &ActiveEventLoop) {
        self.lifecycle.fire_memory_warning();
        if let GpuState::Ready {
            ref mut text_system,
            ..
        }
        | GpuState::Suspended {
            ref mut text_system,
            ..
        } = self.state
        {
            text_system.purge_caches();
        }
    }

    pub(crate) fn handle_about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        #[cfg(feature = "hot-reload")]
        {
            self.check_shader_reload();
            self.check_narrate_reload();
        }

        if let Some(ref window) = self.window {
            window.request_redraw();
        }
    }
}

/// Set up the HTML canvas for WASM rendering.
///
/// Attaches the winit canvas to `<body>` and sizes it to the viewport at the
/// device pixel ratio. Returns `Err` if any web-sys API is unavailable (e.g.
/// restricted iframe, headless context).
#[cfg(target_arch = "wasm32")]
pub(super) fn setup_wasm_canvas(window: &winit::window::Window) -> crate::error::PlevResult<()> {
    use crate::error::PlevError;
    use winit::platform::web::WindowExtWebSys;

    let canvas = window
        .canvas()
        .ok_or(PlevError::Wasm("canvas not available on window"))?;

    let doc = web_sys::window()
        .and_then(|w| w.document())
        .ok_or(PlevError::Wasm("no document available"))?;

    let body = doc
        .body()
        .ok_or(PlevError::Wasm("no <body> element in document"))?;

    body.append_child(&canvas)
        .map_err(|_| PlevError::Wasm("failed to append canvas to <body>"))?;

    let win = web_sys::window().ok_or(PlevError::Wasm("window object unavailable for sizing"))?;
    let dpr = win.device_pixel_ratio();

    let w = win
        .inner_width()
        .ok()
        .and_then(|v| v.as_f64())
        .unwrap_or(800.0);
    let h = win
        .inner_height()
        .ok()
        .and_then(|v| v.as_f64())
        .unwrap_or(600.0);

    canvas.set_width((w * dpr).round().max(1.0) as u32);
    canvas.set_height((h * dpr).round().max(1.0) as u32);

    Ok(())
}
