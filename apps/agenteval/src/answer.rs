//! the person of the script answers what an agent asks her. every request
//! of an agent that the dispatcher leaves waiting for a person
//! (`awaiting_confirmation`: an agent's question, `Request::ask`, or an
//! action whose policy asks a person, as `discard`) is answered
//! `--answer-after` later, oldest first, with a click on the modal's
//! button, as a person at the screen answers: the view queues it and the
//! workbench confirms or declines it through the dispatcher as
//! `Actor::Human` (`Workbench::click`, the path the ide's tests take).
//! the click is recorded as her input, at the button and with no target:
//! the modal is bound to no control, and the control under it is not what
//! she pointed at. with `--answer none` she never answers, and the
//! request waits, as before this scenario.

use std::time::{Duration, Instant};

use accord::{Millis, Receipt, RequestId, Stage};
use engine::widgets::overlay::OverlayKind;
use idestudio::capability::Workbench;
use record::InputKind;

use crate::args::Answer;
use crate::arms::plev::Lab;
use crate::report::row;

/// the person's answers to the requests that waited for her.
#[derive(Debug)]
pub struct Answers {
    pub answer: Answer,
    /// how long after the dispatcher left a request waiting she answers.
    pub after: Duration,
    /// every request of an agent that waited for a person, in the order
    /// the dispatcher took them.
    pub waited: Vec<Waited>,
}

/// one request of an agent that waited for a person.
#[derive(Clone, Debug)]
pub struct Waited {
    pub request: RequestId,
    /// when the laboratory saw it waiting, on the workbench's clock.
    pub t: Millis,
    /// when the person answers it, on the laboratory's clock.
    due: Instant,
    pub answered: Option<Answered>,
    /// why the person could not answer it, when she could not.
    pub missed: Option<String>,
}

/// the person's click on the modal that asked a request.
#[derive(Clone, Debug)]
pub struct Answered {
    /// on the workbench's clock.
    pub t: Millis,
    pub allowed: bool,
    /// the point of the click, in logical window pixels.
    pub at: [f32; 2],
    /// the modal she answered, as it read: title and body.
    pub modal: Option<(String, String)>,
}

impl Answers {
    pub fn new(answer: Answer, after: Duration) -> Self {
        Answers {
            answer,
            after,
            waited: Vec::new(),
        }
    }

    /// the dispatcher left `request`, an agent's, waiting for a person at
    /// `t` (workbench clock); she answers it `after` from now.
    pub fn waits(&mut self, request: RequestId, t: Millis) {
        if self.waited.iter().any(|w| w.request == request) {
            return;
        }
        self.waited.push(Waited {
            request,
            t,
            due: Instant::now() + self.after,
            answered: None,
            missed: None,
        });
    }

    /// the next instant the person answers a request, when she answers
    /// at all and one waits.
    pub fn next_due(&self) -> Option<Instant> {
        if self.answer == Answer::None {
            return None;
        }
        self.open().map(|w| w.due).min()
    }

    fn open(&self) -> impl Iterator<Item = &Waited> {
        self.waited
            .iter()
            .filter(|w| w.answered.is_none() && w.missed.is_none())
    }

    /// the oldest request she has not answered whose time came.
    fn due(&self, now: Instant) -> Option<RequestId> {
        self.open().find(|w| w.due <= now).map(|w| w.request)
    }

    fn get(&mut self, request: RequestId) -> Option<&mut Waited> {
        self.waited.iter_mut().find(|w| w.request == request)
    }

    /// what the person did, one line per request that waited for her.
    pub fn rows(&self) -> Vec<(String, String)> {
        let mut rows = vec![row(
            "scenario",
            &format!(
                "{}, {} ms after",
                self.answer.name(),
                self.after.as_millis()
            ),
        )];
        for w in &self.waited {
            let what = match (&w.answered, &w.missed) {
                (Some(a), _) => format!(
                    "{} at {} ms (waiting since {} ms), a click at ({:.0}, {:.0}) on {:?}",
                    if a.allowed { "allowed" } else { "declined" },
                    a.t,
                    w.t,
                    a.at[0],
                    a.at[1],
                    a.modal
                        .as_ref()
                        .map_or("no modal", |(title, _)| title.as_str())
                ),
                (None, Some(why)) => format!("not answered: {why} (waiting since {} ms)", w.t),
                (None, None) => format!("not answered (waiting since {} ms)", w.t),
            };
            rows.push(row(&format!("request {}", w.request), &what));
        }
        rows
    }
}

