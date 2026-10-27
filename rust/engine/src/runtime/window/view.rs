//! A view the engine's own [`App`](super::App) hosts in its window, for an
//! app that is one canvas and nothing more (a graph, a chart, a scene from
//! another tool) and needs no shell of its own: the pointer reaches it as
//! widget events in logical px, it ticks while it moves (the app keeps
//! asking for frames until `tick` says it rests), and it draws each frame
//! between the compositor's `begin_frame` and its resolve. `App::with_view`
//! installs it; the uigallery and idestudio keep their own shells.

use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};

use crate::drawing::compositor::Compositor;
use crate::styling::theme::Theme;
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

pub trait HostedView {
    /// One pointer event; `bounds` is the whole window, logical px.
    fn handle_event(&mut self, event: &WidgetEvent, bounds: Rect) -> EventResult;

    /// Advances motion by `dt` seconds; `true` while something still moves.
    fn tick(&mut self, _dt: f32) -> bool {
        false
    }

    fn render(&mut self, c: &mut Compositor, bounds: Rect, theme: &Theme);
}

/// Lines a wheel notch scrolls, as widget `Scroll` deltas count them (px).
const LINE_PX: f32 = 40.0;

/// winit's pointer events, kept as widget events until the next frame.
#[derive(Default)]
pub(crate) struct Pointer {
    cursor: (f32, f32),
    pub(crate) queue: Vec<WidgetEvent>,
}

impl Pointer {
    /// Takes the pointer events of `event`; `scale` turns physical px into logical.
    pub(crate) fn take(&mut self, event: &WindowEvent, scale: f64) {
        let (x, y) = self.cursor;
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor = ((position.x / scale) as f32, (position.y / scale) as f32);
                let (x, y) = self.cursor;
                self.queue.push(WidgetEvent::MouseMove { x, y });
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                self.queue.push(match state {
                    ElementState::Pressed => WidgetEvent::MouseDown { x, y },
                    ElementState::Released => WidgetEvent::MouseUp { x, y },
                });
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let delta = match delta {
                    MouseScrollDelta::LineDelta(_, lines) => -lines * LINE_PX,
                    MouseScrollDelta::PixelDelta(p) => -(p.y / scale) as f32,
                };
                self.queue.push(WidgetEvent::Scroll { x, y, delta });
            }
            _ => {}
        }
    }
}
