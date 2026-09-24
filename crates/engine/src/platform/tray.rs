//! The system tray icon as another view of an app's capabilities
//! (task-76 item 5): the integration that stays when the window is
//! hidden or behind another. A [`TrayModel`] is pure data the app builds
//! from its projection: the icon, a tooltip, a menu of the same items the
//! bar shows (a [`Submenu`] of `platform::menu`, so a theme, a page, a
//! window and a quit are the same `MenuItem`s), and, for a click on the
//! icon itself, the one capability the app chose (bring the window
//! forward, say). Every click comes back as a [`Clicked`] and becomes the
//! same `Request::observe` by `Actor::Human` the button makes, through
//! the app's one dispatcher; the platform decides nothing.
//!
//! The same shape as `menu.rs`: one trait ([`Tray`]), the real one where
//! `tray-icon` builds without a feature ([`NativeTray`], macOS and
//! windows; `native.rs`), [`FakeTray`] everywhere for the tests (it keeps
//! the model shown and takes injected clicks), [`NoTray`] on linux (until
//! its bring-up picks a backend), wasm and mobile, [`default_tray`]
//! choosing.
//!
//! The icon is the process's: one `NativeTray` per process, made and
//! polled on the main thread with the loop running (`tray-icon` refuses
//! another thread). Creating one shows it at once, so no test makes a real
//! one; the fake is the proof.

use std::collections::VecDeque;
use std::fmt;
use std::sync::{Arc, Mutex};

use sem::{SemId, Value};

use super::menu::{Clicked, MenuError, MenuItem, Submenu, locked};

#[cfg(any(target_os = "macos", target_os = "windows"))]
mod native;
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub use native::NativeTray;

/// The icon's pixels: 32bpp RGBA, rows top to bottom. `template` asks the
/// platform to draw the alpha as the shape and tint it itself (macOS: a
/// template image follows the bar's light or dark look).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrayImage {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    pub template: bool,
}

impl TrayImage {
    /// `width` by `height` pixels of `rgba`, four bytes each; refused when
    /// the bytes do not fit the size, or the size is empty.
    pub fn from_rgba(width: u32, height: u32, rgba: Vec<u8>) -> Result<TrayImage, TrayError> {
        let pixels = (width as usize) * (height as usize);
        if pixels == 0 {
            return Err(TrayError::BadIcon(
                "an icon has at least one pixel".to_owned(),
            ));
        }
        if rgba.len() != pixels * 4 {
            return Err(TrayError::BadIcon(format!(
                "{width}x{height} wants {} bytes, {} given",
                pixels * 4,
                rgba.len()
            )));
        }
        Ok(TrayImage {
            width,
            height,
            rgba,
            template: false,
        })
    }

    /// A mark the engine draws itself, for an app without an icon of its
    /// own: a ring, `size` pixels a side (at least 4), as a template.
    pub fn mark(size: u32) -> TrayImage {
        let size = size.max(4);
        let c = size as f32 / 2.0;
        let outer = c - 0.5;
        let inner = c * 0.45;
        let mut rgba = Vec::with_capacity((size * size * 4) as usize);
        for y in 0..size {
            for x in 0..size {
                let dx = x as f32 + 0.5 - c;
                let dy = y as f32 + 0.5 - c;
                let d = (dx * dx + dy * dy).sqrt();
                let cover = (outer - d + 0.5).clamp(0.0, 1.0) - (inner - d + 0.5).clamp(0.0, 1.0);
                let a = (cover.clamp(0.0, 1.0) * 255.0).round() as u8;
                rgba.extend_from_slice(&[0, 0, 0, a]);
            }
        }
        TrayImage {
            width: size,
            height: size,
            rgba,
            template: true,
        }
    }

    pub fn template(mut self, on: bool) -> TrayImage {
        self.template = on;
        self
    }
}

/// The tray as the app wants it shown: the icon, its tooltip, the menu on
/// it, and what a click on the icon itself asks for. Built by the app
/// from its projection; pure data, the same on every target.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrayModel {
    pub icon: TrayImage,
    pub tooltip: String,
    /// The menu on the icon: its entries show, its label does not.
    pub menu: Submenu,
    /// The capability a click on the icon asks for, if any. With one, the
    /// left click asks it and the right click opens the menu; without
    /// one, either click opens the menu.
    pub on_click: Option<Clicked>,
}

impl TrayModel {
    pub fn new(icon: TrayImage, tooltip: impl Into<String>) -> TrayModel {
        let tooltip = tooltip.into();
        TrayModel {
            icon,
            menu: Submenu::new(tooltip.clone()),
            tooltip,
            on_click: None,
        }
    }

