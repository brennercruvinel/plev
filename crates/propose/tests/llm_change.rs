//! The language model as a transformer (task-70 phase 2), with models
//! that answer from a script: an answer of operations is applied; one
//! with an operation refused goes back with why and the model answers
//! again; three refusals return the reasons and change nothing; a model
//! that fails stops at once. The prompt carries the intent, the
//! operations, the rules of identity, the registry (a secret as its
//! length) and the surface shown with what is hidden; an agent's
//! promotion of an action kept from agents is refused whatever the model
//! answers.

// shared with the other test binaries: this one reads less of it.
#[allow(dead_code)]
mod support;

use std::collections::VecDeque;

use propose::{Llm, Model, ModelError, ProposeError, Transformed, transform_verified};
use sem::{Actor, SemId};
use support::{Changing, VAULT_PANEL, agent, compact, todo, vault};
use uir::{GenerationPolicy, Op};

/// A model that answers from a script and keeps every prompt it read.
#[derive(Default)]
struct Scripted {
    replies: VecDeque<Result<String, ModelError>>,
    prompts: Vec<String>,
}

impl Scripted {
    fn new<'a>(replies: impl IntoIterator<Item = &'a str>) -> Scripted {
        Scripted {
            replies: replies.into_iter().map(|r| Ok(r.to_owned())).collect(),
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

fn sid(text: &str) -> SemId {
    SemId::parse(text).unwrap()
}

const HIDE_COUNTS: &str =
    r#"{"ops": [{"op": "hide", "id": "todo_panel.counts"}, {"op": "density", "step": "denser"}]}"#;
const HIDE_ROOT: &str =
    r#"{"ops": [{"op": "density", "step": "denser"}, {"op": "hide", "id": "todo_panel.card"}]}"#;

#[test]
fn an_answer_of_operations_is_applied() {
    let changing = Changing::over(todo(), &compact());
    let mut model = Scripted::new([&*format!("here you go:\n```json\n{HIDE_COUNTS}\n```")]);
    let out = transform_verified(&mut Llm::new(&mut model), &changing.ask("too much"));
    let Ok(Transformed::Changed { ops, adapted }) = out else {
        panic!("applied: {out:?}");
    };
    assert_eq!(
        ops,
        [
            Op::Hide {
                id: sid("todo_panel.counts")
            },
            Op::Density {
                step: uir::Step::Denser
            }
        ]
    );
    assert_eq!(support::aside(&adapted), ["todo_panel.counts"]);
    assert_eq!(model.prompts.len(), 1);
}

/// The first answer hides the root: refused. The second prompt carries
/// that answer and the operation refused with why; the second answer
/// applies.
#[test]
fn an_operation_refused_goes_back_with_why_and_the_model_answers_again() {
    let changing = Changing::over(todo(), &compact());
    let mut model = Scripted::new([HIDE_ROOT, HIDE_COUNTS]);
    let out = transform_verified(&mut Llm::new(&mut model), &changing.ask("less"));
    assert!(matches!(out, Ok(Transformed::Changed { .. })), "{out:?}");
    assert_eq!(model.prompts.len(), 2);
    let second = &model.prompts[1];
    assert!(
        second.starts_with(&model.prompts[0]),
        "the first prompt, whole"
    );
    assert!(second.contains("## attempt 1 was refused"));
    assert!(second.contains(HIDE_ROOT), "the answer quoted");
    assert!(
        second.contains("- operation 2, `hide todo_panel.card`, was refused: "),
        "{second}"
    );
    // an answer that is no list of operations is refused the same way
    let mut model = Scripted::new([r#"{"plev_ir": 1}"#, HIDE_COUNTS]);
    let out = transform_verified(&mut Llm::new(&mut model), &changing.ask("less"));
    assert!(matches!(out, Ok(Transformed::Changed { .. })), "{out:?}");
    assert!(
        model.prompts[1]
            .contains("the answer is not one object `{\"ops\": [...]}`: it has `plev_ir`")
    );
}

/// Three refusals: the reasons of the last come back, and nothing
/// changed (the surface asked about is the one it was).
#[test]
fn three_refusals_return_the_reasons_and_change_nothing() {
    let changing = Changing::over(todo(), &compact());
    let before = changing.current.clone();
    let mut model = Scripted::new([
        HIDE_ROOT,
        HIDE_ROOT,
        r#"{"ops": [{"op": "hide", "id": "todo_panel.nope"}]}"#,
    ]);
    let e = transform_verified(&mut Llm::new(&mut model), &changing.ask("less")).unwrap_err();
    match &e {
        ProposeError::Unchanged { attempts, errors } => {
            assert_eq!(*attempts, Llm::<Scripted>::ATTEMPTS);
            assert_eq!(errors.len(), 1);
            assert!(
                errors[0].contains("`hide todo_panel.nope`, was refused"),
                "{errors:?}"
            );
        }
        other => panic!("{other:?}"),
    }
    assert!(
        e.to_string()
            .starts_with("3 answers and none changed the surface")
    );
    assert_eq!(model.prompts.len(), 3);
    assert_eq!(changing.current, before);
    let fails = Scripted {
        replies: VecDeque::from([Err(ModelError("timed out".to_owned()))]),
        prompts: Vec::new(),
    };
    let e = transform_verified(&mut Llm::new(fails), &changing.ask("less")).unwrap_err();
    assert_eq!(e, ProposeError::Model("timed out".to_owned()));
}

/// What the prompt carries: the intent; the operations and the rules of
/// identity; the verifier's rules, the policy and who asks; the
/// registry, where the vault's secret code is its length and never its
/// text; the surface's IR and what is hidden; and how a change reads,
/// with no app named.
#[test]
fn the_prompt_carries_the_operations_the_registry_and_the_surface() {
    let mut changing = Changing::under(vault(), VAULT_PANEL, GenerationPolicy::by(Actor::Human));
    let (hidden, _) = changing
        .current
        .apply(
            &Op::Hide {
                id: sid("vault_panel.note"),
            },
            &Actor::Human,
            changing.ask("").context(),
        )
        .unwrap();
    changing.current = hidden;
    let changing = changing.by(agent());
    let mut model = Scripted::new([r#"{"ops": []}"#]);
    let e = transform_verified(
        &mut Llm::new(&mut model),
        &changing.ask("tem informação demais"),
    )
    .unwrap_err();
    assert!(
        matches!(e, ProposeError::NoChange(_)),
        "no operation is no change: {e:?}"
    );
    let text = &model.prompts[0];
    assert!(text.contains("## the intent\n\ntem informação demais\n"));
    for op in [
        r#"{"op": "hide", "id": "<id>"}"#,
        r#"{"op": "show", "id": "<id>"}"#,
        r#""op": "group""#,
        r#""op": "reorder""#,
        r#""op": "emphasize""#,
        r#""op": "density""#,
        r#""op": "restyle""#,
        r#""op": "swap_view""#,
        r#""op": "promote""#,
    ] {
        assert!(text.contains(op), "{op}");
    }
    assert!(text.contains("only `hide` takes one out of the tree"));
    assert!(text.contains("an item's own id"));
    assert!(text.contains("author human, at most 256 nodes"));
    assert!(text.contains("the change is asked by agent:claude"));
    assert!(text.contains(
        "action vault.open open: takes nothing, only people, assistive technology, tests may ask"
    ));
    // the secret: its length, as the snapshot keeps it, never its text
    assert!(text.contains(
        "vault.code (text_input) \"code\", in vault, secret: its value is only its length"
    ));
    assert!(text.contains("state vault.code:text: int = 17"), "{text}");
    assert!(!text.contains("hunter2"));
    assert!(text.contains("its id is `vault_panel`. its IR:"));
    assert!(text.contains(r#""sem":"vault_panel.lock""#));
    assert!(text.contains(
        "hidden now (`show` brings one back):\n- `vault_panel.note`, from `vault_panel.root`"
    ));
    let reads =
        &text[text.find("## how a change reads").unwrap()..text.find("## the operations").unwrap()];
    assert!(reads.contains("to review items is to see what each one is"));
    assert!(!reads.to_lowercase().contains("silence"));
}

/// An agent asks, and the model answers a promotion of the vault's open,
/// which the vault keeps from agents: refused every time, the reasons
/// say who; the surface does not change.
#[test]
fn an_agent_cannot_promote_an_action_kept_from_agents() {
    let changing =
        Changing::under(vault(), VAULT_PANEL, GenerationPolicy::by(Actor::Human)).by(agent());
    let promote = r#"{"ops": [{"op": "promote", "target": "vault.open", "action": "open", "into": "vault_panel.root"}]}"#;
    let mut model = Scripted::new([promote, promote, promote]);
    match transform_verified(
        &mut Llm::new(&mut model),
        &changing.ask("put the open here"),
    ) {
        Err(ProposeError::Unchanged { errors, .. }) => {
            assert!(errors[0].contains("agent:claude"), "{errors:?}");
        }
        other => panic!("{other:?}"),
    }
}
