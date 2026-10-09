//! `WireError`, what can go wrong outside the protocol: the socket, its
//! directory, the app name, a reply that does not parse, a closed
//! connection. a refusal of the protocol is a `ProtoError`, carried here
//! when a client meets one.

use std::io;
use std::path::PathBuf;

use crate::protocol::ProtoError;

#[derive(Debug, thiserror::Error)]
pub enum WireError {
    /// an app name that cannot name a socket file: 1 to 32 characters of
    /// `[A-Za-z0-9_-]`.
    #[error("app name {0:?} is not 1 to 32 characters of [A-Za-z0-9_-]")]
    BadApp(String),
    /// a socket or file operation failed.
    #[error("{what}: {source}")]
    Io {
        what: String,
        #[source]
        source: io::Error,
    },
    /// the socket directory exists and is not a directory this user owns
    /// with mode 0700 (a symlink, a file, another user's directory).
    #[error("{0} is not a private directory of this user; the agent port stays closed")]
    NotPrivate(PathBuf),
    /// a file where the socket goes that is not a socket.
    #[error("{0} exists and is not a socket; it is left alone")]
    NotASocket(PathBuf),
    /// no app listens under that name (or at all).
    #[error("no plev app with an agent port{}", named(.0))]
    NoApp(Option<String>),
    /// more than one app listens under that name: pass the socket path.
    #[error("{} apps listen{}: {}; pass --app with a socket path", .found.len(), named(.name), list(.found))]
    Ambiguous {
        name: Option<String>,
        found: Vec<PathBuf>,
    },
    /// the app refused a message.
    #[error(transparent)]
    Refused(#[from] ProtoError),
    /// the app sent a line this client cannot read.
    #[error("unreadable reply: {0}")]
    BadReply(String),
    /// the connection ended before the reply.
    #[error("the app closed the connection")]
    Closed,
    /// no reply in time.
    #[error("no reply within {0} ms")]
    Timeout(u64),
}

fn named(name: &Option<String>) -> String {
    name.as_ref()
        .map(|n| format!(" named {n:?}"))
        .unwrap_or_default()
}

fn list(found: &[PathBuf]) -> String {
    let names: Vec<String> = found.iter().map(|p| p.display().to_string()).collect();
    names.join(", ")
}
