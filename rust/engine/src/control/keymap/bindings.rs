//! JSON keymap and multi-stroke keystroke matching.
//!
//! A keymap is an ordered list of sections, each with an optional context
//! predicate and a map of keystroke sequences to action names:
//!
//! ```json
//! [
//!   { "bindings": { "cmd-q": "app::Quit" } },
//!   { "context": "Editor && mode == insert",
//!     "bindings": { "cmd-s": "file::Save", "cmd-k cmd-s": "zed::OpenKeymap" } }
//! ]
//! ```
//!
//! Resolution follows the Zed model: among the bindings whose sequence and
//! predicate match, the one whose predicate matched the *deepest* context
//! wins; on equal depth the binding declared *later* wins (so appending a
//! user keymap overrides defaults). Binding a sequence to `null` disables
//! it at that precedence level.

use std::collections::BTreeMap;
use std::fmt;

use serde::Deserialize;

use super::context::{ContextStack, Predicate};
use super::keystroke::Keystroke;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Error produced while loading a keymap.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeymapError {
    /// The JSON document is malformed or has the wrong shape.
    Json(String),
    /// A binding key is not a valid keystroke sequence.
    Keystroke { binding: String, message: String },
    /// A section's `context` string failed to parse.
    Context { context: String, message: String },
}

impl fmt::Display for KeymapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeymapError::Json(message) => write!(f, "invalid keymap JSON: {message}"),
            KeymapError::Keystroke { binding, message } => {
                write!(f, "invalid binding `{binding}`: {message}")
            }
            KeymapError::Context { context, message } => {
                write!(f, "invalid context `{context}`: {message}")
            }
        }
    }
}

impl std::error::Error for KeymapError {}

// ---------------------------------------------------------------------------
// Keymap
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct KeymapBinding {
    keystrokes: Vec<Keystroke>,
    /// `None` means the sequence is disabled (`null` in JSON).
    action: Option<String>,
    /// `None` means the binding applies in any context.
    predicate: Option<Predicate>,
}

/// Ordered collection of keystroke bindings loaded from JSON sections.
#[derive(Clone, Debug, Default)]
pub struct Keymap {
    bindings: Vec<KeymapBinding>,
}

/// JSON shape of one keymap section.
#[derive(Deserialize)]
struct SectionJson {
    #[serde(default)]
    context: Option<String>,
    bindings: BTreeMap<String, Option<String>>,
}

impl Keymap {
    pub fn new() -> Self {
        Self::default()
    }

    /// Builds a keymap from a JSON document (an array of sections).
    pub fn from_json(json: &str) -> Result<Self, KeymapError> {
        let mut keymap = Self::new();
        keymap.add_json(json)?;
        Ok(keymap)
    }

    /// Appends the sections of a JSON document to this keymap.
    ///
    /// Later additions take precedence over earlier ones at equal context
    /// depth, so loading defaults first and a user keymap second lets the
    /// user override individual bindings.
    pub fn add_json(&mut self, json: &str) -> Result<(), KeymapError> {
        let sections: Vec<SectionJson> =
            serde_json::from_str(json).map_err(|e| KeymapError::Json(e.to_string()))?;
        for section in sections {
            let predicate = match &section.context {
                None => None,
                Some(context) => {
                    Some(Predicate::parse(context).map_err(|e| KeymapError::Context {
                        context: context.clone(),
                        message: e.to_string(),
                    })?)
                }
            };
            for (sequence, action) in section.bindings {
                let keystrokes =
                    Keystroke::parse_sequence(&sequence).map_err(|e| KeymapError::Keystroke {
                        binding: sequence.clone(),
                        message: e.to_string(),
                    })?;
                self.bindings.push(KeymapBinding {
                    keystrokes,
                    action,
                    predicate: predicate.clone(),
                });
            }
        }
        Ok(())
    }

