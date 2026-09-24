//! The Surface page's own capabilities (task-69 phase 3a): an intent in
//! text and a button that proposes a surface for it.
//!
//! `surface_page.intent` is a text input (`set_value`) and
//! `surface_page.propose` a button whose `propose` reads the intent: the
//! deterministic proposer (`propose::Rules`) turns the intent, the todo's
//! registry, its snapshot and the surface shown into an IR, the verifier
//! admits it (`propose::propose_verified`), and the surface opens or
//! transforms in place (`SurfaceHost::transform`, by `SemId`, so what was
//! typed in a field that stays goes on). `surface_page.status` says what
//! the last proposal came to, the rules' error when they did not
//! understand.
//!
//! Both capabilities run through the page's one dispatcher, the todo's,
//! lent for the request (`TodoPage::with_dispatcher`) to `Page`, an
//! `Operable` that projects the todo, the surface and these nodes, runs
//! these two and passes everything else to the todo's own `execute`. A
//! person's click, the Enter key, an agent through the port and a screen
//! reader take the same path, and the receipt is kept where every other
//! receipt of the page is. The author of the IR is whoever asked
//! (`GenerationPolicy::by`), so a surface an agent proposes binds only
//! what agents may ask for.

use comps::prelude::{Button, EventResult, Rect, TextField, Wake, WidgetEvent};
use engine::compositor::{Compositor, SceneNode, TextNodeKey};
use engine::text::{TextMeasurer, TextStyle};
use engine::theme::Theme;
use propose::{Ask, Rules, propose_verified};
use sem::{
    Actor, Execution, Operable, Outcome, Request, RequestId, Role, SemError, SemId, Semantics,
    SnapshotBuilder, Value,
};
use surface::{Surface, SurfaceHost};
use uir::{Capabilities, GenerationPolicy};

use super::super::app::TodoPage;

/// The ids the page's own capabilities project and bind.
pub(super) struct Ids {
    pub region: SemId,
    pub intent: SemId,
    pub propose: SemId,
    pub status: SemId,
}

impl Ids {
    fn new() -> Result<Self, SemError> {
        let region = SemId::name("surface_page")?;
        Ok(Self {
            intent: region.child("intent")?,
            propose: region.child("propose")?,
            status: region.child("status")?,
            region,
        })
    }
}

/// What the last proposal came to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Said {
    /// The surface shows what the rules proposed for this intent.
    Proposed(String),
    /// Why there is no proposal: the rules' error, or the verifier's.
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

/// The intent field, the propose button, and what the last proposal
/// came to.
pub(super) struct Proposing {
    pub ids: Option<Ids>,
    pub intent: TextField,
    button: Button,
    pub said: Option<Said>,
    /// The surface shows what the rules last proposed (not a hand-written
    /// form).
    pub shown: bool,
    inert: bool,
}

/// Where the proposing row sits: the field and the button, the message
/// under them, and the band they take together.
pub(super) struct Row {
    pub band: Rect,
    pub intent: Rect,
    pub propose: Rect,
    pub message: Rect,
}

/// What a request accepted against the page asks of these capabilities.
enum Op {
    Write(String),
    Propose,
}

fn message_style(theme: &Theme) -> TextStyle {
    theme.typography.caption_sm()
}

fn bounds(r: Rect) -> [f32; 4] {
    [r.x, r.y, r.w, r.h]
}

impl Proposing {
    pub fn new() -> Self {
        let ids = Ids::new()
            .inspect_err(|e| log::error!("surface page: the ids do not parse: {e}"))
            .ok();
        let mut intent = TextField::new("what should it show?");
        let mut button = Button::new("propose").disabled(true);
        if let Some(ids) = &ids {
            intent = intent.sem(ids.intent.clone());
            button = button.sem(ids.propose.clone());
        }
        Self {
            ids,
            intent,
            button,
            said: None,
            shown: false,
            inert: false,
        }
    }

