//! Dialog page, the hoff core kit's ModalPage: a plain and a destructive
//! question that block the page until they are answered. The widget is
//! `engine::widgets::feedback::Modal` on `theme.overlay.modal` (the kit's
//! `components/DeleteFile`: the text centered, the two answers sharing a
//! footer band); the view owns the modal on screen (its overlay stack,
//! the scrim, the focus trap), and the page only says which to open.
//!
//! "Live" opens them: the focus is the cancel (the answer that changes
//! nothing), Escape or a press on the scrim cancels. "Variations" shows
//! both as the ModalPage does, in cards.
//!
//! Projected (task-75 phase 2b): `dialog_page.open` and
//! `dialog_page.delete`, whose `activate` asks the view for the modal a
//! click opens (the modal itself is the view's, `dispatch::overlay`),
//! under the live group `dialog_page.group[0]`; each shown dialog,
//! `dialog_page.group[1].card[<c>]`, a group named by its title and
//! described by its body, its two answers as labels.

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::drawing::compositor::{Compositor, LayerId};
use engine::styling::theme::{Intent, Theme};
use engine::widgets::prelude::{Button, ButtonVariant, EventResult, Modal, Rect, WidgetEvent};

use super::showcase::{self, CARD_H, GroupLayout};
use super::{After, Component, child_id, root_id, under};

const LIVE_CHIP: &str = "Default, Hover, Press, Focus";
const GROUPS: [&str; 2] = ["Live", "Variations"];

/// The plain question the page opens and shows.
pub(crate) fn plain() -> Modal {
    Modal::new(
        "Apply settings?",
        "Your workspace will reload to apply the new configuration.",
        "Apply",
        "Cancel",
    )
    .intent(Intent::Neutral)
}

/// The kit's destructive question (`components/DeleteFile`).
pub(crate) fn destructive() -> Modal {
    Modal::new(
        "Delete this file?",
        "This action cannot be undone. Blinky is a bit nervous about it too.",
        "Yes, delete it",
        "Cancel",
    )
    .intent(Intent::Destructive)
}

/// A dialog shown as a state: its chip, the widget, its two answers.
struct Shown {
    chip: &'static str,
    modal: Modal,
    answers: [&'static str; 2],
}

pub struct DialogPage {
    root: Option<SemId>,
    open: Button,
    delete: Button,
    shown: Vec<Shown>,
    /// The button a screen reader focused (its ring shows): the
    /// destructive one when `true`. Escape or a click lets it go.
    focus: Option<bool>,
}

fn group_id(root: &SemId, g: usize) -> Option<SemId> {
    child_id(Some(root), "group").map(|family| family.key(g.to_string()))
}

fn card_id(root: &SemId, g: usize, c: usize) -> Option<SemId> {
    child_id(group_id(root, g).as_ref(), "card").map(|family| family.key(c.to_string()))
}

impl DialogPage {
    pub fn new() -> Self {
        let root = root_id("dialog_page");
        let open = Button::new("Open dialog").variant(ButtonVariant::Outline);
        let delete = Button::new("Delete file…")
            .variant(ButtonVariant::Warning)
            .icon("trash");
        let open = match child_id(root.as_ref(), "open") {
            Some(id) => open.sem(id),
            None => open,
        };
        let delete = match child_id(root.as_ref(), "delete") {
            Some(id) => delete.sem(id),
            None => delete,
        };
        let shown = vec![
            Shown {
                chip: "Default",
                modal: plain(),
                answers: ["Apply", "Cancel"],
            },
            Shown {
                chip: "Confirm",
                modal: destructive(),
                answers: ["Yes, delete it", "Cancel"],
            },
        ];
        Self {
            root,
            open,
            delete,
            shown,
            focus: None,
        }
    }

    /// The live cards hold a button each; a shown dialog's card is as
    /// tall as the dialog at the card's width.
    fn layout(&self, content: Rect, theme: &Theme) -> Vec<GroupLayout> {
        let frame = CARD_H - showcase::body(Rect::new(0.0, 0.0, 0.0, CARD_H), theme).h;
        let card_w = showcase::card_width(2, content, theme);
        let body_w = showcase::body(Rect::new(0.0, 0.0, card_w, CARD_H), theme).w;
        let probe = Rect::new(0.0, 0.0, body_w, f32::MAX / 4.0);
        let tallest = self
            .shown
            .iter()
            .map(|s| s.modal.dialog_rect_in(theme, probe).h)
            .fold(0.0, f32::max);
        let button_h = self.open.preferred_size(theme).1;
        let live = showcase::card_height(button_h, theme);
        let groups = [
            (2, vec![live; 2]),
            (2, vec![CARD_H.max(frame + tallest); self.shown.len()]),
        ];
        showcase::layout_sized(groups, content, theme)
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        showcase::content_height(&self.layout(content, theme), content)
    }

    fn button_rects(&self, content: Rect, theme: &Theme) -> [Rect; 2] {
        let cards = &self.layout(content, theme)[0].cards;
        [
            showcase::centered(self.open.preferred_size(theme), cards[0]),
            showcase::centered(self.delete.preferred_size(theme), cards[1]),
        ]
    }

