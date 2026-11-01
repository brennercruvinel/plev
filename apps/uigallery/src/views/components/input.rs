//! Input page: the plev's Input group and, under it, the kit's
//! InputsPage (`templates/InputsPage/index.tsx`), its groups in its order
//! (`kit.rs`, a form page). The Input group is three single-line
//! [`engine::widgets::form::TextField`]s, the kit's input box
//! (`components/Field/index.tsx:66`) under its label (`:36`, the
//! [`Label`]), behind chips naming what each shows (the states, the
//! clipboard, the input method). The page owns the keyboard focus: a
//! click in a field places the caret, Tab reaches the fields (the view's
//! keyboard, in the order of the projection), Escape blurs; the
//! characters and editing keys the shell forwards go to the focused one,
//! in the kit's groups or in the Input group.
//!
//! Projected (task-75 phase 2b): each field of the Input group,
//! `input_page.field[<i>]`, as the comps projects it, named by its label,
//! then the kit's groups as a form page projects them
//! (`input_page.group[<g>].card[<c>]`), the order Tab walks; `set_value`
//! puts a field's text as typing would. The focus a reader
//! asks for is the page's own (the caret).

mod kit;

use accord::{Request, SemId, SnapshotBuilder, Value};
use engine::drawing::compositor::{Compositor, LayerId};
use engine::styling::theme::Theme;
use engine::widgets::form::TextField;
use engine::widgets::form::label::Label;
use engine::widgets::prelude::{EventResult, Rect, Wake, WidgetEvent};

use super::field::FormPage;
use super::{After, Component, EditKey, child_id, root_id, showcase, under};
use crate::views::ImeEvent;
use crate::views::text_input::clipboard::Keys;
use crate::views::text_input::host::TextHost;

/// Number of fields on the page.
pub const COUNT: usize = 3;

/// What each field asks for: its label and its name.
const LABELS: [&str; COUNT] = ["Full name", "Email address", "Project codename"];

/// An example of what each field takes, its placeholder.
const PLACEHOLDERS: [&str; COUNT] = ["Ada Lovelace", "ada@hoff.dev", "Nebula"];

/// What each field's card shows.
const CHIPS: [&str; COUNT] = [
    "Default, Click on, Focus, Hover, Typing",
    "Select, Copy, Cut, Paste",
    "Input method, Right to left",
];

/// The group's title.
const TITLE: &str = "Input";

pub struct InputPage {
    root: Option<SemId>,
    labels: [Label; COUNT],
    fields: [TextField; COUNT],
    /// The kit's groups, under the Input group.
    kit: FormPage,
}

impl InputPage {
    pub fn new() -> Self {
        let root = root_id(kit::ROOT);
        let family = child_id(root.as_ref(), "field");
        let mut i = 0;
        let fields = PLACEHOLDERS.map(|placeholder| {
            let field = TextField::new(placeholder);
            let id = family.as_ref().map(|f| f.key(i.to_string()));
            i += 1;
            match id {
                Some(id) => field.sem(id),
                None => field,
            }
        });
        Self {
            root,
            labels: LABELS.map(Label::new),
            fields,
            kit: kit::page(),
        }
    }

    /// Where the kit's groups go: under the Input group, from the y of
    /// its rule.
    fn kit_area(&self, content: Rect, theme: &Theme) -> Rect {
        let top = self.grid(content, theme).rule_y;
        Rect::new(content.x, top, content.w, (content.bottom() - top).max(0.0))
    }

    /// The field behind `id`.
    fn at(&self, id: &SemId) -> Option<usize> {
        self.fields.iter().position(|f| f.sem_id() == Some(id))
    }

    /// Each card's label and field, `w-60` wide at most, in a card
    /// `card_w` wide: the label's size and the field's.
    fn sizes(&self, card_w: f32, theme: &Theme) -> [((f32, f32), (f32, f32)); COUNT] {
        let w = theme
            .form
            .field
            .width
            .min((card_w - theme.spacing.lg * 2.0).max(0.0));
        let field = (w, TextField::height(theme));
        std::array::from_fn(|i| (self.labels[i].size(Some(w), theme), field))
    }

    /// The showcase's group: the title's y, the cards and the rule's y,
    /// each card as tall as its label over its field needs.
    fn grid(&self, content: Rect, theme: &Theme) -> showcase::GroupLayout {
        let gap = theme.form.field.label_gap;
        let card_w = showcase::card_width(COUNT, content, theme);
        let heights = self
            .sizes(card_w, theme)
            .map(|((_, lh), (_, fh))| showcase::card_height(lh + gap + fh, theme));
        let wanted = std::iter::once((COUNT, heights.to_vec()));
        let mut grid = showcase::layout_sized(wanted, content, theme);
        grid.pop().expect("one group")
    }

