//! Random IRs over the registry of `support::caps()`: the verifier never
//! panics and always answers `Ok` or at least one error; the serde form
//! round trips; a verified IR binds only declared capabilities; the IR's
//! JSON holds no number but its version and its proportions; the schema
//! accepts every IR whose names and proportions are in range. Random edits
//! of a valid document: the reader never panics, every error it answers
//! points at a place that is in the document, what it accepts writes back
//! to itself, and whatever the schema accepts the reader reads but for the
//! grammar of an id, a state ref or an action name, which the schema
//! leaves to the reader (a string there, the reader says which one is not
//! an id).

// the validator for the keywords the schema uses (no schema dependency).
// shared with rules.rs: this binary uses the registry only.
#[allow(dead_code)]
use crate::support;

use accord::{ActionName, SemId, StateRef};
use plevir::{
    Align, Axis, CapabilityRef, ColorRole, Component, ControlSize, Elevation, GenerationPolicy, Ir,
    IrNode, Layout, Props, Repeat, Rule, ShapeRole, Size, SizeToken, SpacingToken, Surface,
    TypographyRole, decode, names, schema, verify,
};
use proptest::prelude::*;
use proptest::sample::select;
use serde_json::Value;
use support::caps;

const IDS: &[&str] = &[
    "commit.submit",
    "commit.message",
    "changes.file[a.rs]",
    "review",
    "x[{item}].y[{item}]",
];

const REFS: &[&str] = &[
    "commit.message:text",
    "changes:files",
    "changes:staged",
    "counter:count",
    "commit.form:open",
    "changes.file[{item}]:status",
    "changes.file[{item}]:staged",
    "changes.file[a.rs]:status",
    "odd[{item}]:v",
    "nope:x",
];

const CAPS: &[(&str, &str)] = &[
    ("commit.submit", "submit"),
    ("commit.message", "set_value"),
    ("commit.form", "cancel"),
    ("changes.file[{item}]", "stage"),
    ("changes.file[{item}]", "discard"),
    ("changes.file[a.rs]", "discard"),
    ("half[{item}]", "go"),
    ("admin", "reset"),
    ("trash", "empty"),
    ("nope", "x"),
];

const LABELS: &[&str] = &["", "  ", "commit", "stage", "a file", "é tudo"];

fn sem_id() -> impl Strategy<Value = SemId> {
    prop_oneof![
        (0u8..40).prop_map(|i| format!("n{i}")),
        (0u8..12).prop_map(|i| format!("t{i}[{{item}}]")),
        select(IDS).prop_map(str::to_owned),
    ]
    .prop_map(|text| SemId::parse(&text).expect("the pool holds valid ids"))
}

fn label() -> impl Strategy<Value = Option<String>> {
    prop_oneof![
        Just(None),
        select(LABELS).prop_map(|l| Some(l.to_owned())),
        Just(Some("x".repeat(300))),
    ]
}

fn props() -> impl Strategy<Value = Props> {
    (
        proptest::option::of(select(SpacingToken::ALL)),
        proptest::option::of(select(TypographyRole::ALL)),
        proptest::option::of(select(ColorRole::ALL)),
        proptest::option::of(select(ColorRole::ALL)),
        proptest::option::of(select(Elevation::ALL)),
        proptest::option::of(select(ControlSize::ALL)),
        proptest::option::of(select(ShapeRole::ALL)),
    )
        .prop_map(
            |(padding, typography, color, background, elevation, control_size, shape)| Props {
                padding,
                typography,
                color,
                background,
                elevation,
                control_size,
                shape,
            },
        )
}

/// a layout; `bounded` keeps a split's ratio within what the verifier and
/// the schema allow.
fn layout(bounded: bool) -> impl Strategy<Value = Option<Layout>> {
    let ratio = if bounded { 10u8..=90 } else { 0u8..=255 };
    prop_oneof![
        Just(None),
        (
            select(Axis::ALL),
            proptest::option::of(select(SpacingToken::ALL)),
            proptest::option::of(select(Align::ALL))
        )
            .prop_map(|(axis, gap, align)| Some(Layout::Stack { axis, gap, align })),
        (
            select(SizeToken::ALL),
            proptest::option::of(select(SpacingToken::ALL))
        )
            .prop_map(|(min_col, gap)| Some(Layout::Grid { min_col, gap })),
        (select(Axis::ALL), ratio).prop_map(|(axis, ratio)| Some(Layout::Split { axis, ratio })),
        Just(Some(Layout::Overlay)),
    ]
}

