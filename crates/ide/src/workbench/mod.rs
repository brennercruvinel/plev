//! The git workspace as a capability: the workbench owns the views
//! (`WorkspaceView`) and the git side (`GitSession`), and says what they
//! mean (`sem::Semantics`, `project.rs`). The ide binary and the
//! showcase's Git page both hold one, so neither carries its own copy of
//! the wiring.

mod project;
#[cfg(test)]
mod tests;

use std::path::Path;

use git::GitClient;
use sem::{SemErrors, Semantics, Snapshot, SnapshotBuilder};

use crate::session::GitSession;
use crate::views::workspace::WorkspaceView;

pub use project::DIFF_LINES;

pub struct Workbench {
    workspace: WorkspaceView,
    /// The repository, once connected; a workbench without one draws an
    /// empty workspace.
    git: Option<GitSession>,
}

impl Workbench {
    /// A workspace of `vw` by `vh` logical pixels, not connected yet.
    pub fn new(vw: f32, vh: f32) -> Self {
        Self {
            workspace: WorkspaceView::new(vw, vh),
            git: None,
        }
    }

    /// Takes the repository behind `client` and starts loading it (the
    /// results come back as the client's events, for [`Workbench::apply`]).
    pub fn connect(&mut self, client: GitClient) {
        let git = GitSession::new(client);
        self.workspace.repo_label = git.repo_label();
        git.refresh();
        self.git = Some(git);
    }

    /// The working tree root of the connected repository.
    pub fn workdir(&self) -> Option<&Path> {
        self.git.as_ref().map(|git| git.client().workdir())
    }

    /// Reloads status, log and branches (results stream back as events).
    pub fn refresh(&self) {
        if let Some(git) = &self.git {
            git.refresh();
        }
    }

    pub fn workspace(&self) -> &WorkspaceView {
        &self.workspace
    }

    /// The views, for what is not a capability: layout, hover, scroll,
    /// drag, theme, typing into the commit field.
    pub fn workspace_mut(&mut self) -> &mut WorkspaceView {
        &mut self.workspace
    }

    /// The workspace projected now.
    pub fn snapshot(&self) -> Result<Snapshot, SemErrors> {
        let mut b = SnapshotBuilder::new();
        self.project(&mut b);
        b.finish()
    }
}
