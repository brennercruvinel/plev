//! Input page: three single-line [`engine::widgets::form::TextField`]s,
//! the kit's input box (`components/Field/index.tsx:66`) under its label
//! (`:36`, the [`Label`]), as the kit's showcase draws a component (a
//! group with a title and a rule, cards behind a chip naming what each
//! shows: the states, the clipboard, the input method). The page owns
//! the keyboard focus: a click in a field places the caret, Tab reaches the fields (the view's keyboard,
//! in the order of the projection), Escape blurs; the characters and
//! editing keys the shell forwards go to the focused one.
//!
//! Projected (task-75 phase 2b): each field, `input_page.field[<i>]`, as the
//! comps projects it, named by its label; `set_value` puts its text as
//! typing would. The focus a reader asks for is the page's own (the
//! caret).

use accord::{Request, SemId, SnapshotBuilder, Value};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;
use engine::widgets::form::TextField;
use engine::widgets::form::label::Label;
use engine::widgets::prelude::{EventResult, Rect, Wake, WidgetEvent};

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
}

impl InputPage {
    pub fn new() -> Self {
        let root = root_id("input_page");
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
        }
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
        self.grid(content, theme).rule_y - content.y
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
        let mut r = EventResult::IGNORED;
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
            None => Wake::idle(),
        }
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let grid = self.grid(content, theme);
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
            None => false,
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
            None => false,
        }
    }

    /// A step of the input method: the focused field composes, and a
    /// commit is typed into it as `handle_text` types; `Disabled` drops a
    /// composition wherever one is. `false` when nothing changed.
    fn handle_ime(&mut self, ime: ImeEvent<'_>) -> bool {
        let (committed, preedit, cursor) = match ime {
            ImeEvent::Disabled => {
                let mut dropped = false;
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
            None => false,
        }
    }

    /// A right-click: the field under the pointer takes the keys for its
    /// menu. One focused with a selection keeps it (the person selected
    /// what to copy); another takes a caret at the click, as a press
    /// there. `false` off the fields.
    fn right_click(&mut self, x: f32, y: f32, content: Rect, theme: &Theme) -> bool {
        let rects = self.rects(content, theme);
        let Some(i) = rects.iter().position(|r| r.contains(x, y)) else {
            return false;
        };
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
            None => Keys::Nobody,
        }
    }

    /// Where the input method's candidate window goes: the focused
    /// field's caret, laid out in `content`. `None` with no field focused.
    fn ime_area(&self, content: Rect, theme: &Theme) -> Option<Rect> {
        let i = self.focused()?;
        Some(self.fields[i].ime_area(self.rects(content, theme)[i], theme))
    }

    /// Escape blurs the focused field. `false` when none was.
    fn handle_escape(&mut self) -> bool {
        if self.focused().is_some() {
            self.set_focused(None);
            true
        } else {
            false
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
        self.set_focused(self.at(id));
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine::drawing::compositor::SceneNode;
    use engine::drawing::text::TextMeasurer;

    fn content() -> Rect {
        Rect::new(288.0, 80.0, 760.0, 700.0)
    }

    fn texts(page: &InputPage) -> Vec<String> {
        let theme = Theme::hoff();
        let mut c = Compositor::new();
        c.begin_frame();
        page.render(&mut c, content(), &theme);
        c.layer(engine::drawing::compositor::LayerId::DEFAULT)
            .unwrap()
            .nodes()
            .iter()
            .filter_map(|n| match n {
                SceneNode::Text { key, .. } => Some(key.text.clone()),
                _ => None,
            })
            .collect()
    }

    /// A field takes the focus the view's Tab asks for (the projection's
    /// order, `dispatch/keyboard.rs`), and Tab itself is no edit; Escape
    /// blurs.
    #[test]
    fn a_field_takes_the_focus_tab_asks_for_and_escape_blurs() {
        let mut p = InputPage::new();
        assert_eq!(p.focused(), None);
        assert!(!p.handle_edit_key(EditKey::Tab), "Tab is the view's");
        let field = p.fields[2].sem_id().cloned().expect("an id");
        assert!(p.focus(&field));
        assert_eq!(p.focused(), Some(2));
        assert!(p.handle_escape());
        assert_eq!(p.focused(), None);
        assert!(!p.handle_escape());
    }

    /// A click inside a field focuses it and places the caret on the
    /// clicked glyph (real shaping via TextMeasurer, not chars * factor).
    #[test]
    fn click_focuses_field_and_places_caret() {
        let theme = Theme::hoff();
        let mut p = InputPage::new();
        let field = p.rects(content(), &theme)[1];

        let (cx, cy) = field.center();
        let r = p.handle_event(&WidgetEvent::MouseDown { x: cx, y: cy }, content(), &theme);
        assert!(r.changed);
        assert_eq!(p.focused(), Some(1));

        assert!(p.handle_text("Brenner"));
        assert_eq!(p.value(1), "Brenner");
        let style = TextField::text_style(&theme);
        let x =
            field.x + theme.spacing.md + TextMeasurer::cursor_x_styled("Brenner", &style, None, 4);
        p.handle_event(&WidgetEvent::MouseDown { x, y: cy }, content(), &theme);
        assert_eq!(p.cursor(1), 4);

        // A click outside every field blurs but is not swallowed.
        let r = p.handle_event(
            &WidgetEvent::MouseDown {
                x: content().x + 1.0,
                y: content().y + content().h - 1.0,
            },
            content(),
            &theme,
        );
        assert_eq!(p.focused(), None);
        assert!(r.changed);
    }

    /// Characters only reach a field while one is focused, and editing
    /// keys edit the focused buffer.
    #[test]
    fn typing_requires_focus_and_editing_keys_edit() {
        let mut p = InputPage::new();
        assert!(!p.handle_text("x"), "no focus: the shell keeps the key");
        p.set_focused(Some(0));
        p.handle_text("abc");
        p.handle_edit_key(EditKey::Left);
        p.handle_edit_key(EditKey::Backspace);
        assert_eq!(p.value(0), "ac");
        p.handle_edit_key(EditKey::End);
        p.handle_edit_key(EditKey::Delete);
        assert_eq!(p.value(0), "ac");
        p.handle_escape();
        assert!(
            !p.handle_edit_key(EditKey::Backspace),
            "blurred: not consumed"
        );
    }

    /// Each field draws under its label, the placeholder an example
    /// while empty, the typed value once typed; nothing else speaks for
    /// the value.
    #[test]
    fn a_field_draws_under_its_label_and_shows_what_is_typed() {
        let theme = Theme::hoff_core();
        let mut p = InputPage::new();
        let before = texts(&p);
        for (label, placeholder) in LABELS.iter().zip(PLACEHOLDERS) {
            assert!(before.iter().any(|t| t == label), "{label}");
            assert!(before.iter().any(|t| t == placeholder), "{placeholder}");
        }
        for (label, field) in p.parts(content(), &theme) {
            assert!(label.bottom() < field.y, "{label:?} over {field:?}");
            assert_eq!(label.x, field.x, "the label starts where its box does");
        }
        p.set_focused(Some(0));
        p.handle_text("Ada");
        let after = texts(&p);
        assert!(after.iter().any(|t| t == "Ada"), "typed value rendered");
        assert!(!after.iter().any(|t| t.contains("live value")));
    }

    /// The accent focus ring sits 4px (offset + stroke) outside the
    /// focused field, and nowhere when nothing is focused.
    #[test]
    fn focus_ring_tracks_the_focused_field() {
        let theme = Theme::hoff();
        let mut p = InputPage::new();
        let rings = |p: &InputPage| -> Vec<(f32, f32)> {
            let accent = theme.colors.accent.0;
            let mut c = Compositor::new();
            c.begin_frame();
            p.render(&mut c, content(), &theme);
            c.layer(engine::drawing::compositor::LayerId::DEFAULT)
                .unwrap()
                .nodes()
                .iter()
                .filter_map(|n| match *n {
                    SceneNode::RoundedRect {
                        x,
                        y,
                        color,
                        border_width,
                        border_color,
                        ..
                    } if color[3] == 0.0 && border_width == 2.0 && border_color == accent => {
                        Some((x, y))
                    }
                    _ => None,
                })
                .collect()
        };
        assert!(rings(&p).is_empty(), "no focus, no ring");
        p.set_focused(Some(0));
        let f = p.rects(content(), &theme)[0];
        assert_eq!(rings(&p), vec![(f.x - 4.0, f.y - 4.0)]);
    }

    /// A value wider than its field stays on one line inside it, focused
    /// (clipped to the text's box, slid to the caret) or blurred (cut
    /// with an ellipsis). It used to be laid out in the box's width and
    /// wrap, a second line drawn under the field.
    #[test]
    fn a_long_value_stays_on_one_line_inside_its_field() {
        let long = "a value far wider than the field it was typed into, ".repeat(4);
        for theme in [Theme::hoff_core(), Theme::hoff()] {
            let style = TextField::text_style(&theme);
            let field = InputPage::new().rects(content(), &theme)[0];
            for focused in [true, false] {
                let mut p = InputPage::new();
                p.set_focused(Some(0));
                assert!(p.handle_text(&long));
                if !focused {
                    p.set_focused(None);
                }
                let mut c = Compositor::new();
                c.begin_frame();
                p.render(&mut c, content(), &theme);
                let nodes = c
                    .layer(engine::drawing::compositor::LayerId::DEFAULT)
                    .unwrap()
                    .nodes()
                    .to_vec();
                let mut clip: Option<(f32, f32)> = None;
                let mut seen = 0;
                for n in &nodes {
                    match n {
                        SceneNode::PushClip { x, w, .. } => clip = Some((*x, *w)),
                        SceneNode::PopClip => clip = None,
                        SceneNode::Text { key, x, .. } if key.text.starts_with("a value") => {
                            seen += 1;
                            let width = key.max_width_bits.map(f32::from_bits);
                            let (w, h) = TextMeasurer::measure_styled(&key.text, &style, width);
                            assert!(h <= style.line_height + 0.5, "wrapped: {h}");
                            let (left, right) = clip.map_or((*x, *x + w), |(cx, cw)| (cx, cx + cw));
                            assert!(left >= field.x && right <= field.x + field.w, "outside");
                        }
                        _ => {}
                    }
                }
                assert_eq!(seen, 1, "the value draws once");
            }
        }
    }

    /// While a field is focused the page asks for the caret's next flip,
    /// not for a frame every tick; idle it asks for nothing.
    #[test]
    fn focused_field_asks_for_the_next_flip() {
        let mut p = InputPage::new();
        assert_eq!(p.wake(0.016), Wake::idle(), "idle page requests no frames");
        p.set_focused(Some(0));
        let w = p.wake(0.016);
        assert!(!w.now && !w.is_continuous(), "no flip yet, no frame: {w:?}");
        assert!(w.next.is_some(), "the next flip is pending");
    }
}
