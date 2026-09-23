//! typed errors of the recorder and of the reader. neither ever panics on
//! what it is given.

use sem::SemId;

/// a recorded stream that cannot be read back as it claims.
#[derive(Debug, PartialEq, thiserror::Error)]
pub enum ReadError {
    /// a change the recorder always stores as a whole snapshot turned up
    /// in a change list.
    #[error("a change list holds {0}, which the recorder stores as a whole snapshot")]
    Structural(&'static str),
    /// a change names a node the state at that point does not hold.
    #[error("a recorded change names {0}, which the state does not hold")]
    UnknownNode(SemId),
    /// a visual chunk does not decode or does not play.
    #[error("a visual chunk does not read back: {0}")]
    Chunk(String),
}

/// a frame the visual track could not take; it is counted
/// (`Stats::frame_errors`) and not recorded.
#[derive(Debug, PartialEq, thiserror::Error)]
pub enum FrameError {
    /// the frame does not fit the monster IR limits (u16 node count and
    /// asset ids of the container).
    #[error("the frame does not fit the monster IR: {0}")]
    Raise(#[from] monster::RaiseError),
}
