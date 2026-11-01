//! Per-widget behavior tests, one file per comps category, over the
//! helpers below (a shared bounds rect, pointer events, a click).

mod buttons;
mod content;
mod feedback;
mod fixtures;
mod form;
mod message;
mod nav;
mod navigation;
mod overlay;
mod selection;
mod semantics;

use crate::drawing::compositor::{Compositor, LayerId, SceneNode};
use crate::styling::theme::{Intent, Theme};
use crate::widgets::prelude::*;
use crate::widgets::widget::*;

fn hoff() -> Theme {
    Theme::hoff()
}

const B: Rect = Rect {
    x: 10.0,
    y: 10.0,
    w: 100.0,
    h: 30.0,
};

fn move_to(x: f32, y: f32) -> WidgetEvent {
    WidgetEvent::MouseMove { x, y }
}
fn down(x: f32, y: f32) -> WidgetEvent {
    WidgetEvent::MouseDown { x, y }
}
fn up(x: f32, y: f32) -> WidgetEvent {
    WidgetEvent::MouseUp { x, y }
}
fn wheel(x: f32, y: f32, delta: f32) -> WidgetEvent {
    WidgetEvent::Scroll { x, y, delta }
}

fn click(mut result: impl FnMut(&WidgetEvent) -> EventResult) -> EventResult {
    let r1 = result(&down(20.0, 20.0));
    let r2 = result(&up(20.0, 20.0));
    r1.merge(r2)
}
