//! The deterministic proposer over real projections: every intent of its
//! grammar gives an IR the verifier accepts; the nodes whose capability
//! stays keep the current surface's ids; the words that choose are the
//! app's; an intent it does not understand is a clear error, never an
//! empty surface; the same rules read an app they were not written for
//! (the ide's workbench); and "items" repeats over a list found by
//! structure, in the todo and in the ide, keeping the template's ids
//! (task-95).

// shared with the other test binaries: this one reads less of it.
#[allow(dead_code)]
use crate::support;

use accord::{Actor, Actors, Policy, Role, SnapshotBuilder};
use offers::{Ask, Part, ProposeError, Proposer, Rules, propose_verified};
use plevir::{Capabilities, GenerationPolicy, Ir, IrNode, Layout, VerifiedIr, verify};
use support::{Asking, bindings, columns, compact, ide, ids, todo, verified};

fn propose(asking: &Asking, intent: &str) -> VerifiedIr {
    propose_verified(&mut Rules, &asking.ask(intent))
        .unwrap_or_else(|e| panic!("\"{intent}\": {e}"))
}

fn has(ir: &VerifiedIr, id: &str, binding: &str) -> bool {
    bindings(&ir.ir().surface.root)
        .iter()
        .any(|(at, b)| at == id && b.iter().any(|x| x == binding))
}

fn binds(ir: &VerifiedIr, binding: &str) -> bool {
    bindings(&ir.ir().surface.root)
        .iter()
        .any(|(_, b)| b.iter().any(|x| x == binding))
}

const ADD: &str = "set_value=todo.input set_value";
const SUBMIT: &str = "activate=todo.input submit";
const ALL: &str = "activate=todo.filter[all] select";
const ACTIVE: &str = "activate=todo.filter[active] select";
const COMPLETED: &str = "activate=todo.filter[completed] select";
const N_ACTIVE: &str = "value=todo.counter:active";
const N_DONE: &str = "value=todo.counter:done";

/// Each intent of the grammar, in english and in portuguese, over the
/// todo with no surface open and over the page's surface: the IR
/// verifies, and it binds what the intent asked for and nothing else.
#[test]
fn every_intent_of_the_grammar_verifies_over_the_todo() {
    let cases: [(&[&str], &[&str], &[&str]); 5] = [
        (
            &["add", "adicionar", "the add field"],
            &[ADD, SUBMIT],
            &[ALL, N_ACTIVE],
        ),
        (
            &["filter", "filtro", "filtros"],
            &[ALL, ACTIVE, COMPLETED],
            &[ADD, N_DONE],
        ),
        (
            &["counts", "stats", "contagem"],
            &[N_ACTIVE, N_DONE],
            &[ADD, ALL],
        ),
        (
            &["everything", "tudo", "add, filter and counts"],
            &[ADD, SUBMIT, ALL, ACTIVE, COMPLETED, N_ACTIVE, N_DONE],
            &[],
        ),
        (
            &["only what is active and the add field"],
            &[ADD, SUBMIT, ACTIVE, N_ACTIVE],
            &[ALL, COMPLETED, N_DONE],
        ),
    ];
    let fresh = Asking::new(todo());
    let over = Asking::new(todo()).over(verified(&compact(), &todo()));
    for asking in [&fresh, &over] {
        for (intents, bound, absent) in cases {
            for intent in intents {
                let ir = propose(asking, intent);
                for b in bound {
                    assert!(binds(&ir, b), "\"{intent}\" binds {b}");
                }
                for b in absent {
                    assert!(!binds(&ir, b), "\"{intent}\" does not bind {b}");
                }
            }
        }
    }
    // the forms, alone (the whole app) and with a part
    for (intent, grid) in [
        ("compact", false),
        ("compacto", false),
        ("two columns", true),
        ("duas colunas", true),
        ("counts in two columns", true),
        ("uma coluna", false),
    ] {
        let ir = propose(&fresh, intent);
        let root = &ir.ir().surface.root;
        assert_eq!(
            matches!(root.layout, Some(Layout::Grid { .. })),
            grid,
            "\"{intent}\""
        );
    }
    assert!(binds(&propose(&fresh, "duas colunas"), ADD));
    assert!(!binds(&propose(&fresh, "counts in two columns"), ADD));
}

