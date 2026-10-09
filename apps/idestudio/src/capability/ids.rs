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
    /// `notices`: the toasts, keyed by toast id.
    pub notices: SemId,
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
            notices: SemId::name("notices")?,
            changes,
            stacks,
        })
    }

    /// The toast behind a `notices[<id>]`.
    pub(crate) fn notice_of(&self, id: &SemId) -> Option<u64> {
        key_under(&self.notices, id)?.parse().ok()
    }

    /// `changes.file[<path>]`.
    pub(crate) fn file(&self, path: &str) -> SemId {
        self.file.key(path)
    }

    /// `stacks.commit[<sha>]`.
    pub(crate) fn commit(&self, sha: &str) -> SemId {
        self.commit.key(sha)
    }

    /// The path behind a `changes.file[<path>]`.
    pub(crate) fn path_of(&self, id: &SemId) -> Option<String> {
        key_under(&self.file, id)
    }

    /// The sha behind a `stacks.commit[<sha>]`.
    pub(crate) fn sha_of(&self, id: &SemId) -> Option<String> {
        key_under(&self.commit, id)
    }
}

/// The key of `id` when it is `base[<key>]` and nothing more, with the
/// escapes of the canonical text (`\]`, `\\`) undone.
fn key_under(base: &SemId, id: &SemId) -> Option<String> {
    let inner = id
        .as_str()
        .strip_prefix(base.as_str())?
        .strip_prefix('[')?
        .strip_suffix(']')?;
    let mut key = String::with_capacity(inner.len());
    let mut escaped = false;
    for c in inner.chars() {
        if escaped {
            key.push(c);
            escaped = false;
        } else if c == '\\' {
            escaped = true;
        } else if c == ']' {
            // An unescaped `]` closes the key early: a segment follows.
            return None;
        } else {
            key.push(c);
        }
    }
    (!escaped).then_some(key)
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

    #[test]
    fn a_key_comes_back_from_its_id_escapes_included() {
        let ids = Ids::new().expect("constant ids");
        for path in ["a.txt", "dir/b c.rs", "odd]name", "back\\slash", "[x]"] {
            assert_eq!(ids.path_of(&ids.file(path)).as_deref(), Some(path));
        }
        assert_eq!(ids.sha_of(&ids.commit("f00d")).as_deref(), Some("f00d"));
        assert_eq!(ids.notice_of(&ids.notices.key("7")), Some(7));
        assert_eq!(ids.notice_of(&ids.notices.key("x")), None);
    }

    #[test]
    fn an_id_that_is_not_one_key_under_the_base_has_none() {
        let ids = Ids::new().expect("constant ids");
        let longer = ids.file("a").child("x").expect("name segment");
        assert_eq!(ids.path_of(&longer), None);
        assert_eq!(ids.path_of(&ids.file("a").key("b")), None);
        assert_eq!(ids.path_of(&ids.changes), None);
        assert_eq!(ids.path_of(&ids.commit("a")), None);
    }
}
