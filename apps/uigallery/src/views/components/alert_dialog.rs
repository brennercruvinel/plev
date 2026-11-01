//! Alert Dialog page, the hoff core kit's dialog (`components/DeleteFile`,
//! the ModalPage template): a destructive question that blocks the page
//! until it is answered. The widget is `engine::widgets::feedback::Modal`
//! on `theme.overlay.modal`; the view owns the modal on screen (its
//! overlay stack, the scrim, the focus trap).
//!
//! "Live" opens it: the scrim covers the window, the focus is the cancel
//! (the answer that changes nothing), Escape or a press on the scrim
//! cancels. "States" shows the dialog as the kit's ModalPage does, in
//! cards: the destructive question and a plain one.
//!
//! Projected: the live button `alert_dialog_page.group[0].card[0]`, whose
//! `activate` opens the modal (the view's `dialog`); a shown dialog,
//! `alert_dialog_page.group[1].card[<c>]`, a group named by its title and
//! described by its body, its two answers as labels.

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::drawing::compositor::{Compositor, LayerId};
use engine::styling::theme::Theme;
use engine::widgets::prelude::{Button, ButtonVariant, EventResult, Modal, Rect, WidgetEvent};

use super::showcase::{self, GroupLayout};
use super::{After, Component, child_id, dialog, root_id, under};

const LIVE_CHIP: &str = "Default, Hover, Press, Focus";
const GROUPS: [&str; 2] = ["Live", "States"];

/// A dialog shown as a state: its chip, the widget, its two answers.
struct Shown {
    chip: &'static str,
    modal: Modal,
    answers: [&'static str; 2],
}

pub struct AlertDialogPage {
    root: Option<SemId>,
    button: Button,
    shown: Vec<Shown>,
    focus: bool,
}

fn group_id(root: &SemId, g: usize) -> Option<SemId> {
    child_id(Some(root), "group").map(|family| family.key(g.to_string()))
}

fn card_id(root: &SemId, g: usize, c: usize) -> Option<SemId> {
    child_id(group_id(root, g).as_ref(), "card").map(|family| family.key(c.to_string()))
}

impl AlertDialogPage {
    pub fn new() -> Self {
        let root = root_id("alert_dialog_page");
        let mut button = Button::new("Delete file").variant(ButtonVariant::Warning);
        if let Some(id) = root.as_ref().and_then(|r| card_id(r, 0, 0)) {
            button = button.sem(id);
        }
        let shown = vec![
            Shown {
                chip: "Destructive",
                modal: dialog::destructive(),
                answers: ["Yes, delete it", "Cancel"],
            },
            Shown {
                chip: "Neutral",
                modal: dialog::plain(),
                answers: ["Apply", "Cancel"],
            },
        ];
        Self {
            root,
            button,
            shown,
            focus: false,
        }
    }

    /// The live card holds the button; a shown dialog's card is as tall
    /// as the dialog at the card's width.
    fn layout(&self, content: Rect, theme: &Theme) -> Vec<GroupLayout> {
        let frame = showcase::body_height(0.0, theme);
        // The body a shown dialog's card will have at this width.
        let card_w = showcase::card_width(2, content, theme);
        let body_w = showcase::body(Rect::new(0.0, 0.0, card_w, 0.0), theme).w;
        let probe = Rect::new(0.0, 0.0, body_w, f32::MAX / 4.0);
        // Each card as tall as its sheet and the art that rises above it;
        // a row is as tall as its tallest card.
        let heights: Vec<f32> = self
            .shown
            .iter()
            .map(|s| {
                let d = s.modal.dialog_rect_in(theme, probe);
                frame + d.h + s.modal.overhang(theme, d.w)
            })
            .collect();
        let live = showcase::card_height(self.button.preferred_size(theme).1, theme);
        showcase::layout_sized([(1, vec![live]), (2, heights)], content, theme)
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        showcase::content_height(&self.layout(content, theme), content)
    }

    fn button_rect(&self, content: Rect, theme: &Theme) -> Rect {
        let card = self.layout(content, theme)[0].cards[0];
        showcase::centered(self.button.preferred_size(theme), card)
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
                d.y = body.y + s.modal.overhang(theme, d.w);
                d
            })
            .collect()
    }

    /// A click on the live button opens the modal, through the view.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
    ) -> (EventResult, Option<After>) {
        let mut r = EventResult::IGNORED;
        if matches!(event, WidgetEvent::MouseDown { .. }) && self.focus {
            self.set_focus(false);
            r = EventResult::changed();
        }
        let rect = self.button_rect(content, theme);
        let pressed = self.button.handle_event(event, rect);
        let after = pressed
            .clicked
            .then_some(After::Modal { destructive: true });
        (r.merge(pressed), after)
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let layout = self.layout(content, theme);
        for (g, at) in layout.iter().enumerate() {
            showcase::title(c, GROUPS[g], content, at.title_y, theme);
            if g + 1 != layout.len() {
                showcase::rule(c, content, at.rule_y, theme);
            }
        }
        showcase::card(c, layout[0].cards[0], LIVE_CHIP, theme);
        self.button
            .render(c, self.button_rect(content, theme), theme);
        let dialogs = self.dialogs(content, theme);
        for ((shown, card), dialog) in self.shown.iter().zip(&layout[1].cards).zip(dialogs) {
            showcase::card(c, *card, shown.chip, theme);
            shown.modal.render_in(c, LayerId::DEFAULT, theme, dialog);
        }
    }

    fn set_focus(&mut self, on: bool) {
        self.focus = on;
        self.button.set_focused(on);
    }
}

impl Component for AlertDialogPage {
    fn root(&self) -> Option<&SemId> {
        self.root.as_ref()
    }

    fn project(&self, b: &mut SnapshotBuilder, root: &SemId) {
        let (Some(live), Some(states)) = (group_id(root, 0), group_id(root, 1)) else {
            return;
        };
        b.node(&live, Role::Group, GROUPS[0]).parent(root);
        if let Some(id) = self.button.sem_id() {
            self.button
                .project(b, id, Some(&live))
                .description(LIVE_CHIP);
            if self.focus {
                b.focus(id);
            }
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

    /// The live button's `activate`: the destructive modal opens.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        if self.button.sem_id() == Some(&req.target) && req.action.as_str() == "activate" {
            return Ok(After::Modal { destructive: true });
        }
        Err(format!(
            "the alert dialog page has no {} on {}",
            req.action, req.target
        ))
    }

    fn focus(&mut self, id: &SemId) -> bool {
        if !self.root.as_ref().is_some_and(|root| under(root, id)) {
            return false;
        }
        self.set_focus(self.button.sem_id() == Some(id));
        true
    }

    fn blur(&mut self) -> bool {
        let had = self.focus;
        self.set_focus(false);
        had
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// From 280 to 1600 every 8 px, core and hoff: every shown dialog fits
    /// its card's body, the live button its card.
    #[test]
    fn every_dialog_fits_its_card_across_the_width_range() {
        for theme in [Theme::hoff_core(), Theme::hoff()] {
            let page = AlertDialogPage::new();
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
                    assert!(
                        dialog.bottom() <= body.bottom() + 0.01,
                        "{w}: taller than its card"
                    );
                }
                let button = page.button_rect(content, &theme);
                let card = layout[0].cards[0];
                assert!(
                    button.x >= card.x && button.right() <= card.right(),
                    "{w}: the button"
                );
                w += 8.0;
            }
        }
    }
}
