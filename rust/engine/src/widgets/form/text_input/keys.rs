//! The keys a single-line field understands beyond characters: the
//! editing keys and the clipboard chords. Apps map their key events onto
//! these and hand them to the field ([`super::super::TextField`]).

/// Non-character editing keys a field understands. Apps map their key
/// events onto these; Enter and Tab stay with the screen (submit and
/// focus traversal are not the field's business). Left and Right move on
/// screen (in a right-to-left run Left goes forward in the text); the
/// `Select` variants are the same keys with Shift, extending the
/// selection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditKey {
    Backspace,
    Delete,
    Left,
    Right,
    Home,
    End,
    SelectAll,
    SelectLeft,
    SelectRight,
    SelectHome,
    SelectEnd,
}

/// The clipboard keys of a field: the primary modifier (cmd on macOS,
/// ctrl elsewhere) with c, x, v or a. Apps map their chords onto these
/// ([`ClipKey::of`]) and hand the field their clipboard
/// ([`TextField::clip`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClipKey {
    Copy,
    Cut,
    Paste,
    SelectAll,
}

impl ClipKey {
    /// The clipboard key a character names with the primary modifier held
    /// (either case); `None` for any other.
    pub fn of(key: &str) -> Option<Self> {
        Some(match key {
            "c" | "C" => Self::Copy,
            "x" | "X" => Self::Cut,
            "v" | "V" => Self::Paste,
            "a" | "A" => Self::SelectAll,
            _ => return None,
        })
    }
}
