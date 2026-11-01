//! The Menus page as a text host: the folder picker's search box and the
//! new folder's name take the keys, the input method and the clipboard.

use engine::styling::theme::Theme;
use engine::widgets::prelude::Rect;
use engine::widgets::selection::Folders;

use super::super::EditKey;
use super::MenusPage;
use crate::views::ImeEvent;
use crate::views::text_input::clipboard::Keys;
use crate::views::text_input::host::TextHost;

impl TextHost for MenusPage {
    /// Type `s` into the focused box. `false` when none is focused.
    fn handle_text(&mut self, s: &str) -> bool {
        self.folders_mut()
            .and_then(Folders::field_mut)
            .is_some_and(|f| f.insert(s))
    }

    /// The editing keys edit the focused box; Tab is the view's.
    fn handle_edit_key(&mut self, key: EditKey) -> bool {
        if key == EditKey::Tab {
            return false;
        }
        self.folders_mut()
            .and_then(Folders::field_mut)
            .is_some_and(|f| f.edit(key.into()))
    }

    /// The input method composes in the focused box; `Disabled` drops a
    /// composition wherever one is.
    fn handle_ime(&mut self, ime: ImeEvent<'_>) -> bool {
        let Some(folders) = self.folders_mut() else {
            return false;
        };
        let (committed, preedit, cursor) = match ime {
            ImeEvent::Disabled => {
                let mut dropped = folders.search.end_compose();
                if let Some(f) = folders.naming_mut() {
                    dropped |= f.end_compose();
                }
                return dropped;
            }
            ImeEvent::Preedit(text, cursor) => ("", text, cursor),
            ImeEvent::Commit(text) => (text, "", None),
        };
        folders
            .field_mut()
            .is_some_and(|f| f.compose(committed, preedit, cursor))
    }

    fn ime_area(&self, content: Rect, theme: &Theme) -> Option<Rect> {
        let f = self.folders()?;
        let field = f.field()?;
        let (search, name) = self.field_rects(content, theme);
        let at = if field.is_focused() && f.search.is_focused() {
            search?
        } else {
            name?
        };
        Some(field.ime_area_in(at, theme, Folders::slots(theme)))
    }

    /// A right-click on a box gives it the keys, for its menu.
    fn right_click(&mut self, x: f32, y: f32, content: Rect, theme: &Theme) -> bool {
        let (search, name) = self.field_rects(content, theme);
        let on_name = name.is_some_and(|r| r.contains(x, y));
        if !on_name && !search.is_some_and(|r| r.contains(x, y)) {
            return false;
        }
        if let Some(f) = self.folders_mut() {
            f.focus_field(on_name);
        }
        true
    }

    fn keys(&mut self) -> Keys<'_> {
        match self.folders_mut().and_then(Folders::field_mut) {
            Some(f) => Keys::Field(f),
            None => Keys::Nobody,
        }
    }

    fn handle_escape(&mut self) -> bool {
        MenusPage::handle_escape(self)
    }

    /// Enter in the name's box adds the folder.
    fn handle_enter(&mut self) -> bool {
        let naming = self
            .folders()
            .and_then(Folders::naming)
            .is_some_and(|f| f.is_focused());
        naming && self.commit_name()
    }
}
