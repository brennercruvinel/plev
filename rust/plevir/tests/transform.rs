//! The nine operations of `uir::transform`, one by one, on a surface over
//! the registry of `support::caps()`: what each changes, that the result
//! passes the verifier, that its inverse gives the surface before byte for
//! byte, and where each is refused or fails the verifier.

// shared with the other uir tests: this binary uses the registry only.
#[allow(dead_code)]
mod support;

use sem::{Actor, SemId, ValueKind};
use serde_json::{Value, json};
use support::caps;
use uir::{
    Adapted, Capabilities, ChartKind, ColorRole, Component, Context, ControlSize, DataCap,
    Elevation, GenerationPolicy, Inverse, Ir, IrNode, Layout, Op, Rule, SpacingToken, Step,
    TransformError, TypographyRole, verify,
};

/// A surface over `caps()`: a title, the files repeated with a status and
/// a stage button, a split of the message and the count, a row with the
/// commit, and a table of the files.
fn doc() -> Value {
    json!({
        "plev_ir": 1,
        "surface": {
            "id": "review",
            "title": "review the commit",
            "root": {
                "sem": "review.root",
                "layout": { "stack": { "axis": "vertical", "gap": "md" } },
                "children": [
                    { "sem": "review.title", "component": "text", "label": "the commit" },
                    {
                        "sem": "review.files",
                        "component": "list",
                        "label": "files",
                        "layout": { "stack": { "axis": "vertical", "gap": "xs" } },
                        "repeat": {
                            "each": "changes:files",
                            "item": {
                                "sem": "review.file[{item}]",
                                "layout": { "stack": { "axis": "horizontal", "gap": "sm" } },
                                "children": [
                                    {
                                        "sem": "review.file[{item}].status",
                                        "component": "badge",
                                        "bind": { "state": { "text": "changes.file[{item}]:status" } }
                                    },
                                    {
                                        "sem": "review.file[{item}].staged",
                                        "component": "checkbox",
                                        "label": "staged",
                                        "bind": {
                                            "state": { "checked": "changes.file[{item}]:staged" },
                                            "actions": { "toggle": {
                                                "target": "changes.file[{item}]", "action": "stage"
                                            } }
                                        }
                                    }
                                ]
                            }
                        }
                    },
                    {
                        "sem": "review.split",
                        "layout": { "split": { "axis": "horizontal", "ratio": 60 } },
                        "children": [
                            {
                                "sem": "review.message",
                                "component": "text_field",
                                "label": "commit message",
                                "bind": {
                                    "state": { "text": "commit.message:text" },
                                    "actions": { "set_value": {
                                        "target": "commit.message", "action": "set_value"
                                    } }
                                }
                            },
                            {
                                "sem": "review.count",
                                "component": "text",
                                "label": "files",
                                "bind": { "state": { "text": "counter:count" } }
                            }
                        ]
                    },
                    {
                        "sem": "review.row",
                        "props": { "padding": "sm" },
                        "layout": { "stack": { "axis": "horizontal", "gap": "sm", "align": "center" } },
                        "children": [
                            {
                                "sem": "review.commit",
                                "component": "button",
                                "label": "commit",
                                "bind": { "actions": { "activate": {
                                    "target": "commit.submit", "action": "submit"
                                } } }
                            }
                        ]
                    },
                    {
                        "sem": "review.table",
                        "component": "table",
                        "label": "the files",
                        "bind": { "state": { "rows": "changes:files" } }
                    }
                ]
            }
        }
    })
}

fn id(text: &str) -> SemId {
    SemId::parse(text).unwrap()
}

fn human() -> Actor {
    Actor::Human
}

fn agent() -> Actor {
    Actor::Agent {
        name: "claude".to_owned(),
    }
}

/// `doc` read, verified against `caps` under `policy`, nothing aside.
fn adapted_over(doc: &Value, caps: &Capabilities, policy: &GenerationPolicy) -> Adapted {
    let ir: Ir = serde_json::from_value(doc.clone()).expect("the document reads");
    Adapted::new(verify(&ir, caps, policy).unwrap_or_else(|e| panic!("{e:#?}")))
}

