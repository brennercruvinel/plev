//! Conversion from winit keyboard events to [`Keystroke`].
//!
//! Kept separate from the parser so the logical-key mapping can be unit
//! tested without constructing a `winit::event::KeyEvent` (which carries
//! private platform-specific fields).

use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{Key, ModifiersState, NamedKey};

use super::keystroke::{Keystroke, Modifiers};

/// Converts a winit key event into a [`Keystroke`].
///
/// Returns `None` for key releases, bare modifier presses and keys the
/// keymap cannot represent.
pub fn keystroke_from_key_event(event: &KeyEvent, modifiers: ModifiersState) -> Option<Keystroke> {
    if event.state != ElementState::Pressed {
        return None;
    }
    keystroke_from_logical_key(&event.logical_key, modifiers)
}

/// Converts a winit logical key plus modifier state into a [`Keystroke`].
///
/// `cmd` maps to winit's Super modifier: the Command key on macOS, the
/// Windows/Super key on other platforms.
pub fn keystroke_from_logical_key(key: &Key, modifiers: ModifiersState) -> Option<Keystroke> {
    let key = match key {
        Key::Named(named) => named_key_name(*named)?.to_string(),
        Key::Character(text) if text == " " => "space".to_string(),
        Key::Character(text) => {
            if text.is_empty() {
                return None;
            }
            // Shift may report uppercase characters ("P"); the keymap stores
            // lowercase keys plus an explicit shift modifier.
            text.to_lowercase()
        }
        _ => return None,
    };

    Some(Keystroke {
        modifiers: Modifiers {
            cmd: modifiers.super_key(),
            ctrl: modifiers.control_key(),
            alt: modifiers.alt_key(),
            shift: modifiers.shift_key(),
        },
        key,
    })
}

fn named_key_name(named: NamedKey) -> Option<&'static str> {
    Some(match named {
        NamedKey::Escape => "escape",
        NamedKey::Enter => "enter",
        NamedKey::Tab => "tab",
        NamedKey::Space => "space",
        NamedKey::Backspace => "backspace",
        NamedKey::Delete => "delete",
        NamedKey::Insert => "insert",
        NamedKey::ArrowUp => "up",
        NamedKey::ArrowDown => "down",
        NamedKey::ArrowLeft => "left",
        NamedKey::ArrowRight => "right",
        NamedKey::Home => "home",
        NamedKey::End => "end",
        NamedKey::PageUp => "pageup",
        NamedKey::PageDown => "pagedown",
        NamedKey::F1 => "f1",
        NamedKey::F2 => "f2",
        NamedKey::F3 => "f3",
        NamedKey::F4 => "f4",
        NamedKey::F5 => "f5",
        NamedKey::F6 => "f6",
        NamedKey::F7 => "f7",
        NamedKey::F8 => "f8",
        NamedKey::F9 => "f9",
        NamedKey::F10 => "f10",
        NamedKey::F11 => "f11",
        NamedKey::F12 => "f12",
        // Bare modifiers and everything else have no keystroke form.
        _ => return None,
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn character_with_cmd_shift() {
        let k = keystroke_from_logical_key(
            &Key::Character("p".into()),
            ModifiersState::SUPER | ModifiersState::SHIFT,
        )
        .unwrap();
        assert_eq!(k.to_string(), "cmd-shift-p");
    }

    #[test]
    fn uppercase_character_is_lowercased() {
        let k =
            keystroke_from_logical_key(&Key::Character("P".into()), ModifiersState::SHIFT).unwrap();
        assert_eq!(k.key, "p");
        assert!(k.modifiers.shift);
    }

    #[test]
    fn named_keys_map_to_keystroke_names() {
        let cases = [
            (NamedKey::Escape, "escape"),
            (NamedKey::Enter, "enter"),
            (NamedKey::Tab, "tab"),
            (NamedKey::Space, "space"),
            (NamedKey::Backspace, "backspace"),
            (NamedKey::Delete, "delete"),
            (NamedKey::ArrowUp, "up"),
            (NamedKey::ArrowDown, "down"),
            (NamedKey::ArrowLeft, "left"),
            (NamedKey::ArrowRight, "right"),
            (NamedKey::Home, "home"),
            (NamedKey::End, "end"),
            (NamedKey::PageUp, "pageup"),
            (NamedKey::PageDown, "pagedown"),
            (NamedKey::F5, "f5"),
            (NamedKey::F12, "f12"),
        ];
        for (named, expected) in cases {
            let k =
                keystroke_from_logical_key(&Key::Named(named), ModifiersState::empty()).unwrap();
            assert_eq!(k.key, expected);
            assert!(!k.modifiers.any());
        }
    }

    #[test]
    fn ctrl_maps_to_ctrl_not_cmd() {
        let k = keystroke_from_logical_key(&Key::Character("x".into()), ModifiersState::CONTROL)
            .unwrap();
        assert!(k.modifiers.ctrl);
        assert!(!k.modifiers.cmd);
        assert_eq!(k.to_string(), "ctrl-x");
    }

    #[test]
    fn super_maps_to_cmd() {
        let k =
            keystroke_from_logical_key(&Key::Character("s".into()), ModifiersState::SUPER).unwrap();
        assert!(k.modifiers.cmd);
        assert_eq!(k.to_string(), "cmd-s");
    }

    #[test]
    fn alt_maps_to_alt() {
        let k =
            keystroke_from_logical_key(&Key::Character("a".into()), ModifiersState::ALT).unwrap();
        assert!(k.modifiers.alt);
        assert_eq!(k.to_string(), "alt-a");
    }

    #[test]
    fn bare_modifier_press_yields_none() {
        for named in [
            NamedKey::Shift,
            NamedKey::Control,
            NamedKey::Alt,
            NamedKey::Super,
            NamedKey::Meta,
        ] {
            assert_eq!(
                keystroke_from_logical_key(&Key::Named(named), ModifiersState::empty()),
                None
            );
        }
    }

    #[test]
    fn space_character_maps_to_space_name() {
        let k = keystroke_from_logical_key(&Key::Character(" ".into()), ModifiersState::empty())
            .unwrap();
        assert_eq!(k.key, "space");
    }

    #[test]
    fn converted_keystroke_round_trips_through_parser() {
        let k = keystroke_from_logical_key(
            &Key::Character("k".into()),
            ModifiersState::SUPER | ModifiersState::ALT,
        )
        .unwrap();
        let reparsed: Keystroke = k.to_string().parse().unwrap();
        assert_eq!(reparsed, k);
    }
}
