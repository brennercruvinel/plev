// App state and event processing for the input demo.

use std::sync::Arc;

use plev::compositor::Compositor;
use plev::gpu::GpuContext;
use plev::input::{InputEvent, InputState, PressState, ViewId};
use plev::text::TextSystem;
use plev::winit::window::Window;

pub(crate) enum State {
    Uninitialized,
    Ready {
        gpu: GpuContext,
        text_system: TextSystem,
    },
}

pub(crate) struct InputDemoApp {
    pub(crate) window: Option<Arc<Window>>,
    pub(crate) state: State,
    pub(crate) compositor: Compositor,
    pub(crate) input_state: InputState,
    pub(crate) click_count: u32,
    pub(crate) button_hovered: bool,
    pub(crate) button_view_id: Option<ViewId>,
}

impl InputDemoApp {
    pub(crate) fn new() -> Self {
        Self {
            window: None,
            state: State::Uninitialized,
            compositor: Compositor::new(),
            input_state: InputState::new(),
            click_count: 0,
            button_hovered: false,
            button_view_id: None,
        }
    }

    pub(crate) fn process_events(&mut self) {
        let events = self.input_state.drain_events();
        for event in events {
            match event {
                InputEvent::Click(click) => {
                    if click.state == PressState::Pressed {
                        if let Some(btn_id) = self.button_view_id {
                            if click.view_id == btn_id {
                                self.click_count += 1;
                                log::info!("Button clicked! Count: {}", self.click_count);
                            }
                        }
                    }
                }
                InputEvent::Hover(hover) => {
                    if let Some(btn_id) = self.button_view_id {
                        if hover.view_id == btn_id {
                            self.button_hovered = hover.entered;
                        }
                    }
                }
                InputEvent::Key(key) => {
                    log::info!("Key event: {:?} state={:?}", key.key, key.state);
                }
                InputEvent::Scroll(scroll) => {
                    log::info!(
                        "Scroll: dx={:.1} dy={:.1} on view {:?}",
                        scroll.delta_x,
                        scroll.delta_y,
                        scroll.view_id
                    );
                }
            }
        }
    }
}
