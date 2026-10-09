//! The composite offer of the Silences page (task-70 phase 3): the
//! page's `propose::compose::Composer`, heard on the gallery's one
//! dispatcher, and the band under the proposing row where the person
//! sees a sequence they repeated and answers it: "make it one action" or
//! "not now". The band and its nodes (`silences_surface.composite`, with
//! `install` and `dismiss`) exist only while an offer stands, so nothing
//! the page projected before moves. Both answers are kept for a person
//! (`PERSON_ONLY`): an agent is refused `not_authorized` by the
//! dispatcher, and nothing installs unless a person asks.
//!
//! A composite installed is an action of the app on every cut; asked, its
//! `execute` answers `Started` and keeps a `propose::compose::Chain`,
//! which runs the steps through the same lending of the dispatcher right
//! after (`composed`), and goes on from the page's wake when a step
//! waited for a frame.

use engine::widgets::prelude::{Button, ButtonSize, EventResult, Rect, WidgetEvent};
use engine::drawing::compositor::{Compositor, SceneNode, TextNodeKey};
use engine::drawing::text::{TextMeasurer, TextStyle};
use engine::styling::theme::Theme;
use offers::compose::{Chain, Composer, Composite, Found, Shape, Target};
use accord::{
    Dispatcher, Execution, Millis, Policy, Request, RequestId, Role, SemId, Snapshot,
    SnapshotBuilder, Value,
};
use plevir::IrNode;

use super::super::SilencesPage;
use crate::views::apps::todo::TodoPage;
use crate::views::dispatcher::PERSON_ONLY;

/// How many times in a row a sequence is repeated before the page offers
/// it: the seed has two cuts, and an offer is only an offer.
pub const REPEATS: usize = 2;

/// The band's ids: the band, and its two answers.
struct Ids {
    band: SemId,
    install: SemId,
    dismiss: SemId,
}

/// The person's answer to the offer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Answer {
    Install,
    Dismiss,
}

/// Where the band goes: the whole of it, the sentence, the two buttons.
pub struct Band {
    pub band: Rect,
    pub text: Rect,
    pub install: Rect,
    pub dismiss: Rect,
}

/// The composer of the page and the band that offers what it found.
pub struct Offer {
    pub composer: Composer,
    /// The composites running, each a chain of the page's requests.
    pub chains: Vec<Chain>,
    /// The composer hears the gallery's dispatcher (the sink is added
    /// before the page's first request).
    listening: bool,
    ids: Option<Ids>,
    install: Button,
    dismiss: Button,
    /// The offer in words, the label its composite would have.
    label: Option<String>,
}

fn style(theme: &Theme) -> TextStyle {
    theme.typography.caption_sm()
}

fn bounds(r: Rect) -> [f32; 4] {
    [r.x, r.y, r.w, r.h]
}

impl Offer {
    pub fn new(region: Option<&SemId>) -> Offer {
        let ids = region.and_then(|region| {
            let band = region.child("composite").ok()?;
            Some(Ids {
                install: band.child("install").ok()?,
                dismiss: band.child("dismiss").ok()?,
                band,
            })
        });
        let mut install = Button::new("make it one action").size(ButtonSize::Sm);
        let mut dismiss = Button::new("not now").size(ButtonSize::Sm);
        if let Some(ids) = &ids {
            install = install.sem(ids.install.clone());
            dismiss = dismiss.sem(ids.dismiss.clone());
        }
        Offer {
            composer: Composer::new(REPEATS),
            chains: Vec::new(),
            listening: false,
            ids,
            install,
            dismiss,
            label: None,
        }
    }

    /// The composer hears every stage of the gallery's dispatcher from
    /// now on; once.
    pub fn listen(&mut self, todo: &mut TodoPage) {
        if !self.listening {
            todo.add_sink(Box::new(self.composer.listener()));
            self.listening = true;
        }
    }

    /// Reads the composer again: the offer standing, among the sequences
    /// `admit` takes (the page's own nodes), named over `app`.
    pub fn refresh(&mut self, app: Option<&Snapshot>, admit: impl Fn(&Shape) -> bool) {
        let label = match (self.composer.offer(admit), app) {
            (Some(found), Some(app)) => Composite::new(found.shape.clone(), app, &[])
                .ok()
                .map(|c| c.label),
            (Some(found), None) => Some(words(found)),
            (None, _) => None,
        };
        self.label = label;
    }