fn adapted() -> Adapted {
    adapted_over(&doc(), &caps(), &GenerationPolicy::default())
}

/// `op` by `by` on `before`, with its inverse checked: the result is
/// verified, undoing it gives `before` byte for byte, nothing aside
/// changed but what `op` set aside or showed, and no id of `before` left
/// the surface (in its tree or aside).
#[track_caller]
fn apply_by(before: &Adapted, op: &Op, by: &Actor, cx: Context<'_>) -> Adapted {
    let (after, inverse) = before
        .apply(op, by, cx)
        .unwrap_or_else(|e| panic!("{op}: {e}"));
    assert!(verify(after.ir(), cx.caps, cx.policy).is_ok(), "{op}");
    let back = after
        .undo(&inverse, cx)
        .unwrap_or_else(|e| panic!("undo {op}: {e}"));
    assert_eq!(back, *before, "undo {op}");
    assert_eq!(
        serde_json::to_string(back.ir()).unwrap(),
        serde_json::to_string(before.ir()).unwrap(),
        "undo {op}: byte for byte"
    );
    let kept = after.ids();
    let gone: Vec<SemId> = before.ids().difference(&kept).cloned().collect();
    assert!(gone.is_empty(), "{op}: {gone:?} left the surface");
    after
}

#[track_caller]
fn apply(before: &Adapted, op: &Op) -> Adapted {
    let (caps, policy) = (caps(), GenerationPolicy::default());
    let cx = Context {
        caps: &caps,
        policy: &policy,
    };
    apply_by(before, op, &human(), cx)
}

/// `op` on `before`, refused: the error.
#[track_caller]
fn refused(before: &Adapted, op: &Op) -> TransformError {
    let (caps, policy) = (caps(), GenerationPolicy::default());
    let cx = Context {
        caps: &caps,
        policy: &policy,
    };
    match before.apply(op, &human(), cx) {
        Ok(_) => panic!("{op} applies"),
        Err(e) => e,
    }
}

/// `op` on `before` fails the verifier with an error of `rule`.
#[track_caller]
fn unverified(before: &Adapted, op: &Op, rule: Rule) {
    match refused(before, op) {
        TransformError::Unverified(errors) => {
            assert!(errors.iter().any(|e| e.rule == rule), "{op}: {errors:#?}");
        }
        other => panic!("{op}: {other}"),
    }
}

/// The children of `parent`, by id.
fn children(a: &Adapted, parent: &str) -> Vec<String> {
    node(a, parent)
        .children
        .iter()
        .map(|c| c.sem.to_string())
        .collect()
}

fn node<'a>(a: &'a Adapted, at: &str) -> &'a IrNode {
    fn find<'a>(n: &'a IrNode, at: &str) -> Option<&'a IrNode> {
        if n.sem.as_str() == at {
            return Some(n);
        }
        n.children
            .iter()
            .find_map(|c| find(c, at))
            .or_else(|| n.repeat.as_ref().and_then(|r| find(&r.item, at)))
    }
    find(&a.ir().surface.root, at).unwrap_or_else(|| panic!("no node {at}"))
}

#[test]
fn hide_takes_a_node_out_of_the_tree_and_keeps_it_aside() {
    let before = adapted();
    let after = apply(
        &before,
        &Op::Hide {
            id: id("review.row"),
        },
    );
    assert_eq!(
        children(&after, "review.root"),
        [
            "review.title",
            "review.files",
            "review.split",
            "review.table"
        ]
    );
    let kept: Vec<_> = after.aside().collect();
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].node.sem, id("review.row"));
    assert_eq!((kept[0].parent.as_str(), kept[0].at), ("review.root", 3));
    // the focus order follows: the commit is no stop any more
    assert!(
        !after
            .verified()
            .focus_order()
            .contains(&id("review.commit"))
    );
    // a template's child: hidden for every item
    let after = apply(
        &before,
        &Op::Hide {
            id: id("review.file[{item}].status"),
        },
    );
    assert_eq!(
        children(&after, "review.file[{item}]"),
        ["review.file[{item}].staged"]
    );
}

