//! The language-model proposer with models that answer from a script, no
//! provider: a model that errs first is repaired by the verifier's errors
//! in the second prompt; a model that always errs exhausts the attempts
//! and its last errors come back, never its IR; a model that fails stops
//! at once; and the prompt carries the schema, the components, the rules,
//! the registry and the ids that must stay.

// shared with the other test binaries: this one reads less of it.
#[allow(dead_code)]
mod support;

use std::collections::VecDeque;

use propose::{Llm, Model, ModelError, ProposeError, prompt, propose_verified};
use support::{Asking, compact, todo, verified};
use uir::{Capabilities, Component, IrError, verify_json};

/// A model that answers from a script and keeps every prompt it read.
#[derive(Default)]
struct Scripted {
    replies: VecDeque<Result<String, ModelError>>,
    prompts: Vec<String>,
}

impl Scripted {
    fn new(replies: impl IntoIterator<Item = Result<String, ModelError>>) -> Scripted {
        Scripted {
            replies: replies.into_iter().collect(),
            prompts: Vec::new(),
        }
    }
}

impl Model for Scripted {
    fn complete(&mut self, prompt: &str) -> Result<String, ModelError> {
        self.prompts.push(prompt.to_owned());
        self.replies
            .pop_front()
            .unwrap_or_else(|| Err(ModelError("the script ran out".to_owned())))
    }
}

fn ok(text: String) -> Result<String, ModelError> {
    Ok(text)
}

