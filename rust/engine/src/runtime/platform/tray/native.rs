//! The platform's tray through `tray-icon`: on macOS an `NSStatusItem` in
//! the menu bar, on windows an icon in the notification area (written
//! against the crate's API and not compiled here: windows is not a target
//! yet). The menu on it is a `muda::Submenu` built by the menu's own code
//! (`menu::native::build`), so the tray shows the same items the bar does
//! and their clicks arrive through the same `muda` handler, into the same
//! queue; the tray's ids carry `TRAY_PREFIX`, and each of the two takes
//! its own when it polls.
//!
//! A click on the icon itself arrives as a `TrayIconEvent` on the
//! platform's thread, through the one handler a process may set (like
//! `muda`'s); the engine sets it once, counts the left clicks released and
//! wakes the shell, which polls on its loop. With the model asking
//! something for the click, the left button asks it and the right button
//! opens the menu; without, either button opens the menu (`tray-icon`'s
//! `menu_on_left_click`).
//!
//! `tray-icon` refuses any thread but the main one and wants the loop
//! running (winit: from `StartCause::Init` on), and creating the icon
//! shows it at once: no test constructs one.

use std::collections::HashMap;
use std::sync::{Mutex, Once};

use muda::{ContextMenu, MenuId};
use tray_icon::{Icon, MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};

use super::super::menu::native::{
    Installed, TRAY_PREFIX, build, refresh, route_clicks_to, take_clicks, wake,
};
use super::super::menu::{Clicked, locked};
use super::{Tray, TrayError, TrayImage, TrayModel};

/// Left clicks released on the icon that nobody polled yet. One tray per
/// process, so a count is the whole story.
static ICON_CLICKS: Mutex<usize> = Mutex::new(0);
static ROUTE: Once = Once::new();

/// Counts every left click released on the icon and wakes the shell.
/// Once: `tray-icon` keeps one handler per process.
fn route_icon_clicks() {
    ROUTE.call_once(|| {
        TrayIconEvent::set_event_handler(Some(|event: TrayIconEvent| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                *locked(&ICON_CLICKS) += 1;
                wake();
            }
        }));
    });
}

/// The tray as `tray-icon` shows it, with the menu's handles so a moved
/// model changes the entries in place.
pub struct NativeTray {
    icon: Option<TrayIcon>,
    menu: Option<Installed>,
    /// Each item's id, as `muda` sends it back, to the capability it names.
    ids: HashMap<MenuId, Clicked>,
    model: Option<TrayModel>,
}

impl NativeTray {
    /// A tray that wakes `wake` on every click (from the platform's
    /// thread) so the shell polls it. Nothing is shown until `install`.
    pub fn new(wake: impl Fn() + Send + Sync + 'static) -> NativeTray {
        route_clicks_to(wake);
        route_icon_clicks();
        NativeTray {
            icon: None,
            menu: None,
            ids: HashMap::new(),
            model: None,
        }
    }

    /// The model shown now, if any.
    pub fn installed(&self) -> Option<&TrayModel> {
        self.model.as_ref()
    }
}

/// The image as `tray-icon` takes it. On macOS this copies the bytes and
/// touches nothing of the platform; `TrayImage` checked the size already.
fn icon(image: &TrayImage) -> Result<Icon, TrayError> {
    Icon::from_rgba(image.rgba.clone(), image.width, image.height)
        .map_err(|e| TrayError::BadIcon(e.to_string()))
}

fn platform(e: tray_icon::Error) -> TrayError {
    TrayError::Platform(e.to_string())
}

impl Tray for NativeTray {
    fn install(&mut self, model: &TrayModel) -> Result<(), TrayError> {
        let mut ids = HashMap::new();
        let menu = build(&model.menu, &mut ids, TRAY_PREFIX)?;
        let image = icon(&model.icon)?;
        let context: Box<dyn ContextMenu> = Box::new(menu.submenu.clone());
        match &self.icon {
            // the icon stays; the menu on it is the new one
            Some(tray) => {
                tray.set_icon_with_as_template(Some(image), model.icon.template)
                    .map_err(platform)?;
                tray.set_tooltip(Some(&model.tooltip)).map_err(platform)?;
                tray.set_menu(Some(context));
                tray.set_show_menu_on_left_click(model.on_click.is_none());
            }
            None => {
                let tray = TrayIconBuilder::new()
                    .with_icon(image)
                    .with_icon_as_template(model.icon.template)
                    .with_tooltip(&model.tooltip)
                    .with_menu(context)
                    .with_menu_on_left_click(model.on_click.is_none())
                    .build()
                    .map_err(platform)?;
                self.icon = Some(tray);
            }
        }
        self.menu = Some(menu);
        self.ids = ids;
        self.model = Some(model.clone());
        log::debug!("tray: {} items in the menu", self.ids.len());
        Ok(())
    }

    /// The same model changes nothing; one of the same shape changes the
    /// icon, the tooltip, the click and the entries in place; another
    /// shape shows the menu again on the same icon.
    fn update(&mut self, model: &TrayModel) -> Result<(), TrayError> {
        match &self.model {
            Some(shown) if shown == model => Ok(()),
            Some(shown) if shown.same_shape(model) => {
                if let Some(tray) = &self.icon {
                    if shown.icon != model.icon {
                        tray.set_icon_with_as_template(
                            Some(icon(&model.icon)?),
                            model.icon.template,
                        )
                        .map_err(platform)?;
                    }
                    if shown.tooltip != model.tooltip {
                        tray.set_tooltip(Some(&model.tooltip)).map_err(platform)?;
                    }
                    if shown.on_click.is_none() != model.on_click.is_none() {
                        tray.set_show_menu_on_left_click(model.on_click.is_none());
                    }
                }
                if let Some(menu) = &mut self.menu {
                    refresh(menu, &model.menu);
                }
                self.model = Some(model.clone());
                Ok(())
            }
            _ => self.install(model),
        }
    }

    fn poll(&mut self) -> Vec<Clicked> {
        let mut out = Vec::new();
        let on_icon = std::mem::take(&mut *locked(&ICON_CLICKS));
        if let Some(asks) = self.model.as_ref().and_then(|m| m.on_click.clone()) {
            out.extend(std::iter::repeat_n(asks, on_icon));
        } else if on_icon > 0 {
            log::debug!("tray: {on_icon} clicks on an icon that asks nothing");
        }
        for id in take_clicks(|id| id.as_ref().starts_with(TRAY_PREFIX)) {
            match self.ids.get(&id) {
                Some(clicked) => out.push(clicked.clone()),
                None => log::debug!("tray: a click on an item no longer shown ({})", id.as_ref()),
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// On macOS `tray-icon` keeps the bytes as they are until they are
    /// shown; nothing of the platform is touched here.
    #[cfg(target_os = "macos")]
    #[test]
    fn the_image_becomes_the_platforms_icon_or_says_why_not() {
        assert!(icon(&TrayImage::mark(18)).is_ok());
        let torn = TrayImage {
            width: 3,
            height: 3,
            rgba: vec![0; 5],
            template: false,
        };
        assert!(matches!(icon(&torn), Err(TrayError::BadIcon(_))));
    }
}
