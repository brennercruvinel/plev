//! The Capture page as a capability: what it means (`sem::Semantics`),
//! what it runs (`sem::Operable`), and the one path every actor takes to
//! run it, the gallery's dispatcher (the todo page's, lent as to the
//! Files page).
//!
//! The semantic model: `capture` (region; `camera` and `microphone`, one
//! line each saying where the device is), `capture.camera` and
//! `capture.microphone` (toggles: `on` is what the person turned on;
//! `toggle`, reading its own `on`, for a person at the screen, assistive
//! technology and a test, never an agent: `PERSON_ONLY`, the policy the
//! shell's window and print capabilities carry, so the dispatcher refuses
//! the agent before `execute` runs), `capture.frame` (image: `showing`,
//! whether a frame is on screen), `capture.level` (progress bar: the
//! microphone's `level`, to a tenth, and the same number as its `value`,
//! from 0 to 1, the one a screen reader reads) and `capture.status`
//! (`message`).
//! The captured pixels and samples are never projected: an agent reads
//! where the devices are, not what they see or hear.

use comps::prelude::{EventResult, Rect};
use engine::perf::stages::{self, Stage};
use engine::platform::capture::DeviceKind;
use sem::{
    Actor, Constraints, Execution, Operable, Outcome, Policy, Receipt, Request, RequestId, Role,
    SemError, SemErrors, SemId, Semantics, Snapshot, SnapshotBuilder, Value,
};

use super::{CapturePage, Said};
use crate::view::apps::app::TodoPage;
use crate::view::dispatch::PERSON_ONLY;

/// The ids the page projects and binds, parsed once.
pub(super) struct Ids {
    pub region: SemId,
    pub camera: SemId,
    pub microphone: SemId,
    pub frame: SemId,
    pub level: SemId,
    pub status: SemId,
}

impl Ids {
    pub(super) fn new() -> Result<Self, SemError> {
        let region = SemId::name("capture")?;
        Ok(Self {
            camera: region.child("camera")?,
            microphone: region.child("microphone")?,
            frame: region.child("frame")?,
            level: region.child("level")?,
            status: region.child("status")?,
            region,
        })
    }

    fn switch(&self, kind: DeviceKind) -> &SemId {
        match kind {
            DeviceKind::Camera => &self.camera,
            DeviceKind::Microphone => &self.microphone,
        }
    }

    /// The device a switch id names.
    fn kind(&self, id: &SemId) -> Option<DeviceKind> {
        DeviceKind::ALL
            .into_iter()
            .find(|kind| self.switch(*kind) == id)
    }

    /// True for every id the page projects.
    fn holds(&self, id: &SemId) -> bool {
        [
            &self.region,
            &self.camera,
            &self.microphone,
            &self.frame,
            &self.level,
            &self.status,
        ]
        .contains(&id)
    }
}

/// `(x, y, w, h)`, the form `Compositor::bind` takes.
pub(super) fn bounds(r: Rect) -> [f32; 4] {
    [r.x, r.y, r.w, r.h]
}

impl Semantics for CapturePage {
    fn project(&self, b: &mut SnapshotBuilder) {
        let Some(ids) = &self.ids else {
            return;
        };
        b.node(&ids.region, Role::Region, "capture")
            .inert(self.inert)
            .state("camera", self.line(DeviceKind::Camera))
            .state("microphone", self.line(DeviceKind::Microphone));
        for kind in DeviceKind::ALL {
            let id = ids.switch(kind);
            let mut node = b.node(id, Role::Toggle, kind.noun());
            node.parent(&ids.region)
                .state("on", self.wanted(kind))
                .focusable(true);
            node.action("toggle")
                .reads(format!("{id}:on"))
                .policy(Policy {
                    actors: PERSON_ONLY,
                    ..Policy::default()
                });
        }
        b.node(&ids.frame, Role::Image, "camera frame")
            .parent(&ids.region)
            .state("showing", self.showing().is_some());
        // the level is the bar's value too, from 0 to 1: a progress bar
        // says its value (task-75 phase 2a); `level` stays for whoever
        // reads it by that name
        let level = Value::Num(self.level_tenths());
        b.node(&ids.level, Role::ProgressBar, "microphone level")
            .parent(&ids.region)
            .state("level", level.clone())
            .state("value", level)
            .constraints(Constraints {
                range: Some((0.0, 1.0)),
                ..Constraints::default()
            });
        let message = self.status_text();
        b.node(&ids.status, Role::Status, message.as_str())
            .parent(&ids.region)
            .state("message", message.as_str());
        if let Some(focused) = self.focused {
            b.focus(ids.switch(focused).clone());
        }
    }
}

