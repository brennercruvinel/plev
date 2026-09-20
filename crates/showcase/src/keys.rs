//! Keyboard bridge for the platform shell: winit named keys map onto the
//! view's winit-free editing keys (Tab focus cycling, the Forms text
//! fields, the App add field and its Enter). Escape stays in app.rs,
//! which owns close/quit.

use winit::keyboard::NamedKey;

use crate::view::{EditKey, ShowcaseView};

/// Route a named key to the view. Mirrors the view handlers' contract:
/// `true` when the key was consumed and a redraw is needed.
pub fn handle_named(view: &mut ShowcaseView, named: &NamedKey) -> bool {
    let key = match named {
        NamedKey::Space => return view.handle_key(" "),
        NamedKey::Enter => return view.handle_enter(),
        NamedKey::Tab => EditKey::Tab,
        NamedKey::Backspace => EditKey::Backspace,
        NamedKey::Delete => EditKey::Delete,
        NamedKey::ArrowLeft => EditKey::Left,
        NamedKey::ArrowRight => EditKey::Right,
        NamedKey::Home => EditKey::Home,
        NamedKey::End => EditKey::End,
        _ => return false,
    };
    view.handle_edit_key(key)
}