    pub fn menu(mut self, menu: Submenu) -> TrayModel {
        self.menu = menu;
        self
    }

    pub fn on_click(mut self, clicked: Clicked) -> TrayModel {
        self.on_click = Some(clicked);
        self
    }

    /// Every item of the menu, depth first, in order.
    pub fn items(&self) -> impl Iterator<Item = &MenuItem> {
        self.menu.items_deep()
    }

    /// The item that asks `action` of `sem` with `param`.
    pub fn find(&self, sem: &SemId, action: &str, param: Option<&Value>) -> Option<&MenuItem> {
        self.items()
            .find(|i| i.sem == *sem && i.action.as_str() == action && i.param.as_ref() == param)
    }

    /// True when the menus have the same shape ([`Submenu::same_shape`]):
    /// the icon, the tooltip, the labels, the marks and the click change
    /// in place; another menu shape is shown again.
    pub fn same_shape(&self, other: &TrayModel) -> bool {
        self.menu.same_shape(&other.menu)
    }
}

/// Why a tray could not be shown.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TrayError {
    /// This target has no tray (linux until its bring-up, wasm, mobile).
    Unsupported,
    /// The icon's bytes do not make an image.
    BadIcon(String),
    /// The platform refused (a `tray-icon` or `muda` error, in its words).
    Platform(String),
}

impl fmt::Display for TrayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TrayError::Unsupported => f.write_str("no tray on this target"),
            TrayError::BadIcon(why) => write!(f, "not an icon: {why}"),
            TrayError::Platform(why) => write!(f, "the platform refused the tray: {why}"),
        }
    }
}

impl std::error::Error for TrayError {}

/// The tray's menu is built by the menu's code; its refusals are the same.
impl From<MenuError> for TrayError {
    fn from(e: MenuError) -> TrayError {
        match e {
            MenuError::Unsupported => TrayError::Unsupported,
            MenuError::BadShortcut(text) => {
                TrayError::Platform(format!("not a shortcut: {text:?}"))
            }
            MenuError::Platform(why) => TrayError::Platform(why),
        }
    }
}

/// The platform's tray. `install` shows `model` as the icon (replacing
/// one shown before); `update` makes it follow a model that moved (the
/// icon, the tooltip, the labels, `enabled` and the marks in place; a
/// changed menu shape is shown again); `poll` hands back the clicks since
/// the last poll, in order (the icon's own click as the model's
/// `on_click`), for the app to dispatch on its loop, before the frame.
pub trait Tray {
    fn install(&mut self, model: &TrayModel) -> Result<(), TrayError>;
    fn update(&mut self, model: &TrayModel) -> Result<(), TrayError>;
    fn poll(&mut self) -> Vec<Clicked>;
}

// -- the fake ---------------------------------------------------------------

/// The tray of a test: keeps the model shown, counts the installs and the
/// updates, and takes injected clicks on the menu ([`FakeTray::click`])
/// and on the icon ([`FakeTray::click_icon`]), which `poll` hands back as
/// the platform would.
#[derive(Debug, Default)]
pub struct FakeTray {
    model: Option<TrayModel>,
    installs: usize,
    updates: usize,
    clicks: VecDeque<Clicked>,
}

impl FakeTray {
    pub fn new() -> FakeTray {
        FakeTray::default()
    }

    pub fn installs(&self) -> usize {
        self.installs
    }

    pub fn updates(&self) -> usize {
        self.updates
    }

    /// A person clicks the item asking `action` of `sem` with `param`:
    /// queued for `poll`. `false`, and nothing queued, when the menu shown
    /// has no such item or it is disabled.
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

    /// A person clicks the icon itself: the model's `on_click` is queued.
    /// `false` when nothing is shown or the model asks nothing for it.
    pub fn click_icon(&mut self) -> bool {
        let Some(clicked) = self.model.as_ref().and_then(|m| m.on_click.clone()) else {
            return false;
        };
        self.clicks.push_back(clicked);
        true
    }

    /// Clicks taken and not polled yet.
    pub fn pending(&self) -> usize {
        self.clicks.len()
    }

    /// The model shown now, if any.
    pub fn installed(&self) -> Option<&TrayModel> {
        self.model.as_ref()
    }
}

impl Tray for FakeTray {
    fn install(&mut self, model: &TrayModel) -> Result<(), TrayError> {
        self.model = Some(model.clone());
        self.installs += 1;
        Ok(())
    }

