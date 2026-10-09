//! The generated surface of the ide (task-96): the intent line, in a
//! strip under the fixed views, and the `SurfaceHost` that holds the
//! surface a proposal opened, in a column beside them while it is open.
//! The fixed views stay as they are: the workspace keeps the window's
//! width with no surface open and is only given less room, by its own
//! `resize` (phase b).
//!
//! The line under the region `ide_surface`, projected as the Silences
//! projects its own (`crates/showcase/src/view/apps/silences/generated.rs`):
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

pub(crate) mod claude;
pub(crate) mod row;
pub(crate) mod scope;

use std::time::Duration;

use comps::prelude::{Button, ButtonSize, Select, TextField, Wake};
use propose::{Llm, Model};
use sem::{Request, Role, SemError, SemId, SnapshotBuilder, Value};
use surface::SurfaceHost;

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
