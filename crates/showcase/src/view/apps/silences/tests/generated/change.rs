//! A change of the review asked through the row (task-70 phase 2): with
//! the review open, `propose` reads the intent as a change of it first.
//! By the rules, a person's "too much information" hides the context
//! (the video and the counts; the rules' row is grouped by sense
//! already) and tightens the spacing, at once, as the person's change; "desfazer" takes it back whole, byte for byte; an
//! agent asks the same way, and the change is the agent's; "I can't
//! compare these" says why nothing fits and changes nothing; an intent
//! for a surface still proposes one. By the Claude (the fake claude), the
//! request starts, the status waits, and the operations arrive, applied
//! with the attempts, the seconds and the cost; a refused answer is asked
//! again with why; three refusals change nothing; "undo" asks no model.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use sem::{Actor, Request, RequestId, SemId, Stage, Value};

use super::super::super::SilencesPage;
use super::super::{TURN, setup, sid, stage};
use crate::view::apps::app::TodoPage;

const REVIEW: &str = "the list with the video and the counts";

/// The rules' review open, by a person through the row.
fn review() -> (TodoPage, SilencesPage) {
    let (mut todo, mut page, _) = setup(false);
    page.generated.proposing.intent.set_text(REVIEW);
    let asked = page.propose(&mut todo).expect("dispatched");
    assert!(matches!(stage(&todo, asked), Stage::Completed(_)));
    (todo, page)
}

/// A person proposes `intent` through the row.
fn person(page: &mut SilencesPage, todo: &mut TodoPage, intent: &str) -> (RequestId, Stage) {
    page.generated.proposing.intent.set_text(intent);
    let asked = page.propose(todo).expect("dispatched");
    (asked, stage(todo, asked))
}

/// The surface the page shows, written.
fn written(page: &SilencesPage) -> String {
    let id = page.generated.id.clone().expect("open");
    let surface = page.generated.host.get(&id).expect("open");
    serde_json::to_string(surface.verified().ir()).expect("an IR writes")
}

fn status(page: &SilencesPage) -> String {
    page.generated.proposing.message()
}

fn shown(page: &SilencesPage, id: &str) -> bool {
    page.projection()
        .expect("the page projects")
        .get(&sid(id))
        .is_some()
}

fn by(page: &SilencesPage) -> Vec<Actor> {
    let id = page.generated.id.clone().expect("open");
    page.generated
        .host
        .changes(&id)
        .iter()
        .map(|c| c.by.clone())
        .collect()
}

#[test]
fn a_person_asks_for_less_through_the_row_and_undoes_it() {
    let (mut todo, mut page) = review();
    let before = written(&page);
    assert_eq!(
        page.generated.host.policy(&sid("proposed")),
        Some(&uir::GenerationPolicy::by(Actor::Human)),
        "the host keeps the policy the review was verified under"
    );
    assert!(shown(&page, "proposed.counts"), "the counts, asked for");
    assert!(shown(&page, "proposed.silences.video.original"));

    let (_, done) = person(&mut page, &mut todo, "too much information");
    let Stage::Completed(outcome) = done else {
        panic!("{done:?}");
    };
    assert_eq!(outcome.result.get("steps"), Some(&Value::Int(3)));
    let Some(Value::List(ops)) = outcome.result.get("ops") else {
        panic!("the ops in lines");
    };
    assert_eq!(
        ops,
        &[
            "hide proposed.silences.video.original",
            "hide proposed.counts",
            "density denser",
        ]
    );
    assert_eq!(
        status(&page),
        "changed by the rules for \"too much information\", 3 steps"
    );
    assert!(!shown(&page, "proposed.counts"), "hidden from the surface");
    assert!(shown(&page, "silences.counts"), "the fixed screen's own");
    assert!(shown(&page, "proposed.silences.cut[1].actions"));
    assert_eq!(by(&page), vec![Actor::Human; 3]);
    assert_eq!(
        page.generated.proposing.intent.text(),
        "too much information"
    );

    let (_, undone) = person(&mut page, &mut todo, "desfazer");
    let Stage::Completed(outcome) = undone else {
        panic!("{undone:?}");
    };
    assert_eq!(outcome.result.get("undone"), Some(&Value::Int(3)));
    assert_eq!(written(&page), before, "byte for byte");
    assert_eq!(status(&page), "undone: the last change, 3 steps");
    assert!(shown(&page, "proposed.counts"));

    let (_, nothing) = person(&mut page, &mut todo, "undo");
    assert!(
        matches!(&nothing, Stage::Failed { error } if error.contains("no transformation to undo")),
        "{nothing:?}"
    );
}

