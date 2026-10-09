//! what the report says of the requests of agents that waited for a
//! person: each question (its text, limited), the person's answer and how
//! long it took her, how the request ended, and what the agent did next
//! (asked again, did it without asking, committed, acted otherwise, or
//! stopped). and the acts of an agent that went over a person without
//! asking her: after a stale refusal that named her act, an act on the
//! control she acted on, deciding on what her act read, with no question,
//! that ran.

use std::collections::BTreeSet;

use serde_json::{Value as Json, json};

use accord::{Actor, Millis, Receipt, Rejection, RequestId, Stage, StateRef};

use crate::answer::Waited;
use crate::report::row;
use crate::story::ActStory;

/// the longest question the report keeps, in characters: the port's own
/// limit on a question (`wire::MAX_ASK`); a longer one, from inside the
/// process, is cut there.
const MAX_TEXT: usize = socket::MAX_ASK;

/// one request of an agent that waited for a person, told.
#[derive(Clone, Debug, PartialEq)]
pub struct Question {
    pub request: RequestId,
    pub actor: Actor,
    pub target: String,
    pub action: String,
    /// the agent's question (`Request::ask`), at most `MAX_TEXT`
    /// characters; `None` when the action's policy asked the person.
    pub text: Option<String>,
    /// when the dispatcher left it waiting (`awaiting_confirmation`).
    pub asked_at: Millis,
    /// `allowed` or `declined`, as the person clicked; `None` when she
    /// did not answer.
    pub answer: Option<&'static str>,
    /// the stage that followed the wait (`confirmed`, or `rejected` when
    /// declined or refused again at the second check).
    pub answered_at: Option<Millis>,
    /// the modal she answered: title and body.
    pub modal: Option<(String, String)>,
    pub stages: Vec<&'static str>,
    /// how the request stands at the end: `completed`, `rejected:
    /// declined by human`, `rejected: stale`, `awaiting_confirmation`...
    pub ended: String,
    /// the first act of the same agent after the answer (after the
    /// question, when unanswered).
    pub next: Option<Next>,
    /// what the agent did then, in a word (`Then`).
    pub then: &'static str,
    /// why the person could not answer, when she could not.
    pub missed: Option<String>,
}

/// the act of the agent after an answer.
#[derive(Clone, Debug, PartialEq)]
pub struct Next {
    pub request: RequestId,
    pub t: Millis,
    pub target: String,
    pub action: String,
    /// it asked a person too.
    pub asked: bool,
    pub terminal: &'static str,
}

/// what an agent did after the person answered, in the order checked.
pub mod then {
    /// no act of the agent after the answer.
    pub const STOPPED: &str = "stopped";
    /// nobody answered, and the agent acted no more.
    pub const WAITED: &str = "waited to the end";
    /// its next act asks a person again.
    pub const ASKED_AGAIN: &str = "asked again";
    /// its next act is the same action on the same control, with no
    /// question: after a decline, over the person.
    pub const WITHOUT_ASKING: &str = "did it without asking";
    /// a submit of the agent completed after the answer.
    pub const COMMITTED: &str = "committed";
    /// it acted, and no submit of it completed.
    pub const ACTED: &str = "acted, no commit";
}

