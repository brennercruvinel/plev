//! Keystroke parsing and formatting.
//!
//! A [`Keystroke`] is the textual unit of the keymap: `"cmd-shift-p"`,
//! `"escape"`, `"f5"`. Multi-stroke sequences such as `"cmd-k cmd-s"` are
//! space-separated and parsed with [`Keystroke::parse_sequence`].
//!
//! `Display` is symmetric with `FromStr`: parsing the displayed form yields
//! the same keystroke (modifiers are emitted in the canonical order
//! `cmd-ctrl-alt-shift`).

use std::fmt;
use std::str::FromStr;

/// Modifier keys held during a keystroke.
///
/// `cmd` is the Super/Meta modifier: the Command key on macOS and the
/// Windows/Super key elsewhere.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Modifiers {
    pub cmd: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

impl Modifiers {
    pub fn none() -> Self {
        Self::default()
    }

    /// `true` when at least one modifier is held.
    pub fn any(&self) -> bool {
        self.cmd || self.ctrl || self.alt || self.shift
    }
}

/// A single key press plus its modifiers, e.g. `cmd-shift-p`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Keystroke {
    pub modifiers: Modifiers,
    /// Normalized key name: a named key (`"escape"`, `"enter"`, `"f5"`, ...)
    /// or a single lowercased character (`"p"`, `"["`, `"-"`).
    pub key: String,
}

/// Error produced when a keystroke string cannot be parsed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InvalidKeystroke {
    message: String,
}

impl InvalidKeystroke {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for InvalidKeystroke {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid keystroke: {}", self.message)
    }
}

impl std::error::Error for InvalidKeystroke {}

/// Named (non-character) keys accepted by the parser.
const NAMED_KEYS: &[&str] = &[
    "escape",
    "enter",
    "tab",
    "space",
    "backspace",
    "delete",
    "insert",
    "up",
    "down",
    "left",
    "right",
    "home",
    "end",
    "pageup",
    "pagedown",
];

fn is_function_key(key: &str) -> bool {
    key.len() >= 2
        && key.starts_with('f')
        && key[1..].chars().all(|c| c.is_ascii_digit())
        && key[1..].parse::<u32>().is_ok_and(|n| (1..=35).contains(&n))
}

fn is_valid_key(key: &str) -> bool {
    NAMED_KEYS.contains(&key) || is_function_key(key) || key.chars().count() == 1
}

impl FromStr for Keystroke {
    type Err = InvalidKeystroke;

    /// Parses strings like `"cmd-shift-p"`, `"ctrl-x"`, `"f5"`, `"escape"`.
    ///
    /// Modifier prefixes may appear in any order. Accepted modifier names:
    /// `cmd` (aliases `super`, `meta`), `ctrl` (alias `control`), `alt`
    /// (alias `option`) and `shift`. Parsing is case-insensitive.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lowered = s.trim().to_lowercase();
        if lowered.is_empty() {
            return Err(InvalidKeystroke::new("keystroke is empty"));
        }

        let mut modifiers = Modifiers::default();
        let mut rest = lowered.as_str();
        while let Some(idx) = rest.find('-') {
            // A trailing '-' is part of the key (e.g. "ctrl--"), not a
            // modifier separator.
            if idx + 1 >= rest.len() {
                break;
            }
            let flag = match &rest[..idx] {
                "cmd" | "super" | "meta" => &mut modifiers.cmd,
                "ctrl" | "control" => &mut modifiers.ctrl,
                "alt" | "option" => &mut modifiers.alt,
                "shift" => &mut modifiers.shift,
                other => {
                    return Err(InvalidKeystroke::new(format!(
                        "unknown modifier `{other}` in `{s}`"
                    )));
                }
            };
            if *flag {
                return Err(InvalidKeystroke::new(format!(
                    "duplicate modifier `{}` in `{s}`",
                    &rest[..idx]
                )));
            }
            *flag = true;
            rest = &rest[idx + 1..];
        }

