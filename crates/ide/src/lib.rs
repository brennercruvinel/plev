//! plev ide as a library: the three-panel git workspace (`views`), the
//! git-to-view adapters, the `GitSession` that wires a `git::GitClient`
//! to a `WorkspaceView`, and the `Workbench` that owns both and makes the
//! workspace a capability. The `ide` binary is one host; the showcase's
//! Git tab embeds the same workbench inside its own chrome.

pub mod adapters;
mod ids;
pub mod session;
pub mod status;
pub mod views;
pub mod workbench;
