//! The gallery's menu bar (task-76 item 4), desktop only: built from what
//! the gallery projects (`ShowcaseView::gallery_semantics`), shown by the
//! engine's menu (`engine::runtime::platform::menu`, `muda` underneath), and read
//! back as clicks the view dispatches as a person's requests, the path a
//! key and a button take. The bar: "plev" (the platform's own items),
//! "view" (the shell's capabilities: the theme, the page to go to, the
//! page in a window of its own, the page on paper with cmd-p, the
//! recording's save while the session is recorded) and the active page's
//! title with its plain actions
//! (every action a person may ask without a value; the rows are left
//! out, their actions belong to the row on screen), when it has any.
//!
//! On macOS the bar is the application's, one for every window, so the
//! shell keeps one menu and installs it once the main window is up; the
//! page's own window shows the same page, so the same bar. After every
//! frame of the main window the bar follows the projection (`sync_menu`):
//! a label, an `enabled`, a check mark in place, a page switch shows the
//! bar again. A click wakes the loop (`UserEvent::Menu`), the clicks are
//! polled there and dispatched (`menu_clicks`), and what the shell owns
//! (the windows, the recording) is taken right after
//! (`detach_requested`, `save_requested`).

use std::sync::Mutex;

use accord::{Role, SemId, Snapshot};
use engine::runtime::platform::menu::{
    MenuItem, MenuModel, NativeMenu, Predefined, Shortcut, Submenu, default_menu,
};

use super::super::{App, UserEvent, system::print, window::record};
use crate::views::ShowcaseView;

/// The shell's capabilities by their ids, as the projection names them
/// (`view::dispatch::shell`).
const THEME: &str = "shell.theme";
const PAGE: &str = "shell.page";
const DETACH: &str = "shell.detach";
const PRINT: &str = "shell.print";
const SAVE: &str = "shell.save_session";

/// The bar as shown, and the model it shows.
pub(crate) struct Menu {
    native: Box<dyn NativeMenu>,
    shown: Option<MenuModel>,
}

impl App {
    /// Shows the bar for the first time, on the loop's thread (the main
    /// one, as `muda` needs). A click wakes the loop through the proxy. A
    /// target without a menu is logged and the shell goes on without one.
    pub(crate) fn install_menu(&mut self) {
        let proxy = Mutex::new(self.access.proxy.clone());
        let mut native = default_menu(move || {
            if let Ok(proxy) = proxy.lock()
                && let Some(proxy) = proxy.as_ref()
            {
                let _ = proxy.send_event(UserEvent::Clicked);
            }
        });
        let model = model(&self.view);
        match native.install(&model) {
            Ok(()) => {
                log::info!("menu: {} items in the bar", model.items().count());
                self.menu = Some(Menu {
                    native,
                    shown: Some(model),
                });
            }
            Err(e) => log::info!("menu: none on this target: {e}"),
        }
    }

    /// The bar through `native` (a fake), shown now. For the tests.
    #[cfg(test)]
    pub(super) fn set_menu(&mut self, native: Box<dyn NativeMenu>) {
        self.menu = Some(Menu {
            native,
            shown: None,
        });
        self.sync_menu();
    }

    /// The bar follows the gallery: after a frame, the model is built
    /// again and, when it moved, the bar is updated (shown, when nothing
    /// was yet). The same model costs a projection and nothing else.
    pub(crate) fn sync_menu(&mut self) {
        let Some(menu) = self.menu.as_mut() else {
            return;
        };
        let model = model(&self.view);
        if menu.shown.as_ref() == Some(&model) {
            return;
        }
        let shown = match &menu.shown {
            None => menu.native.install(&model),
            Some(_) => menu.native.update(&model),
        };
        match shown {
            Ok(()) => menu.shown = Some(model),
            Err(e) => log::warn!("menu: not updated: {e}"),
        }
    }

    /// The clicks since the last poll, each dispatched by the view as the
    /// person's request (`ShowcaseView::menu_click`). `true` when the
    /// gallery's projection moved, so a frame is needed. What the shell
    /// owns is left for the caller: `detach_requested`, `save_requested`.
    pub(crate) fn menu_clicks(&mut self) -> bool {
        let Some(menu) = self.menu.as_mut() else {
            return false;
        };
        let mut changed = false;
        for clicked in menu.native.poll() {
            changed |= self.view.menu_click(&clicked);
        }
        changed
    }
}