/// An agent asks as it proposes: the intent written, `propose`, both
/// requests of the page's own capabilities. The change is the agent's.
#[test]
fn an_agent_asks_for_a_change_as_it_proposes() {
    let (mut todo, mut page) = review();
    let agent = Actor::Agent {
        name: "claude".to_owned(),
    };
    let mut act = |page: &mut SilencesPage, target: &str, action: &str, param| {
        let seen = page.projection().expect("the page projects");
        let req = Request::observe(&seen, agent.clone(), target, action, param).unwrap();
        page.act(req, &mut todo, false).expect("a receipt")
    };
    let wrote = act(
        &mut page,
        "silences_surface.intent",
        "set_value",
        Some(Value::Text("tem informação demais".to_owned())),
    );
    assert!(matches!(wrote.stage(), Some(Stage::Completed(_))));
    let asked = act(&mut page, "silences_surface.propose", "propose", None);
    assert!(
        matches!(asked.stage(), Some(Stage::Completed(_))),
        "{:?}",
        asked.stage()
    );
    assert_eq!(by(&page).len(), 3);
    assert!(by(&page).iter().all(|a| *a == agent));
}

/// The Silences cannot compare its cuts as a table or a chart: the page
/// says why, and the review stays as it was.
#[test]
fn cant_compare_says_why_and_changes_nothing() {
    let (mut todo, mut page) = review();
    let before = written(&page);
    let (_, refused) = person(&mut page, &mut todo, "I can't compare these");
    let Stage::Failed { error: why } = refused else {
        panic!("{refused:?}");
    };
    assert!(
        why.starts_with("nothing here can be swapped for a table or a chart to compare"),
        "{why}"
    );
    assert!(
        why.contains("repeats its items over `silences:cuts`"),
        "{why}"
    );
    assert_eq!(status(&page), why);
    assert_eq!(written(&page), before);
}

/// An intent for a surface, with the review open, proposes a surface in
/// place, as in task-69.
#[test]
fn an_intent_for_a_surface_still_proposes_one() {
    let (mut todo, mut page) = review();
    let (_, done) = person(&mut page, &mut todo, "compare the original with the cuts");
    assert!(matches!(done, Stage::Completed(_)), "{done:?}");
    assert!(shown(&page, "proposed.silences.video_views"));
    assert!(page.generated.host.changes(&sid("proposed")).is_empty());
}

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
mod claude {
    use super::*;
    use crate::view::apps::surface_page::asking::Claude;
    use claude_model::ClaudeCli;

