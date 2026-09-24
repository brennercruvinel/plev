//! The Silences page as a capability: what it means (`sem::Semantics`),
//! what it runs (`sem::Operable`), and the one path every actor takes to
//! run it, the gallery's dispatcher (the todo page's, lent by
//! `TodoPage::with_dispatcher` as the Surface page does: the agent port
//! adds its sink to that one dispatcher and matches acts by `RequestId`,
//! so a second dispatcher would be unheard and would collide on ids).
//!
//! The semantic model: `silences` (region; `cuts`, the ids of the cuts in
//! the order shown), `silences.video` (media: `file`, `duration`, `shown`),
//! `silences.threshold` and `silences.min_duration` (sliders: `value`,
//! `set_value`, `increment`, `decrement`), `silences.detect` (button:
//! `detect_silence`, reading the cuts and both values), `silences.cut[<id>]`
//! for each cut (list item named by its range: `start`, `end`, `duration`,
//! `decision`; `preview_segment` reading `silences.video:shown`,
//! `apply_cut` and `reject_cut` reading `decision`, each disabled when it
//! is the decision already), `silences.counts` (status: `total`,
//! `applied`, `rejected`, `pending`, `cut_secs`), `silences.apply_all`
//! (button: `apply_all` reading `silences.counts:pending`) and
//! `silences.status` (`message`, `busy`).
//!
//! `detect_silence`, and a `preview_segment` whose frame is not decoded
//! yet, answer `Execution::Started` and keep the request open; `settle`,
//! called from the page's wake, reads what the worker sent and ends them
//! with `Dispatcher::complete` or `fail`, through the same lending.

use comps::prelude::EventResult;
use engine::perf::stages::{self, Stage};
use sem::{
    Actor, Execution, Operable, Receipt, Request, RequestId, Role, SemError, SemErrors, SemId,
    Semantics, Snapshot, SnapshotBuilder, Value,
};
use showcase::model::silences::{Decision, Settings, TEST_VIDEO, Video};

use super::worker::{Done, Job};
use super::{Said, SilencesPage};
use crate::view::apps::app::TodoPage;

/// The ids the page projects and binds, parsed once.
pub(super) struct Ids {
    pub region: SemId,
    pub video: SemId,
    pub threshold: SemId,
    pub min_duration: SemId,
    pub detect: SemId,
    /// `silences.cut`, keyed by each cut's id.
    cut: SemId,
    pub counts: SemId,
    pub apply_all: SemId,
    pub status: SemId,
}

impl Ids {
    pub(super) fn new() -> Result<Self, SemError> {
        let region = SemId::name("silences")?;
        Ok(Self {
            video: region.child("video")?,
            threshold: region.child("threshold")?,
            min_duration: region.child("min_duration")?,
            detect: region.child("detect")?,
            cut: region.child("cut")?,
            counts: region.child("counts")?,
            apply_all: region.child("apply_all")?,
            status: region.child("status")?,
            region,
        })
    }

    /// `silences.cut[<id>]`.
    pub(super) fn cut(&self, id: u64) -> SemId {
        self.cut.key(id.to_string())
    }

    /// The cut id behind a `silences.cut[<id>]`.
    pub(super) fn cut_of(&self, target: &SemId) -> Option<u64> {
        target
            .as_str()
            .strip_prefix(self.cut.as_str())?
            .strip_prefix('[')?
            .strip_suffix(']')?
            .parse()
            .ok()
    }

    /// True for every id the page projects.
    pub(super) fn holds(&self, id: &SemId) -> bool {
        *id == self.region
            || *id == self.video
            || *id == self.threshold
            || *id == self.min_duration
            || *id == self.detect
            || *id == self.counts
            || *id == self.apply_all
            || *id == self.status
            || self.cut_of(id).is_some()
    }

    fn target(&self, target: Target) -> SemId {
        match target {
            Target::Threshold => self.threshold.clone(),
            Target::MinDuration => self.min_duration.clone(),
            Target::Detect => self.detect.clone(),
            Target::Cut(id) => self.cut(id),
            Target::ApplyAll => self.apply_all.clone(),
        }
    }

