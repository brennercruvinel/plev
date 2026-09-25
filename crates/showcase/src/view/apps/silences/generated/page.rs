//! The Silences page as its one dispatcher runs a request over the whole
//! of it (task-69 phase 4b): `Page`, an `Operable` that projects the
//! fixed screen, the generated surface and the proposing row, runs the
//! row's capabilities and `close`, and passes everything else to the
//! fixed screen's own `execute`. The pattern of the Surface page's
//! `page::Page`, over this app instead of the todo: the rules propose
//! inside `execute` and the request ends at once; the Claude is asked off
//! the app's thread (`surface_page::asking`), `propose` answers
//! `Execution::Started`, and `settle`, called from the page's wake, ends
//! the request when the answer arrives, through the same lending of the
//! todo page's dispatcher. A cancel ends the proposal it cancelled in the
//! same lending as the cancel itself. The author of a proposal is whoever
//! asked (`GenerationPolicy::by`), and the host keeps the surface with
//! that policy. With the review open, `propose` reads the intent as a
//! change of it first (task-70 phase 2, `surface_page::proposing::changing`):
//! "too much information", "I can't compare these", "bigger numbers",
//! "undo", by the rules at once or by the Claude off the thread, applied
//! by the host as whoever asked.
//!
//! The staging entries at the end (`ShowcaseView::stage_silences_*`) are
//! for the regression harness (`examples/regress`, task-69 checkpoint b),
//! which reaches the gallery only through its public API: the page with
//! frames it did not decode, so no ffmpeg runs; a fixture of IR shown as a
//! proposal would show it, so no model is asked; and an operation of
//! task-70 applied to the surface shown, as the host applies one.

use engine::gpu::image::ImageHandle;
use propose::compose::Composite;
use propose::{Ask, Rules, propose_verified};
use sem::{
    Actor, Execution, Operable, Outcome, Request, RequestId, SemId, Semantics, SnapshotBuilder,
    Value,
};
use surface::Surface;
use uir::{Capabilities, GenerationPolicy, VerifiedIr, verify_json};

use super::super::SilencesPage;
use super::super::jobs::instant_of;
use super::super::semantics::ms;
use super::offer::{Answer as Reply, beside};
use crate::view::ShowcaseView;
use crate::view::apps::app::TodoPage;
use crate::view::apps::surface_page::asking::{Answer, Job, Pending, Proposal};
use crate::view::apps::surface_page::proposing::changing::{self, Changing};
use crate::view::apps::surface_page::proposing::{Op, Proposer, Said, int};

/// The outcome of a proposal shown: the surface and how many nodes it
/// has.
fn result(surface: &SemId, nodes: usize) -> Outcome {
    let mut outcome = Outcome::default();
    outcome
        .result
        .insert("surface".to_owned(), Value::Text(surface.to_string()));
    outcome
        .result
        .insert("nodes".to_owned(), Value::Int(int(nodes)));
    outcome
}

/// The whole page as its dispatcher sees it for a request: the fixed
/// screen (`app`) with its generated block.
pub(in crate::view::apps::silences) struct Page<'a> {
    pub app: &'a mut SilencesPage,
}