/// The bar the gallery shows now.
pub(super) fn model(view: &ShowcaseView) -> MenuModel {
    let gallery = match view.gallery_semantics() {
        Ok(gallery) => gallery,
        Err(e) => {
            log::warn!("menu: the gallery does not validate: {e}");
            return MenuModel::new().menu(app_menu());
        }
    };
    let mut model = MenuModel::new().menu(app_menu()).menu(view_menu(&gallery));
    if let Some(page) = page_menu(view, &gallery) {
        model = model.menu(page);
    }
    model
}

/// The platform's own items, what winit's default bar had.
fn app_menu() -> Submenu {
    Submenu::new("plev")
        .predefined(Predefined::About)
        .separator()
        .predefined(Predefined::Hide)
        .predefined(Predefined::HideOthers)
        .predefined(Predefined::ShowAll)
        .separator()
        .predefined(Predefined::Quit)
}

/// The shell's capabilities: the theme and the page as submenus of
/// options, the page in a window of its own as a check item, the page
/// on paper with its keys (disabled without a destination), the
/// recording's save with its keys while the session is recorded. The
/// tray's menu carries the same entries (`tray.rs`).
pub(super) fn view_menu(gallery: &Snapshot) -> Submenu {
    let mut menu = Submenu::new("view");
    if let Ok(theme) = SemId::parse(THEME) {
        let options = MenuItem::options(gallery, &theme, "select");
        menu = menu.submenu(Submenu::new("theme").items(options));
    }
    if let Ok(page) = SemId::parse(PAGE) {
        let options = MenuItem::options(gallery, &page, "select");
        menu = menu.submenu(Submenu::new("go to").items(options));
    }
    menu = menu.separator();
    if let Some(item) = SemId::parse(DETACH)
        .ok()
        .and_then(|id| MenuItem::of(gallery, &id, "toggle"))
    {
        menu = menu.item(item);
    }
    if let Some(item) = SemId::parse(PRINT)
        .ok()
        .and_then(|id| MenuItem::of(gallery, &id, "activate"))
    {
        let item = match Shortcut::parse(print::PRINT_KEYS) {
            Ok(keys) => item.shortcut(keys),
            Err(_) => item,
        };
        menu = menu.item(item);
    }
    if let Some(item) = SemId::parse(SAVE)
        .ok()
        .and_then(|id| MenuItem::of(gallery, &id, "activate"))
    {
        let item = match Shortcut::parse(record::SAVE_KEYS) {
            Ok(keys) => item.shortcut(keys),
            Err(_) => item,
        };
        menu = menu.separator().item(item);
    }
    menu
}

/// The active page's plain actions, under its title; `None` when the
/// page offers none (a page without a projection, or with values only).
/// The toasts and the overlay the gallery projects over the page are left
/// out: they are on screen for a moment, and the bar is the page's. So is
/// a component page's (task-75 phase 2b): its widgets are samples of the
/// design system ("Button" thirty times, a checkbox's toggle), not what
/// the page does; a reader and an agent reach them through the tree and
/// the port.
fn page_menu(view: &ShowcaseView, gallery: &Snapshot) -> Option<Submenu> {
    let items: Vec<MenuItem> = MenuItem::plain(gallery)
        .into_iter()
        .filter(|item| {
            !view.is_shell(&item.sem)
                && !view.is_overlay(&item.sem)
                && !view.is_component(&item.sem)
                && !is_row(gallery, item)
        })
        .map(|item| {
            let label = page_label(gallery, &item);
            item.label(label)
        })
        .collect();
    if items.is_empty() {
        return None;
    }
    Some(Submenu::new(view.page_title()).items(items))
}

/// A row's actions belong to the row on screen, not to the bar.
fn is_row(gallery: &Snapshot, item: &MenuItem) -> bool {
    matches!(
        gallery.get(&item.sem).map(|n| n.role),
        Some(Role::ListItem | Role::Row | Role::Cell | Role::TreeItem | Role::Option)
    )
}