    /// Each card's label and field rects, the two centered as one block.
    fn parts(&self, content: Rect, theme: &Theme) -> [(Rect, Rect); COUNT] {
        let grid = self.grid(content, theme);
        let gap = theme.form.field.label_gap;
        std::array::from_fn(|i| {
            let card = grid.cards.get(i).copied().unwrap_or(content);
            let ((lw, lh), (fw, fh)) = self.sizes(card.w, theme)[i];
            let block = showcase::centered((fw, lh + gap + fh), card);
            let label = Rect::new(block.x, block.y, lw, lh);
            let field = Rect::new(block.x, block.y + lh + gap, fw, fh);
            (label, field)
        })
    }

    /// Each field's rect.
    pub fn rects(&self, content: Rect, theme: &Theme) -> [Rect; COUNT] {
        self.parts(content, theme).map(|(_, field)| field)
    }

    /// Bottom edge of the page: the last group's rule.
    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        let area = self.kit_area(content, theme);
        area.y - content.y + self.kit.content_height(area, theme)
    }

    pub fn focused(&self) -> Option<usize> {
        self.fields.iter().position(|f| f.is_focused())
    }

    /// Buffer contents, exposed for the tests.
    #[cfg(test)]
    pub fn value(&self, i: usize) -> &str {
        self.fields[i].text()
    }

    /// Caret position (byte index), exposed for the tests.
    #[cfg(test)]
    pub fn cursor(&self, i: usize) -> usize {
        self.fields[i].input.buffer.cursor()
    }

    /// Focus exactly `target` (or nothing), unfocusing the rest.
    pub fn set_focused(&mut self, target: Option<usize>) {
        for (i, field) in self.fields.iter_mut().enumerate() {
            field.set_focused(Some(i) == target);
        }
    }

    /// A press inside a field focuses it and places the caret; a press
    /// anywhere else blurs (and is not swallowed).
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
    ) -> EventResult {
        // What floats over the kit's groups (an open list) has the
        // pointer alone where it is.
        let (x, y) = event.pos();
        let area = self.kit_area(content, theme);
        let over = self.kit.over_overlay(x, y, area, theme);
        let kit = self.kit.handle_event(event, area, theme);
        if over {
            return kit;
        }
        let rects = self.rects(content, theme);
        if let WidgetEvent::MouseDown { x, y } = *event {
            if let Some(i) = rects.iter().position(|f| f.contains(x, y)) {
                self.set_focused(Some(i));
                self.fields[i].handle_event(event, rects[i], theme);
                return EventResult::changed();
            }
            if self.focused().is_some() {
                self.set_focused(None);
                return EventResult::changed();
            }
        }
        let mut r = kit;
        for (field, rect) in self.fields.iter_mut().zip(rects) {
            r = r.merge(field.handle_event(event, rect, theme));
        }
        r
    }

    /// Advance the cursor blink of the focused field: a frame on a flip,
    /// the next flip as the deadline; nothing while no field is focused.
    pub fn wake(&mut self, dt: f32) -> Wake {
        match self.focused() {
            Some(i) => self.fields[i].wake(dt),
            None => self.kit.wake(dt),
        }
    }

    /// Draws the page; what floats over the kit's groups (an open list)
    /// on `overlay`.
    pub fn render(&self, c: &mut Compositor, overlay: LayerId, content: Rect, theme: &Theme) {
        let grid = self.grid(content, theme);
        // The rule under the Input group: the kit's groups follow it.
        showcase::rule(c, content, grid.rule_y, theme);
        self.kit
            .render(c, overlay, self.kit_area(content, theme), theme);
        showcase::title(c, TITLE, content, grid.title_y, theme);
        let parts = self.parts(content, theme);
        let cards = grid.cards.iter().zip(CHIPS);
        let widgets = self.labels.iter().zip(&self.fields).zip(parts);
        for ((card, chip), ((label, field), (label_at, field_at))) in cards.zip(widgets) {
            showcase::card(c, *card, chip, theme);
            label.render(c, label_at, theme);
            field.render(c, field_at, theme);
        }
    }
}

impl TextHost for InputPage {
    /// Type `s` into the focused field. `false` when nothing is focused.
    fn handle_text(&mut self, s: &str) -> bool {
        match self.focused() {
            Some(i) => self.fields[i].insert(s),
            None => self.kit.handle_text(s),
        }
    }

    /// The editing keys edit the focused field. Tab is the view's (it
    /// walks the projection), never an edit.
    fn handle_edit_key(&mut self, key: EditKey) -> bool {
        if key == EditKey::Tab {
            return false;
        }
        match self.focused() {
            Some(i) => self.fields[i].edit(key.into()),
            None => self.kit.handle_edit_key(key),
        }
    }

