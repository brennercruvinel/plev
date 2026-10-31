//! A form page: the kit's showcase (groups with a title and a rule, cards
//! behind a chip naming their states) over widgets of the form family,
//! a field, a label. The Field and Label tabs are each one of these with
//! their own groups. The page owns the keyboard focus as the Input page
//! does: a click in a field places the caret, Tab reaches the fields in
//! the projection's order, Escape blurs; the characters, editing keys,
//! clipboard chords and the input method go to the focused field.
//!
//! Projected: a group per title, `<root>.group[<g>]`, and in it each
//! card's widget, `<root>.group[<g>].card[<c>]`, as the comps projects it,
//! a field with its eye (`….reveal`) and link (`….link`) beside it.
//! `set_value` puts a field's text as typing would, `toggle` reveals or
//! hides a password, and a link's `activate` does nothing more than a
//! click.

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::drawing::compositor::{Compositor, LayerId};
use engine::styling::theme::Theme;
use engine::widgets::form::TextField;
use engine::widgets::prelude::{EventResult, Rect, Wake, WidgetEvent};

use super::super::showcase;
use super::super::{After, Component, EditKey, child_id, root_id, under};
use super::spec::{Part, Spec};
use crate::views::ImeEvent;
use crate::views::text_input::clipboard::Keys;
use crate::views::text_input::host::TextHost;

pub(crate) struct Card {
    /// The chip: which states this card shows.
    pub(crate) title: &'static str,
    pub(crate) spec: Spec,
}

pub(crate) struct Group {
    pub(crate) title: &'static str,
    /// Cards per row at full width.
    pub(crate) columns: usize,
    pub(crate) cards: Vec<Card>,
}

pub(crate) fn card(title: &'static str, spec: impl Into<Spec>) -> Card {
    Card {
        title,
        spec: spec.into(),
    }
}

/// One group laid out: its title's y, each card with its widget's rect,
/// and the y of its rule.
pub(crate) struct Laid {
    pub(crate) title_y: f32,
    pub(crate) cards: Vec<(Rect, Rect)>,
    pub(crate) rule_y: f32,
}

pub struct FormPage {
    pub(crate) root: Option<SemId>,
    pub(crate) groups: Vec<Group>,
}

/// `<root>.group[<g>]`.
fn group_id(root: &SemId, g: usize) -> Option<SemId> {
    child_id(Some(root), "group").map(|family| family.key(g.to_string()))
}

/// `<root>.group[<g>].card[<c>]`.
fn card_id(root: &SemId, g: usize, c: usize) -> Option<SemId> {
    child_id(group_id(root, g).as_ref(), "card").map(|family| family.key(c.to_string()))
}

impl FormPage {
    /// The page `root` (`field_page`), its groups' widgets given their ids.
    pub(crate) fn new(root: &str, mut groups: Vec<Group>) -> Self {
        let root = root_id(root);
        if let Some(root) = &root {
            for (g, group) in groups.iter_mut().enumerate() {
                for (c, card) in std::mem::take(&mut group.cards).into_iter().enumerate() {
                    let spec = match card_id(root, g, c) {
                        Some(id) => card.spec.sem(id),
                        None => card.spec,
                    };
                    group.cards.push(Card { spec, ..card });
                }
            }
        }
        Self { root, groups }
    }

    pub(crate) fn specs(&self) -> impl Iterator<Item = &Spec> {
        self.groups
            .iter()
            .flat_map(|g| g.cards.iter().map(|c| &c.spec))
    }

    fn specs_mut(&mut self) -> impl Iterator<Item = &mut Spec> {
        self.groups
            .iter_mut()
            .flat_map(|g| g.cards.iter_mut().map(|c| &mut c.spec))
    }

    pub(crate) fn spec(&self, (g, c): (usize, usize)) -> Option<&Spec> {
        Some(&self.groups.get(g)?.cards.get(c)?.spec)
    }

    fn spec_mut(&mut self, (g, c): (usize, usize)) -> Option<&mut Spec> {
        Some(&mut self.groups.get_mut(g)?.cards.get_mut(c)?.spec)
    }

    /// The field of `at`, when its card shows one.
    #[cfg(test)]
    pub(crate) fn field(&self, at: (usize, usize)) -> Option<&engine::widgets::form::field::Field> {
        self.spec(at)?.field()
    }

    /// The card whose widget, or a part of it, is `id`.
    fn at(&self, id: &SemId) -> Option<((usize, usize), Part)> {
        self.groups.iter().enumerate().find_map(|(g, group)| {
            group
                .cards
                .iter()
                .enumerate()
                .find_map(|(c, card)| Some(((g, c), card.spec.part(id)?)))
        })
    }

    /// The card whose text box has the caret.
    pub fn focused(&self) -> Option<(usize, usize)> {
        self.groups.iter().enumerate().find_map(|(g, group)| {
            let c = group
                .cards
                .iter()
                .position(|c| c.spec.text().is_some_and(|t| t.is_focused()))?;
            Some((g, c))
        })
    }

    fn focused_text(&mut self) -> Option<&mut TextField> {
        self.focused_spec()?.text_mut()
    }

