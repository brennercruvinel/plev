//! The platform's menu through `muda`: on macOS the application's menu bar
//! (`Menu::init_for_nsapp`, one for every window of the app), on windows
//! a bar on the window whose handle the shell gives it (`init_for_hwnd`,
//! unverified here: windows is not a target yet), on linux nothing until
//! the bring-up picks a backend (the engine turns `muda`'s features off,
//! so `muda` compiles to its no-op backend there).
//!
//! How a click arrives: `muda` fires the item's action on the platform's
//! thread and sends a `MenuEvent { id }` either to its global channel
//! (`MenuEvent::receiver`) or to the one handler a process may set
//! (`MenuEvent::set_event_handler`); with a handler set the channel gets
//! nothing. The engine sets the handler once per process: it queues the
//! id and calls the waker the shell gave it, so a winit loop at `Wait`
//! wakes (`muda`'s own note for winit users) and polls on its thread,
//! before the frame. The queue is the process's, and two menus share it:
//! the bar here and the tray's menu (`platform::tray`, whose items carry
//! [`TRAY_PREFIX`] in their id), each taking its own ids when it polls
//! ([`take_clicks`]) and leaving the other's.
//!
//! Every `muda` type here must be made and used on the main thread
//! (`muda::Menu::new` panics elsewhere on macOS); a shell installs and
//! polls on its loop's thread, which is the main one. No test constructs
//! one: the pure pieces (the accelerator, the key text) are tested alone.

use std::collections::{HashMap, VecDeque};
use std::sync::{Mutex, Once};

use muda::accelerator::{Accelerator, CMD_OR_CTRL, Code, Modifiers};
use muda::{
    CheckMenuItem, Menu, MenuEvent, MenuId, MenuItem as NativeItem, PredefinedMenuItem,
    Submenu as NativeSubmenu,
};

use super::{
    Clicked, Entry, MenuError, MenuModel, NativeMenu, Predefined, Shortcut, Submenu, locked,
};

/// The clicks the platform fired and nobody polled yet, by item id.
static CLICKS: Mutex<VecDeque<MenuId>> = Mutex::new(VecDeque::new());
/// Who to wake on a click: the shell's loop, through its proxy.
static WAKER: Mutex<Option<Box<dyn Fn() + Send + Sync>>> = Mutex::new(None);
static ROUTE: Once = Once::new();

/// What the tray's menu puts before every item id, so the two consumers
/// of the one queue tell their clicks apart. A key (`sem#action[#param]`)
/// never starts with it: an id has no `:`.
pub(in crate::runtime::platform) const TRAY_PREFIX: &str = "tray:";

/// Sends every `muda` click of this process to `CLICKS` and wakes the
/// shell through `wake`. The handler is set once (`muda` keeps one per
/// process); the waker is the last one given, which is the one loop's
/// proxy however many menus the process shows.
pub(in crate::runtime::platform) fn route_clicks_to(wake: impl Fn() + Send + Sync + 'static) {
    *locked(&WAKER) = Some(Box::new(wake));
    ROUTE.call_once(|| {
        MenuEvent::set_event_handler(Some(|event: MenuEvent| {
            locked(&CLICKS).push_back(event.id);
            self::wake();
        }));
    });
}

/// Wakes the shell's loop, if one asked to be woken.
pub(in crate::runtime::platform) fn wake() {
    if let Some(wake) = locked(&WAKER).as_ref() {
        wake();
    }
}

/// The queued ids `mine` claims, in order, taken out of the queue; the
/// rest stay for their owner.
pub(in crate::runtime::platform) fn take_clicks(
    mut mine: impl FnMut(&MenuId) -> bool,
) -> Vec<MenuId> {
    let mut queue = locked(&CLICKS);
    let mut taken = Vec::new();
    queue.retain(|id| {
        if mine(id) {
            taken.push(id.clone());
            false
        } else {
            true
        }
    });
    taken
}

fn is_tray(id: &MenuId) -> bool {
    id.as_ref().starts_with(TRAY_PREFIX)
}

/// The bar as `muda` shows it, with a handle on every entry so a moved
/// model changes the entries in place.
pub struct MudaMenu {
    menu: Option<Menu>,
    bar: Vec<Installed>,
    /// Each item's id, as `muda` sends it back, to the capability it names.
    ids: HashMap<MenuId, Clicked>,
    model: Option<MenuModel>,
    #[cfg(target_os = "windows")]
    hwnd: Option<isize>,
}

/// One submenu as `muda` shows it, with a handle on every entry. The tray
/// shows one as its whole menu (`muda::Submenu` is a context menu too).
pub(in crate::runtime::platform) struct Installed {
    pub(in crate::runtime::platform) submenu: NativeSubmenu,
    entries: Vec<Handle>,
}

