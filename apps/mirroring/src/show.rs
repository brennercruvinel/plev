//! `mirror --show`: the mirrored scene in a window of its own. A thread
//! reads the watch and wakes the loop with each reply; the loop puts the
//! frames back together and draws the last one when the platform asks
//! (render on demand). The window opens at the first frame, the size of
//! the app's window in logical pixels. `b` turns the bindings on and
//! off; Escape or closing the window ends it.
//!
//! A click goes back to the app as an act (task-79 phase 2): the loop
//! finds the control under the pointer in the frame shown
//! (`Viewer::hit`) and hands it to the hand's thread, which presses it
//! (`Hand::press`) on a connection of its own, under the same name, and
//! says every stage it hears (a refusal, a confirmation the person at
//! the app gives there) on stderr and in the window's title. The hand
//! looks at the app after each frame, so a click rests on what the
//! scene showed. A click on no control sends nothing.

use std::sync::mpsc::{self, Receiver};
use std::thread;

use accord::{SemId, Stage};
use engine::runtime::window::Windows;
use socket::{Client, ServerMsg};
use uigallery::shell::window::mirror::{Hand, Viewer, said};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy};
use winit::keyboard::{Key, NamedKey};
use winit::window::{WindowAttributes, WindowId};

use crate::cli::{self, NAME, Opts};

/// What the reading thread and the hand tell the loop.
enum Came {
    Line(Box<ServerMsg>),
    Ended(String),
    Said(String),
}

/// What the loop asks of the hand.
enum Do {
    /// A frame came: look at the app again.
    Look,
    /// A click on this control: press it.
    Press(SemId),
}

struct Show {
    windows: Windows,
    id: Option<WindowId>,
    viewer: Viewer,
    bindings: bool,
    title: String,
    frames: u64,
    stop: Option<u64>,
    /// The pointer over the window, in physical pixels.
    cursor: Option<(f64, f64)>,
    /// The hand's thread; `None` when its connection failed (a click
    /// then says so and sends nothing).
    hand: Option<mpsc::Sender<Do>>,
}

pub fn run(opts: &Opts) -> Result<(), String> {
    let path = cli::socket(opts.app.as_deref())?;
    let mut c = Client::connect(&path).map_err(|e| e.to_string())?;
    c.hello(NAME).map_err(|e| e.to_string())?;
    c.watch_scene(&opts.window).map_err(|e| e.to_string())?;
    let event_loop = EventLoop::<Came>::with_user_event()
        .build()
        .map_err(|e| e.to_string())?;
    let proxy = event_loop.create_proxy();
    let hand = match Hand::connect(&path, NAME) {
        Ok(hand) => Some(start_hand(hand, event_loop.create_proxy())?),
        Err(e) => {
            eprintln!("mirror: no hand, clicks send nothing: {e}");
            None
        }
    };
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
        cursor: None,
        hand,
    };
    event_loop.run_app(&mut show).map_err(|e| e.to_string())
}

/// The hand on a thread of its own: an act may wait long (for a person
/// at the app), and the loop never waits on the app. Looks in a row are
/// one look.
fn start_hand(mut hand: Hand, proxy: EventLoopProxy<Came>) -> Result<mpsc::Sender<Do>, String> {
    let (tx, rx) = mpsc::channel::<Do>();
    thread::Builder::new()
        .name("mirror-hand".into())
        .spawn(move || serve_hand(&mut hand, &rx, &proxy))
        .map_err(|e| e.to_string())?;
    Ok(tx)
}

fn serve_hand(hand: &mut Hand, rx: &Receiver<Do>, proxy: &EventLoopProxy<Came>) {
    let say = |text: String| proxy.send_event(Came::Said(text)).is_ok();
    while let Ok(first) = rx.recv() {
        let mut todo = vec![first];
        todo.extend(rx.try_iter());
        for (at, what) in todo.iter().enumerate() {
            let ok = match what {
                Do::Look if matches!(todo.get(at + 1), Some(Do::Look)) => true,
                Do::Look => match hand.look() {
                    Ok(_) => true,
                    Err(e) => say(format!("the look failed: {e}")),
                },
                Do::Press(target) => {
                    // a confirmation asked, and a person's answer, are
                    // said as they come: the wait may be long
                    let pressed = hand.press(target, |action, reply| {
                        if matches!(
                            reply.stage,
                            Stage::AwaitingConfirmation | Stage::Confirmed { .. }
                        ) {
                            say(said(target, action, &reply.stage));
                        }
                    });
                    say(pressed.said())
                }
            };
            if !ok {
                return;
            }
        }
    }
}

impl Show {
    fn redraw(&mut self) {
        if let Some(id) = self.id {
            self.windows.request_redraw(id);
        }
    }

    /// A left click at the pointer: the control under it to the hand.
    fn click(&mut self, id: WindowId) {
        let (Some((px, py)), Some(host)) = (self.cursor, self.windows.get(id)) else {
            return;
        };
        let scale = host.scale_factor().max(f64::MIN_POSITIVE);
        let (x, y) = ((px / scale) as f32, (py / scale) as f32);
        let Some(target) = self.viewer.hit(x, y).cloned() else {
            eprintln!("mirror: no control at ({x:.0}, {y:.0}): nothing went to the app");
            return;
        };
        match &self.hand {
            Some(hand) if hand.send(Do::Press(target.clone())).is_ok() => {
                eprintln!("mirror: click on {target}");
            }
            _ => eprintln!("mirror: click on {target}, and no hand to press it"),
        }
    }

    /// What the hand said, on stderr and in the window's title.
    fn say(&mut self, text: &str) {
        eprintln!("mirror: {text}");
        if let Some(window) = self
            .id
            .and_then(|id| self.windows.get(id)?.window().cloned())
        {
            window.set_title(&format!("{}: {text}", self.title));
        }
    }
}

impl ApplicationHandler<Came> for Show {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: Came) {
        let msg = match event {
            Came::Line(msg) => msg,
            Came::Said(text) => {
                self.say(&text);
                return;
            }
            Came::Ended(why) => {
                eprintln!("mirror: the watch ended: {why}");
                event_loop.exit();
                return;
            }
        };
        if let socket::Reply::Error(e) = &msg.reply {
            // a refusal: the app's person did not share its scenes
            // (`scene_not_shared`), or the watch was refused otherwise
            eprintln!("mirror: {e}");
            event_loop.exit();
            return;
        }
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
        if let Some(hand) = &self.hand {
            let _ = hand.send(Do::Look);
        }
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
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor = Some((position.x, position.y));
            }
            WindowEvent::CursorLeft { .. } => self.cursor = None,
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => self.click(id),
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