    pub fn set_inert(&mut self, inert: bool) {
        self.inert = inert;
    }

    fn blank(&self) -> bool {
        self.intent.text().trim().is_empty()
    }

    /// True when `id` is one of these nodes.
    pub fn holds(&self, id: &SemId) -> bool {
        self.ids
            .as_ref()
            .is_some_and(|ids| [&ids.region, &ids.intent, &ids.propose, &ids.status].contains(&id))
    }

    /// The region, the intent field, the propose button (its `propose`
    /// reads the intent, and runs only when the intent is not blank) and
    /// the status with the last proposal's message.
    pub fn project(&self, b: &mut SnapshotBuilder) {
        let Some(ids) = &self.ids else {
            return;
        };
        b.node(&ids.region, Role::Region, "propose a surface")
            .inert(self.inert);
        self.intent
            .project(b, &ids.intent, "intent", Some(&ids.region));
        let ready = !self.blank();
        let mut propose = b.node(&ids.propose, Role::Button, "propose");
        propose.parent(&ids.region).enabled(ready);
        propose
            .action("propose")
            .enabled(ready)
            .reads(format!("{}:text", ids.intent));
        let message = self.said.as_ref().map_or_else(String::new, Said::text);
        b.node(&ids.status, Role::Status, "proposal")
            .parent(&ids.region)
            .state("message", message);
        if self.intent.is_focused() {
            b.focus(&ids.intent);
        }
    }

    fn op(&self, req: &Request) -> Option<Op> {
        let ids = self.ids.as_ref()?;
        match req.action.as_str() {
            "set_value" if req.target == ids.intent => match &req.param {
                Some(Value::Text(text)) => Some(Op::Write(text.clone())),
                _ => None,
            },
            "propose" if req.target == ids.propose => Some(Op::Propose),
            _ => None,
        }
    }

    /// The row at `(x, y)`, `w` wide: the field and the button on one
    /// line, the message under them (one line at least, so that a message
    /// arriving moves nothing below).
    pub fn row(&self, x: f32, y: f32, w: f32, theme: &Theme) -> Row {
        let gap = theme.spacing.sm;
        let h = TextField::height(theme);
        let (bw, bh) = self.button.preferred_size(theme);
        let field_w = (w - bw - gap).max(0.0);
        let intent = Rect::new(x, y, field_w, h);
        let propose = Rect::new(x + field_w + gap, y + (h - bh) / 2.0, bw, bh);
        let style = message_style(theme);
        let line = TextMeasurer::measure_styled("Ag", &style, None).1;
        let text = self.said.as_ref().map_or_else(String::new, Said::text);
        let wrapped = TextMeasurer::measure_styled(&text, &style, Some(w)).1;
        let message = Rect::new(x, y + h + gap, w, line.max(wrapped));
        let bottom = message.y + message.h + gap;
        Row {
            band: Rect::new(x, y, w, bottom - y),
            intent,
            propose,
            message,
        }
    }

    /// A person's pointer on the row: the field takes focus, the button
    /// is clicked; true in the second field when the button asks for a
    /// proposal.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        row: &Row,
        theme: &Theme,
    ) -> (EventResult, bool) {
        if self.inert {
            return (EventResult::IGNORED, false);
        }
        self.button.disabled = self.blank();
        let field = self.intent.handle_event(event, row.intent, theme);
        let button = self.button.handle_event(event, row.propose);
        (field.merge(button), button.clicked)
    }

    pub fn wake(&mut self, dt: f32) -> Wake {
        self.intent.wake(dt)
    }

    pub fn render(&mut self, c: &mut Compositor, row: &Row, theme: &Theme) {
        let Some(ids) = &self.ids else {
            return;
        };
        c.bind(&ids.region, bounds(row.band));
        self.intent.render(c, row.intent, theme);
        self.button.disabled = self.blank();
        self.button.render(c, row.propose, theme);
        c.bind(&ids.status, bounds(row.message));
        let Some(said) = &self.said else {
            return;
        };
        let color = match said {
            Said::Proposed(_) => theme.glass.text_faint.0,
            Said::Refused(_) => theme.colors.danger.0,
        };
        let style = message_style(theme);
        c.push(SceneNode::Text {
            key: TextNodeKey::from_style(&said.text(), &style, Some(row.message.w)),
            x: row.message.x,
            y: row.message.y,
            color,
        });
    }
}

