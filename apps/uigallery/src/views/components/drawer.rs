//! Drawer page: the kit's navigation slid in from the window's left over
//! a scrim (`components/Layout` below 1023 px, its `GuidelineSidebar` as
//! a drawer). The widget is `engine::widgets::overlay::drawer::Drawer` on
//! `theme.overlay.drawer`, the sheet's edge panel with navigation inside;
//! the view owns it on screen, as it owns the sheet.
//!
//! "Live" opens it: it slides in, the focus is its current link, a link
//! picked closes it (a toast names the link), and so do the close, a
//! press on the scrim and Escape. "States" shows it open, held to the
//! left of its card, the current link raised.
//!
//! Projected: the live button `drawer_page.group[0].card[0]`, whose
//! `activate` opens the drawer (the view's `drawer`); the shown drawer,
//! `drawer_page.group[1].card[0]`, a group named by the app's name with
//! its links as labels, the current one said so.

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::drawing::compositor::{Compositor, LayerId};
use engine::styling::theme::Theme;
use engine::widgets::overlay::drawer::Drawer;
use engine::widgets::overlay::sheet::paint_scrim;
use engine::widgets::prelude::{Button, EventResult, Rect, WidgetEvent};

use super::showcase::{self, CARD_H, GroupLayout};
use super::{After, Component, child_id, root_id, under};

const LIVE_CHIP: &str = "Default, Hover, Press, Focus";
const STATE_CHIP: &str = "Open, current link";
const GROUPS: [&str; 2] = ["Live", "States"];

/// The drawer the page opens and shows: the kit's guideline navigation,
/// its colors page current.
pub(crate) fn demo() -> Drawer {
    Drawer::new("Research")
        .group(
            "Foundations",
            &[
                ("Branding", "layout-grid"),
                ("Colors", "layers"),
                ("Depths", "square"),
                ("Icons", "house"),
            ],
        )
        .group(
            "Components",
            &[
                ("Buttons", "plus"),
                ("Inputs", "search"),
                ("Modals", "info"),
            ],
        )
        .current(Some(1))
}

pub struct DrawerPage {
    root: Option<SemId>,
    button: Button,
    shown: Drawer,
    focus: bool,
}

fn group_id(root: &SemId, g: usize) -> Option<SemId> {
    child_id(Some(root), "group").map(|family| family.key(g.to_string()))
}

fn card_id(root: &SemId, g: usize, c: usize) -> Option<SemId> {
    child_id(group_id(root, g).as_ref(), "card").map(|family| family.key(c.to_string()))
}

impl DrawerPage {
    pub fn new() -> Self {
        let root = root_id("drawer_page");
        let mut button = Button::new("Open navigation");
        if let Some(id) = root.as_ref().and_then(|r| card_id(r, 0, 0)) {
            button = button.sem(id);
        }
        Self {
            root,
            button,
            shown: demo(),
            focus: false,
        }
    }

    fn layout(&self, content: Rect, theme: &Theme) -> Vec<GroupLayout> {
        let frame = CARD_H - showcase::body(Rect::new(0.0, 0.0, 0.0, CARD_H), theme).h;
        let live = showcase::card_height(self.button.preferred_size(theme).1, theme);
        let state = CARD_H.max(frame + self.shown.content_h(theme));
        showcase::layout_sized([(1, vec![live]), (1, vec![state])], content, theme)
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        showcase::content_height(&self.layout(content, theme), content)
    }

    fn button_rect(&self, content: Rect, theme: &Theme) -> Rect {
        let card = self.layout(content, theme)[0].cards[0];
        showcase::centered(self.button.preferred_size(theme), card)
    }

    /// The shown drawer's container (its card's body) and its panel.
    fn state(&self, content: Rect, theme: &Theme) -> (Rect, Rect) {
        let card = self.layout(content, theme)[1].cards[0];
        let body = showcase::body(card, theme);
        (body, self.shown.panel_rect(theme, body))
    }

    /// A click on the live button opens the drawer, through the view.
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
        let pressed = self
            .button
            .handle_event(event, self.button_rect(content, theme));
        let after = pressed.clicked.then_some(After::Drawer);
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
        showcase::card(c, layout[1].cards[0], STATE_CHIP, theme);
        let (body, panel) = self.state(content, theme);
        paint_scrim(c, LayerId::DEFAULT, body, &theme.overlay.drawer.panel, 1.0);
        self.shown
            .render_in(c, LayerId::DEFAULT, theme, panel, body);
    }

    fn set_focus(&mut self, on: bool) {
        self.focus = on;
        self.button.set_focused(on);
    }
}

impl Component for DrawerPage {
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
        let Some(card) = card_id(root, 1, 0) else {
            return;
        };
        b.node(&card, Role::Group, self.shown.title.as_str())
            .parent(&states)
            .description(STATE_CHIP);
        for (i, link) in self.shown.links().enumerate() {
            let mut node = b.node(card.key(i.to_string()), Role::Label, link.label.as_str());
            node.parent(&card);
            node.state("current", self.shown.current == Some(i));
        }
    }

    /// The live button's `activate`: the drawer opens.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        if self.button.sem_id() == Some(&req.target) && req.action.as_str() == "activate" {
            return Ok(After::Drawer);
        }
        Err(format!(
            "the drawer page has no {} on {}",
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

    /// From 280 to 1600 every 8 px, core and hoff: the shown drawer sits
    /// in its card's body, held to its left, with every link inside it,
    /// and leaves scrim to close it by; the live button fits its card.
    #[test]
    fn the_drawer_fits_its_card_across_the_width_range() {
        for theme in [Theme::hoff_core(), Theme::hoff()] {
            let page = DrawerPage::new();
            let mut w = 280.0;
            while w <= 1600.0 {
                let content = Rect::new(0.0, 0.0, w, 10_000.0);
                let (body, panel) = page.state(content, &theme);
                assert_eq!(panel.x, body.x, "{w}: held to the left");
                assert!(
                    panel.right() <= body.right() + 0.01,
                    "{w}: wider than its card"
                );
                let need = page.shown.content_h(&theme);
                assert!(need <= panel.h + 0.01, "{w}: links cut");
                let free = body.w - panel.w;
                assert!(
                    free >= theme.control.min_touch_target - 0.01,
                    "{w}: no scrim"
                );
                let card = page.layout(content, &theme)[0].cards[0];
                let button = page.button_rect(content, &theme);
                assert!(button.x >= card.x && button.right() <= card.right(), "{w}");
                w += 8.0;
            }
        }
    }
}
