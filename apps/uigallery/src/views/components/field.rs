//! Field page, the hoff core kit's InputsPage for its Field: groups with a
//! title and a bottom rule, cards in rows, one state per card behind a
//! chip that names it (the kit's chips for Email and Password; Disabled
//! and Content are the plev's: a disabled field, and the content a field
//! must survive, a long value, a long label and message, a right-to-left
//! value). The widget is `engine::widgets::form::field::Field`; every
//! visual value comes from the theme, and the page's own numbers are the
//! kit page's (`h-50` cards, `w-60` fields).
//!
//! The page owns the keyboard focus as the Input page does: a click in a
//! field places the caret, Tab reaches the fields in the projection's
//! order, Escape blurs; the characters, editing keys, clipboard chords
//! and the input method the shell forwards go to the focused field.
//!
//! Projected (task-75 phase 2b): a group per title,
//! `field_page.group[<g>]`, and in it each card's field,
//! `field_page.group[<g>].card[<c>]`, as the comps projects it (named by
//! its label, described by its message, secret when a password), with
//! its eye (`….reveal`) and its link (`….link`) beside it. `set_value`
//! puts a field's text as typing would, `toggle` reveals or hides a
//! password, and the link's `activate` does nothing more than a click.

use accord::{Request, Role, SemId, SnapshotBuilder, Value};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;
use engine::widgets::form::field::{Field, FieldPart};
use engine::widgets::prelude::{EventResult, Rect, Wake, WidgetEvent};

use super::showcase;
use super::{After, Component, EditKey, child_id, root_id, under};
use crate::views::ImeEvent;
use crate::views::text_input::clipboard::Keys;

/// A field is the kit page's `w-60` wide, narrower when the card is.
const FIELD_W: f32 = 240.0;

struct Card {
    /// The chip: which states this card shows.
    title: &'static str,
    field: Field,
}

struct Group {
    title: &'static str,
    /// Cards per row at full width.
    columns: usize,
    cards: Vec<Card>,
}

/// One group laid out: its title's y, each card with its field's rect,
/// and the y of its rule.
struct Laid {
    title_y: f32,
    cards: Vec<(Rect, Rect)>,
    rule_y: f32,
}

pub struct FieldPage {
    root: Option<SemId>,
    groups: Vec<Group>,
}

/// `field_page.group[<g>]`.
fn group_id(root: &SemId, g: usize) -> Option<SemId> {
    child_id(Some(root), "group").map(|family| family.key(g.to_string()))
}

/// `field_page.group[<g>].card[<c>]`.
fn card_id(root: &SemId, g: usize, c: usize) -> Option<SemId> {
    child_id(group_id(root, g).as_ref(), "card").map(|family| family.key(c.to_string()))
}

fn card(title: &'static str, field: Field) -> Card {
    Card { title, field }
}

/// The live card's chip, as the kit names it.
const LIVE: &str = "Default, Click on, Focus, Hover, Typing";

fn email() -> Field {
    Field::new("Enter email").label("Email")
}

fn password() -> Field {
    Field::new("Enter password")
        .label("Password")
        .link("Forgot password?")
        .password(true)
}

impl FieldPage {
    pub fn new() -> Self {
        let mut groups = vec![
            Group {
                title: "Email",
                columns: 3,
                cards: vec![
                    card(LIVE, email()),
                    card(
                        "Typed",
                        email().with_text("sophie@hoff.dev").validated(true),
                    ),
                    card(
                        "Message, With icon",
                        email().icon("scan").error("This email is already taken."),
                    ),
                ],
            },
            Group {
                title: "Password",
                columns: 3,
                cards: vec![
                    card(LIVE, password()),
                    card(
                        "Typed",
                        password().with_text("correct horse").validated(true),
                    ),
                    card("Message", password().error("Confirm password is incorrect")),
                ],
            },
            Group {
                title: "Disabled",
                columns: 2,
                cards: vec![
                    card("Disabled", email().disabled(true)),
                    card(
                        "Disabled, Typed",
                        email().with_text("sophie@hoff.dev").disabled(true),
                    ),
                ],
            },
            Group {
                title: "Content",
                columns: 3,
                cards: vec![
                    card(
                        "Long value",
                        email().with_text(
                            "sophie.marguerite.delacroix-oyelaran@research.hoff.example",
                        ),
                    ),
                    card(
                        "Long label, Long message",
                        Field::new("Geben Sie Ihre geschäftliche E-Mail-Adresse ein")
                            .label("Geschäftliche E-Mail-Adresse für Rechnungen und Belege")
                            .helper(
                                "An diese Adresse senden wir Rechnungen, Belege und \
                                 Hinweise zu Ihrem Arbeitsbereich.",
                            ),
                    ),
                    card(
                        "Right to left",
                        Field::new("הזינו כתובת דוא״ל")
                            .label("דוא״ל")
                            .with_text("שלום עולם"),
                    ),
                ],
            },
        ];
        let root = root_id("field_page");
        if let Some(root) = &root {
            for (g, group) in groups.iter_mut().enumerate() {
                for (c, card) in std::mem::take(&mut group.cards).into_iter().enumerate() {
                    let field = match card_id(root, g, c) {
                        Some(id) => card.field.sem(id),
                        None => card.field,
                    };
                    group.cards.push(Card { field, ..card });
                }
            }
        }
        Self { root, groups }
    }

