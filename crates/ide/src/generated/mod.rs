//! The generated surface of the ide (task-96): the intent line and the
//! `SurfaceHost` that holds the surface a proposal opened, in a column
//! beside the fixed views, which stay as they are (the workspace is only
//! given less width, by its own `resize`).
//!
//! The line under the region `ide_surface`, projected as the Silences
//! projects its own (`crates/showcase/src/view/apps/silences/generated.rs`):
//! `ide_surface.intent` a text input (`set_value`); `ide_surface.proposer`
//! a select of who proposes (`rules`, and `claude` where the program is,
//! the rules chosen first); `ide_surface.propose` a button whose
//! `propose` reads the intent and the proposer; `ide_surface.status` what
//! the last proposal came to; `ide_surface.close` the button that
//! dismisses the surface, enabled while one is open.
//!
//! Every capability here runs through the workbench's one dispatcher
//! (`crate::workbench`, `generated.rs` there): this module is the block
//! itself, its widgets, what they project and what a request asks of them.
//! Where it sits and what a pointer does on it is `row.rs`.

pub(crate) mod row;

use comps::prelude::{Button, ButtonSize, Select, TextField, Wake};
use sem::{Request, Role, SemError, SemId, SnapshotBuilder, Value};
use surface::SurfaceHost;

/// The region the line projects under.
pub(crate) const REGION: &str = "ide_surface";

/// The proposers as the select names them.
const RULES: &str = "rules";

/// The ids the line projects and binds.
pub(crate) struct Ids {
    pub region: SemId,
    pub intent: SemId,
    pub proposer: SemId,
    pub propose: SemId,
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
}

/// What the last proposal came to.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Said {
    /// The surface shows what the rules proposed for this intent.
    Proposed(String),
    /// Why there is no proposal: the rules' error or the verifier's.
    Refused(String),
}

impl Said {
    fn text(&self) -> String {
        match self {
            Said::Proposed(intent) => format!("proposed by the rules for \"{intent}\""),
            Said::Refused(why) => why.clone(),
        }
    }
}

/// What a request accepted against the workbench asks of the line.
pub(crate) enum Op {
    Write(String),
    Propose,
    Choose(String),
    ChooseAt(usize),
    Unfold,
    Fold,
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
    pub(crate) close: Button,
    pub said: Option<Said>,
}

impl Generated {
    pub fn new() -> Self {
        let ids = Ids::new()
            .inspect_err(|e| log::error!("ide: the surface line's ids do not parse: {e}"))
            .ok();
        let mut intent = TextField::new("what should it show?");
        let mut proposer = Select::new([RULES], 0);
        let mut propose = Button::new("propose").disabled(true);
        let mut close = Button::new("close").size(ButtonSize::Sm).disabled(true);
        if let Some(ids) = &ids {
            intent = intent.sem(ids.intent.clone());
            proposer = proposer.sem(ids.proposer.clone());
            propose = propose.sem(ids.propose.clone());
            close = close.sem(ids.close.clone());
        }
        Self {
            host: SurfaceHost::new(),
            open: None,
            ids,
            intent,
            proposer,
            propose,
            close,
            said: None,
        }
    }

    fn blank(&self) -> bool {
        self.intent.text().trim().is_empty()
    }

    /// Who proposes now.
    pub fn chosen(&self) -> Proposer {
        Proposer::Rules
    }

    /// The status line: what the last proposal came to.
    pub fn message(&self) -> String {
        self.said.as_ref().map_or_else(String::new, Said::text)
    }

    /// The buttons follow the state: propose only with an intent, close
    /// only with a surface open.
    pub fn refresh(&mut self) {
        self.propose.disabled = self.blank();
        self.close.disabled = self.open.is_none();
    }

    /// The surface, then the line: the region; the intent field; the
    /// proposer; the propose button (its `propose` reads the intent and
    /// the proposer, and runs only with an intent); the status with the
    /// message; and the close button, enabled while a surface is open,
    /// its action `close`. After the app's own projection: the surface's
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
        let ready = !self.blank();
        let mut propose = b.node(&ids.propose, Role::Button, "propose");
        propose.parent(&ids.region).enabled(ready);
        propose
            .action("propose")
            .enabled(ready)
            .reads(format!("{}:text", ids.intent))
            .reads(format!("{}:selected", ids.proposer));
        let mut status = b.node(&ids.status, Role::Status, "proposal");
        status
            .parent(&ids.region)
            .state("message", self.message())
            .state("busy", false);
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
            _ => None,
        }
    }

    /// The caret of the intent field, and the surfaces' own motion (a
    /// transition, a surface fading out after its close).
    pub fn wake(&mut self, dt: f32) -> Wake {
        self.intent.wake(dt).merge(self.host.wake(dt))
    }
}