enum Handle {
    Item(NativeItem),
    Check(CheckMenuItem),
    /// A separator or a platform item: nothing to change later; the
    /// native submenu keeps it.
    Predefined,
    Submenu(Installed),
}

impl MudaMenu {
    /// A menu that wakes `wake` on every click (from the platform's
    /// thread) so the shell polls it. Nothing is shown until `install`.
    pub fn new(wake: impl Fn() + Send + Sync + 'static) -> MudaMenu {
        route_clicks_to(wake);
        MudaMenu {
            menu: None,
            bar: Vec::new(),
            ids: HashMap::new(),
            model: None,
            #[cfg(target_os = "windows")]
            hwnd: None,
        }
    }

    /// The model shown now, if any.
    pub fn installed(&self) -> Option<&MenuModel> {
        self.model.as_ref()
    }

    /// The window the bar belongs to, on windows (macOS has one bar per
    /// app and ignores this). Unverified: windows is not a target yet.
    #[cfg(target_os = "windows")]
    pub fn attach_window(&mut self, window: &winit::window::Window) -> Result<(), MenuError> {
        use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
        let handle = window
            .window_handle()
            .map_err(|e| MenuError::Platform(e.to_string()))?;
        let RawWindowHandle::Win32(win32) = handle.as_raw() else {
            return Err(MenuError::Platform("not a win32 window".to_owned()));
        };
        let hwnd = win32.hwnd.get() as isize;
        self.hwnd = Some(hwnd);
        if let Some(menu) = &self.menu {
            // SAFETY: `hwnd` came from the live `winit` window this call
            // was given, which outlives the call.
            unsafe { menu.init_for_hwnd(hwnd) }.map_err(platform)?;
        }
        Ok(())
    }

    /// Shows `menu`, built for `model`, as the platform's bar.
    fn show(&mut self, menu: Menu) -> Result<(), MenuError> {
        #[cfg(target_os = "macos")]
        menu.init_for_nsapp();
        #[cfg(target_os = "windows")]
        if let Some(hwnd) = self.hwnd {
            // SAFETY: the handle was taken from a live window in
            // `attach_window`; a window that closed detaches nothing here,
            // and `muda` answers an error rather than touching it.
            unsafe { menu.init_for_hwnd(hwnd) }.map_err(platform)?;
        }
        // the one shown before leaves after the new one is the app's
        self.menu = Some(menu);
        Ok(())
    }
}

impl NativeMenu for MudaMenu {
    fn install(&mut self, model: &MenuModel) -> Result<(), MenuError> {
        let menu = Menu::new();
        let mut ids = HashMap::new();
        let mut bar = Vec::with_capacity(model.menus.len());
        for submenu in &model.menus {
            let installed = build(submenu, &mut ids, "")?;
            menu.append(&installed.submenu).map_err(platform)?;
            bar.push(installed);
        }
        self.show(menu)?;
        self.bar = bar;
        self.ids = ids;
        self.model = Some(model.clone());
        log::debug!("menu: {} items shown", self.ids.len());
        Ok(())
    }

    /// The same model changes nothing; one of the same shape changes the
    /// labels, the `enabled` and the check marks in place; another shape
    /// is shown again.
    fn update(&mut self, model: &MenuModel) -> Result<(), MenuError> {
        match &self.model {
            Some(shown) if shown == model => Ok(()),
            Some(shown) if shown.same_shape(model) => {
                for (installed, submenu) in self.bar.iter_mut().zip(&model.menus) {
                    refresh(installed, submenu);
                }
                self.model = Some(model.clone());
                Ok(())
            }
            _ => self.install(model),
        }
    }

    fn poll(&mut self) -> Vec<Clicked> {
        take_clicks(|id| !is_tray(id))
            .into_iter()
            .filter_map(|id| {
                let clicked = self.ids.get(&id).cloned();
                if clicked.is_none() {
                    log::debug!("menu: a click on an item no longer shown ({})", id.as_ref());
                }
                clicked
            })
            .collect()
    }
}

