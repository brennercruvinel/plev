//! The generated surface of the ide (task-96): the intent line, in a
//! strip under the fixed views, and the `SurfaceHost` that holds the
//! surface a proposal opened, in a column beside them while it is open.
//! The fixed views stay as they are: the workspace keeps the window's
//! width with no surface open and is only given less room, by its own
//! `resize` (phase b).
//!
//! The line under the region `ide_surface`, projected as the Silences
//! projects its own (`apps/uigallery/src/views/apps/silences/generative/mod.rs`):
//! `ide_surface.intent` a text input (`set_value`); `ide_surface.proposer`
//! a select of who proposes (`rules`, and `claude` where the program is,
//! the rules chosen first); `ide_surface.propose` a button whose
//! `propose` reads the intent and the proposer; `ide_surface.cancel` a
//! button enabled only while the Claude is asked; `ide_surface.status`
//! what the last proposal came to (and, while the Claude is asked, the
//! attempt and the seconds); `ide_surface.close` the button that
//! dismisses the surface, enabled while one is open.
//!
//! The rules propose inside the request; the Claude off the workbench's
//! thread (`claude.rs`), shown only what the review needs (`scope.rs`).
//! Every capability here runs through the workbench's one dispatcher
//! (`crate::workbench`, `generated.rs` there): this module is the block
//! itself, its widgets, what they project and what a request asks of them.
//! Where it sits and what a pointer does on it is `row.rs`.

mod asking;
pub(crate) mod claude;
mod column;
pub(crate) mod row;
pub(crate) mod scope;

use std::time::Duration;

use ::claude::ClaudeCli;
use accord::{
    Actor, Execution, Outcome, Request, RequestId, Role, SemError, SemErrors, SemId, Snapshot,
    SnapshotBuilder, Value,
};
use engine::widgets::prelude::{Button, ButtonSize, Select, TextField, Wake};
use offers::{Ask, Llm, Model, Rules, propose_verified};
use plevir::{Capabilities, GenerationPolicy, VerifiedIr};
use screen::SurfaceHost;

use crate::capability::Workbench;
use claude::{Claude, POLL, Pending};

/// The region the line projects under.
pub(crate) const REGION: &str = "ide_surface";

/// The proposers as the select names them.
const RULES: &str = "rules";
const CLAUDE: &str = "claude";

/// The ids the line projects and binds.
pub(crate) struct Ids {
    pub region: SemId,
    pub intent: SemId,
    pub proposer: SemId,
    pub propose: SemId,
    pub cancel: SemId,
    pub close: SemId,
    pub status: SemId,
}

impl Ids {
    fn new() -> Result<Self, SemError> {
        let region = SemId::name(REGION)?;
        Ok(Self {
            intent: region.child("intent")?,
            proposer: region.child("proposer")?,
            propose: region.child("propose")?,
            cancel: region.child("cancel")?,
            close: region.child("close")?,
            status: region.child("status")?,
            region,
        })
    }

    /// The option row `<proposer>[<index>]` behind `target`, projected
    /// while the select is open.
    fn option_of(&self, target: &SemId) -> Option<usize> {
        target
            .as_str()
            .strip_prefix(self.proposer.as_str())?
            .strip_prefix('[')?
            .strip_suffix(']')?
            .parse()
            .ok()
    }
}

/// Who the select chooses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Proposer {
    Rules,
    Claude,
}

/// What the last proposal came to.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Said {
    /// The surface shows what the rules proposed for this intent.
    Proposed(String),
    /// The surface shows what the Claude proposed for this intent: in how
    /// many attempts, how long it took, what it cost (as the child counts
    /// it; none when it did not say).
    Answered {
        intent: String,
        attempts: usize,
        took: Duration,
        cost_usd: Option<f64>,
    },
    /// Why there is no proposal: the rules' error, the verifier's, the
    /// model's, or who cancelled it.
    Refused(String),
}

impl Said {
    fn text(&self) -> String {
        match self {
            Said::Proposed(intent) => format!("proposed by the rules for \"{intent}\""),
            Said::Answered {
                intent,
                attempts,
                took,
                cost_usd,
            } => {
                let cost =
                    cost_usd.map_or_else(|| "cost unknown".to_owned(), |c| format!("{c:.3} USD"));
                format!(
                    "proposed by the Claude for \"{intent}\" in {:.1} s, {attempts} attempts, {cost}",
                    took.as_secs_f64()
                )
            }
            Said::Refused(why) => why.clone(),
        }
    }
}