impl Operable for CapturePage {
    /// A toggle ends now: the person's wish flips and the device follows
    /// (started or stopped, never waited for); where the device got to
    /// is projected as it answers.
    fn execute(&mut self, _id: RequestId, req: &Request) -> Execution {
        let kind = self
            .ids
            .as_ref()
            .filter(|_| req.action.as_str() == "toggle")
            .and_then(|ids| ids.kind(&req.target));
        let Some(kind) = kind else {
            return Execution::Failed(format!(
                "the capture page has no {} on {}",
                req.action, req.target
            ));
        };
        self.switch_mut(kind).toggle();
        let on = self.wanted(kind);
        self.said = Said::Turned(kind, on);
        self.follow();
        self.look();
        let mut outcome = Outcome::default();
        outcome.result.insert("on".to_owned(), Value::Bool(on));
        outcome
            .result
            .insert("state".to_owned(), Value::Text(self.line(kind)));
        Execution::Done(outcome)
    }
}

impl CapturePage {
    /// The page projected now.
    pub(crate) fn snapshot(&self) -> Result<Snapshot, SemErrors> {
        let _probe = stages::span(Stage::Projection);
        let mut b = SnapshotBuilder::new();
        self.project(&mut b);
        b.finish()
    }

    /// Runs `req` through the gallery's dispatcher, over this page.
    pub(crate) fn dispatch(&mut self, req: Request, todo: &mut TodoPage) -> RequestId {
        todo.with_dispatcher(|d, _, now| d.dispatch(self, req, now))
    }

    /// A request from an actor outside the page's own input (an agent
    /// through the port, a screen reader), through the gallery's one
    /// dispatcher; the receipt as it stands after the dispatch.
    pub(crate) fn act(
        &mut self,
        req: Request,
        todo: &mut TodoPage,
        inert: bool,
    ) -> Option<Receipt> {
        self.set_inert(inert);
        let id = self.dispatch(req, todo);
        todo.receipt(id).cloned()
    }

    /// Keyboard focus asked from outside the page's pointer handling: a
    /// switch takes it; another node of the page takes it from the
    /// switches. `false` for an id the page does not project, and while
    /// inert.
    pub(crate) fn focus(&mut self, id: &SemId) -> bool {
        if self.inert {
            return false;
        }
        let Some(ids) = &self.ids else {
            return false;
        };
        if !ids.holds(id) {
            return false;
        }
        let on = ids.kind(id);
        self.set_focus(on);
        true
    }

    /// A person's click or key as a `toggle`: observe the page as it is
    /// now, ask through the dispatcher, and ask for a frame when the
    /// projected state moved. A refused request is still handled: the
    /// person aimed at the page.
    pub(super) fn ask(&mut self, todo: &mut TodoPage, kind: DeviceKind) -> EventResult {
        let Some(target) = self.ids.as_ref().map(|ids| ids.switch(kind).clone()) else {
            return EventResult::IGNORED;
        };
        let seen = match self.snapshot() {
            Ok(seen) => seen,
            Err(errors) => {
                log::warn!("capture: the projection does not validate: {errors}");
                return EventResult::IGNORED;
            }
        };
        let req = match Request::observe(&seen, Actor::Human, &target, "toggle", None) {
            Ok(req) => req,
            Err(e) => {
                log::warn!("capture: {e}");
                return EventResult::IGNORED;
            }
        };
        let id = self.dispatch(req, todo);
        log::debug!("capture: {target} toggle: {:?}", todo.receipt(id));
        let moved = self
            .snapshot()
            .map_or(true, |now| now.revision() != seen.revision());
        if moved {
            EventResult::clicked()
        } else {
            EventResult {
                handled: true,
                ..EventResult::IGNORED
            }
        }
    }
}
