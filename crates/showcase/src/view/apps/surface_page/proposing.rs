//! The Surface page's own capabilities (task-69 phases 3a and 3b-ii): an
//! intent in text, who proposes for it, a button that proposes, a button
//! that cancels, and a status that says what came of it.
//!
//! `surface_page.intent` is a text input (`set_value`);
//! `surface_page.proposer` a select between `rules` and `claude`
//! (`claude` offered, and chosen, only where the program is:
//! `asking::Claude::find`); `surface_page.propose` a button whose
//! `propose` reads the intent and the proposer; `surface_page.cancel` a
//! button enabled only while the Claude is asked; `surface_page.status`
//! the message of the last proposal, and as state whether the page is
//! busy, the attempt, the seconds and the cost. An agent reads and acts
//! on the same nodes.
//!
//! The rules (`propose::Rules`) propose inside `execute`, in a fraction
//! of a millisecond, and the request ends at once. The Claude takes
//! seconds to a minute and a half per attempt, up to three: `propose`
//! answers `Execution::Started`, a worker runs the proposal (`asking`),
//! the page keeps asking for frames while it waits, and when the answer
//! arrives `Page::settle` opens or transforms the surface
//! (`SurfaceHost::transform`, by `SemId`, so what was typed in a field
//! that stays goes on) and ends the request with `Dispatcher::complete`
//! or `fail`, through the same lending of the todo's dispatcher
//! (`TodoPage::with_dispatcher`), at the page clock's now. One proposal
//! at a time: while one is asked the propose button and its action are
//! disabled. A cancel raises the worker's signal (the child dies by its
//! pid) and fails the request as cancelled by whoever asked.
//!
//! Every capability runs through the page's one dispatcher, the todo's,
//! lent for the request to `page::Page`, the `Operable` over the whole
//! page (`page.rs`): this file is the row itself, its widgets, where they
//! sit, what they project and what a person's pointer does on them. The
//! row is shared with the Silences page (task-69 phase 4b), which puts it
//! over its fixed screen under its own region (`silences_surface`): the
//! region's name is the row's one parameter, the rest is the same.

use std::time::Duration;

use comps::prelude::{Button, EventResult, Rect, Select, TextField, Wake, WidgetEvent};
use engine::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use engine::text::{TextMeasurer, TextStyle};
use engine::theme::{IconSize, Theme};
use propose::{Llm, Model};
use sem::{Request, Role, SemError, SemId, SnapshotBuilder, Value};

use super::asking::{Claude, POLL, Pending};

/// The ids the page's own capabilities project and bind.
pub(crate) struct Ids {
    pub region: SemId,
    pub intent: SemId,
    pub proposer: SemId,
    pub propose: SemId,
    pub cancel: SemId,
    pub status: SemId,
}