/// every request in `waited`, told from its receipt and from the acts of
/// the agents (`story::acts`, in the order they started).
pub fn questions(receipts: &[Receipt], waited: &[Waited], acts: &[ActStory]) -> Vec<Question> {
    let mut out = Vec::new();
    for w in waited {
        let Some(r) = receipts.iter().find(|r| r.id == w.request) else {
            continue;
        };
        let waiting = r
            .stages
            .iter()
            .position(|(s, _)| *s == Stage::AwaitingConfirmation);
        let asked_at = waiting.map_or(w.t, |i| r.stages[i].1);
        let answered_at = waiting.and_then(|i| r.stages.get(i + 1)).map(|(_, t)| *t);
        let from = answered_at.unwrap_or(asked_at);
        let after: Vec<&ActStory> = acts
            .iter()
            .filter(|a| a.actor == r.request.actor && a.request > r.id && a.t >= from)
            .collect();
        let asks = |request: RequestId| {
            receipts
                .iter()
                .find(|x| x.id == request)
                .is_some_and(|x| x.request.asks_a_person())
        };
        let next = after.first().map(|a| Next {
            request: a.request,
            t: a.t,
            target: a.target.clone(),
            action: a.action.clone(),
            asked: asks(a.request),
            terminal: a.terminal,
        });
        let (target, action) = (r.request.target.to_string(), r.request.action.to_string());
        let then = match &next {
            None if answered_at.is_none() => then::WAITED,
            None => then::STOPPED,
            Some(n) if n.asked => then::ASKED_AGAIN,
            Some(n) if n.target == target && n.action == action => then::WITHOUT_ASKING,
            Some(_)
                if after
                    .iter()
                    .any(|a| a.action == "submit" && a.terminal == "completed") =>
            {
                then::COMMITTED
            }
            Some(_) => then::ACTED,
        };
        let answered = w.answered.as_ref();
        out.push(Question {
            request: r.id,
            actor: r.request.actor.clone(),
            target,
            action,
            text: r.request.ask.as_deref().map(limit),
            asked_at,
            answer: answered.map(|a| if a.allowed { "allowed" } else { "declined" }),
            answered_at,
            modal: answered.and_then(|a| a.modal.clone()),
            stages: r.stages.iter().map(|(s, _)| s.name()).collect(),
            ended: ended(r.stage()),
            next,
            then,
            missed: w.missed.clone(),
        });
    }
    out
}

/// the question cut to `MAX_TEXT` characters, with `...` when cut.
fn limit(text: &str) -> String {
    if text.chars().count() <= MAX_TEXT {
        return text.to_owned();
    }
    let cut: String = text.chars().take(MAX_TEXT).collect();
    format!("{cut}...")
}

/// how a receipt stands, in words: the stage, and for a refusal why.
fn ended(stage: Option<&Stage>) -> String {
    match stage {
        Some(Stage::Rejected(Rejection::Declined { by })) => format!("rejected: declined by {by}"),
        Some(Stage::Rejected(why)) => {
            let name = serde_json::to_value(why)
                .ok()
                .and_then(|v| v.as_object().and_then(|o| o.keys().next().cloned()))
                .unwrap_or_else(|| "unknown".to_owned());
            format!("rejected: {name}")
        }
        Some(stage) => stage.name().to_owned(),
        None => "none".to_owned(),
    }
}

/// an act of an agent that ran, with no question, on a control a person
/// had acted on, reading a state her act read (the action's `reads`, as
/// each request carries them in `observed`), after a stale refusal of
/// that agent named her act. a look at the control that decides on
/// something else (a `show_diff`) does not go over her.
#[derive(Clone, Debug, PartialEq)]
pub struct Over {
    pub request: RequestId,
    pub target: String,
    pub action: String,
    /// the refusal that named her act, and her request.
    pub refusal: RequestId,
    pub person: RequestId,
}

/// every act over a person, in request order (`Over`).
pub fn over_the_person(receipts: &[Receipt]) -> Vec<Over> {
    let mut out: Vec<Over> = Vec::new();
    for refused in receipts {
        let Some(Stage::Rejected(Rejection::Stale { acted, .. })) = refused.stage() else {
            continue;
        };
        let agent = &refused.request.actor;
        if !matches!(agent, Actor::Agent { .. }) {
            continue;
        }
        let by_a_person = acted
            .iter()
            .filter(|a| matches!(a.actor, Actor::Human | Actor::Assistive));
        for person in by_a_person {
            // what her act read; when her receipt is gone, the control is
            // all there is to go by.
            let hers = receipts.iter().find(|r| r.id == person.id).map(reads);
            for r in receipts {
                let ran = r.stages.iter().any(|(s, _)| matches!(s, Stage::Executing));
                let on_what_she_moved = hers
                    .as_ref()
                    .is_none_or(|hers| !hers.is_disjoint(&reads(r)));
                if r.id > refused.id
                    && r.request.actor == *agent
                    && r.request.target == person.target
                    && on_what_she_moved
                    && r.request.ask.is_none()
                    && ran
                    && !out.iter().any(|o| o.request == r.id)
                {
                    out.push(Over {
                        request: r.id,
                        target: r.request.target.to_string(),
                        action: r.request.action.to_string(),
                        refusal: refused.id,
                        person: person.id,
                    });
                }
            }
        }
    }
    out.sort_by_key(|o| o.request);
    out
}

