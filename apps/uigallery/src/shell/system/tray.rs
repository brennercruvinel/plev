//! The gallery's tray icon (task-76 item 5), desktop only: built from what
//! the gallery projects (`ShowcaseView::gallery_semantics`), shown by the
//! engine's tray (`engine::platform::tray`, `tray-icon` underneath on
//! macOS and windows, none on linux until its bring-up), and read back as
//! clicks the view dispatches as a person's requests, the path a key, a
//! button and a bar item take. The menu on the icon: "show plev"
//! (`shell.show`, which the click on the icon itself asks too), then the
//! shell's entries as the bar's "view" menu has them (the theme, the page
//! to go to, the page in a window of its own, the recording's save while
//! the session is recorded), then the platform's quit. The active page's
//! actions stay in the bar: the tray is for the app behind its window.
//!
//! The icon is the process's: the shell shows it once the main window is
//! up (the main thread, the loop running, as `tray-icon` needs) and,
//! after every frame of the main window, rebuilds the model and updates
//! it when it moved (`sync_tray`): a mark, an `enabled`, a label in
//! place; the save appearing shows the menu again. A click wakes the loop
//! (`UserEvent::Clicked`), the clicks are polled there and dispatched
//! (`tray_clicks`), and what the shell owns is taken right after
//! (`show_requested`, `detach_requested`, `save_requested`).

use std::sync::Mutex;

use engine::runtime::platform::menu::{MenuItem, Predefined, Submenu};
use engine::runtime::platform::tray::{Tray, TrayImage, TrayModel, default_tray};
use accord::SemId;

use super::menu::view_menu;
use super::super::{App, UserEvent};
use crate::views::ShowcaseView;

/// The shell's capability the icon asks for, as the projection names it
/// (`view::dispatch::shell`).
const SHOW: &str = "shell.show";

/// The mark's side in pixels: the bar shows it 18 points tall, so twice
/// that is crisp on a two-times display.
const MARK: u32 = 36;

/// The tray as shown, and the model it shows.
pub(super) struct TrayIcon {
    native: Box<dyn Tray>,
    shown: Option<TrayModel>,
}

impl App {
    /// Shows the tray for the first time, on the loop's thread (the main
    /// one, as `tray-icon` needs). A click wakes the loop through the
    /// proxy. A target without a tray is logged and the shell goes on
    /// without one.
    pub(super) fn install_tray(&mut self) {
        let proxy = Mutex::new(self.access.proxy.clone());
        let mut native = default_tray(move || {
            if let Ok(proxy) = proxy.lock()
                && let Some(proxy) = proxy.as_ref()
            {
                let _ = proxy.send_event(UserEvent::Clicked);
            }
        });
        let model = model(&self.view);
        match native.install(&model) {
            Ok(()) => {
                log::info!("tray: {} items in the menu", model.items().count());
                self.tray = Some(TrayIcon {
                    native,
                    shown: Some(model),
                });
            }
            Err(e) => log::info!("tray: none on this target: {e}"),
        }
    }

    /// The tray through `native` (a fake), shown now. For the tests.
    #[cfg(test)]
    pub(super) fn set_tray(&mut self, native: Box<dyn Tray>) {
        self.tray = Some(TrayIcon {
            native,
            shown: None,
        });
        self.sync_tray();
    }

    /// The tray follows the gallery: after a frame, the model is built
    /// again and, when it moved, the tray is updated (shown, when nothing
    /// was yet). The same model costs a projection and nothing else.
    pub(super) fn sync_tray(&mut self) {
        let Some(tray) = self.tray.as_mut() else {
            return;
        };
        let model = model(&self.view);
        if tray.shown.as_ref() == Some(&model) {
            return;
        }
        let shown = match &tray.shown {
            None => tray.native.install(&model),
            Some(_) => tray.native.update(&model),
        };
        match shown {
            Ok(()) => tray.shown = Some(model),
            Err(e) => log::warn!("tray: not updated: {e}"),
        }
    }

    /// The clicks since the last poll (on the menu and on the icon), each
    /// dispatched by the view as the person's request
    /// (`ShowcaseView::menu_click`). `true` when the gallery's projection
    /// moved, so a frame is needed. What the shell owns is left for the
    /// caller: `show_requested`, `detach_requested`, `save_requested`.
    pub(super) fn tray_clicks(&mut self) -> bool {
        let Some(tray) = self.tray.as_mut() else {
            return false;
        };
        let mut changed = false;
        for clicked in tray.native.poll() {
            changed |= self.view.menu_click(&clicked);
        }
        changed
    }
}