impl Ids {
    /// The row's ids under `region` (`surface_page`, `silences_surface`).
    fn new(region: &str) -> Result<Self, SemError> {
        let region = SemId::name(region)?;
        Ok(Self {
            intent: region.child("intent")?,
            proposer: region.child("proposer")?,
            propose: region.child("propose")?,
            cancel: region.child("cancel")?,
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

/// The two proposers as the select names them: the words a `select`
/// takes.
const RULES: &str = "rules";
const CLAUDE: &str = "claude";

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

fn usd(cost: Option<f64>) -> String {
    cost.map_or_else(|| "cost unknown".to_owned(), |c| format!("{c:.3} USD"))
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
                let noun = if *attempts == 1 {
                    "attempt"
                } else {
                    "attempts"
                };
                format!(
                    "proposed by the Claude for \"{intent}\" in {:.1} s, {attempts} {noun}, {}",
                    took.as_secs_f64(),
                    usd(*cost_usd)
                )
            }
            Said::Refused(why) => why.clone(),
        }
    }
}

/// The intent field, the proposer, the two buttons, the Claude when it
/// is there, the proposal it is answering, and what the last one came to.
pub(crate) struct Proposing {
    pub ids: Option<Ids>,
    pub intent: TextField,
    pub proposer: Select,
    button: Button,
    cancel: Button,
    pub claude: Claude,
    pub pending: Option<Pending>,
    pub said: Option<Said>,
    /// The surface shows what was last proposed (not a hand-written
    /// form).
    pub shown: bool,
    inert: bool,
}

/// Where the proposing row sits: the field, the proposer and the propose
/// button (the two under the field when the width does not hold the
/// three), the message with the cancel button at its end, and the band
/// they take together.
pub(crate) struct Row {
    pub band: Rect,
    pub intent: Rect,
    pub proposer: Rect,
    pub propose: Rect,
    pub cancel: Rect,
    pub message: Rect,
}

/// What a person's pointer on the row asked.
#[derive(Default)]
pub(crate) struct Asked {
    pub propose: bool,
    pub cancel: bool,
    /// The open options panel took the event: nothing under it sees it.
    pub taken: bool,
}

/// What a request accepted against the page asks of these capabilities.
pub(crate) enum Op {
    Write(String),
    Propose,
    Choose(String),
    ChooseAt(usize),
    Open,
    Close,
    Cancel,
}

fn message_style(theme: &Theme) -> TextStyle {
    theme.typography.caption_sm()
}

fn bounds(r: Rect) -> [f32; 4] {
    [r.x, r.y, r.w, r.h]
}

pub(crate) fn int(n: usize) -> i64 {
    i64::try_from(n).unwrap_or(i64::MAX)
}

fn secs(d: Duration) -> i64 {
    i64::try_from(d.as_secs()).unwrap_or(i64::MAX)
}

/// How many times the Claude is asked for one proposal.
fn attempts() -> usize {
    Llm::<&mut dyn Model>::ATTEMPTS
}

impl Proposing {
    /// The row under `region`, over the Claude found on this machine
    /// (none in the crate's own tests, none off the desktop).
    pub fn new(region: &str) -> Self {
        Self::with(region, Claude::find())
    }

    /// The row under `region` over `claude`: the select offers the
    /// Claude, and chooses it, when there is a program; the rules alone
    /// otherwise.
    pub fn with(region: &str, claude: Claude) -> Self {
        let ids = Ids::new(region)
            .inspect_err(|e| log::error!("surface page: the ids do not parse: {e}"))
            .ok();
        let mut intent = TextField::new("what should it show?");
        let mut proposer = Self::select(&claude);
        let mut button = Button::new("propose").disabled(true);
        let mut cancel = Button::new("cancel").disabled(true);
        if let Some(ids) = &ids {
            intent = intent.sem(ids.intent.clone());
            proposer = proposer.sem(ids.proposer.clone());
            button = button.sem(ids.propose.clone());
            cancel = cancel.sem(ids.cancel.clone());
        }
        Self {
            ids,
            intent,
            proposer,
            button,
            cancel,
            claude,
            pending: None,
            said: None,
            shown: false,
            inert: false,
        }
    }

    fn select(claude: &Claude) -> Select {
        if claude.is_some() {
            Select::new([RULES, CLAUDE], 1)
        } else {
            Select::new([RULES], 0)
        }
    }

    /// The Claude the page asks from now on (a test's fake, the real one
    /// in the proof): the select offers it and chooses it.
    #[cfg(test)]
    pub fn set_claude(&mut self, claude: Claude) {
        let mut select = Self::select(&claude);
        if let Some(ids) = &self.ids {
            select = select.sem(ids.proposer.clone());
        }
        self.proposer = select;
        self.claude = claude;
    }

    pub fn set_inert(&mut self, inert: bool) {
        self.inert = inert;
    }

    fn blank(&self) -> bool {
        self.intent.text().trim().is_empty()
    }

    /// True while the Claude is asked.
    pub fn waiting(&self) -> bool {
        self.pending.is_some()
    }

