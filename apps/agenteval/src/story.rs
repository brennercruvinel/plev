//! the story of the run as the report tells it: every act of an agent
//! against the observations that were served, the value each `submit`
//! carried and the one the app held when it arrived, whether the person
//! made the stale check work at all, and whether the agent recovered.

use std::collections::BTreeSet;

use sem::{Acted, Actor, Millis, Receipt, Rejection, RequestId, Revision, Stage, StateRef, Value};
use serde_json::{Value as Json, json};
use wire::Traffic;

use crate::host::{ActSeen, Intervention, Served};

/// one act of an agent, told against the observations.
#[derive(Clone, Debug)]
pub struct ActStory {
    pub request: RequestId,
    pub t: Millis,
    pub actor: Actor,
    pub target: String,
    pub action: String,
    pub stages: Vec<&'static str>,
    pub terminal: &'static str,
    /// the revision the act named (`observed.revision`). it traces the
    /// act and decides nothing: a revision is a hash of what the app
    /// projects, so a world put back as it was has the revision it had
    /// then, and a revision a terminal stage carried is served by no
    /// observation.
    pub revision: Revision,
    /// the observation that served that revision, when one did: its index
    /// among the served observations and its time.
    pub served_index: Option<usize>,
    pub served_at: Option<Millis>,
    /// observations of the same actor served after that one and before the
    /// act: 0 means the act rests on the freshest observation it had.
    pub observations_between: usize,
    /// the observation right before the act was scoped to the act's target
    /// and served the revision the act rests on: the shape of `plev act`
    /// without `--from-revision`, which observes the target first and can
    /// never be stale.
    pub fresh_before: bool,
    pub stale: bool,
    /// the `changes:staged` the request carried in `observed`: what the
    /// agent rested on (for an act on `from_revision`, what it was last
    /// shown, task-89). `None` when the act does not read it.
    pub seen_staged: Option<BTreeSet<String>>,
    /// for a `submit`, the `changes:staged` the app held when the
    /// dispatcher took it; `None` when the laboratory could not tell (the
    /// projection it looked at was not the one the dispatcher checked).
    pub live_staged: Option<BTreeSet<String>>,
}

/// the acts of agents in `receipts`, in the order they started.
pub fn acts(receipts: &[Receipt], seen: &[ActSeen], served: &[Served]) -> Vec<ActStory> {
    let staged_at = StateRef::parse("changes:staged").ok();
    let mut out = Vec::new();
    for r in receipts {
        if !matches!(r.request.actor, Actor::Agent { .. }) {
            continue;
        }
        let Some(at) = seen.iter().find(|a| a.request == r.id) else {
            continue;
        };
        let revision = r.request.observed.revision;
        let before = &served[..at.served_before.min(served.len())];
        let served_index = before.iter().position(|s| s.revision == revision);
        let observations_between = served_index.map_or(0, |i| {
            before[i + 1..]
                .iter()
                .filter(|s| s.actor == r.request.actor)
                .count()
        });
        let fresh_before = before
            .iter()
            .rev()
            .find(|s| s.actor == r.request.actor)
            .is_some_and(|last| {
                last.revision == revision
                    && last.scope.as_ref().is_some_and(|s| *s == r.request.target)
            });
        let seen_staged = r
            .request
            .observed
            .values
            .iter()
            .find(|(read, _)| Some(read) == staged_at.as_ref())
            .and_then(|(_, value)| match value {
                Some(Value::Set(staged)) => Some(staged.clone()),
                _ => None,
            });
        let live_staged = at
            .live
            .as_ref()
            .filter(|(looked, _)| r.validated_against == Some(*looked))
            .map(|(_, staged)| staged.clone());
        let stages: Vec<&'static str> = r.stages.iter().map(|(s, _)| s.name()).collect();
        out.push(ActStory {
            request: r.id,
            t: r.stages.first().map_or(0, |(_, t)| *t),
            actor: r.request.actor.clone(),
            target: r.request.target.to_string(),
            action: r.request.action.to_string(),
            terminal: r.stage().map_or("none", Stage::name),
            stale: matches!(r.stage(), Some(Stage::Rejected(Rejection::Stale { .. }))),
            stages,
            revision,
            served_index,
            served_at: served_index.map(|i| before[i].t),
            observations_between,
            fresh_before,
            seen_staged,
            live_staged,
        });
    }
    out
}