/// The acceptance of phase 2 depends on this: over the page's surface,
/// every node whose capability stays keeps its id (and its label), and
/// so do the containers that hold the same capabilities; the surface
/// keeps its id, so the page transforms it in place.
#[test]
fn the_ids_of_what_stays_are_the_current_surfaces() {
    let over = Asking::new(todo()).over(verified(&compact(), &todo()));
    let ir = propose(&over, "only what is active and the add field");
    assert_eq!(ir.ir().surface.id.as_str(), "todo_panel");
    assert_eq!(ir.ir().surface.title, "quick todo");
    assert_eq!(
        ids(&ir.ir().surface.root),
        [
            "todo_panel.card",
            "todo_panel.entry",
            "todo_panel.input",
            "todo_panel.add",
            "todo_panel.filter",
            "todo_panel.filter.active",
            "todo_panel.counts",
            "todo_panel.active",
        ]
    );
    assert!(has(&ir, "todo_panel.add", SUBMIT));
    let root = &ir.ir().surface.root;
    let add = &root.children[0].children[1];
    assert_eq!(add.label.as_deref(), Some("add"), "the label stays too");

    // from the two-column form to the compact one: the controls keep
    // their ids, the filter its container, and the counts take the id of
    // the column that held them there
    let over = Asking::new(todo()).over(verified(&columns(), &todo()));
    let ir = propose(&over, "compact");
    let now = ids(&ir.ir().surface.root);
    for id in [
        "todo_panel.card",
        "todo_panel.entry",
        "todo_panel.input",
        "todo_panel.add",
        "todo_panel.filter",
        "todo_panel.filter.all",
        "todo_panel.filter.active",
        "todo_panel.filter.completed",
        "todo_panel.active",
        "todo_panel.done",
    ] {
        assert!(now.contains(&id.to_owned()), "{id} in {now:?}");
    }
    let counts = ir.ir().surface.root.children.last().expect("the counts");
    assert_eq!(counts.sem.as_str(), "todo_panel.side");
    assert!(!now.contains(&"todo_panel.counts".to_owned()));
    assert!(matches!(
        ir.ir().surface.root.layout,
        Some(Layout::Stack { .. })
    ));
}

/// A proposal over a proposal: the rules' own ids go on as the fixture's
/// did, through a change of form and a narrower choice; with no form
/// word, the form is the current one.
#[test]
fn a_proposal_over_a_proposal_keeps_its_ids() {
    let fresh = Asking::new(todo());
    let first = propose(&fresh, "everything");
    assert_eq!(first.ir().surface.id.as_str(), "proposed");
    assert_eq!(first.ir().surface.title, "everything");
    let before = bindings(&first.ir().surface.root);

    let then = Asking::new(todo()).over(first.clone());
    let columns = propose(&then, "two columns");
    assert_eq!(columns.ir().surface.id, first.ir().surface.id);
    let mut after = bindings(&columns.ir().surface.root);
    let mut sorted = before.clone();
    sorted.sort();
    after.sort();
    assert_eq!(after, sorted, "the same nodes bound, by the same ids");

    let again = Asking::new(todo()).over(columns.clone());
    let narrow = propose(&again, "only active");
    assert!(matches!(
        narrow.ir().surface.root.layout,
        Some(Layout::Grid { .. })
    ));
    for (id, b) in bindings(&narrow.ir().surface.root) {
        assert!(before.contains(&(id.clone(), b)), "{id} kept its id");
    }
    let first_ids = ids(&first.ir().surface.root);
    for id in ids(&narrow.ir().surface.root) {
        let side = id.ends_with(".side") || id.ends_with(".main");
        assert!(side || first_ids.contains(&id), "{id} is new");
    }
}