#[test]
fn hide_is_refused_for_the_root_an_item_an_instance_and_what_is_hidden() {
    let before = adapted();
    let why = refused(
        &before,
        &Op::Hide {
            id: id("review.root"),
        },
    )
    .to_string();
    assert!(why.contains("the root"), "{why}");
    let why = refused(
        &before,
        &Op::Hide {
            id: id("review.file[{item}]"),
        },
    )
    .to_string();
    assert!(
        why.contains("item of the repeat of `review.files`"),
        "{why}"
    );
    assert_eq!(
        refused(
            &before,
            &Op::Hide {
                id: id("review.file[a.rs].staged")
            }
        ),
        TransformError::Instance {
            id: id("review.file[a.rs].staged"),
            template: id("review.file[{item}].staged"),
        }
    );
    assert_eq!(
        refused(&before, &Op::Hide { id: id("nope") }),
        TransformError::NotFound(id("nope"))
    );
    let hidden = apply(
        &before,
        &Op::Hide {
            id: id("review.row"),
        },
    );
    let why = refused(
        &hidden,
        &Op::Hide {
            id: id("review.row"),
        },
    )
    .to_string();
    assert!(why.contains("hidden already"), "{why}");
    let why = refused(
        &hidden,
        &Op::Hide {
            id: id("review.commit"),
        },
    )
    .to_string();
    assert!(why.contains("show `review.row` first"), "{why}");
    // a split holds exactly two children: the verifier refuses one
    unverified(
        &before,
        &Op::Hide {
            id: id("review.count"),
        },
        Rule::Vocabulary,
    );
}

#[test]
fn show_puts_a_hidden_node_back_where_it_was() {
    let before = adapted();
    let hidden = apply(
        &before,
        &Op::Hide {
            id: id("review.row"),
        },
    );
    let shown = apply(
        &hidden,
        &Op::Show {
            id: id("review.row"),
        },
    );
    assert_eq!(shown, before);
    assert_eq!(shown.aside().count(), 0);
    // nested: the child waits for its parent
    let inner = apply(
        &before,
        &Op::Hide {
            id: id("review.commit"),
        },
    );
    let both = apply(
        &inner,
        &Op::Hide {
            id: id("review.row"),
        },
    );
    let why = refused(
        &both,
        &Op::Show {
            id: id("review.commit"),
        },
    )
    .to_string();
    assert!(why.contains("its place `review.row` is hidden"), "{why}");
    let row = apply(
        &both,
        &Op::Show {
            id: id("review.row"),
        },
    );
    assert!(children(&row, "review.row").is_empty());
    let all = apply(
        &row,
        &Op::Show {
            id: id("review.commit"),
        },
    );
    assert_eq!(all, before);
    let why = refused(
        &before,
        &Op::Show {
            id: id("review.row"),
        },
    )
    .to_string();
    assert!(why.contains("not hidden"), "{why}");
}

#[test]
fn show_at_a_place_the_parent_no_longer_has_goes_last_and_undoes_exactly() {
    let before = adapted();
    let a = apply(
        &before,
        &Op::Hide {
            id: id("review.table"),
        },
    );
    let b = apply(
        &a,
        &Op::Hide {
            id: id("review.row"),
        },
    );
    // the table was fifth; the root has three children now
    let c = apply(
        &b,
        &Op::Show {
            id: id("review.table"),
        },
    );
    assert_eq!(
        children(&c, "review.root"),
        [
            "review.title",
            "review.files",
            "review.split",
            "review.table"
        ]
    );
    let kept = c.aside().next().expect("the row is still aside");
    assert_eq!(kept.at, 3);
}

#[test]
fn group_puts_siblings_under_a_new_container_in_the_place_of_the_first() {
    let before = adapted();
    let after = apply(
        &before,
        &Op::Group {
            ids: vec![id("review.table"), id("review.title")],
            into: id("review.head"),
        },
    );
    assert_eq!(
        children(&after, "review.root"),
        ["review.head", "review.files", "review.split", "review.row"]
    );
    // in the order they were, laid out as their parent lays them out
    assert_eq!(
        children(&after, "review.head"),
        ["review.title", "review.table"]
    );
    let head = node(&after, "review.head");
    assert_eq!(head.layout, node(&before, "review.root").layout);
    assert_eq!(head.component, None);
    // inside a template the group carries the item
    let after = apply(
        &before,
        &Op::Group {
            ids: vec![
                id("review.file[{item}].status"),
                id("review.file[{item}].staged"),
            ],
            into: id("review.file[{item}].cells"),
        },
    );
    assert_eq!(
        children(&after, "review.file[{item}]"),
        ["review.file[{item}].cells"]
    );
}

