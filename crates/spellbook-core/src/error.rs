//! One error type for every operation the core exposes.
//!
//! This replaces the HTTP status codes the old axum layer answered with. The
//! variants are not cosmetic: the UI branches on them exactly where the
//! JavaScript used to branch on the status. `Conflict` in particular carries
//! the "this card is already in the deck, ask before adding another copy"
//! case, which the deck screen turns into a confirmation dialog rather than
//! an error toast.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The resource genuinely isn't there - an unknown card name, a deck id
    /// that no longer exists.
    NotFound(String),
    /// The request itself is wrong: a blank name, a quantity that would break
    /// a deck rule.
    BadRequest(String),
    /// Something on our side failed - almost always the database being
    /// unavailable.
    Internal(String),
    /// Valid, but conflicts with work already in flight or with existing data.
    /// The UI asks the user how to proceed instead of reporting a failure.
    Conflict(String),
    /// An upstream we don't control failed: Scryfall or EDHREC unreachable or
    /// answering garbage. Distinct from `Internal` so the UI can say "try
    /// again later" rather than "something broke".
    Upstream(String),
}

impl Error {
    /// The overwhelmingly common failure: app.db wouldn't open.
    pub fn db_unavailable() -> Self {
        Error::Internal("Banco indisponível".into())
    }

    /// Message shown to the user. Portuguese, like the rest of the app.
    pub fn detail(&self) -> &str {
        match self {
            Error::NotFound(m)
            | Error::BadRequest(m)
            | Error::Internal(m)
            | Error::Conflict(m)
            | Error::Upstream(m) => m,
        }
    }

    pub fn is_conflict(&self) -> bool {
        matches!(self, Error::Conflict(_))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.detail())
    }
}

impl std::error::Error for Error {}

impl From<rusqlite::Error> for Error {
    fn from(e: rusqlite::Error) -> Self {
        Error::Internal(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
