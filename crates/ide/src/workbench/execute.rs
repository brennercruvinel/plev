//! What the workspace runs: one handler per capability, for every actor.
//! `execute` is the only place where the projected state and the
//! repository change; a click, a key, an agent and a test all end here,
//! through the dispatcher, after their request was checked.

use std::collections::BTreeSet;

use git::GitCommand;
use sem::{Execution, Operable, Outcome, Request, RequestId, Value};

use super::Workbench;

const NO_REPOSITORY: &str = "no repository is open";

/// What an accepted request asks the workspace to do.
enum Op {
    ShowFileDiff(String),
    ShowCommitDiff(String),
    Stage(String),
    Unstage(String),
    Discard(String),
    Ignore(String),
    Open,
    Cancel,
    SetMessage(String),
    /// Commit, expecting the index to stage exactly this set: the one the
    /// actor observed, which the dispatcher found still projected.
    Submit(BTreeSet<String>),
}

impl Workbench {
    fn op(&self, req: &Request) -> Option<Op> {
        let ids = self.workspace.ids.as_ref()?;
        let target = &req.target;
        let path = || ids.path_of(target);
        match req.action.as_str() {
            "open" if *target == ids.form => Some(Op::Open),
            "cancel" if *target == ids.form => Some(Op::Cancel),
            "set_value" if *target == ids.message => match &req.param {
                Some(Value::Text(text)) => Some(Op::SetMessage(text.clone())),
                _ => None,
            },
            "submit" if *target == ids.submit => {
                let observed = req.observed.values.iter().find_map(|(at, value)| {
                    (at.node == ids.changes && at.key.as_str() == "staged")
                        .then_some(value.as_ref())
                        .flatten()
                });
                match observed {
                    Some(Value::Set(staged)) => Some(Op::Submit(staged.clone())),
                    _ => None,
                }
            }
            "show_diff" => path()
                .map(Op::ShowFileDiff)
                .or_else(|| ids.sha_of(target).map(Op::ShowCommitDiff)),
            "stage" => path().map(Op::Stage),
            "unstage" => path().map(Op::Unstage),
            "discard" => path().map(Op::Discard),
            "ignore" => path().map(Op::Ignore),
            _ => None,
        }
    }

    /// Queues `command` on the worker, or says why it cannot.
    fn send(&self, command: GitCommand) -> Result<(), String> {
        let Some(git) = &self.git else {
            return Err(NO_REPOSITORY.to_owned());
        };
        if git.send(command) {
            Ok(())
        } else {
            Err("the git worker is gone".to_owned())
        }
    }

    /// Sends a mutation and, once it left, shows its effect before git
    /// answers (the status refresh after the answer reconciles the view).
    fn mutate(&mut self, command: GitCommand, show: impl FnOnce(&mut Self)) -> Execution {
        match self.send(command) {
            Ok(()) => {
                show(self);
                Execution::Done(Outcome::default())
            }
            Err(why) => Execution::Failed(why),
        }
    }

    /// Drops the rows of `path` and, when the diff showed it, the diff.
    fn drop_path(&mut self, path: &str) {
        self.workspace.unassigned.remove_path(path);
        let shown = self.workspace.ids.as_ref().map(|ids| ids.file(path));
        if shown.is_some() && self.workspace.diff.target == shown {
            self.workspace.diff.clear();
            if let Some(git) = &mut self.git {
                git.forget_diff();
            }
        }
    }

    fn show_file_diff(&mut self, path: &str) -> Execution {
        let target = self.workspace.ids.as_ref().map(|ids| ids.file(path));
        self.workspace.diff.show_file(path, target);
        if let Some(git) = &mut self.git {
            git.diff_file(path);
        }
        Execution::Done(Outcome::default())
    }

    fn show_commit_diff(&mut self, sha: &str) -> Execution {
        let ws = &self.workspace;
        let message = ws
            .stacks
            .stacks
            .iter()
            .flat_map(|s| &s.commits)
            .find(|c| c.sha == sha)
            .map(|c| c.message.clone());
        let Some(message) = message else {
            return Execution::Failed(format!("no commit {sha} in the stacks"));
        };
        let target = ws.ids.as_ref().map(|ids| ids.commit(sha));
        self.workspace.diff.show_commit(&message, sha, target);
        if let Some(git) = &mut self.git {
            git.diff_commit(sha);
        }
        Execution::Done(Outcome::default())
    }

    fn submit(&mut self, expect_staged: BTreeSet<String>) -> Execution {
        let message = self.workspace.commit_form.message().to_owned();
        let command = GitCommand::CommitIf {
            message,
            expect_staged,
        };
        self.mutate(command, |bench| {
            let form = &mut bench.workspace.commit_form;
            form.take_message();
            form.hide();
        })
    }
}

impl Operable for Workbench {
    fn execute(&mut self, _id: RequestId, req: &Request) -> Execution {
        let Some(op) = self.op(req) else {
            return Execution::Failed(format!(
                "the workspace has no {} on {}",
                req.action, req.target
            ));
        };
        let done = || Execution::Done(Outcome::default());
        match op {
            Op::Open => {
                self.workspace.commit_form.show();
                done()
            }
            Op::Cancel => {
                let form = &mut self.workspace.commit_form;
                form.take_message();
                form.hide();
                done()
            }
            Op::SetMessage(text) => {
                self.workspace.commit_form.set_message(&text);
                done()
            }
            Op::ShowFileDiff(path) => self.show_file_diff(&path),
            Op::ShowCommitDiff(sha) => self.show_commit_diff(&sha),
            Op::Stage(path) => self.mutate(GitCommand::Stage { path: path.clone() }, |b| {
                b.workspace.unassigned.set_staged(&path, true);
            }),
            Op::Unstage(path) => self.mutate(GitCommand::Unstage { path: path.clone() }, |b| {
                b.workspace.unassigned.set_staged(&path, false);
            }),
            Op::Discard(path) => self.mutate(GitCommand::Discard { path: path.clone() }, |b| {
                b.drop_path(&path);
            }),
            Op::Ignore(path) => self.mutate(GitCommand::Ignore { path: path.clone() }, |b| {
                b.drop_path(&path);
            }),
            Op::Submit(expect) => self.submit(expect),
        }
    }
}