    pub fn chosen(&self) -> Proposer {
        match self.proposer.selected_label() {
            Some(CLAUDE) => Proposer::Claude,
            _ => Proposer::Rules,
        }
    }

    /// True when `id` is one of these nodes (an option row of the
    /// proposer included).
    pub fn holds(&self, id: &SemId) -> bool {
        self.ids.as_ref().is_some_and(|ids| {
            [
                &ids.region,
                &ids.intent,
                &ids.proposer,
                &ids.propose,
                &ids.cancel,
                &ids.status,
            ]
            .contains(&id)
                || ids.option_of(id).is_some()
        })
    }

    /// The status line: the wait, or what the last proposal came to.
    pub fn message(&self) -> String {
        match &self.pending {
            Some(p) => format!(
                "asking the Claude for \"{}\": attempt {} of {}, {} s",
                p.intent,
                p.attempt().max(1),
                attempts(),
                p.elapsed().as_secs()
            ),
            None => self.said.as_ref().map_or_else(String::new, Said::text),
        }
    }

    /// The buttons follow the state: propose only with an intent and
    /// nothing asked, cancel only while asking.
    fn refresh(&mut self) {
        self.button.disabled = self.blank() || self.waiting();
        self.cancel.disabled = !self.waiting();
    }

    /// The region; the intent field; the proposer (a select, `select`
    /// with the name, `open`, `close`); the propose button (its `propose`
    /// reads the intent and the proposer, and runs only with an intent
    /// and nothing asked); the cancel button (enabled only while asking);
    /// and the status with the message, `busy`, and while asking or after
    /// the Claude answered the `attempt`, the `seconds` and the
    /// `cost_usd` when known.
    pub fn project(&self, b: &mut SnapshotBuilder) {
        let Some(ids) = &self.ids else {
            return;
        };
        b.node(&ids.region, Role::Region, "propose a surface")
            .inert(self.inert);
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
                    .state("attempt", int(p.attempt()))
                    .state("seconds", secs(p.elapsed()));
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
                    .state("attempt", int(*attempts))
                    .state("seconds", secs(*took));
                if let Some(cost) = cost_usd {
                    status.state("cost_usd", *cost);
                }
            }
            _ => {}
        }
        if self.intent.is_focused() {
            b.focus(&ids.intent);
        }
    }

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
            "open" if *target == ids.proposer => Some(Op::Open),
            "close" if *target == ids.proposer => Some(Op::Close),
            "activate" if *target == ids.cancel => Some(Op::Cancel),
            _ => None,
        }
    }

    /// The width the proposer takes: its widest option, the paddings and
    /// the chevron, as the select draws them.
    fn proposer_w(&self, theme: &Theme) -> f32 {
        let style = theme.typography.base_2m();
        let text = self
            .proposer
            .options
            .iter()
            .map(|o| TextMeasurer::measure_styled(o, &style, None).0)
            .fold(0.0, f32::max);
        let chevron = theme.control.icon(IconSize::Sm);
        (text + theme.spacing.lg * 2.0 + theme.spacing.xs + chevron).ceil()
    }

    /// The row at `(x, y)`, `w` wide: the field, the proposer and the
    /// button on one line while the field keeps a card's minimum width
    /// (an intent is a sentence), the two under the field otherwise;
    /// then the message (one line at least, so that a message arriving
    /// moves nothing below) with the cancel button at its end.
    pub fn row(&self, x: f32, y: f32, w: f32, theme: &Theme) -> Row {
        let gap = theme.spacing.sm;
        let h = TextField::height(theme);
        let (bw, bh) = self.button.preferred_size(theme);
        let (cw, ch) = self.cancel.preferred_size(theme);
        let sw = self.proposer_w(theme).min(w);
        let controls = sw + gap + bw + gap;
        let (intent, left, line) = if w - controls >= theme.size.card_min_w {
            let field_w = w - controls;
            (Rect::new(x, y, field_w, h), x + field_w + gap, y)
        } else {
            (Rect::new(x, y, w, h), x, y + h + gap)
        };
        let proposer = Rect::new(left, line, sw, h);
        let propose = Rect::new(left + sw + gap, line + (h - bh) / 2.0, bw, bh);
        let below = line + h + gap;
        let style = message_style(theme);
        let one = TextMeasurer::measure_styled("Ag", &style, None).1;
        let text_w = (w - cw - gap).max(0.0);
        let wrapped = TextMeasurer::measure_styled(&self.message(), &style, Some(text_w)).1;
        let text_h = one.max(wrapped);
        let band_h = text_h.max(ch);
        let message = Rect::new(x, below + (band_h - text_h) / 2.0, text_w, text_h);
        let cancel = Rect::new(x + w - cw, below + (band_h - ch) / 2.0, cw, ch);
        let bottom = below + band_h + gap;
        Row {
            band: Rect::new(x, y, w, bottom - y),
            intent,
            proposer,
            propose,
            cancel,
            message,
        }
    }

    /// A person's pointer on the row: the field takes focus, the
    /// proposer opens and chooses, the buttons are clicked. While the
    /// proposer's panel is open it sees the event alone.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        row: &Row,
        theme: &Theme,
    ) -> (EventResult, Asked) {
        if self.inert {
            return (EventResult::IGNORED, Asked::default());
        }
        self.refresh();
        if self.proposer.is_open() {
            let panel = self.proposer.handle_event(event, row.proposer, theme);
            let asked = Asked {
                taken: true,
                ..Asked::default()
            };
            return (panel, asked);
        }
        let field = self.intent.handle_event(event, row.intent, theme);
        let select = self.proposer.handle_event(event, row.proposer, theme);
        let button = self.button.handle_event(event, row.propose);
        let cancel = self.cancel.handle_event(event, row.cancel);
        let asked = Asked {
            propose: button.clicked,
            cancel: cancel.clicked,
            taken: false,
        };
        (field.merge(select).merge(button).merge(cancel), asked)
    }

    /// Escape closes the proposer's panel; true when it was open.
    pub fn escape(&mut self) -> bool {
        let open = self.proposer.is_open();
        if open {
            self.proposer.close();
        }
        open
    }

    /// The caret, and while the Claude is asked a look at the channel
    /// every `POLL`.
    pub fn wake(&mut self, dt: f32) -> Wake {
        let field = self.intent.wake(dt);
        if self.waiting() {
            field.merge(Wake::after(POLL))
        } else {
            field
        }
    }

    pub fn render(&mut self, c: &mut Compositor, row: &Row, theme: &Theme) {
        self.refresh();
        let Some(ids) = &self.ids else {
            return;
        };
        c.bind(&ids.region, bounds(row.band));
        self.intent.render(c, row.intent, theme);
        self.proposer.render(c, row.proposer, theme);
        self.button.render(c, row.propose, theme);
        self.cancel.render(c, row.cancel, theme);
        c.bind(&ids.status, bounds(row.message));
        let text = self.message();
        if text.is_empty() {
            return;
        }
        let color = match (&self.pending, &self.said) {
            (None, Some(Said::Refused(_))) => theme.colors.danger.0,
            _ => theme.glass.text_faint.0,
        };
        let style = message_style(theme);
        c.push(SceneNode::Text {
            key: TextNodeKey::from_style(&text, &style, Some(row.message.w)),
            x: row.message.x,
            y: row.message.y,
            color,
        });
    }

    /// The proposer's options panel, when open: drawn after everything
    /// else on the page, so it lies over the surface.
    pub fn render_open(&self, c: &mut Compositor, row: &Row, theme: &Theme) {
        if self.proposer.is_open() {
            self.proposer
                .render_dropdown(c, LayerId::DEFAULT, row.proposer, theme);
        }
    }
}