    /// Appends a single binding programmatically. `action: None` disables
    /// the sequence; `context: None` applies in any context.
    pub fn bind(
        &mut self,
        sequence: &str,
        action: Option<&str>,
        context: Option<&str>,
    ) -> Result<(), KeymapError> {
        let keystrokes =
            Keystroke::parse_sequence(sequence).map_err(|e| KeymapError::Keystroke {
                binding: sequence.to_string(),
                message: e.to_string(),
            })?;
        let predicate = match context {
            None => None,
            Some(context) => Some(Predicate::parse(context).map_err(|e| KeymapError::Context {
                context: context.to_string(),
                message: e.to_string(),
            })?),
        };
        self.bindings.push(KeymapBinding {
            keystrokes,
            action: action.map(str::to_string),
            predicate,
        });
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }

    /// Resolves `typed` against this keymap for the given context stack.
    fn resolve(&self, typed: &[Keystroke], stack: &ContextStack) -> ResolveOutcome {
        // Rank: context-free bindings (`None`) sort below any context match;
        // deeper contexts and later declaration indices win.
        let mut best_rank: Option<(Option<usize>, usize)> = None;
        let mut resolved = None;
        let mut pending = false;

        for (index, binding) in self.bindings.iter().enumerate() {
            if binding.keystrokes.len() == typed.len() && binding.keystrokes == typed {
                let depth = match &binding.predicate {
                    None => None,
                    Some(predicate) => match predicate.match_depth(stack) {
                        Some(depth) => Some(depth),
                        None => continue,
                    },
                };
                let rank = (depth, index);
                if best_rank.is_none_or(|best| rank > best) {
                    best_rank = Some(rank);
                    resolved = Some(binding.action.clone());
                }
            } else if binding.keystrokes.len() > typed.len()
                && binding.keystrokes[..typed.len()] == *typed
                && binding.predicate.as_ref().is_none_or(|p| p.matches(stack))
            {
                pending = true;
            }
        }

        ResolveOutcome { resolved, pending }
    }
}

struct ResolveOutcome {
    /// `Some(Some(action))` when a binding won, `Some(None)` when the
    /// winning binding is disabled (`null`), `None` when nothing matched.
    resolved: Option<Option<String>>,
    /// At least one longer sequence in a matching context starts with the
    /// typed strokes.
    pending: bool,
}

// ---------------------------------------------------------------------------
// KeymapMatcher
// ---------------------------------------------------------------------------

/// Result of feeding one keystroke to the [`KeymapMatcher`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MatchResult {
    /// A binding resolved; carries the action name to build via the
    /// [`ActionRegistry`](super::ActionRegistry).
    Complete(String),
    /// The strokes typed so far are a prefix of at least one matching
    /// multi-stroke sequence; more input is needed.
    Pending,
    /// Nothing matched (or the winning binding was `null`); pending state
    /// was reset.
    None,
}

/// Stateful matcher that feeds keystrokes through a [`Keymap`], tracking
/// multi-stroke sequences such as `cmd-k cmd-s`.
#[derive(Clone, Debug, Default)]
pub struct KeymapMatcher {
    keymap: Keymap,
    pending: Vec<Keystroke>,
}

impl KeymapMatcher {
    pub fn new(keymap: Keymap) -> Self {
        Self {
            keymap,
            pending: Vec::new(),
        }
    }

    pub fn keymap(&self) -> &Keymap {
        &self.keymap
    }

    /// Mutable access to the keymap, e.g. to append a user keymap. Resets
    /// any pending multi-stroke state.
    pub fn keymap_mut(&mut self) -> &mut Keymap {
        self.pending.clear();
        &mut self.keymap
    }

    /// `true` while in the middle of a multi-stroke sequence.
    pub fn is_pending(&self) -> bool {
        !self.pending.is_empty()
    }

    /// Discards any pending multi-stroke state (e.g. on focus change or
    /// timeout).
    pub fn clear_pending(&mut self) {
        self.pending.clear();
    }