/// whether the person made the stale check work in this run, and why.
#[derive(Clone, Debug, PartialEq)]
pub struct Measured {
    pub valid: bool,
    pub text: String,
    /// the `submit` the verdict is about: the one that rested on the old
    /// world when valid, the first after the person otherwise.
    pub request: Option<RequestId>,
    pub seen: Option<BTreeSet<String>>,
    pub live: Option<BTreeSet<String>>,
}

impl Measured {
    fn invalid(text: String, about: Option<&ActStory>) -> Measured {
        Measured {
            valid: false,
            text,
            request: about.map(|a| a.request),
            seen: about.and_then(|a| a.seen_staged.clone()),
            live: about.and_then(|a| a.live_staged.clone()),
        }
    }

    pub fn json(&self) -> Json {
        json!({
            "valid": self.valid,
            "request": self.request.map(|r| r.to_string()),
            "seen": self.seen,
            "live": self.live,
        })
    }
}

/// the stale check is measured when a `submit` that reached the
/// dispatcher after the person's act carried the `changes:staged` of
/// before it (`Intervention::staged_before`) while the app held another:
/// the agent acted on the old world, whatever revision it named. the
/// order is the dispatcher's: a request after the person's has a greater
/// id. the text says the value seen and the value live, and why.
pub fn stale_measurement(acts: &[ActStory], intervention: Option<&Intervention>) -> Measured {
    let Some(person) = intervention else {
        return Measured::invalid(
            "no intervention in this run: the stale check was not exercised".to_owned(),
            None,
        );
    };
    let before = &person.staged_before;
    let submits = || acts.iter().filter(|a| a.action == "submit");
    let after: Vec<&ActStory> = submits().filter(|a| a.request > person.request).collect();
    let old_world = after.iter().find(|a| {
        a.seen_staged.as_ref() == Some(before)
            && a.live_staged.as_ref().is_some_and(|live| live != before)
    });
    if let Some(a) = old_world {
        return Measured {
            valid: true,
            text: format!(
                "valid: submit {} reached the dispatcher after the person's request {} carrying \
                 changes:staged {}, the value before it, while the app held {}; it ended {}",
                a.request,
                person.request,
                set(a.seen_staged.as_ref()),
                set(a.live_staged.as_ref()),
                a.terminal
            ),
            request: Some(a.request),
            seen: a.seen_staged.clone(),
            live: a.live_staged.clone(),
        };
    }
    let late = submits()
        .find(|a| a.request < person.request)
        .map(|a| {
            format!(
                "; the person acted after the agent's first submit ({}), too late for it",
                a.request
            )
        })
        .unwrap_or_default();
    let Some(first) = after.first().copied() else {
        return Measured::invalid(
            format!(
                "invalid: no submit reached the dispatcher after the person's request {}{late}; \
                 the stale check was not exercised",
                person.request
            ),
            None,
        );
    };
    let why = match (&first.seen_staged, &first.live_staged) {
        (_, None) => "the value the app held then is unknown".to_owned(),
        (None, _) => "it carried no changes:staged".to_owned(),
        (Some(seen), Some(_)) if seen == before => format!(
            "it carried changes:staged {}, the value before the person acted, and the app held \
             the same again: the world was put back before the submit",
            set(Some(seen))
        ),
        (Some(seen), Some(live)) => format!(
            "it carried changes:staged {}, not the value before the person acted ({}), and the \
             app held {}: the agent saw the change before it submitted",
            set(Some(seen)),
            set(Some(before)),
            set(Some(live))
        ),
    };
    Measured::invalid(
        format!(
            "invalid: the first submit after the person's request {} was {}: {why}{late}; the \
             stale check was not exercised by the person",
            person.request, first.request
        ),
        Some(first),
    )
}

/// a set as the report prints it: `{a, b}`, or `unknown`.
fn set(staged: Option<&BTreeSet<String>>) -> String {
    match staged {
        Some(s) => format!("{{{}}}", s.iter().cloned().collect::<Vec<_>>().join(", ")),
        None => "unknown".to_owned(),
    }
}

