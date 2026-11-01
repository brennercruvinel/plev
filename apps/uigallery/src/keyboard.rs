//! Keyboard bridge for the platform shell: winit named keys map onto the
//! view's winit-free editing keys (Tab focus cycling, the Forms text
//! fields, the App add field and its Enter). Escape stays in the
//! shell's window events (`shell/window/events`): it closes what is open,
//! never the window.

use winit::keyboard::NamedKey;

use crate::views::{EditKey, NavKey, ShowcaseView};

/// Route a named key to the view; with `shift` the arrows, Home and End
/// are their `Select` keys. Mirrors the view handlers' contract: `true`
/// when the key was consumed and a redraw is needed.
pub fn handle_named(view: &mut ShowcaseView, named: &NamedKey, shift: bool) -> bool {
    let key = match (named, shift) {
        (NamedKey::Space, _) => return view.handle_key(" "),
        (NamedKey::Enter, _) => return view.handle_enter(),
        (NamedKey::ArrowUp, _) => return view.handle_nav(NavKey::Up),
        (NamedKey::ArrowDown, _) => return view.handle_nav(NavKey::Down),
        (NamedKey::Tab, _) => EditKey::Tab,
        (NamedKey::Backspace, _) => EditKey::Backspace,
        (NamedKey::Delete, _) => EditKey::Delete,
        (NamedKey::ArrowLeft, false) => EditKey::Left,
        (NamedKey::ArrowLeft, true) => EditKey::SelectLeft,
        (NamedKey::ArrowRight, false) => EditKey::Right,
        (NamedKey::ArrowRight, true) => EditKey::SelectRight,
        (NamedKey::Home, false) => EditKey::Home,
        (NamedKey::Home, true) => EditKey::SelectHome,
        (NamedKey::End, false) => EditKey::End,
        (NamedKey::End, true) => EditKey::SelectEnd,
        _ => return false,
    };
    view.handle_edit_key(key)
}
