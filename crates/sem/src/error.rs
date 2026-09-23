//! the error model: `SemError` for one violation, returned by the parse
//! functions; `SemErrors` for every violation of a projection at once.

/// one violation of the contract.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum SemError {
    /// a text that is not the canonical form of a `SemId`, or a state ref
    /// without its `:`.
    #[error("invalid id {text:?}: {reason}")]
    InvalidId { text: String, reason: String },
    /// a text that is not a name `[a-z0-9_]+` where a state key or an
    /// action name is expected.
    #[error("invalid key {text:?}: {reason}")]
    InvalidKey { text: String, reason: String },
}
