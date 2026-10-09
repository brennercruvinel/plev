//! The accessibility band of the Silences page (task-70 phase 4): where
//! the person turns their preferences on and off, for every surface the
//! page shows, the open one and every one that opens after (the host
//! makes them over each, `SurfaceHost::prefer`). The person opens it by
//! asking for it through the row, "accessibility" or "acessibilidade",
//! the same `propose` a change of the surface takes (an agent is refused
//! there); under the row, beside the composite's offer, the band has
//! "larger text", "more contrast", "denser", "sparser" and "done".
//!
//! The band and its buttons (`silences_surface.access`) exist only while
//! it is open, so nothing the page projected before moves, and every one
//! is kept for a person (`PERSON_ONLY`): an agent is refused
//! `not_authorized` by the dispatcher. No node holds a state and the
//! labels never change: what is on is not in the projection, for an agent
//! or anyone reading it. The person sees it on the band (a preference on
//! reads dark, as a decision taken on a cut does) and on the surface; a
//! person with a screen reader hears it, since task-75 phase 2a: the
//! reader's snapshot alone (`ShowcaseView::reader_semantics`) gives each
//! preference button its `on` (`Access::reader_states`), which the tree
//! says as a toggle button's state. The port and a model's prompt never
//! get it.

use comps::prelude::{Button, ButtonSize, EventResult, Rect, WidgetEvent};
use engine::compositor::{Compositor, SceneNode, TextNodeKey};
use engine::text::{TextMeasurer, TextStyle};
use engine::theme::Theme;
use sem::{Policy, Request, RequestId, Role, SemId, SnapshotBuilder};
use uir::{Preference, Preferences};

use super::super::SilencesPage;
use crate::view::apps::app::TodoPage;
use crate::view::dispatch::PERSON_ONLY;

/// The words that open the band, alone in the intent, any case.
pub const WORDS: [&str; 2] = ["accessibility", "acessibilidade"];

/// What the band says.
const SAYS: &str = "accessibility, for you, over every surface here";

/// True when `intent` asks for the band: one of `WORDS`, alone.
pub fn asked(intent: &str) -> bool {
    let word = intent
        .trim()
        .trim_end_matches(['.', '!', '?'])
        .to_lowercase();
    WORDS.contains(&word.as_str())
}

/// A button of the band pressed: a preference flipped, or the band closed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pressed {
    Flip(Preference),
    Done,
}

/// The buttons, in the band's order.
const BUTTONS: [(Pressed, &str, &str); 5] = [
    (
        Pressed::Flip(Preference::LargerText),
        "larger_text",
        "larger text",
    ),
    (
        Pressed::Flip(Preference::MoreContrast),
        "more_contrast",
        "more contrast",
    ),
    (Pressed::Flip(Preference::Denser), "denser", "denser"),
    (Pressed::Flip(Preference::Sparser), "sparser", "sparser"),
    (Pressed::Done, "done", "done"),
];

/// Where the band goes: the whole of it, the sentence, each button.
pub struct Band {
    pub band: Rect,
    pub text: Rect,
    pub buttons: [Rect; 5],
}

/// The band: open or not, its ids, its buttons.
pub struct Access {
    open: bool,
    /// The band's id, then each button's, in `BUTTONS` order.
    ids: Option<(SemId, [SemId; 5])>,
    buttons: [Button; 5],
}

fn style(theme: &Theme) -> TextStyle {
    theme.typography.caption_sm()
}

impl Access {
    pub fn new(region: Option<&SemId>) -> Access {
        let ids: Option<(SemId, [SemId; 5])> = region.and_then(|region| {
            let band = region.child("access").ok()?;
            let mut each = Vec::with_capacity(BUTTONS.len());
            for (_, name, _) in BUTTONS {
                each.push(band.child(name).ok()?);
            }
            Some((band, each.try_into().ok()?))
        });
        let buttons = std::array::from_fn(|at| {
            let button = Button::new(BUTTONS[at].2).size(ButtonSize::Sm);
            match &ids {
                Some((_, each)) => button.sem(each[at].clone()),
                None => button,
            }
        });
        Access {
            open: false,
            ids,
            buttons,
        }
    }

