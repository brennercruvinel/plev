//! Lightweight shortcut matching: the "the app wants Cmd+O and Cmd+1..6"
//! case, without standing up the full keymap.
//!
//! Use this when the bindings are a fixed, compile-time list and a plain
//! string id per binding is enough (tab jumps, one global command or
//! two). Reach for the full [`KeymapMatcher`](super::KeymapMatcher)
//! instead when you need context-dependent bindings (`ContextStack`),
//! multi-stroke sequences, user-configurable keymaps, or dispatch into
//! the [`ActionRegistry`](crate::actions::ActionRegistry).
//!
//! ```rust
//! use engine::actions::shortcuts::ShortcutMap;
//! use engine::actions::Keystroke;
//!
//! let map = ShortcutMap::new()
//!     .bind("cmd-o", "open")
//!     .bind("cmd-1", "tab:1");
//! let ks: Keystroke = "cmd-o".parse().unwrap();
//! assert_eq!(map.get(&ks), Some("open"));
//! ```

use winit::keyboard::ModifiersState;

use super::keystroke::Keystroke;
use super::winit_keys::keystroke_from_logical_key;

/// An ordered list of keystroke → id bindings. First match wins (bind
/// order is the priority order).
#[derive(Clone, Debug, Default)]
pub struct ShortcutMap {
    bindings: Vec<(Keystroke, String)>,
}

impl ShortcutMap {
    pub fn new() -> Self {
        Self::default()
    }

    /// Bind a keystroke string (`"cmd-o"`, `"ctrl-shift-p"`, `"f5"`) to
    /// an id. Panics on an unparseable keystroke — bindings are
    /// programmer constants, so a malformed one is a bug, not a runtime
    /// condition (same contract as `"lit".parse::<Keystroke>()` in the
    /// declarative keymap).
    pub fn bind(mut self, keystroke: &str, id: impl Into<String>) -> Self {
        let keystroke = keystroke
            .parse::<Keystroke>()
            .unwrap_or_else(|e| panic!("ShortcutMap::bind: {e}"));
        self.bindings.push((keystroke, id.into()));
        self
    }

    /// The id of the first binding matching `keystroke`.
    pub fn get(&self, keystroke: &Keystroke) -> Option<&str> {
        self.bindings
            .iter()
            .find(|(ks, _)| ks == keystroke)
            .map(|(_, id)| id.as_str())
    }

    /// winit convenience: match a logical key plus modifier state (what
    /// the app shell already tracks from `WindowEvent::ModifiersChanged`).
    pub fn match_logical_key(
        &self,
        key: &winit::keyboard::Key,
        modifiers: ModifiersState,
    ) -> Option<&str> {
        let keystroke = keystroke_from_logical_key(key, modifiers)?;
        self.get(&keystroke)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binds_and_matches_exact_keystrokes() {
        let map = ShortcutMap::new()
            .bind("cmd-o", "open")
            .bind("cmd-1", "tab:1")
            .bind("cmd-6", "tab:6");
        let ks = |s: &str| s.parse::<Keystroke>().unwrap();
        assert_eq!(map.get(&ks("cmd-o")), Some("open"));
        assert_eq!(map.get(&ks("cmd-1")), Some("tab:1"));
        assert_eq!(map.get(&ks("cmd-6")), Some("tab:6"));
        // Extra or missing modifiers are NOT a match.
        assert_eq!(map.get(&ks("o")), None);
        assert_eq!(map.get(&ks("cmd-shift-o")), None);
        assert_eq!(map.get(&ks("cmd-2")), None);
    }

    #[test]
    fn first_binding_wins_on_duplicates() {
        let map = ShortcutMap::new()
            .bind("cmd-o", "first")
            .bind("cmd-o", "second");
        let ks = "cmd-o".parse::<Keystroke>().unwrap();
        assert_eq!(map.get(&ks), Some("first"));
    }

    #[test]
    #[should_panic(expected = "ShortcutMap::bind")]
    fn invalid_keystroke_panics_at_bind_time() {
        let _ = ShortcutMap::new().bind("cmd-", "broken");
    }

    #[test]
    fn matches_winit_logical_keys() {
        let map = ShortcutMap::new().bind("cmd-o", "open");
        let mods = ModifiersState::SUPER;
        let key = winit::keyboard::Key::Character("o".into());
        assert_eq!(map.match_logical_key(&key, mods), Some("open"));
        // Shift reports "O"; the bridge normalizes to lowercase + shift,
        // which must not match the plain cmd-o binding.
        let key = winit::keyboard::Key::Character("O".into());
        assert_eq!(
            map.match_logical_key(&key, mods | ModifiersState::SHIFT),
            None
        );
        // No modifiers: no match.
        assert_eq!(map.match_logical_key(&key, ModifiersState::default()), None);
    }
}
