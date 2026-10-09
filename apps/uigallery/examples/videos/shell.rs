//! The shell without a window, for the demo: the view, the frame on
//! screen (a compositor, never a GPU), the port and the recording when
//! there is one; and what the agent asks of it between two of its
//! messages, the person at the window.

use std::sync::mpsc::{self, Sender};
use std::thread;
use std::time::{Duration, Instant};

use engine::widgets::prelude::WidgetEvent;
use engine::drawing::compositor::Compositor;
use record::InputKind;
use accord::SemId;
use uigallery::shell::window::record::Recording;
use uigallery::views::ShowcaseView;
use socket::AgentPort;

use crate::run::TURN;

/// What the agent asks the shell's thread to do between two of its
/// messages: the person at the window.
pub enum Poke {
    /// A click on the middle of where `id` shows in the frame on screen.
    Click(SemId),
}

pub type Answer = Sender<Result<(), String>>;
pub type Pokes = Sender<(Poke, Answer)>;

pub fn poke(pokes: &Pokes, what: Poke) -> Result<(), String> {
    let (done, wait) = mpsc::channel();
    pokes
        .send((what, done))
        .map_err(|_| "the shell went away".to_owned())?;
    wait.recv()
        .map_err(|_| "the shell did not answer".to_owned())?
}

pub struct Shell {
    pub view: ShowcaseView,
    pub frame: Compositor,
    pub port: AgentPort,
    pub recording: Option<Recording>,
    pub last: Instant,
    pub frames: usize,
}

impl Shell {
    /// A frame drawn, and recorded with the state it was drawn from.
    pub fn draw(&mut self) {
        self.view.render(&mut self.frame);
        if let Some(recording) = &self.recording {
            recording.frame(&mut self.view, &self.frame);
        }
        self.frames += 1;
    }

    /// One turn of the loop: the port served (an act records the state
    /// it left), the page ticked by the real time gone, and a frame when
    /// either changed something.
    pub fn turn(&mut self) -> bool {
        let dt = self.last.elapsed().as_secs_f32();
        self.last = Instant::now();
        let recorder = self.recording.as_ref().map(Recording::recorder);
        let acted = self
            .view
            .serve_agent_recorded(&mut self.port, &self.frame, recorder);
        let moved = self.view.tick(dt);
        if acted || moved {
            self.draw();
        }
        acted || moved
    }

    /// Where `id` shows in the frame on screen, whole: the person scrolls
    /// the page down with the wheel (over the intent row, a frame per
    /// notch) until the control is inside the window, as the generated
    /// surface's list may start below the fold.
    fn reveal(&mut self, id: &SemId) -> Result<[f32; 4], String> {
        let wheel = self
            .frame
            .binding(&SemId::parse("silences_surface.intent").map_err(|e| e.to_string())?)
            .map(|b| (b.rect[0] + b.rect[2] / 2.0, b.rect[1] + b.rect[3] / 2.0))
            .ok_or_else(|| "the intent row does not show".to_owned())?;
        for _ in 0..60 {
            let bound = self
                .frame
                .binding(id)
                .map(|b| (b.rect, b.visible_rect()))
                .ok_or_else(|| format!("{id} is not bound in the frame on screen"))?;
            // whole within a pixel: the frame's clip is whole physical
            // pixels, the rect is not
            if let (rect, Some(visible)) = bound
                && visible[2] >= rect[2] - 1.0
                && visible[3] >= rect[3] - 1.0
            {
                return Ok(visible);
            }
            let (x, y) = wheel;
            let scrolled = self
                .view
                .handle_event(&WidgetEvent::Scroll { x, y, delta: 48.0 });
            if !scrolled {
                return Err(format!(
                    "{id} sits outside the window ({:?}, visible {:?}, the page scrolled to {}) \
                     and the page does not scroll further",
                    bound.0,
                    bound.1,
                    self.view.page_offset()
                ));
            }
            self.draw();
        }
        Err(format!("{id} never came whole into the window"))
    }

    /// The person clicks the middle of `id` in the frame on screen: the
    /// press and the release recorded with the control under them, the
    /// state after each when it moved, and a frame.
    pub fn click(&mut self, id: &SemId) -> Result<(), String> {
        let [x, y, w, h] = self.reveal(id)?;
        let (x, y) = (x + w / 2.0, y + h / 2.0);
        if self.view.handle_event(&WidgetEvent::MouseMove { x, y }) {
            self.draw();
        }
        for (kind, event) in [
            (InputKind::PointerDown, WidgetEvent::MouseDown { x, y }),
            (InputKind::PointerUp, WidgetEvent::MouseUp { x, y }),
        ] {
            if let Some(recording) = &self.recording {
                recording.input(&mut self.view, kind, Some([x, y]), &self.frame);
            }
            if self.view.handle_event(&event)
                && let Some(recording) = &self.recording
            {
                recording.state(&mut self.view);
            }
        }
        self.draw();
        Ok(())
    }

    /// Turns until the page has asked for nothing over a stretch (the
    /// exit of a closed surface fades out), at most `at_most`.
    pub fn settle(&mut self, at_most: Duration) {
        let start = Instant::now();
        let mut quiet = 0;
        while quiet < 30 && start.elapsed() < at_most {
            thread::sleep(TURN);
            if self.turn() {
                quiet = 0;
            } else {
                quiet += 1;
            }
        }
    }
}
