//! The logical identities of the git workspace: what the workbench
//! projects (`crate::workbench`) and what the views bind to the rects they
//! draw and hit-test with. A file is keyed by its path and a commit by its
//! sha, never by a row index, so a capability keeps its id while its row
//! moves, scrolls away or comes back.

use sem::{SemError, SemId};

pub(crate) struct Ids {
    /// `changes`: the changed files.
    pub changes: SemId,
    /// `changes.file`, keyed by path.
    pub file: SemId,
    /// `diff`: the diff panel.
    pub diff: SemId,
    /// `commit.form`.
    pub form: SemId,
    /// `commit.message`.
    pub message: SemId,
    /// `commit.submit`.
    pub submit: SemId,
    /// `stacks`: the branches and their commits.
    pub stacks: SemId,
    /// `stacks.commit`, keyed by sha.
    pub commit: SemId,
}

impl Ids {
    pub(crate) fn new() -> Result<Self, SemError> {
        let changes = SemId::name("changes")?;
        let commit = SemId::name("commit")?;
        let stacks = SemId::name("stacks")?;
        Ok(Self {
            file: changes.child("file")?,
            diff: SemId::name("diff")?,
            form: commit.child("form")?,
            message: commit.child("message")?,
            submit: commit.child("submit")?,
            commit: stacks.child("commit")?,
            changes,
            stacks,
        })
    }

    /// `changes.file[<path>]`.
    pub(crate) fn file(&self, path: &str) -> SemId {
        self.file.key(path)
    }

    /// `stacks.commit[<sha>]`.
    pub(crate) fn commit(&self, sha: &str) -> SemId {
        self.commit.key(sha)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_constant_ids_parse() {
        let ids = Ids::new().expect("constant ids");
        assert_eq!(ids.form.as_str(), "commit.form");
        assert_eq!(ids.file("src/lib.rs").as_str(), "changes.file[src/lib.rs]");
        assert_eq!(ids.commit("abc").as_str(), "stacks.commit[abc]");
    }
}