    /// Feeds one keystroke and resolves it against the keymap.
    ///
    /// A fully matched binding wins over longer sequences that are still
    /// pending (there is no timeout machinery yet). Pending state resets on
    /// [`MatchResult::Complete`] and [`MatchResult::None`].
    pub fn match_keystroke(&mut self, keystroke: &Keystroke, stack: &ContextStack) -> MatchResult {
        self.pending.push(keystroke.clone());
        let outcome = self.keymap.resolve(&self.pending, stack);
        match outcome.resolved {
            Some(Some(action)) => {
                self.pending.clear();
                MatchResult::Complete(action)
            }
            // The winning binding is `null`: the sequence is disabled.
            Some(None) => {
                self.pending.clear();
                MatchResult::None
            }
            None if outcome.pending => MatchResult::Pending,
            None => {
                self.pending.clear();
                MatchResult::None
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actions::context::KeyContext;

    fn ks(s: &str) -> Keystroke {
        s.parse().unwrap()
    }

    fn stack(names: &[&str]) -> ContextStack {
        names
            .iter()
            .map(|n| KeyContext::new(*n))
            .collect::<Vec<_>>()
            .into()
    }

    fn matcher(json: &str) -> KeymapMatcher {
        KeymapMatcher::new(Keymap::from_json(json).unwrap())
    }

    // -- Loading ------------------------------------------------------------

    #[test]
    fn loads_simple_keymap_json() {
        let keymap = Keymap::from_json(
            r#"[{ "bindings": { "cmd-s": "file::Save", "cmd-q": "app::Quit" } }]"#,
        )
        .unwrap();
        assert_eq!(keymap.len(), 2);
    }

    #[test]
    fn rejects_malformed_json() {
        let err = Keymap::from_json("not json").unwrap_err();
        assert!(matches!(err, KeymapError::Json(_)));
    }

    #[test]
    fn rejects_wrong_shape() {
        // Action values must be strings or null.
        let err = Keymap::from_json(r#"[{ "bindings": { "cmd-s": 42 } }]"#).unwrap_err();
        assert!(matches!(err, KeymapError::Json(_)));
    }

    #[test]
    fn rejects_invalid_keystroke_with_context() {
        let err =
            Keymap::from_json(r#"[{ "bindings": { "cmd-bogus-key": "a::B" } }]"#).unwrap_err();
        match err {
            KeymapError::Keystroke { binding, message } => {
                assert_eq!(binding, "cmd-bogus-key");
                assert!(message.contains("unknown"));
            }
            other => panic!("expected Keystroke error, got {other:?}"),
        }
    }

    #[test]
    fn rejects_invalid_context_predicate() {
        let err = Keymap::from_json(
            r#"[{ "context": "Editor &&", "bindings": { "cmd-s": "file::Save" } }]"#,
        )
        .unwrap_err();
        match err {
            KeymapError::Context { context, .. } => assert_eq!(context, "Editor &&"),
            other => panic!("expected Context error, got {other:?}"),
        }
    }

    // -- Basic matching -----------------------------------------------------

    #[test]
    fn matches_simple_binding() {
        let mut m = matcher(r#"[{ "bindings": { "cmd-s": "file::Save" } }]"#);
        assert_eq!(
            m.match_keystroke(&ks("cmd-s"), &stack(&["Workspace"])),
            MatchResult::Complete("file::Save".into())
        );
        assert!(!m.is_pending());
    }

    #[test]
    fn unbound_keystroke_returns_none() {
        let mut m = matcher(r#"[{ "bindings": { "cmd-s": "file::Save" } }]"#);
        assert_eq!(
            m.match_keystroke(&ks("cmd-x"), &stack(&["Workspace"])),
            MatchResult::None
        );
    }

    #[test]
    fn context_free_binding_matches_any_stack() {
        let mut m = matcher(r#"[{ "bindings": { "cmd-q": "app::Quit" } }]"#);
        assert_eq!(
            m.match_keystroke(&ks("cmd-q"), &ContextStack::new()),
            MatchResult::Complete("app::Quit".into())
        );
        assert_eq!(
            m.match_keystroke(&ks("cmd-q"), &stack(&["Workspace", "Editor"])),
            MatchResult::Complete("app::Quit".into())
        );
    }

    #[test]
    fn contextual_binding_requires_matching_context() {
        let json = r#"[{ "context": "Editor", "bindings": { "cmd-s": "editor::Save" } }]"#;
        let mut m = matcher(json);
        assert_eq!(
            m.match_keystroke(&ks("cmd-s"), &stack(&["Workspace", "Editor"])),
            MatchResult::Complete("editor::Save".into())
        );
        assert_eq!(
            m.match_keystroke(&ks("cmd-s"), &stack(&["Workspace", "Terminal"])),
            MatchResult::None
        );
    }

    #[test]
    fn binding_with_attribute_predicate() {
        let json = r#"[{ "context": "Editor && mode == insert",
                         "bindings": { "escape": "editor::NormalMode" } }]"#;
        let mut m = matcher(json);
        let insert: ContextStack =
            vec![KeyContext::new("Editor").with_attr("mode", "insert")].into();
        let normal: ContextStack =
            vec![KeyContext::new("Editor").with_attr("mode", "normal")].into();
        assert_eq!(
            m.match_keystroke(&ks("escape"), &insert),
            MatchResult::Complete("editor::NormalMode".into())
        );
        assert_eq!(m.match_keystroke(&ks("escape"), &normal), MatchResult::None);
    }

    // -- Precedence ---------------------------------------------------------

    #[test]
    fn deeper_context_wins_over_shallower() {
        let json = r#"[
            { "context": "Editor",    "bindings": { "cmd-s": "editor::Save" } },
            { "context": "Workspace", "bindings": { "cmd-s": "workspace::SaveAll" } }
        ]"#;
        let mut m = matcher(json);
        // Editor is deeper than Workspace, so it wins even though the
        // Workspace section is declared later.
        assert_eq!(
            m.match_keystroke(&ks("cmd-s"), &stack(&["Workspace", "Editor"])),
            MatchResult::Complete("editor::Save".into())
        );
        // Without an Editor in the stack, the Workspace binding applies.
        assert_eq!(
            m.match_keystroke(&ks("cmd-s"), &stack(&["Workspace", "Terminal"])),
            MatchResult::Complete("workspace::SaveAll".into())
        );
    }

    #[test]
    fn contextual_binding_beats_context_free() {
        let json = r#"[
            { "context": "Editor", "bindings": { "cmd-s": "editor::Save" } },
            { "bindings": { "cmd-s": "global::Save" } }
        ]"#;
        let mut m = matcher(json);
        // The context-free binding is declared later but still loses to the
        // contextual one.
        assert_eq!(
            m.match_keystroke(&ks("cmd-s"), &stack(&["Editor"])),
            MatchResult::Complete("editor::Save".into())
        );
        assert_eq!(
            m.match_keystroke(&ks("cmd-s"), &stack(&["Terminal"])),
            MatchResult::Complete("global::Save".into())
        );
    }

    #[test]
    fn later_declaration_wins_at_equal_depth() {
        let json = r#"[
            { "context": "Editor", "bindings": { "cmd-s": "editor::SaveOld" } },
            { "context": "Editor", "bindings": { "cmd-s": "editor::SaveNew" } }
        ]"#;
        let mut m = matcher(json);
        assert_eq!(
            m.match_keystroke(&ks("cmd-s"), &stack(&["Editor"])),
            MatchResult::Complete("editor::SaveNew".into())
        );
    }

    #[test]
    fn later_context_free_binding_wins() {
        let json = r#"[
            { "bindings": { "cmd-q": "app::QuitOld" } },
            { "bindings": { "cmd-q": "app::QuitNew" } }
        ]"#;
        let mut m = matcher(json);
        assert_eq!(
            m.match_keystroke(&ks("cmd-q"), &stack(&["Workspace"])),
            MatchResult::Complete("app::QuitNew".into())
        );
    }

    #[test]
    fn user_keymap_appended_overrides_defaults() {
        let defaults = r#"[
            { "bindings": { "cmd-q": "app::Quit" } },
            { "context": "Editor", "bindings": { "cmd-s": "editor::Save" } }
        ]"#;
        let user = r#"[
            { "context": "Editor", "bindings": { "cmd-s": "user::FancySave" } }
        ]"#;
        let mut keymap = Keymap::from_json(defaults).unwrap();
        keymap.add_json(user).unwrap();
        let mut m = KeymapMatcher::new(keymap);
        assert_eq!(
            m.match_keystroke(&ks("cmd-s"), &stack(&["Editor"])),
            MatchResult::Complete("user::FancySave".into())
        );
        // Untouched defaults keep working.
        assert_eq!(
            m.match_keystroke(&ks("cmd-q"), &stack(&["Editor"])),
            MatchResult::Complete("app::Quit".into())
        );
    }