/// How an item of the page reads in the bar: the action's own label when
/// it declares one; a button by its label; a tab by its list and itself
/// ("filter: Active"); anything else by its label and the action ("new
/// todo: submit").
fn page_label(gallery: &Snapshot, item: &MenuItem) -> String {
    let Some(node) = gallery.get(&item.sem) else {
        return item.label.clone();
    };
    let declared = node
        .actions
        .iter()
        .find(|a| a.name == item.action)
        .and_then(|a| a.label.clone());
    if let Some(label) = declared {
        return label;
    }
    match node.role {
        Role::Button => node.label.clone(),
        Role::Tab => {
            let list = node
                .parent
                .as_ref()
                .and_then(|p| gallery.get(p))
                .map(|p| p.label.as_str())
                .unwrap_or_default();
            if list.is_empty() {
                node.label.clone()
            } else {
                format!("{list}: {}", node.label)
            }
        }
        _ => format!("{}: {}", node.label, item.action.as_str().replace('_', " ")),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use accord::{SemId, Value};
    use engine::runtime::platform::menu::{Entry, FakeMenu, MenuItem, MenuModel, Shortcut, locked};

    use super::super::super::{App, window::record};
    use crate::views::{PAGES, ShowcaseView};

    fn sid(text: &str) -> SemId {
        SemId::parse(text).expect("a valid id")
    }

    fn text(value: &str) -> Value {
        Value::Text(value.to_owned())
    }

    /// The shell on `page` with a bare main window and a fake menu, kept
    /// by the test through the shared handle.
    fn app(page: &str) -> (App, Arc<Mutex<FakeMenu>>) {
        let mut view = ShowcaseView::new(1200.0, 800.0);
        view.jump_to_page(page);
        view.resize(1200.0, 800.0, 1.0);
        let mut app = App::with_bare_window(view);
        let fake = Arc::new(Mutex::new(FakeMenu::new()));
        app.set_menu(Box::new(fake.clone()));
        (app, fake)
    }

    fn shown(fake: &Arc<Mutex<FakeMenu>>) -> MenuModel {
        locked(fake).installed().cloned().expect("a bar is shown")
    }

    fn labels(model: &MenuModel) -> Vec<String> {
        model.menus.iter().map(|m| m.label.clone()).collect()
    }

    /// The labels of the checked items of the submenu called `name` of
    /// the "view" menu.
    fn checked_in(model: &MenuModel, name: &str) -> Vec<String> {
        let view = &model.menus[1];
        let sub = view
            .entries
            .iter()
            .find_map(|e| match e {
                Entry::Submenu(s) if s.label == name => Some(s),
                _ => None,
            })
            .unwrap_or_else(|| panic!("a {name} submenu"));
        sub.items_deep()
            .filter(|i| i.checked == Some(true))
            .map(|i| i.label.clone())
            .collect()
    }

    /// A component page projects its widgets (task-75 phase 2b), and the
    /// bar leaves them out: samples of the design system, not what the
    /// page does. The Toggle Group page shows the todo page, and keeps its
    /// bar.
    #[test]
    fn the_bar_leaves_the_component_pages_out() {
        for page in [
            "Button", "Checkbox", "Toast", "Dialog", "Tabs", "Tree", "Input",
        ] {
            let (app, fake) = app(page);
            assert!(app.view.semantics().is_some(), "{page} projects");
            let gallery = app.view.gallery_semantics().unwrap();
            assert!(
                !MenuItem::plain(&gallery).is_empty(),
                "{page} has plain actions"
            );
            assert_eq!(labels(&shown(&fake)), ["plev", "view"], "{page}");
        }
        let (_, fake) = app("Toggle Group");
        let bar = shown(&fake);
        assert_eq!(labels(&bar), ["plev", "view", "Toggle Group"]);
        assert!(
            bar.find(&sid("todo.filter[active]"), "select", None)
                .is_some()
        );
    }

    /// The toasts and the modal the view projects over the page (task-75
    /// phase 2a) are on screen for a moment: the bar stays the page's.
    #[test]
    fn the_bar_leaves_the_toasts_and_the_modal_out() {
        let (mut app, fake) = app("Todo");
        let before = shown(&fake);
        let theme = app.view.theme.clone();
        app.view.toasts.push(
            "saved",
            engine::widgets::prelude::Intent::Constructive,
            &theme,
        );
        app.sync_menu();
        let bar = shown(&fake);
        assert!(bar.find(&sid("toasts[0]"), "dismiss", None).is_none());
        assert_eq!(bar, before, "the same bar");
    }

    #[test]
    fn the_bar_shows_the_shell_and_the_active_page_and_a_click_runs_through_the_view() {
        let (mut app, fake) = app("Todo");
        let bar = shown(&fake);
        assert_eq!(labels(&bar), ["plev", "view", "Todo"]);
        assert_eq!(checked_in(&bar, "theme"), ["core"]);
        assert_eq!(checked_in(&bar, "go to"), ["Todo"]);
        let Entry::Submenu(go) = &bar.menus[1].entries[1] else {
            panic!("the go to submenu");
        };
        assert_eq!(go.items_deep().count(), PAGES.len());
        let detach = bar.find(&sid("shell.detach"), "toggle", None).unwrap();
        assert_eq!(detach.checked, Some(false));
        assert!(detach.enabled);
        assert!(
            bar.find(&sid("shell.save_session"), "activate", None)
                .is_none(),
            "no recording, no save"
        );
        let page: Vec<(String, bool)> = bar.menus[2]
            .items_deep()
            .map(|i| (i.label.clone(), i.enabled))
            .collect();
        assert_eq!(
            page,
            [
                ("new todo: submit".to_owned(), false),
                ("filter: All".to_owned(), true),
                ("filter: Active".to_owned(), true),
                ("filter: Completed".to_owned(), true),
            ],
            "the rows' toggle and remove stay on the rows"
        );

        // a theme picked in the bar: the view dispatches it, and the bar
        // follows on the next sync
        assert!(locked(&fake).click(&sid("shell.theme"), "select", Some(&text("dark"))));
        assert!(app.menu_clicks());
        assert_eq!(app.view.theme_name, "dark");
        app.sync_menu();
        assert_eq!(locked(&fake).installs(), 1);
        assert_eq!(locked(&fake).updates(), 1);
        assert_eq!(checked_in(&shown(&fake), "theme"), ["dark"]);
        app.sync_menu();
        assert_eq!(locked(&fake).updates(), 1, "an unmoved bar is not updated");

        // a page action, the same way; a disabled one takes no click
        assert!(locked(&fake).click(&sid("todo.filter[active]"), "select", None));
        assert!(app.menu_clicks());
        assert_eq!(
            app.view
                .semantics()
                .unwrap()
                .value(&"todo.filter:selected".parse().unwrap()),
            Some(&text("active"))
        );
        assert!(!locked(&fake).click(&sid("todo.input"), "submit", None));

        // the window: the ask waits for the shell, which owns the windows
        assert!(locked(&fake).click(&sid("shell.detach"), "toggle", None));
        app.menu_clicks();
        assert!(app.view.take_detach_request());

        // another page: its menu leaves the bar (the Card page projects
        // nothing), the go to mark moves
        assert!(locked(&fake).click(&sid("shell.page"), "select", Some(&text("Card"))));
        assert!(app.menu_clicks());
        assert_eq!(app.view.page_title(), "Card");
        app.sync_menu();
        let bar = shown(&fake);
        assert_eq!(labels(&bar), ["plev", "view"]);
        assert_eq!(checked_in(&bar, "go to"), ["Card"]);
        assert!(
            !bar.find(&sid("shell.detach"), "toggle", None)
                .unwrap()
                .checked
                .unwrap()
        );

        // the recording's save, with its keys, while the session is recorded
        app.view.set_recording(true);
        app.sync_menu();
        let save = shown(&fake)
            .find(&sid("shell.save_session"), "activate", None)
            .cloned()
            .expect("the save shows");
        assert_eq!(
            save.shortcut,
            Some(Shortcut::parse(record::SAVE_KEYS).unwrap())
        );
        assert!(locked(&fake).click(&sid("shell.save_session"), "activate", None));
        app.menu_clicks();
        assert!(app.view.take_save_request());
        app.view.set_recording(false);
        app.sync_menu();
        assert!(
            shown(&fake)
                .find(&sid("shell.save_session"), "activate", None)
                .is_none()
        );
    }

    #[test]
    fn a_page_with_values_only_and_a_placeholder_show_no_page_menu() {
        let (app, fake) = app("Silences");
        let bar = shown(&fake);
        assert_eq!(labels(&bar), ["plev", "view", "Silences"]);
        let page: Vec<String> = bar.menus[2].items_deep().map(|i| i.label.clone()).collect();
        // the fixed screen's actions, then the proposing row's (the page
        // projects both; the golden keeps the fixed screen alone)
        assert_eq!(
            page,
            [
                "threshold (dB): increment",
                "threshold (dB): decrement",
                "minimum silence (s): increment",
                "minimum silence (s): decrement",
                "detect silences",
                "apply all",
                "proposer: open",
                "proposer: close",
                "propose",
                "cancel",
                "close the surface",
            ]
        );
        drop(app);
        if let Some(placeholder) = PAGES.iter().find(|p| p.pane().is_none()) {
            let (_, fake) = self::app(placeholder.title);
            let bar = shown(&fake);
            assert_eq!(labels(&bar), ["plev", "view"]);
            assert!(
                !bar.find(&sid("shell.detach"), "toggle", None)
                    .unwrap()
                    .enabled
            );
        }
    }
}