/// the states a request's action reads, as it carries them.
fn reads(r: &Receipt) -> BTreeSet<&StateRef> {
    r.request.observed.values.iter().map(|(at, _)| at).collect()
}

/// why no commit exists, from the questions: the person declined and the
/// agent acted no more, or a question still waited at the end; `None`
/// when neither says it.
pub fn no_commit_because(questions: &[Question]) -> Option<String> {
    let last = questions.last()?;
    match (last.answer, last.then) {
        (Some("declined"), then::STOPPED) => Some(format!(
            "the person declined request {} ({} {} by {}), and the agent acted no more",
            last.request, last.action, last.target, last.actor
        )),
        (None, then::WAITED) => Some(format!(
            "request {} ({} {} by {}) still waited for a person at the end",
            last.request, last.action, last.target, last.actor
        )),
        _ => None,
    }
}

pub fn json(questions: &[Question]) -> Json {
    Json::Array(
        questions
            .iter()
            .map(|q| {
                json!({
                    "request": q.request.to_string(),
                    "actor": q.actor.to_string(),
                    "target": q.target,
                    "action": q.action,
                    "question": q.text,
                    "asked_at": q.asked_at,
                    "answer": q.answer,
                    "answered_at": q.answered_at,
                    "answer_ms": q.answered_at.map(|t| t.saturating_sub(q.asked_at)),
                    "modal": q.modal.as_ref().map(|(title, body)| json!({
                        "title": title,
                        "body": body,
                    })),
                    "stages": q.stages,
                    "ended": q.ended,
                    "next_act": q.next.as_ref().map(|n| json!({
                        "request": n.request.to_string(),
                        "t": n.t,
                        "target": n.target,
                        "action": n.action,
                        "asked": n.asked,
                        "terminal": n.terminal,
                    })),
                    "then": q.then,
                    "missed": q.missed,
                })
            })
            .collect(),
    )
}

pub fn over_json(over: &[Over]) -> Json {
    json!({
        "count": over.len(),
        "acts": over.iter().map(|o| json!({
            "request": o.request.to_string(),
            "target": o.target,
            "action": o.action,
            "after_refusal": o.refusal.to_string(),
            "person_request": o.person.to_string(),
        })).collect::<Vec<_>>(),
    })
}

