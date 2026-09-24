//! `mirror --show`: the mirrored scene in a window of its own. A thread
//! reads the watch and wakes the loop with each reply; the loop puts the
//! frames back together and draws the last one when the platform asks
//! (render on demand). The window opens at the first frame, the size of
//! the app's window in logical pixels. `b` turns the bindings on and
//! off; Escape or closing the window ends it.

use std::thread;

use engine::window::Windows;
use showcase::app::mirror::Viewer;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{WindowAttributes, WindowId};
use wire::{Client, ServerMsg};

use crate::cli::{self, Opts};

/// What the reading thread tells the loop.
enum Came {
    Line(Box<ServerMsg>),
    Ended(String),
}

struct Show {
    windows: Windows,
    id: Option<WindowId>,
    viewer: Viewer,
    bindings: bool,
    title: String,
    frames: u64,
    stop: Option<u64>,
}

pub fn run(opts: &Opts) -> Result<(), String> {
    let path = cli::socket(opts.app.as_deref())?;
    let mut c = Client::connect(&path).map_err(|e| e.to_string())?;
    c.watch_scene(&opts.window).map_err(|e| e.to_string())?;
    let event_loop = EventLoop::<Came>::with_user_event()
        .build()
        .map_err(|e| e.to_string())?;
    let proxy = event_loop.create_proxy();
    thread::Builder::new()
        .name("mirror-read".into())
        .spawn(move || {
            loop {
                match c.recv() {
                    Ok(msg) => {
                        if proxy.send_event(Came::Line(Box::new(msg))).is_err() {
                            return;
                        }
                    }
                    Err(e) => {
                        let _ = proxy.send_event(Came::Ended(e.to_string()));
                        return;
                    }
                }
            }
        })
        .map_err(|e| e.to_string())?;
    let mut show = Show {
        windows: Windows::new(),
        id: None,
        viewer: Viewer::new(),
        bindings: opts.bindings,
        title: format!("mirror: {} of {}", opts.window, path.display()),
        frames: 0,
        stop: opts.frames,
    };
    event_loop.run_app(&mut show).map_err(|e| e.to_string())
}

impl Show {
    fn redraw(&mut self) {
        if let Some(id) = self.id {
            self.windows.request_redraw(id);
        }
    }
}

impl ApplicationHandler<Came> for Show {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: Came) {
        let msg = match event {
            Came::Line(msg) => msg,
            Came::Ended(why) => {
                eprintln!("mirror: the watch ended: {why}");
                event_loop.exit();
                return;
            }
        };
        let seq = match self.viewer.take(msg.reply) {
            Ok(Some(seq)) => seq,
            Ok(None) => return,
            Err(e) => {
                eprintln!("mirror: {e}");
                event_loop.exit();
                return;
            }
        };
        self.frames += 1;
        log::debug!("frame {seq}");
        if self.id.is_none() {
            let Some(head) = self.viewer.head() else {
                return;
            };
            let attrs = WindowAttributes::default()
                .with_title(self.title.clone())
                .with_inner_size(LogicalSize::new(head.width, head.height));
            match self.windows.open(event_loop, attrs) {
                Ok(id) => self.id = Some(id),
                Err(e) => {
                    eprintln!("mirror: no window: {e}");
                    event_loop.exit();
                    return;
                }
            }
        }
        self.redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
                match event.logical_key {
                    Key::Named(NamedKey::Escape) => event_loop.exit(),
                    Key::Character(c) if c.as_str() == "b" => {
                        self.bindings = !self.bindings;
                        self.redraw();
                    }
                    _ => {}
                }
            }
            WindowEvent::Resized(size) => {
                if let Some(host) = self.windows.get_mut(id) {
                    host.resize(size.width, size.height);
                }
                self.redraw();
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                if let Some(host) = self.windows.get_mut(id) {
                    host.scale_factor_changed(scale_factor);
                }
                self.redraw();
            }
            WindowEvent::RedrawRequested => {
                let clear = self.viewer.head().map_or([0.0, 0.0, 0.0, 1.0], |h| h.clear);
                let Some(host) = self.windows.get_mut(id) else {
                    return;
                };
                host.begin_frame();
                self.viewer.draw(host.compositor_mut(), self.bindings);
                host.present(clear, "mirror");
                if self.stop.is_some_and(|n| self.frames >= n) {
                    event_loop.exit();
                }
            }
            _ => {}
        }
    }
}
