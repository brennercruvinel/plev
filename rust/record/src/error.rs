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

/// a session file that cannot be written or read back. the reader is
/// strict: anything the writer would not have written is one of these,
/// never a panic.
#[derive(Debug, thiserror::Error)]
pub enum FileError {
    #[error("session file: {0}")]
    Io(#[from] std::io::Error),
    #[error("the file ends inside {0}")]
    Truncated(&'static str),
    #[error("not a session: the magic is not MONS")]
    BadMagic,
    #[error(
        "this .monster is an animation (MON0), not a session; play it with \
         monster_player or monster::MonsterPlayer"
    )]
    Animation,
    #[error("unsupported session version {0}; this reader reads 0 and 1")]
    UnsupportedVersion(u16),
    #[error("unsupported flags {0:#06x}; version 0 defines none")]
    UnsupportedFlags(u16),
    #[error("unknown section tag {0:#04x}")]
    UnknownSection(u8),
    #[error("section '{0}' appears twice")]
    DuplicateSection(char),
    #[error("section '{0}' is missing")]
    MissingSection(char),
    #[error("the section index does not tile the file: {0}")]
    SectionLayout(&'static str),
    #[error("sha256 mismatch on section '{0}'")]
    Checksum(char),
    #[error("{0} is not valid utf-8")]
    BadUtf8(&'static str),
    #[error("{0} has bytes past its last field")]
    Trailing(&'static str),
    #[error("event line {line} does not read: {message}")]
    Event { line: usize, message: String },
    #[error("event line {line} goes back in time")]
    EventOrder { line: usize },
    #[error("chunk {chunk} of the index is invalid: {reason}")]
    Index { chunk: usize, reason: &'static str },
    #[error("asset {asset} is invalid: {reason}")]
    Asset { asset: usize, reason: &'static str },
    #[error("chunk {chunk} does not decode as a monster file: {reason}")]
    Chunk { chunk: usize, reason: String },
    /// an id of section B's table does not parse, or is there twice.
    #[error("bound id {at} is invalid: {reason}")]
    BoundId { at: usize, reason: String },
    /// the bindings of a chunk (section B) are not what the writer writes.
    #[error("the bindings of chunk {chunk} are invalid: {reason}")]
    Bindings { chunk: usize, reason: &'static str },
    #[error("the header says the session starts at {header} ms, its content at {content} ms")]
    StartMismatch { header: u64, content: u64 },
    #[error("{0} does not fit its u32 field")]
    TooLarge(&'static str),
    #[error("an event does not serialize: {0}")]
    Serialize(String),
}