    /// The fake claude in a directory of its own, in `mode`, answering
    /// `good` (and `bad` first in `repair`) as the claude's json result.
    fn fake(page: &mut SilencesPage, mode: &str, good: &str, bad: &str) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let result = |text: &str| {
            serde_json::json!({
                "type": "result", "subtype": "success", "is_error": false,
                "num_turns": 1, "total_cost_usd": 0.05, "result": text
            })
            .to_string()
        };
        std::fs::write(dir.path().join("mode"), mode).unwrap();
        std::fs::write(dir.path().join("good"), result(good)).unwrap();
        std::fs::write(dir.path().join("bad"), result(bad)).unwrap();
        let mut cli = ClaudeCli::new();
        cli.program =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../claude-model/tests/fake-claude");
        cli.dir = dir.path().to_owned();
        cli.timeout = Duration::from_secs(20);
        page.generated.proposing.set_claude(Claude::with(cli));
        dir
    }

    /// Wakes the page until the request `asked` ends.
    fn pump(page: &mut SilencesPage, todo: &mut TodoPage, asked: RequestId) -> Stage {
        let started = Instant::now();
        while !todo.receipt(asked).unwrap().is_finished() {
            assert!(started.elapsed() < Duration::from_secs(20), "it answered");
            page.wake(TURN, todo);
            std::thread::sleep(Duration::from_millis(25));
        }
        stage(todo, asked)
    }

    fn prompts(dir: &Path) -> usize {
        (1..10)
            .take_while(|n| dir.join(format!("prompt.{n}")).exists())
            .count()
    }

    const DENSER: &str = r#"{"ops": [{"op": "density", "step": "denser"}]}"#;
    const HIDE_ROOT: &str = r#"{"ops": [{"op": "hide", "id": "proposed.card"}]}"#;

    /// The Claude answers the change off the thread: the request starts,
    /// the status says it asks, and the operations arrive, applied as the
    /// person's, with the attempts, the seconds and the cost. The prompt
    /// was the change's. "undo" then asks no model.
    #[test]
    fn the_claude_changes_the_review_off_the_thread() {
        let (mut todo, mut page) = review();
        let before = written(&page);
        let dir = fake(&mut page, "ok", DENSER, "");
        let (asked, started) = person(&mut page, &mut todo, "too much information");
        assert_eq!(started, Stage::Executing);
        assert!(page.generated.proposing.waiting());
        assert!(status(&page).starts_with("asking the Claude for \"too much information\""));
        let Stage::Completed(outcome) = pump(&mut page, &mut todo, asked) else {
            panic!("{:?}", stage(&todo, asked));
        };
        assert_eq!(outcome.result.get("steps"), Some(&Value::Int(1)));
        assert_eq!(outcome.result.get("attempts"), Some(&Value::Int(1)));
        assert!(matches!(
            outcome.result.get("cost_usd"),
            Some(Value::Num(_))
        ));
        assert!(
            status(&page)
                .starts_with("changed by the Claude for \"too much information\", 1 step, in ")
        );
        assert!(
            status(&page).ends_with(", 1 attempt, 0.050 USD"),
            "{}",
            status(&page)
        );
        assert_eq!(by(&page), [Actor::Human]);
        let prompt = std::fs::read_to_string(dir.path().join("prompt.1")).unwrap();
        assert!(prompt.contains("## the operations"));
        assert!(prompt.contains("its id is `proposed`. its IR:"));

        let (_, undone) = person(&mut page, &mut todo, "undo");
        assert!(matches!(undone, Stage::Completed(_)), "{undone:?}");
        assert_eq!(written(&page), before);
        assert_eq!(prompts(dir.path()), 1, "no model for an undo");
    }

    /// The first answer hides the root: refused, asked again with why;
    /// the second applies. Three refusals end the request with why, and
    /// the review stays as it was.
    #[test]
    fn a_refused_answer_is_asked_again_and_three_change_nothing() {
        let (mut todo, mut page) = review();
        let dir = fake(&mut page, "repair", DENSER, HIDE_ROOT);
        let (asked, _) = person(&mut page, &mut todo, "too much information");
        let Stage::Completed(outcome) = pump(&mut page, &mut todo, asked) else {
            panic!("{:?}", stage(&todo, asked));
        };
        assert_eq!(outcome.result.get("attempts"), Some(&Value::Int(2)));
        let second = std::fs::read_to_string(dir.path().join("prompt.2")).unwrap();
        assert!(second.contains("operation 1, `hide proposed.card`, was refused"));

        let (mut todo, mut page) = review();
        let before = written(&page);
        let dir = fake(&mut page, "ok", HIDE_ROOT, "");
        let (asked, _) = person(&mut page, &mut todo, "too much information");
        let Stage::Failed { error: why } = pump(&mut page, &mut todo, asked) else {
            panic!("{:?}", stage(&todo, asked));
        };
        assert!(
            why.starts_with("3 answers and none changed the surface"),
            "{why}"
        );
        assert_eq!(prompts(dir.path()), 3);
        assert_eq!(written(&page), before);
        assert!(
            page.generated
                .host
                .changes(&SemId::parse("proposed").unwrap())
                .is_empty()
        );
    }
}
