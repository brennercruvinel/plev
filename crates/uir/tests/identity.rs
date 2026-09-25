//! Identity, as a property: whatever operation applies to a verified
//! surface, the surface it makes passes the verifier, no id leaves the
//! surface (every id before is in its tree or set aside, and only `Hide`
//! takes one out of the tree), and its inverse gives the surface before
//! byte for byte. Over random surfaces that verify by construction, with
//! random sequences of operations undone in the order a history undoes
//! them; and over the real fixtures, every operation on every node.

// shared with the other uir tests: this binary uses the registry only.
#[allow(dead_code)]
mod support;

use std::collections::BTreeSet;
use std::path::PathBuf;

use proptest::prelude::*;
use proptest::sample::select;
use sem::{Actor, SemId, Snapshot};
use support::caps;
use uir::{
    Adapted, Align, Axis, Capabilities, CapabilityRef, ChartKind, ColorRole, Component, Context,
    ControlSize, Elevation, GenerationPolicy, Inverse, Ir, IrNode, Layout, Op, Props, Repeat,
    ShapeRole, SizeToken, SpacingToken, Step, Surface, TransformError, TypographyRole, parse,
    verify,
};

fn id(text: &str) -> SemId {
    SemId::parse(text).expect("a valid id")
}

fn cap(target: &str, action: &str) -> CapabilityRef {
    CapabilityRef::parse(target, action).expect("a valid capability")
}

/// A leaf that verifies over `caps()` wherever it sits outside a
/// template, its id to be given.
fn leaf(kind: u8) -> IrNode {
    let mut n = IrNode::new(id("x"));
    let label = |n: &mut IrNode, text: &str| n.label = Some(text.to_owned());
    let state = |n: &mut IrNode, prop: &str, at: &str| {
        n.bind
            .state
            .insert(prop.to_owned(), at.parse().expect("a valid ref"));
    };
    match kind % 10 {
        0 => {
            n.component = Some(Component::Text);
            label(&mut n, "a text");
        }
        1 => {
            n.component = Some(Component::Text);
            state(&mut n, "text", "counter:count");
        }
        2 => {
            n.component = Some(Component::Stat);
            label(&mut n, "files");
            state(&mut n, "value", "counter:count");
        }
        3 => {
            n.component = Some(Component::Badge);
            state(&mut n, "text", "counter:count");
        }
        4 => {
            n.component = Some(Component::Button);
            label(&mut n, "commit");
            n.bind
                .actions
                .insert("activate".to_owned(), cap("commit.submit", "submit"));
        }
        5 => {
            n.component = Some(Component::TextField);
            label(&mut n, "message");
            state(&mut n, "text", "commit.message:text");
            n.bind
                .actions
                .insert("set_value".to_owned(), cap("commit.message", "set_value"));
        }
        6 => {
            n.component = Some(Component::Table);
            label(&mut n, "the files");
            state(&mut n, "rows", "changes:files");
        }
        7 => {
            n.component = Some(Component::Chart(ChartKind::Bar));
            label(&mut n, "the files");
            state(&mut n, "data", "changes:files");
        }
        8 => {
            n.component = Some(Component::Checkbox);
            label(&mut n, "open");
            state(&mut n, "checked", "commit.form:open");
            n.bind
                .actions
                .insert("toggle".to_owned(), cap("commit.form", "cancel"));
        }
        _ => n.component = Some(Component::Separator),
    }
    n
}