/// The page as its one dispatcher sees it for a request: the todo, the
/// surface and the proposing row, one projection; the proposing row's
/// two capabilities run here, everything else in the todo's `execute`.
pub(super) struct Page<'a> {
    pub todo: &'a mut TodoPage,
    pub host: &'a mut SurfaceHost,
    /// The surface the page shows, by id.
    pub id: &'a mut Option<SemId>,
    /// Why no surface opened, when none did.
    pub refused: &'a mut Option<String>,
    pub proposing: &'a mut Proposing,
}

impl Page<'_> {
    /// Runs `req` through the todo's dispatcher over the whole page.
    pub fn dispatch(self, req: Request) -> RequestId {
        let Page {
            todo,
            host,
            id,
            refused,
            proposing,
        } = self;
        todo.with_dispatcher(|dispatcher, todo, now| {
            let mut page = Page {
                todo,
                host,
                id,
                refused,
                proposing,
            };
            dispatcher.dispatch(&mut page, req, now)
        })
    }

    /// The rules propose for the intent, as `author` wrote it, over the
    /// todo as it is and the surface shown; verified, the surface opens
    /// or takes the new form in place.
    fn propose(&mut self, author: &Actor) -> Execution {
        let intent = self.proposing.intent.text().trim().to_owned();
        let app = match self.todo.snapshot() {
            Ok(app) => app,
            Err(e) => return Execution::Failed(e.to_string()),
        };
        let caps = Capabilities::from_snapshot(&app);
        let policy = GenerationPolicy::by(author.clone());
        let now = self.id.as_ref().and_then(|id| self.host.get(id));
        let ask = Ask {
            intent: &intent,
            caps: &caps,
            app: &app,
            current: now.map(Surface::verified),
            policy: &policy,
        };
        let proposed = propose_verified(&mut Rules, &ask).map_err(|e| e.to_string());
        let opened = proposed.and_then(|ir| {
            let open = self.host.get(&ir.ir().surface.id).is_some();
            let surface = if open {
                self.host.transform(ir, &app)
            } else {
                self.host.open(ir, &app)
            };
            surface
                .map(|s| (s.id().clone(), s.nodes().count()))
                .map_err(|e| e.to_string())
        });
        match opened {
            Ok((surface, nodes)) => {
                *self.id = Some(surface.clone());
                *self.refused = None;
                self.proposing.said = Some(Said::Proposed(intent));
                self.proposing.shown = true;
                let mut outcome = Outcome::default();
                let nodes = i64::try_from(nodes).unwrap_or(i64::MAX);
                outcome
                    .result
                    .insert("surface".to_owned(), Value::Text(surface.to_string()));
                outcome.result.insert("nodes".to_owned(), Value::Int(nodes));
                Execution::Done(outcome)
            }
            Err(why) => {
                self.proposing.said = Some(Said::Refused(why.clone()));
                Execution::Failed(why)
            }
        }
    }
}

impl Semantics for Page<'_> {
    fn project(&self, b: &mut SnapshotBuilder) {
        self.todo.project(b);
        self.host.project(b);
        self.proposing.project(b);
    }
}

impl Operable for Page<'_> {
    fn execute(&mut self, id: RequestId, req: &Request) -> Execution {
        match self.proposing.op(req) {
            Some(Op::Write(text)) => {
                self.proposing.intent.set_text(&text);
                Execution::Done(Outcome::default())
            }
            Some(Op::Propose) => self.propose(&req.actor),
            None => self.todo.execute(id, req),
        }
    }
}
