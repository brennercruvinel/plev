//! the name grammar `[a-z0-9_]+`, shared by `SemId` name segments, state
//! keys and action names, and the two name types built on it.

use std::borrow::Borrow;
use std::fmt;
use std::str::FromStr;
use std::sync::Arc;

use crate::failures::SemError;
use crate::encoding::text_serde;

/// true for a character a name may hold.
pub(crate) fn is_name_char(c: char) -> bool {
    c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'
}

/// why `text` is not a name, or `None` when it is one.
pub(crate) fn name_fault(text: &str) -> Option<String> {
    if text.is_empty() {
        return Some("empty name".to_owned());
    }
    text.char_indices()
        .find(|&(_, c)| !is_name_char(c))
        .map(|(at, c)| format!("invalid character {c:?} at byte {at}"))
}

/// a validated name type: `parse` rejects with `SemError::InvalidKey`, the
/// wire form is the text itself.
macro_rules! name_type {
    ($(#[$doc:meta])* $ty:ident, $what:literal) => {
        $(#[$doc])*
        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $ty(Arc<str>);

        impl $ty {
            /// parses a name `[a-z0-9_]+`.
            pub fn parse(text: &str) -> Result<Self, SemError> {
                match name_fault(text) {
                    None => Ok(Self(Arc::from(text))),
                    Some(reason) => Err(SemError::InvalidKey {
                        text: text.to_owned(),
                        reason: format!("{}: {reason}", $what),
                    }),
                }
            }

            /// the name as text.
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $ty {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl FromStr for $ty {
            type Err = SemError;

            fn from_str(text: &str) -> Result<Self, Self::Err> {
                Self::parse(text)
            }
        }

        impl TryFrom<&str> for $ty {
            type Error = SemError;

            fn try_from(text: &str) -> Result<Self, Self::Error> {
                Self::parse(text)
            }
        }

        impl TryFrom<String> for $ty {
            type Error = SemError;

            fn try_from(text: String) -> Result<Self, Self::Error> {
                Self::parse(&text)
            }
        }

        impl From<&$ty> for $ty {
            fn from(name: &$ty) -> Self {
                name.clone()
            }
        }

        impl AsRef<str> for $ty {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }

        impl Borrow<str> for $ty {
            fn borrow(&self) -> &str {
                &self.0
            }
        }

        text_serde!($ty);
    };
}

name_type!(
    /// the name of one piece of a node's state, `[a-z0-9_]+`:
    /// `staged`, `text`, `checked`.
    StateKey,
    "state key"
);

name_type!(
    /// the name of an action a node accepts, `[a-z0-9_]+`: `submit`,
    /// `toggle`. the key binding in `engine::actions` stays a binding; this is
    /// the capability a binding can target.
    ActionName,
    "action name"
);
