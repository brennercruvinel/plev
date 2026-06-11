//! Todo App -- proof of life demo using TextInput, Animation, and Compositor.
//!
//! Run: `cargo run --example todo_app`
#![allow(dead_code)]

mod rendering;
mod state;
mod ui;

use plev::winit::event_loop::EventLoop;
use state::TodoApp;

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let mut app = TodoApp::new();
    event_loop.run_app(&mut app).unwrap();
}
