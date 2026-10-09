//! Keyboard bridge for the platform shell: winit named keys map onto the
//! view's winit-free editing keys (field editing + Enter submission).
//! Escape stays in app.rs, which owns close/quit.

use winit::keyboard::NamedKey;

use crate::view::{EditKey, UrnauiView};

/// Route a named key to the view; with `shift` the arrows, Home and End
/// are their `Select` keys. Mirrors the view handlers' contract: `true`
/// when the key was consumed and a redraw is needed.
pub fn handle_named(view: &mut UrnauiView, named: &NamedKey, shift: bool) -> bool {
    let key = match (named, shift) {
        (NamedKey::Space, _) => return view.handle_key(" "),
        (NamedKey::Tab, _) => EditKey::Tab,
        (NamedKey::Enter, _) => EditKey::Enter,
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
