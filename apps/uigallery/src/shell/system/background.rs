//! `PLEV_BACKGROUND=1` on macOS: the window comes up without taking the
//! focus, whatever app is in front, so it can open beside someone at work
//! (a run the person watches, not one they use), the way the ide's does:
//! the app never activates (its activation policy is prohibited, so no
//! Dock icon and no menu bar either), the window is ordered front without
//! becoming key, and it sits below every other window, so it covers
//! nothing and takes no click meant for another app. Nothing changes
//! without it.
//!
//! Keys reach it posted to its pid; a mouse event posted to its pid does
//! not, since AppKit hands one only to the window on top at its point
//! (checked on a covered screen: an Escape arrived, a click at the bottom
//! level or at the normal one, under the app in front, never did).

use winit::event_loop::EventLoopBuilder;
use winit::platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS};
use winit::window::{WindowAttributes, WindowLevel};

use super::super::UserEvent;

fn wanted() -> bool {
    std::env::var_os("PLEV_BACKGROUND").is_some_and(|v| v == "1")
}

/// Asking not to activate over the app in front is not enough: since
/// macOS 14 activation is cooperative, and a process started from the
/// terminal of the app in front is let in over it. An app whose activation
/// policy is prohibited may not be activated at all, and still shows its
/// window.
pub(super) fn event_loop(builder: &mut EventLoopBuilder<UserEvent>) {
    if wanted() {
        builder
            .with_activation_policy(ActivationPolicy::Prohibited)
            .with_activate_ignoring_other_apps(false);
    }
}

pub(super) fn window(attrs: WindowAttributes) -> WindowAttributes {
    if wanted() {
        attrs
            .with_active(false)
            .with_window_level(WindowLevel::AlwaysOnBottom)
    } else {
        attrs
    }
}
