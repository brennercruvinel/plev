//! Set/printing autocomplete. Port of `desktop/ui/js/ui/set-picker.js`.
//!
//! `engine::ui::widgets::Select` is a closed option list and does not fit:
//! this field searches as you type, keeps free text when nothing is chosen
//! ("a promo or a set the index doesn't know is still worth recording"), and
//! - scoped to a card - lists that card's printings on focus, before any
//! typing. So it is built on `TextInput` plus a hand-drawn suggestion list,
//! like the JS was.
//!
//! Because suggestions need an async `SearchSets` round trip, this component
//! is the one (with [`super::add_card`]) that takes `&mut ScreenCtx` - and
//! uses it only to send commands. The answers land back via
//! [`SetPicker::on_suggestions`], fed by the owning screen.

use engine::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use engine::text::{TextMeasurer, TextStyle};
use engine::theme::{Theme, TypographyScale};
use engine::ui::widgets::{EventResult, Rect, WidgetEvent, glass_pill, menu_shadow, rounded_rect};
use spellbook_core::client::Command;
use spellbook_core::ops::cards::SetInfo;

use super::super::{EditKey, ScreenCtx, with_alpha};
use super::field::LabeledField;

/// Suggestion rows shown at once. The JS listed everything the API
/// returned; a modal panel cannot scroll a 40-row list without taking over
/// the window, so the picker shows the top of the same ranked list.
const MAX_ROWS: usize = 8;
const ROW_H: f32 = 38.0;
/// Keystroke debounce, the JS's 180ms.
const DEBOUNCE: f32 = 0.180;
/// Code-style label style for the picker rows (base-2sm, like the kit).
fn row_style() -> TextStyle {
    TypographyScale::hoff().base_2sm()
}

/// `Name (CODE)` in the field, as the JS's `choose` wrote it.
fn display_name(set: &SetInfo) -> String {
    match &set.name {
        Some(name) if !name.is_empty() => format!("{} ({})", name, set.code.to_uppercase()),
        _ => set.code.to_uppercase(),
    }
}

/// The trailing span of a row: "DMU · 2022" (code always: for recent sets
/// that is what people remember, per the JS module docs).
fn row_meta(set: &SetInfo) -> String {
    let year: String = set
        .released_at
        .as_deref()
        .unwrap_or("")
        .chars()
        .take(4)
        .collect();
    if year.is_empty() {
        set.code.to_uppercase()
    } else {
        format!("{} · {year}", set.code.to_uppercase())
    }
}

pub struct SetPicker {
    /// Visible text: either the typed query or the chosen "Name (CODE)".
    pub field: LabeledField,
    /// The stored value: the chosen code, or the trimmed free text while
    /// nothing was chosen (the hidden input of the JS).
    code: Option<String>,
    /// Scoped to this card's printings (the JS's `card` argument).
    card: Option<String>,
    suggestions: Vec<SetInfo>,
    /// A request is in flight; its answer pairs with the query it carried.
    in_flight: bool,
    /// A fresh edit waits out the debounce before asking.
    dirty: bool,
    since_edit: f32,
    active: usize,
    hovered: Option<usize>,
    open: bool,
}

impl SetPicker {
    pub fn new(label: impl Into<String>, theme: &Theme, card: Option<String>) -> Self {
        Self {
            field: LabeledField::new(label, "Nome ou código da edição…", theme),
            code: None,
            card,
            suggestions: Vec::new(),
            in_flight: false,
            dirty: false,
            since_edit: 0.0,
            active: 0,
            hovered: None,
            open: false,
        }
    }

    /// The stored code (`getSetCode` in the JS): the chosen one, or the
    /// trimmed text as it stands. Empty becomes `None`.
    pub fn code(&self) -> Option<String> {
        self.code.clone().filter(|c| !c.is_empty())
    }

