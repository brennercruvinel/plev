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

use accord::{Request, Role, SemId, SnapshotBuilder, Value};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;
use engine::widgets::form::field::{Field, FieldPart};
use engine::widgets::prelude::{EventResult, Rect, Wake, WidgetEvent};

use super::super::showcase;
use super::super::{After, Component, EditKey, child_id, root_id, under};
use super::spec::Spec;
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

    pub(crate) fn fields(&self) -> impl Iterator<Item = &Field> {
        self.groups
            .iter()
            .flat_map(|g| g.cards.iter().filter_map(|c| c.spec.field()))
    }

    fn fields_mut(&mut self) -> impl Iterator<Item = &mut Field> {
        self.groups
            .iter_mut()
            .flat_map(|g| g.cards.iter_mut().filter_map(|c| c.spec.field_mut()))
    }

    pub(crate) fn field(&self, (g, c): (usize, usize)) -> Option<&Field> {
        self.groups.get(g)?.cards.get(c)?.spec.field()
    }

    fn field_mut(&mut self, (g, c): (usize, usize)) -> Option<&mut Field> {
        self.groups.get_mut(g)?.cards.get_mut(c)?.spec.field_mut()
    }

    /// The card whose widget, or the eye or link of whose field, is `id`.
    fn at(&self, id: &SemId) -> Option<((usize, usize), FieldPart)> {
        self.groups.iter().enumerate().find_map(|(g, group)| {
            group
                .cards
                .iter()
                .enumerate()
                .find_map(|(c, card)| Some(((g, c), card.spec.part(id)?)))
        })
    }

    /// The card whose field has the caret.
    pub fn focused(&self) -> Option<(usize, usize)> {
        self.groups.iter().enumerate().find_map(|(g, group)| {
            let c = group
                .cards
                .iter()
                .position(|c| c.spec.field().is_some_and(|f| f.input.is_focused()))?;
            Some((g, c))
        })
    }

    fn focused_field(&mut self) -> Option<&mut Field> {
        let at = self.focused()?;
        self.field_mut(at)
    }

    /// The caret goes to the field of `at` (one not disabled), or to none.
    pub fn set_focused(&mut self, at: Option<(usize, usize)>) {
        for (g, group) in self.groups.iter_mut().enumerate() {
            for (c, card) in group.cards.iter_mut().enumerate() {
                if let Some(f) = card.spec.field_mut() {
                    f.input.set_focused(at == Some((g, c)));
                }
            }
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
                    .map(|c| c.spec.size(card_w, side, theme))
                    .collect()
            })
            .collect();
        let sized = self.groups.iter().zip(&sizes).map(|(g, s)| {
            let h = s.iter().map(|(_, h)| showcase::card_height(*h, theme));
            (g.columns, h.collect())
        });
        showcase::layout_sized(sized, content, theme)
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
        if matches!(event, WidgetEvent::MouseDown { .. })
            && self.fields().any(|f| f.part_focus().is_some())
        {
            for field in self.fields_mut() {
                field.set_part_focus(None);
            }
            result = EventResult::changed();
        }
        let layout = self.layout(content, theme);
        for (group, laid) in self.groups.iter_mut().zip(&layout) {
            for (card, (_, rect)) in group.cards.iter_mut().zip(&laid.cards) {
                result = result.merge(card.spec.handle_event(event, *rect, theme));
            }
        }
        result
    }

    /// The caret blink of the focused field; nothing while none is.
    pub fn wake(&mut self, dt: f32) -> Wake {
        self.focused_field()
            .map_or(Wake::idle(), |f| f.input.wake(dt))
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let layout = self.layout(content, theme);
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
        self.focused_field().is_some_and(|f| f.input.insert(s))
    }

    fn handle_edit_key(&mut self, key: EditKey) -> bool {
        if key == EditKey::Tab {
            return false;
        }
        self.focused_field()
            .is_some_and(|f| f.input.edit(key.into()))
    }

    fn handle_ime(&mut self, ime: ImeEvent<'_>) -> bool {
        let (committed, preedit, cursor) = match ime {
            ImeEvent::Disabled => {
                let mut dropped = false;
                for field in self.fields_mut() {
                    dropped |= field.input.end_compose();
                }
                return dropped;
            }
            ImeEvent::Preedit(text, cursor) => ("", text, cursor),
            ImeEvent::Commit(text) => (text, "", None),
        };
        self.focused_field()
            .is_some_and(|f| f.input.compose(committed, preedit, cursor))
    }

    /// A right-click on a field's box: it takes the keys for its menu,
    /// keeping a selection it has, or taking a caret at the click.
    fn right_click(&mut self, x: f32, y: f32, content: Rect, theme: &Theme) -> bool {
        let layout = self.layout(content, theme);
        let hit = layout.iter().enumerate().find_map(|(g, laid)| {
            laid.cards.iter().enumerate().find_map(|(c, (_, rect))| {
                let field = self.field((g, c))?;
                (field.part_at(x, y, *rect, theme) == Some(FieldPart::Input))
                    .then_some(((g, c), *rect))
            })
        });
        let Some((at, rect)) = hit else {
            return false;
        };
        if self.focused() == Some(at) && self.field(at).is_some_and(|f| f.input.has_selection()) {
            return true;
        }
        self.set_focused(Some(at));
        if let Some(field) = self.field_mut(at) {
            field.handle_event(&WidgetEvent::MouseDown { x, y }, rect, theme);
        }
        true
    }

    fn keys(&mut self) -> Keys<'_> {
        match self.focused_field() {
            Some(f) => Keys::Field(&mut f.input),
            None => Keys::Nobody,
        }
    }

    fn ime_area(&self, content: Rect, theme: &Theme) -> Option<Rect> {
        let at = self.focused()?;
        let rect = self.widget_rect(at, content, theme)?;
        let field = self.field(at)?;
        Some(field.input.ime_area(field.layout(rect, theme).input, theme))
    }

    /// Escape blurs the focused field, or lets go of the eye or the link
    /// a reader focused. `false` when nothing was.
    fn handle_escape(&mut self) -> bool {
        let had = self.focused().is_some() || self.fields().any(|f| f.part_focus().is_some());
        self.set_focused(None);
        for field in self.fields_mut() {
            field.set_part_focus(None);
        }
        had
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
        for field in self.fields() {
            let id = match field.part_focus() {
                Some(FieldPart::Reveal) => field.reveal_id(),
                Some(FieldPart::Link) => field.link_id(),
                _ if field.input.is_focused() => field.sem_id().cloned(),
                _ => None,
            };
            if let Some(id) = id {
                b.focus(&id);
            }
        }
    }

    /// A field's `set_value` puts its text, as typing it would; the eye's
    /// `toggle` reveals or hides the password, as a click on it does; the
    /// link's `activate` does nothing more than a click.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        let refused = || format!("the page has no {} on {}", req.action, req.target);
        let (at, part) = self.at(&req.target).ok_or_else(refused)?;
        let field = self.field_mut(at).ok_or_else(refused)?;
        match (part, req.action.as_str(), &req.param) {
            (FieldPart::Input, "set_value", Some(Value::Text(text))) => {
                field.input.set_text(text);
                Ok(After::Nothing)
            }
            (FieldPart::Reveal, "toggle", None) => {
                if field.toggle_reveal() {
                    Ok(After::Nothing)
                } else {
                    Err(refused())
                }
            }
            (FieldPart::Link, "activate", None) => Ok(After::Nothing),
            _ => Err(refused()),
        }
    }

    /// A field takes the page's focus (its caret); its eye or link takes
    /// the page's focus ring; another node of the page takes it from all.
    fn focus(&mut self, id: &SemId) -> bool {
        if !self.root.as_ref().is_some_and(|root| under(root, id)) {
            return false;
        }
        let hit = self.at(id);
        self.handle_escape();
        match hit {
            Some((at, FieldPart::Input)) => self.set_focused(Some(at)),
            Some((at, part)) => {
                if let Some(field) = self.field_mut(at) {
                    field.set_part_focus(Some(part));
                }
            }
            None => {}
        }
        true
    }

    fn blur(&mut self) -> bool {
        self.handle_escape()
    }
}
