//! Sheet page: a panel held to any edge of the window, over a scrim,
//! blocking the page until it is answered. The widget is
//! `engine::widgets::overlay::sheet::Sheet` on `theme.overlay.sheet`: the
//! kit's drawer surface (`components/Layout`) with its settings panel's
//! title and option rows (`components/Settings`), and the dialog's footer.
//! The view owns the sheet on screen (its overlay stack, the scrim, the
//! focus trap), as it owns the modal.
//!
//! "Live" opens it from each edge: it slides in, the focus is its close,
//! Escape, the close, the cancel or a press on the scrim cancels, the
//! save confirms. "States" shows it held to an edge of its card, as it
//! sits in a window: from the right and from the bottom.
//!
//! Projected: each live button `sheet_page.group[0].card[<e>]`, whose
//! `activate` opens the sheet (the view's `sheet`) from its edge; a shown
//! sheet, `sheet_page.group[1].card[<c>]`, a group named by its title and
//! described by its description, its rows as labels.

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::drawing::compositor::{Compositor, LayerId};
use engine::styling::theme::Theme;
use engine::widgets::overlay::sheet::{Edge, Sheet, SheetRow, edge_rect, paint_scrim};
use engine::widgets::prelude::{Button, EventResult, Rect, WidgetEvent};

use super::showcase::{self, GroupLayout};
use super::{After, Component, child_id, root_id, under};

const LIVE_CHIP: &str = "Default, Hover, Press, Focus";
const GROUPS: [&str; 2] = ["Live", "States"];
/// The edges the states show the sheet held to.
const SHOWN: [Edge; 2] = [Edge::Right, Edge::Bottom];

/// The sheet the page opens and shows: the kit's profile settings.
pub(crate) fn demo(edge: Edge) -> Sheet {
    Sheet::new(
        "Profile",
        "Changes apply to every workspace once you save them.",
        "Save",
        "Cancel",
    )
    .edge(edge)
    .row(SheetRow::new("Private profile", "Off"))
    .row(SheetRow::new("Avatar", "1.png").hint("Use 800x800 px (PNG/JPG)"))
    .row(SheetRow::new("Display name", "Sophie Bennett"))
}

fn edge_title(edge: Edge) -> &'static str {
    match edge {
        Edge::Top => "Top",
        Edge::Right => "Right",
        Edge::Bottom => "Bottom",
        Edge::Left => "Left",
    }
}

pub struct SheetPage {
    root: Option<SemId>,
    /// One live button per edge, in `Edge::ALL`'s order.
    buttons: Vec<Button>,
    shown: Vec<Sheet>,
    focus: Option<usize>,
}

fn group_id(root: &SemId, g: usize) -> Option<SemId> {
    child_id(Some(root), "group").map(|family| family.key(g.to_string()))
}

fn card_id(root: &SemId, g: usize, c: usize) -> Option<SemId> {
    child_id(group_id(root, g).as_ref(), "card").map(|family| family.key(c.to_string()))
}

impl SheetPage {
    pub fn new() -> Self {
        let root = root_id("sheet_page");
        let buttons = Edge::ALL
            .iter()
            .enumerate()
            .map(|(e, edge)| {
                let button = Button::new(format!("Open from the {}", edge.name()));
                match root.as_ref().and_then(|r| card_id(r, 0, e)) {
                    Some(id) => button.sem(id),
                    None => button,
                }
            })
            .collect();
        Self {
            root,
            buttons,
            shown: SHOWN.iter().map(|e| demo(*e)).collect(),
            focus: None,
        }
    }

    /// The body a state's card needs at `body_w`: a side sheet's content
    /// at its width, a bottom one's with its touch target of scrim.
    fn state_h(sheet: &Sheet, theme: &Theme, body_w: f32) -> f32 {
        let probe = Rect::new(0.0, 0.0, body_w, f32::MAX / 4.0);
        let panel = edge_rect(sheet.edge, theme.overlay.sheet.panel.extent, probe, theme);
        if sheet.edge.is_side() {
            sheet.content_h(theme, panel.w)
        } else {
            sheet.content_h(theme, body_w) + theme.control.min_touch_target
        }
    }

    fn layout(&self, content: Rect, theme: &Theme) -> Vec<GroupLayout> {
        let frame = showcase::body_height(0.0, theme);
        let card_w = showcase::card_width(2, content, theme);
        let body_w = showcase::body(Rect::new(0.0, 0.0, card_w, 0.0), theme).w;
        let tallest = self
            .shown
            .iter()
            .map(|s| Self::state_h(s, theme, body_w))
            .fold(0.0, f32::max);
        let button_h = self.buttons.iter().map(|b| b.preferred_size(theme).1);
        let live = showcase::card_height(button_h.fold(0.0, f32::max), theme);
        let groups = [
            (GROUPS[0], 2, vec![live; self.buttons.len()]),
            (GROUPS[1], 2, vec![frame + tallest; self.shown.len()]),
        ];
        showcase::layout_sized(groups, content, theme)
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        showcase::content_height(&self.layout(content, theme), content)
    }

    fn button_rects(&self, content: Rect, theme: &Theme) -> Vec<Rect> {
        let layout = self.layout(content, theme);
        self.buttons
            .iter()
            .zip(&layout[0].cards)
            .map(|(b, card)| showcase::centered(b.preferred_size(theme), *card))
            .collect()
    }

    /// Each shown sheet's container (its card's body) and its panel.
    fn states(&self, content: Rect, theme: &Theme) -> Vec<(Rect, Rect)> {
        let layout = self.layout(content, theme);
        self.shown
            .iter()
            .zip(&layout[1].cards)
            .map(|(s, card)| {
                let body = showcase::body(*card, theme);
                (body, s.panel_rect(theme, body))
            })
            .collect()
    }