    /// Prefill from a stored code, as when an edit form loads an existing
    /// copy. With no `SetInfo` at hand the bare uppercase code is shown,
    /// which is what the old stored value looked like.
    pub fn set_code(&mut self, code: Option<&str>, set: Option<&SetInfo>) {
        self.code = code.map(str::to_string);
        match (code, set) {
            (Some(_), Some(s)) => self.field.set_value(&display_name(s)),
            (Some(c), None) => self.field.set_value(&c.to_uppercase()),
            (None, _) => self.field.set_value(""),
        }
    }

    /// Field rect below the label.
    pub fn field_rect(&self, rect: Rect) -> Rect {
        self.field.field_rect(rect)
    }

    /// The suggestion list hangs off the field, drawn on the overlay layer.
    fn list_rect(&self, field: Rect) -> Rect {
        let rows = self.suggestions.len().min(MAX_ROWS) as f32;
        Rect::new(field.x, field.y + field.h + 4.0, field.w, rows * (ROW_H + 4.0) + 8.0)
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Scope the list to one card's printings, or drop the scope.
    pub fn set_card(&mut self, card: Option<String>) {
        self.card = card;
    }

    fn close(&mut self) {
        self.open = false;
        self.suggestions.clear();
        self.dirty = false;
        self.active = 0;
        self.hovered = None;
    }

    fn choose(&mut self, set: &SetInfo) {
        self.field.set_value(&display_name(set));
        self.code = Some(set.code.clone());
        self.close();
    }

    /// Focus gained or lost. With a card scoped, focusing lists its
    /// printings straight away (the JS's `focus` handler): "which printing
    /// of this card do I have?" has no useful typing step.
    pub fn set_focused(&mut self, focused: bool, ctx: &mut ScreenCtx) {
        if focused {
            self.field.input.focus();
            if self.card.is_some() && !self.in_flight {
                self.in_flight = true;
                ctx.send(Command::SearchSets {
                    q: String::new(),
                    card: self.card.clone(),
                    limit: 40,
                });
            }
        } else {
            self.field.input.unfocus();
            self.close();
        }
    }

    /// Typed text. Invalidates the previous pick: the raw text stands in as
    /// the stored code until something is chosen, exactly as the JS hid the
    /// typed value in its hidden input.
    pub fn handle_text(&mut self, s: &str) -> EventResult {
        if !self.field.handle_text(s) {
            return EventResult::IGNORED;
        }
        self.code = Some(self.field.value().trim().to_string());
        self.dirty = !self.field.value().trim().is_empty();
        self.open = true;
        if self.field.value().trim().is_empty() {
            self.close();
        }
        EventResult::changed()
    }

    /// Cursor blink + the debounced query. Returns `true` while the blink
    /// needs frames.
    pub fn tick(&mut self, dt: f32, ctx: &mut ScreenCtx) -> bool {
        self.since_edit += dt;
        if self.dirty && !self.in_flight && self.since_edit >= DEBOUNCE {
            self.dirty = false;
            self.in_flight = true;
            ctx.send(Command::SearchSets {
                q: self.field.value().trim().to_string(),
                card: self.card.clone(),
                limit: 40,
            });
        }
        self.field.tick(dt)
    }

    /// The screen hands the `SetsFound` payload down. The worker answers in
    /// order, so each answer pairs with the oldest unanswered request; an
    /// answer landing after a blur is dropped (the JS's blur `close`).
    pub fn on_suggestions(&mut self, sets: &[SetInfo]) -> bool {
        if !self.in_flight {
            return false;
        }
        self.in_flight = false;
        if !self.field.input.focused {
            return false;
        }
        self.suggestions = sets.to_vec();
        self.active = 0;
        self.open = !self.suggestions.is_empty();
        true
    }

    /// Pointer event, in the dialog's own coordinates - the list hangs off
    /// the field even though it renders on the overlay layer.
    pub fn handle_event(&mut self, event: &WidgetEvent, field: Rect) -> EventResult {
        match *event {
            WidgetEvent::MouseMove { x, y } => {
                let list = self.list_rect(field);
                let hovered = (self.open && list.contains(x, y))
                    .then(|| ((y - list.y - 4.0) / (ROW_H + 4.0)).floor() as usize)
                    .filter(|i| *i < self.suggestions.len());
                if hovered != self.hovered {
                    self.hovered = hovered;
                    if let Some(i) = hovered {
                        self.active = i;
                    }
                    EventResult::changed()
                } else {
                    EventResult::IGNORED
                }
            }
            WidgetEvent::MouseDown { x, y } => {
                // mousedown, not click: blur would close the list before the
                // click landed (the JS comment, kept).
                let list = self.list_rect(field);
                if self.open && list.contains(x, y) {
                    let i = ((y - list.y - 4.0) / (ROW_H + 4.0)).floor() as usize;
                    if let Some(set) = self.suggestions.get(i).cloned() {
                        self.choose(&set);
                        return EventResult::clicked();
                    }
                    return EventResult::changed();
                }
                if field.contains(x, y) {
                    self.field.click(x - field.x);
                    return EventResult::changed();
                }
                // Outside: blur closes the list.
                if self.field.input.focused || self.open {
                    self.field.input.unfocus();
                    self.close();
                    return EventResult::changed();
                }
                EventResult::IGNORED
            }
            _ => EventResult::IGNORED,
        }
    }

    /// Enter takes the active row; the rest belongs to the text buffer.
    /// Escape is the shell's concern - it calls [`SetPicker::handle_escape`].
    pub fn handle_edit_key(&mut self, key: EditKey) -> (bool, bool) {
        if self.open && key == EditKey::Enter && !self.suggestions.is_empty() {
            let set = self.suggestions[self.active.min(self.suggestions.len() - 1)].clone();
            self.choose(&set);
            return (true, true);
        }
        self.field.handle_edit_key(key)
    }

    /// Escape closes the list (not the dialog), the JS's `Escape` handler.
    pub fn handle_escape(&mut self) -> bool {
        if self.open {
            self.close();
            return true;
        }
        false
    }

    /// Label + field in normal flow.
    pub fn render(&self, c: &mut Compositor, rect: Rect, theme: &Theme) {
        self.field.render(c, rect, theme);
    }

    /// The suggestion list, on the overlay layer: it must float over the
    /// dialog's own content, which is why the JS version called
    /// `scrollIntoView` inside a scrolling modal - here it draws on top.
    pub fn render_overlay(&self, c: &mut Compositor, layer: LayerId, rect: Rect, theme: &Theme) {
        if !self.open || self.suggestions.is_empty() {
            return;
        }
        let field = self.field_rect(rect);
        let list = self.list_rect(field);
        let radius = theme.radius.lg;
        c.push_to_layer(layer, menu_shadow(list, radius));
        for node in glass_pill(list, radius, theme.glass.edge_soft.0, 1.5, theme.glass.popover.0) {
            c.push_to_layer(layer, node);
        }
        let style = row_style();
        for (i, set) in self.suggestions.iter().take(MAX_ROWS).enumerate() {
            let ry = list.y + 4.0 + i as f32 * (ROW_H + 4.0);
            let row = Rect::new(list.x + 4.0, ry, list.w - 8.0, ROW_H);
            if self.hovered == Some(i) || (self.hovered.is_none() && self.active == i) {
                c.push_to_layer(
                    layer,
                    rounded_rect(
                        row.x,
                        row.y,
                        row.w,
                        row.h,
                        theme.radius.md,
                        theme.glass.surface_hover.0,
                    ),
                );
            }
            let meta = row_meta(set);
            let (meta_w, _) = TextMeasurer::measure_styled(&meta, &style, None);
            c.push_to_layer(
                layer,
                SceneNode::Text {
                    key: TextNodeKey::from_style(
                        set.name.as_deref().unwrap_or(""),
                        &style,
                        Some((row.w - meta_w - 30.0).max(20.0)),
                    ),
                    x: row.x + 10.0,
                    y: row.y + TextMeasurer::vertical_center(&style, ROW_H),
                    color: with_alpha(
                        theme.colors.text.0,
                        theme.colors.text.0[3] * 0.8,
                    ),
                },
            );
            c.push_to_layer(
                layer,
                SceneNode::Text {
                    key: TextNodeKey::from_style(&meta, &style, None),
                    x: row.x + row.w - meta_w - 10.0,
                    y: row.y + TextMeasurer::vertical_center(&style, ROW_H),
                    color: theme.colors.text_dim.0,
                },
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(code: &str, name: &str, year: &str) -> SetInfo {
        SetInfo {
            code: code.into(),
            name: Some(name.into()),
            released_at: Some(format!("{year}-01-01")),
            set_type: None,
            cards: None,
        }
    }

    /// The field shows "Name (CODE)", the stored value is the bare code -
    /// the pair the JS kept in its two inputs.
    #[test]
    fn a_choice_shows_the_name_and_stores_the_code() {
        assert_eq!(
            display_name(&set("dmu", "Dominaria United", "2022")),
            "Dominaria United (DMU)"
        );
    }

    #[test]
    fn the_meta_line_is_the_code_and_the_year() {
        assert_eq!(row_meta(&set("dmu", "Dominaria United", "2022")), "DMU · 2022");
        let no_date = SetInfo {
            released_at: None,
            ..set("dmu", "Dominaria United", "2022")
        };
        assert_eq!(row_meta(&no_date), "DMU");
    }

    /// Focused SetPicker bookkeeping without a data thread.
    fn test_picker(card: Option<String>) -> SetPicker {
        let theme = Theme::hoff();
        let mut picker = SetPicker::new("Edição", &theme, card);
        // Focus directly: set_focused needs a live ctx, which the pure-logic
        // tests do not have.
        picker.field.input.focus();
        picker
    }

    /// Free text counts as the stored code until a choice replaces the pick
    /// (the JS's hidden-input overwrite on `input`).
    #[test]
    fn typing_replaces_the_pick_with_free_text() {
        let mut picker = test_picker(None);
        picker.handle_text("promo");
        assert_eq!(picker.code().as_deref(), Some("promo"));

        // Choosing a suggestion swaps it for the real code.
        picker.in_flight = true;
        picker.on_suggestions(&[set("dmu", "Dominaria United", "2022")]);
        picker.handle_edit_key(EditKey::Enter);
        assert_eq!(picker.code().as_deref(), Some("dmu"));
        assert_eq!(picker.field.value(), "Dominaria United (DMU)");
        assert!(!picker.is_open());
    }

    /// Scoped to a card, an empty query still opens the printings list once
    /// the answer lands (the JS's focus handler asked with "").
    #[test]
    fn an_answer_opens_the_list_only_while_focused() {
        let mut picker = test_picker(Some("Sol Ring".into()));
        picker.in_flight = true;
        assert!(picker.on_suggestions(&[set("c21", "Commander 2021", "2021")]));
        assert!(picker.is_open());

        // The same answer after a blur is dropped.
        let mut picker = test_picker(None);
        picker.in_flight = true;
        picker.field.input.unfocus();
        assert!(!picker.on_suggestions(&[set("dmu", "Dominaria United", "2022")]));
        assert!(!picker.is_open());
    }

    /// Escape closes the list but leaves the dialog (false) alone.
    #[test]
    fn escape_closes_the_suggestions_first() {
        let mut picker = test_picker(None);
        picker.in_flight = true;
        picker.on_suggestions(&[set("dmu", "Dominaria United", "2022")]);
        assert!(picker.handle_escape());
        assert!(!picker.is_open());
        assert!(!picker.handle_escape());
    }
}