    /// Enter and the arrows: the kit's widgets' (a select, a color
    /// picker); the Input group's fields leave them to the walk.
    fn handle_enter(&mut self) -> bool {
        self.focused().is_none() && self.kit.handle_enter()
    }

    fn handle_nav(&mut self, down: bool) -> bool {
        self.focused().is_none() && self.kit.handle_nav(down)
    }

    /// A step of the input method: the focused field composes, and a
    /// commit is typed into it as `handle_text` types; `Disabled` drops a
    /// composition wherever one is. `false` when nothing changed.
    fn handle_ime(&mut self, ime: ImeEvent<'_>) -> bool {
        let (committed, preedit, cursor) = match ime {
            ImeEvent::Disabled => {
                let mut dropped = self.kit.handle_ime(ImeEvent::Disabled);
                for field in &mut self.fields {
                    dropped |= field.end_compose();
                }
                return dropped;
            }
            ImeEvent::Preedit(text, cursor) => ("", text, cursor),
            ImeEvent::Commit(text) => (text, "", None),
        };
        match self.focused() {
            Some(i) => self.fields[i].compose(committed, preedit, cursor),
            None => self.kit.handle_ime(ime),
        }
    }

    /// A right-click: the field under the pointer takes the keys for its
    /// menu. One focused with a selection keeps it (the person selected
    /// what to copy); another takes a caret at the click, as a press
    /// there. `false` off the fields.
    fn right_click(&mut self, x: f32, y: f32, content: Rect, theme: &Theme) -> bool {
        let rects = self.rects(content, theme);
        let Some(i) = rects.iter().position(|r| r.contains(x, y)) else {
            let took = self
                .kit
                .right_click(x, y, self.kit_area(content, theme), theme);
            if took {
                self.set_focused(None);
            }
            return took;
        };
        self.kit.blur();
        if self.focused() == Some(i) && self.fields[i].has_selection() {
            return true;
        }
        self.set_focused(Some(i));
        self.fields[i].handle_event(&WidgetEvent::MouseDown { x, y }, rects[i], theme);
        true
    }

    /// The focused field has the keys (a chord, the field's menu).
    fn keys(&mut self) -> Keys<'_> {
        match self.focused() {
            Some(i) => Keys::Field(&mut self.fields[i]),
            None => self.kit.keys(),
        }
    }

    /// Where the input method's candidate window goes: the focused
    /// field's caret, laid out in `content`. `None` with no field focused.
    fn ime_area(&self, content: Rect, theme: &Theme) -> Option<Rect> {
        let Some(i) = self.focused() else {
            return self.kit.ime_area(self.kit_area(content, theme), theme);
        };
        Some(self.fields[i].ime_area(self.rects(content, theme)[i], theme))
    }

    /// Escape blurs the focused field. `false` when none was.
    fn handle_escape(&mut self) -> bool {
        if self.focused().is_some() {
            self.set_focused(None);
            true
        } else {
            self.kit.handle_escape()
        }
    }
}

impl Component for InputPage {
    fn root(&self) -> Option<&SemId> {
        self.root.as_ref()
    }

    fn project(&self, b: &mut SnapshotBuilder, root: &SemId) {
        for (field, label) in self.fields.iter().zip(LABELS) {
            if let Some(id) = field.sem_id() {
                field.project(b, id, label, Some(root));
            }
        }
        self.kit.project(b, root);
        let focused = self.focused().and_then(|i| self.fields.get(i));
        if let Some(id) = focused.and_then(TextField::sem_id) {
            b.focus(id);
        }
    }

    /// A field's `set_value` puts its text, as typing it would.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        match (self.at(&req.target), req.action.as_str(), &req.param) {
            (Some(i), "set_value", Some(Value::Text(text))) => {
                self.fields[i].set_text(text);
                Ok(After::Nothing)
            }
            (None, _, _) => self.kit.run(req),
            _ => Err(format!(
                "the input page has no {} on {}",
                req.action, req.target
            )),
        }
    }

    /// A field takes the page's focus (its caret); another node of the page
    /// takes it from the fields.
    fn focus(&mut self, id: &SemId) -> bool {
        if !self.root.as_ref().is_some_and(|root| under(root, id)) {
            return false;
        }
        let own = self.at(id);
        self.set_focused(own);
        if own.is_some() {
            self.kit.blur();
            true
        } else {
            self.kit.focus(id)
        }
    }

    fn blur(&mut self) -> bool {
        let had = self.focused().is_some();
        self.set_focused(None);
        self.kit.blur() || had
    }
}

#[cfg(test)]
mod tests;
