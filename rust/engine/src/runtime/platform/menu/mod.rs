//! Native menus as another view of an app's capabilities. A menu item is
//! not a command of its own: it names a node of the app's projection
//! (`accord::Snapshot`) and one of its actions, and a click on it becomes the
//! same `accord::Request` by `Actor::Human` that a click on the button does,
//! through the app's one dispatcher. The app decides which capabilities
//! enter the menu and under which submenu ([`MenuModel`]); the platform
//! only shows them ([`NativeMenu`]) and hands the clicks back
//! ([`Clicked`]). When the projection moves, the app builds the model
//! again and the menu follows: a label, an `enabled`, a check mark, and
//! an item whose capability is gone leaves.
//!
//! The same shape as `files.rs`: one trait, the real one under the desktop
//! cfg ([`MudaMenu`], `native.rs`), [`FakeMenu`] everywhere for the tests
//! (it keeps the model installed and takes injected clicks), [`NoMenu`] on
//! wasm and mobile, [`default_menu`] choosing.
//!
//! What is per app and what is per window is the platform's: on macOS the
//! menu bar is the application's (`NSApp.mainMenu`), one for every window;
//! on windows a bar belongs to a window (`init_for_hwnd`); linux has no
//! backend until its bring-up picks one. An app that shows the same
//! capabilities in every window installs one model; the shell that owns
//! the windows is where the menu lives, beside the state it projects.

use std::collections::VecDeque;
use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use accord::{ActionName, Actor, Request, Role, SemError, SemId, Snapshot, StateKey, Value};

/// The `muda` side; the tray (`platform::tray`) builds its menu with it.
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
pub(in crate::runtime::platform) mod native;
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
pub use native::MudaMenu;

/// A keyboard shortcut shown beside a menu item and installed as its key
/// equivalent. `primary` is cmd on macOS and ctrl elsewhere. A shortcut
/// always has a modifier: a bare letter as a key equivalent would take the
/// letter away from every text field of the window.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Shortcut {
    pub primary: bool,
    pub shift: bool,
    pub alt: bool,
    pub ctrl: bool,
    /// An ASCII letter or digit, lowercase.
    pub key: char,
}

impl Shortcut {
    /// `primary` and `key`: `cmd-s` on macOS, `ctrl-s` elsewhere.
    pub fn primary(key: char) -> Result<Shortcut, MenuError> {
        Shortcut::parse(&format!("cmd-{key}"))
    }

    /// Parses `cmd-shift-s`: modifiers first (`cmd`, `command`, `primary`,
    /// `ctrl`, `control`, `shift`, `alt`, `option`), one key last, `-` or
    /// `+` between them. `cmd` reads as the primary modifier. At least one
    /// modifier, and a letter or a digit for the key.
    pub fn parse(text: &str) -> Result<Shortcut, MenuError> {
        let bad = || MenuError::BadShortcut(text.to_owned());
        let mut out = Shortcut {
            primary: false,
            shift: false,
            alt: false,
            ctrl: false,
            key: ' ',
        };
        let tokens: Vec<&str> = text
            .split(['-', '+'])
            .map(str::trim)
            .filter(|t| !t.is_empty())
            .collect();
        let (key, mods) = tokens.split_last().ok_or_else(bad)?;
        let mut chars = key.chars();
        match (chars.next(), chars.next()) {
            (Some(c), None) if c.is_ascii_alphanumeric() => out.key = c.to_ascii_lowercase(),
            _ => return Err(bad()),
        }
        for m in mods {
            match m.to_ascii_lowercase().as_str() {
                "cmd" | "command" | "primary" => out.primary = true,
                "ctrl" | "control" => out.ctrl = true,
                "shift" => out.shift = true,
                "alt" | "option" => out.alt = true,
                _ => return Err(bad()),
            }
        }
        if !(out.primary || out.ctrl || out.alt) {
            return Err(bad());
        }
        Ok(out)
    }

    /// The shortcut with shift held too.
    pub fn with_shift(mut self) -> Shortcut {
        self.shift = true;
        self
    }
}