/// The compact fixture with one binding to a state the app does not
/// declare, and the other counter bound to a kind a stat does not show.
fn wrong() -> String {
    compact()
        .replace("todo.counter:active", "todo.counter:gone")
        .replace("todo.counter:done", "todo.filter:selected")
        .replace(r#""component": "stat""#, r#""component": "progress""#)
}

fn errors_of(text: &str, asking: &Asking) -> Vec<IrError> {
    let caps = Capabilities::from_snapshot(&asking.app);
    match verify_json(text, &caps, &asking.policy) {
        Err(errors) => errors,
        Ok(_) => panic!("the text verifies"),
    }
}

/// The acceptance: the first answer is refused, the second prompt carries
/// that answer and every error of the verifier (pointer and reason), and
/// the second answer, wrapped in prose and a fence as models write it,
/// verifies.
#[test]
fn a_model_that_errs_first_is_repaired_by_the_verifiers_errors() {
    let asking = Asking::new(todo()).over(verified(&compact(), &todo()));
    let expected = errors_of(&wrong(), &asking);
    assert!(expected.len() >= 2, "{expected:#?}");
    let fenced = format!("here is the surface:\n```json\n{}\n```\n", compact());
    let mut model = Scripted::new([ok(wrong()), ok(fenced)]);
    let mut llm = Llm::new(&mut model);
    let ir = propose_verified(&mut llm, &asking.ask("the compact todo")).expect("verifies");
    assert_eq!(ir.ir().surface.id.as_str(), "todo_panel");

    assert_eq!(model.prompts.len(), 2);
    let (first, second) = (&model.prompts[0], &model.prompts[1]);
    assert!(
        second.starts_with(first.as_str()),
        "the repair keeps the prompt"
    );
    assert!(!first.contains("was refused"));
    assert!(second.contains("## attempt 1 was refused"));
    assert!(second.contains("todo.counter:gone"), "the answer is quoted");
    for e in &expected {
        assert!(second.contains(&e.pointer), "{} in the prompt", e.pointer);
        assert!(second.contains(&e.reason), "{} in the prompt", e.reason);
        assert!(second.contains(&e.to_string()));
    }
}

/// A model that never answers an IR the verifier accepts: three attempts
/// (`Llm::ATTEMPTS`), then `Exhausted` with the errors of the last
/// answer, and no IR. The answers that are not JSON are refused like the
/// others, at the document.
#[test]
fn a_model_that_always_errs_exhausts_the_attempts() {
    let asking = Asking::new(todo()).over(verified(&compact(), &todo()));
    let last = compact().replace(r#""plev_ir": 1"#, r#""plev_ir": 2"#);
    let expected = errors_of(&last, &asking);
    let mut model = Scripted::new([
        ok("I would rather describe it in words.".to_owned()),
        ok(wrong()),
        ok(last),
        ok(compact()),
    ]);
    let mut llm = Llm::new(&mut model);
    assert_eq!(llm.attempts(), 3);
    let e = propose_verified(&mut llm, &asking.ask("the compact todo")).unwrap_err();
    match &e {
        ProposeError::Exhausted { attempts, errors } => {
            assert_eq!(*attempts, Llm::<Scripted>::ATTEMPTS);
            assert_eq!(errors, &expected);
        }
        other => panic!("exhausted: {other:?}"),
    }
    assert!(e.to_string().starts_with("3 proposals and none verified"));
    assert_eq!(model.prompts.len(), 3, "the fourth answer is never asked");
    assert!(model.prompts[1].contains("not JSON"));
    assert!(model.prompts[2].contains("## attempt 2 was refused"));

    let mut once = Scripted::new([ok(wrong()), ok(compact())]);
    let mut llm = Llm::with_attempts(&mut once, 0);
    assert_eq!(llm.attempts(), 1, "a model is asked at least once");
    let e = propose_verified(&mut llm, &asking.ask("the compact todo")).unwrap_err();
    assert!(matches!(e, ProposeError::Exhausted { attempts: 1, .. }));
}

/// A model that does not answer stops the proposal at once: its failure
/// is the error, and it is not asked again.
#[test]
fn a_model_that_fails_is_the_error_at_once() {
    let asking = Asking::new(todo());
    let mut model = Scripted::new([Err(ModelError("timed out".to_owned())), ok(compact())]);
    let e = propose_verified(&mut Llm::new(&mut model), &asking.ask("anything")).unwrap_err();
    assert_eq!(e, ProposeError::Model("timed out".to_owned()));
    assert_eq!(e.to_string(), "the model failed: timed out");
    assert_eq!(model.prompts.len(), 1);
}

/// What the prompt carries: the intent; the verifier's eight rules and
/// the policy; every component; the registry, every state and every
/// action of the app with its kind; the surface shown now, its id and
/// every id to keep with what it binds; and the JSON schema of the IR.
#[test]
fn the_prompt_carries_the_schema_the_registry_and_the_ids_to_keep() {
    let asking = Asking::new(todo()).over(verified(&compact(), &todo()));
    let intent = "only what is active and the add field";
    let text = prompt(&asking.ask(intent));
    assert!(text.contains(&format!("## the intent\n\n{intent}\n")));
    for rule in [
        "schema",
        "identity",
        "vocabulary",
        "state",
        "action",
        "policy",
        "access",
        "limits",
    ] {
        assert!(text.contains(&format!(". {rule}: ")), "rule {rule}");
    }
    assert!(text.contains("author agent:unknown, at most 256 nodes, depth 16"));
    for c in Component::ALL {
        assert!(text.contains(&format!("- {}", c.name())) || c.name() == "chart");
    }
    assert!(text.contains(r#"- {"chart": "line" | "bar" | "area" | "donut"}"#));
    // the registry: every state and every action of the app
    let caps = Capabilities::from_snapshot(&asking.app);
    for d in caps.data() {
        assert!(
            text.contains(&format!("state {}: {}", d.at, d.kind.as_str())),
            "{}",
            d.at
        );
    }
    for a in caps.actions() {
        assert!(
            text.contains(&format!("action {} {}: takes", a.target, a.action)),
            "{} {}",
            a.target,
            a.action
        );
    }
    assert!(text.contains("todo.input (text_input) \"new todo\", in todo"));
    assert!(text.contains("action todo.input set_value: takes a text"));
    assert!(text.contains("action todo.item[1] remove: takes nothing, destructive"));
    assert!(text.contains("action todo.input submit: takes nothing, disabled now"));
    // the surface shown now and the ids that must stay
    assert!(text.contains("its id is `todo_panel`: keep it"));
    for id in [
        "todo_panel.input",
        "todo_panel.add",
        "todo_panel.filter.all",
        "todo_panel.filter.active",
        "todo_panel.filter.completed",
        "todo_panel.active",
        "todo_panel.done",
    ] {
        assert!(text.contains(&format!("- `{id}`: ")), "{id}");
    }
    assert!(text.contains("- `todo_panel.add`: button, activate -> todo.input submit"));
    // the schema, whole
    let schema = serde_json::to_string_pretty(&uir::schema()).unwrap();
    assert!(text.contains(&schema));

    let fresh = Asking::new(todo());
    let text = prompt(&fresh.ask(intent));
    assert!(text.contains("no surface is shown now"));
    assert!(!text.contains("keep the id of every node"));
}