impl Lab {
    /// the person answers every request whose time came, oldest first, in
    /// the modal; `true` when she clicked. the state and the frame are
    /// recorded after each click, as after any change.
    pub fn answer(&mut self) -> bool {
        let allowed = match self.person.answers.answer {
            Answer::None => return false,
            Answer::Allow => true,
            Answer::Decline => false,
        };
        let mut clicked = false;
        while let Some(request) = self.person.answers.due(Instant::now()) {
            if !waits(&self.app.bench, request) {
                if let Some(w) = self.person.answers.get(request) {
                    w.missed = Some("it no longer waited when the person came to it".to_owned());
                }
                continue;
            }
            // the buttons as last drawn: a modal opened since the last
            // frame is drawn first.
            if self.app.bench.workspace().modal_buttons().is_none() {
                self.record();
            }
            let Some((allow, cancel)) = self.app.bench.workspace().modal_buttons() else {
                if let Some(w) = self.person.answers.get(request) {
                    w.missed = Some("no modal asked it".to_owned());
                }
                continue;
            };
            let [x, y, w, h] = if allowed { allow } else { cancel };
            let at = [x + w / 2.0, y + h / 2.0];
            let modal = modal_on_screen(&self.app.bench);
            let before: Vec<RequestId> = self
                .person
                .answers
                .open()
                .map(|w| w.request)
                .filter(|r| waits(&self.app.bench, *r))
                .collect();
            self.app.bench.click(at[0], at[1]);
            clicked = true;
            // the modal asks the oldest request that waits, which is the
            // oldest she has not answered: whichever one the click ended
            // is the one she answered.
            let ended: Vec<RequestId> = before
                .into_iter()
                .filter(|r| !waits(&self.app.bench, *r))
                .collect();
            // her click is recorded at the instant the answer landed: the
            // stage that followed the wait, stamped by the dispatcher
            // inside the click. reading the clock a second time here
            // could cross a millisecond and leave the click one ahead or
            // behind the stage it caused.
            let t = ended
                .first()
                .and_then(|r| answered_at(self.app.bench.receipt(*r)))
                .unwrap_or_else(|| self.app.bench.now());
            self.rec.input(t, InputKind::PointerDown, Some(at), None);
            for r in &ended {
                if let Some(w) = self.person.answers.get(*r) {
                    w.answered = Some(Answered {
                        t,
                        allowed,
                        at,
                        modal: modal.clone(),
                    });
                }
            }
            self.record();
            if ended.is_empty() {
                if let Some(w) = self.person.answers.get(request) {
                    w.missed = Some("the click on the modal ended no request".to_owned());
                }
            }
        }
        clicked
    }
}

/// `request` still waits for a person.
fn waits(bench: &Workbench, request: RequestId) -> bool {
    bench.receipt(request).and_then(Receipt::stage) == Some(&Stage::AwaitingConfirmation)
}

/// the title and body of the modal on screen, the topmost one.
fn modal_on_screen(bench: &Workbench) -> Option<(String, String)> {
    bench
        .workspace()
        .overlay_mgr
        .stack
        .iter()
        .rev()
        .find_map(|overlay| match &overlay.kind {
            OverlayKind::Modal { title, body, .. } => Some((title.clone(), body.clone())),
            _ => None,
        })
}

/// the instant the person's answer landed on `receipt`: the stage right
/// after its wait for her (`AwaitingConfirmation`), the same one the
/// report calls `answered_at`.
fn answered_at(receipt: Option<&accord::Receipt>) -> Option<Millis> {
    let stages = &receipt?.stages;
    let waiting = stages
        .iter()
        .position(|(stage, _)| matches!(stage, accord::Stage::AwaitingConfirmation))?;
    stages.get(waiting + 1).map(|(_, t)| *t)
}