/// Nothing in the rules names the todo: over the ide's workbench the
/// entry is the commit message with the commit button beside it (no
/// action of the field commits it), the counts are the diff's line count,
/// and there is no choice to filter by.
#[test]
fn the_same_rules_read_the_ide() {
    let asking = Asking::new(ide());
    let ir = propose(&asking, "add");
    let b = bindings(&ir.ir().surface.root);
    let field = b.iter().find(|(id, _)| id.ends_with("commit.message"));
    assert!(field.is_some_and(|(_, b)| b.contains(&"text=commit.message:text".to_owned())));
    let button = &ir.ir().surface.root.children[0].children[1];
    assert_eq!(button.label.as_deref(), Some("Commit"));
    assert!(binds(&ir, "activate=commit.submit submit"));
    assert!(binds(&propose(&asking, "counts"), "value=diff:count"));
    match propose_verified(&mut Rules, &asking.ask("filter")) {
        Err(ProposeError::Nothing { part, .. }) => assert_eq!(part, Part::Choice),
        other => panic!("no choice: {other:?}"),
    }
}

/// An intent the rules do not read is an error that says why, never an
/// empty surface: no word of the grammar, or a word that is not the
/// app's (the app's words are listed). the choosing word is the app's,
/// in the app's language: "ativo" is not a word of the todo, "active" is.
#[test]
fn an_intent_the_rules_do_not_understand_is_a_clear_error() {
    let asking = Asking::new(todo());
    let refuse = |intent: &str| propose_verified(&mut Rules, &asking.ask(intent)).unwrap_err();
    for blank in ["", "   ", "the only field"] {
        let e = refuse(blank);
        assert!(matches!(e, ProposeError::NotUnderstood { .. }), "{e:?}");
        assert!(e.to_string().contains("adicionar"), "{e}");
    }
    let e = refuse("só o que está ativo e o campo de adicionar");
    let ProposeError::NoMatch { words, app } = &e else {
        panic!("no match: {e:?}");
    };
    assert_eq!(words, &["ativo"]);
    for w in [
        "active",
        "all",
        "completed",
        "done",
        "filter",
        "new",
        "todo",
    ] {
        assert!(app.contains(&w.to_owned()), "{w} in {app:?}");
    }
    assert!(
        e.to_string()
            .starts_with("\"ativo\" is not a word of the app; its words are active, all"),
        "{e}"
    );
    let e = refuse("make it purple and sparkly");
    assert!(
        matches!(&e, ProposeError::NoMatch { words, .. } if words == &["purple", "sparkly"]),
        "{e:?}"
    );
    let empty = Asking::new(accord::Snapshot::default());
    let e = propose_verified(&mut Rules, &empty.ask("everything")).unwrap_err();
    assert_eq!(e, ProposeError::Empty);
    let e = propose_verified(&mut Rules, &empty.ask("add")).unwrap_err();
    assert!(matches!(
        e,
        ProposeError::Nothing {
            part: Part::Entry,
            ..
        }
    ));
}

/// The rules bind only what the IR's author may ask for: an option the
/// app keeps from agents is not proposed to an agent, and is to a person.
#[test]
fn the_rules_bind_only_what_the_author_may_ask_for() {
    let mut b = SnapshotBuilder::new();
    b.node("view", Role::TabList, "view")
        .state("selected", "list");
    b.node("view[list]", Role::Tab, "list")
        .parent("view")
        .action("select");
    b.node("view[admin]", Role::Tab, "admin")
        .parent("view")
        .action("select")
        .policy(Policy {
            actors: Actors {
                agent: false,
                ..Actors::default()
            },
            ..Policy::default()
        });
    let app = b.finish().unwrap();
    let agent = Asking::new(app.clone());
    let ir = propose(&agent, "filter");
    assert!(binds(&ir, "activate=view[list] select"));
    assert!(!binds(&ir, "activate=view[admin] select"));
    let mut person = Asking::new(app);
    person.policy = GenerationPolicy::by(Actor::Human);
    assert!(binds(
        &propose(&person, "filter"),
        "activate=view[admin] select"
    ));
}

/// The same ask gives the same IR.
#[test]
fn the_rules_are_deterministic() {
    let over = Asking::new(todo()).over(verified(&compact(), &todo()));
    let intent = "only what is active and the add field";
    let a = Rules.propose(&over.ask(intent)).unwrap();
    let b = Rules.propose(&over.ask(intent)).unwrap();
    assert_eq!(a, b);
}