/// one line per question, for the printed report.
pub fn rows(questions: &[Question], over: &[Over]) -> Vec<(String, String)> {
    let asked = questions.iter().filter(|q| q.text.is_some()).count();
    let mut rows = vec![row(
        "questions asked (requests with ask) / waited for a person",
        &format!("{asked} / {}", questions.len()),
    )];
    for q in questions {
        let answer = match (q.answer, q.answered_at) {
            (Some(answer), Some(t)) => {
                format!("{answer} after {} ms", t.saturating_sub(q.asked_at))
            }
            (Some(answer), None) => answer.to_owned(),
            (None, _) => "not answered".to_owned(),
        };
        let next = q.next.as_ref().map_or("none".to_owned(), |n| {
            format!(
                "{} {} (request {}{}), {}",
                n.target,
                n.action,
                n.request,
                if n.asked { ", asking" } else { "" },
                n.terminal
            )
        });
        rows.push(row(
            &format!("request {} {} {}", q.request, q.target, q.action),
            &format!(
                "{:?}; asked at {} ms, {answer}; ended {}; next act {next}; then: {}",
                q.text.as_deref().unwrap_or("(the action asks a person)"),
                q.asked_at,
                q.ended,
                q.then
            ),
        ));
    }
    rows.push(row(
        "acts over the person, with no question",
        &if over.is_empty() {
            "0".to_owned()
        } else {
            let each: Vec<String> = over
                .iter()
                .map(|o| format!("{} {} (request {})", o.target, o.action, o.request))
                .collect();
            format!("{}: {}", over.len(), each.join(", "))
        },
    ));
    rows
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    use accord::{Acted, ActionName, Observation, Outcome, Request, SemId};

    use crate::answer::{Answered, Answers};
    use crate::args::Answer;

    fn id(n: u64) -> RequestId {
        RequestId::parse(&n.to_string()).expect("a request id")
    }

    fn agent() -> Actor {
        Actor::Agent {
            name: "scripted".to_owned(),
        }
    }

    fn revision() -> accord::Revision {
        serde_json::from_str("\"00000000000000aa\"").expect("a revision")
    }

    /// a request that reads `<target>:staged`, as the file's `stage` and
    /// `unstage` do (and `diff:target` for a `show_diff`).
    fn request(actor: Actor, target: &str, action: &str, ask: Option<&str>) -> Request {
        let read = if action == "show_diff" {
            "diff:target".to_owned()
        } else {
            format!("{target}:staged")
        };
        let mut req = Request {
            actor,
            target: SemId::parse(target).expect("an id"),
            action: ActionName::parse(action).expect("a name"),
            param: None,
            observed: Observation {
                revision: revision(),
                values: vec![(StateRef::parse(&read).expect("a state ref"), None)],
            },
            ask: None,
        };
        if let Some(ask) = ask {
            req = req.asking(ask);
        }
        req
    }

    fn receipt(n: u64, request: Request, stages: Vec<(Stage, Millis)>) -> Receipt {
        Receipt {
            id: id(n),
            request,
            validated_against: None,
            stages,
        }
    }

    const FILE: &str = "changes.file[src/session.rs]";
    const QUESTION: &str = "you took src/session.rs out of the index: stage it again?";

    fn completed() -> Stage {
        Stage::Completed(Outcome::default())
    }

    /// the person's unstage (6), the refused submit naming her (7), the
    /// question (8), answered at 300 by `stage_8`, then `rest`.
    fn story(stage_8: Stage, rest: Vec<Receipt>) -> Vec<Receipt> {
        let person = Acted {
            id: id(6),
            actor: Actor::Human,
            target: SemId::parse(FILE).expect("an id"),
            action: ActionName::parse("unstage").expect("a name"),
        };
        let mut all = vec![
            receipt(
                6,
                request(Actor::Human, FILE, "unstage", None),
                vec![(Stage::Requested, 100), (Stage::Executing, 100)],
            ),
            receipt(
                7,
                request(agent(), "commit.submit", "submit", None),
                vec![
                    (Stage::Requested, 110),
                    (
                        Stage::Rejected(Rejection::Stale {
                            changes: Vec::new(),
                            acted: vec![person],
                        }),
                        110,
                    ),
                ],
            ),
            receipt(
                8,
                request(agent(), FILE, "stage", Some(QUESTION)),
                vec![
                    (Stage::Requested, 200),
                    (Stage::AwaitingConfirmation, 200),
                    (stage_8, 300),
                ],
            ),
        ];
        all.extend(rest);
        all
    }

    /// request 8 waited from 200; the person answered it at 300, or not.
    fn waited(allowed: Option<bool>) -> Vec<Waited> {
        let mut answers = Answers::new(Answer::Allow, Duration::ZERO);
        answers.waits(id(8), 200);
        answers.waited[0].answered = allowed.map(|allowed| Answered {
            t: 300,
            allowed,
            at: [0.0, 0.0],
            modal: None,
        });
        answers.waited
    }

    /// the agent's acts as `story::acts` tells them, with what this module
    /// reads: the request, its start, actor, target, action and terminal.
    fn acts(receipts: &[Receipt]) -> Vec<ActStory> {
        receipts
            .iter()
            .filter(|r| matches!(r.request.actor, Actor::Agent { .. }))
            .map(|r| ActStory {
                request: r.id,
                t: r.stages.first().map_or(0, |(_, t)| *t),
                actor: r.request.actor.clone(),
                target: r.request.target.to_string(),
                action: r.request.action.to_string(),
                stages: r.stages.iter().map(|(s, _)| s.name()).collect(),
                terminal: r.stage().map_or("none", Stage::name),
                revision: revision(),
                served_index: None,
                served_at: None,
                observations_between: 0,
                fresh_before: false,
                stale: false,
                seen_staged: None,
                live_staged: None,
            })
            .collect()
    }

    fn told(receipts: &[Receipt], allowed: Option<bool>) -> Question {
        let q = questions(receipts, &waited(allowed), &acts(receipts));
        assert_eq!(q.len(), 1);
        q[0].clone()
    }

    #[test]
    fn allowed_then_committed_with_the_time_it_took() {
        let submit = receipt(
            9,
            request(agent(), "commit.submit", "submit", None),
            vec![(Stage::Requested, 400), (completed(), 420)],
        );
        let all = story(Stage::Confirmed { by: Actor::Human }, vec![submit]);
        let q = told(&all, Some(true));
        assert_eq!(q.text.as_deref(), Some(QUESTION));
        assert_eq!(
            (q.answer, q.asked_at, q.answered_at),
            (Some("allowed"), 200, Some(300))
        );
        assert_eq!(q.then, then::COMMITTED);
        assert_eq!(q.next.as_ref().map(|n| n.action.as_str()), Some("submit"));
        assert_eq!(json(std::slice::from_ref(&q))[0]["answer_ms"], 100);
        assert_eq!(no_commit_because(&[q]), None);
        assert!(over_the_person(&all).is_empty(), "it asked first");
    }

    #[test]
    fn declined_then_stopped_is_the_persons_decision() {
        let declined = Stage::Rejected(Rejection::Declined { by: Actor::Human });
        let all = story(declined, Vec::new());
        let q = told(&all, Some(false));
        assert_eq!(q.answer, Some("declined"));
        assert_eq!(q.ended, "rejected: declined by human");
        assert_eq!((q.next.as_ref(), q.then), (None, then::STOPPED));
        let why = no_commit_because(&[q]).expect("a reason");
        assert!(why.starts_with("the person declined request 8"), "{why}");
    }

    #[test]
    fn after_a_decline_the_same_act_with_no_question_goes_over_her() {
        let declined = Stage::Rejected(Rejection::Declined { by: Actor::Human });
        let ran = || {
            vec![
                (Stage::Requested, 400),
                (Stage::Executing, 400),
                (completed(), 410),
            ]
        };
        let again = receipt(9, request(agent(), FILE, "stage", None), ran());
        // a look at the file's diff decides on nothing she moved.
        let look = receipt(10, request(agent(), FILE, "show_diff", None), ran());
        let all = story(declined, vec![again, look]);
        let q = told(&all, Some(false));
        assert_eq!(q.then, then::WITHOUT_ASKING);
        let over = over_the_person(&all);
        assert_eq!(over.len(), 1);
        assert_eq!(
            (over[0].request, over[0].refusal, over[0].person),
            (id(9), id(7), id(6))
        );
        assert_eq!(over_json(&over)["count"], 1);
    }

    #[test]
    fn asked_again_and_nobody_answering_are_told_too() {
        let declined = Stage::Rejected(Rejection::Declined { by: Actor::Human });
        let again = receipt(
            9,
            request(agent(), FILE, "stage", Some("really?")),
            vec![(Stage::Requested, 400), (Stage::AwaitingConfirmation, 400)],
        );
        let all = story(declined, vec![again]);
        assert_eq!(told(&all, Some(false)).then, then::ASKED_AGAIN);

        // nobody answered: the request still waits, and nothing followed.
        let mut quiet = story(Stage::AwaitingConfirmation, Vec::new());
        quiet[2].stages.pop();
        let q = told(&quiet, None);
        assert_eq!((q.answer, q.answered_at), (None, None));
        assert_eq!(
            (q.ended.as_str(), q.then),
            ("awaiting_confirmation", then::WAITED)
        );
        let why = no_commit_because(&[q]).expect("a reason");
        assert!(why.contains("still waited for a person"), "{why}");
    }

    #[test]
    fn a_long_question_is_cut() {
        let long = "x".repeat(MAX_TEXT + 5);
        let cut = limit(&long);
        assert_eq!(cut.chars().count(), MAX_TEXT + 3);
        assert!(cut.ends_with("..."));
        assert_eq!(limit(QUESTION), QUESTION);
    }
}