fn size(bounded: bool) -> impl Strategy<Value = Size> {
    let flex = if bounded { 1u8..=12 } else { 0u8..=255 };
    (
        proptest::option::of(select(SizeToken::ALL)),
        proptest::option::of(select(SizeToken::ALL)),
        proptest::option::of(flex),
    )
        .prop_map(|(min, max, flex)| Size { min, max, flex })
}

fn state_ref() -> impl Strategy<Value = StateRef> {
    select(REFS).prop_map(|r| r.parse().expect("the pool holds valid refs"))
}

fn capability() -> impl Strategy<Value = CapabilityRef> {
    select(CAPS).prop_map(|(target, action)| CapabilityRef {
        target: SemId::parse(target).expect("a valid id"),
        action: ActionName::parse(action).expect("a valid name"),
    })
}

/// a node without children; `bounded` also keeps its binding names to the
/// ones some component accepts.
fn leaf(bounded: bool) -> impl Strategy<Value = IrNode> {
    let (states, events) = names();
    let bogus: &[&str] = if bounded { &[] } else { &["bogus"] };
    let state_names: Vec<String> = states.iter().chain(bogus).map(|s| s.to_string()).collect();
    let event_names: Vec<String> = events.iter().chain(bogus).map(|s| s.to_string()).collect();
    (
        sem_id(),
        proptest::option::of(select(Component::ALL.to_vec())),
        label(),
        props(),
        layout(bounded),
        size(bounded),
        proptest::collection::btree_map(select(state_names), state_ref(), 0..3),
        proptest::collection::btree_map(select(event_names), capability(), 0..3),
    )
        .prop_map(
            |(sem, component, label, props, layout, size, state, actions)| {
                let mut node = IrNode::new(sem);
                node.component = component;
                node.label = label;
                node.props = props;
                node.layout = layout;
                node.size = size;
                node.bind.state = state;
                node.bind.actions = actions;
                node
            },
        )
}

fn node(bounded: bool) -> impl Strategy<Value = IrNode> {
    leaf(bounded).prop_recursive(4, 48, 4, move |inner| {
        (
            leaf(bounded),
            proptest::collection::vec(inner.clone(), 0..4),
            proptest::option::of((state_ref(), inner)),
        )
            .prop_map(|(mut node, children, repeat)| {
                node.children = children;
                node.repeat = repeat.map(|(each, item)| Box::new(Repeat { each, item }));
                node
            })
    })
}

fn ir(bounded: bool) -> impl Strategy<Value = Ir> {
    (sem_id(), select(LABELS), node(bounded)).prop_map(|(id, title, root)| {
        Ir::new(Surface {
            id,
            title: title.to_owned(),
            root,
        })
    })
}

/// every action a verified IR binds, walked by hand.
fn bound(n: &IrNode, out: &mut Vec<CapabilityRef>) {
    out.extend(n.bind.actions.values().cloned());
    for child in &n.children {
        bound(child, out);
    }
    if let Some(repeat) = &n.repeat {
        bound(&repeat.item, out);
    }
}

/// every number in a JSON value, with the key it sits under.
fn numbers<'v>(v: &'v Value, key: &'v str, out: &mut Vec<&'v str>) {
    match v {
        Value::Number(_) => out.push(key),
        Value::Array(list) => list.iter().for_each(|x| numbers(x, key, out)),
        Value::Object(map) => map.iter().for_each(|(k, x)| numbers(x, k, out)),
        _ => {}
    }
}