/// A proposer that answers what the verifier refuses: `propose_verified`
/// returns the verifier's errors, all of them, and no IR.
#[test]
fn propose_verified_never_returns_what_the_verifier_refuses() {
    struct Careless;
    impl Proposer for Careless {
        fn propose(&mut self, _: &Ask<'_>) -> Result<Ir, ProposeError> {
            let text = compact().replace("todo.counter:active", "todo.counter:gone");
            Ok(serde_json::from_str(&text).unwrap())
        }
    }
    let asking = Asking::new(todo());
    let caps = Capabilities::from_snapshot(&asking.app);
    let wrong: Ir = Careless.propose(&asking.ask("anything")).unwrap();
    let expected = verify(&wrong, &caps, &asking.policy).unwrap_err();
    match propose_verified(&mut Careless, &asking.ask("anything")) {
        Err(ProposeError::Refused(errors)) => assert_eq!(errors, expected),
        other => panic!("refused: {other:?}"),
    }
    // and the rules under a policy their surface does not fit
    let mut tight = Asking::new(todo());
    tight.policy.max_nodes = 3;
    let e = propose_verified(&mut Rules, &tight.ask("everything")).unwrap_err();
    assert!(matches!(e, ProposeError::Refused(_)), "{e:?}");
}

/// The repeat of a proposal's first list: what it repeats over and its
/// template.
fn repeat(ir: &VerifiedIr) -> Option<(String, IrNode)> {
    support::nodes(&ir.ir().surface.root)
        .into_iter()
        .find_map(|n| n.repeat.as_deref())
        .map(|r| (r.each.to_string(), r.item.clone()))
}

/// "items" over the todo: a list that repeats over `todo:items`, found by
/// structure (a List whose values key its node's children), each item
/// its text and a checkbox bound to `completed` and its `toggle` (the
/// action that reads only it); never the destructive `remove`, and
/// nothing of the other parts. In both languages, and in both forms.
#[test]
fn the_items_are_a_repeat_over_the_todos_list() {
    let fresh = Asking::new(todo());
    for intent in [
        "items",
        "itens",
        "the list",
        "a lista",
        "os itens em duas colunas",
    ] {
        let ir = propose(&fresh, intent);
        let (each, item) = repeat(&ir).unwrap_or_else(|| panic!("\"{intent}\" repeats"));
        assert_eq!(each, "todo:items", "\"{intent}\"");
        assert!(item.sem.as_str().ends_with("[{item}]"), "{}", item.sem);
        assert_eq!(item.label, None, "an instance is named by its item");
        let b: Vec<String> = bindings(&item).into_iter().flat_map(|(_, b)| b).collect();
        assert_eq!(
            b,
            [
                "text=todo.item[{item}]:text",
                "checked=todo.item[{item}]:completed",
                "toggle=todo.item[{item}] toggle",
            ],
            "\"{intent}\""
        );
        for other in [ADD, ALL, N_ACTIVE] {
            assert!(!binds(&ir, other), "\"{intent}\" binds {other}");
        }
    }
    let everything = propose(&fresh, "everything");
    assert!(repeat(&everything).is_some(), "everything holds the list");
    assert!(binds(&everything, ADD) && binds(&everything, N_DONE));
}

/// The same rules over the ide's workbench: the list is `changes:files`
/// (the `changes:staged` set holds only some of the files: a selection,
/// not the list), each file its status as text (no two actions set it
/// alone: no badge) and a button per action of no parameter, grouped;
/// `stage` reads two states, so no checkbox, and the destructive
/// `discard` is left out. The app's own word for its items
/// ("files") chooses the list too.
#[test]
fn the_same_rules_list_the_ides_files() {
    let asking = Asking::new(ide());
    for intent in ["items", "the files"] {
        let ir = propose(&asking, intent);
        let (each, item) = repeat(&ir).unwrap_or_else(|| panic!("\"{intent}\" repeats"));
        assert_eq!(each, "changes:files", "\"{intent}\"");
        let b: Vec<String> = bindings(&item).into_iter().flat_map(|(_, b)| b).collect();
        assert_eq!(
            b,
            [
                "text=changes.file[{item}]:status",
                "activate=changes.file[{item}] stage",
                "activate=changes.file[{item}] unstage",
                "activate=changes.file[{item}] ignore",
                "activate=changes.file[{item}] show_diff",
            ],
            "\"{intent}\""
        );
        // by sense: the status alone, the four actions as one group
        assert_eq!(item.children.len(), 2);
        assert_eq!(item.children[0].component, Some(plevir::Component::Text));
        let actions = &item.children[1];
        assert!(
            actions.sem.as_str().ends_with("[{item}].actions"),
            "{}",
            actions.sem
        );
        let labels: Vec<Option<&str>> = actions
            .children
            .iter()
            .map(|n| n.label.as_deref())
            .collect();
        assert_eq!(
            labels,
            [
                Some("stage"),
                Some("unstage"),
                Some("ignore"),
                Some("show diff")
            ]
        );
    }
    assert!(!binds(&propose(&asking, "counts"), "toggle"));
}

