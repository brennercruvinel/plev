//! plev ide as a library: the three-panel git workspace (`views`), the
//! git-to-view adapters and the `GitSession` that wires a `git::GitClient`
//! to a `WorkspaceView`. The `ide` binary is one host; the showcase's Git
//! tab embeds the same workspace inside its own chrome.

pub mod adapters;
pub mod session;
pub mod status;
pub mod views;
