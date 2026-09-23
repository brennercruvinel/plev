//! the error model: `SemError` for one violation, returned by the parse
//! functions; `SemErrors` for every violation of a projection at once.

/// one violation of the contract.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum SemError {
    /// a text that is not the canonical form of a `SemId`.
    #[error("invalid id {text:?}: {reason}")]
    InvalidId { text: String, reason: String },
}