impl Page<'_> {
    /// Runs `req` through the todo's dispatcher over the whole page. A
    /// cancel ends the proposal it cancelled in the same lending.
    pub fn dispatch(self, req: Request, todo: &mut TodoPage) -> RequestId {
        let mut page = self;
        page.app.generated.offer.listen(todo);
        let asked = todo.with_dispatcher(|dispatcher, _, now| {
            let asked = dispatcher.dispatch(&mut page, req, now);
            if let Some((pending, why)) = page.cancelled() {
                if let Err(e) = dispatcher.fail(pending, why.clone(), now) {
                    log::warn!("silences: the cancelled proposal did not end: {e}");
                }
                page.refuse(why);
            }
            asked
        });
        page.app.refresh_offer();
        asked
    }

    /// The proposal someone cancelled, when one was: its request and why
    /// it ends.
    fn cancelled(&self) -> Option<(RequestId, String)> {
        let pending = self.app.generated.proposing.pending.as_ref()?;
        let by = pending.cancelled.as_ref()?;
        Some((pending.id, format!("cancelled by {by}")))
    }

    /// No proposal is asked any more; the status says `why`.
    fn refuse(&mut self, why: String) {
        self.app.generated.proposing.said = Some(Said::Refused(why));
        self.app.generated.proposing.pending = None;
    }

    /// What happened to the proposal the Claude is answering since the
    /// page last looked: a cancel ends it now; the attempt or the seconds
    /// moved; or the answer arrived, the surface opens or takes the form
    /// in place and the request ends `completed` (or `failed`, with the
    /// reason). True when what the page shows moved.
    pub fn settle(&mut self, todo: &mut TodoPage) -> bool {
        if let Some((pending, why)) = self.cancelled() {
            let ended = todo.with_dispatcher(|d, _, now| d.fail(pending, why.clone(), now));
            if let Err(e) = ended {
                log::warn!("silences: the cancelled proposal did not end: {e}");
            }
            self.refuse(why);
            return true;
        }
        let Some(pending) = self.app.generated.proposing.pending.as_mut() else {
            return false;
        };
        let (moved, answer) = pending.poll();
        let Some(answer) = answer else {
            return moved;
        };
        if let Some(pending) = self.app.generated.proposing.pending.take() {
            self.arrived(pending, answer, todo);
        }
        true
    }

    /// The Claude answered `pending`: the surface shows the proposal, or
    /// the change is applied, and the request completes with it, the
    /// attempts, the seconds and the cost; or the request fails with why
    /// there is none.
    fn arrived(&mut self, pending: Pending, answer: Answer, todo: &mut TodoPage) {
        let Answer {
            proposed,
            attempts,
            took,
            cost_usd,
        } = answer;
        let id = pending.id;
        let done = match proposed.map_err(|e| e.to_string()) {
            Err(why) => Err(why),
            Ok(Proposal::Change(ops)) => {
                let app = self.app.snapshot().map_err(|e| e.to_string());
                let generated = &mut self.app.generated;
                app.and_then(|app| {
                    changing::arrived(
                        &mut generated.host,
                        generated.id.as_ref(),
                        &mut generated.proposing,
                        pending,
                        ops,
                        (attempts, took, cost_usd),
                        &app,
                    )
                })
            }
            Ok(Proposal::Surface(ir)) => {
                let Pending { intent, policy, .. } = pending;
                self.show(*ir, policy).map(|(surface, nodes)| {
                    let mut outcome = result(&surface, nodes);
                    outcome
                        .result
                        .insert("attempts".to_owned(), Value::Int(int(attempts)));
                    outcome
                        .result
                        .insert("seconds".to_owned(), Value::Num(took.as_secs_f64()));
                    if let Some(cost) = cost_usd {
                        outcome
                            .result
                            .insert("cost_usd".to_owned(), Value::Num(cost));
                    }
                    let proposing = &mut self.app.generated.proposing;
                    proposing.said = Some(Said::Answered {
                        intent,
                        attempts,
                        took,
                        cost_usd,
                    });
                    proposing.shown = true;
                    outcome
                })
            }
        };
        let ended = match done {
            Ok(outcome) => todo.with_dispatcher(|d, _, now| d.complete(id, outcome, now)),
            Err(why) => {
                self.app.generated.proposing.said = Some(Said::Refused(why.clone()));
                todo.with_dispatcher(|d, _, now| d.fail(id, why, now))
            }
        };
        if let Err(e) = ended {
            log::warn!("silences: the proposal's request did not end: {e}");
        }
    }

    /// Shows `ir`, verified under `policy`, over the app as it is: the
    /// surface opens, or takes the form in place when its id is open
    /// (`SurfaceHost::transform_under`, reconciled by `SemId`); the host
    /// keeps the policy for its changes. Its id and how many nodes it has.
    pub fn show(
        &mut self,
        ir: VerifiedIr,
        policy: GenerationPolicy,
    ) -> Result<(SemId, usize), String> {
        let app = self.app.snapshot().map_err(|e| e.to_string())?;
        let generated = &mut self.app.generated;
        let open = generated.host.get(&ir.ir().surface.id).is_some();
        let surface = if open {
            generated.host.transform_under(ir, &app, policy)
        } else {
            generated.host.open_under(ir, &app, policy)
        };
        let (id, nodes) = surface
            .map(|s| (s.id().clone(), s.nodes().count()))
            .map_err(|e| e.to_string())?;
        generated.id = Some(id.clone());
        Ok((id, nodes))
    }

    fn refused(&mut self, why: String) -> Execution {
        self.app.generated.proposing.said = Some(Said::Refused(why.clone()));
        Execution::Failed(why)
    }

    /// A proposal for the intent, as `author` wrote it, over the app as
    /// it is (the fixed screen's registry, never the row's) and the
    /// surface shown: with a surface open, a change of it first
    /// (`changing::change`), else a surface. By the rules it is verified
    /// and shown now, and the request is done; by the Claude the worker
    /// starts, with everything it needs owned, and the request `id` stays
    /// open until `settle` ends it.
    fn propose(&mut self, id: RequestId, author: &Actor) -> Execution {
        let intent = self.app.generated.proposing.intent.text().trim().to_owned();
        let app = match self.app.snapshot() {
            Ok(app) => app,
            Err(e) => return Execution::Failed(e.to_string()),
        };
        let generated = &mut self.app.generated;
        if let Some(open) = generated.id.clone() {
            let (host, proposing) = (&mut generated.host, &mut generated.proposing);
            if let Changing::Ran(done) =
                changing::change(host, &open, proposing, id, author, app.clone())
            {
                return done;
            }
        }
        let caps = Capabilities::from_snapshot(&app);
        let policy = GenerationPolicy::by(author.clone());
        let generated = &self.app.generated;
        let current: Option<VerifiedIr> = generated
            .id
            .as_ref()
            .and_then(|id| generated.host.get(id))
            .map(Surface::verified)
            .cloned();
        match generated.proposing.chosen() {
            Proposer::Rules => {
                let ask = Ask {
                    intent: &intent,
                    caps: &caps,
                    app: &app,
                    current: current.as_ref(),
                    policy: &policy,
                };
                let proposed = propose_verified(&mut Rules, &ask).map_err(|e| e.to_string());
                match proposed.and_then(|ir| self.show(ir, policy.clone())) {
                    Ok((surface, nodes)) => {
                        let proposing = &mut self.app.generated.proposing;
                        proposing.said = Some(Said::Proposed(intent));
                        proposing.shown = true;
                        Execution::Done(result(&surface, nodes))
                    }
                    Err(why) => self.refused(why),
                }
            }
            Proposer::Claude => {
                if generated.proposing.waiting() {
                    let why = "a proposal is already being asked of the Claude: wait for it, \
                               or cancel it";
                    return self.refused(why.to_owned());
                }
                let job = Job::Propose(Box::new(current));
                let started = generated.proposing.claude.start(
                    id,
                    intent,
                    app,
                    caps,
                    job,
                    policy,
                    author.clone(),
                );
                match started {
                    Ok(pending) => {
                        self.app.generated.proposing.pending = Some(pending);
                        Execution::Started
                    }
                    Err(why) => self.refused(why),
                }
            }
        }
    }

    /// The person's answer to the composite offered: accepted, the
    /// sequence becomes a composite of the app (its action is declared on
    /// the nodes it runs on from the next projection) and, with a surface
    /// open, its button is promoted there by the host, as `by` asked it,
    /// beside the controls of its steps; declined, it goes and is not
    /// offered again in this session. Only a person gets here: both
    /// answers are `PERSON_ONLY`.
    fn answered(&mut self, answer: Reply, by: &Actor) -> Execution {
        let mut outcome = Outcome::default();
        let offer = &mut self.app.generated.offer;
        match answer {
            Reply::Install => {
                let app = match self.app.snapshot() {
                    Ok(app) => app,
                    Err(e) => return Execution::Failed(e.to_string()),
                };
                let composer = &mut self.app.generated.offer.composer;
                let composite = match composer.accept(&app) {
                    Ok(composite) => composite.clone(),
                    Err(why) => return Execution::Failed(why),
                };
                let name = Value::Text(composite.name.to_string());
                outcome.result.insert("installed".to_owned(), name);
                let label = Value::Text(composite.label.clone());
                outcome.result.insert("label".to_owned(), label);
                match self.promote(&composite, by) {
                    Some(Ok(line)) => {
                        outcome
                            .result
                            .insert("promoted".to_owned(), Value::Text(line));
                    }
                    Some(Err(why)) => {
                        log::info!("silences: the composite is not promoted: {why}");
                        outcome
                            .result
                            .insert("not_promoted".to_owned(), Value::Text(why));
                    }
                    None => {}
                }
            }
            Reply::Dismiss => match offer.composer.decline() {
                Some(shape) => {
                    let steps = shape.steps.len();
                    outcome
                        .result
                        .insert("dismissed".to_owned(), Value::Int(int(steps)));
                }
                None => return Execution::Failed("nothing is offered".to_owned()),
            },
        }
        Execution::Done(outcome)
    }

    /// With a surface open, a button for `composite` in it, beside the
    /// controls of its steps, as the host promotes an action the app
    /// declares (`uir::Op::Promote`, verified again, with its inverse in
    /// the history): the line of the operation, or why there is none.
    /// `None` without a surface.
    fn promote(&mut self, composite: &Composite, by: &Actor) -> Option<Result<String, String>> {
        let id = self.app.generated.id.clone()?;
        let current = self.app.generated.host.current(&id)?;
        let root = &current.verified().ir().surface.root;
        let into = match beside(root, composite) {
            Some(into) => into,
            None if composite.shape.collection().is_none() => root.sem.clone(),
            None => return Some(Err("the surface shows none of its steps".to_owned())),
        };
        let target = composite.template()?;
        let op = uir::Op::Promote {
            target,
            action: composite.name.clone(),
            into,
        };
        let line = op.to_string();
        let app = match self.app.snapshot() {
            Ok(app) => app,
            Err(e) => return Some(Err(e.to_string())),
        };
        let host = &mut self.app.generated.host;
        Some(
            host.apply(&id, op, by.clone(), &app)
                .map(|_| line)
                .map_err(|e| e.to_string()),
        )
    }

    /// The surface is dismissed: out of the projection at once, its
    /// picture fading where it was (`SurfaceHost::close`); the row says
    /// nothing until the next proposal.
    fn close(&mut self) -> Execution {
        let generated = &mut self.app.generated;
        let Some(id) = generated.id.take() else {
            return Execution::Failed("no surface is open".to_owned());
        };
        generated.host.close(&id);
        generated.proposing.said = None;
        generated.proposing.shown = false;
        let mut outcome = Outcome::default();
        outcome
            .result
            .insert("closed".to_owned(), Value::Text(id.to_string()));
        Execution::Done(outcome)
    }
}

