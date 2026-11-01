//! The clipboard in the gallery's text fields (task-74, the person's
//! request): cmd on macOS, ctrl elsewhere, with c, x, v or a copies, cuts,
//! pastes or selects all in the field that has the keys, through the
//! gallery's one clipboard (`engine::runtime::platform::clipboard`: the system's, made on
//! first use, unless a test puts a `LocalClipboard`). A letter with cmd
//! or ctrl held is never typed into a field.
//!
//! The clipboard is the person's alone: only these keys and the field's
//! menu reach it, never a request through the dispatcher, so an agent
//! neither reads nor writes it. A paste goes into the field as typing
//! would, and the recording keeps the key by its kind.

use engine::runtime::platform::clipboard::{ClipboardProvider, default_clipboard};
use engine::widgets::form::{ClipKey, TextField};

use super::super::{Pane, ShowcaseView};

/// Who has the keys on the active page, for a chord or the field's menu.
pub(in crate::views) enum Keys<'a> {
    /// No text field: the chord goes where a bare key goes.
    Nobody,
    /// A text field the page reaches.
    Field(&'a mut TextField),
    /// A field of a generated surface (the `surface` crate's): it takes a
    /// paste as typing and select all as its editing key; its copy and
    /// cut are not there yet.
    Surface,
}

/// The gallery's clipboard, made on first use.
pub(in crate::views) fn board(
    clipboard: &mut Option<Box<dyn ClipboardProvider>>,
) -> &mut dyn ClipboardProvider {
    clipboard.get_or_insert_with(default_clipboard).as_mut()
}

impl ShowcaseView {
    /// The clipboard the fields use from now on (a test's
    /// `LocalClipboard`).
    pub fn set_clipboard(&mut self, clipboard: Box<dyn ClipboardProvider>) {
        self.clipboard = Some(clipboard);
    }

    /// A character pressed with cmd or ctrl held; `primary` when that is
    /// the platform's primary modifier (cmd on macOS, ctrl elsewhere).
    /// While a text field has the keys, the primary chords of c, x, v and
    /// a are its clipboard and any other chord is dropped, never typed;
    /// with none, the key goes where a bare one goes (`handle_key`), as
    /// before. `true` when a frame is needed.
    pub fn handle_chord(&mut self, key: &str, primary: bool) -> bool {
        // ⌘K (Ctrl+K off macOS), on any page: the command palette.
        if primary && key.eq_ignore_ascii_case("k") {
            return self.toggle_palette();
        }
        let clip = ClipKey::of(key).filter(|_| primary);
        match self.clip_to(clip) {
            Some(changed) => changed,
            None => self.handle_key(key),
        }
    }

    /// `clip` to the field of the active page that has the keys (`None`:
    /// only whether one has them). `None` when no field has them.
    pub(in crate::views) fn clip_to(&mut self, clip: Option<ClipKey>) -> Option<bool> {
        let pane = self.pane();
        let held = self.overlay_holds_pointer();
        let clipboard = &mut self.clipboard;
        let keys = if held {
            Keys::Nobody
        } else {
            self.panes.keys(pane)
        };
        match (keys, clip) {
            (Keys::Nobody, _) => None,
            (Keys::Field(field), Some(clip)) => Some(field.clip(clip, board(clipboard))),
            (Keys::Surface, Some(ClipKey::Paste)) => {
                let text = board(clipboard).get_text().unwrap_or_default();
                let text = engine::widgets::form::text_input::single_line(&text);
                Some(self.panes.type_in_surface(pane, &text))
            }
            (Keys::Surface, Some(ClipKey::SelectAll)) => Some(match pane {
                Some(Pane::Surface) => self.panes.surface.select_all_in(&mut self.panes.todo),
                Some(Pane::Silences) => self.panes.silences.select_all_in(&mut self.panes.todo),
                _ => false,
            }),
            (_, _) => Some(false),
        }
    }
}

impl super::super::pages::panes::Panes {
    /// Who has the keys on `pane`.
    pub(in crate::views) fn keys(&mut self, pane: Option<Pane>) -> Keys<'_> {
        if self.text_host(pane).is_some() {
            return match self.text_host_mut(pane) {
                Some(host) => host.keys(),
                None => Keys::Nobody,
            };
        }
        match pane {
            Some(Pane::Todo) => self.todo.keys(),
            Some(Pane::Surface) => self.surface.keys(&mut self.todo),
            Some(Pane::Silences) => self.silences.keys(),
            Some(Pane::Git) => self.git.keys(),
            _ => Keys::Nobody,
        }
    }

    /// `text` typed, a character at a time, into the field of the
    /// generated surface on `pane` that has the keys.
    fn type_in_surface(&mut self, pane: Option<Pane>, text: &str) -> bool {
        let mut typed = false;
        for c in text.chars() {
            let one = c.to_string();
            typed |= match pane {
                Some(Pane::Surface) => self.surface.handle_text(&one, &mut self.todo),
                Some(Pane::Silences) => self.silences.handle_text(&one, &mut self.todo),
                _ => false,
            };
        }
        typed
    }
}