/// every JSON pointer of a value, the document included.
fn pointers(v: &Value, at: String, out: &mut Vec<String>) {
    match v {
        Value::Array(list) => {
            for (i, x) in list.iter().enumerate() {
                pointers(x, format!("{at}/{i}"), out);
            }
        }
        Value::Object(map) => {
            for (k, x) in map {
                pointers(
                    x,
                    format!("{at}/{}", k.replace('~', "~0").replace('/', "~1")),
                    out,
                );
            }
        }
        _ => {}
    }
    out.push(at);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn the_verifier_always_answers_and_never_panics(ir in ir(false)) {
        let caps = caps();
        for policy in [
            GenerationPolicy::default(),
            GenerationPolicy { max_nodes: 5, max_depth: 2, max_text: 4, ..GenerationPolicy::default() },
        ] {
            match verify(&ir, &caps, &policy) {
                Ok(verified) => {
                    let mut caps_bound = Vec::new();
                    bound(&verified.ir().surface.root, &mut caps_bound);
                    for c in caps_bound {
                        let declared = caps.action(&c.target, &c.action).is_some()
                            || c.target.as_str().contains("[{item}]");
                        prop_assert!(declared, "{c:?} is not declared");
                    }
                }
                Err(errors) => prop_assert!(!errors.is_empty()),
            }
        }
    }

    #[test]
    fn the_schema_accepts_every_ir_within_its_ranges(ir in ir(true)) {
        let doc = serde_json::to_value(&ir).unwrap();
        let errors = support::jsonschema::validate(&schema(), &doc);
        prop_assert!(errors.is_empty(), "{errors:#?}");
    }

    #[test]
    fn the_serde_form_round_trips(ir in ir(false)) {
        let json = serde_json::to_value(&ir).unwrap();
        prop_assert_eq!(&serde_json::from_value::<Ir>(json.clone()).unwrap(), &ir);
        prop_assert_eq!(&decode(&json).unwrap(), &ir);
        let mut found = Vec::new();
        numbers(&json, "", &mut found);
        for key in found {
            prop_assert!(["plev_ir", "ratio", "flex"].contains(&key), "a number under {key}");
        }
    }

    #[test]
    fn an_edited_document_reads_or_says_where_it_breaks(
        ir in ir(false),
        pick in any::<prop::sample::Index>(),
        edit in 0u8..6,
    ) {
        let mut doc = serde_json::to_value(&ir).unwrap();
        let mut all = Vec::new();
        pointers(&doc, String::new(), &mut all);
        let at = pick.get(&all).clone();
        let replacement = match edit {
            0 => Value::Null,
            1 => serde_json::json!(12),
            2 => serde_json::json!("#ff0000"),
            3 => serde_json::json!({ "unknown": true }),
            4 => serde_json::json!([]),
            _ => serde_json::json!("n1"),
        };
        if let Some(slot) = doc.pointer_mut(&at) {
            *slot = replacement;
        }
        let fits = support::jsonschema::validate(&schema(), &doc).is_empty();
        if let (true, Err(errors)) = (fits, decode(&doc)) {
            for e in errors {
                let grammar = matches!(e.rule, Rule::Identity | Rule::State | Rule::Action)
                    && doc.pointer(&e.pointer).is_some_and(Value::is_string);
                prop_assert!(grammar, "the schema takes what the reader refuses: {e} in {doc}");
            }
        }
        match decode(&doc) {
            Ok(read) => {
                let again = serde_json::to_value(&read).unwrap();
                prop_assert_eq!(decode(&again).unwrap(), read);
            }
            Err(errors) => {
                prop_assert!(!errors.is_empty());
                for e in errors {
                    prop_assert!(doc.pointer(&e.pointer).is_some(), "{e} points outside {doc}");
                }
            }
        }
    }
}

#[test]
fn a_tree_deeper_than_any_policy_is_refused_without_exhausting_the_stack() {
    let mut node = IrNode::new(SemId::parse("leaf").unwrap());
    for i in 0..2000 {
        let mut parent = IrNode::new(SemId::parse(&format!("n{i}")).unwrap());
        parent.children.push(node);
        node = parent;
    }
    let ir = Ir::new(Surface {
        id: SemId::parse("deep").unwrap(),
        title: "deep".to_owned(),
        root: node,
    });
    let unbounded = GenerationPolicy {
        max_nodes: usize::MAX,
        max_depth: usize::MAX,
        ..GenerationPolicy::default()
    };
    let errors = verify(&ir, &caps(), &unbounded).unwrap_err();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule, plevir::Rule::Limits);
    assert!(
        errors[0].reason.contains("depth 257"),
        "{}",
        errors[0].reason
    );
}
