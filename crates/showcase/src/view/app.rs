//! App section: the canonical small-but-complete todo app over the tested
//! domain model (showcase::model::todo). plev::text_input::TextInput owns
//! editing, focus, blink and cursor byte mapping; rows pair the design
//! system Checkbox with a strike-through whose width comes from
//! TextMeasurer (never chars * factor); filters are HOFF glass pills; the
//! list scrolls internally (ScrollState) so the footer never overflows.
//!
//! Wiring contract for the chrome (view/mod.rs + main.rs), mirroring the
//! Forms section: route Key::Character/Space through `handle_text` before
//! the digit/theme hotkeys, view::EditKey through `handle_edit_key`,
//! Escape through `handle_escape` (before popping overlays), map
//! NamedKey::Enter to `handle_enter`, and add `tick` to the view tick so
//! item tweens and the cursor blink advance.

mod draw;
mod layout;
#[cfg(test)]
mod tests;

use plev::input::scroll::ScrollState;
use plev::text_input::TextInput;
use plev::ui::widgets::{Checkbox, EventResult, Rect, WidgetEvent};
use showcase::model::todo::{Filter, TodoModel};

use super::EditKey;
use layout::*;

pub struct AppSection {
    model: TodoModel,
    input: TextInput,
    scroll: ScrollState,
    /// One retained Checkbox per visible row, in visible order (hover and
    /// press survive frames; `checked` is driven by the model).
    rows: Vec<(u64, Checkbox)>,
    hover_row: Option<u64>,
    hover_delete: Option<u64>,
    hover_pill: Option<usize>,
}

impl AppSection {
    pub fn new() -> Self {
        let mut model = TodoModel::new();
        model.add("Absorb the plev demos into the showcase");
        model.add("Measure text, never estimate it");
        if let Some(id) = model.add("Port the todo domain (tested first)") {
            model.toggle(id);
        }
        // Seeds start at rest: settle their enter/strike tweens.
        for _ in 0..64 {
            if !model.update(0.25) {
                break;
            }
        }
        Self {
            model,
            input: TextInput::new()
                .with_placeholder("What needs doing? Enter adds it.")
                .with_font_size(INPUT_FONT),
            scroll: ScrollState::new(),
            rows: Vec::new(),
            hover_row: None,
            hover_delete: None,
            hover_pill: None,
        }
    }

    /// Natural height: the card fills the viewport; when the viewport is
    /// shorter than the card minimum, the page scroll covers the rest.
    pub fn content_height(&self, content: Rect) -> f32 {
        content.h.max(MIN_H)
    }

    fn counter_text(&self) -> String {
        let c = self.model.counts();
        format!("{} active · {} done", c.active, c.completed)
    }

    /// Mirror the visible items into the retained row widgets.
    fn sync_rows(&mut self) {
        let mut rows = Vec::with_capacity(self.model.visible_items().len());
        for item in self.model.visible_items() {
            let mut cb = self
                .rows
                .iter()
                .find(|(id, _)| *id == item.id())
                .map(|(_, cb)| cb.clone())
                .unwrap_or_else(|| Checkbox::new(false));
            cb.checked = item.completed();
            rows.push((item.id(), cb));
        }
        self.rows = rows;
    }

    fn sync_scroll(&mut self, l: &Layout) {
        self.scroll.set_viewport(l.list.h);
        self.scroll.set_content(self.rows.len() as f32 * ROW_H);
    }