#[test]
fn group_is_refused_across_parents_on_a_taken_id_and_fails_what_the_verifier_refuses() {
    let before = adapted();
    let why = refused(
        &before,
        &Op::Group {
            ids: vec![id("review.title"), id("review.commit")],
            into: id("review.mixed"),
        },
    )
    .to_string();
    assert!(why.contains("not a sibling"), "{why}");
    let why = refused(
        &before,
        &Op::Group {
            ids: vec![id("review.title")],
            into: id("review.table"),
        },
    )
    .to_string();
    assert!(why.contains("already a node"), "{why}");
    let why = refused(
        &before,
        &Op::Group {
            ids: vec![id("review.title"), id("review.title")],
            into: id("review.g"),
        },
    )
    .to_string();
    assert!(why.contains("named twice"), "{why}");
    // a hidden node's id is taken too
    let hidden = apply(
        &before,
        &Op::Hide {
            id: id("review.row"),
        },
    );
    let why = refused(
        &hidden,
        &Op::Group {
            ids: vec![id("review.title")],
            into: id("review.commit"),
        },
    )
    .to_string();
    assert!(why.contains("hidden in `review.row`"), "{why}");
    // the verifier: a template's group without the item, and a split
    // left with one child
    unverified(
        &before,
        &Op::Group {
            ids: vec![id("review.file[{item}].status")],
            into: id("review.cells"),
        },
        Rule::Identity,
    );
    unverified(
        &before,
        &Op::Group {
            ids: vec![id("review.message"), id("review.count")],
            into: id("review.pane"),
        },
        Rule::Vocabulary,
    );
}

#[test]
fn reorder_puts_the_named_children_first_and_keeps_the_rest_in_order() {
    let before = adapted();
    let after = apply(
        &before,
        &Op::Reorder {
            parent: id("review.root"),
            ids: vec![id("review.row"), id("review.files")],
        },
    );
    assert_eq!(
        children(&after, "review.root"),
        [
            "review.row",
            "review.files",
            "review.title",
            "review.split",
            "review.table"
        ]
    );
    // the focus order is the order a person sees
    assert_eq!(
        after.verified().focus_order()[0],
        id("review.commit"),
        "the commit is first now"
    );
    let why = refused(
        &before,
        &Op::Reorder {
            parent: id("review.root"),
            ids: vec![id("review.title")],
        },
    )
    .to_string();
    assert!(why.contains("in that order already"), "{why}");
    let why = refused(
        &before,
        &Op::Reorder {
            parent: id("review.files"),
            ids: vec![id("review.file[{item}]")],
        },
    )
    .to_string();
    assert!(why.contains("the order of `changes:files`"), "{why}");
    let why = refused(
        &before,
        &Op::Reorder {
            parent: id("review.root"),
            ids: vec![id("review.commit")],
        },
    )
    .to_string();
    assert!(why.contains("not a child"), "{why}");
}

