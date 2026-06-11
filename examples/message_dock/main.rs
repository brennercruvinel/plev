//! MessageDock -- animated floating dock component built with Phi.
//!
//! Inspired by modern chat docks with character avatars, hover animations,
//! expand/collapse transitions, and message input -- all rendered via GPU
//! using Phi's compositor, text system, and input hit-testing.
//!
//! Run: cargo run --example message_dock
#![allow(dead_code)]

mod rendering;
mod state;
mod ui;

use plev::winit::event_loop::EventLoop;
use rendering::App;

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}