    /// Where each shown dialog sits: centered at the top of its card's
    /// body, as wide as the body lets it be.
    fn dialogs(&self, content: Rect, theme: &Theme) -> Vec<Rect> {
        let layout = self.layout(content, theme);
        self.shown
            .iter()
            .zip(&layout[1].cards)
            .map(|(s, card)| {
                let body = showcase::body(*card, theme);
                let mut d = s.modal.dialog_rect_in(theme, body);
                d.y = body.y;
                d
            })
            .collect()
    }

    /// Which button `id` is: `Some(destructive)`.
    fn which(&self, id: &SemId) -> Option<bool> {
        if self.open.sem_id() == Some(id) {
            Some(false)
        } else if self.delete.sem_id() == Some(id) {
            Some(true)
        } else {
            None
        }
    }

    /// The reader's focus on one button, or on none.
    fn set_focus(&mut self, on: Option<bool>) {
        self.focus = on;
        self.open.set_focused(on == Some(false));
        self.delete.set_focused(on == Some(true));
    }

    /// Route an event; the second value is `Some(destructive)` when a
    /// button asked for its dialog.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
    ) -> (EventResult, Option<bool>) {
        let blurred = matches!(event, WidgetEvent::MouseDown { .. }) && self.blur();
        let rects = self.button_rects(content, theme);
        let mut action = None;
        let r = self.open.handle_event(event, rects[0]);
        if r.clicked {
            action = Some(false);
        }
        let d = self.delete.handle_event(event, rects[1]);
        if d.clicked {
            action = Some(true);
        }
        let mut result = r.merge(d);
        if blurred {
            result = result.merge(EventResult::changed());
        }
        (result, action)
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let layout = self.layout(content, theme);
        for (g, at) in layout.iter().enumerate() {
            showcase::title(c, GROUPS[g], content, at.title_y, theme);
            if g + 1 != layout.len() {
                showcase::rule(c, content, at.rule_y, theme);
            }
        }
        let rects = self.button_rects(content, theme);
        for ((button, rect), card) in [&self.open, &self.delete]
            .into_iter()
            .zip(rects)
            .zip(&layout[0].cards)
        {
            showcase::card(c, *card, LIVE_CHIP, theme);
            button.render(c, rect, theme);
        }
        let dialogs = self.dialogs(content, theme);
        for ((shown, card), dialog) in self.shown.iter().zip(&layout[1].cards).zip(dialogs) {
            showcase::card(c, *card, shown.chip, theme);
            shown.modal.render_in(c, LayerId::DEFAULT, theme, dialog);
        }
    }
}

impl Component for DialogPage {
    fn root(&self) -> Option<&SemId> {
        self.root.as_ref()
    }

    fn project(&self, b: &mut SnapshotBuilder, root: &SemId) {
        let (Some(live), Some(states)) = (group_id(root, 0), group_id(root, 1)) else {
            return;
        };
        b.node(&live, Role::Group, GROUPS[0]).parent(root);
        for button in [&self.open, &self.delete] {
            if let Some(id) = button.sem_id() {
                button.project(b, id, Some(&live)).description(LIVE_CHIP);
            }
        }
        let focused = match self.focus {
            Some(true) => self.delete.sem_id(),
            Some(false) => self.open.sem_id(),
            None => None,
        };
        if let Some(id) = focused {
            b.focus(id);
        }
        b.node(&states, Role::Group, GROUPS[1]).parent(root);
        for (c, shown) in self.shown.iter().enumerate() {
            let Some(card) = card_id(root, 1, c) else {
                continue;
            };
            b.node(&card, Role::Group, shown.modal.title.as_str())
                .parent(&states)
                .description(shown.modal.body.as_str());
            for (i, answer) in shown.answers.iter().enumerate() {
                b.node(card.key(i.to_string()), Role::Label, *answer)
                    .parent(&card);
            }
        }
    }

    /// A button's `activate` opens its modal, as its click does.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        match (self.which(&req.target), req.action.as_str()) {
            (Some(destructive), "activate") => Ok(After::Modal { destructive }),
            _ => Err(format!(
                "the dialog page has no {} on {}",
                req.action, req.target
            )),
        }
    }

    fn focus(&mut self, id: &SemId) -> bool {
        if !self.root.as_ref().is_some_and(|root| under(root, id)) {
            return false;
        }
        self.set_focus(self.which(id));
        true
    }

    fn blur(&mut self) -> bool {
        let had = self.focus.is_some();
        self.set_focus(None);
        had
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// From 280 to 1600 every 8 px, core and hoff: every shown dialog fits
    /// its card's body, each live button its card.
    #[test]
    fn every_dialog_fits_its_card_across_the_width_range() {
        for theme in [Theme::hoff_core(), Theme::hoff()] {
            let page = DialogPage::new();
            let mut w = 280.0;
            while w <= 1600.0 {
                let content = Rect::new(0.0, 0.0, w, 10_000.0);
                let layout = page.layout(content, &theme);
                for (dialog, card) in page.dialogs(content, &theme).iter().zip(&layout[1].cards) {
                    let body = showcase::body(*card, &theme);
                    assert!(
                        dialog.x >= body.x - 0.01 && dialog.right() <= body.right() + 0.01,
                        "{w}: wider than its card"
                    );
                    assert!(dialog.bottom() <= body.bottom() + 0.01, "{w}: taller");
                }
                let rects = page.button_rects(content, &theme);
                for (button, card) in rects.iter().zip(&layout[0].cards) {
                    assert!(
                        button.x >= card.x && button.right() <= card.right(),
                        "{w}: a button"
                    );
                }
                w += 8.0;
            }
        }
    }
}
