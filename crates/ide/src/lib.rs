//! plev ide as a library: the three-panel git workspace (`views`), the
//! git-to-view adapters, the `GitSession` that wires a `git::GitClient`
//! to a `WorkspaceView`, and the `Workbench` that owns both and makes the
//! workspace a capability. The `ide` binary is one host; the showcase's
//! Git tab embeds the same workbench inside its own chrome. `agent` serves
//! the workbench to agents through the port (unix only); `record` keeps
//! its session for the inspector (`PLEV_RECORD=1`, debug). `lab_repo` is
//! the laboratory's repository, built in a temp dir: the `ide-lab` bin and
//! the tests that rerun its recorded sessions start from it.

pub mod adapters;
#[cfg(unix)]
pub mod agent;
mod ids;
pub mod lab_repo;
pub mod record;
pub mod session;
pub mod status;
pub mod views;
pub mod workbench;
