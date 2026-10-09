//! The input method's route in the gallery (task-74 phase 4): what the
//! platform's IME sends, winit-free, to the page that takes text from it,
//! and where its candidate window goes. The Input page's focused field
//! composes inline and takes a commit as typing; the Editor page takes it
//! through `EditorView::handle_ime`. The shell turns the IME on only while
//! `ime_area` answers (`app/events/ime.rs`): with no field focused the
//! letters are the gallery's shortcuts, and a Japanese input source would
//! compose them instead.

use comps::prelude::Rect;

use super::{Pane, ShowcaseView};

/// A step of the platform's input method, as the shell forwards it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImeEvent<'a> {
    /// The text being composed and the input method's caret in it, a
    /// byte range (`None`: at its end). Empty, the composition ended.
    Preedit(&'a str, Option<(usize, usize)>),
    /// Text the input method commits: typed, as a key's character is.
    Commit(&'a str),
    /// The input method went away: a composition is dropped.
    Disabled,
}

impl ShowcaseView {
    /// A step of the input method, to the active page: the Input page's
    /// focused field (not beneath an overlay that holds the pointer, as
    /// with `handle_key`), or the Editor. `Disabled` reaches both
    /// wherever they are, so a composition left on a page the person
    /// switched away from never shows again. `true` when a frame is
    /// needed.
    pub fn handle_ime(&mut self, ime: ImeEvent<'_>) -> bool {
        if ime == ImeEvent::Disabled {
            let input = self.panes.input.handle_ime(ime);
            let editor = self.panes.editor.handle_ime(ime);
            return input || editor;
        }
        match self.pane() {
            Some(Pane::Editor) => self.panes.editor.handle_ime(ime),
            Some(Pane::Input) if !self.overlay_holds_pointer() => self.panes.input.handle_ime(ime),
            _ => false,
        }
    }

    /// Where the input method's candidate window goes, in the main
    /// window's logical pixels: the caret of the Input page's focused
    /// field, or the editor's. `None` when nothing on the page takes the
    /// input method; the shell turns it off then.
    pub fn ime_area(&self) -> Option<Rect> {
        match self.pane()? {
            Pane::Editor => Some(self.panes.editor.ime_area()),
            Pane::Input if !self.overlay_holds_pointer() => {
                self.panes.input.ime_area(self.page_rect(), &self.theme)
            }
            _ => None,
        }
    }
}
