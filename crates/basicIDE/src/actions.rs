//! Keymap-dispatched application actions.
//!
//! Keyboard input goes winit `KeyEvent` → `Keystroke` → [`KeymapMatcher`]
//! (with the current context stack) → action name → [`ActionRegistry`] →
//! `App::dispatch`. The bindings live in `keymap.json` (embedded), so
//! shortcuts are data, not `match` arms — a user keymap can later be
//! appended on top of the default one to override bindings.

use plev::actions::{ActionRegistry, Keymap};

plev::actions!(app, [Quit]);
plev::actions!(theme, [Toggle]);
plev::actions!(nav, [Up, Down]);
plev::actions!(file, [ShowDiff, Stage, Discard]);
plev::actions!(commit, [OpenForm, Submit, Cancel, Backspace]);
plev::actions!(overlay, [Close]);

/// All actions the app can dispatch, in palette-friendly order.
pub fn registry() -> ActionRegistry {
    let mut registry = ActionRegistry::new();
    registry.register_default::<Quit>();
    registry.register_default::<Toggle>();
    registry.register_default::<Up>();
    registry.register_default::<Down>();
    registry.register_default::<ShowDiff>();
    registry.register_default::<Stage>();
    registry.register_default::<Discard>();
    registry.register_default::<OpenForm>();
    registry.register_default::<Submit>();
    registry.register_default::<Cancel>();
    registry.register_default::<Backspace>();
    registry.register_default::<Close>();
    registry
}

/// The embedded default keymap (same shortcuts the hardcoded handler had:
/// T, C, arrows, Enter, Esc — plus S/D for stage/discard).
pub fn default_keymap() -> Keymap {
    Keymap::from_json(include_str!("keymap.json")).expect("embedded keymap.json is valid")
}

#[cfg(test)]
mod tests {
    use super::*;
    use plev::actions::{ContextStack, KeyContext, KeymapMatcher, MatchResult};

    fn stack(names: &[&str]) -> ContextStack {
        names
            .iter()
            .map(|n| KeyContext::new(*n))
            .collect::<Vec<_>>()
            .into()
    }

    fn resolve(matcher: &mut KeymapMatcher, key: &str, contexts: &[&str]) -> MatchResult {
        matcher.match_keystroke(&key.parse().unwrap(), &stack(contexts))
    }

    /// Every action name referenced by the embedded keymap must be buildable
    /// from the registry — otherwise a shortcut silently does nothing.
    #[test]
    fn every_keymap_action_is_registered() {
        let registry = registry();
        let json = include_str!("keymap.json");
        let mut checked = 0;
        for token in json.split('"').filter(|t| t.contains("::")) {
            assert!(
                registry.contains(token),
                "keymap binds `{token}` but the registry doesn't know it"
            );
            checked += 1;
        }
        assert!(checked >= 10, "expected to find bindings in keymap.json");
    }

    #[test]
    fn workspace_bindings_resolve() {
        let mut matcher = KeymapMatcher::new(default_keymap());
        for (key, action) in [
            ("escape", "app::Quit"),
            ("t", "theme::Toggle"),
            ("shift-t", "theme::Toggle"),
            ("c", "commit::OpenForm"),
            ("up", "nav::Up"),
            ("down", "nav::Down"),
            ("enter", "file::ShowDiff"),
            ("s", "file::Stage"),
            ("d", "file::Discard"),
        ] {
            assert_eq!(
                resolve(&mut matcher, key, &["Workspace"]),
                MatchResult::Complete(action.into()),
                "binding for `{key}`"
            );
        }
    }

    /// Deeper contexts override Workspace bindings (Zed precedence model).
    #[test]
    fn escape_depends_on_context_depth() {
        let mut matcher = KeymapMatcher::new(default_keymap());
        assert_eq!(
            resolve(&mut matcher, "escape", &["Workspace"]),
            MatchResult::Complete("app::Quit".into())
        );
        assert_eq!(
            resolve(&mut matcher, "escape", &["Workspace", "CommitForm"]),
            MatchResult::Complete("commit::Cancel".into())
        );
        assert_eq!(
            resolve(
                &mut matcher,
                "escape",
                &["Workspace", "CommitForm", "Overlay"]
            ),
            MatchResult::Complete("overlay::Close".into())
        );
    }

    /// While the commit form is open, list navigation is disabled (`null`
    /// bindings) and Enter submits instead of opening a diff.
    #[test]
    fn commit_form_disables_navigation() {
        let mut matcher = KeymapMatcher::new(default_keymap());
        let contexts = ["Workspace", "CommitForm"];
        assert_eq!(resolve(&mut matcher, "up", &contexts), MatchResult::None);
        assert_eq!(resolve(&mut matcher, "down", &contexts), MatchResult::None);
        assert_eq!(
            resolve(&mut matcher, "enter", &contexts),
            MatchResult::Complete("commit::Submit".into())
        );
    }
}