/// `cmd-shift-s` on macOS, `ctrl-shift-s` elsewhere: the modifiers in the
/// order primary, ctrl, alt, shift, then the key.
impl fmt::Display for Shortcut {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.primary {
            f.write_str(if cfg!(target_os = "macos") {
                "cmd-"
            } else {
                "ctrl-"
            })?;
        }
        if self.ctrl {
            f.write_str("ctrl-")?;
        }
        if self.alt {
            f.write_str("alt-")?;
        }
        if self.shift {
            f.write_str("shift-")?;
        }
        write!(f, "{}", self.key)
    }
}

/// One item: a capability of the projection, as the menu shows it. The
/// item is the pair (node, action) and, for an option of a select, the
/// value it asks for; the label, the shortcut, `enabled` and the check
/// mark are how it shows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuItem {
    pub sem: SemId,
    pub action: ActionName,
    pub param: Option<Value>,
    pub label: String,
    pub shortcut: Option<Shortcut>,
    pub enabled: bool,
    /// `Some` for an item with a check mark (a toggle's `on`, a checkbox's
    /// `checked`, the selected option of a select): on or off.
    pub checked: Option<bool>,
}

impl MenuItem {
    /// `action` of the node `sem` in `snapshot`, as the menu shows it: the
    /// action's label or the node's, enabled when the node and the action
    /// are and the node is not inert, a check mark from a toggle's `on` or
    /// a checkbox's `checked`. `None` when the node is not there or does
    /// not declare the action.
    pub fn of(snapshot: &Snapshot, sem: &SemId, action: &str) -> Option<MenuItem> {
        let node = snapshot.get(sem)?;
        let action = ActionName::parse(action).ok()?;
        let decl = node.actions.iter().find(|a| a.name == action)?;
        let checked = match node.role {
            Role::Toggle => bool_state(node, "on"),
            Role::Checkbox => bool_state(node, "checked"),
            _ => None,
        };
        Some(MenuItem {
            sem: sem.clone(),
            action,
            param: None,
            label: decl.label.clone().unwrap_or_else(|| node.label.clone()),
            shortcut: None,
            enabled: node.enabled && decl.enabled && !node.inert,
            checked,
        })
    }

    /// One item per accepted option of `sem` (its `constraints.options`),
    /// each asking `action` with the option as its text param, checked on
    /// the one `selected`. Empty when the node is not there, declares no
    /// such action or has no options.
    pub fn options(snapshot: &Snapshot, sem: &SemId, action: &str) -> Vec<MenuItem> {
        let Some(base) = MenuItem::of(snapshot, sem, action) else {
            return Vec::new();
        };
        let Some(node) = snapshot.get(sem) else {
            return Vec::new();
        };
        let selected = text_state(node, "selected");
        node.constraints
            .options
            .iter()
            .flatten()
            .map(|option| MenuItem {
                param: Some(Value::Text(option.clone())),
                label: option.clone(),
                checked: Some(selected.as_deref() == Some(option.as_str())),
                ..base.clone()
            })
            .collect()
    }

    /// Every action without a parameter that a person may ask for, in
    /// projection order, each as [`MenuItem::of`] shows it. The app keeps
    /// the ones it wants (a row's actions belong to the row on screen).
    pub fn plain(snapshot: &Snapshot) -> Vec<MenuItem> {
        let mut out = Vec::new();
        for (id, node) in snapshot.nodes() {
            for decl in &node.actions {
                if decl.param.is_none()
                    && decl.policy.actors.human
                    && let Some(item) = MenuItem::of(snapshot, id, decl.name.as_str())
                {
                    out.push(item);
                }
            }
        }
        out
    }

    pub fn label(mut self, label: impl Into<String>) -> MenuItem {
        self.label = label.into();
        self
    }

    pub fn shortcut(mut self, shortcut: Shortcut) -> MenuItem {
        self.shortcut = Some(shortcut);
        self
    }

    /// What a click asks the dispatcher for.
    pub fn clicked(&self) -> Clicked {
        Clicked {
            sem: self.sem.clone(),
            action: self.action.clone(),
            param: self.param.clone(),
        }
    }

    /// The text that names this item apart from every other of the same
    /// menu: the node, the action and the value asked for.
    pub fn key(&self) -> String {
        match &self.param {
            Some(param) => format!("{}#{}#{}", self.sem, self.action, param_text(param)),
            None => format!("{}#{}", self.sem, self.action),
        }
    }
}

