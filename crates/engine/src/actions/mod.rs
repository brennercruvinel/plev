//! Typed actions, keystrokes, key contexts and the JSON keymap.
//!
//! This module implements the Zed-style action dispatch model: input becomes
//! a [`Keystroke`], the [`KeymapMatcher`] resolves it against the current
//! [`ContextStack`] into an action *name*, and the [`ActionRegistry`] builds
//! the corresponding boxed [`Action`] for dispatch.
//!
//! The per-frame [`ActionQueue`](crate::input::dispatch::ActionQueue) keeps working
//! for widget-to-parent communication; this module adds the global,
//! declaratively-bound layer on top of it.

pub mod context;
pub mod keymap;
pub mod keystroke;
pub mod shortcuts;
pub mod winit_keys;

pub use context::{ContextStack, KeyContext, Predicate, PredicateParseError};
pub use keymap::{Keymap, KeymapError, KeymapMatcher, MatchResult};
pub use keystroke::{InvalidKeystroke, Keystroke, Modifiers};
pub use winit_keys::{keystroke_from_key_event, keystroke_from_logical_key};

use std::any::Any;
use std::fmt;

use indexmap::IndexMap;

// ---------------------------------------------------------------------------
// Action
// ---------------------------------------------------------------------------

/// A named, dispatchable command.
///
/// Actions are identified by a `"namespace::Name"` string (the keymap binds
/// keystrokes to these names) and dispatched as `Box<dyn Action>`. Handlers
/// recover the concrete type via [`downcast_ref`](dyn Action::downcast_ref).
///
/// Unit actions are declared with the [`actions!`](crate::actions!) macro;
/// actions carrying data implement this trait manually.
pub trait Action: Any {
    /// Stable identifier in the form `"namespace::Name"`.
    fn name(&self) -> &'static str;

    /// Manual dyn-clone: clones the concrete action into a new box.
    fn boxed_clone(&self) -> Box<dyn Action>;

    /// Upcast used to implement downcasting on `dyn Action`.
    fn as_any(&self) -> &dyn Any;
}

impl Clone for Box<dyn Action> {
    fn clone(&self) -> Self {
        self.boxed_clone()
    }
}

impl fmt::Debug for dyn Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Action({})", self.name())
    }
}

impl dyn Action {
    /// `true` when the concrete type of this action is `A`.
    pub fn is<A: Action>(&self) -> bool {
        self.as_any().is::<A>()
    }

    /// Downcasts to a concrete action type.
    pub fn downcast_ref<A: Action>(&self) -> Option<&A> {
        self.as_any().downcast_ref::<A>()
    }
}

/// Declares unit-struct actions within a namespace.
///
/// ```rust
/// engine::actions!(file, [Save, SaveAs, Open]);
///
/// use engine::actions::Action;
/// assert_eq!(Save.name(), "file::Save");
/// ```
#[macro_export]
macro_rules! actions {
    ($namespace:ident, [ $($name:ident),* $(,)? ]) => {
        $(
            #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
            pub struct $name;

            impl $crate::actions::Action for $name {
                fn name(&self) -> &'static str {
                    concat!(stringify!($namespace), "::", stringify!($name))
                }

                fn boxed_clone(&self) -> ::std::boxed::Box<dyn $crate::actions::Action> {
                    ::std::boxed::Box::new(*self)
                }

                fn as_any(&self) -> &dyn ::std::any::Any {
                    self
                }
            }
        )*
    };
}

// ---------------------------------------------------------------------------
// ActionRegistry
// ---------------------------------------------------------------------------

type ActionFactory = Box<dyn Fn() -> Box<dyn Action>>;

/// Explicit name → factory registry used to build actions from the names
/// resolved by the keymap (and to enumerate them, e.g. for a command
/// palette).
#[derive(Default)]
pub struct ActionRegistry {
    factories: IndexMap<String, ActionFactory>,
}