/// The whole page projected: the fixed screen, then its generated block
/// (the surface, the row, the close).
pub(in crate::view::apps::silences) fn project_all(app: &SilencesPage, b: &mut SnapshotBuilder) {
    app.project(b);
    app.generated.project(b);
}

impl Semantics for Page<'_> {
    fn project(&self, b: &mut SnapshotBuilder) {
        project_all(self.app, b);
    }
}

impl Operable for Page<'_> {
    fn execute(&mut self, id: RequestId, req: &Request) -> Execution {
        if self.app.generated.is_close(req) {
            return self.close();
        }
        if let Some(answer) = self.app.generated.offer.answer(req) {
            return self.answered(answer, &req.actor);
        }
        let Some(op) = self.app.generated.proposing.op(req) else {
            return self.app.execute(id, req);
        };
        let done = || Execution::Done(Outcome::default());
        let proposing = &mut self.app.generated.proposing;
        match op {
            Op::Write(text) => {
                proposing.intent.set_text(&text);
                done()
            }
            Op::Propose => self.propose(id, &req.actor),
            Op::Choose(name) => {
                if proposing.proposer.select_label(&name) {
                    done()
                } else {
                    Execution::Failed(format!("no proposer is named {name:?}"))
                }
            }
            Op::ChooseAt(at) => {
                if proposing.proposer.select_index(at) {
                    done()
                } else {
                    Execution::Failed(format!("no proposer at {at}"))
                }
            }
            Op::Open => {
                proposing.proposer.open();
                done()
            }
            Op::Close => {
                proposing.proposer.close();
                done()
            }
            Op::Cancel => match proposing.pending.as_mut() {
                Some(pending) => {
                    pending.cancel(req.actor.clone());
                    let mut outcome = Outcome::default();
                    outcome
                        .result
                        .insert("cancelled".to_owned(), Value::Text(pending.id.to_string()));
                    Execution::Done(outcome)
                }
                None => Execution::Failed("nothing is being asked".to_owned()),
            },
        }
    }
}