/// The tray the gallery shows now: the mark, "plev" as the tooltip, the
/// show item (and the icon's own click) when the shell projects it, the
/// shell's entries, the platform's quit.
pub(super) fn model(view: &ShowcaseView) -> TrayModel {
    let model = TrayModel::new(TrayImage::mark(MARK), "plev");
    let gallery = match view.gallery_semantics() {
        Ok(gallery) => gallery,
        Err(e) => {
            log::warn!("tray: the gallery does not validate: {e}");
            return model.menu(Submenu::new("plev").predefined(Predefined::Quit));
        }
    };
    let mut model = model;
    let mut menu = Submenu::new("plev");
    if let Some(show) = SemId::parse(SHOW)
        .ok()
        .and_then(|id| MenuItem::of(&gallery, &id, "activate"))
    {
        model = model.on_click(show.clicked());
        menu = menu.item(show).separator();
    }
    menu.entries.extend(view_menu(&gallery).entries);
    model.menu(menu.separator().predefined(Predefined::Quit))
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use engine::runtime::platform::menu::{Entry, locked};
    use engine::runtime::platform::tray::{FakeTray, TrayModel};
    use accord::{SemId, Value};

    use super::super::super::App;
    use crate::views::{PAGES, ShowcaseView};

    fn sid(text: &str) -> SemId {
        SemId::parse(text).expect("a valid id")
    }

    fn text(value: &str) -> Value {
        Value::Text(value.to_owned())
    }

    /// The shell on `page` with a bare main window and a fake tray, kept
    /// by the test through the shared handle.
    fn app(page: &str) -> (App, Arc<Mutex<FakeTray>>) {
        let mut view = ShowcaseView::new(1200.0, 800.0);
        view.jump_to_page(page);
        view.resize(1200.0, 800.0, 1.0);
        let mut app = App::with_bare_window(view);
        let fake = Arc::new(Mutex::new(FakeTray::new()));
        app.set_tray(Box::new(fake.clone()));
        (app, fake)
    }

    fn shown(fake: &Arc<Mutex<FakeTray>>) -> TrayModel {
        locked(fake).installed().cloned().expect("a tray is shown")
    }

    /// The labels of the checked items of the submenu called `name`.
    fn checked_in(model: &TrayModel, name: &str) -> Vec<String> {
        model
            .menu
            .entries
            .iter()
            .find_map(|e| match e {
                Entry::Submenu(s) if s.label == name => Some(s),
                _ => None,
            })
            .unwrap_or_else(|| panic!("a {name} submenu"))
            .items_deep()
            .filter(|i| i.checked == Some(true))
            .map(|i| i.label.clone())
            .collect()
    }

    #[test]
    fn the_tray_shows_the_shell_and_its_clicks_run_through_the_view() {
        let (mut app, fake) = app("Todo");
        let tray = shown(&fake);
        assert_eq!(tray.tooltip, "plev");
        assert_eq!(tray.icon.width, 36);
        assert!(tray.icon.template);

        // the show item first, and the icon's own click asks the same
        let Entry::Item(show) = &tray.menu.entries[0] else {
            panic!("the show item first: {:?}", tray.menu.entries[0]);
        };
        assert_eq!(show.sem, sid("shell.show"));
        assert_eq!(show.label, "show plev");
        assert!(show.enabled);
        assert_eq!(tray.on_click.as_ref(), Some(&show.clicked()));
        assert!(matches!(tray.menu.entries[1], Entry::Separator));
        assert_eq!(checked_in(&tray, "theme"), ["core"]);
        assert_eq!(checked_in(&tray, "go to"), ["Todo"]);
        let Entry::Submenu(go) = &tray.menu.entries[3] else {
            panic!("the go to submenu");
        };
        assert_eq!(go.items_deep().count(), PAGES.len());
        assert!(tray.find(&sid("shell.detach"), "toggle", None).is_some());
        assert!(
            tray.find(&sid("shell.save_session"), "activate", None)
                .is_none(),
            "no recording, no save"
        );
        assert!(
            tray.find(&sid("todo.filter[active]"), "select", None)
                .is_none(),
            "the page's actions stay in the bar"
        );
        assert!(matches!(
            tray.menu.entries.last(),
            Some(Entry::Predefined(engine::runtime::platform::menu::Predefined::Quit))
        ));

        // the icon clicked: the shell is asked to show the window
        assert!(!app.view.take_show_request());
        assert!(locked(&fake).click_icon());
        assert!(!app.tray_clicks(), "showing moves no projection");
        assert!(app.view.take_show_request());
        assert!(!app.view.take_show_request(), "taken once");

        // a theme picked in the tray: the view dispatches it, the tray
        // follows on the next sync, in place
        assert!(locked(&fake).click(&sid("shell.theme"), "select", Some(&text("dark"))));
        assert!(app.tray_clicks());
        assert_eq!(app.view.theme_name, "dark");
        app.sync_tray();
        assert_eq!(locked(&fake).installs(), 1);
        assert_eq!(locked(&fake).updates(), 1);
        assert_eq!(checked_in(&shown(&fake), "theme"), ["dark"]);
        app.sync_tray();
        assert_eq!(locked(&fake).updates(), 1, "an unmoved tray is not updated");

        // another page: the same shape (the page's menu is the bar's),
        // the go to mark moves
        assert!(locked(&fake).click(&sid("shell.page"), "select", Some(&text("Card"))));
        assert!(app.tray_clicks());
        assert_eq!(app.view.page_title(), "Card");
        app.sync_tray();
        let moved = shown(&fake);
        assert!(tray.same_shape(&moved));
        assert_eq!(checked_in(&moved, "go to"), ["Card"]);
        assert_eq!(locked(&fake).installs(), 1, "the same icon, its menu moved");

        // the window item, as in the bar: the ask waits for the shell
        assert!(locked(&fake).click(&sid("shell.detach"), "toggle", None));
        app.tray_clicks();
        assert!(app.view.take_detach_request());

        // the recording's save appears while the session is recorded
        app.view.set_recording(true);
        app.sync_tray();
        assert!(
            shown(&fake)
                .find(&sid("shell.save_session"), "activate", None)
                .is_some()
        );
        assert!(locked(&fake).click(&sid("shell.save_session"), "activate", None));
        app.tray_clicks();
        assert!(app.view.take_save_request());
        app.view.set_recording(false);
        app.sync_tray();
        assert!(
            shown(&fake)
                .find(&sid("shell.save_session"), "activate", None)
                .is_none()
        );
    }
}