/// Over the page's surface, which shows the items, a proposal that keeps
/// them keeps the ids of the list and of its template, so every instance
/// (`todo_panel.item[<id>]` and its checkbox) goes on through the change,
/// and the checkbox keeps its label. An intent without the items shows
/// none (the phase 3a tests pin that the rest is as it was).
#[test]
fn a_proposal_keeps_the_ids_of_the_list_and_its_template() {
    for current in [compact(), columns()] {
        let over = Asking::new(todo()).over(verified(&current, &todo()));
        for intent in ["items", "two columns", "compact", "everything"] {
            let ir = propose(&over, intent);
            let root = &ir.ir().surface.root;
            let list = support::nodes(root)
                .into_iter()
                .find(|n| n.repeat.is_some())
                .unwrap_or_else(|| panic!("\"{intent}\" keeps the list"));
            assert_eq!(list.sem.as_str(), "todo_panel.items", "\"{intent}\"");
            let (_, item) = repeat(&ir).expect("a template");
            assert_eq!(item.sem.as_str(), "todo_panel.item[{item}]");
            let ids: Vec<(String, Option<String>)> = item
                .children
                .iter()
                .map(|n| (n.sem.to_string(), n.label.clone()))
                .collect();
            assert_eq!(
                ids,
                [
                    ("todo_panel.item[{item}].text".to_owned(), None),
                    (
                        "todo_panel.item[{item}].done".to_owned(),
                        Some("done".to_owned())
                    ),
                ],
                "\"{intent}\""
            );
        }
        let narrow = propose(&over, "only what is active and the add field");
        assert!(repeat(&narrow).is_none(), "no items asked, none shown");
    }
}

/// A list found by structure only: its values must be the keys of its
/// node's children, all of them. An empty list proves nothing (and a
/// template over it would not verify); a list whose values name no
/// child, or only some, is no list of items.
#[test]
fn a_list_is_found_by_structure_and_an_empty_one_is_nothing() {
    let app = |items: &[&str], children: &[&str]| {
        let mut b = SnapshotBuilder::new();
        let list: Vec<String> = items.iter().map(|s| (*s).to_owned()).collect();
        b.node("shelf", Role::List, "shelf")
            .state("books", accord::Value::List(list));
        for c in children {
            let id = format!("shelf.book[{c}]");
            let mut n = b.node(id.as_str(), Role::ListItem, *c);
            n.parent("shelf").state("read", false);
            n.action("flip").reads(format!("{id}:read"));
        }
        b.finish().unwrap()
    };
    let found = Asking::new(app(&["a", "b"], &["a", "b"]));
    let ir = propose(&found, "the books");
    let (each, item) = repeat(&ir).expect("the shelf's books");
    assert_eq!(each, "shelf:books");
    let b: Vec<String> = bindings(&item).into_iter().flat_map(|(_, b)| b).collect();
    assert_eq!(
        b,
        [
            "checked=shelf.book[{item}]:read",
            "toggle=shelf.book[{item}] flip"
        ]
    );
    for (items, children) in [
        (&[][..], &["a"][..]),
        (&["a", "b"][..], &["a"][..]),
        (&["a"][..], &["a", "b"][..]),
    ] {
        let asking = Asking::new(app(items, children));
        match propose_verified(&mut Rules, &asking.ask("items")) {
            Err(ProposeError::Nothing { part, .. }) => assert_eq!(part, Part::List),
            other => panic!("{items:?} over {children:?}: {other:?}"),
        }
    }
}