    pub fn open(&mut self) {
        self.open = true;
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    /// True for the band's own nodes.
    pub fn holds(&self, id: &SemId) -> bool {
        self.ids
            .as_ref()
            .is_some_and(|(band, each)| band == id || each.contains(id))
    }

    /// The button `req` presses, while the band is open.
    pub fn pressed(&self, req: &Request) -> Option<Pressed> {
        let (_, each) = self.ids.as_ref().filter(|_| self.open)?;
        if req.action.as_str() != "activate" {
            return None;
        }
        let at = each.iter().position(|id| *id == req.target)?;
        Some(BUTTONS[at].0)
    }

    /// The id of the button that does `pressed`.
    pub fn id(&self, pressed: Pressed) -> Option<&SemId> {
        let (_, each) = self.ids.as_ref()?;
        let at = BUTTONS.iter().position(|(p, _, _)| *p == pressed)?;
        each.get(at)
    }

    /// The band at `(x, y)`, `w` wide, while it is open: the sentence,
    /// then the buttons at its end.
    pub fn band(&self, x: f32, y: f32, w: f32, theme: &Theme) -> Option<Band> {
        if !self.open {
            return None;
        }
        let gap = theme.spacing.sm;
        let sizes = self.buttons.each_ref().map(|b| b.preferred_size(theme));
        let buttons_w: f32 = sizes.iter().map(|s| s.0 + gap).sum();
        let text_w = (w - buttons_w).max(0.0);
        let text_h = TextMeasurer::measure_styled(SAYS, &style(theme), Some(text_w)).1;
        let h = sizes.iter().fold(text_h, |h, s| h.max(s.1));
        let mut right = x + w;
        let mut rects = [Rect::new(0.0, 0.0, 0.0, 0.0); 5];
        for (at, (bw, bh)) in sizes.iter().enumerate().rev() {
            right -= bw;
            rects[at] = Rect::new(right, y + (h - bh) / 2.0, *bw, *bh);
            right -= gap;
        }
        Some(Band {
            band: Rect::new(x, y, w, h + gap),
            text: Rect::new(x, y + (h - text_h) / 2.0, text_w, text_h),
            buttons: rects,
        })
    }

    /// The band and its buttons, while it is open, under `region`: each
    /// button kept for a person, none with a state.
    pub fn project(&self, b: &mut SnapshotBuilder, region: &SemId) {
        let Some((band, each)) = self.ids.as_ref().filter(|_| self.open) else {
            return;
        };
        b.node(band, Role::Group, "accessibility").parent(region);
        let kept = Policy {
            actors: PERSON_ONLY,
            ..Policy::default()
        };
        for (id, (_, _, label)) in each.iter().zip(BUTTONS) {
            b.node(id, Role::Button, label)
                .parent(band)
                .action("activate")
                .policy(kept);
        }
    }

    /// Whether the band is open.
    pub fn is_open(&self) -> bool {
        self.open
    }

    /// What only a screen reader is told, into `b`, which holds the
    /// page's projection with the band: each preference button of the
    /// open band says whether its preference is `on` (task-75 phase 2a).
    /// Never in the page's projection: a preference is the person's, and
    /// the port and a model's prompt read that one.
    pub fn reader_states(&self, b: &mut SnapshotBuilder, on: Preferences) {
        let Some((_, each)) = self.ids.as_ref().filter(|_| self.open) else {
            return;
        };
        for (id, (pressed, _, _)) in each.iter().zip(BUTTONS) {
            let Pressed::Flip(preference) = pressed else {
                continue;
            };
            if let Some(mut node) = b.reopen(id) {
                node.state("on", on.has(preference));
            }
        }
    }

    /// A person's pointer on the band: the button clicked, if any.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        band: &Band,
    ) -> (EventResult, Option<Pressed>) {
        let mut r = EventResult::IGNORED;
        let mut pressed = None;
        for ((button, rect), (p, _, _)) in self.buttons.iter_mut().zip(band.buttons).zip(BUTTONS) {
            let got = button.handle_event(event, rect);
            if got.clicked {
                pressed = Some(p);
            }
            r = r.merge(got);
        }
        (r, pressed)
    }

    /// The band, each preference on drawn dark.
    pub fn render(&mut self, c: &mut Compositor, band: &Band, theme: &Theme, on: Preferences) {
        let Some((id, _)) = &self.ids else {
            return;
        };
        c.bind(id, [band.band.x, band.band.y, band.band.w, band.band.h]);
        let style = style(theme);
        c.push(SceneNode::Text {
            key: TextNodeKey::from_style(SAYS, &style, Some(band.text.w)),
            x: band.text.x,
            y: band.text.y,
            color: theme.colors.text.0,
        });
        for ((button, rect), (p, _, _)) in self.buttons.iter_mut().zip(band.buttons).zip(BUTTONS) {
            button.variant = SilencesPage::variant(matches!(p, Pressed::Flip(p) if on.has(p)));
            button.render(c, rect, theme);
        }
    }
}

impl SilencesPage {
    /// A person presses a button of the accessibility band.
    pub(in crate::view::apps::silences) fn press(
        &mut self,
        todo: &mut TodoPage,
        pressed: Pressed,
    ) -> Option<RequestId> {
        let target = self.generated.access.id(pressed)?.clone();
        self.ask_page(todo, &target, "activate")
    }
}
