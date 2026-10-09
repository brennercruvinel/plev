//! the one rule that keeps a secret inside the process. a `secret` field
//! already leaves the snapshot as its length
//! (docs/ADR/semantic-contract-as-pure-projection.md); what travels beside
//! the snapshot does not: a request's parameter goes as given, and so can
//! an observed value, a stale delta, an outcome or an error text. whoever
//! carries a request or a stage out of the process (the agent port, the
//! recorder) reduces everything aimed at a secret node here, with the
//! snapshot's own rule: `Text` becomes `Int(chars)`, `List` and `Set`
//! become `Int(items)`, `Bool`, `Int` and `Num` stay; a free text about a
//! secret target (an error, a refusal reason, an agent's question) is
//! replaced by `REDACTED`.
//!
//! which node is secret is read from a snapshot at one instant, the one
//! the caller holds as the state now. a node that snapshot does not hold,
//! or no snapshot at all, counts as secret: the rule fails closed. pure,
//! like the rest of the crate: no I/O, no clock.

use crate::decision::Rejection;
use crate::mutation::Change;
use crate::identity::SemId;
use crate::receipts::{Outcome, Stage};
use crate::requests::Request;
use crate::snapshot::Snapshot;
use crate::property::Value;

/// what replaces a free text about a secret target, which could quote it.
pub const REDACTED: &str = "redacted: the target is a secret field";

/// true when `state` marks `id` secret or does not hold it: fail closed.
pub fn is_secret(state: Option<&Snapshot>, id: &SemId) -> bool {
    state
        .and_then(|s| s.get(id))
        .is_none_or(|node| node.constraints.secret)
}

/// a secret value as the snapshot keeps it: its length (`Text` -> chars,
/// `List` and `Set` -> items; `Bool`, `Int` and `Num` as they are).
pub fn length(v: &Value) -> Value {
    let n = match v {
        Value::Text(text) => text.chars().count(),
        Value::List(items) => items.len(),
        Value::Set(items) => items.len(),
        Value::Bool(_) | Value::Int(_) | Value::Num(_) => return v.clone(),
    };
    Value::Int(i64::try_from(n).unwrap_or(i64::MAX))
}

/// the secret nodes that matter to one request, read from `state` at one
/// instant: the request's own nodes (its target and every node its
/// observed values name) that `state` marks secret or does not hold, and
/// every node `state` marks secret. captured once, it redacts every stage
/// of that request the same way, whatever the app projects later.
#[derive(Clone, Debug, Default)]
pub struct Secrets {
    /// sorted, without repeats.
    nodes: Vec<SemId>,
}

impl Secrets {
    pub fn of(req: &Request, state: Option<&Snapshot>) -> Secrets {
        let named = std::iter::once(&req.target)
            .chain(req.observed.values.iter().map(|(at, _)| &at.node))
            .filter(|id| is_secret(state, id));
        let marked = state
            .into_iter()
            .flat_map(|s| s.nodes().values())
            .filter(|node| node.constraints.secret)
            .map(|node| &node.id);
        let mut nodes: Vec<SemId> = named.chain(marked).cloned().collect();
        nodes.sort();
        nodes.dedup();
        Secrets { nodes }
    }

    /// true when `id` is one of the secret nodes captured.
    pub fn holds(&self, id: &SemId) -> bool {
        self.nodes.binary_search(id).is_ok()
    }

    /// the request with its parameter (when the target is secret) and
    /// every observed value of a secret node as their length; an agent's
    /// question about a secret target (`Request::ask`) is a free text that
    /// could quote it, and becomes `REDACTED`.
    pub fn request(&self, req: &Request) -> Request {
        let mut out = req.clone();
        if self.holds(&req.target) {
            out.param = out.param.as_ref().map(length);
            out.ask = out.ask.as_ref().map(|_| REDACTED.to_owned());
        }
        for (at, seen) in &mut out.observed.values {
            if self.holds(&at.node) {
                *seen = seen.as_ref().map(length);
            }
        }
        out
    }

