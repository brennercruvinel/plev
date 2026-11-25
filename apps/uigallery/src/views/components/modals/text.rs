//! The Modals page as a text host: the characters, the editing keys, the
//! input method, the clipboard chords, Enter and Escape go to the box that
//! has the keys (a value of Settings edited in place, the shared file's
//! Invite box).

use engine::styling::theme::Theme;
use engine::widgets::form::TextField;
use engine::widgets::prelude::Rect;

use super::super::{Component, EditKey};
use super::ModalsPage;
use crate::views::ImeEvent;
use crate::views::text_input::clipboard::Keys;
use crate::views::text_input::host::TextHost;

impl ModalsPage {
    fn focused_text(&mut self) -> Option<&mut TextField> {
        self.each_mut().find_map(|s| s.body.text_mut())
    }
}

impl TextHost for ModalsPage {
    fn handle_text(&mut self, s: &str) -> bool {
        self.focused_text().is_some_and(|t| t.insert(s))
    }

    fn handle_edit_key(&mut self, key: EditKey) -> bool {
        if key == EditKey::Tab {
            return false;
        }
        self.focused_text().is_some_and(|t| t.edit(key.into()))
    }

    fn handle_ime(&mut self, ime: ImeEvent<'_>) -> bool {
        let (committed, preedit, cursor) = match ime {
            ImeEvent::Disabled => {
                let mut dropped = false;
                for s in self.each_mut() {
                    dropped |= s.body.end_compose();
                }
                return dropped;
            }
            ImeEvent::Preedit(text, cursor) => ("", text, cursor),
            ImeEvent::Commit(text) => (text, "", None),
        };
        self.focused_text()
            .is_some_and(|t| t.compose(committed, preedit, cursor))
    }

    fn ime_area(&self, content: Rect, theme: &Theme) -> Option<Rect> {
        let rects = self.rects(content, theme).into_iter().flatten();
        self.groups
            .iter()
            .flat_map(|g| g.cards.iter())
            .zip(rects)
            .find_map(|(s, (_, rect))| s.body.ime_area(rect, theme))
    }

    /// The field menu is not offered on the modals: a right-click is the
    /// page's as any click.
    fn right_click(&mut self, _x: f32, _y: f32, _content: Rect, _theme: &Theme) -> bool {
        false
    }

    fn keys(&mut self) -> Keys<'_> {
        match self.focused_text() {
            Some(t) => Keys::Field(t),
            None => Keys::Nobody,
        }
    }

    /// Escape: an open list closes first; then a box edited lets go (the
    /// value as it was); then the focus goes.
    fn handle_escape(&mut self) -> bool {
        if self.each_mut().any(|s| s.body.has_open()) {
            return self.blur();
        }
        for s in self.each_mut() {
            if s.body.escape_text() {
                return true;
            }
        }
        self.blur()
    }

    /// Enter in a value edited saves it.
    fn handle_enter(&mut self) -> bool {
        self.each_mut().any(|s| s.body.enter())
    }
}