fn int(n: u64) -> i64 {
    i64::try_from(n).unwrap_or(i64::MAX)
}

/// What a request accepted against the workbench asks of the line.
pub(crate) enum Op {
    Write(String),
    Propose,
    Choose(String),
    ChooseAt(usize),
    Unfold,
    Fold,
    /// The proposal the Claude is answering is cancelled.
    Cancel,
    /// The surface open is dismissed.
    Dismiss,
}

/// The block: the host, the surface it shows, and the line.
pub(crate) struct Generated {
    pub host: SurfaceHost,
    /// The surface open (the IR's id), when one is.
    pub open: Option<SemId>,
    /// `None` only if the constant ids do not parse (a test proves they
    /// do), which leaves the line without semantics.
    pub ids: Option<Ids>,
    pub intent: TextField,
    pub proposer: Select,
    pub(crate) propose: Button,
    pub(crate) cancel: Button,
    pub(crate) close: Button,
    pub said: Option<Said>,
    /// The Claude the line may ask, when the program is there.
    pub claude: Claude,
    /// The proposal the Claude is answering, when one is.
    pub pending: Option<Pending>,
}

impl Generated {
    /// The line over `claude`: the select offers it when there is a
    /// program, and chooses the rules first either way.
    pub fn new(claude: Claude) -> Self {
        let ids = Ids::new()
            .inspect_err(|e| log::error!("ide: the surface line's ids do not parse: {e}"))
            .ok();
        let mut intent = TextField::new("what should it show?");
        let mut proposer = if claude.is_some() {
            Select::new([RULES, CLAUDE], 0)
        } else {
            Select::new([RULES], 0)
        };
        let mut propose = Button::new("propose").disabled(true);
        let mut cancel = Button::new("cancel").size(ButtonSize::Sm).disabled(true);
        let mut close = Button::new("close").size(ButtonSize::Sm).disabled(true);
        if let Some(ids) = &ids {
            intent = intent.sem(ids.intent.clone());
            proposer = proposer.sem(ids.proposer.clone());
            propose = propose.sem(ids.propose.clone());
            cancel = cancel.sem(ids.cancel.clone());
            close = close.sem(ids.close.clone());
        }
        Self {
            host: SurfaceHost::new(),
            open: None,
            ids,
            intent,
            proposer,
            propose,
            cancel,
            close,
            said: None,
            claude,
            pending: None,
        }
    }

    fn blank(&self) -> bool {
        self.intent.text().trim().is_empty()
    }

    /// True while the Claude is asked.
    pub fn waiting(&self) -> bool {
        self.pending.is_some()
    }

    /// Who proposes now.
    pub fn chosen(&self) -> Proposer {
        match self.proposer.selected_label() {
            Some(CLAUDE) => Proposer::Claude,
            _ => Proposer::Rules,
        }
    }

    /// The status line: the wait, or what the last proposal came to.
    pub fn message(&self) -> String {
        match &self.pending {
            Some(p) => format!(
                "asking the Claude for \"{}\": attempt {} of {}, {} s",
                p.intent,
                p.attempt().max(1),
                Llm::<&mut dyn Model>::ATTEMPTS,
                p.elapsed().as_secs()
            ),
            None => self.said.as_ref().map_or_else(String::new, Said::text),
        }
    }

    /// The buttons follow the state: propose only with an intent and
    /// nothing asked, cancel only while asking, close only with a surface
    /// open.
    pub fn refresh(&mut self) {
        self.propose.disabled = self.blank() || self.waiting();
        self.cancel.disabled = !self.waiting();
        self.close.disabled = self.open.is_none();
    }

