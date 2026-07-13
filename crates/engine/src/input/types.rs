use std::time::Duration;
use web_time::Instant;
use winit::event::{ElementState, MouseButton};
use winit::keyboard::NamedKey;

use super::gesture;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Started,
    Changed,
    Ended,
    Cancelled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SwipeDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Debug)]
pub struct TapEvent {
    pub position: Point,
}

#[derive(Clone, Debug)]
pub struct DoubleTapEvent {
    pub position: Point,
}

#[derive(Clone, Debug)]
pub struct LongPressEvent {
    pub position: Point,
    pub duration: Duration,
}

#[derive(Clone, Debug)]
pub struct SwipeEvent {
    pub start_position: Point,
    pub end_position: Point,
    pub direction: SwipeDirection,
    pub velocity: f64,
}

#[derive(Clone, Debug)]
pub struct DragEvent {
    pub position: Point,
    pub start_position: Point,
    pub delta: Point,
    pub phase: Phase,
}

#[derive(Clone, Debug)]
pub struct PinchEvent {
    pub center: Point,
    pub scale: f64,
    pub delta_scale: f64,
    pub phase: Phase,
}

#[derive(Clone, Debug)]
pub enum GestureEvent {
    Tap(TapEvent),
    DoubleTap(DoubleTapEvent),
    LongPress(LongPressEvent),
    Swipe(SwipeEvent),
    Drag(DragEvent),
    Pinch(PinchEvent),
}

pub const TOUCH_SLOP: f64 = 10.0;
pub const TAP_MAX_DURATION: Duration = Duration::from_millis(300);
pub const LONG_PRESS_DURATION: Duration = Duration::from_millis(500);
pub const DOUBLE_TAP_TIMEOUT: Duration = Duration::from_millis(300);
pub const DOUBLE_TAP_SLOP: f64 = 100.0;
pub const SWIPE_MIN_VEL: f64 = 200.0;
pub const SWIPE_MIN_DIST: f64 = 50.0;

pub struct TouchInputState {
    recognizer: gesture::GestureRecognizer,
}

impl Default for TouchInputState {
    fn default() -> Self {
        Self::new()
    }
}

impl TouchInputState {
    pub fn new() -> Self {
        Self {
            recognizer: gesture::GestureRecognizer::new(),
        }
    }

    pub fn handle_touch(&mut self, touch: &winit::event::Touch, now: Instant) {
        let position = Point {
            x: touch.location.x,
            y: touch.location.y,
        };
        match touch.phase {
            winit::event::TouchPhase::Started => {
                self.recognizer.touch_start(touch.id, position, now)
            }
            winit::event::TouchPhase::Moved => self.recognizer.touch_move(touch.id, position, now),
            winit::event::TouchPhase::Ended => self.recognizer.touch_end(touch.id, position, now),
            winit::event::TouchPhase::Cancelled => self.recognizer.touch_cancel(touch.id),
        }
    }

    pub fn tick(&mut self, now: Instant) {
        self.recognizer.tick(now);
    }

    pub fn drain_events(&mut self) -> Vec<GestureEvent> {
        self.recognizer.drain_events()
    }

    pub fn is_touch_active(&self) -> bool {
        self.recognizer.is_touch_active()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ViewId(pub u64);

#[derive(Clone, Debug)]
pub struct HitRegion {
    pub view_id: ViewId,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub focusable: bool,
    pub layer_visible: bool,
    pub layer_opacity: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventResponse {
    Handled,
    Ignored,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PressState {
    Pressed,
    Released,
}

impl From<ElementState> for PressState {
    fn from(state: ElementState) -> Self {
        match state {
            ElementState::Pressed => PressState::Pressed,
            ElementState::Released => PressState::Released,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointerButton {
    Primary,
    Secondary,
    Middle,
    Other(u16),
}

impl From<MouseButton> for PointerButton {
    fn from(button: MouseButton) -> Self {
        match button {
            MouseButton::Left => PointerButton::Primary,
            MouseButton::Right => PointerButton::Secondary,
            MouseButton::Middle => PointerButton::Middle,
            MouseButton::Other(id) => PointerButton::Other(id),
            _ => PointerButton::Other(0),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ModifierState {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeyInput {
    Named(NamedKey),
    Character(String),
}

#[derive(Clone, Debug)]
pub struct ClickEvent {
    pub view_id: ViewId,
    pub position: (f32, f32),
    pub button: PointerButton,
    pub state: PressState,
    pub modifiers: ModifierState,
}

#[derive(Clone, Debug)]
pub struct PlevKeyEvent {
    pub view_id: ViewId,
    pub key: KeyInput,
    pub state: PressState,
    pub text: Option<String>,
    pub modifiers: ModifierState,
    pub repeat: bool,
}

#[derive(Clone, Debug)]
pub struct HoverEvent {
    pub view_id: ViewId,
    pub position: (f32, f32),
    pub entered: bool,
}

#[derive(Clone, Debug)]
pub struct ScrollEvent {
    pub view_id: ViewId,
    pub position: (f32, f32),
    pub delta_x: f32,
    pub delta_y: f32,
    pub modifiers: ModifierState,
}

#[derive(Clone, Debug)]
pub enum InputEvent {
    Click(ClickEvent),
    Key(PlevKeyEvent),
    Hover(HoverEvent),
    Scroll(ScrollEvent),
}