    /// What a request accepted against the page asks it to do.
    fn op(&self, req: &Request) -> Option<Op> {
        let num = || match &req.param {
            Some(Value::Num(n)) => Some(*n),
            Some(Value::Int(n)) => Some(*n as f64),
            _ => None,
        };
        let slider = if req.target == self.threshold {
            Some(Which::Threshold)
        } else if req.target == self.min_duration {
            Some(Which::MinDuration)
        } else {
            None
        };
        match (req.action.as_str(), slider) {
            ("set_value", Some(which)) => num().map(|v| Op::Set(which, v)),
            ("increment", Some(which)) => Some(Op::Nudge(which, 1.0)),
            ("decrement", Some(which)) => Some(Op::Nudge(which, -1.0)),
            ("detect_silence", _) if req.target == self.detect => Some(Op::Detect),
            ("apply_all", _) if req.target == self.apply_all => Some(Op::ApplyAll),
            ("preview_segment", _) => self.cut_of(&req.target).map(Op::Preview),
            ("apply_cut", _) => self
                .cut_of(&req.target)
                .map(|id| Op::Decide(id, Decision::Apply)),
            ("reject_cut", _) => self
                .cut_of(&req.target)
                .map(|id| Op::Decide(id, Decision::Reject)),
            _ => None,
        }
    }
}

/// What the page's own input asks for, named without the ids.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Target {
    Threshold,
    MinDuration,
    Detect,
    Cut(u64),
    ApplyAll,
}

/// The two sliders.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Which {
    Threshold,
    MinDuration,
}

/// The one handler of each capability.
enum Op {
    Set(Which, f64),
    Nudge(Which, f32),
    Detect,
    Preview(u64),
    Decide(u64, Decision),
    ApplyAll,
}

fn count(n: usize) -> Value {
    Value::Int(i64::try_from(n).unwrap_or(i64::MAX))
}

/// Seconds at ffmpeg's six decimals: a difference or a sum of the values
/// it printed carries no binary noise into the projection.
fn micros(secs: f64) -> f64 {
    (secs * 1e6).round() / 1e6
}

/// `m:ss.d`, the way the screen writes an instant.
pub(super) fn clock(t: f64) -> String {
    let t = if t.is_finite() { t.max(0.0) } else { 0.0 };
    let m = (t / 60.0).floor();
    let s = t - m * 60.0;
    format!("{m}:{s:04.1}")
}

/// The instant in milliseconds, the key of a frame.
pub(super) fn ms(t: f64) -> u64 {
    if t.is_finite() && t > 0.0 {
        (t * 1000.0).round() as u64
    } else {
        0
    }
}