    /// The surface, then the line: the region; the intent field; the
    /// proposer; the propose button (its `propose` reads the intent and
    /// the proposer, and runs only with an intent and nothing asked); the
    /// cancel button (enabled only while the Claude is asked); the status
    /// with the message, `busy`, and while asking or after the Claude
    /// answered the `attempt`, the `seconds` and the `cost_usd` when
    /// known; and the close button, enabled while a surface is open, its
    /// action `close`. After the app's own projection: the surface's
    /// actions read the app's state.
    pub fn project(&self, b: &mut SnapshotBuilder) {
        self.host.project(b);
        let Some(ids) = &self.ids else {
            return;
        };
        b.node(&ids.region, Role::Region, "propose a surface");
        self.intent
            .project(b, &ids.intent, "intent", Some(&ids.region));
        self.proposer
            .project(b, &ids.proposer, "proposer", Some(&ids.region));
        let waiting = self.waiting();
        let ready = !self.blank() && !waiting;
        let mut propose = b.node(&ids.propose, Role::Button, "propose");
        propose.parent(&ids.region).enabled(ready);
        propose
            .action("propose")
            .enabled(ready)
            .reads(format!("{}:text", ids.intent))
            .reads(format!("{}:selected", ids.proposer));
        let mut cancel = b.node(&ids.cancel, Role::Button, "cancel");
        cancel.parent(&ids.region).enabled(waiting);
        cancel.action("activate").enabled(waiting);
        let mut status = b.node(&ids.status, Role::Status, "proposal");
        status
            .parent(&ids.region)
            .state("message", self.message())
            .state("busy", waiting);
        match (&self.pending, &self.said) {
            (Some(p), _) => {
                status
                    .state("attempt", int(p.attempt() as u64))
                    .state("seconds", int(p.elapsed().as_secs()));
            }
            (
                None,
                Some(Said::Answered {
                    attempts,
                    took,
                    cost_usd,
                    ..
                }),
            ) => {
                status
                    .state("attempt", int(*attempts as u64))
                    .state("seconds", int(took.as_secs()));
                if let Some(cost) = cost_usd {
                    status.state("cost_usd", *cost);
                }
            }
            _ => {}
        }
        let open = self.open.is_some();
        let mut close = b.node(&ids.close, Role::Button, "close the surface");
        close.parent(&ids.region).enabled(open);
        close.action("close").enabled(open);
        if self.intent.is_focused() {
            b.focus(&ids.intent);
        }
    }

    /// What `req` asks of the line, when it aims at it.
    pub fn op(&self, req: &Request) -> Option<Op> {
        let ids = self.ids.as_ref()?;
        let target = &req.target;
        match req.action.as_str() {
            "set_value" if *target == ids.intent => match &req.param {
                Some(Value::Text(text)) => Some(Op::Write(text.clone())),
                _ => None,
            },
            "propose" if *target == ids.propose => Some(Op::Propose),
            "select" if *target == ids.proposer => match &req.param {
                Some(Value::Text(name)) => Some(Op::Choose(name.clone())),
                _ => None,
            },
            "select" => ids.option_of(target).map(Op::ChooseAt),
            "open" if *target == ids.proposer => Some(Op::Unfold),
            "close" if *target == ids.proposer => Some(Op::Fold),
            "close" if *target == ids.close => Some(Op::Dismiss),
            "activate" if *target == ids.cancel => Some(Op::Cancel),
            _ => None,
        }
    }

    /// The caret of the intent field, the surfaces' own motion (a
    /// transition, a surface fading out after its close), and while the
    /// Claude is asked a look at the channel every `POLL`.
    pub fn wake(&mut self, dt: f32) -> Wake {
        let own = self.intent.wake(dt).merge(self.host.wake(dt));
        if self.waiting() {
            own.merge(Wake::after(POLL))
        } else {
            own
        }
    }
}

// The workbench's side of the generated surface (task-96,
// `crate::generated`): the block turned on; the workspace's own
// projection apart from it, the registry a proposal binds and the state a
// surface shows; what a request asks of the line, run first by `execute`
// (propose, close, the field, the proposer); a request aimed at a node of
// an open surface routed to the capability the IR bound before the
// dispatcher (`route`), so a person's click, an agent through the port
// and a test reach the same one; and the window laid out with the line's
// strip and an open surface's column. A person's pointer and keys on
// them, and their drawing, are `column.rs`.
//
// The rules propose inside `execute`, in a fraction of a millisecond, and
// the request ends at once, `completed` with the surface and how many
// nodes it has, or `failed` with why there is none (the line says it
// too). Only a `VerifiedIr` opens: `propose::propose_verified` is the way.
/// The outcome of a proposal shown: the surface and how many nodes it
/// has.
pub(crate) fn shown(surface: &SemId, nodes: usize) -> Outcome {
    let mut outcome = Outcome::default();
    outcome
        .result
        .insert("surface".to_owned(), Value::Text(surface.to_string()));
    let nodes = i64::try_from(nodes).unwrap_or(i64::MAX);
    outcome.result.insert("nodes".to_owned(), Value::Int(nodes));
    outcome
}