/// a `submit` completed after a `submit` was rejected stale.
pub fn recovered(acts: &[ActStory]) -> bool {
    let mut stale_at = None;
    for a in acts.iter().filter(|a| a.action == "submit") {
        if a.stale {
            stale_at.get_or_insert(a.t);
        } else if a.terminal == "completed" && stale_at.is_some_and(|s| a.t >= s) {
            return true;
        }
    }
    false
}

pub fn served_json(served: &[Served]) -> Json {
    Json::Array(
        served
            .iter()
            .enumerate()
            .map(|(i, s)| {
                json!({
                    "index": i,
                    "t": s.t,
                    "actor": s.actor.to_string(),
                    "scope": s.scope.as_ref().map(ToString::to_string),
                    "revision": s.revision.to_string(),
                })
            })
            .collect(),
    )
}

/// what every act of the run rested on, as the port counted it
/// (`Traffic`): `from_revision` or `observed` when all of them did, `None`
/// otherwise. the laboratory does not infer it from the revision an act
/// named: an act chained on a stage's revision rests on `from_revision`
/// on a revision no observation served (task-87).
pub fn form(t: &Traffic) -> Option<&'static str> {
    match (t.acts_from_revision, t.acts_observed) {
        _ if t.acts == 0 => None,
        (all, _) if all == t.acts => Some("from_revision"),
        (_, all) if all == t.acts => Some("observed"),
        _ => None,
    }
}

/// the same, said: `every act on from_revision (--from-revision)`, or
/// how many took each form.
pub fn forms_text(t: &Traffic) -> String {
    match form(t) {
        Some("from_revision") => "every act on from_revision (--from-revision)".to_owned(),
        Some(_) => "every act with its observed values".to_owned(),
        None if t.acts == 0 => "no act".to_owned(),
        None => format!(
            "{} of {} acts on from_revision, {} with observed values",
            t.acts_from_revision, t.acts, t.acts_observed
        ),
    }
}

/// the acts, each with `form`, what every act of the run rested on
/// (`null` when they did not all rest on the same).
pub fn acts_json(acts: &[ActStory], form: Option<&str>) -> Json {
    Json::Array(
        acts.iter()
            .map(|a| {
                json!({
                    "request": a.request.to_string(),
                    "t": a.t,
                    "actor": a.actor.to_string(),
                    "target": a.target,
                    "action": a.action,
                    "stages": a.stages,
                    "terminal": a.terminal,
                    "revision": a.revision.to_string(),
                    "revision_served_index": a.served_index,
                    "revision_served_at": a.served_at,
                    "observations_between": a.observations_between,
                    "fresh_before": a.fresh_before,
                    "form": form,
                    "seen_staged": a.seen_staged,
                    "live_staged": a.live_staged,
                })
            })
            .collect(),
    )
}

/// who, among the app's actors, ran something on the state a stale
/// refusal rested on (`Rejection::Stale::acted`), one entry each.
pub fn acted_json(acted: &[Acted]) -> Json {
    Json::Array(
        acted
            .iter()
            .map(|a| {
                json!({
                    "request": a.id.to_string(),
                    "actor": a.actor.to_string(),
                    "action": a.action.to_string(),
                    "target": a.target.to_string(),
                })
            })
            .collect(),
    )
}