/// Staging, for the regression harness only (`examples/regress`): the
/// Silences page in a state a person reaches with ffmpeg and a model,
/// reached without either.
impl ShowcaseView {
    /// Puts in the Silences page's cache a frame for every instant it
    /// shows or its surface binds and it lacks: the instant shown, the
    /// result's, the middle of each cut, and each frame ref of the
    /// surface's images. `frame` makes each one from its instant (ms), so
    /// the page asks the worker for none and no ffmpeg runs. The instants
    /// filled, in that order.
    pub fn stage_silences_frames(
        &mut self,
        frame: &mut dyn FnMut(u64) -> Result<ImageHandle, String>,
    ) -> Vec<u64> {
        let page = &mut self.panes.silences;
        let mut wanted = vec![ms(page.model.shown()), ms(page.result_instant())];
        wanted.extend(page.model.cuts().iter().map(|c| ms(c.segment().middle())));
        if let Some(video) = page.ids.as_ref().map(|ids| ids.video.clone()) {
            let sources = page.generated.host.sources();
            wanted.extend(sources.iter().filter_map(|s| instant_of(&video, s)));
        }
        let mut filled = Vec::new();
        for at in wanted {
            if let std::collections::hash_map::Entry::Vacant(e) = page.frames.entry(at) {
                e.insert(frame(at));
                filled.push(at);
            }
        }
        filled
    }