impl Semantics for SilencesPage {
    fn project(&self, b: &mut SnapshotBuilder) {
        let Some(ids) = &self.ids else {
            return;
        };
        let cuts: Vec<String> = self
            .model
            .cuts()
            .iter()
            .map(|c| ids.cut(c.id()).to_string())
            .collect();
        b.node(&ids.region, Role::Region, "silences")
            .inert(self.inert)
            .state("cuts", Value::List(cuts));
        let name = self
            .model
            .video()
            .map_or(TEST_VIDEO, |v| v.name.as_str())
            .to_owned();
        b.node(&ids.video, Role::Media, name.as_str())
            .parent(&ids.region)
            .state("file", name.as_str())
            .state("duration", self.model.duration())
            .state("shown", self.model.shown());
        self.threshold
            .project(b, &ids.threshold, "threshold (dB)", Some(&ids.region));
        self.min_duration.project(
            b,
            &ids.min_duration,
            "minimum silence (s)",
            Some(&ids.region),
        );
        let busy = self.worker.busy();
        let mut detect = b.node(&ids.detect, Role::Button, "detect silences");
        detect.parent(&ids.region).enabled(!busy);
        detect
            .action("detect_silence")
            .enabled(!busy)
            .reads(format!("{}:cuts", ids.region))
            .reads(format!("{}:value", ids.threshold))
            .reads(format!("{}:value", ids.min_duration));
        let shown = format!("{}:shown", ids.video);
        for cut in self.model.cuts() {
            let id = ids.cut(cut.id());
            let seg = cut.segment();
            let decision = cut.decision();
            let label = format!("{} to {}", clock(seg.start), clock(seg.end));
            let mut node = b.node(&id, Role::ListItem, label);
            node.parent(&ids.region)
                .state("start", seg.start)
                .state("end", seg.end)
                .state("duration", micros(seg.duration()))
                .state("decision", decision.name());
            node.action("preview_segment").reads(shown.as_str());
            let reads = format!("{id}:decision");
            node.action("apply_cut")
                .enabled(decision != Decision::Apply)
                .reads(reads.as_str());
            node.action("reject_cut")
                .enabled(decision != Decision::Reject)
                .reads(reads.as_str());
        }
        let counts = self.model.counts();
        b.node(&ids.counts, Role::Status, self.counts_text())
            .parent(&ids.region)
            .state("total", count(counts.total))
            .state("applied", count(counts.applied))
            .state("rejected", count(counts.rejected))
            .state("pending", count(counts.pending))
            .state("cut_secs", micros(counts.cut_secs));
        let some = counts.pending > 0;
        let mut apply_all = b.node(&ids.apply_all, Role::Button, "apply all");
        apply_all.parent(&ids.region).enabled(some);
        apply_all
            .action("apply_all")
            .enabled(some)
            .reads(format!("{}:pending", ids.counts));
        let (message, busy, _) = self.message();
        b.node(&ids.status, Role::Status, "status")
            .parent(&ids.region)
            .state("message", message)
            .state("busy", busy);
        match self.focus {
            Some(Which::Threshold) => b.focus(&ids.threshold),
            Some(Which::MinDuration) => b.focus(&ids.min_duration),
            None => {}
        }
    }
}

impl Operable for SilencesPage {
    fn execute(&mut self, id: RequestId, req: &Request) -> Execution {
        let Some(op) = self.ids.as_ref().and_then(|ids| ids.op(req)) else {
            return Execution::Failed(format!(
                "the silences page has no {} on {}",
                req.action, req.target
            ));
        };
        let mut outcome = sem::Outcome::default();
        match op {
            Op::Set(which, v) => {
                self.slider_mut(which).set_value(v as f32);
            }
            Op::Nudge(which, dir) => {
                let slider = self.slider_mut(which);
                let moved = if dir > 0.0 {
                    slider.increment()
                } else {
                    slider.decrement()
                };
                if !moved {
                    return Execution::Failed("the slider is at the end of its range".to_owned());
                }
            }
            Op::Detect => {
                if self.worker.busy() {
                    return Execution::Failed("ffmpeg is busy: wait for it".to_owned());
                }
                self.worker.submit(Job::Detect(self.settings()));
                self.detecting = Some(id);
                return Execution::Started;
            }
            Op::Preview(cut) => {
                let Some(seg) = self.model.cut(cut).map(|c| c.segment()) else {
                    return Execution::Failed(format!("no cut {cut}"));
                };
                let at = seg.middle();
                self.model.show(at);
                let key = ms(at);
                outcome.result.insert("shown".to_owned(), Value::Num(at));
                if !self.frames.contains_key(&key) {
                    self.worker.submit(Job::Frame { ms: key });
                    self.previews.push((id, key));
                    return Execution::Started;
                }
            }
            Op::Decide(cut, decision) => {
                if self.model.decide(cut, decision).is_none() {
                    return Execution::Failed(format!("no cut {cut}"));
                }
                outcome
                    .result
                    .insert("decision".to_owned(), Value::from(decision.name()));
            }
            Op::ApplyAll => {
                let applied = self.model.apply_all();
                outcome
                    .result
                    .insert("applied".to_owned(), count(applied.newly));
                outcome
                    .result
                    .insert("cut_secs".to_owned(), Value::Num(applied.counts.cut_secs));
            }
        }
        Execution::Done(outcome)
    }
}

