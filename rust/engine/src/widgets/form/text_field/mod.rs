//! Text field: a single-line input box drawn from the theme's form recipe
//! (`theme.form.input`: a surface at rest, hovered and focused, its
//! geometry, its text and the caret, selection and composition marks).
//! The core theme's recipe is the kit's Field; every other theme's is the
//! glass field it drew before. The editing itself lives in
//! [`TextInput`]; this widget adds the look, the pointer contract and the
//! focus ring every form control shares.
//!
//! One `TextStyle` (the recipe's `text`) measures the caret, hit tests
//! clicks and draws the text, so the caret always lands on the glyph that
//! was clicked.
//!
//! A value wider than the box is never wrapped: focused, it draws under a
//! clip of the text's box, slid so the caret stays in; blurred, it draws
//! cut with an ellipsis, as a placeholder too long for the box does
//! ([`LineFit`]). A masked field ([`TextField::masked`]) draws a bullet
//! per grapheme.
//!
//! An input method's composition ([`TextField::compose`]) draws inline, in
//! place of the selection or at the caret, underlined, with the caret at
//! the input method's; it is only drawn: the text, the projection and the
//! value hold what was committed.
//!
//! With a [`SemId`] ([`TextField::sem`]) the render binds the bounds it
//! draws and hit-tests with, and [`TextField::project`] declares the
//! field in a semantic projection; the app still owns the field.

use crate::drawing::text::{TextMeasurer, TextStyle};
use crate::runtime::platform::clipboard::ClipboardProvider;
use crate::styling::theme::Theme;
use accord::{Constraints, NodeBuilder, ParamSpec, Role, SemId, SnapshotBuilder, ValueKind};

use crate::widgets::form::text_input::view::LineFit;
use crate::widgets::form::text_input::{Accept, TextInput};
use crate::widgets::form::tokens::{InputRecipe, Look, Slot};
use crate::widgets::wake::Wake;
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

pub use super::text_input::keys::{ClipKey, EditKey};

pub(crate) mod render;

/// Retained single-line text field over a [`TextInput`].
#[derive(Clone, Debug)]
pub struct TextField {
    pub input: TextInput,
    pub disabled: bool,
    hovered: bool,
    sem: Option<SemId>,
    /// What takes the start and the end of the box (an icon, a button),
    /// which sets the text's inset there.
    slots: [Slot; 2],
    /// Which input box it draws (`theme.form.recipe`).
    look: Look,
}

impl TextField {
    pub fn new(placeholder: impl AsRef<str>) -> Self {
        Self {
            input: TextInput::new().with_placeholder(placeholder.as_ref()),
            disabled: false,
            hovered: false,
            sem: None,
            slots: [Slot::Pad; 2],
            look: Look::Field,
        }
    }

    /// Draw `look`'s input box instead of the Field's.
    pub fn look(mut self, look: Look) -> Self {
        self.look = look;
        self
    }

    pub fn look_of(&self) -> Look {
        self.look
    }

