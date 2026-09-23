//! What a person reads about a request: why the dispatcher (or the index)
//! refused it, what git said when it failed, what another actor asks them
//! to confirm, and that what they declined did not run. Made from the
//! receipt, in the words of the ui.

use engine::theme::Intent;
use sem::{Actor, Change, Receipt, Rejection, Request, SemId, Stage};

use super::Workbench;

/// How an actor is named in a sentence.
fn who(actor: &Actor) -> &str {
    match actor {
        Actor::Human => "you",
        Actor::Agent { name } | Actor::Test { name } => name,
        Actor::Assistive => "assistive technology",
    }
}

impl Workbench {
    /// The toast receipt `receipt` deserves: a refusal says what changed
    /// or why the action is not available, a decline that nothing ran, a
    /// failure what went wrong.
    pub(super) fn notice(&self, receipt: &Receipt) -> Option<(String, Intent)> {
        let req = &receipt.request;
        match receipt.stage()? {
            Stage::Rejected(why) => Some(self.refusal(req, why)),
            Stage::Failed { error } => Some((
                format!("{} failed: {error}", self.doing(req)),
                Intent::Destructive,
            )),
            _ => None,
        }
    }

    /// The action and, when it is a file or a commit, what it acts on:
    /// `stage b.txt`, `submit`.
    fn doing(&self, req: &Request) -> String {
        let action = req.action.as_str();
        match &self.workspace.ids {
            Some(ids)
                if ids.path_of(&req.target).is_some() || ids.sha_of(&req.target).is_some() =>
            {
                format!("{action} {}", self.name(&req.target))
            }
            _ => action.to_owned(),
        }
    }

    /// The modal asking a person to confirm `req`: title, body and the
    /// confirm button's label.
    pub(super) fn confirmation(&self, req: &Request) -> (String, String, &'static str) {
        let asker = who(&req.actor);
        let name = self.name(&req.target);
        match req.action.as_str() {
            "discard" => (
                "Discard changes?".to_owned(),
                format!("{asker} asks to discard all changes to {name}. This cannot be undone."),
                "Discard",
            ),
            action => (
                format!("Allow {action}?"),
                format!("{asker} asks to {action} {name}."),
                "Allow",
            ),
        }
    }

    /// The toast when a person declined `req`, which waited for them.
    fn declined(&self, req: &Request) -> String {
        format!(
            "Declined: {} of {} by {} did not run",
            req.action,
            self.name(&req.target),
            who(&req.actor)
        )
    }

    fn refusal(&self, req: &Request, why: &Rejection) -> (String, Intent) {
        let quiet = Intent::Informational;
        let (reason, intent) = match why {
            // A person's own answer, not a refusal of the workspace.
            Rejection::Declined { .. } => return (self.declined(req), quiet),
            Rejection::Stale { changes } => {
                let moved: Vec<String> = changes.iter().map(|c| self.moved(c)).collect();
                (moved.join("; "), Intent::Destructive)
            }
            Rejection::Disabled { .. } => (self.unavailable(req), quiet),
            Rejection::NotFound { target } => (format!("{} is gone", self.name(target)), quiet),
            Rejection::UnknownAction { target, action } => {
                (format!("{} has no {action}", self.name(target)), quiet)
            }
            Rejection::NotAuthorized { actor } => {
                (format!("{} may not {}", who(actor), req.action), quiet)
            }
            Rejection::Inert { target } => (format!("{} is busy", self.name(target)), quiet),
            Rejection::InvalidParam { reason } => (reason.clone(), quiet),
            Rejection::InvalidProjection { .. } => (
                "the workspace could not be read".to_owned(),
                Intent::Destructive,
            ),
        };
        let head = match &req.actor {
            Actor::Human => "Refused".to_owned(),
            other => format!("Refused {}'s {}", who(other), req.action),
        };
        (format!("{head}: {reason}"), intent)
    }

    /// What one stale change means to a person.
    fn moved(&self, change: &Change) -> String {
        let Change::StateChanged {
            at, added, removed, ..
        } = change
        else {
            return "the workspace changed".to_owned();
        };
        let Some(ids) = &self.workspace.ids else {
            return format!("{at} changed");
        };
        if at.node == ids.changes && at.key.as_str() == "staged" {
            let mut parts: Vec<String> = removed
                .iter()
                .map(|p| format!("{p} left the staged files"))
                .collect();
            parts.extend(added.iter().map(|p| format!("{p} was staged")));
            if !parts.is_empty() {
                return parts.join(", ");
            }
        }
        if at.node == ids.message {
            return "the commit message changed".to_owned();
        }
        if at.node == ids.form {
            return "the commit form was opened or closed".to_owned();
        }
        if at.node == ids.diff {
            return "the diff shown changed".to_owned();
        }
        match ids.path_of(&at.node) {
            Some(path) => format!("{path} changed in the index"),
            None => format!("{at} changed"),
        }
    }

    /// Why a disabled action is not available now.
    fn unavailable(&self, req: &Request) -> String {
        let ws = &self.workspace;
        let name = self.name(&req.target);
        match req.action.as_str() {
            "submit" if !ws.commit_form.visible => "the commit form is closed".to_owned(),
            "submit" if ws.commit_form.message().trim().is_empty() => {
                "the commit message is empty".to_owned()
            }
            "submit" => "nothing is staged".to_owned(),
            "stage" => format!("{name} has nothing to stage"),
            "unstage" => format!("{name} has nothing to unstage"),
            "open" => "the commit form is already open".to_owned(),
            "cancel" | "set_value" => "the commit form is closed".to_owned(),
            action => format!("{action} is not available for {name}"),
        }
    }

    /// A target as a person knows it: a file by its path, a commit by its
    /// short sha, anything else by its id.
    fn name(&self, id: &SemId) -> String {
        let Some(ids) = &self.workspace.ids else {
            return id.to_string();
        };
        if let Some(path) = ids.path_of(id) {
            return path;
        }
        if let Some(sha) = ids.sha_of(id) {
            return sha.chars().take(7).collect();
        }
        id.to_string()
    }
}