    /// The widget whose text box has the caret.
    fn focused_spec(&mut self) -> Option<&mut Spec> {
        let at = self.focused()?;
        self.spec_mut(at)
    }

    /// The caret goes to the box of `at` (one not disabled), or to none.
    pub fn set_focused(&mut self, at: Option<(usize, usize)>) {
        for (g, group) in self.groups.iter_mut().enumerate() {
            for (c, card) in group.cards.iter_mut().enumerate() {
                let on = at == Some((g, c));
                if card.spec.text().is_some_and(|t| t.is_focused() != on) {
                    card.spec.set_focused(on);
                }
            }
        }
    }

    /// Blur the focused field and let go of a part a reader focused.
    /// `false` when nothing was.
    fn blur_all(&mut self) -> bool {
        let had = self.focused().is_some() || self.part_focused();
        self.set_focused(None);
        self.clear_part_focus();
        had
    }

    /// Whether a reader's focus is on a part other than a box.
    fn part_focused(&self) -> bool {
        self.specs().any(|s| s.part_focus().is_some())
    }

    fn clear_part_focus(&mut self) {
        for spec in self.specs_mut() {
            spec.set_part_focus(None);
        }
    }

    /// Per group: the title's y, the cards with their widgets, the rule's
    /// y, on the showcase's grid, a card as tall as its widget needs (at
    /// least `h-50`) and the widget centered in it, clear of the chip.
    pub(crate) fn layout(&self, content: Rect, theme: &Theme) -> Vec<Laid> {
        let side = theme.spacing.lg;
        let sizes: Vec<Vec<(f32, f32)>> = self
            .groups
            .iter()
            .map(|g| {
                let card_w = showcase::card_width(g.columns, content, theme);
                g.cards
                    .iter()
                    .map(|c| c.spec.size_in(card_w, side, theme))
                    .collect()
            })
            .collect();
        let wanted = self.groups.iter().zip(&sizes).map(|(g, s)| {
            let h = s.iter().map(|(_, h)| showcase::card_height(*h, theme));
            (g.columns, h.collect())
        });
        showcase::layout_sized(wanted, content, theme)
            .into_iter()
            .zip(&sizes)
            .map(|(grid, s)| Laid {
                title_y: grid.title_y,
                cards: grid
                    .cards
                    .iter()
                    .zip(s)
                    .map(|(card, size)| (*card, showcase::centered(*size, *card)))
                    .collect(),
                rule_y: grid.rule_y,
            })
            .collect()
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        self.layout(content, theme)
            .last()
            .map_or(0.0, |laid| laid.rule_y - content.y)
    }

    /// The widget of `at`, laid out in `content`.
    pub(crate) fn widget_rect(
        &self,
        at: (usize, usize),
        content: Rect,
        theme: &Theme,
    ) -> Option<Rect> {
        Some(self.layout(content, theme).get(at.0)?.cards.get(at.1)?.1)
    }

    /// Every widget takes the pointer at its rect: hover, a press that
    /// focuses a field and blurs the others, the eye, the link, a label.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
    ) -> EventResult {
        let mut result = EventResult::IGNORED;
        if matches!(event, WidgetEvent::MouseDown { .. }) && self.part_focused() {
            self.clear_part_focus();
            result = EventResult::changed();
        }
        let layout = self.layout(content, theme);
        // What floats over the page (an open list) has the pointer first,
        // and alone where it is.
        let (x, y) = event.pos();
        for (group, laid) in self.groups.iter_mut().zip(&layout) {
            for (card, (_, rect)) in group.cards.iter_mut().zip(&laid.cards) {
                if card.spec.over_overlay(x, y, *rect, theme) {
                    return result.merge(card.spec.handle_event(event, *rect, theme));
                }
            }
        }
        for (group, laid) in self.groups.iter_mut().zip(&layout) {
            for (card, (_, rect)) in group.cards.iter_mut().zip(&laid.cards) {
                result = result.merge(card.spec.handle_event(event, *rect, theme));
            }
        }
        result
    }

    /// The caret blink of the focused field; nothing while none is.
    pub fn wake(&mut self, dt: f32) -> Wake {
        self.focused_text().map_or(Wake::idle(), |t| t.wake(dt))
    }

    /// Draws the page; what floats over it (an open list) on `overlay`.
    pub fn render(&self, c: &mut Compositor, overlay: LayerId, content: Rect, theme: &Theme) {
        let layout = self.layout(content, theme);
        for (group, laid) in self.groups.iter().zip(&layout) {
            for (card, (_, widget)) in group.cards.iter().zip(&laid.cards) {
                card.spec.render_overlay(c, overlay, *widget, theme);
            }
        }
        let last = self.groups.len().saturating_sub(1);
        for (i, (group, laid)) in self.groups.iter().zip(&layout).enumerate() {
            showcase::title(c, group.title, content, laid.title_y, theme);
            for (card, (rect, widget)) in group.cards.iter().zip(&laid.cards) {
                showcase::card(c, *rect, card.title, theme);
                card.spec.render(c, *widget, theme);
            }
            // The rule under the group (`border-b`, none on the last).
            if i != last {
                showcase::rule(c, content, laid.rule_y, theme);
            }
        }
    }
}

