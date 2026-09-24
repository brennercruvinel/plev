//! The window's outside actors, desktop only: a screen reader (the
//! AccessKit bridge) and agents (the port), and how their threads wake the
//! event loop.

use winit::event_loop::{ActiveEventLoop, EventLoopProxy};
use winit::window::Window;

use super::{TITLE, UserEvent};
use crate::view::ShowcaseView;

/// The window's outside actors (a screen reader, agents through the port)
/// and how their threads wake the event loop.
#[derive(Default)]
pub(super) struct Access {
    pub(super) bridge: Option<engine::accessibility::Bridge>,
    pub(super) agent: Option<wire::AgentPort>,
    pub(super) proxy: Option<EventLoopProxy<UserEvent>>,
    /// The windows' scenes, written for the port's watches (`mirror`).
    pub(super) scenes: super::mirror::Scenes,
}

impl Access {
    /// Creates the bridge for `window`, which is still hidden. Without a
    /// bridge the window still opens, and the reason is logged.
    pub(super) fn attach(&mut self, event_loop: &ActiveEventLoop, window: &Window) {
        let proxy = self.proxy.clone();
        let wake = move || {
            if let Some(proxy) = &proxy {
                let _ = proxy.send_event(UserEvent::Access);
            }
        };
        match engine::accessibility::Bridge::new(event_loop, window, TITLE, wake) {
            Ok(bridge) => self.bridge = Some(bridge),
            Err(e) => log::error!("accessibility: {e}"),
        }
    }

    /// What the screen reader asked, on the main thread: a request goes
    /// through the active page's dispatcher as `Actor::Assistive`, a focus
    /// move to the page's focus.
    pub(super) fn carry_out(&mut self, view: &mut ShowcaseView) {
        let Some(bridge) = self.bridge.as_mut() else {
            return;
        };
        for assist in bridge.take() {
            match assist {
                engine::accessibility::Assist::Request(req) => {
                    let asked = format!("{} {}", req.target, req.action);
                    match view.act(req) {
                        Some(receipt) => {
                            log::debug!("accessibility: {asked}: {:?}", receipt.stage());
                        }
                        None => log::debug!("accessibility: no page takes {asked}"),
                    }
                }
                engine::accessibility::Assist::Focus(id) => {
                    if !view.assist_focus(&id) {
                        log::debug!("accessibility: no page takes focus on {id}");
                    }
                }
            }
        }
    }
}