        if !is_valid_key(rest) {
            return Err(InvalidKeystroke::new(format!(
                "unknown key `{rest}` in `{s}`"
            )));
        }

        Ok(Keystroke {
            modifiers,
            key: rest.to_string(),
        })
    }
}

impl Keystroke {
    /// Parses a space-separated sequence: `"cmd-k cmd-s"` → two keystrokes.
    pub fn parse_sequence(s: &str) -> Result<Vec<Keystroke>, InvalidKeystroke> {
        let strokes = s
            .split_whitespace()
            .map(Keystroke::from_str)
            .collect::<Result<Vec<_>, _>>()?;
        if strokes.is_empty() {
            return Err(InvalidKeystroke::new("keystroke sequence is empty"));
        }
        Ok(strokes)
    }
}

impl fmt::Display for Keystroke {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.modifiers.cmd {
            write!(f, "cmd-")?;
        }
        if self.modifiers.ctrl {
            write!(f, "ctrl-")?;
        }
        if self.modifiers.alt {
            write!(f, "alt-")?;
        }
        if self.modifiers.shift {
            write!(f, "shift-")?;
        }
        write!(f, "{}", self.key)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn ks(s: &str) -> Keystroke {
        s.parse().unwrap()
    }

    #[test]
    fn parses_single_character() {
        let k = ks("p");
        assert_eq!(k.key, "p");
        assert!(!k.modifiers.any());
    }

    #[test]
    fn parses_cmd_shift_p() {
        let k = ks("cmd-shift-p");
        assert_eq!(k.key, "p");
        assert!(k.modifiers.cmd);
        assert!(k.modifiers.shift);
        assert!(!k.modifiers.ctrl);
        assert!(!k.modifiers.alt);
    }

    #[test]
    fn parses_ctrl_x() {
        let k = ks("ctrl-x");
        assert_eq!(k.key, "x");
        assert!(k.modifiers.ctrl);
        assert!(!k.modifiers.cmd);
    }

    #[test]
    fn parses_all_modifiers() {
        let k = ks("cmd-ctrl-alt-shift-z");
        assert!(k.modifiers.cmd && k.modifiers.ctrl && k.modifiers.alt && k.modifiers.shift);
        assert_eq!(k.key, "z");
    }

    #[test]
    fn modifier_order_is_irrelevant() {
        assert_eq!(ks("shift-cmd-p"), ks("cmd-shift-p"));
        assert_eq!(ks("alt-ctrl-a"), ks("ctrl-alt-a"));
    }

    #[test]
    fn parses_modifier_aliases() {
        assert_eq!(ks("super-s"), ks("cmd-s"));
        assert_eq!(ks("meta-s"), ks("cmd-s"));
        assert_eq!(ks("control-s"), ks("ctrl-s"));
        assert_eq!(ks("option-s"), ks("alt-s"));
    }

    #[test]
    fn parsing_is_case_insensitive() {
        assert_eq!(ks("Cmd-Shift-P"), ks("cmd-shift-p"));
        assert_eq!(ks("ESCAPE"), ks("escape"));
    }

    #[test]
    fn parses_named_keys() {
        for name in [
            "escape",
            "enter",
            "tab",
            "space",
            "backspace",
            "delete",
            "insert",
            "up",
            "down",
            "left",
            "right",
            "home",
            "end",
            "pageup",
            "pagedown",
        ] {
            let k = ks(name);
            assert_eq!(k.key, name);
            assert!(!k.modifiers.any());
        }
    }

    #[test]
    fn parses_function_keys() {
        assert_eq!(ks("f1").key, "f1");
        assert_eq!(ks("f5").key, "f5");
        assert_eq!(ks("f12").key, "f12");
        assert_eq!(ks("f35").key, "f35");
        let k = ks("ctrl-f5");
        assert!(k.modifiers.ctrl);
        assert_eq!(k.key, "f5");
    }