/// the same in one line: `human unstage changes.file[src/session.rs]`.
pub fn acted_text(acted: &[Acted]) -> String {
    let each: Vec<String> = acted
        .iter()
        .map(|a| format!("{} {} {}", a.actor, a.action, a.target))
        .collect();
    each.join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host::Trigger;

    fn id(n: u64) -> RequestId {
        RequestId::parse(&n.to_string()).expect("a request id")
    }

    fn revision() -> Revision {
        serde_json::from_str("\"00000000000000aa\"").expect("a revision")
    }

    fn paths(each: &[&str]) -> BTreeSet<String> {
        each.iter().map(|p| (*p).to_owned()).collect()
    }

    const ALL: &[&str] = &["src/login.rs", "src/session.rs", "tests/login_test.rs"];
    const TWO: &[&str] = &["src/login.rs", "tests/login_test.rs"];

    fn submit(
        n: u64,
        seen: Option<&[&str]>,
        live: Option<&[&str]>,
        terminal: &'static str,
    ) -> ActStory {
        ActStory {
            request: id(n),
            t: n,
            actor: Actor::Agent {
                name: "scripted".to_owned(),
            },
            target: "commit.submit".to_owned(),
            action: "submit".to_owned(),
            stages: vec!["requested", terminal],
            terminal,
            revision: revision(),
            served_index: None,
            served_at: None,
            observations_between: 0,
            fresh_before: false,
            stale: terminal == "rejected",
            seen_staged: seen.map(paths),
            live_staged: live.map(paths),
        }
    }

    fn person(n: u64) -> Intervention {
        Intervention {
            t: n,
            trigger: Trigger::Submit,
            served_index: 0,
            path: "src/session.rs".to_owned(),
            revision_served: revision(),
            head: String::new(),
            request: id(n),
            revision_matched: true,
            staged_before: paths(ALL),
            staged_after: paths(TWO),
        }
    }

    #[test]
    fn a_submit_on_the_old_world_after_the_person_is_the_measurement() {
        let acts = [
            submit(7, Some(ALL), Some(TWO), "rejected"),
            submit(9, Some(ALL), Some(ALL), "completed"),
        ];
        let m = stale_measurement(&acts, Some(&person(6)));
        assert!(m.valid, "{}", m.text);
        assert_eq!(m.request, Some(id(7)));
        assert_eq!(m.seen, Some(paths(ALL)));
        assert_eq!(m.live, Some(paths(TWO)));
        assert!(
            m.text.contains(
                "carrying changes:staged {src/login.rs, src/session.rs, tests/login_test.rs}"
            ) && m
                .text
                .contains("while the app held {src/login.rs, tests/login_test.rs}"),
            "the text says the value seen and the value live: {}",
            m.text
        );
    }

    #[test]
    fn a_world_put_back_is_not_the_old_one_whatever_it_carries() {
        // the agent's first submit came before the person (refused by its
        // own write, task-89), and the one after carries the three files
        // again because the agent put the file back: the same value, the
        // same revision, and not the old world.
        let acts = [
            submit(6, Some(ALL), Some(ALL), "rejected"),
            submit(9, Some(ALL), Some(ALL), "completed"),
        ];
        let m = stale_measurement(&acts, Some(&person(7)));
        assert!(!m.valid);
        assert_eq!(m.request, Some(id(9)));
        assert!(m.text.contains("the world was put back"), "{}", m.text);
        assert!(m.text.contains("too late for it"), "{}", m.text);
    }

    #[test]
    fn a_submit_on_the_new_world_did_not_exercise_the_check() {
        let acts = [submit(8, Some(TWO), Some(TWO), "completed")];
        let m = stale_measurement(&acts, Some(&person(6)));
        assert!(!m.valid);
        assert!(
            m.text.contains("saw the change before it submitted"),
            "{}",
            m.text
        );
    }

    #[test]
    fn no_person_no_submit_after_or_no_live_value_say_so() {
        let acts = [submit(5, Some(ALL), Some(ALL), "completed")];
        let none = stale_measurement(&acts, None);
        assert!(!none.valid && none.text.starts_with("no intervention"));

        let early = stale_measurement(&acts, Some(&person(6)));
        assert!(!early.valid);
        assert!(
            early.text.contains("no submit reached the dispatcher"),
            "{}",
            early.text
        );

        let blind = stale_measurement(&[submit(7, Some(ALL), None, "rejected")], Some(&person(6)));
        assert!(!blind.valid);
        assert!(blind.text.contains("unknown"), "{}", blind.text);
    }

    /// the port's counts say the form, never the revision an act named.
    #[test]
    fn the_form_of_the_acts_is_the_ports_count() {
        let t = |acts, from_revision, observed| Traffic {
            acts,
            acts_from_revision: from_revision,
            acts_observed: observed,
            ..Traffic::default()
        };
        assert_eq!(form(&t(9, 9, 0)), Some("from_revision"));
        assert_eq!(
            forms_text(&t(9, 9, 0)),
            "every act on from_revision (--from-revision)"
        );
        assert_eq!(form(&t(2, 0, 2)), Some("observed"));
        assert_eq!(form(&t(3, 2, 0)), None, "one with both or neither");
        assert_eq!(
            forms_text(&t(4, 2, 1)),
            "2 of 4 acts on from_revision, 1 with observed values"
        );
        assert_eq!(
            (form(&t(0, 0, 0)), forms_text(&t(0, 0, 0)).as_str()),
            (None, "no act")
        );
    }
}