/// A list of the changed files: a badge and a stage button per file.
fn list() -> IrNode {
    let mut badge = IrNode::new(id("x[{item}]"));
    badge.component = Some(Component::Badge);
    badge.bind.state.insert(
        "text".to_owned(),
        "changes.file[{item}]:status".parse().expect("a valid ref"),
    );
    let mut stage = IrNode::new(id("x[{item}]"));
    stage.component = Some(Component::Button);
    stage.label = Some("stage".to_owned());
    stage
        .bind
        .actions
        .insert("activate".to_owned(), cap("changes.file[{item}]", "stage"));
    let mut item = IrNode::new(id("x[{item}]"));
    item.children = vec![badge, stage];
    let mut list = IrNode::new(id("x"));
    list.component = Some(Component::List);
    list.label = Some("files".to_owned());
    list.repeat = Some(Box::new(Repeat {
        each: "changes:files".parse().expect("a valid ref"),
        item,
    }));
    list
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

/// A layout for a container of `n` children: a stack, a grid, nothing,
/// or a split when there are two.
fn layout(n: usize) -> impl Strategy<Value = Option<Layout>> {
    let stack = (
        select(Axis::ALL),
        proptest::option::of(select(SpacingToken::ALL)),
        proptest::option::of(select(Align::ALL)),
    )
        .prop_map(|(axis, gap, align)| Some(Layout::Stack { axis, gap, align }));
    let grid = (
        select(SizeToken::ALL),
        proptest::option::of(select(SpacingToken::ALL)),
    )
        .prop_map(|(min_col, gap)| Some(Layout::Grid { min_col, gap }));
    let split = (select(Axis::ALL), 10u8..=90)
        .prop_map(move |(axis, ratio)| (n == 2).then_some(Layout::Split { axis, ratio }));
    prop_oneof![Just(None), stack, grid, split]
}

/// A node that verifies by construction: a leaf, a list, or a container
/// (plain or a card) of such nodes.
fn node() -> impl Strategy<Value = IrNode> {
    let leaf = prop_oneof![
        8 => (any::<u8>(), props()).prop_map(|(k, p)| {
            let mut n = leaf(k);
            n.props = p;
            n
        }),
        1 => Just(list()),
    ];
    leaf.prop_recursive(3, 40, 4, |inner| {
        (
            proptest::collection::vec(inner, 0..5),
            any::<bool>(),
            props(),
        )
            .prop_flat_map(|(children, card, props)| {
                let n = children.len();
                (Just((children, card, props)), layout(n))
            })
            .prop_map(|((children, card, props), layout)| {
                let mut n = IrNode::new(id("x"));
                if card {
                    n.component = Some(Component::Card);
                }
                n.props = props;
                n.layout = layout;
                n.children = children;
                n
            })
    })
}

/// Gives every node of `n` a unique id: `n<i>` outside a template,
/// `t<i>[{item}]` inside one.
fn name(n: &mut IrNode, next: &mut usize, template: bool) {
    *next += 1;
    n.sem = if template {
        id(&format!("t{next}[{{item}}]"))
    } else {
        id(&format!("n{next}"))
    };
    for child in &mut n.children {
        name(child, next, template);
    }
    if let Some(repeat) = &mut n.repeat {
        name(&mut repeat.item, next, true);
    }
}

fn surface() -> impl Strategy<Value = Ir> {
    node().prop_map(|mut root| {
        let mut next = 0;
        name(&mut root, &mut next, false);
        // a root that is a leaf is wrapped: the root holds the surface
        let root = if root.children.is_empty() && root.repeat.is_none() {
            let mut wrap = IrNode::new(id("n0"));
            wrap.children.push(root);
            wrap
        } else {
            root
        };
        Ir::new(Surface {
            id: id("s"),
            title: "a surface".to_owned(),
            root,
        })
    })
}

/// An operation, by indices into what the surface holds when it applies.
#[derive(Clone, Debug)]
enum Pick {
    Hide(usize),
    Show(usize),
    Group(usize, usize, usize),
    Reorder(usize, usize),
    Emphasize(usize),
    Density(bool),
    Restyle(usize, usize),
    Swap(usize, usize),
    Promote(usize, usize),
}

fn pick() -> impl Strategy<Value = Pick> {
    let i = || 0usize..64;
    prop_oneof![
        i().prop_map(Pick::Hide),
        i().prop_map(Pick::Show),
        (i(), i(), 1usize..4).prop_map(|(a, b, n)| Pick::Group(a, b, n)),
        (i(), i()).prop_map(|(a, b)| Pick::Reorder(a, b)),
        i().prop_map(Pick::Emphasize),
        any::<bool>().prop_map(Pick::Density),
        (i(), i()).prop_map(|(a, b)| Pick::Restyle(a, b)),
        (i(), i()).prop_map(|(a, b)| Pick::Swap(a, b)),
        (i(), i()).prop_map(|(a, b)| Pick::Promote(a, b)),
    ]
}

/// The components that swap with one another, by family.
const FAMILIES: &[&[Component]] = &[
    &[Component::Text, Component::Stat, Component::Badge],
    &[Component::Checkbox, Component::Switch, Component::Chip],
    &[
        Component::Table,
        Component::Chart(ChartKind::Bar),
        Component::Chart(ChartKind::Line),
        Component::Chart(ChartKind::Area),
        Component::Chart(ChartKind::Donut),
    ],
];

/// The actions a promotion may name: declared or not, kept from agents,
/// destructive, shown by the random surfaces.
const PROMOTE: &[(&str, &str)] = &[
    ("commit.form", "cancel"),
    ("commit.submit", "submit"),
    ("trash", "empty"),
    ("admin", "reset"),
    ("changes.file[a.rs]", "discard"),
    ("changes.file[b.rs]", "unstage"),
    ("nope", "x"),
];

fn nodes(n: &IrNode) -> Vec<&IrNode> {
    let mut out = vec![n];
    for child in &n.children {
        out.extend(nodes(child));
    }
    if let Some(repeat) = &n.repeat {
        out.extend(nodes(&repeat.item));
    }
    out
}

/// The operation `p` names on `a`; the `k`th one keeps a group's id new.
fn resolve(a: &Adapted, p: &Pick, k: usize) -> Op {
    let all = nodes(&a.ir().surface.root);
    let at = |i: usize| all[i % all.len()];
    let aside: Vec<&SemId> = a.aside().map(|kept| &kept.node.sem).collect();
    match *p {
        Pick::Hide(i) => Op::Hide {
            id: at(i).sem.clone(),
        },
        Pick::Show(i) => match aside.get(i % aside.len().max(1)) {
            Some(&hidden) => Op::Show { id: hidden.clone() },
            None => Op::Show {
                id: at(i).sem.clone(),
            },
        },
        Pick::Group(i, j, n) => {
            let parent = at(i);
            let kids = &parent.children;
            let ids = if kids.is_empty() {
                vec![parent.sem.clone()]
            } else {
                (0..n)
                    .map(|s| kids[(j + s) % kids.len()].sem.clone())
                    .collect()
            };
            let into = if parent.sem.as_str().contains("{item}") {
                id(&format!("g{k}[{{item}}]"))
            } else {
                id(&format!("g{k}"))
            };
            Op::Group { ids, into }
        }
        Pick::Reorder(i, j) => {
            let parents: Vec<&IrNode> = all
                .iter()
                .copied()
                .filter(|n| n.children.len() > 1)
                .collect();
            let parent = parents
                .get(i % parents.len().max(1))
                .copied()
                .unwrap_or_else(|| at(i));
            let kids: Vec<SemId> = parent.children.iter().map(|c| c.sem.clone()).collect();
            let ids = if kids.is_empty() {
                vec![parent.sem.clone()]
            } else {
                vec![kids[j % kids.len()].clone()]
            };
            Op::Reorder {
                parent: parent.sem.clone(),
                ids,
            }
        }
        Pick::Emphasize(i) => Op::Emphasize {
            id: at(i).sem.clone(),
        },
        Pick::Density(sparser) => Op::Density {
            step: if sparser { Step::Sparser } else { Step::Denser },
        },
        Pick::Restyle(i, r) => Op::Restyle {
            id: at(i).sem.clone(),
            role: ColorRole::ALL[r % ColorRole::ALL.len()],
        },
        Pick::Swap(i, c) => {
            // mostly a node and a component of one family of swaps
            let shown: Vec<&IrNode> = all
                .iter()
                .copied()
                .filter(|n| {
                    n.component
                        .is_some_and(|c| FAMILIES.iter().any(|f| f.contains(&c)))
                })
                .collect();
            let node = shown
                .get(i % shown.len().max(1))
                .copied()
                .unwrap_or_else(|| at(i));
            let family = node
                .component
                .and_then(|now| FAMILIES.iter().find(|f| f.contains(&now)));
            let component = match family {
                Some(f) if c % 4 != 0 => f[c % f.len()],
                _ => Component::ALL[c % Component::ALL.len()],
            };
            Op::SwapView {
                id: node.sem.clone(),
                component,
            }
        }
        Pick::Promote(c, i) => {
            let (target, action) = PROMOTE[c % PROMOTE.len()];
            Op::Promote {
                target: id(target),
                action: action.parse().expect("a valid name"),
                into: at(i).sem.clone(),
            }
        }
    }
}

fn tree_ids(a: &Adapted) -> BTreeSet<SemId> {
    nodes(&a.ir().surface.root)
        .into_iter()
        .map(|n| n.sem.clone())
        .collect()
}

/// `op` on `before`, when it applies: the surface it makes verifies, no
/// id leaves the surface, only `Hide` takes ids out of the tree (the
/// hidden node's), and the inverse gives `before` byte for byte. The
/// surface and its inverse; `None` when refused.
fn checked(before: &Adapted, op: &Op, cx: Context<'_>) -> Option<(Adapted, Inverse)> {
    let (after, inverse) = match before.apply(op, &Actor::Human, cx) {
        Ok(done) => done,
        Err(TransformError::Refused(_) | TransformError::NotFound(_)) => return None,
        Err(TransformError::Instance { .. }) => panic!("{op}: the picks name no instance"),
        Err(TransformError::Unverified(errors)) => {
            assert!(!errors.is_empty(), "{op}");
            return None;
        }
    };
    assert!(verify(after.ir(), cx.caps, cx.policy).is_ok(), "{op}");
    let gone: Vec<_> = before.ids().difference(&after.ids()).cloned().collect();
    assert!(gone.is_empty(), "{op}: {gone:?} left the surface");
    let out: BTreeSet<SemId> = tree_ids(before)
        .difference(&tree_ids(&after))
        .cloned()
        .collect();
    match op {
        Op::Hide { id } => {
            let hidden = after
                .aside()
                .find(|kept| kept.node.sem == *id)
                .expect("the hidden node is aside");
            let held: BTreeSet<SemId> = nodes(&hidden.node)
                .into_iter()
                .map(|n| n.sem.clone())
                .collect();
            assert_eq!(out, held, "{op}");
        }
        _ => assert!(out.is_empty(), "{op}: {out:?} left the tree without a hide"),
    }
    let back = after
        .undo(&inverse, cx)
        .unwrap_or_else(|e| panic!("undo {op}: {e}"));
    assert_eq!(back, *before, "undo {op}");
    assert_eq!(
        serde_json::to_string(back.ir()).expect("an IR writes"),
        serde_json::to_string(before.ir()).expect("an IR writes"),
        "undo {op}: byte for byte"
    );
    Some((after, inverse))
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// Operations one after another, each checked, then undone in the
    /// reverse order: the surface of the start, byte for byte.
    #[test]
    fn every_operation_keeps_identity_and_its_inverse_goes_back(
        ir in surface(),
        picks in proptest::collection::vec(pick(), 1..8),
    ) {
        let caps = caps();
        let policy = GenerationPolicy::default();
        let cx = Context { caps: &caps, policy: &policy };
        let start = match verify(&ir, &caps, &policy) {
            Ok(v) => Adapted::new(v),
            Err(errors) => panic!("a surface built to verify does not: {errors:#?}"),
        };
        let mut now = start.clone();
        let mut undo = Vec::new();
        for (k, p) in picks.iter().enumerate() {
            let op = resolve(&now, p, k);
            if let Some((after, inverse)) = checked(&now, &op, cx) {
                undo.push(inverse);
                now = after;
            }
        }
        while let Some(inverse) = undo.pop() {
            now = now.undo(&inverse, cx).unwrap_or_else(|e| panic!("undo: {e}"));
        }
        prop_assert_eq!(&now, &start);
        prop_assert_eq!(
            serde_json::to_string(now.ir()).expect("an IR writes"),
            serde_json::to_string(start.ir()).expect("an IR writes")
        );
    }
}

fn path(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn read(rel: &str) -> String {
    let at = path(rel);
    std::fs::read_to_string(&at).unwrap_or_else(|e| panic!("{}: {e}", at.display()))
}

/// Every operation on every node of `ir_rel`, verified against the
/// registry of `snap_rel`: each one that applies checked, and each kind
/// applied at least once where it can.
fn every_operation(ir_rel: &str, snap_rel: &str) -> BTreeSet<&'static str> {
    let snap: Snapshot = serde_json::from_str(&read(snap_rel)).expect("a sem snapshot");
    let caps = Capabilities::from_snapshot(&snap);
    let policy = GenerationPolicy::default();
    let cx = Context {
        caps: &caps,
        policy: &policy,
    };
    let ir = parse(&read(ir_rel)).unwrap_or_else(|e| panic!("{ir_rel}: {e:#?}"));
    let start = Adapted::new(verify(&ir, &caps, &policy).unwrap_or_else(|e| panic!("{e:#?}")));
    let mut applied = BTreeSet::new();
    let mut note = |op: &Op| {
        let kind = match op {
            Op::Hide { .. } => "hide",
            Op::Show { .. } => "show",
            Op::Group { .. } => "group",
            Op::Reorder { .. } => "reorder",
            Op::Emphasize { .. } => "emphasize",
            Op::Density { .. } => "density",
            Op::Restyle { .. } => "restyle",
            Op::SwapView { .. } => "swap_view",
            Op::Promote { .. } => "promote",
        };
        applied.insert(kind);
    };
    let all: Vec<IrNode> = nodes(&start.ir().surface.root)
        .into_iter()
        .cloned()
        .collect();
    let mut ops = vec![
        Op::Density { step: Step::Denser },
        Op::Density {
            step: Step::Sparser,
        },
    ];
    for n in &all {
        let sem = n.sem.clone();
        ops.push(Op::Hide { id: sem.clone() });
        ops.push(Op::Emphasize { id: sem.clone() });
        for &role in ColorRole::ALL {
            ops.push(Op::Restyle {
                id: sem.clone(),
                role,
            });
        }
        for component in Component::ALL {
            ops.push(Op::SwapView {
                id: sem.clone(),
                component,
            });
        }
        if let Some(last) = n.children.last() {
            ops.push(Op::Reorder {
                parent: sem.clone(),
                ids: vec![last.sem.clone()],
            });
        }
        for pair in n.children.windows(2) {
            let into = if sem.as_str().contains("{item}") {
                id("grouped[{item}]")
            } else {
                id("grouped")
            };
            ops.push(Op::Group {
                ids: pair.iter().map(|c| c.sem.clone()).collect(),
                into,
            });
        }
        for declared in caps.actions() {
            ops.push(Op::Promote {
                target: declared.target.clone(),
                action: declared.action.clone(),
                into: sem.clone(),
            });
        }
    }
    for op in &ops {
        let Some((after, _)) = checked(&start, op, cx) else {
            continue;
        };
        note(op);
        if let Op::Hide { id } = op {
            let show = Op::Show { id: id.clone() };
            if checked(&after, &show, cx).is_some() {
                note(&show);
            }
        }
    }
    applied
}

#[test]
fn every_operation_on_every_node_of_the_fixtures_keeps_identity() {
    const ALL: &[&str] = &[
        "density",
        "emphasize",
        "group",
        "hide",
        "promote",
        "reorder",
        "restyle",
        "show",
        "swap_view",
    ];
    let but = |left: &[&str]| -> BTreeSet<&'static str> {
        ALL.iter().copied().filter(|k| !left.contains(k)).collect()
    };
    for (ir, snap, kinds) in [
        // a form: nothing a family of swaps holds
        (
            "fixtures/ide-commit-form.ir.json",
            "fixtures/ide-workbench.snapshot.json",
            but(&["swap_view"]),
        ),
        (
            "fixtures/ide-review-commit.ir.json",
            "fixtures/ide-workbench.snapshot.json",
            but(&[]),
        ),
        // the todo surface shows every action the todo declares
        (
            "fixtures/showcase-todo.ir.json",
            "fixtures/showcase-todo.snapshot.json",
            but(&["promote"]),
        ),
        (
            "../showcase/fixtures/ir/silences-review-claude.ir.json",
            "fixtures/showcase-silences.snapshot.json",
            but(&[]),
        ),
    ] {
        assert_eq!(every_operation(ir, snap), kinds, "{ir}");
    }
}