    /// Shows the IR `json` on the Silences page as a proposal shows it:
    /// verified by the verifier against the app's registry (the fixed
    /// screen's projection, never the row's) and opened beside the fixed
    /// screen, or transformed in place. The row says nothing of it: no
    /// proposer was asked. The surface's id, or every reason.
    pub fn stage_silences_ir(&mut self, json: &str) -> Result<SemId, String> {
        let page = &mut self.panes.silences;
        let app = page.snapshot().map_err(|e| e.to_string())?;
        let caps = Capabilities::from_snapshot(&app);
        let ir = verify_json(json, &caps, &GenerationPolicy::default()).map_err(|errors| {
            let all: Vec<String> = errors.iter().map(ToString::to_string).collect();
            all.join("; ")
        })?;
        Page { app: page }
            .show(ir, GenerationPolicy::default())
            .map(|(id, _)| id)
    }

    /// Applies `op`, asked by `by`, to the surface the Silences page
    /// shows, as the host applies one (`SurfaceHost::apply`): checked and
    /// verified again against the app's registry, under the policy the
    /// surface was shown under, reconciled by `SemId`, and kept in the
    /// host's history with its inverse. The row says nothing: a person or
    /// an agent asks for a change through the row's `propose`. The
    /// operation in a line, or why it made no surface.
    pub fn stage_silences_op(&mut self, op: uir::Op, by: Actor) -> Result<String, String> {
        let page = &mut self.panes.silences;
        let app = page.snapshot().map_err(|e| e.to_string())?;
        let generated = &mut page.generated;
        let id = generated.id.clone().ok_or("no surface is open")?;
        let line = op.to_string();
        generated
            .host
            .apply(&id, op, by, &app)
            .map_err(|e| e.to_string())?;
        Ok(line)
    }
}