/// `submenu` as `muda` items, appended in order; every item's id, `prefix`
/// then its key, goes into `ids` with the capability it names.
pub(in crate::runtime::platform) fn build(
    submenu: &Submenu,
    ids: &mut HashMap<MenuId, Clicked>,
    prefix: &str,
) -> Result<Installed, MenuError> {
    let native = NativeSubmenu::new(mnemonic_free(&submenu.label), true);
    let mut entries = Vec::with_capacity(submenu.entries.len());
    for entry in &submenu.entries {
        let handle = match entry {
            Entry::Item(item) => {
                let id = MenuId::new(format!("{prefix}{}", item.key()));
                ids.insert(id.clone(), item.clicked());
                let accelerator = item.shortcut.and_then(accelerator);
                let text = mnemonic_free(&item.label);
                match item.checked {
                    Some(on) => {
                        let h = CheckMenuItem::with_id(id, text, item.enabled, on, accelerator);
                        native.append(&h).map_err(platform)?;
                        Handle::Check(h)
                    }
                    None => {
                        let h = NativeItem::with_id(id, text, item.enabled, accelerator);
                        native.append(&h).map_err(platform)?;
                        Handle::Item(h)
                    }
                }
            }
            Entry::Separator => {
                native
                    .append(&PredefinedMenuItem::separator())
                    .map_err(platform)?;
                Handle::Predefined
            }
            Entry::Predefined(kind) => {
                native.append(&predefined(*kind)).map_err(platform)?;
                Handle::Predefined
            }
            Entry::Submenu(inner) => {
                let h = build(inner, ids, prefix)?;
                native.append(&h.submenu).map_err(platform)?;
                Handle::Submenu(h)
            }
        };
        entries.push(handle);
    }
    Ok(Installed {
        submenu: native,
        entries,
    })
}

/// The entries of `installed` follow `submenu`, of the same shape: only
/// what shows differently is touched.
pub(in crate::runtime::platform) fn refresh(installed: &mut Installed, submenu: &Submenu) {
    for (handle, entry) in installed.entries.iter_mut().zip(&submenu.entries) {
        match (handle, entry) {
            (Handle::Item(h), Entry::Item(item)) => {
                let text = mnemonic_free(&item.label);
                if h.text() != text {
                    h.set_text(text);
                }
                if h.is_enabled() != item.enabled {
                    h.set_enabled(item.enabled);
                }
            }
            (Handle::Check(h), Entry::Item(item)) => {
                let text = mnemonic_free(&item.label);
                if h.text() != text {
                    h.set_text(text);
                }
                if h.is_enabled() != item.enabled {
                    h.set_enabled(item.enabled);
                }
                let on = item.checked.unwrap_or(false);
                if h.is_checked() != on {
                    h.set_checked(on);
                }
            }
            (Handle::Submenu(h), Entry::Submenu(inner)) => refresh(h, inner),
            _ => {}
        }
    }
}

fn predefined(kind: Predefined) -> PredefinedMenuItem {
    match kind {
        Predefined::About => PredefinedMenuItem::about(None, None),
        Predefined::Hide => PredefinedMenuItem::hide(None),
        Predefined::HideOthers => PredefinedMenuItem::hide_others(None),
        Predefined::ShowAll => PredefinedMenuItem::show_all(None),
        Predefined::Quit => PredefinedMenuItem::quit(None),
    }
}

/// `muda` reads `&` as a mnemonic marker; a label shows its own `&`.
pub(super) fn mnemonic_free(label: &str) -> String {
    label.replace('&', "&&")
}

/// The `muda` accelerator of a shortcut: the primary modifier is cmd on
/// macOS and ctrl elsewhere; the key is the physical key of the letter or
/// digit. `None` for a key `muda` has no code for.
pub(super) fn accelerator(shortcut: Shortcut) -> Option<Accelerator> {
    let mut mods = Modifiers::empty();
    if shortcut.primary {
        mods |= CMD_OR_CTRL;
    }
    if shortcut.ctrl {
        mods |= Modifiers::CONTROL;
    }
    if shortcut.alt {
        mods |= Modifiers::ALT;
    }
    if shortcut.shift {
        mods |= Modifiers::SHIFT;
    }
    let key = shortcut.key;
    let name = if key.is_ascii_digit() {
        format!("Digit{key}")
    } else {
        format!("Key{}", key.to_ascii_uppercase())
    };
    let code: Code = name.parse().ok()?;
    Some(Accelerator::new(mods, code))
}

fn platform(e: muda::Error) -> MenuError {
    MenuError::Platform(e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_shortcut_becomes_the_platform_accelerator() {
        let save = accelerator(Shortcut::parse("cmd-shift-s").unwrap()).unwrap();
        assert_eq!(save.key(), Code::KeyS);
        assert_eq!(save.modifiers(), CMD_OR_CTRL | Modifiers::SHIFT);
        let digit = accelerator(Shortcut::parse("alt-1").unwrap()).unwrap();
        assert_eq!(digit.key(), Code::Digit1);
        assert_eq!(digit.modifiers(), Modifiers::ALT);
        let both = accelerator(Shortcut::parse("ctrl-alt-d").unwrap()).unwrap();
        assert_eq!(both.modifiers(), Modifiers::CONTROL | Modifiers::ALT);
        assert_eq!(both.key(), Code::KeyD);
    }

    #[test]
    fn a_label_keeps_its_ampersand() {
        assert_eq!(mnemonic_free("cut & paste"), "cut && paste");
        assert_eq!(mnemonic_free("plain"), "plain");
    }
}