fn bool_state(node: &accord::Node, key: &str) -> Option<bool> {
    match node.state.get(&StateKey::parse(key).ok()?) {
        Some(Value::Bool(on)) => Some(*on),
        _ => None,
    }
}

fn text_state(node: &accord::Node, key: &str) -> Option<String> {
    match node.state.get(&StateKey::parse(key).ok()?) {
        Some(Value::Text(text)) => Some(text.clone()),
        _ => None,
    }
}

/// A value as one line of text, for an item's key.
fn param_text(value: &Value) -> String {
    match value {
        Value::Bool(b) => b.to_string(),
        Value::Int(i) => i.to_string(),
        Value::Num(n) => n.to_string(),
        Value::Text(t) => t.clone(),
        Value::List(items) => items.join(","),
        Value::Set(items) => items.iter().cloned().collect::<Vec<_>>().join(","),
    }
}

/// An item the platform runs itself, in the platform's words: the about
/// panel, hiding, quitting. It names no capability and sends no click.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Predefined {
    About,
    Hide,
    HideOthers,
    ShowAll,
    Quit,
}

/// One entry of a submenu.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Entry {
    Item(MenuItem),
    Separator,
    Predefined(Predefined),
    Submenu(Submenu),
}

/// A titled list of entries: one of the bar's menus, or a menu inside
/// another.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Submenu {
    pub label: String,
    pub entries: Vec<Entry>,
}

impl Submenu {
    pub fn new(label: impl Into<String>) -> Submenu {
        Submenu {
            label: label.into(),
            entries: Vec::new(),
        }
    }

    pub fn item(mut self, item: MenuItem) -> Submenu {
        self.entries.push(Entry::Item(item));
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = MenuItem>) -> Submenu {
        self.entries.extend(items.into_iter().map(Entry::Item));
        self
    }

    pub fn separator(mut self) -> Submenu {
        self.entries.push(Entry::Separator);
        self
    }

    pub fn predefined(mut self, item: Predefined) -> Submenu {
        self.entries.push(Entry::Predefined(item));
        self
    }

    pub fn submenu(mut self, submenu: Submenu) -> Submenu {
        self.entries.push(Entry::Submenu(submenu));
        self
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Every item in this submenu and below, depth first, in order.
    pub fn items_deep(&self) -> impl Iterator<Item = &MenuItem> {
        let mut out = Vec::new();
        collect_items(self, &mut out);
        out.into_iter()
    }

    /// True when the two have the same label and the same entries in the
    /// same places, items by key, shortcut and kind (checked or not),
    /// submenus recursively: what differs then is only a label, an
    /// `enabled` or a check mark, which a native menu changes in place.
    pub fn same_shape(&self, other: &Submenu) -> bool {
        self.label == other.label
            && self.entries.len() == other.entries.len()
            && self
                .entries
                .iter()
                .zip(&other.entries)
                .all(|(a, b)| match (a, b) {
                    (Entry::Item(a), Entry::Item(b)) => {
                        a.key() == b.key()
                            && a.shortcut == b.shortcut
                            && a.checked.is_some() == b.checked.is_some()
                    }
                    (Entry::Separator, Entry::Separator) => true,
                    (Entry::Predefined(a), Entry::Predefined(b)) => a == b,
                    (Entry::Submenu(a), Entry::Submenu(b)) => a.same_shape(b),
                    _ => false,
                })
    }
}

fn collect_items<'a>(submenu: &'a Submenu, out: &mut Vec<&'a MenuItem>) {
    for entry in &submenu.entries {
        match entry {
            Entry::Item(item) => out.push(item),
            Entry::Submenu(inner) => collect_items(inner, out),
            Entry::Separator | Entry::Predefined(_) => {}
        }
    }
}

/// The bar: its menus in order. Built by the app from its projection;
/// pure data, the same on every target.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MenuModel {
    pub menus: Vec<Submenu>,
}

impl MenuModel {
    pub fn new() -> MenuModel {
        MenuModel::default()
    }

    pub fn menu(mut self, submenu: Submenu) -> MenuModel {
        self.menus.push(submenu);
        self
    }

    /// Every item of the bar, depth first, in order.
    pub fn items(&self) -> impl Iterator<Item = &MenuItem> {
        self.menus.iter().flat_map(Submenu::items_deep)
    }