    /// A click on a live button opens the sheet from its edge, through
    /// the view.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
    ) -> (EventResult, Option<After>) {
        let mut r = EventResult::IGNORED;
        if matches!(event, WidgetEvent::MouseDown { .. }) && self.focus.is_some() {
            self.set_focus(None);
            r = EventResult::changed();
        }
        let rects = self.button_rects(content, theme);
        let mut after = None;
        for ((button, rect), edge) in self.buttons.iter_mut().zip(rects).zip(Edge::ALL) {
            let pressed = button.handle_event(event, rect);
            if pressed.clicked {
                after = Some(After::Sheet(edge));
            }
            r = r.merge(pressed);
        }
        (r, after)
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
        for ((button, rect), card) in self.buttons.iter().zip(rects).zip(&layout[0].cards) {
            showcase::card(c, *card, LIVE_CHIP, theme);
            button.render(c, rect, theme);
        }
        let states = self.states(content, theme);
        for ((sheet, card), (body, panel)) in self.shown.iter().zip(&layout[1].cards).zip(states) {
            showcase::card(c, *card, edge_title(sheet.edge), theme);
            paint_scrim(c, LayerId::DEFAULT, body, &theme.overlay.sheet.panel, 1.0);
            sheet.render_in(c, LayerId::DEFAULT, theme, panel, body);
        }
    }

    fn set_focus(&mut self, at: Option<usize>) {
        self.focus = at;
        for (i, b) in self.buttons.iter_mut().enumerate() {
            b.set_focused(at == Some(i));
        }
    }
}

impl Component for SheetPage {
    fn root(&self) -> Option<&SemId> {
        self.root.as_ref()
    }

    fn project(&self, b: &mut SnapshotBuilder, root: &SemId) {
        let (Some(live), Some(states)) = (group_id(root, 0), group_id(root, 1)) else {
            return;
        };
        b.node(&live, Role::Group, GROUPS[0]).parent(root);
        for (i, button) in self.buttons.iter().enumerate() {
            if let Some(id) = button.sem_id() {
                button.project(b, id, Some(&live)).description(LIVE_CHIP);
                if self.focus == Some(i) {
                    b.focus(id);
                }
            }
        }
        b.node(&states, Role::Group, GROUPS[1]).parent(root);
        for (c, sheet) in self.shown.iter().enumerate() {
            let Some(card) = card_id(root, 1, c) else {
                continue;
            };
            b.node(&card, Role::Group, sheet.title.as_str())
                .parent(&states)
                .description(sheet.description.as_str());
            for (i, row) in sheet.rows.iter().enumerate() {
                let name = format!("{}: {}", row.label, row.value);
                b.node(card.key(i.to_string()), Role::Label, name)
                    .parent(&card);
            }
        }
    }

    /// A live button's `activate`: the sheet opens from its edge.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        let at = self
            .buttons
            .iter()
            .position(|b| b.sem_id() == Some(&req.target));
        match (at, req.action.as_str()) {
            (Some(e), "activate") => Ok(After::Sheet(Edge::ALL[e])),
            _ => Err(format!(
                "the sheet page has no {} on {}",
                req.action, req.target
            )),
        }
    }

    fn focus(&mut self, id: &SemId) -> bool {
        if !self.root.as_ref().is_some_and(|root| under(root, id)) {
            return false;
        }
        let at = self.buttons.iter().position(|b| b.sem_id() == Some(id));
        self.set_focus(at);
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

    /// From 280 to 1600 every 8 px, core and hoff: every shown sheet sits
    /// in its card's body, held to its edge, its content inside it; every
    /// live button fits its card.
    #[test]
    fn every_sheet_fits_its_card_across_the_width_range() {
        for theme in [Theme::hoff_core(), Theme::hoff()] {
            let page = SheetPage::new();
            let mut w = 280.0;
            while w <= 1600.0 {
                let content = Rect::new(0.0, 0.0, w, 10_000.0);
                for ((body, panel), sheet) in page.states(content, &theme).iter().zip(&page.shown) {
                    let inside = panel.x >= body.x - 0.01
                        && panel.right() <= body.right() + 0.01
                        && panel.y >= body.y - 0.01
                        && panel.bottom() <= body.bottom() + 0.01;
                    assert!(inside, "{w}: {:?} outside its card", sheet.edge);
                    let need = sheet.content_h(&theme, panel.w);
                    assert!(need <= panel.h + 0.01, "{w}: {:?} cut", sheet.edge);
                }
                let layout = page.layout(content, &theme);
                for (rect, card) in page
                    .button_rects(content, &theme)
                    .iter()
                    .zip(&layout[0].cards)
                {
                    assert!(
                        rect.x >= card.x && rect.right() <= card.right(),
                        "{w}: a button"
                    );
                }
                w += 8.0;
            }
        }
    }

    #[test]
    fn a_live_button_asks_for_the_sheet_from_its_edge() {
        let mut page = SheetPage::new();
        let Some(id) = page.buttons[2].sem_id().cloned() else {
            panic!("the ids parse");
        };
        let root = page.root.clone().unwrap();
        let mut b = SnapshotBuilder::new();
        b.node(&root, Role::Region, "Sheet");
        page.project(&mut b, &root);
        let seen = b.finish().unwrap();
        let req = Request::observe(&seen, accord::Actor::Human, &id, "activate", None).unwrap();
        assert_eq!(page.run(&req), Ok(After::Sheet(Edge::Bottom)));
    }
}