    fn recipe<'t>(&self, theme: &'t Theme) -> &'t InputRecipe {
        theme.form.recipe(self.look)
    }

    /// The style this field's text is measured, hit-tested and drawn
    /// with: its look's.
    pub fn style(&self, theme: &Theme) -> TextStyle {
        theme.typography.run(self.recipe(theme).text)
    }

    /// This field's height: its look's, never shorter than its line.
    pub fn box_height(&self, theme: &Theme) -> f32 {
        let r = self.recipe(theme);
        r.height.max(theme.typography.run(r.text).line_height)
    }

    /// Draw a bullet per grapheme in place of the text (a password); the
    /// value stays the text, and copy and cut give nothing.
    pub fn masked(mut self, masked: bool) -> Self {
        self.input.masked = masked;
        self
    }

    pub fn set_masked(&mut self, masked: bool) {
        self.input.masked = masked;
    }

    pub fn is_masked(&self) -> bool {
        self.input.masked
    }

    /// What takes the start and the end of the box: the text keeps clear
    /// of it (`theme.form` gives each slot's inset).
    pub fn slots(mut self, start: Slot, end: Slot) -> Self {
        self.slots = [start, end];
        self
    }

    pub fn set_slots(&mut self, start: Slot, end: Slot) {
        self.slots = [start, end];
    }

    /// The logical identity the render binds its bounds to
    /// (`Compositor::bind_to_layer`).
    pub fn sem(mut self, id: SemId) -> Self {
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    /// Declares this field as `id` in a projection: a `Role::TextInput`
    /// named `label`, its `text` state, editable, enabled unless disabled,
    /// and a `set_value` action that takes a text checked against the
    /// node's constraints and reads the `text` it replaces, so a value
    /// decided on a text another actor changed goes stale. It only reads
    /// the widget; the app owns it and runs `set_value`. Pass the id given
    /// to [`TextField::sem`] so the node and the binding name the same
    /// control. Returns the node for the app to add to (a `submit`, a
    /// `max_len`).
    #[track_caller]
    pub fn project<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        label: impl Into<String>,
        parent: Option<&SemId>,
    ) -> NodeBuilder<'b> {
        let mut node = b.node(id, Role::TextInput, label);
        if let Some(parent) = parent {
            node.parent(parent);
        }
        node.state("text", self.text())
            .constraints(Constraints {
                editable: true,
                ..Constraints::default()
            })
            .enabled(!self.disabled);
        node.action("set_value")
            .param(ParamSpec {
                kind: ValueKind::Text,
                constrained: true,
            })
            .reads(format!("{id}:text"));
        node
    }

    pub fn with_text(mut self, text: &str) -> Self {
        self.input.buffer.set_text(text);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn text(&self) -> &str {
        self.input.buffer.text()
    }

    pub fn set_text(&mut self, text: &str) {
        self.input.buffer.set_text(text);
    }

    /// What the field takes as typed, pasted or composed: the characters
    /// and how many ([`Accept`]).
    pub fn accept(mut self, accept: Accept) -> Self {
        self.input.accept = accept;
        self
    }

    /// The text set whole, as an agent's `set_value` or a reader's sets
    /// it: `false`, the text as it was, when the field does not take it
    /// as it is (a letter in a code, more than its cells).
    pub fn set_value(&mut self, text: &str) -> bool {
        let ok = self.input.accept.takes(text);
        if ok {
            self.set_text(text);
        }
        ok
    }

    pub fn is_empty(&self) -> bool {
        self.input.buffer.is_empty()
    }

    pub fn is_focused(&self) -> bool {
        self.input.focused
    }

    pub fn is_hovered(&self) -> bool {
        self.hovered
    }

    pub fn focus(&mut self) {
        if !self.disabled {
            self.input.focus();
        }
    }

    pub fn unfocus(&mut self) {
        self.input.unfocus();
    }

    /// Focus on `true`, blur on `false` (the owner's focus chain calls
    /// this; a field never steals focus on its own).
    pub fn set_focused(&mut self, focused: bool) {
        if focused {
            self.focus();
        } else {
            self.unfocus();
        }
    }

    /// The text style the field measures, hit-tests and draws with.
    pub fn text_style(theme: &Theme) -> TextStyle {
        theme.typography.run(theme.form.input.text)
    }

    /// Canonical field height: the recipe's, never shorter than its line
    /// of text.
    pub fn height(theme: &Theme) -> f32 {
        let input = &theme.form.input;
        input
            .height
            .max(theme.typography.run(input.text).line_height)
    }

    /// Type `s` (characters or a pasted string). `false` when unfocused.
    pub fn insert(&mut self, s: &str) -> bool {
        if !self.input.focused || self.disabled {
            return false;
        }
        for c in s.chars() {
            self.input.handle_char(c);
        }
        true
    }

    /// Route a non-character editing key. `false` when unfocused.
    pub fn edit(&mut self, key: EditKey) -> bool {
        if !self.input.focused || self.disabled {
            return false;
        }
        match key {
            EditKey::Backspace => self.input.handle_backspace(),
            EditKey::Delete => self.input.handle_delete(),
            EditKey::Left => self.input.handle_left(),
            EditKey::Right => self.input.handle_right(),
            EditKey::Home => self.input.handle_home(),
            EditKey::End => self.input.handle_end(),
            EditKey::SelectAll => self.input.handle_select_all(),
            EditKey::SelectLeft => self.input.handle_select_left(),
            EditKey::SelectRight => self.input.handle_select_right(),
            EditKey::SelectHome => self.input.handle_select_home(),
            EditKey::SelectEnd => self.input.handle_select_end(),
        }
        true
    }

    /// Copy the selection to `clipboard` (the person's keys or menu; a
    /// field gives an agent no way to it). `false` with nothing selected,
    /// unfocused or disabled.
    pub fn copy(&self, clipboard: &mut dyn ClipboardProvider) -> bool {
        !self.disabled && self.input.handle_copy(clipboard)
    }

    /// Copy the selection, then delete it. `false` with nothing selected,
    /// unfocused or disabled.
    pub fn cut(&mut self, clipboard: &mut dyn ClipboardProvider) -> bool {
        !self.disabled && self.input.handle_cut(clipboard)
    }

    /// Type what `clipboard` holds over the selection, on one line: its
    /// line breaks and tabs become spaces
    /// ([`single_line`](crate::widgets::form::text_input::single_line)). `false`
    /// with an empty clipboard, unfocused or disabled.
    pub fn paste(&mut self, clipboard: &mut dyn ClipboardProvider) -> bool {
        !self.disabled && self.input.handle_paste(clipboard)
    }

    /// Select the whole text. `false` when unfocused or disabled.
    pub fn select_all(&mut self) -> bool {
        self.edit(EditKey::SelectAll)
    }

    /// A clipboard key: [`copy`](Self::copy), [`cut`](Self::cut),
    /// [`paste`](Self::paste) or [`select_all`](Self::select_all). `true`
    /// when it did something.
    pub fn clip(&mut self, key: ClipKey, clipboard: &mut dyn ClipboardProvider) -> bool {
        match key {
            ClipKey::Copy => self.copy(clipboard),
            ClipKey::Cut => self.cut(clipboard),
            ClipKey::Paste => self.paste(clipboard),
            ClipKey::SelectAll => self.select_all(),
        }
    }

    /// Some text is selected: what copy and cut take.
    pub fn has_selection(&self) -> bool {
        self.input
            .buffer
            .selected_text()
            .is_some_and(|t| !t.is_empty())
    }

    /// IME commit / preedit, forwarded to the core.
    pub fn ime(&mut self, committed: &str, preedit: &str) {
        self.input.handle_ime(committed, preedit);
    }

    /// An input method's step, with its caret: `committed` is typed, the
    /// same way as [`insert`](Self::insert) (over the selection), and
    /// `preedit` draws inline until the next step (see
    /// [`TextInput::handle_ime_at`]). `false` when unfocused or disabled.
    pub fn compose(
        &mut self,
        committed: &str,
        preedit: &str,
        cursor: Option<(usize, usize)>,
    ) -> bool {
        if !self.input.focused || self.disabled {
            return false;
        }
        self.insert(committed);
        self.input.handle_ime_at("", preedit, cursor);
        true
    }

    /// The input method went away: the composition is dropped, the text
    /// stays. `true` when there was one.
    pub fn end_compose(&mut self) -> bool {
        self.input.clear_preedit()
    }

    /// Where an input method puts its candidate window: the caret, the
    /// preedit's while composing, as a rect in the coordinates of `bounds`,
    /// one caret wide and one line tall.
    pub fn ime_area(&self, bounds: Rect, theme: &Theme) -> Rect {
        self.ime_area_in(bounds, theme, self.slots)
    }

    /// [`ime_area`](Self::ime_area) with the ends taken by `slots`, for an
    /// owner whose addons measure in the theme (an input group).
    pub fn ime_area_in(&self, bounds: Rect, theme: &Theme, slots: [Slot; 2]) -> Rect {
        let style = self.style(theme);
        let (x, y, w) = self.text_box(bounds, &style, theme, slots);
        let fit = self.fit(&style, w, theme);
        Rect::new(
            x - fit.scroll + self.input.caret_x(&style, fit.width),
            y,
            theme.control.focus_ring_width,
            style.line_height,
        )
    }

    /// The text's box inside `bounds`: its origin, `x` and `y`, and its
    /// width, clear of what takes either end.
    fn text_box(
        &self,
        bounds: Rect,
        style: &TextStyle,
        theme: &Theme,
        slots: [Slot; 2],
    ) -> (f32, f32, f32) {
        let pad = self.recipe(theme).pad_x;
        let [start, end] = slots.map(|slot| match slot {
            Slot::Pad => pad,
            slot => theme.form.inset(slot),
        });
        (
            bounds.x + start,
            bounds.y + TextMeasurer::vertical_center(style, bounds.h),
            (bounds.w - start - end).max(0.0),
        )
    }

    /// How the text as drawn sits in a box `w` wide: laid out in the box
    /// while it fits, unbounded and slid to the caret (focused) when not.
    fn fit(&self, style: &TextStyle, w: f32, theme: &Theme) -> LineFit {
        let shown = self.input.shown();
        let (line_w, _) = TextMeasurer::measure_styled(&shown, style, None);
        let caret = self.input.focused.then(|| {
            (
                self.input.caret_x(style, None),
                theme.control.focus_ring_width,
            )
        });
        LineFit::new(line_w, w, caret)
    }

    /// Advance the caret blink by `dt` seconds. Focused, the ask is the
    /// caret's: a frame when it flipped (`now`) and the time to the next
    /// flip (`next`); blurred, nothing. The owner merges it with its
    /// other motion and the shell sleeps until the earliest deadline.
    pub fn wake(&mut self, dt: f32) -> Wake {
        self.input.wake(dt)
    }

    /// Hover tracks the bounds; a press inside focuses and places the
    /// caret on the clicked glyph. A press outside blurs, so a screen
    /// with several fields needs no extra bookkeeping.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        theme: &Theme,
    ) -> EventResult {
        self.handle_event_in(event, bounds, theme, self.slots)
    }

    /// [`handle_event`](Self::handle_event) with the ends taken by `slots`.
    pub fn handle_event_in(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        theme: &Theme,
        slots: [Slot; 2],
    ) -> EventResult {
        if self.disabled {
            if self.hovered || self.input.focused {
                self.hovered = false;
                self.input.unfocus();
                return EventResult::changed();
            }
            return EventResult::IGNORED;
        }
        // where a press lands: the box, grown to the least target around
        // it (44 px with a finger, `widgets::hit`); the drawing is the box
        let hit = crate::widgets::hit::hit_rect(bounds, theme);
        match *event {
            WidgetEvent::MouseMove { x, y } => {
                let inside = hit.contains(x, y);
                if inside != self.hovered {
                    self.hovered = inside;
                    EventResult::changed()
                } else {
                    EventResult::IGNORED
                }
            }
            WidgetEvent::MouseDown { x, y } => {
                if hit.contains(x, y) {
                    let style = self.style(theme);
                    let (text_x, _, text_w) = self.text_box(bounds, &style, theme, slots);
                    let fit = self.fit(&style, text_w, theme);
                    self.input
                        .handle_click_in(x - text_x + fit.scroll, &style, fit.width);
                    EventResult::clicked()
                } else if self.input.focused {
                    self.input.unfocus();
                    EventResult::changed()
                } else {
                    EventResult::IGNORED
                }
            }
            _ => EventResult::IGNORED,
        }
    }
}