impl SilencesPage {
    /// The page projected now.
    pub(crate) fn snapshot(&self) -> Result<Snapshot, SemErrors> {
        let _probe = stages::span(Stage::Projection);
        let mut b = SnapshotBuilder::new();
        self.project(&mut b);
        b.finish()
    }

    /// Runs `req` through the gallery's dispatcher (the todo page lends
    /// it, with its clock's now), over this page.
    pub(crate) fn dispatch(&mut self, req: Request, todo: &mut TodoPage) -> RequestId {
        todo.with_dispatcher(|d, _, now| d.dispatch(self, req, now))
    }

    /// A request from an actor outside the page's own input (an agent
    /// through the port, a screen reader); the receipt as it stands.
    pub(crate) fn act(
        &mut self,
        req: Request,
        todo: &mut TodoPage,
        inert: bool,
    ) -> Option<Receipt> {
        self.inert = inert;
        let id = self.dispatch(req, todo);
        todo.receipt(id).cloned()
    }

    /// Keyboard focus asked from outside the page's pointer handling: a
    /// slider takes it; another node of the page takes it from the
    /// sliders. `false` for an id the page does not project, and while
    /// inert.
    pub(crate) fn focus(&mut self, id: &SemId) -> bool {
        let Some(ids) = &self.ids else {
            return false;
        };
        if self.inert || !ids.holds(id) {
            return false;
        }
        let which = if *id == ids.threshold {
            Some(Which::Threshold)
        } else if *id == ids.min_duration {
            Some(Which::MinDuration)
        } else {
            None
        };
        self.set_focus(which);
        true
    }