    // -- Null bindings ------------------------------------------------------

    #[test]
    fn null_disables_binding() {
        let defaults = r#"[{ "bindings": { "cmd-q": "app::Quit" } }]"#;
        let user = r#"[{ "bindings": { "cmd-q": null } }]"#;
        let mut keymap = Keymap::from_json(defaults).unwrap();
        keymap.add_json(user).unwrap();
        let mut m = KeymapMatcher::new(keymap);
        assert_eq!(
            m.match_keystroke(&ks("cmd-q"), &stack(&["Workspace"])),
            MatchResult::None
        );
        assert!(!m.is_pending());
    }

    #[test]
    fn deeper_null_shadows_shallower_binding() {
        let json = r#"[
            { "context": "Workspace", "bindings": { "cmd-w": "workspace::Close" } },
            { "context": "Editor",    "bindings": { "cmd-w": null } }
        ]"#;
        let mut m = matcher(json);
        assert_eq!(
            m.match_keystroke(&ks("cmd-w"), &stack(&["Workspace", "Editor"])),
            MatchResult::None
        );
        // Outside the editor, the workspace binding still applies.
        assert_eq!(
            m.match_keystroke(&ks("cmd-w"), &stack(&["Workspace", "Terminal"])),
            MatchResult::Complete("workspace::Close".into())
        );
    }

    // -- Multi-stroke -------------------------------------------------------

    #[test]
    fn multi_stroke_sequence_completes() {
        let mut m = matcher(r#"[{ "bindings": { "cmd-k cmd-s": "zed::OpenKeymap" } }]"#);
        let s = stack(&["Workspace"]);
        assert_eq!(m.match_keystroke(&ks("cmd-k"), &s), MatchResult::Pending);
        assert!(m.is_pending());
        assert_eq!(
            m.match_keystroke(&ks("cmd-s"), &s),
            MatchResult::Complete("zed::OpenKeymap".into())
        );
        assert!(!m.is_pending());
    }

    #[test]
    fn pending_then_wrong_key_resets() {
        let mut m = matcher(r#"[{ "bindings": { "cmd-k cmd-s": "zed::OpenKeymap" } }]"#);
        let s = stack(&["Workspace"]);
        assert_eq!(m.match_keystroke(&ks("cmd-k"), &s), MatchResult::Pending);
        assert_eq!(m.match_keystroke(&ks("x"), &s), MatchResult::None);
        assert!(!m.is_pending());
        // The matcher recovers: the full sequence works afterwards.
        assert_eq!(m.match_keystroke(&ks("cmd-k"), &s), MatchResult::Pending);
        assert_eq!(
            m.match_keystroke(&ks("cmd-s"), &s),
            MatchResult::Complete("zed::OpenKeymap".into())
        );
    }

    #[test]
    fn prefix_only_pends_in_matching_context() {
        let json = r#"[{ "context": "Editor", "bindings": { "cmd-k cmd-s": "a::B" } }]"#;
        let mut m = matcher(json);
        assert_eq!(
            m.match_keystroke(&ks("cmd-k"), &stack(&["Terminal"])),
            MatchResult::None
        );
        assert_eq!(
            m.match_keystroke(&ks("cmd-k"), &stack(&["Editor"])),
            MatchResult::Pending
        );
    }

    #[test]
    fn three_stroke_sequence() {
        let mut m = matcher(r#"[{ "bindings": { "ctrl-x ctrl-y ctrl-z": "deep::Action" } }]"#);
        let s = stack(&["Workspace"]);
        assert_eq!(m.match_keystroke(&ks("ctrl-x"), &s), MatchResult::Pending);
        assert_eq!(m.match_keystroke(&ks("ctrl-y"), &s), MatchResult::Pending);
        assert_eq!(
            m.match_keystroke(&ks("ctrl-z"), &s),
            MatchResult::Complete("deep::Action".into())
        );
    }

    #[test]
    fn complete_binding_wins_over_pending_prefix() {
        // Documented behavior until timeout machinery exists: a fully
        // matched binding fires immediately even when a longer sequence
        // shares the prefix.
        let json = r#"[{ "bindings": {
            "cmd-k": "short::Action",
            "cmd-k cmd-s": "long::Action"
        } }]"#;
        let mut m = matcher(json);
        assert_eq!(
            m.match_keystroke(&ks("cmd-k"), &stack(&["Workspace"])),
            MatchResult::Complete("short::Action".into())
        );
    }

    #[test]
    fn clear_pending_resets_sequence() {
        let mut m = matcher(r#"[{ "bindings": { "cmd-k cmd-s": "a::B" } }]"#);
        let s = stack(&["Workspace"]);
        assert_eq!(m.match_keystroke(&ks("cmd-k"), &s), MatchResult::Pending);
        m.clear_pending();
        assert!(!m.is_pending());
        // "cmd-s" alone no longer completes the sequence.
        assert_eq!(m.match_keystroke(&ks("cmd-s"), &s), MatchResult::None);
    }

    #[test]
    fn deeper_context_wins_for_multi_stroke() {
        let json = r#"[
            { "context": "Workspace", "bindings": { "cmd-k cmd-t": "workspace::Theme" } },
            { "context": "Editor",    "bindings": { "cmd-k cmd-t": "editor::Theme" } }
        ]"#;
        let mut m = matcher(json);
        let s = stack(&["Workspace", "Editor"]);
        assert_eq!(m.match_keystroke(&ks("cmd-k"), &s), MatchResult::Pending);
        assert_eq!(
            m.match_keystroke(&ks("cmd-t"), &s),
            MatchResult::Complete("editor::Theme".into())
        );
    }

    // -- Programmatic bindings ----------------------------------------------

    #[test]
    fn bind_adds_bindings_programmatically() {
        let mut keymap = Keymap::new();
        keymap.bind("cmd-p", Some("palette::Toggle"), None).unwrap();
        keymap
            .bind("cmd-s", Some("editor::Save"), Some("Editor"))
            .unwrap();
        let mut m = KeymapMatcher::new(keymap);
        assert_eq!(
            m.match_keystroke(&ks("cmd-p"), &ContextStack::new()),
            MatchResult::Complete("palette::Toggle".into())
        );
        assert_eq!(
            m.match_keystroke(&ks("cmd-s"), &stack(&["Editor"])),
            MatchResult::Complete("editor::Save".into())
        );
    }

    #[test]
    fn bind_rejects_invalid_input() {
        let mut keymap = Keymap::new();
        assert!(keymap.bind("not-a-key", Some("a::B"), None).is_err());
        assert!(
            keymap
                .bind("cmd-s", Some("a::B"), Some("Editor &&"))
                .is_err()
        );
    }
}