impl Workbench {
    /// Turns on the intent line and the host of generated surfaces: the
    /// line in a strip along the bottom of the window, under the fixed
    /// views, which keep the window's width; a surface in a column at the
    /// right while it is open. Off by default: a workbench without it
    /// projects, draws and runs as before. The ide's window turns it on. The Claude is offered when
    /// its program is found (`claude`, or what `PLEV_CLAUDE` names), never
    /// run to find it; the rules are chosen first either way.
    pub fn host_surfaces(&mut self) {
        self.host_surfaces_over(Claude::find());
    }

    /// [`Workbench::host_surfaces`], with this Claude to offer, or none
    /// (the rules alone): a test gives the repository's fake here.
    pub fn host_surfaces_with(&mut self, claude: Option<ClaudeCli>) {
        self.host_surfaces_over(Claude::with(claude));
    }

    fn host_surfaces_over(&mut self, claude: Claude) {
        if self.generated.is_none() {
            self.generated = Some(Box::new(Generated::new(claude)));
            self.lay_out();
        }
    }

    /// Whether the line and the host are on.
    pub fn hosts_surfaces(&self) -> bool {
        self.generated.is_some()
    }

    /// The surfaces open, when the block is on.
    pub fn surfaces(&self) -> Option<&SurfaceHost> {
        self.generated.as_ref().map(|g| &g.host)
    }

    /// The id of the surface open, when one is.
    pub fn surface_open(&self) -> Option<&SemId> {
        self.generated.as_ref().and_then(|g| g.open.as_ref())
    }

    /// The workspace's own projection, without the line and the surface:
    /// the registry every proposal is verified against
    /// (`uir::Capabilities::from_snapshot`) and the state a surface shows.
    /// With the block off it is [`Workbench::snapshot`].
    pub fn app_snapshot(&self) -> Result<Snapshot, SemErrors> {
        let mut b = SnapshotBuilder::new();
        self.project_app(&mut b);
        b.finish()
    }

    /// The window is `w` by `h` logical pixels: the workspace takes it,
    /// less the line's strip and an open surface's column when the block
    /// is on.
    pub fn resize(&mut self, w: f32, h: f32) {
        self.window = (w, h);
        match self.generated {
            Some(_) => self.lay_out(),
            None => self.workspace.resize(w, h),
        }
    }

    /// The workspace gets the window less the line's strip and an open
    /// surface's column, when the block is on; nothing moves otherwise.
    pub(crate) fn lay_out(&mut self) {
        let Some(g) = &self.generated else {
            return;
        };
        let (w, h) = self.window;
        let (ws, hs) = g.workspace_size(w, h, self.workspace.theme());
        if (self.workspace.vw, self.workspace.vh) != (ws, hs) {
            self.workspace.resize(ws, hs);
        }
    }

    /// `req` as the capability it asks for when it aims at a node of an
    /// open surface; as it is otherwise.
    pub(crate) fn route(&self, req: Request) -> Request {
        match &self.generated {
            Some(g) => g.host.route(req),
            None => req,
        }
    }

    /// Every open surface reads the workspace again when it moved.
    pub(crate) fn sync_surfaces(&mut self) {
        if self.generated.as_ref().is_none_or(|g| g.host.is_empty()) {
            return;
        }
        match self.app_snapshot() {
            Ok(app) => {
                if let Some(g) = self.generated.as_mut() {
                    g.host.sync(&app);
                }
            }
            Err(e) => log::warn!("ide: the projection does not validate: {e}"),
        }
    }

    /// What `req` asks of the line, run now; `None` when it aims
    /// elsewhere (the workspace, whose `execute` runs it).
    pub(crate) fn execute_generated(&mut self, id: RequestId, req: &Request) -> Option<Execution> {
        let g = self.generated.as_mut()?;
        let op = g.op(req)?;
        let done = Execution::Done(Outcome::default());
        Some(match op {
            Op::Write(text) => {
                g.intent.set_text(&text);
                done
            }
            Op::Propose => self.propose(id, &req.actor),
            Op::Choose(name) => {
                if g.proposer.select_label(&name) {
                    done
                } else {
                    Execution::Failed(format!("no proposer is named {name:?}"))
                }
            }
            Op::ChooseAt(at) => {
                if g.proposer.select_index(at) {
                    done
                } else {
                    Execution::Failed(format!("no proposer at {at}"))
                }
            }
            Op::Unfold => {
                g.proposer.open();
                done
            }
            Op::Fold => {
                g.proposer.close();
                done
            }
            Op::Cancel => match g.pending.as_mut() {
                Some(pending) => {
                    pending.cancel(req.actor.clone());
                    let mut outcome = Outcome::default();
                    let what = Value::Text(pending.id.to_string());
                    outcome.result.insert("cancelled".to_owned(), what);
                    Execution::Done(outcome)
                }
                None => Execution::Failed("nothing is being asked".to_owned()),
            },
            Op::Dismiss => self.dismiss(),
        })
    }