    /// A person's click or key as a request: observe the page as it is
    /// now, ask through the dispatcher, and ask for a frame when the
    /// projected state moved. A refused request is still handled: the
    /// person aimed at the page.
    pub(super) fn ask(
        &mut self,
        todo: &mut TodoPage,
        target: Target,
        action: &str,
        param: Option<Value>,
    ) -> EventResult {
        let Some(target) = self.ids.as_ref().map(|ids| ids.target(target)) else {
            return EventResult::IGNORED;
        };
        let seen = match self.snapshot() {
            Ok(seen) => seen,
            Err(errors) => {
                log::warn!("silences: the projection does not validate: {errors}");
                return EventResult::IGNORED;
            }
        };
        let req = match Request::observe(&seen, Actor::Human, &target, action, param) {
            Ok(req) => req,
            Err(e) => {
                log::warn!("silences: {e}");
                return EventResult::IGNORED;
            }
        };
        let id = self.dispatch(req, todo);
        log::debug!("silences: {target} {action}: {:?}", todo.receipt(id));
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

    /// What the worker sent since the page last looked: a detection
    /// replaces the cuts and ends its request; a frame goes into the cache,
    /// ends the previews waiting for it and asks for the frames still
    /// missing. True when what the page shows moved.
    pub(super) fn settle(&mut self, todo: &mut TodoPage) -> bool {
        let done = self.worker.poll();
        let moved = !done.is_empty();
        for d in done {
            match d {
                Done::Detected { with, result, took } => {
                    let id = self.detecting.take();
                    let ended = match result {
                        Ok(detected) => {
                            let video = Video {
                                name: TEST_VIDEO.to_owned(),
                                duration: detected.duration.unwrap_or(0.0),
                            };
                            self.model.set_detected(video, with, &detected.segments);
                            let cuts = self.model.cuts().len();
                            self.said = Said::Detected { took };
                            self.want_frames();
                            let mut outcome = sem::Outcome::default();
                            outcome.result.insert("cuts".to_owned(), count(cuts));
                            outcome
                                .result
                                .insert("seconds".to_owned(), Value::Num(took.as_secs_f64()));
                            id.map(|id| {
                                todo.with_dispatcher(|d, _, now| d.complete(id, outcome, now))
                            })
                        }
                        Err(why) => {
                            self.said = Said::Failed(why.clone());
                            id.map(|id| todo.with_dispatcher(|d, _, now| d.fail(id, why, now)))
                        }
                    };
                    if let Some(Err(e)) = ended {
                        log::warn!("silences: the detection's request did not end: {e}");
                    }
                }
                Done::Frame { ms, result, took } => {
                    log::debug!("silences: the frame at {ms} ms took {took:?}");
                    #[cfg(test)]
                    self.frame_took.push(took);
                    if let Err(why) = &result {
                        self.said = Said::Failed(why.clone());
                    }
                    let (ready, rest): (Vec<_>, Vec<_>) =
                        self.previews.drain(..).partition(|(_, at)| *at == ms);
                    self.previews = rest;
                    for (id, _) in ready {
                        let ended = todo.with_dispatcher(|d, _, now| match &result {
                            Ok(_) => {
                                let mut outcome = sem::Outcome::default();
                                outcome
                                    .result
                                    .insert("shown".to_owned(), Value::Num(ms as f64 / 1000.0));
                                d.complete(id, outcome, now)
                            }
                            Err(why) => d.fail(id, why.clone(), now),
                        });
                        if let Err(e) = ended {
                            log::warn!("silences: the preview's request did not end: {e}");
                        }
                    }
                    let ok = result.is_ok();
                    self.frames.insert(ms, result);
                    if ok {
                        self.want_frames();
                    }
                }
            }
        }
        moved
    }

    /// The frames the screen shows and does not have yet: the instant
    /// shown and the middle of each cut, one job each, in that order.
    fn want_frames(&mut self) {
        let mut wanted = vec![ms(self.model.shown())];
        wanted.extend(self.model.cuts().iter().map(|c| ms(c.segment().middle())));
        for at in wanted {
            if !self.frames.contains_key(&at) {
                self.worker.submit(Job::Frame { ms: at });
            }
        }
    }

    /// The sliders' values as the settings a detection would run with.
    pub(super) fn settings(&self) -> Settings {
        Settings {
            threshold_db: self.threshold.value(),
            min_duration: self.min_duration.value(),
        }
        .clamped()
    }

    pub(super) fn slider_mut(&mut self, which: Which) -> &mut comps::prelude::Slider {
        match which {
            Which::Threshold => &mut self.threshold,
            Which::MinDuration => &mut self.min_duration,
        }
    }

    pub(super) fn set_focus(&mut self, which: Option<Which>) {
        self.focus = which;
        self.threshold.set_focused(which == Some(Which::Threshold));
        self.min_duration
            .set_focused(which == Some(Which::MinDuration));
    }

    /// `2 silences, 1 applied, 0 rejected, 2.0 s cut`.
    pub(super) fn counts_text(&self) -> String {
        let c = self.model.counts();
        format!(
            "{} {}, {} applied, {} rejected, {:.1} s cut",
            c.total,
            if c.total == 1 { "silence" } else { "silences" },
            c.applied,
            c.rejected,
            c.cut_secs
        )
    }

    /// The status line: what runs now, or what the last run came to and
    /// whether the sliders moved since; with `busy`, and whether it is a
    /// failure.
    pub(super) fn message(&self) -> (String, bool, bool) {
        let with =
            |s: Settings| format!("{:.0} dB, {:.1} s minimum", s.threshold_db, s.min_duration);
        match self.worker.running() {
            Some(Job::Detect(s)) => {
                return (format!("detecting silences at {}", with(s)), true, false);
            }
            Some(Job::Frame { ms }) => {
                let at = clock(ms as f64 / 1000.0);
                return (format!("decoding the frame at {at}"), true, false);
            }
            None => {}
        }
        let counts = self.model.counts();
        let found = format!(
            "{} {}",
            counts.total,
            if counts.total == 1 {
                "silence"
            } else {
                "silences"
            }
        );
        let detected = with(self.model.detected_with());
        let mut text = match &self.said {
            Said::Seed => {
                format!("{found} from the recorded run at {detected}: detect runs ffmpeg here")
            }
            Said::Detected { took, .. } => {
                format!("{found} in {} ms at {detected}", took.as_millis())
            }
            Said::Failed(why) => return (why.clone(), false, true),
        };
        if self.settings() != self.model.detected_with() {
            text.push_str(&format!(
                "; the sliders moved to {}: detect again",
                with(self.settings())
            ));
        }
        (text, false, false)
    }
}