    fn update(&mut self, model: &TrayModel) -> Result<(), TrayError> {
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

// -- no tray at all ---------------------------------------------------------

/// The tray of a target without one: linux until its bring-up picks a
/// backend, wasm and mobile. Installing is `Unsupported`, nothing is shown
/// and nothing is ever clicked; the app's capabilities stay on screen.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoTray;

impl Tray for NoTray {
    fn install(&mut self, _model: &TrayModel) -> Result<(), TrayError> {
        Err(TrayError::Unsupported)
    }

    fn update(&mut self, _model: &TrayModel) -> Result<(), TrayError> {
        Err(TrayError::Unsupported)
    }

    fn poll(&mut self) -> Vec<Clicked> {
        Vec::new()
    }
}

/// A tray shared by handle: a test keeps one `Arc` and gives the shell
/// the other as its `Box<dyn Tray>`, then clicks through its own. A
/// poisoned lock still answers: the tray holds no invariant a panic
/// elsewhere could break.
impl<T: Tray> Tray for Arc<Mutex<T>> {
    fn install(&mut self, model: &TrayModel) -> Result<(), TrayError> {
        locked(self).install(model)
    }

    fn update(&mut self, model: &TrayModel) -> Result<(), TrayError> {
        locked(self).update(model)
    }

    fn poll(&mut self) -> Vec<Clicked> {
        locked(self).poll()
    }
}

/// The tray of the current target, woken by `wake` on every click (from
/// the platform's thread; a shell sends itself an event through its loop
/// proxy and polls on the loop): native on macOS and windows, none
/// elsewhere.
pub fn default_tray(wake: impl Fn() + Send + Sync + 'static) -> Box<dyn Tray> {
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    {
        Box::new(NativeTray::new(wake))
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let _ = wake;
        Box::new(NoTray)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sem::{ActionName, Constraints, Policy, Role, Snapshot, SnapshotBuilder};

    use crate::platform::menu::Predefined;

    fn sid(text: &str) -> SemId {
        SemId::parse(text).unwrap()
    }

    /// A small shell: a button that shows the window, a select and a
    /// toggle, the button disabled or not.
    fn snapshot(theme: &str, can_show: bool) -> Snapshot {
        let mut b = SnapshotBuilder::new();
        b.node("shell", Role::Region, "plev");
        b.node("shell.show", Role::Button, "show the window")
            .parent("shell")
            .enabled(can_show)
            .action("activate")
            .policy(Policy {
                actors: sem::Actors {
                    agent: false,
                    ..sem::Actors::default()
                },
                ..Policy::default()
            });
        b.node("shell.theme", Role::Select, "theme")
            .parent("shell")
            .state("selected", theme)
            .constraints(Constraints {
                editable: true,
                options: Some(vec!["core".into(), "hoff".into()]),
                ..Constraints::default()
            })
            .action("select")
            .param(sem::ParamSpec {
                kind: sem::ValueKind::Text,
                constrained: true,
            })
            .reads("shell.theme:selected");
        b.node("shell.detach", Role::Toggle, "page in a window of its own")
            .parent("shell")
            .state("on", false)
            .action("toggle")
            .reads("shell.detach:on");
        b.finish().unwrap()
    }

    fn model(snap: &Snapshot) -> TrayModel {
        let show = MenuItem::of(snap, &sid("shell.show"), "activate").unwrap();
        let menu = Submenu::new("plev")
            .item(show.clone())
            .separator()
            .submenu(Submenu::new("theme").items(MenuItem::options(
                snap,
                &sid("shell.theme"),
                "select",
            )))
            .item(MenuItem::of(snap, &sid("shell.detach"), "toggle").unwrap())
            .separator()
            .predefined(Predefined::Quit);
        TrayModel::new(TrayImage::mark(18), "plev")
            .menu(menu)
            .on_click(show.clicked())
    }

    #[test]
    fn the_mark_is_a_ring_and_an_image_checks_its_bytes() {
        let mark = TrayImage::mark(18);
        assert_eq!((mark.width, mark.height), (18, 18));
        assert_eq!(mark.rgba.len(), 18 * 18 * 4);
        assert!(mark.template);
        let alpha = |x: u32, y: u32| mark.rgba[((y * 18 + x) * 4 + 3) as usize];
        assert_eq!(alpha(0, 0), 0, "the corner is clear");
        assert_eq!(alpha(9, 9), 0, "the hole is clear");
        assert_eq!(alpha(9, 1), 255, "the ring is solid");
        assert!(mark.rgba.chunks(4).all(|p| p[..3] == [0, 0, 0]));

        assert_eq!(TrayImage::mark(1).width, 4, "never smaller than 4");
        let ok = TrayImage::from_rgba(2, 1, vec![255; 8]).unwrap();
        assert!(!ok.template);
        assert!(ok.template(true).template);
        assert!(matches!(
            TrayImage::from_rgba(2, 1, vec![255; 7]),
            Err(TrayError::BadIcon(_))
        ));
        assert!(matches!(
            TrayImage::from_rgba(0, 4, vec![]),
            Err(TrayError::BadIcon(_))
        ));
        assert_eq!(TrayError::BadIcon("x".into()).to_string(), "not an icon: x");
    }

    #[test]
    fn the_model_holds_its_items_and_knows_its_shape() {
        let snap = snapshot("core", true);
        let model = model(&snap);
        assert_eq!(model.tooltip, "plev");
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
        assert_eq!(
            model.on_click,
            Some(Clicked {
                sem: sid("shell.show"),
                action: ActionName::parse("activate").unwrap(),
                param: None,
            })
        );

        // the same capabilities with another theme and the button
        // disabled: the shape holds
        let moved = self::model(&snapshot("hoff", false));
        assert_ne!(model, moved);
        assert!(model.same_shape(&moved));
        assert!(
            !moved
                .find(&sid("shell.show"), "activate", None)
                .unwrap()
                .enabled
        );

        // an item gone: another shape; a bare model has an empty menu
        let fewer = TrayModel::new(TrayImage::mark(18), "plev");
        assert!(fewer.menu.is_empty());
        assert_eq!(fewer.menu.label, "plev");
        assert!(!model.same_shape(&fewer));
        assert!(fewer.on_click.is_none());
    }

    #[test]
    fn the_fake_shows_the_model_and_takes_clicks_on_the_menu_and_the_icon() {
        let snap = snapshot("core", false);
        let model = model(&snap);
        let mut fake = FakeTray::new();
        assert!(fake.installed().is_none());
        assert!(!fake.click_icon(), "nothing shown yet");
        fake.install(&model).unwrap();
        assert_eq!(fake.installs(), 1);
        assert_eq!(fake.installed(), Some(&model));

        assert!(fake.click(&sid("shell.detach"), "toggle", None));
        assert!(
            !fake.click(&sid("shell.show"), "activate", None),
            "disabled"
        );
        assert!(
            !fake.click(&sid("shell.page"), "select", None),
            "not in the menu"
        );
        assert!(
            fake.click_icon(),
            "the icon asks even while the item is disabled"
        );
        assert_eq!(fake.pending(), 2);
        let clicks = fake.poll();
        assert_eq!(clicks.len(), 2);
        assert_eq!(clicks[0].sem, sid("shell.detach"));
        assert_eq!(clicks[1], model.on_click.clone().unwrap());
        assert!(fake.poll().is_empty());

        // the same model again is no update; a moved one is
        fake.update(&model).unwrap();
        assert_eq!(fake.updates(), 0);
        let moved = self::model(&snapshot("hoff", true));
        fake.update(&moved).unwrap();
        assert_eq!(fake.updates(), 1);
        assert!(
            fake.click(&sid("shell.show"), "activate", None),
            "enabled now"
        );

        // a model without a click on the icon queues nothing for it
        let mute = TrayModel::new(TrayImage::mark(18), "plev");
        fake.update(&mute).unwrap();
        assert!(!fake.click_icon());
    }

    #[test]
    fn a_shared_fake_answers_through_one_handle_and_is_clicked_through_the_other() {
        let model = model(&snapshot("core", true));
        let fake = Arc::new(Mutex::new(FakeTray::new()));
        let mut shell: Box<dyn Tray> = Box::new(fake.clone());
        shell.install(&model).unwrap();
        assert!(locked(&fake).click_icon());
        assert_eq!(shell.poll().len(), 1);
        assert_eq!(locked(&fake).installs(), 1);
    }

    #[test]
    fn no_tray_is_unsupported_and_never_clicked() {
        let mut none = NoTray;
        let model = TrayModel::new(TrayImage::mark(18), "plev");
        assert_eq!(none.install(&model), Err(TrayError::Unsupported));
        assert_eq!(none.update(&model), Err(TrayError::Unsupported));
        assert!(none.poll().is_empty());
        assert_eq!(TrayError::Unsupported.to_string(), "no tray on this target");
        assert_eq!(
            TrayError::from(MenuError::Platform("x".into())),
            TrayError::Platform("x".into())
        );
        assert_eq!(
            TrayError::from(MenuError::Unsupported),
            TrayError::Unsupported
        );
    }
}