impl TextHost for FormPage {
    fn handle_text(&mut self, s: &str) -> bool {
        self.focused_text().is_some_and(|t| t.insert(s))
    }

    fn handle_edit_key(&mut self, key: EditKey) -> bool {
        if key == EditKey::Tab {
            return false;
        }
        self.focused_spec().is_some_and(|s| s.edit(key.into()))
    }

    fn handle_enter(&mut self) -> bool {
        self.focused_spec().is_some_and(|s| s.enter())
    }

    fn handle_nav(&mut self, down: bool) -> bool {
        self.focused_spec().is_some_and(|s| s.nav(down))
    }

    fn handle_ime(&mut self, ime: ImeEvent<'_>) -> bool {
        let (committed, preedit, cursor) = match ime {
            ImeEvent::Disabled => {
                let mut dropped = false;
                for spec in self.specs_mut() {
                    dropped |= spec.text_mut().is_some_and(|t| t.end_compose());
                }
                return dropped;
            }
            ImeEvent::Preedit(text, cursor) => ("", text, cursor),
            ImeEvent::Commit(text) => (text, "", None),
        };
        self.focused_text()
            .is_some_and(|t| t.compose(committed, preedit, cursor))
    }

    /// A right-click on a field's box: it takes the keys for its menu,
    /// keeping a selection it has, or taking a caret at the click.
    fn right_click(&mut self, x: f32, y: f32, content: Rect, theme: &Theme) -> bool {
        let layout = self.layout(content, theme);
        let hit = layout.iter().enumerate().find_map(|(g, laid)| {
            laid.cards.iter().enumerate().find_map(|(c, (_, rect))| {
                let spec = self.spec((g, c))?;
                spec.on_box(x, y, *rect, theme).then_some(((g, c), *rect))
            })
        });
        let Some((at, rect)) = hit else {
            return false;
        };
        let selected = self
            .spec(at)
            .and_then(|s| s.text())
            .is_some_and(|t| t.has_selection());
        if self.focused() == Some(at) && selected {
            return true;
        }
        self.set_focused(Some(at));
        if let Some(spec) = self.spec_mut(at) {
            spec.handle_event(&WidgetEvent::MouseDown { x, y }, rect, theme);
        }
        true
    }

    fn keys(&mut self) -> Keys<'_> {
        match self.focused_text() {
            Some(t) => Keys::Field(t),
            None => Keys::Nobody,
        }
    }

    fn ime_area(&self, content: Rect, theme: &Theme) -> Option<Rect> {
        let at = self.focused()?;
        let rect = self.widget_rect(at, content, theme)?;
        self.spec(at)?.ime_area(rect, theme)
    }

    /// Escape blurs the focused field, or lets go of the eye or the link
    /// a reader focused. `false` when nothing was.
    fn handle_escape(&mut self) -> bool {
        if self.focused_spec().is_some_and(|s| s.escape()) {
            return true;
        }
        self.blur_all()
    }
}

impl Component for FormPage {
    fn root(&self) -> Option<&SemId> {
        self.root.as_ref()
    }

    fn project(&self, b: &mut SnapshotBuilder, root: &SemId) {
        for (g, group) in self.groups.iter().enumerate() {
            let Some(at) = group_id(root, g) else {
                continue;
            };
            b.node(&at, Role::Group, group.title).parent(root);
            for card in &group.cards {
                card.spec.project(b, &at);
            }
        }
        for spec in self.specs() {
            let part = match spec.part_focus() {
                Some(part) => Some(part),
                None if spec.text().is_some_and(|t| t.is_focused()) => Some(Part::Box),
                None => None,
            };
            if let Some(id) = part.and_then(|p| spec.part_id(p)) {
                b.focus(&id);
            }
        }
    }

    /// A box's `set_value` puts its text, as typing it would; the eye's
    /// `toggle` reveals or hides the password, as a click on it does; a
    /// link's or an icon button's `activate` does nothing more than a
    /// click.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        let refused = || format!("the page has no {} on {}", req.action, req.target);
        let (at, part) = self.at(&req.target).ok_or_else(refused)?;
        let spec = self.spec_mut(at).ok_or_else(refused)?;
        if spec.run(part, req.action.as_str(), req.param.as_ref()) {
            Ok(After::Nothing)
        } else {
            Err(refused())
        }
    }

    /// A field takes the page's focus (its caret); its eye or link takes
    /// the page's focus ring; another node of the page takes it from all.
    fn focus(&mut self, id: &SemId) -> bool {
        if !self.root.as_ref().is_some_and(|root| under(root, id)) {
            return false;
        }
        let hit = self.at(id);
        self.blur_all();
        match hit {
            Some((at, Part::Box)) => self.set_focused(Some(at)),
            Some((at, part)) => {
                if let Some(spec) = self.spec_mut(at) {
                    spec.set_part_focus(Some(part));
                }
            }
            None => {}
        }
        true
    }

    fn blur(&mut self) -> bool {
        self.blur_all()
    }
}