    #[test]
    fn rejects_invalid_function_keys() {
        assert!("f0".parse::<Keystroke>().is_err());
        assert!("f36".parse::<Keystroke>().is_err());
        assert!("f1x".parse::<Keystroke>().is_err());
    }

    #[test]
    fn parses_named_key_with_modifier() {
        let k = ks("cmd-enter");
        assert!(k.modifiers.cmd);
        assert_eq!(k.key, "enter");
    }

    #[test]
    fn parses_minus_as_key() {
        let k = ks("-");
        assert_eq!(k.key, "-");
        let k = ks("ctrl--");
        assert!(k.modifiers.ctrl);
        assert_eq!(k.key, "-");
    }

    #[test]
    fn parses_punctuation_keys() {
        assert_eq!(ks("[").key, "[");
        assert_eq!(ks("cmd-/").key, "/");
        assert_eq!(ks(",").key, ",");
    }

    #[test]
    fn rejects_empty_string() {
        let err = "".parse::<Keystroke>().unwrap_err();
        assert!(err.to_string().contains("empty"));
        assert!("   ".parse::<Keystroke>().is_err());
    }

    #[test]
    fn rejects_unknown_modifier() {
        let err = "hyper-x".parse::<Keystroke>().unwrap_err();
        assert!(err.to_string().contains("unknown modifier"));
        assert!(err.to_string().contains("hyper"));
    }

    #[test]
    fn rejects_unknown_key() {
        let err = "cmd-escapee".parse::<Keystroke>().unwrap_err();
        assert!(err.to_string().contains("unknown key"));
        assert!("abc".parse::<Keystroke>().is_err());
    }

    #[test]
    fn rejects_dangling_modifier() {
        // "cmd-" has no key after the separator.
        assert!("cmd-".parse::<Keystroke>().is_err());
    }

    #[test]
    fn rejects_duplicate_modifier() {
        let err = "cmd-cmd-p".parse::<Keystroke>().unwrap_err();
        assert!(err.to_string().contains("duplicate modifier"));
    }

    #[test]
    fn display_round_trips() {
        for s in [
            "p",
            "cmd-s",
            "cmd-shift-p",
            "ctrl-alt-delete",
            "cmd-ctrl-alt-shift-z",
            "escape",
            "f5",
            "cmd-enter",
            "ctrl--",
            "space",
            "shift-tab",
        ] {
            let k = ks(s);
            assert_eq!(k.to_string(), s, "display should match canonical input");
            assert_eq!(ks(&k.to_string()), k, "parse(display(k)) == k");
        }
    }

    #[test]
    fn display_normalizes_modifier_order() {
        assert_eq!(
            ks("shift-alt-ctrl-cmd-a").to_string(),
            "cmd-ctrl-alt-shift-a"
        );
    }

    #[test]
    fn parses_sequence() {
        let seq = Keystroke::parse_sequence("cmd-k cmd-s").unwrap();
        assert_eq!(seq.len(), 2);
        assert_eq!(seq[0], ks("cmd-k"));
        assert_eq!(seq[1], ks("cmd-s"));
    }

    #[test]
    fn parses_single_stroke_sequence() {
        let seq = Keystroke::parse_sequence("cmd-s").unwrap();
        assert_eq!(seq, vec![ks("cmd-s")]);
    }

    #[test]
    fn sequence_tolerates_extra_whitespace() {
        let seq = Keystroke::parse_sequence("  cmd-k   cmd-s ").unwrap();
        assert_eq!(seq.len(), 2);
    }

    #[test]
    fn empty_sequence_is_an_error() {
        assert!(Keystroke::parse_sequence("").is_err());
        assert!(Keystroke::parse_sequence("   ").is_err());
    }

    #[test]
    fn sequence_propagates_stroke_errors() {
        assert!(Keystroke::parse_sequence("cmd-k bogus-key").is_err());
    }
}