#[test]
fn an_op_is_data_in_json() {
    let op = Op::Hide {
        id: id("review.controls"),
    };
    let text = serde_json::to_string(&op).unwrap();
    assert_eq!(text, r#"{"op":"hide","id":"review.controls"}"#);
    let group: Op = serde_json::from_value(json!({
        "op": "group", "ids": ["a", "b"], "into": "c"
    }))
    .unwrap();
    assert_eq!(
        group,
        Op::Group {
            ids: vec![id("a"), id("b")],
            into: id("c"),
        }
    );
    for bad in [
        json!({ "op": "hide" }),
        json!({ "op": "hide", "id": "A" }),
        json!({ "op": "hide", "id": "a", "colour": "red" }),
        json!({ "op": "delete", "id": "a" }),
    ] {
        assert!(serde_json::from_value::<Op>(bad.clone()).is_err(), "{bad}");
    }
    assert_eq!(group.to_string(), "group a, b into c");
}

#[test]
fn emphasize_takes_one_step_up_the_token_the_node_is_drawn_by() {
    let before = adapted();
    // a text with no style is body (16): one size up the ramp is title
    // (20), then headline, h4, and no further
    let mut text = apply(
        &before,
        &Op::Emphasize {
            id: id("review.title"),
        },
    );
    assert_eq!(
        node(&text, "review.title").props.typography,
        Some(TypographyRole::Title)
    );
    for next in [TypographyRole::Headline, TypographyRole::H4] {
        text = apply(
            &text,
            &Op::Emphasize {
                id: id("review.title"),
            },
        );
        assert_eq!(node(&text, "review.title").props.typography, Some(next));
    }
    let why = refused(
        &text,
        &Op::Emphasize {
            id: id("review.title"),
        },
    )
    .to_string();
    assert!(why.contains("as emphasized as a text gets"), "{why}");
    // a button: a control size up, `md` from none
    let button = apply(
        &before,
        &Op::Emphasize {
            id: id("review.commit"),
        },
    );
    assert_eq!(
        node(&button, "review.commit").props.control_size,
        Some(ControlSize::Md)
    );
    // a container, a list: an elevation up, `raised` from none
    let row = apply(
        &before,
        &Op::Emphasize {
            id: id("review.row"),
        },
    );
    assert_eq!(
        node(&row, "review.row").props.elevation,
        Some(Elevation::Raised)
    );
    assert_eq!(
        node(&row, "review.row").props.padding,
        Some(SpacingToken::Sm)
    );
    // a table draws no token of emphasis
    let why = refused(
        &before,
        &Op::Emphasize {
            id: id("review.table"),
        },
    )
    .to_string();
    assert!(why.contains("a table draws no token of emphasis"), "{why}");
}

#[test]
fn density_moves_every_gap_and_padding_set_one_step() {
    let before = adapted();
    let sparse = apply(
        &before,
        &Op::Density {
            step: Step::Sparser,
        },
    );
    let gap = |a: &Adapted, at: &str| match node(a, at).layout {
        Some(Layout::Stack { gap, .. }) => gap,
        other => panic!("{at}: {other:?}"),
    };
    assert_eq!(gap(&sparse, "review.root"), Some(SpacingToken::Lg));
    assert_eq!(gap(&sparse, "review.files"), Some(SpacingToken::Sm));
    assert_eq!(gap(&sparse, "review.file[{item}]"), Some(SpacingToken::Md));
    assert_eq!(
        node(&sparse, "review.row").props.padding,
        Some(SpacingToken::Md)
    );
    // the split keeps its ratio, the rest keeps what it had
    assert_eq!(
        node(&sparse, "review.split").layout,
        node(&before, "review.split").layout
    );
    let dense = apply(&before, &Op::Density { step: Step::Denser });
    assert_eq!(gap(&dense, "review.root"), Some(SpacingToken::Sm));
    // `xs` holds at the end of the scale
    assert_eq!(gap(&dense, "review.files"), Some(SpacingToken::Xs));
    // nothing set, nothing to move
    let bare = adapted_over(
        &json!({ "plev_ir": 1, "surface": { "id": "s", "title": "t",
            "root": { "sem": "s.root", "children": [
                { "sem": "s.text", "component": "text", "label": "hello" } ] } } }),
        &caps(),
        &GenerationPolicy::default(),
    );
    let why = refused(&bare, &Op::Density { step: Step::Denser }).to_string();
    assert!(why.contains("nothing to move"), "{why}");
}

#[test]
fn restyle_sets_a_text_s_color_or_a_container_s_fill_by_role() {
    let before = adapted();
    let text = apply(
        &before,
        &Op::Restyle {
            id: id("review.title"),
            role: ColorRole::Accent,
        },
    );
    assert_eq!(
        node(&text, "review.title").props.color,
        Some(ColorRole::Accent)
    );
    let row = apply(
        &before,
        &Op::Restyle {
            id: id("review.row"),
            role: ColorRole::BgPanel,
        },
    );
    assert_eq!(
        node(&row, "review.row").props.background,
        Some(ColorRole::BgPanel)
    );
    for (at, role, says) in [
        ("review.title", ColorRole::Bg, "a text's color is one of"),
        (
            "review.row",
            ColorRole::Danger,
            "a container's fill is one of",
        ),
        ("review.commit", ColorRole::Accent, "theme's own colors"),
    ] {
        let why = refused(&before, &Op::Restyle { id: id(at), role }).to_string();
        assert!(why.contains(says), "{at}: {why}");
    }
    let why = refused(
        &text,
        &Op::Restyle {
            id: id("review.title"),
            role: ColorRole::Accent,
        },
    )
    .to_string();
    assert!(why.contains("already"), "{why}");
}

/// `caps()` with a number to show and an action kept from agents.
fn more_caps() -> Capabilities {
    caps().with_data(DataCap {
        at: "meter:level".parse().unwrap(),
        kind: ValueKind::Num,
        label: "level".to_owned(),
    })
}

#[test]
fn swap_view_shows_the_same_datum_by_another_component_of_its_family() {
    let before = adapted();
    let chart = apply(
        &before,
        &Op::SwapView {
            id: id("review.table"),
            component: Component::Chart(ChartKind::Bar),
        },
    );
    let table = node(&chart, "review.table");
    assert_eq!(table.component, Some(Component::Chart(ChartKind::Bar)));
    assert_eq!(
        table.bind.state.get("data").map(ToString::to_string),
        Some("changes:files".to_owned())
    );
    assert!(!table.bind.state.contains_key("rows"));
    let line = apply(
        &chart,
        &Op::SwapView {
            id: id("review.table"),
            component: Component::Chart(ChartKind::Line),
        },
    );
    assert_eq!(
        node(&line, "review.table").component,
        Some(Component::Chart(ChartKind::Line))
    );
    // a text of a count as a stat and as a badge
    for to in [Component::Stat, Component::Badge] {
        let swapped = apply(
            &before,
            &Op::SwapView {
                id: id("review.count"),
                component: to,
            },
        );
        assert_eq!(node(&swapped, "review.count").component, Some(to));
    }
    // the checkbox of every file as a switch, and as a chip that toggles
    let switch = apply(
        &before,
        &Op::SwapView {
            id: id("review.file[{item}].staged"),
            component: Component::Switch,
        },
    );
    let staged = node(&switch, "review.file[{item}].staged");
    assert!(staged.bind.state.contains_key("on"));
    assert!(staged.bind.actions.contains_key("toggle"));
    apply(
        &switch,
        &Op::SwapView {
            id: id("review.file[{item}].staged"),
            component: Component::Chip,
        },
    );
}

#[test]
fn swap_view_is_refused_across_families_and_for_a_list_and_fails_what_the_verifier_refuses() {
    let before = adapted();
    for (at, to, says) in [
        (
            "review.files",
            Component::Table,
            "ids of its items would leave the surface",
        ),
        (
            "review.commit",
            Component::Chip,
            "no chip shows what a button shows",
        ),
        ("review.row", Component::Card, "layout container"),
        ("review.title", Component::Text, "a text already"),
    ] {
        let why = refused(
            &before,
            &Op::SwapView {
                id: id(at),
                component: to,
            },
        )
        .to_string();
        assert!(why.contains(says), "{at}: {why}");
    }
    // a chart is always named; a table of a set is no chart's data
    let unnamed = support::without(doc(), "/surface/root/children/4/label");
    let unnamed = adapted_over(&unnamed, &caps(), &GenerationPolicy::default());
    unverified(
        &unnamed,
        &Op::SwapView {
            id: id("review.table"),
            component: Component::Chart(ChartKind::Bar),
        },
        Rule::Access,
    );
    let set = support::with(
        doc(),
        "/surface/root/children/4/bind/state/rows",
        json!("changes:staged"),
    );
    let set = adapted_over(&set, &caps(), &GenerationPolicy::default());
    unverified(
        &set,
        &Op::SwapView {
            id: id("review.table"),
            component: Component::Chart(ChartKind::Donut),
        },
        Rule::State,
    );
    // a stat of a number is no badge's text
    let level = support::with(
        doc(),
        "/surface/root/children/2/children/1/bind/state/text",
        json!("meter:level"),
    );
    let (caps, policy) = (more_caps(), GenerationPolicy::default());
    let level = adapted_over(&level, &caps, &policy);
    let cx = Context {
        caps: &caps,
        policy: &policy,
    };
    let stat = apply_by(
        &level,
        &Op::SwapView {
            id: id("review.count"),
            component: Component::Stat,
        },
        &human(),
        cx,
    );
    let badge = Op::SwapView {
        id: id("review.count"),
        component: Component::Badge,
    };
    match stat.apply(&badge, &human(), cx) {
        Err(TransformError::Unverified(errors)) => {
            assert!(errors.iter().any(|e| e.rule == Rule::State), "{errors:#?}");
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn promote_adds_a_button_for_an_action_the_surface_does_not_show() {
    let before = adapted();
    let after = apply(
        &before,
        &Op::Promote {
            target: id("commit.form"),
            action: "cancel".parse().unwrap(),
            into: id("review.row"),
        },
    );
    assert_eq!(
        children(&after, "review.row"),
        ["review.commit", "review.row.cancel"]
    );
    let button = node(&after, "review.row.cancel");
    assert_eq!(button.component, Some(Component::Button));
    assert_eq!(button.label.as_deref(), Some("commit"), "the app's label");
    assert!(
        after
            .verified()
            .focus_order()
            .contains(&id("review.row.cancel"))
    );
    // an action the app keeps from agents, confirmed always, destructive
    let empty = apply(
        &before,
        &Op::Promote {
            target: id("trash"),
            action: "empty".parse().unwrap(),
            into: id("review.root"),
        },
    );
    assert!(empty.ids().contains(&id("review.root.empty")));
}

#[test]
fn promote_is_refused_for_what_is_shown_undeclared_or_takes_a_value() {
    let before = adapted();
    let promote = |target: &str, action: &str, into: &str| Op::Promote {
        target: id(target),
        action: action.parse().unwrap(),
        into: id(into),
    };
    for (op, says) in [
        (
            promote("commit.submit", "submit", "review.root"),
            "`review.commit` shows `submit`",
        ),
        (
            promote("changes.file[a.rs]", "stage", "review.root"),
            "`review.file[{item}].staged` shows `stage`",
        ),
        (
            promote("commit.submit", "push", "review.root"),
            "declares no `push`",
        ),
        (
            promote("commit.message", "set_value", "review.root"),
            "takes a text",
        ),
        (
            promote("commit.form", "cancel", "review.commit"),
            "holds no children",
        ),
        (
            promote("commit.form", "cancel", "review.files"),
            "holds no children",
        ),
    ] {
        let why = refused(&before, &op).to_string();
        assert!(why.contains(says), "{op}: {why}");
    }
    // hidden: then it is a `Show`
    let hidden = apply(
        &before,
        &Op::Hide {
            id: id("review.row"),
        },
    );
    let why = refused(&hidden, &promote("commit.submit", "submit", "review.root")).to_string();
    assert!(why.contains("hidden `review.row`: show it"), "{why}");
    // destructive, confirmed for agents only: the verifier refuses the
    // button
    unverified(
        &before,
        &promote("changes.file[a.rs]", "discard", "review.root"),
        Rule::Policy,
    );
}

/// An action kept from agents (`admin reset`, `caps()`): promoted for a
/// person on a surface a person wrote, never for an agent, and whatever
/// binds it keeps binding it through a group, a reorder and a swap.
#[test]
fn an_action_kept_from_agents_stays_kept_and_is_promoted_only_for_whoever_may_ask() {
    let caps = caps();
    let by_human = GenerationPolicy::by(human());
    let cx = Context {
        caps: &caps,
        policy: &by_human,
    };
    let before = adapted_over(&doc(), &caps, &by_human);
    let reset = Op::Promote {
        target: id("admin"),
        action: "reset".parse().unwrap(),
        into: id("review.row"),
    };
    let why = before
        .apply(&reset, &agent(), cx)
        .err()
        .map(|e| e.to_string());
    assert!(
        why.as_deref()
            .is_some_and(|w| w.contains("keeps `reset` on `admin` from agent:claude")),
        "{why:?}"
    );
    let promoted = apply_by(&before, &reset, &human(), cx);
    // written by an agent, the surface may not bind it, whoever asks
    let by_agent = GenerationPolicy::default();
    let agents = adapted_over(&doc(), &caps, &by_agent);
    let refused = agents.apply(
        &reset,
        &human(),
        Context {
            caps: &caps,
            policy: &by_agent,
        },
    );
    assert!(
        matches!(&refused, Err(TransformError::Unverified(e)) if e.iter().any(|e| e.rule == Rule::Policy)),
        "{refused:?}"
    );
    // through the other operations the node binds the same capability
    let cap = |a: &Adapted| {
        node(a, "review.row.reset")
            .bind
            .actions
            .get("activate")
            .map(|c| (c.target.to_string(), c.action.to_string()))
    };
    let admin = Some(("admin".to_owned(), "reset".to_owned()));
    let grouped = apply_by(
        &promoted,
        &Op::Group {
            ids: vec![id("review.row.reset")],
            into: id("review.row.danger"),
        },
        &human(),
        cx,
    );
    let reordered = apply_by(
        &grouped,
        &Op::Reorder {
            parent: id("review.row"),
            ids: vec![id("review.row.danger")],
        },
        &human(),
        cx,
    );
    for a in [&promoted, &grouped, &reordered] {
        assert_eq!(cap(a), admin);
    }
    assert_eq!(
        caps.action(&id("admin"), &"reset".parse().unwrap())
            .map(|c| c.policy.actors.agent),
        Some(false)
    );
}

#[test]
fn an_inverse_is_data_too() {
    let before = adapted();
    let (caps, policy) = (caps(), GenerationPolicy::default());
    let cx = Context {
        caps: &caps,
        policy: &policy,
    };
    let (_, inverse) = before
        .apply(
            &Op::Hide {
                id: id("review.row"),
            },
            &human(),
            cx,
        )
        .unwrap();
    assert_eq!(
        inverse,
        Inverse::Op(Op::Show {
            id: id("review.row")
        })
    );
    assert_eq!(
        serde_json::to_value(&inverse).unwrap(),
        json!({ "op": { "op": "show", "id": "review.row" } })
    );
    let (_, looks) = before
        .apply(
            &Op::Emphasize {
                id: id("review.title"),
            },
            &human(),
            cx,
        )
        .unwrap();
    assert_eq!(
        serde_json::to_value(&looks).unwrap(),
        json!({ "looks": [{ "id": "review.title" }] })
    );
}

/// A promote on the template of an item (task-70 phase 4): the action is
/// found on the items the template stands for, the button goes into the
/// item's template, one per item, each bound to its own item; an action
/// no item declares is refused before, and one an item keeps from
/// agents is refused for them.
#[test]
fn promote_on_the_template_of_an_item_finds_the_action_on_its_items() {
    let before = adapted();
    let promote = |action: &str| Op::Promote {
        target: id("changes.file[{item}]"),
        action: action.parse().unwrap(),
        into: id("review.file[{item}]"),
    };
    let after = apply(&before, &promote("unstage"));
    assert_eq!(
        children(&after, "review.file[{item}]"),
        [
            "review.file[{item}].status",
            "review.file[{item}].staged",
            "review.file[{item}].unstage"
        ]
    );
    let bound = node(&after, "review.file[{item}].unstage")
        .bind
        .actions
        .get("activate")
        .map(|c| (c.target.to_string(), c.action.to_string()));
    assert_eq!(
        bound,
        Some(("changes.file[{item}]".to_owned(), "unstage".to_owned()))
    );
    assert_eq!(
        node(&after, "review.file[{item}].unstage").label.as_deref(),
        Some("a.rs"),
        "the label of the first item that declares it"
    );
    let why = refused(&before, &promote("push")).to_string();
    assert!(
        why.contains("declares no `push` on `changes.file[{item}]`"),
        "{why}"
    );
    // destructive and confirmed for agents only: the verifier refuses it
    // on the template as on one item
    unverified(&before, &promote("discard"), Rule::Policy);
}