    /// the stage with what it says about secret state reduced: a stale
    /// delta on a secret node keeps lengths and loses its items; the
    /// refusal reason of a parameter, the values of an outcome and the
    /// error of a secret target are not kept. who acted, in a stale
    /// refusal, goes as it is: request ids, actors, targets and action
    /// names, no value.
    pub fn stage(&self, target: &SemId, stage: &Stage) -> Stage {
        let secret = self.holds(target);
        match stage {
            Stage::Rejected(Rejection::Stale { changes, acted }) => {
                Stage::Rejected(Rejection::Stale {
                    changes: changes.iter().map(|c| self.change(c)).collect(),
                    acted: acted.clone(),
                })
            }
            Stage::Rejected(Rejection::InvalidParam { .. }) if secret => {
                Stage::Rejected(Rejection::InvalidParam {
                    reason: REDACTED.to_owned(),
                })
            }
            Stage::Completed(outcome) if secret => Stage::Completed(Outcome {
                result: outcome
                    .result
                    .iter()
                    .map(|(name, v)| (name.clone(), length(v)))
                    .collect(),
            }),
            Stage::Failed { .. } if secret => Stage::Failed {
                error: REDACTED.to_owned(),
            },
            other => other.clone(),
        }
    }

    fn change(&self, c: &Change) -> Change {
        match c {
            Change::StateChanged {
                at, before, after, ..
            } if self.holds(&at.node) => Change::StateChanged {
                at: at.clone(),
                before: before.as_ref().map(length),
                after: after.as_ref().map(length),
                added: Vec::new(),
                removed: Vec::new(),
            },
            other => other.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use indexmap::IndexMap;

    use super::{REDACTED, Secrets, is_secret, length};
    use crate::{
        Acted, Actor, Change, Constraints, Observation, Outcome, Rejection, Request, RequestId,
        Role, Snapshot, SnapshotBuilder, Stage, StateRef, Value,
    };

    /// a login form: a clear user field, a secret password, a submit that
    /// reads both.
    fn login(password: &str) -> Snapshot {
        let mut b = SnapshotBuilder::new();
        b.node("login.user", Role::TextInput, "user")
            .state("text", "ana")
            .action("set");
        b.node("login.password", Role::TextInput, "password")
            .state("text", password)
            .constraints(Constraints {
                secret: true,
                ..Constraints::default()
            })
            .action("set");
        b.node("login.submit", Role::Button, "sign in")
            .action("submit")
            .reads("login.user:text")
            .reads("login.password:text");
        b.finish().unwrap()
    }

    fn sid(text: &str) -> crate::SemId {
        text.parse().unwrap()
    }

    fn at(text: &str) -> StateRef {
        StateRef::parse(text).unwrap()
    }

    fn set(req: &Snapshot, target: &str, param: &str) -> Request {
        Request::observe(req, Actor::Human, target, "set", Some(param.into())).unwrap()
    }

    #[test]
    fn a_length_is_what_the_snapshot_keeps_of_a_secret() {
        assert_eq!(length(&Value::Text("héllo".into())), Value::Int(5));
        assert_eq!(length(&Value::Text(String::new())), Value::Int(0));
        assert_eq!(
            length(&Value::List(vec!["a".into(), "a".into()])),
            Value::Int(2)
        );
        assert_eq!(
            length(&Value::Set(BTreeSet::from(["x".to_owned()]))),
            Value::Int(1)
        );
        for scalar in [Value::Bool(true), Value::Int(-3), Value::Num(0.5)] {
            assert_eq!(length(&scalar), scalar);
        }
    }

    #[test]
    fn a_node_is_secret_when_marked_or_unknown() {
        let snap = login("hunter2");
        assert!(is_secret(Some(&snap), &sid("login.password")));
        assert!(!is_secret(Some(&snap), &sid("login.user")));
        assert!(is_secret(Some(&snap), &sid("nowhere")), "unknown: closed");
        assert!(is_secret(None, &sid("login.user")), "no state: closed");
    }

    #[test]
    fn the_secrets_of_a_request_are_its_secret_or_unknown_nodes_and_every_marked_one() {
        let snap = login("hunter2");
        let submit = Request::observe(&snap, Actor::Human, "login.submit", "submit", None).unwrap();
        let s = Secrets::of(&submit, Some(&snap));
        assert!(s.holds(&sid("login.password")), "observed and marked");
        assert!(!s.holds(&sid("login.submit")), "the target is clear");
        assert!(!s.holds(&sid("login.user")), "observed and clear");
        assert!(!s.holds(&sid("nowhere")), "not named, not marked");

        // a request on a clear node still captures the marked ones.
        let user = set(&snap, "login.user", "bob");
        let s = Secrets::of(&user, Some(&snap));
        assert!(!s.holds(&sid("login.user")));
        assert!(s.holds(&sid("login.password")));

        // a target the state does not hold is secret, and so is every
        // node when there is no state.
        let mut gone = user.clone();
        gone.target = sid("login.token");
        assert!(Secrets::of(&gone, Some(&snap)).holds(&sid("login.token")));
        let s = Secrets::of(&submit, None);
        assert!(s.holds(&sid("login.submit")) && s.holds(&sid("login.user")));
        assert!(!s.holds(&sid("nowhere")));
    }

    #[test]
    fn a_request_keeps_only_lengths_on_secret_nodes() {
        let snap = login("hunter2");
        let mut submit =
            Request::observe(&snap, Actor::Human, "login.submit", "submit", None).unwrap();
        // an observation built by hand, with the raw secret in it.
        submit.observed = Observation {
            revision: snap.revision(),
            values: vec![
                (at("login.user:text"), Some("ana".into())),
                (at("login.password:text"), Some("hunter2".into())),
                (at("login.password:text"), None),
            ],
        };
        let s = Secrets::of(&submit, Some(&snap));
        let out = s.request(&submit);
        assert_eq!(out.param, None);
        assert_eq!(
            out.observed.values,
            [
                (at("login.user:text"), Some("ana".into())),
                (at("login.password:text"), Some(Value::Int(7))),
                (at("login.password:text"), None),
            ]
        );
        assert_eq!(out.observed.revision, submit.observed.revision);

        let typed = set(&snap, "login.password", "hunter2");
        let out = Secrets::of(&typed, Some(&snap)).request(&typed);
        assert_eq!(
            out.param,
            Some(Value::Int(7)),
            "the param of a secret target"
        );
        let clear = set(&snap, "login.user", "bob");
        let out = Secrets::of(&clear, Some(&snap)).request(&clear);
        assert_eq!(out, clear, "nothing secret: the same request");
    }

    /// an agent's question to a person (task-92) is a free text: about a
    /// secret target it could quote the secret, so it goes as `REDACTED`;
    /// about a clear target it goes as it was asked.
    #[test]
    fn a_question_about_a_secret_target_is_redacted() {
        let snap = login("hunter2");
        let agent = Actor::Agent {
            name: "claude".into(),
        };
        let secret = Request::observe(
            &snap,
            agent.clone(),
            "login.password",
            "set",
            Some("hunter2".into()),
        )
        .unwrap()
        .asking("set the password to hunter2?");
        let out = Secrets::of(&secret, Some(&snap)).request(&secret);
        assert_eq!(out.ask.as_deref(), Some(REDACTED));
        assert_eq!(out.param, Some(Value::Int(7)));
        assert!(!format!("{out:?}").contains("hunter2"));

        let clear = Request::observe(&snap, agent, "login.user", "set", Some("bob".into()))
            .unwrap()
            .asking("rename ana to bob?");
        let out = Secrets::of(&clear, Some(&snap)).request(&clear);
        assert_eq!(out, clear, "a clear target keeps its question");

        // no state: every node counts as secret, the question too.
        let out = Secrets::of(&clear, None).request(&clear);
        assert_eq!(out.ask.as_deref(), Some(REDACTED));
    }

    #[test]
    fn a_stage_about_a_secret_target_loses_its_free_text_and_keeps_lengths() {
        let snap = login("hunter2");
        let typed = set(&snap, "login.password", "hunter2");
        let s = Secrets::of(&typed, Some(&snap));
        let target = sid("login.password");
        let outcome = Outcome {
            result: IndexMap::from([
                ("echo".to_owned(), Value::Text("hunter2".into())),
                ("ok".to_owned(), Value::Bool(true)),
            ]),
        };
        let completed = s.stage(&target, &Stage::Completed(outcome));
        let Stage::Completed(out) = &completed else {
            panic!("{completed:?}");
        };
        assert_eq!(out.result.get("echo"), Some(&Value::Int(7)));
        assert_eq!(out.result.get("ok"), Some(&Value::Bool(true)));
        let failed = Stage::Failed {
            error: "hunter2 is weak".into(),
        };
        assert_eq!(
            s.stage(&target, &failed),
            Stage::Failed {
                error: REDACTED.to_owned()
            }
        );
        let invalid = Stage::Rejected(Rejection::InvalidParam {
            reason: "hunter2 is not a list".into(),
        });
        assert_eq!(
            s.stage(&target, &invalid),
            Stage::Rejected(Rejection::InvalidParam {
                reason: REDACTED.to_owned()
            })
        );
        for plain in [
            Stage::Requested,
            Stage::Accepted,
            Stage::AwaitingConfirmation,
            Stage::Executing,
            Stage::Confirmed { by: Actor::Human },
            Stage::Rejected(Rejection::Inert {
                target: target.clone(),
            }),
        ] {
            assert_eq!(s.stage(&target, &plain), plain);
        }

        // the same stages about a clear target go out as they are.
        let clear = sid("login.user");
        let s = Secrets::of(&set(&snap, "login.user", "bob"), Some(&snap));
        assert_eq!(s.stage(&clear, &failed), failed);
        assert_eq!(s.stage(&clear, &invalid), invalid);
    }

    #[test]
    fn a_stale_delta_on_a_secret_node_keeps_lengths_and_loses_its_items() {
        let snap = login("hunter2");
        let submit = Request::observe(&snap, Actor::Human, "login.submit", "submit", None).unwrap();
        let s = Secrets::of(&submit, Some(&snap));
        let secret = Change::StateChanged {
            at: at("login.password:text"),
            before: Some("hunter2".into()),
            after: Some(Value::List(vec!["hunter3".into(), "x".into()])),
            added: vec!["hunter3".into()],
            removed: vec!["hunter2".into()],
        };
        let clear = Change::StateChanged {
            at: at("login.user:text"),
            before: Some("ana".into()),
            after: Some("bob".into()),
            added: Vec::new(),
            removed: Vec::new(),
        };
        let other = Change::NodeRemoved(sid("login.password"));
        // who acted is ids, an actor and an action name: nothing of a
        // value, so it goes out as it is, also about a secret node.
        let acted = vec![Acted {
            id: RequestId::parse("4").unwrap(),
            actor: Actor::Human,
            target: sid("login.password"),
            action: "set".parse().unwrap(),
        }];
        let stale = Stage::Rejected(Rejection::Stale {
            changes: vec![secret, clear.clone(), other.clone()],
            acted: acted.clone(),
        });
        let reduced = [
            Change::StateChanged {
                at: at("login.password:text"),
                before: Some(Value::Int(7)),
                after: Some(Value::Int(2)),
                added: Vec::new(),
                removed: Vec::new(),
            },
            clear,
            other,
        ];
        // the refusal of a clear target and of the secret one alike.
        for target in ["login.submit", "login.password"] {
            let out = s.stage(&sid(target), &stale);
            let Stage::Rejected(Rejection::Stale {
                changes,
                acted: kept,
            }) = out
            else {
                panic!("{out:?}");
            };
            assert_eq!(changes, reduced, "{target}");
            assert_eq!(kept, acted, "{target}");
            assert!(!format!("{changes:?}").contains("hunter"));
        }
    }
}