    fn fields(&self) -> impl Iterator<Item = &Field> {
        self.groups
            .iter()
            .flat_map(|g| g.cards.iter().map(|c| &c.field))
    }

    fn fields_mut(&mut self) -> impl Iterator<Item = &mut Field> {
        self.groups
            .iter_mut()
            .flat_map(|g| g.cards.iter_mut().map(|c| &mut c.field))
    }

    fn field_mut(&mut self, (g, c): (usize, usize)) -> Option<&mut Field> {
        Some(&mut self.groups.get_mut(g)?.cards.get_mut(c)?.field)
    }

    /// The card whose field is `id`, or whose eye or link is.
    fn at(&self, id: &SemId) -> Option<((usize, usize), FieldPart)> {
        self.groups.iter().enumerate().find_map(|(g, group)| {
            group.cards.iter().enumerate().find_map(|(c, card)| {
                let f = &card.field;
                let part = if f.sem_id() == Some(id) {
                    FieldPart::Input
                } else if f.reveal_id().as_ref() == Some(id) {
                    FieldPart::Reveal
                } else if f.link_id().as_ref() == Some(id) {
                    FieldPart::Link
                } else {
                    return None;
                };
                Some(((g, c), part))
            })
        })
    }

    /// The card whose field has the caret.
    pub fn focused(&self) -> Option<(usize, usize)> {
        self.groups.iter().enumerate().find_map(|(g, group)| {
            let c = group
                .cards
                .iter()
                .position(|c| c.field.input.is_focused())?;
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
                card.field.input.set_focused(at == Some((g, c)));
            }
        }
    }

    /// Per group: the title's y, the cards with their fields, the rule's
    /// y, on the showcase's grid (`showcase::layout`: columns, title,
    /// padding), but a row is as tall as its tallest field needs (at least
    /// `h-50`), and a field is centered in its card, clear of the chip.
    fn layout(&self, content: Rect, theme: &Theme) -> Vec<Laid> {
        let gap = theme.spacing.sm;
        let band = vitrine::chip_band(theme);
        let side = theme.spacing.lg;
        let mut y = content.y;
        let mut out = Vec::with_capacity(self.groups.len());
        for group in &self.groups {
            let frame = Rect::new(content.x, y, content.w, content.h);
            let spans = [(group.columns, group.cards.len())];
            let Some(grid) = showcase::layout(spans, frame, theme).pop() else {
                continue;
            };
            let first = grid.cards.first().map_or(grid.title_y, |r| r.y);
            let cols = grid
                .cards
                .iter()
                .take_while(|r| r.y == first)
                .count()
                .max(1);
            // The showcase's padding under the last row.
            let pad = grid.rule_y - grid.cards.last().map_or(first, |r| r.bottom());
            let mut row_y = first;
            let mut cards = Vec::with_capacity(group.cards.len());
            for (row, frames) in group.cards.chunks(cols).zip(grid.cards.chunks(cols)) {
                let field_w = FIELD_W.min(frames[0].w - side * 2.0).max(0.0);
                let heights: Vec<f32> = row
                    .iter()
                    .map(|card| card.field.height_for(field_w, theme))
                    .collect();
                let tallest = heights.iter().copied().fold(0.0, f32::max);
                let card_h = showcase::CARD_H.max(band * 2.0 + tallest);
                for (h, frame) in heights.into_iter().zip(frames) {
                    let card = Rect::new(frame.x, row_y, frame.w, card_h);
                    cards.push((card, showcase::centered((field_w, h), card)));
                }
                row_y += card_h + gap;
            }
            let rule_y = row_y - gap + pad;
            out.push(Laid {
                title_y: grid.title_y,
                cards,
                rule_y,
            });
            y = rule_y;
        }
        out
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        self.layout(content, theme)
            .last()
            .map_or(0.0, |laid| laid.rule_y - content.y)
    }

    /// The field of `at`, laid out in `content`.
    fn field_rect(&self, at: (usize, usize), content: Rect, theme: &Theme) -> Option<Rect> {
        Some(self.layout(content, theme).get(at.0)?.cards.get(at.1)?.1)
    }