impl ActionRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a factory under an explicit action name. Re-registering a
    /// name replaces the previous factory.
    pub fn register(&mut self, name: impl Into<String>, factory: ActionFactory) {
        self.factories.insert(name.into(), factory);
    }

    /// Registers a `Default`-constructible action under its own name.
    pub fn register_default<A: Action + Default>(&mut self) {
        let name = A::default().name();
        self.register(name, Box::new(|| Box::new(A::default())));
    }

    /// Builds a fresh instance of the action registered under `name`.
    pub fn build(&self, name: &str) -> Option<Box<dyn Action>> {
        self.factories.get(name).map(|factory| factory())
    }

    pub fn contains(&self, name: &str) -> bool {
        self.factories.contains_key(name)
    }

    /// Registered action names in registration order (for the palette).
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.factories.keys().map(String::as_str)
    }

    pub fn len(&self) -> usize {
        self.factories.len()
    }

    pub fn is_empty(&self) -> bool {
        self.factories.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    crate::actions!(test_ns, [Save, Open, Quit]);

    // A manually implemented action carrying data.
    #[derive(Clone, Debug, PartialEq, Eq)]
    struct OpenFile {
        path: String,
    }

    impl Action for OpenFile {
        fn name(&self) -> &'static str {
            "file::OpenFile"
        }

        fn boxed_clone(&self) -> Box<dyn Action> {
            Box::new(self.clone())
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    #[test]
    fn macro_generates_namespaced_names() {
        assert_eq!(Save.name(), "test_ns::Save");
        assert_eq!(Open.name(), "test_ns::Open");
        assert_eq!(Quit.name(), "test_ns::Quit");
    }

    #[test]
    fn macro_accepts_trailing_comma() {
        crate::actions!(trailing, [One, Two,]);
        assert_eq!(One.name(), "trailing::One");
        assert_eq!(Two.name(), "trailing::Two");
    }

    #[test]
    fn boxed_action_reports_name() {
        let action: Box<dyn Action> = Box::new(Save);
        assert_eq!(action.name(), "test_ns::Save");
        assert_eq!(format!("{action:?}"), "Action(test_ns::Save)");
    }

    #[test]
    fn boxed_action_downcasts() {
        let action: Box<dyn Action> = Box::new(Save);
        assert!(action.is::<Save>());
        assert!(!action.is::<Open>());
        assert!(action.downcast_ref::<Save>().is_some());
        assert!(action.downcast_ref::<Open>().is_none());
    }

    #[test]
    fn boxed_clone_preserves_concrete_type_and_data() {
        let action: Box<dyn Action> = Box::new(OpenFile {
            path: "src/main.rs".into(),
        });
        let cloned = action.clone();
        assert_eq!(cloned.name(), "file::OpenFile");
        let downcast = cloned.downcast_ref::<OpenFile>().unwrap();
        assert_eq!(downcast.path, "src/main.rs");
    }

    #[test]
    fn registry_builds_registered_actions() {
        let mut registry = ActionRegistry::new();
        registry.register_default::<Save>();
        registry.register_default::<Open>();

        let action = registry.build("test_ns::Save").unwrap();
        assert_eq!(action.name(), "test_ns::Save");
        assert!(action.is::<Save>());
    }

    #[test]
    fn registry_build_unknown_name_returns_none() {
        let registry = ActionRegistry::new();
        assert!(registry.build("missing::Action").is_none());
    }

    #[test]
    fn registry_explicit_factory() {
        let mut registry = ActionRegistry::new();
        registry.register(
            "file::OpenFile",
            Box::new(|| {
                Box::new(OpenFile {
                    path: "default.rs".into(),
                })
            }),
        );
        let action = registry.build("file::OpenFile").unwrap();
        assert_eq!(
            action.downcast_ref::<OpenFile>().unwrap().path,
            "default.rs"
        );
    }

    #[test]
    fn registry_lists_names_in_registration_order() {
        let mut registry = ActionRegistry::new();
        registry.register_default::<Quit>();
        registry.register_default::<Save>();
        registry.register_default::<Open>();
        let names: Vec<&str> = registry.names().collect();
        assert_eq!(names, ["test_ns::Quit", "test_ns::Save", "test_ns::Open"]);
        assert_eq!(registry.len(), 3);
        assert!(!registry.is_empty());
    }

    #[test]
    fn registry_contains() {
        let mut registry = ActionRegistry::new();
        registry.register_default::<Save>();
        assert!(registry.contains("test_ns::Save"));
        assert!(!registry.contains("test_ns::Open"));
    }

    #[test]
    fn reregistering_replaces_factory() {
        let mut registry = ActionRegistry::new();
        registry.register("a::B", Box::new(|| Box::new(Save)));
        registry.register("a::B", Box::new(|| Box::new(Open)));
        let action = registry.build("a::B").unwrap();
        assert!(action.is::<Open>());
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn keymap_to_registry_end_to_end() {
        // The full pipeline: keystroke → keymap → action name → registry →
        // boxed action.
        let mut registry = ActionRegistry::new();
        registry.register_default::<Save>();

        let keymap = Keymap::from_json(
            r#"[{ "context": "Editor", "bindings": { "cmd-s": "test_ns::Save" } }]"#,
        )
        .unwrap();
        let mut matcher = KeymapMatcher::new(keymap);

        let stack: ContextStack = vec![KeyContext::new("Editor")].into();
        let keystroke: Keystroke = "cmd-s".parse().unwrap();
        let result = matcher.match_keystroke(&keystroke, &stack);
        let MatchResult::Complete(name) = result else {
            panic!("expected Complete, got {result:?}");
        };
        let action = registry.build(&name).unwrap();
        assert!(action.is::<Save>());
    }
}