    /// The item that asks `action` of `sem` with `param`.
    pub fn find(&self, sem: &SemId, action: &str, param: Option<&Value>) -> Option<&MenuItem> {
        self.items()
            .find(|i| i.sem == *sem && i.action.as_str() == action && i.param.as_ref() == param)
    }

    /// The bar's menus by [`Submenu::same_shape`], pairwise.
    pub fn same_shape(&self, other: &MenuModel) -> bool {
        self.menus.len() == other.menus.len()
            && self
                .menus
                .iter()
                .zip(&other.menus)
                .all(|(a, b)| a.same_shape(b))
    }
}

/// A person clicked an item: the capability it names. The app turns it
/// into a request over the projection it shows now ([`Clicked::request`])
/// and dispatches it as it dispatches a click on the button.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Clicked {
    pub sem: SemId,
    pub action: ActionName,
    pub param: Option<Value>,
}

impl Clicked {
    /// The request a person makes by this click, having seen `seen`: the
    /// same `Request::observe` by `Actor::Human` a click on the button
    /// makes, so the dispatcher checks it the same way (stale, inert,
    /// disabled, the policy).
    pub fn request(&self, seen: &Snapshot) -> Result<Request, SemError> {
        Request::observe(
            seen,
            Actor::Human,
            &self.sem,
            &self.action,
            self.param.clone(),
        )
    }
}

/// Why a menu could not be shown.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MenuError {
    /// This target has no native menu (wasm, android, ios).
    Unsupported,
    /// A shortcut text that does not parse.
    BadShortcut(String),
    /// The platform refused (a muda error, in its words).
    Platform(String),
}

impl fmt::Display for MenuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MenuError::Unsupported => f.write_str("no native menu on this target"),
            MenuError::BadShortcut(text) => write!(f, "not a shortcut: {text:?}"),
            MenuError::Platform(why) => write!(f, "the platform refused the menu: {why}"),
        }
    }
}

impl std::error::Error for MenuError {}

/// The platform's menu. `install` shows `model` as the bar (replacing
/// one shown before); `update` makes the bar follow a model that moved
/// (labels, `enabled`, check marks in place; a changed shape is shown
/// again); `poll` hands back the clicks since the last poll, in order,
/// for the app to dispatch on its loop, before the frame.
pub trait NativeMenu {
    fn install(&mut self, model: &MenuModel) -> Result<(), MenuError>;
    fn update(&mut self, model: &MenuModel) -> Result<(), MenuError>;
    fn poll(&mut self) -> Vec<Clicked>;
}

// -- the fake ---------------------------------------------------------------

/// The menu of a test: keeps the model installed, counts the installs and
/// the updates, and takes injected clicks ([`FakeMenu::click`]), which
/// `poll` hands back as the platform would.
#[derive(Debug, Default)]
pub struct FakeMenu {
    model: Option<MenuModel>,
    installs: usize,
    updates: usize,
    clicks: VecDeque<Clicked>,
}

impl FakeMenu {
    pub fn new() -> FakeMenu {
        FakeMenu::default()
    }

    pub fn installs(&self) -> usize {
        self.installs
    }

    pub fn updates(&self) -> usize {
        self.updates
    }

    /// A person clicks the item asking `action` of `sem` with `param`:
    /// queued for `poll`. `false`, and nothing queued, when the menu shown
    /// has no such item or it is disabled (a disabled item takes no
    /// click on the platform either).
    pub fn click(&mut self, sem: &SemId, action: &str, param: Option<&Value>) -> bool {
        let Some(item) = self.model.as_ref().and_then(|m| m.find(sem, action, param)) else {
            return false;
        };
        if !item.enabled {
            return false;
        }
        self.clicks.push_back(item.clicked());
        true
    }

    /// Clicks taken and not polled yet.
    pub fn pending(&self) -> usize {
        self.clicks.len()
    }

    /// The model shown now, if any.
    pub fn installed(&self) -> Option<&MenuModel> {
        self.model.as_ref()
    }
}

impl NativeMenu for FakeMenu {
    fn install(&mut self, model: &MenuModel) -> Result<(), MenuError> {
        self.model = Some(model.clone());
        self.installs += 1;
        Ok(())
    }

