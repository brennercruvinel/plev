//! The input method in the shell (task-74 phase 4): `WindowEvent::Ime`
//! goes to the view winit-free (`ShowcaseView::handle_ime`), and on the
//! desktop the main window's IME is on only while the page has a caret for
//! it (`ShowcaseView::ime_area`), its candidate window at that caret. The
//! decision is taken after each frame the main window draws: the focus
//! only moves by something that asks for a frame. The page's own window
//! (`D`) keeps the IME off.
//!
//! A commit is recorded as a typed key is, by its kind and never its text
//! (`record_key`); a preedit is only drawn, so nothing of it is recorded.

use winit::event::Ime;
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
use winit::{
    dpi::{LogicalPosition, LogicalSize},
    keyboard::Key,
    window::WindowId,
};

use super::super::super::App;
use crate::views::ImeEvent;

/// winit's IME event as the view takes it; `Enabled` tells the view
/// nothing.
fn translate(ime: &Ime) -> Option<ImeEvent<'_>> {
    match ime {
        Ime::Enabled => None,
        Ime::Preedit(text, cursor) => Some(ImeEvent::Preedit(text, *cursor)),
        Ime::Commit(text) => Some(ImeEvent::Commit(text)),
        Ime::Disabled => Some(ImeEvent::Disabled),
    }
}

impl App {
    /// A step of the input method, from whichever window has the focus:
    /// a commit is the person typing (recorded, the state after it, a
    /// frame); a preedit or its end is only a frame.
    pub(in crate::shell) fn ime(&mut self, ime: &Ime) {
        let Some(event) = translate(ime) else {
            return;
        };
        let commit = matches!(event, ImeEvent::Commit(text) if !text.is_empty());
        // The key path's recording: a save key goes to nobody else. An
        // input method never commits a bare `s` with cmd and shift held.
        #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
        if let ImeEvent::Commit(text) = event
            && commit
            && self.record_key(&Key::Character(text.into()))
        {
            return;
        }
        if !self.view.handle_ime(event) {
            return;
        }
        if commit {
            self.changed();
        } else {
            self.invalidate();
        }
    }

    /// After a frame of window `id`: the main window's IME on while the
    /// page has a caret for it, with its candidate window there, and off
    /// otherwise (winit keeps a repeated on or off a no-op).
    #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
    pub(in crate::shell) fn sync_ime(&self, id: WindowId) {
        if Some(id) != self.main {
            return;
        }
        let Some(window) = self.windows.get(id).and_then(|host| host.window()) else {
            return;
        };
        let area = self.view.ime_area();
        if let Some(r) = area {
            window.set_ime_cursor_area(LogicalPosition::new(r.x, r.y), LogicalSize::new(r.w, r.h));
        }
        window.set_ime_allowed(area.is_some());
    }
}

#[cfg(test)]
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
mod tests {
    use record::{InputKind, Kind, Limits};
    use accord::{StateRef, Value};
    use winit::event::Ime;

    use super::super::super::super::App;
    use super::super::super::record::Recording;
    use crate::views::{EditKey, ShowcaseView};

    fn text(app: &App) -> Option<Value> {
        let at = StateRef::parse("input_page.field[0]:text").unwrap();
        app.view.semantics()?.value(&at).cloned()
    }

    /// winit's events through the shell: `Enabled` is nothing, a preedit
    /// is only drawn, the commit is typed and recorded by its kind; the
    /// saved session holds one `text` input and never the preedit.
    #[test]
    fn a_commit_is_typed_and_recorded_by_its_kind_and_the_preedit_never() {
        let dir = tempfile::TempDir::new().expect("a temp dir");
        let mut view = ShowcaseView::new(1280.0, 800.0);
        view.jump_to_page("Input");
        let mut app = App::with_bare_window(view);
        app.record(Some(Recording::new(dir.path(), Limits::default())));
        assert!(app.view.handle_edit_key(EditKey::Tab), "a field focused");

        app.ime(&Ime::Enabled);
        app.ime(&Ime::Preedit("にほんご".into(), Some((0, 12))));
        assert_eq!(text(&app), Some(Value::from("")));
        app.ime(&Ime::Preedit(String::new(), None));
        app.ime(&Ime::Commit("日本語".into()));
        assert_eq!(text(&app), Some(Value::from("日本語")));
        app.ime(&Ime::Disabled);

        let (path, _) = app.recording.as_mut().unwrap().save().expect("saved");
        let session = record::Session::open(&path).expect("the session opens");
        let inputs: Vec<InputKind> = session
            .events()
            .iter()
            .filter_map(|e| match &e.kind {
                Kind::Input { kind, .. } => Some(*kind),
                _ => None,
            })
            .collect();
        assert_eq!(inputs, [InputKind::Text], "the commit, and only it");
        let bytes = std::fs::read(&path).expect("the file reads");
        let holds = |text: &str| bytes.windows(text.len()).any(|w| w == text.as_bytes());
        assert!(holds("日本語"), "the state after the commit is readable");
        assert!(!holds("にほんご"), "the preedit is never in the recording");
    }
}
