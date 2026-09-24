//! cmd+click to the code, in a desktop debug build (where the engine has
//! `hot_reload`; the parent gates this module on the same terms). A left
//! click with cmd held (ctrl off macOS) never reaches the page, press or
//! release: it asks `engine::hot_reload::view_source::origin_at` what made
//! the pixel under the cursor in the frame on screen, and opens the one
//! site it names in the editor (`PLEV_EDITOR`, else zed, else `code -g`).
//! Anything else is logged at warn, the level the shell shows: no control
//! and no text there, a control with no source, a text several literals
//! hold (each listed), a formatted text. It draws nothing and asks for no
//! frame, so cmd alone or cmd+click changes nothing on screen.

use engine::hot_reload::Site;
use engine::hot_reload::view_source::{self, origin_at};
use winit::event::ElementState;
use winit::keyboard::ModifiersState;

use super::App;

impl App {
    /// `true` when this left click is a view-source click, press and its
    /// release both: the caller hands it to nobody else.
    pub(super) fn view_source(&mut self, state: ElementState) -> bool {
        self.view_source_with(state, view_source::open)
    }

    /// [`view_source`](Self::view_source) with the opener given: the
    /// window opens the editor; a test keeps the site.
    fn view_source_with(
        &mut self,
        state: ElementState,
        open: impl FnOnce(&Site) -> std::io::Result<String>,
    ) -> bool {
        match state {
            ElementState::Pressed if primary(self.modifiers) => {
                self.source_click = true;
                let (x, y) = self.cursor;
                let snapshot = self.view.semantics();
                let origin = origin_at(&self.compositor, snapshot.as_ref(), x, y);
                match origin.site().map(|site| (site, open(site))) {
                    Some((_, Ok(ran))) => log::info!("view source: {origin}; ran {ran}"),
                    Some((_, Err(e))) => log::warn!("view source: {origin}; {e}"),
                    None => log::warn!("view source: {origin}"),
                }
                true
            }
            ElementState::Pressed => false,
            ElementState::Released => std::mem::take(&mut self.source_click),
        }
    }
}

/// The view-source modifier: cmd on macOS, ctrl elsewhere (the super key
/// belongs to the window manager there).
fn primary(modifiers: ModifiersState) -> bool {
    if cfg!(target_os = "macos") {
        modifiers.super_key()
    } else {
        modifiers.control_key()
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use comps::prelude::WidgetEvent;
    use sem::{SemId, Value};

    use super::*;
    use crate::view::ShowcaseView;

    const PRIMARY: ModifiersState = if cfg!(target_os = "macos") {
        ModifiersState::SUPER
    } else {
        ModifiersState::CONTROL
    };

    /// The shell on the todo page with one frame drawn, the cursor on the
    /// first item's row (its checkbox side).
    fn todo() -> App {
        let mut view = ShowcaseView::new(1200.0, 800.0);
        view.jump_to_page("Todo");
        view.resize(1200.0, 800.0, 1.0);
        let mut app = App::with_view(view);
        app.view.render(&mut app.compositor);
        let item = SemId::parse("todo.item[1]").unwrap();
        let [x, y, _, h] = app.compositor.binding(&item).unwrap().rect;
        app.cursor = (x + 19.0, y + h / 2.0);
        app
    }

    fn completed(app: &App) -> Value {
        let item = SemId::parse("todo.item[1]").unwrap();
        let snap = app.view.semantics().unwrap();
        snap.get(&item).unwrap().state["completed"].clone()
    }

    #[test]
    fn a_primary_click_opens_the_projection_and_never_reaches_the_page() {
        let mut app = todo();
        let before = completed(&app);
        app.modifiers = PRIMARY;
        let opened = RefCell::new(Vec::new());
        let open = |site: &Site| {
            opened.borrow_mut().push(site.clone());
            Ok(String::new())
        };
        assert!(app.view_source_with(ElementState::Pressed, open));
        // cmd let go before the button: the release is still taken
        app.modifiers = ModifiersState::empty();
        assert!(app.view_source_with(ElementState::Released, |_| unreachable!()));
        assert_eq!(completed(&app), before, "the item was not toggled");

        let opened = opened.into_inner();
        assert_eq!(opened.len(), 1);
        let site = &opened[0];
        assert!(
            site.file
                .ends_with("crates/showcase/src/view/apps/app/semantics.rs")
        );
        let src = std::fs::read_to_string(&site.file).unwrap();
        let line = src.lines().nth(site.line as usize - 1).unwrap();
        assert!(line.contains("b.node(&id, Role::ListItem"), "{line}");
    }

    #[test]
    fn a_plain_click_is_the_page_s() {
        let mut app = todo();
        let before = completed(&app);
        assert!(!app.view_source_with(ElementState::Pressed, |_| unreachable!()));
        assert!(!app.view_source_with(ElementState::Released, |_| unreachable!()));
        // what the shell then does with it: the page toggles the item
        let (x, y) = app.cursor;
        app.view.handle_event(&WidgetEvent::MouseDown { x, y });
        app.view.handle_event(&WidgetEvent::MouseUp { x, y });
        assert_ne!(completed(&app), before);
    }

    #[test]
    fn a_primary_click_on_nothing_opens_nothing() {
        let mut app = todo();
        app.modifiers = PRIMARY;
        // the margin between the sidebar and the page title
        app.cursor = (270.0, 20.0);
        assert!(app.view_source_with(ElementState::Pressed, |_| unreachable!()));
        assert!(app.view_source_with(ElementState::Released, |_| unreachable!()));
    }
}