    /// Every field takes the pointer at its rect: hover, a press that
    /// focuses one and blurs the others, the eye, the link.
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
                result = result.merge(card.field.handle_event(event, *rect, theme));
            }
        }
        result
    }

    /// Type `s` into the focused field. `false` when none is focused.
    pub fn handle_text(&mut self, s: &str) -> bool {
        self.focused_field().is_some_and(|f| f.input.insert(s))
    }

    /// The editing keys edit the focused field. Tab is the view's (it
    /// walks the projection), never an edit.
    pub fn handle_edit_key(&mut self, key: EditKey) -> bool {
        if key == EditKey::Tab {
            return false;
        }
        self.focused_field()
            .is_some_and(|f| f.input.edit(key.into()))
    }

    /// A step of the input method: the focused field composes;
    /// `Disabled` drops a composition wherever one is.
    pub fn handle_ime(&mut self, ime: ImeEvent<'_>) -> bool {
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
    /// `false` off the boxes.
    pub fn right_click(&mut self, x: f32, y: f32, content: Rect, theme: &Theme) -> bool {
        let layout = self.layout(content, theme);
        let hit = layout.iter().enumerate().find_map(|(g, laid)| {
            laid.cards.iter().enumerate().find_map(|(c, (_, rect))| {
                let field = &self.groups[g].cards[c].field;
                (field.part_at(x, y, *rect, theme) == Some(FieldPart::Input))
                    .then_some(((g, c), *rect))
            })
        });
        let Some((at, rect)) = hit else {
            return false;
        };
        if self.focused() == Some(at) && self.field_mut(at).is_some_and(|f| f.input.has_selection())
        {
            return true;
        }
        self.set_focused(Some(at));
        if let Some(field) = self.field_mut(at) {
            field.handle_event(&WidgetEvent::MouseDown { x, y }, rect, theme);
        }
        true
    }

    /// The focused field has the keys (a chord, the field's menu).
    pub(in crate::views) fn keys(&mut self) -> Keys<'_> {
        match self.focused_field() {
            Some(f) => Keys::Field(&mut f.input),
            None => Keys::Nobody,
        }
    }

    /// Where the input method's candidate window goes: the focused
    /// field's caret, laid out in `content`.
    pub fn ime_area(&self, content: Rect, theme: &Theme) -> Option<Rect> {
        let at = self.focused()?;
        let rect = self.field_rect(at, content, theme)?;
        let field = &self.groups[at.0].cards[at.1].field;
        Some(field.input.ime_area(field.layout(rect, theme).input, theme))
    }

    /// Escape blurs the focused field. `false` when none was.
    /// Escape blurs the focused field, or lets go of the eye or the link
    /// a reader focused. `false` when nothing was.
    pub fn handle_escape(&mut self) -> bool {
        let had = self.focused().is_some() || self.fields().any(|f| f.part_focus().is_some());
        self.set_focused(None);
        for field in self.fields_mut() {
            field.set_part_focus(None);
        }
        had
    }

    /// The caret blink of the focused field; nothing while none is.
    pub fn wake(&mut self, dt: f32) -> Wake {
        self.focused_field()
            .map_or(Wake::idle(), |f| f.input.wake(dt))
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let layout = self.layout(content, theme);
        let last = self.groups.len() - 1;
        for (i, (group, laid)) in self.groups.iter().zip(&layout).enumerate() {
            showcase::title(c, group.title, content, laid.title_y, theme);
            for (card, (rect, field)) in group.cards.iter().zip(&laid.cards) {
                let (_, chip) = vitrine::chip(*rect, card.title, theme);
                showcase::card(c, *rect, &chip, theme);
                card.field.render(c, *field, theme);
            }
            // The rule under the group (`border-b`, none on the last).
            if i != last {
                showcase::rule(c, content, laid.rule_y, theme);
            }
        }
    }
}

impl Component for FieldPage {
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
                if let Some(id) = card.field.sem_id() {
                    card.field.project(b, id, Some(&at));
                }
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
        let refused = || format!("the field page has no {} on {}", req.action, req.target);
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

    /// A field takes the page's focus (its caret); another node of the
    /// page takes it from the fields.
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

/// What the showcase does not give yet, kept to the minimum until it
/// does: a chip cut with an ellipsis to its card, and the band the chip
/// takes, which a card taller than `h-50` keeps clear. The geometry is the
/// showcase's chip (`showcase::card`).
mod vitrine {
    use engine::drawing::text::{TextMeasurer, TextStyle};
    use engine::styling::theme::Theme;
    use engine::widgets::prelude::Rect;

    fn chip_style(theme: &Theme) -> TextStyle {
        theme.typography.exact(theme.typography.caption, 16.0, 500)
    }

    /// The chip naming a card's states, in its top-left corner, and its
    /// text: cut with an ellipsis where the card is too narrow for it.
    pub(super) fn chip(card: Rect, title: &str, theme: &Theme) -> (Rect, String) {
        let style = chip_style(theme);
        let (inset, pad) = (theme.spacing.sm, theme.spacing.md);
        let text = TextMeasurer::truncate_to_width(title, &style, card.w - (inset + pad) * 2.0);
        let (tw, _) = TextMeasurer::measure_styled(&text, &style, None);
        let rect = Rect::new(
            card.x + inset,
            card.y + inset,
            tw + pad * 2.0,
            style.line_height + theme.spacing.xs * 2.0,
        );
        (rect, text)
    }

    /// The band the chip takes at the top of a card, and keeps free at
    /// its bottom so a centered field clears it.
    pub(super) fn chip_band(theme: &Theme) -> f32 {
        let style = chip_style(theme);
        theme.spacing.sm * 2.0 + style.line_height + theme.spacing.xs * 2.0
    }
}

#[cfg(test)]
mod tests;