    /// True for the band's own nodes.
    pub fn holds(&self, id: &SemId) -> bool {
        self.ids
            .as_ref()
            .is_some_and(|ids| *id == ids.band || *id == ids.install || *id == ids.dismiss)
    }

    /// The answer `req` gives, when it is one.
    pub fn answer(&self, req: &Request) -> Option<Answer> {
        let ids = self.ids.as_ref()?;
        match req.action.as_str() {
            "install" if req.target == ids.install => Some(Answer::Install),
            "dismiss" if req.target == ids.dismiss => Some(Answer::Dismiss),
            _ => None,
        }
    }

    /// The id of an answer's button.
    pub fn id(&self, answer: Answer) -> Option<&SemId> {
        let ids = self.ids.as_ref()?;
        Some(match answer {
            Answer::Install => &ids.install,
            Answer::Dismiss => &ids.dismiss,
        })
    }

    /// What the band says.
    fn text(&self) -> Option<String> {
        let found = self.composer.offered()?;
        let label = self.label.clone().unwrap_or_else(|| words(found));
        Some(format!(
            "you did \"{label}\"{}, {} times in a row",
            on(&found.shape),
            found.repeats
        ))
    }

    /// The band at `(x, y)`, `w` wide, while an offer stands: the
    /// sentence, then the two buttons at its end.
    pub fn band(&self, x: f32, y: f32, w: f32, theme: &Theme) -> Option<Band> {
        let text = self.text()?;
        let gap = theme.spacing.sm;
        let (iw, ih) = self.install.preferred_size(theme);
        let (dw, dh) = self.dismiss.preferred_size(theme);
        let text_w = (w - iw - dw - gap * 2.0).max(0.0);
        let text_h = TextMeasurer::measure_styled(&text, &style(theme), Some(text_w)).1;
        let h = text_h.max(ih).max(dh);
        let dismiss = Rect::new(x + w - dw, y + (h - dh) / 2.0, dw, dh);
        let install = Rect::new(dismiss.x - gap - iw, y + (h - ih) / 2.0, iw, ih);
        Some(Band {
            band: Rect::new(x, y, w, h + gap),
            text: Rect::new(x, y + (h - text_h) / 2.0, text_w, text_h),
            install,
            dismiss,
        })
    }

    /// The band's nodes, while an offer stands, under `region`.
    pub fn project(&self, b: &mut SnapshotBuilder, region: &SemId) {
        let (Some(ids), Some(found), Some(text)) =
            (&self.ids, self.composer.offered(), self.text())
        else {
            return;
        };
        let label = self.label.clone().unwrap_or_else(|| words(found));
        let repeats = i64::try_from(found.repeats).unwrap_or(i64::MAX);
        b.node(&ids.band, Role::Group, text.as_str())
            .parent(region)
            .state("sequence", label.as_str())
            .state("repeats", Value::Int(repeats));
        let kept = Policy {
            actors: PERSON_ONLY,
            ..Policy::default()
        };
        b.node(&ids.install, Role::Button, "make it one action")
            .parent(&ids.band)
            .action("install")
            .policy(kept);
        b.node(&ids.dismiss, Role::Button, "not now")
            .parent(&ids.band)
            .action("dismiss")
            .policy(kept);
    }

    /// A person's pointer on the band: the answer clicked, if any.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        band: &Band,
    ) -> (EventResult, Option<Answer>) {
        let install = self.install.handle_event(event, band.install);
        let dismiss = self.dismiss.handle_event(event, band.dismiss);
        let answer = if install.clicked {
            Some(Answer::Install)
        } else if dismiss.clicked {
            Some(Answer::Dismiss)
        } else {
            None
        };
        (install.merge(dismiss), answer)
    }

    pub fn render(&mut self, c: &mut Compositor, band: &Band, theme: &Theme) {
        let (Some(ids), Some(text)) = (&self.ids, self.text()) else {
            return;
        };
        c.bind(&ids.band, bounds(band.band));
        let style = style(theme);
        c.push(SceneNode::Text {
            key: TextNodeKey::from_style(&text, &style, Some(band.text.w)),
            x: band.text.x,
            y: band.text.y,
            color: theme.colors.text.0,
        });
        self.install.render(c, band.install, theme);
        self.dismiss.render(c, band.dismiss, theme);
    }
}