    /// A proposal for the intent, as `author` wrote it, over the
    /// workspace as it is (its own registry, never the line's) and the
    /// surface shown. By the rules it is verified and shown now, and the
    /// request is done; by the Claude a worker starts with what the
    /// review scopes for it (`asking.rs`), and the request `id` stays
    /// open until the answer ends it.
    fn propose(&mut self, id: RequestId, author: &Actor) -> Execution {
        let app = match self.app_snapshot() {
            Ok(app) => app,
            Err(e) => return Execution::Failed(e.to_string()),
        };
        let Some(g) = self.generated.as_ref() else {
            return Execution::Failed("no surface is hosted here".to_owned());
        };
        let intent = g.intent.text().trim().to_owned();
        let caps = Capabilities::from_snapshot(&app);
        let policy = GenerationPolicy::by(author.clone());
        let current: Option<VerifiedIr> = g
            .open
            .as_ref()
            .and_then(|id| g.host.current(id))
            .map(|asked| asked.verified().clone());
        let ask = Ask {
            intent: &intent,
            caps: &caps,
            app: &app,
            current: current.as_ref(),
            policy: &policy,
        };
        let proposed = propose_verified(&mut Rules, &ask).map_err(|e| e.to_string());
        if g.chosen() == Proposer::Claude {
            return self.ask_claude(id, intent, &app, proposed, policy);
        }
        match proposed.and_then(|ir| self.show(ir, policy, &app)) {
            Ok((surface, nodes)) => {
                self.say(Said::Proposed(intent));
                Execution::Done(shown(&surface, nodes))
            }
            Err(why) => {
                self.say(Said::Refused(why.clone()));
                Execution::Failed(why)
            }
        }
    }

    /// What the last proposal came to, on the line; a message longer than
    /// the lines the strip holds takes the room it needs now.
    pub(crate) fn say(&mut self, said: Said) {
        if let Some(g) = self.generated.as_mut() {
            g.said = Some(said);
        }
        self.lay_out();
    }

    /// Shows `ir`, verified under `policy`, over `app`: the surface
    /// opens, or takes the form in place when its id is open (reconciled
    /// by `SemId`), and the workspace makes room for it. One surface at a
    /// time: an open one of another id is dismissed first. Its id and how
    /// many nodes it has.
    pub(crate) fn show(
        &mut self,
        ir: VerifiedIr,
        policy: GenerationPolicy,
        app: &Snapshot,
    ) -> Result<(SemId, usize), String> {
        let g = self.generated.as_mut().ok_or("no surface is hosted here")?;
        let id = &ir.ir().surface.id;
        if let Some(other) = g.open.take_if(|open| open != id) {
            g.host.close(&other);
        }
        let open = g.host.get(id).is_some();
        let surface = if open {
            g.host.transform_under(ir, app, policy)
        } else {
            g.host.open_under(ir, app, policy)
        };
        let (id, nodes) = surface
            .map(|s| (s.id().clone(), s.nodes().count()))
            .map_err(|e| e.to_string())?;
        g.open = Some(id.clone());
        self.lay_out();
        Ok((id, nodes))
    }

    /// The surface is dismissed: out of the projection, the routing and
    /// the hit test at once, its picture fading where it was
    /// (`SurfaceHost::close`), its column gone and the workspace back to
    /// the window's width, and the line says nothing until the next
    /// proposal.
    fn dismiss(&mut self) -> Execution {
        let Some(g) = self.generated.as_mut() else {
            return Execution::Failed("no surface is hosted here".to_owned());
        };
        let Some(id) = g.open.take() else {
            return Execution::Failed("no surface is open".to_owned());
        };
        g.host.close(&id);
        g.said = None;
        self.lay_out();
        let mut outcome = Outcome::default();
        outcome
            .result
            .insert("closed".to_owned(), Value::Text(id.to_string()));
        Execution::Done(outcome)
    }
}