    fn update(&mut self, model: &MenuModel) -> Result<(), MenuError> {
        if self.model.as_ref() == Some(model) {
            return Ok(());
        }
        self.model = Some(model.clone());
        self.updates += 1;
        Ok(())
    }

    fn poll(&mut self) -> Vec<Clicked> {
        self.clicks.drain(..).collect()
    }
}

// -- no menu at all ---------------------------------------------------------

/// The menu of a target without one: wasm and mobile. Installing is
/// `Unsupported`, nothing is shown and nothing is ever clicked; the app's
/// capabilities stay where they are, on screen.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoMenu;

impl NativeMenu for NoMenu {
    fn install(&mut self, _model: &MenuModel) -> Result<(), MenuError> {
        Err(MenuError::Unsupported)
    }

    fn update(&mut self, _model: &MenuModel) -> Result<(), MenuError> {
        Err(MenuError::Unsupported)
    }

    fn poll(&mut self) -> Vec<Clicked> {
        Vec::new()
    }
}

/// A menu shared by handle: a test keeps one `Arc` and gives the shell
/// the other as its `Box<dyn NativeMenu>`, then clicks through its own.
/// A poisoned lock still answers: the menu holds no invariant a panic
/// elsewhere could break.
impl<M: NativeMenu> NativeMenu for Arc<Mutex<M>> {
    fn install(&mut self, model: &MenuModel) -> Result<(), MenuError> {
        locked(self).install(model)
    }

    fn update(&mut self, model: &MenuModel) -> Result<(), MenuError> {
        locked(self).update(model)
    }

    fn poll(&mut self) -> Vec<Clicked> {
        locked(self).poll()
    }
}

/// The lock of a shared menu, poisoned or not.
pub fn locked<M>(shared: &Mutex<M>) -> MutexGuard<'_, M> {
    shared.lock().unwrap_or_else(PoisonError::into_inner)
}

