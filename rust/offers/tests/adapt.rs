//! The way out of a change (task-70 phase 2): a transformer answers
//! operations, and `transform_verified` applies each one in order over
//! the surface asked about, verified again, by who asked. It returns the
//! operations only with the verified surface they make; the first one
//! refused is the error, with its place; no operation is no change; and
//! an undo passes through to the host, which knows the last change.

// shared with the other test binaries: this one reads less of it.
#[allow(dead_code)]
use crate::support;

use accord::{Actor, SemId};
use offers::{Adapt, Adaptation, ProposeError, Transformed, Transformer, transform_verified};
use plevir::{GenerationPolicy, Op, Step, TransformError};
use support::{Changing, VAULT_PANEL, agent, compact, todo, vault};

fn id(text: &str) -> SemId {
    SemId::parse(text).unwrap()
}

/// A transformer that answers what it was given, and keeps who asked.
struct Fixed {
    answer: Adaptation,
    asked_by: Vec<Actor>,
}

impl Fixed {
    fn new(answer: Adaptation) -> Fixed {
        Fixed {
            answer,
            asked_by: Vec::new(),
        }
    }
}

impl Transformer for Fixed {
    fn transform(&mut self, ask: &Adapt<'_>) -> Result<Adaptation, ProposeError> {
        self.asked_by.push(ask.by.clone());
        Ok(self.answer.clone())
    }
}

#[test]
fn the_way_out_applies_every_operation_and_returns_the_verified_surface() {
    let changing = Changing::over(todo(), &compact());
    let ops = vec![
        Op::Hide {
            id: id("todo_panel.counts"),
        },
        Op::Density { step: Step::Denser },
    ];
    let mut fixed = Fixed::new(Adaptation::Ops(ops.clone()));
    let out = transform_verified(&mut fixed, &changing.ask("too much")).expect("changes");
    let Transformed::Changed { ops: done, adapted } = out else {
        panic!("a change");
    };
    assert_eq!(done, ops);
    // the same surface the operations make one after the other
    let cx = changing.ask("").context();
    let (hidden, _) = changing
        .current
        .apply(&ops[0], &Actor::Human, cx)
        .expect("hides");
    let (dense, _) = hidden.apply(&ops[1], &Actor::Human, cx).expect("tightens");
    assert_eq!(*adapted, dense);
    let aside: Vec<&SemId> = adapted.aside().map(|a| &a.node.sem).collect();
    assert_eq!(aside, [&id("todo_panel.counts")]);
    assert_eq!(fixed.asked_by, [Actor::Human]);
}

#[test]
fn the_first_operation_refused_is_the_error_with_its_place() {
    let changing = Changing::over(todo(), &compact());
    let mut fixed = Fixed::new(Adaptation::Ops(vec![
        Op::Density { step: Step::Denser },
        Op::Hide {
            id: id("todo_panel.card"),
        },
        Op::Hide {
            id: id("todo_panel.nope"),
        },
    ]));
    match transform_verified(&mut fixed, &changing.ask("less")) {
        Err(ProposeError::Unapplied { at, op, error }) => {
            assert_eq!(at, 2);
            assert_eq!(op, "hide todo_panel.card");
            assert!(matches!(error, TransformError::Refused(_)), "{error:?}");
        }
        other => panic!("refused: {other:?}"),
    }
    let e = transform_verified(&mut fixed, &changing.ask("less")).unwrap_err();
    assert!(
        e.to_string()
            .starts_with("operation 2, hide todo_panel.card, was refused: "),
        "{e}"
    );
}

#[test]
fn no_operation_is_no_change_and_an_undo_passes_through() {
    let changing = Changing::over(todo(), &compact());
    let e = transform_verified(
        &mut Fixed::new(Adaptation::Ops(Vec::new())),
        &changing.ask("x"),
    )
    .unwrap_err();
    assert!(matches!(e, ProposeError::NoChange(_)), "{e:?}");
    let undo = transform_verified(&mut Fixed::new(Adaptation::Undo), &changing.ask("undo"));
    assert_eq!(undo, Ok(Transformed::Undo));
}

/// A person's surface over the vault: a person may promote the open, an
/// agent may not, whatever the transformer answers; the lock stays kept
/// from agents after the person's promotion.
#[test]
fn a_promotion_is_checked_for_who_asks() {
    let promote = Op::Promote {
        target: id("vault.open"),
        action: "open".parse().unwrap(),
        into: id("vault_panel.root"),
    };
    let person = Changing::under(vault(), VAULT_PANEL, GenerationPolicy::by(Actor::Human));
    let mut fixed = Fixed::new(Adaptation::Ops(vec![promote.clone()]));
    let Ok(Transformed::Changed { adapted, .. }) = transform_verified(&mut fixed, &person.ask("x"))
    else {
        panic!("a person promotes the open");
    };
    let button = adapted
        .ir()
        .surface
        .root
        .children
        .last()
        .expect("the promoted button");
    assert_eq!(button.sem, id("vault_panel.root.open"));

    let by_agent =
        Changing::under(vault(), VAULT_PANEL, GenerationPolicy::by(Actor::Human)).by(agent());
    match transform_verified(&mut fixed, &by_agent.ask("x")) {
        Err(ProposeError::Unapplied {
            error: TransformError::Refused(why),
            ..
        }) => assert!(why.contains("agent:claude"), "{why}"),
        other => panic!("an agent does not promote the open: {other:?}"),
    }
}