    pub fn handle_event(&mut self, event: &WidgetEvent, content: Rect) -> EventResult {
        self.sync_rows();
        let l = compute(content, &self.counter_text());
        self.sync_scroll(&l);
        let mut r = EventResult::IGNORED;

        // Field focus: a click inside places the cursor (TextInput maps the
        // local x to a byte via real shaping); a click anywhere else blurs
        // without consuming the event.
        if let WidgetEvent::MouseDown { x, y } = *event {
            if l.input.contains(x, y) {
                self.input.handle_click(x - l.input.x - INPUT_PAD);
                return EventResult::changed();
            }
            if self.input.focused {
                self.input.unfocus();
                r.changed = true;
            }
        }

        // Filter pills.
        match *event {
            WidgetEvent::MouseMove { x, y } => {
                let hit = l.pills.iter().position(|p| p.contains(x, y));
                if hit != self.hover_pill {
                    self.hover_pill = hit;
                    r.changed = true;
                }
            }
            WidgetEvent::MouseDown { x, y } => {
                if let Some(i) = l.pills.iter().position(|p| p.contains(x, y)) {
                    if self.model.set_filter(Filter::ALL[i]) {
                        self.scroll.scroll_to(0.0);
                        return r.merge(EventResult::clicked());
                    }
                    r.handled = true;
                }
            }
            _ => {}
        }

        // Rows: checkbox toggle, delete, hover bookkeeping. A MouseDown
        // outside the list viewport never reaches the (clipped) rows.
        let gated_down =
            matches!(*event, WidgetEvent::MouseDown { x, y } if !l.list.contains(x, y));
        let offset = self.scroll.offset();
        let (mut hover_row, mut hover_delete) = (self.hover_row, self.hover_delete);
        if matches!(event, WidgetEvent::MouseMove { .. }) {
            (hover_row, hover_delete) = (None, None);
        }
        let (mut toggle, mut delete) = (None, None);
        for (i, (id, cb)) in self.rows.iter_mut().enumerate() {
            let row = row_rect(l.list, i, offset);
            if row.y + ROW_H < l.list.y || row.y > l.list.y + l.list.h {
                continue;
            }
            let del = delete_rect(row);
            match *event {
                WidgetEvent::MouseMove { x, y } if l.list.contains(x, y) && row.contains(x, y) => {
                    hover_row = Some(*id);
                    if del.contains(x, y) {
                        hover_delete = Some(*id);
                    }
                }
                WidgetEvent::MouseDown { x, y } if !gated_down && del.contains(x, y) => {
                    delete = Some(*id);
                    continue;
                }
                _ => {}
            }
            if !gated_down {
                let res = cb.handle_event(event, checkbox_rect(row));
                if res.clicked {
                    toggle = Some(*id);
                }
                r = r.merge(res);
            }
        }
        if (hover_row, hover_delete) != (self.hover_row, self.hover_delete) {
            (self.hover_row, self.hover_delete) = (hover_row, hover_delete);
            r.changed = true;
        }
        if let Some(id) = toggle {
            self.model.toggle(id);
            r = r.merge(EventResult::clicked());
        }
        if let Some(id) = delete {
            self.model.delete(id);
            r = r.merge(EventResult::clicked());
        }

        // Wheel over the list: internal scroll first; at the clamps the
        // event stays unhandled so the page scroll can take over.
        if let WidgetEvent::Scroll { x, y, delta } = *event
            && l.list.contains(x, y)
        {
            let old = self.scroll.offset();
            self.scroll.scroll_by(delta);
            if self.scroll.offset() != old {
                r = r.merge(EventResult::changed());
            }
        }
        r
    }

    /// Printable characters (Key::Character / Space) forwarded by the
    /// chrome before its own hotkeys, mirroring forms::handle_text: while
    /// the add field is focused, "t" types instead of switching themes.
    #[allow(dead_code)] // chrome wiring is the integration pass; exercised by the tests below
    pub fn handle_text(&mut self, s: &str) -> bool {
        if !self.input.focused {
            return false;
        }
        let mut chars = s.chars();
        match (chars.next(), chars.next()) {
            (Some(c), None) if !c.is_control() => {
                self.input.handle_char(c);
                true
            }
            _ => false,
        }
    }

    /// Editing keys, same route as forms::handle_edit_key. Tab is left to
    /// the chrome; Enter has its own hook below.
    #[allow(dead_code)] // chrome wiring is the integration pass; exercised by the tests below
    pub fn handle_edit_key(&mut self, key: EditKey) -> bool {
        if !self.input.focused {
            return false;
        }
        match key {
            EditKey::Backspace => self.input.handle_backspace(),
            EditKey::Delete => self.input.handle_delete(),
            EditKey::Left => self.input.handle_left(),
            EditKey::Right => self.input.handle_right(),
            EditKey::Home => self.input.handle_home(),
            EditKey::End => self.input.handle_end(),
            EditKey::Tab => return false,
        }
        true
    }

    /// Enter adds the trimmed field text as a todo, clears the field and
    /// reveals the newest row. Consumed (even when empty) while focused.
    #[allow(dead_code)] // chrome wiring is the integration pass; exercised by the tests below
    pub fn handle_enter(&mut self) -> bool {
        if !self.input.focused {
            return false;
        }
        if self.model.add(self.input.buffer.text()).is_some() {
            self.input.buffer.set_text("");
            self.input.reset_blink();
            self.scroll
                .set_content(self.model.visible_items().len() as f32 * ROW_H);
            self.scroll.scroll_to(f32::MAX);
        }
        true
    }

    /// Escape blurs the field; mirrors forms::handle_escape (the chrome
    /// asks the section before popping overlays or quitting).
    #[allow(dead_code)] // chrome wiring is the integration pass; exercised by the tests below
    pub fn handle_escape(&mut self) -> bool {
        if self.input.focused {
            self.input.unfocus();
            return true;
        }
        false
    }

    /// Advance item tweens and the cursor blink. Returns true while frames
    /// are needed (tweens live, or the focused cursor must keep blinking).
    #[allow(dead_code)] // chrome wiring is the integration pass; exercised by the tests below
    pub fn tick(&mut self, dt: f32) -> bool {
        let animating = self.model.update(dt);
        self.input.tick(dt);
        animating || self.input.focused
    }
}
