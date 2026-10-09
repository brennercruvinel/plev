//! App section: the canonical small-but-complete todo app over the tested
//! domain model (showcase::model::todo). comps::form::text_input::TextInput owns
//! editing, focus, blink and cursor byte mapping; rows pair the design
//! system Checkbox with a strike-through whose width comes from
//! TextMeasurer (never chars * factor); filters are HOFF glass pills; the
//! list scrolls internally (ScrollState) so the footer never overflows.
//!
//! The page is a capability (`semantics.rs`): it projects what it means,
//! runs what it offers in `execute`, and owns the `sem::Dispatcher` every
//! actor goes through. A click on a checkbox, a delete button or a filter
//! pill, and the Enter key, become requests over the page as it is at
//! that instant; nothing here calls the model directly. Typing, caret
//! moves and focus stay with the field: they are editing, not a
//! capability.
//!
//! Wiring contract for the chrome (view/mod.rs + main.rs), mirroring the
//! Forms section: route Key::Character/Space through `handle_text` before
//! the digit/theme hotkeys, view::EditKey through `handle_edit_key`,
//! Escape through `handle_escape` (before popping overlays), map
//! NamedKey::Enter to `handle_enter`, and merge `wake` into the view's
//! wake so item tweens and the cursor blink advance.

mod draw;
mod layout;
mod semantics;
#[cfg(test)]
mod tests;

use comps::form::TextField;
use comps::prelude::{Checkbox, EventResult, Rect, Wake, WidgetEvent};
use engine::input::scroll::ScrollState;
use engine::theme::Theme;
use sem::Dispatcher;
use showcase::model::todo::{Filter, TodoModel};

use super::EditKey;
use crate::view::clipboard::Keys;
use layout::*;
use semantics::{Ids, Target};

pub struct TodoPage {
    model: TodoModel,
    input: TextField,
    scroll: ScrollState,
    /// One retained Checkbox per visible row, in visible order (hover and
    /// press survive frames; `checked` is driven by the model).
    rows: Vec<(u64, Checkbox)>,
    hover_row: Option<u64>,
    hover_delete: Option<u64>,
    hover_pill: Option<usize>,
    /// The ids the page projects and binds; `None` only if the constant
    /// ids failed to parse (a test proves they do not), which leaves the
    /// page without semantics: every request is then refused.
    ids: Option<Ids>,
    /// The one path from a request to `execute`, for every actor.
    dispatcher: Dispatcher,
    /// The monotonic clock receipts are stamped with (`sem` never reads
    /// one).
    clock: web_time::Instant,
    /// Set by the view while an overlay holds the pointer (a modal, a
    /// menu): the page projects itself inert and takes no input.
    inert: bool,
    /// The filter tab a screen reader moved the focus to: the pills draw
    /// no focus of their own, so the page keeps it and projects it. A
    /// click of the person, the field taking the focus and Escape let it
    /// go.
    tab_focus: Option<Filter>,
}

impl TodoPage {
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
        let ids = Ids::new()
            .inspect_err(|e| log::error!("todo: the page ids do not parse: {e}"))
            .ok();
        let mut input = TextField::new("What needs doing? Enter adds it.");
        if let Some(ids) = &ids {
            input = input.sem(ids.input.clone());
        }
        Self {
            model,
            input,
            scroll: ScrollState::new(),
            rows: Vec::new(),
            hover_row: None,
            hover_delete: None,
            hover_pill: None,
            ids,
            dispatcher: Dispatcher::new(),
            clock: web_time::Instant::now(),
            inert: false,
            tab_focus: None,
        }
    }

    /// The view says whether an overlay holds the pointer. While it does,
    /// `todo` is projected inert, so any request comes back `Inert` (the
    /// click already cannot pass), and keys fall through to the chrome.
    pub fn set_inert(&mut self, inert: bool) {
        self.inert = inert;
    }

    /// The add field's rect for `content` (the chrome tests click it).
    #[cfg(test)]
    pub(crate) fn input_rect(&self, content: Rect, theme: &Theme) -> Rect {
        compute(content, &self.counter_text(), theme).input
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

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
    ) -> EventResult {
        self.sync_rows();
        let l = compute(content, &self.counter_text(), theme);
        self.sync_scroll(&l);
        let mut r = EventResult::IGNORED;

        // Field focus: a click inside places the cursor (the field maps
        // the local x to a byte via real shaping); a click anywhere else
        // blurs without consuming the event. A click is the person's, so
        // a tab a screen reader focused lets its focus go.
        if let WidgetEvent::MouseDown { .. } = *event {
            self.tab_focus = None;
            let fr = self.input.handle_event(event, l.input, theme);
            if fr.clicked {
                return EventResult::changed();
            }
            r.changed |= fr.changed;
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
                let hit = l
                    .pills
                    .iter()
                    .zip(Filter::ALL)
                    .find(|(p, _)| p.contains(x, y));
                if let Some((_, filter)) = hit {
                    let res = self.ask(Target::Tab(filter), "select", None);
                    if res.changed {
                        return r.merge(res);
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
            r = r.merge(self.ask(Target::Item(id), "toggle", None));
        }
        if let Some(id) = delete {
            r = r.merge(self.ask(Target::Item(id), "remove", None));
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
    pub fn handle_text(&mut self, s: &str) -> bool {
        if self.inert || !self.input.is_focused() {
            return false;
        }
        let mut chars = s.chars();
        match (chars.next(), chars.next()) {
            (Some(c), None) if !c.is_control() => {
                self.input.insert(&c.to_string());
                true
            }
            _ => false,
        }
    }

    /// The add field has the keys while focused (a chord), not while an
    /// overlay holds the page.
    pub(in crate::view) fn keys(&mut self) -> Keys<'_> {
        if self.inert || !self.input.is_focused() {
            return Keys::Nobody;
        }
        Keys::Field(&mut self.input)
    }

    /// Editing keys, same route as forms::handle_edit_key. Tab is left to
    /// the chrome; Enter has its own hook below.
    pub fn handle_edit_key(&mut self, key: EditKey) -> bool {
        if self.inert || !self.input.is_focused() {
            return false;
        }
        match key {
            EditKey::Tab => return false,
            other => {
                self.input.edit(other.into());
            }
        }
        true
    }

    /// Enter asks `todo.input` to `submit`: `execute` adds the trimmed
    /// field text as a todo, clears the field and reveals the newest row.
    /// Consumed (even when blank, which `submit` refuses) while focused.
    pub fn handle_enter(&mut self) -> bool {
        if !self.input.is_focused() {
            return false;
        }
        self.ask(Target::Input, "submit", None);
        true
    }

    /// Escape blurs the field, or lets go of a focused tab; mirrors
    /// forms::handle_escape (the chrome asks the section before popping
    /// overlays or quitting). An inert page leaves it to the overlay.
    pub fn handle_escape(&mut self) -> bool {
        if !self.inert && (self.input.is_focused() || self.tab_focus.is_some()) {
            self.input.unfocus();
            self.tab_focus = None;
            return true;
        }
        false
    }

    /// Advance item tweens and the cursor blink: frames while a tween
    /// runs, and for the focused caret only the next flip.
    pub fn wake(&mut self, dt: f32) -> Wake {
        Wake::moving(self.model.update(dt)).merge(self.input.wake(dt))
    }
}
