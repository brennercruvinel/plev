//! Snake Game -- auto-playing AI snake built entirely on plev primitives.
//!
//! Run: `cargo run --example snake`
//!
//! Demonstrates: Rect, RoundedRect (SDF), Text, per-frame scene rebuild,
//! timer-based game tick, keyboard input, dirty tracking.
#![allow(dead_code)]

mod state;
mod ui;
mod rendering;

use plev::winit::event_loop::EventLoop;
use rendering::SnakeApp;

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let mut app = SnakeApp::new();
    event_loop.run_app(&mut app).unwrap();
}