/// Where a composite's button goes in a surface: the container that holds
/// the control of one of its steps (the first step that has one), beside
/// it; inside the item's template when the step is an item's.
pub fn beside(root: &IrNode, composite: &Composite) -> Option<SemId> {
    composite.shape.steps.iter().find_map(|step| {
        let target = step.target.template();
        let sends = |n: &IrNode| {
            n.bind
                .actions
                .values()
                .any(|c| c.action == step.action && c.target == target)
        };
        holder(root, &sends)
    })
}

/// The container under `node` (itself included) with a child that
/// `sends`; never a repeat's holder (its item is the template).
fn holder(node: &IrNode, sends: &dyn Fn(&IrNode) -> bool) -> Option<SemId> {
    if node.repeat.is_none() && node.children.iter().any(sends) {
        return Some(node.sem.clone());
    }
    let item = node.repeat.as_ref().map(|r| &r.item);
    node.children
        .iter()
        .chain(item)
        .find_map(|child| holder(child, sends))
}

/// A sequence in words without the app: its actions, ", then ".
fn words(found: &Found) -> String {
    let steps: Vec<String> = found
        .shape
        .steps
        .iter()
        .map(|s| s.action.as_str().replace('_', " "))
        .collect();
    steps.join(", then ")
}

/// " on each cut" for a sequence over the items of `silences.cut`.
fn on(shape: &Shape) -> String {
    let each = shape.steps.iter().find_map(|s| match &s.target {
        Target::Item(base) => base.as_str().rsplit('.').next().map(str::to_owned),
        Target::Node(_) => None,
    });
    each.map_or_else(String::new, |each| format!(" on each {each}"))
}

impl SilencesPage {
    /// A person answers the composite offered: makes it one action, or
    /// not now.
    pub(in crate::views::apps::silences) fn answer(
        &mut self,
        todo: &mut TodoPage,
        answer: Answer,
    ) -> Option<RequestId> {
        let target = self.generated.offer.id(answer)?.clone();
        let action = match answer {
            Answer::Install => "install",
            Answer::Dismiss => "dismiss",
        };
        self.ask_page(todo, &target, action)
    }

    /// The composite offer read again from the page's composer: only a
    /// sequence of the page's own capabilities (its nodes, and the cuts
    /// as the items of their collection) is offered.
    pub(in crate::views::apps::silences) fn refresh_offer(&mut self) {
        let app = self.snapshot().ok();
        let ids = &self.ids;
        self.generated.offer.refresh(app.as_ref(), |shape| {
            ids.as_ref().is_some_and(|ids| {
                shape.steps.iter().all(|s| match &s.target {
                    offers::compose::Target::Node(id) => ids.holds(id),
                    offers::compose::Target::Item(base) => base == ids.collection(),
                })
            })
        });
    }

    /// A request through the gallery's dispatcher, over this page: the
    /// composer hears it (the sink comes before the page's first
    /// request), the composites it started run their steps in the same
    /// lending, and the offer is read again after.
    pub(in crate::views::apps::silences) fn composed<R>(
        &mut self,
        todo: &mut TodoPage,
        ask: impl FnOnce(&mut SilencesPage, &mut Dispatcher, Millis) -> R,
    ) -> R {
        self.generated.offer.listen(todo);
        let out = todo.with_dispatcher(|d, _, now| {
            let out = ask(self, d, now);
            self.run_chains(d, now);
            out
        });
        self.refresh_offer();
        out
    }

    /// `execute` of a composite installed: its chain is kept, to run
    /// right after the dispatch, and the request stays open until it
    /// ends. `None` for any other request.
    pub(in crate::views::apps::silences) fn start_composite(
        &mut self,
        id: RequestId,
        req: &Request,
    ) -> Option<Execution> {
        let offer = &mut self.generated.offer;
        let chain = offer.composer.start(id, req)?;
        offer.chains.push(chain);
        Some(Execution::Started)
    }

    /// The composites running go on over the page until each ends or
    /// waits for a step that started (a frame to decode). True when one
    /// was running.
    pub(in crate::views::apps::silences) fn run_chains(
        &mut self,
        d: &mut Dispatcher,
        now: Millis,
    ) -> bool {
        let mut chains = std::mem::take(&mut self.generated.offer.chains);
        if chains.is_empty() {
            return false;
        }
        chains.retain_mut(|chain| !chain.advance(d, self, now));
        chains.append(&mut self.generated.offer.chains);
        self.generated.offer.chains = chains;
        true
    }
}
