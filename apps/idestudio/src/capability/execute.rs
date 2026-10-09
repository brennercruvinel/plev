//! What the workspace runs: one handler per capability, for every actor.
//! `execute` is the only place where the projected state and the
//! repository change; a click, a key, an agent and a test all end here,
//! through the dispatcher, after their request was checked.

use std::collections::BTreeSet;

use accord::{Execution, Operable, Outcome, Request, RequestId, Value};
use gitdrv::GitCommand;

use super::Workbench;
use super::truth::Inflight;

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
    /// Dismiss a toast.
    Dismiss(u64),
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
            "dismiss" => ids.notice_of(target).map(Op::Dismiss),
            _ => None,
        }
    }

    /// Queues `command` on the worker, or says why it cannot.
    fn send(&mut self, command: GitCommand) -> Result<(), String> {
        let Some(git) = &mut self.git else {
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
    /// Request `id` stays executing until the worker answers, which ends
    /// its receipt (`truth.rs`).
    fn mutate(
        &mut self,
        id: RequestId,
        command: GitCommand,
        waits: Inflight,
        show: impl FnOnce(&mut Self),
    ) -> Execution {
        match self.send(command) {
            Ok(()) => {
                self.inflight.push_back((id, waits));
                show(self);
                Execution::Started
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

    /// Sends the commit with the staged set the actor observed: the worker
    /// checks it against the real index right before `git commit`. The
    /// form keeps its message until the commit exists, so a commit
    /// refused at the index can be asked for again as it is.
    fn submit(&mut self, id: RequestId, expect: BTreeSet<String>) -> Execution {
        let message = self.workspace.commit_form.message().to_owned();
        let command = GitCommand::CommitIf {
            message: message.clone(),
            expect_staged: expect.clone(),
        };
        self.mutate(id, command, Inflight::Commit { message, expect }, |_| {})
    }
}

impl Operable for Workbench {
    /// Runs request `id`. Everything that reaches the repository leaves
    /// for the git worker (`Execution::Started`) and ends when the worker
    /// answers; the rest ends here. What aims at the intent line of the
    /// generated surface runs there first (`generated.rs`).
    fn execute(&mut self, id: RequestId, req: &Request) -> Execution {
        if let Some(done) = self.execute_generated(id, req) {
            return done;
        }
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
            Op::Stage(path) => {
                let command = GitCommand::Stage { path: path.clone() };
                self.mutate(id, command, Inflight::Stage(path.clone()), |b| {
                    b.workspace.unassigned.set_staged(&path, true);
                })
            }
            Op::Unstage(path) => {
                let command = GitCommand::Unstage { path: path.clone() };
                self.mutate(id, command, Inflight::Unstage(path.clone()), |b| {
                    b.workspace.unassigned.set_staged(&path, false);
                })
            }
            Op::Discard(path) => {
                let command = GitCommand::Discard { path: path.clone() };
                self.mutate(id, command, Inflight::Discard(path.clone()), |b| {
                    b.drop_path(&path);
                })
            }
            Op::Ignore(path) => {
                let command = GitCommand::Ignore { path: path.clone() };
                self.mutate(id, command, Inflight::Ignore(path.clone()), |b| {
                    b.drop_path(&path);
                })
            }
            Op::Submit(expect) => self.submit(id, expect),
            Op::Dismiss(notice) => {
                if self.workspace.toasts.dismiss_id(notice) {
                    done()
                } else {
                    Execution::Failed(format!("notice {notice} is not on screen"))
                }
            }
        }
    }
}