/// The menu of the current target, woken by `wake` on every click (from
/// the platform's thread; a shell sends itself an event through its loop
/// proxy and polls on the loop): native on the desktop, none on wasm and
/// mobile.
pub fn default_menu(wake: impl Fn() + Send + Sync + 'static) -> Box<dyn NativeMenu> {
    #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
    {
        Box::new(MudaMenu::new(wake))
    }
    #[cfg(any(target_arch = "wasm32", target_os = "android", target_os = "ios"))]
    {
        let _ = wake;
        Box::new(NoMenu)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use accord::{Constraints, Policy, SnapshotBuilder};

    fn sid(text: &str) -> SemId {
        SemId::parse(text).unwrap()
    }

    /// A small app: a toggle, a select and a button, one of them disabled.
    fn snapshot(on: bool, theme: &str, can_save: bool) -> Snapshot {
        let mut b = SnapshotBuilder::new();
        b.node("shell", Role::Region, "plev");
        b.node("shell.detach", Role::Toggle, "page in a window of its own")
            .parent("shell")
            .state("on", on)
            .action("toggle")
            .reads("shell.detach:on");
        b.node("shell.theme", Role::Select, "theme")
            .parent("shell")
            .state("selected", theme)
            .constraints(Constraints {
                editable: true,
                options: Some(vec!["core".into(), "hoff".into(), "dark".into()]),
                ..Constraints::default()
            })
            .action("select")
            .param(accord::ParamSpec {
                kind: accord::ValueKind::Text,
                constrained: true,
            })
            .reads("shell.theme:selected");
        b.node("shell.save", Role::Button, "save the session")
            .parent("shell")
            .enabled(can_save)
            .action("activate")
            .policy(Policy {
                actors: accord::Actors {
                    agent: false,
                    ..accord::Actors::default()
                },
                ..Policy::default()
            });
        b.node("shell.agent_only", Role::Button, "for agents")
            .parent("shell")
            .action("activate")
            .policy(Policy {
                actors: accord::Actors {
                    human: false,
                    ..accord::Actors::default()
                },
                ..Policy::default()
            });
        b.finish().unwrap()
    }

    #[test]
    fn a_shortcut_parses_with_a_modifier_and_prints_for_the_platform() {
        let save = Shortcut::parse("cmd-shift-s").unwrap();
        assert!(save.primary && save.shift && !save.alt && !save.ctrl);
        assert_eq!(save.key, 's');
        let shown = save.to_string();
        if cfg!(target_os = "macos") {
            assert_eq!(shown, "cmd-shift-s");
        } else {
            assert_eq!(shown, "ctrl-shift-s");
        }
        assert_eq!(
            Shortcut::parse("Ctrl+Alt+D").unwrap(),
            Shortcut {
                primary: false,
                shift: false,
                alt: true,
                ctrl: true,
                key: 'd',
            }
        );
        assert_eq!(Shortcut::primary('q').unwrap().with_shift().key, 'q');
        for bad in ["d", "shift-d", "cmd-", "cmd-dd", "meta-d", "cmd-shift", ""] {
            assert_eq!(
                Shortcut::parse(bad),
                Err(MenuError::BadShortcut(bad.to_owned())),
                "{bad}"
            );
        }
    }

    #[test]
    fn an_item_reads_its_node_and_the_options_read_the_select() {
        let snap = snapshot(true, "hoff", false);
        let detach = MenuItem::of(&snap, &sid("shell.detach"), "toggle").unwrap();
        assert_eq!(detach.label, "page in a window of its own");
        assert_eq!(detach.checked, Some(true));
        assert!(detach.enabled);
        assert_eq!(detach.key(), "shell.detach#toggle");

        let save = MenuItem::of(&snap, &sid("shell.save"), "activate").unwrap();
        assert!(!save.enabled, "the node is disabled");
        assert_eq!(save.checked, None);

        assert!(MenuItem::of(&snap, &sid("shell.save"), "toggle").is_none());
        assert!(MenuItem::of(&snap, &sid("nobody"), "toggle").is_none());

        let themes = MenuItem::options(&snap, &sid("shell.theme"), "select");
        let labels: Vec<&str> = themes.iter().map(|i| i.label.as_str()).collect();
        assert_eq!(labels, ["core", "hoff", "dark"]);
        let checked: Vec<bool> = themes.iter().map(|i| i.checked.unwrap()).collect();
        assert_eq!(checked, [false, true, false]);
        assert_eq!(themes[2].param, Some(Value::Text("dark".into())));
        assert_eq!(themes[2].key(), "shell.theme#select#dark");
        assert!(MenuItem::options(&snap, &sid("shell.detach"), "toggle").is_empty());
    }

    #[test]
    fn the_plain_actions_are_the_ones_a_person_may_ask_without_a_value() {
        let snap = snapshot(false, "core", true);
        let plain = MenuItem::plain(&snap);
        let keys: Vec<String> = plain.iter().map(MenuItem::key).collect();
        assert_eq!(keys, ["shell.detach#toggle", "shell.save#activate"]);
    }

    #[test]
    fn a_click_is_the_request_the_button_makes() {
        let snap = snapshot(false, "core", true);
        let item = MenuItem::of(&snap, &sid("shell.detach"), "toggle").unwrap();
        let by_menu = item.clicked().request(&snap).unwrap();
        let by_button =
            Request::observe(&snap, Actor::Human, "shell.detach", "toggle", None).unwrap();
        assert_eq!(by_menu, by_button);
        assert_eq!(by_menu.actor, Actor::Human);
        let option = &MenuItem::options(&snap, &sid("shell.theme"), "select")[2];
        let req = option.clicked().request(&snap).unwrap();
        assert_eq!(req.param, Some(Value::Text("dark".into())));
    }

    #[test]
    fn the_model_holds_its_items_and_knows_its_shape() {
        let snap = snapshot(false, "core", true);
        let model = MenuModel::new()
            .menu(
                Submenu::new("plev")
                    .predefined(Predefined::About)
                    .separator()
                    .predefined(Predefined::Quit),
            )
            .menu(
                Submenu::new("view")
                    .submenu(Submenu::new("theme").items(MenuItem::options(
                        &snap,
                        &sid("shell.theme"),
                        "select",
                    )))
                    .separator()
                    .item(MenuItem::of(&snap, &sid("shell.detach"), "toggle").unwrap()),
            );
        assert_eq!(model.items().count(), 4);
        assert!(
            model
                .find(
                    &sid("shell.theme"),
                    "select",
                    Some(&Value::Text("hoff".into()))
                )
                .is_some()
        );
        assert!(model.find(&sid("shell.theme"), "select", None).is_none());

        // the same capabilities with the toggle on and another theme: the
        // shape holds, the marks moved
        let moved = snapshot(true, "dark", true);
        let again = MenuModel::new()
            .menu(
                Submenu::new("plev")
                    .predefined(Predefined::About)
                    .separator()
                    .predefined(Predefined::Quit),
            )
            .menu(
                Submenu::new("view")
                    .submenu(Submenu::new("theme").items(MenuItem::options(
                        &moved,
                        &sid("shell.theme"),
                        "select",
                    )))
                    .separator()
                    .item(MenuItem::of(&moved, &sid("shell.detach"), "toggle").unwrap()),
            );
        assert_ne!(model, again);
        assert!(model.same_shape(&again));

        // an item gone: another shape
        let fewer = MenuModel::new().menu(Submenu::new("plev").predefined(Predefined::Quit));
        assert!(!model.same_shape(&fewer));
        let relabeled = MenuModel {
            menus: vec![
                model.menus[0].clone(),
                Submenu {
                    label: "window".into(),
                    ..model.menus[1].clone()
                },
            ],
        };
        assert!(!model.same_shape(&relabeled));
    }

    #[test]
    fn the_fake_shows_the_model_and_takes_clicks_on_enabled_items_only() {
        let snap = snapshot(false, "core", false);
        let model = MenuModel::new().menu(
            Submenu::new("view")
                .item(MenuItem::of(&snap, &sid("shell.detach"), "toggle").unwrap())
                .item(MenuItem::of(&snap, &sid("shell.save"), "activate").unwrap()),
        );
        let mut fake = FakeMenu::new();
        assert!(fake.installed().is_none());
        assert!(
            !fake.click(&sid("shell.detach"), "toggle", None),
            "nothing shown yet"
        );
        fake.install(&model).unwrap();
        assert_eq!(fake.installs(), 1);
        assert_eq!(fake.installed(), Some(&model));
        assert!(fake.click(&sid("shell.detach"), "toggle", None));
        assert!(
            !fake.click(&sid("shell.save"), "activate", None),
            "disabled"
        );
        assert!(
            !fake.click(&sid("shell.theme"), "select", None),
            "not in the menu"
        );
        assert_eq!(fake.pending(), 1);
        let clicks = fake.poll();
        assert_eq!(
            clicks,
            [Clicked {
                sem: sid("shell.detach"),
                action: ActionName::parse("toggle").unwrap(),
                param: None,
            }]
        );
        assert!(fake.poll().is_empty());

        // the same model again is no update; a moved one is
        fake.update(&model).unwrap();
        assert_eq!(fake.updates(), 0);
        let moved = snapshot(true, "core", true);
        let model = MenuModel::new().menu(
            Submenu::new("view")
                .item(MenuItem::of(&moved, &sid("shell.detach"), "toggle").unwrap())
                .item(MenuItem::of(&moved, &sid("shell.save"), "activate").unwrap()),
        );
        fake.update(&model).unwrap();
        assert_eq!(fake.updates(), 1);
        assert!(
            fake.click(&sid("shell.save"), "activate", None),
            "enabled now"
        );
    }

    #[test]
    fn a_shared_fake_answers_through_one_handle_and_is_clicked_through_the_other() {
        let snap = snapshot(false, "core", true);
        let model = MenuModel::new().menu(
            Submenu::new("view").item(MenuItem::of(&snap, &sid("shell.save"), "activate").unwrap()),
        );
        let fake = Arc::new(Mutex::new(FakeMenu::new()));
        let mut shell: Box<dyn NativeMenu> = Box::new(fake.clone());
        shell.install(&model).unwrap();
        assert!(locked(&fake).click(&sid("shell.save"), "activate", None));
        assert_eq!(shell.poll().len(), 1);
        assert_eq!(locked(&fake).installs(), 1);
    }

    #[test]
    fn no_menu_is_unsupported_and_never_clicked() {
        let mut none = NoMenu;
        let model = MenuModel::new().menu(Submenu::new("plev").predefined(Predefined::Quit));
        assert_eq!(none.install(&model), Err(MenuError::Unsupported));
        assert_eq!(none.update(&model), Err(MenuError::Unsupported));
        assert!(none.poll().is_empty());
        assert_eq!(
            MenuError::Unsupported.to_string(),
            "no native menu on this target"
        );
    }
}
