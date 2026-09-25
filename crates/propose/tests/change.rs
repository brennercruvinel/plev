//! The rules as a transformer (task-70 phase 2): a change of the surface
//! shown, read in english and portuguese, turned into operations by the
//! structure and the registry. "too much" hides the blocks of context,
//! groups the controls of a row that mixes them with readouts, and
//! tightens the spacing; "can't compare" swaps a table of numbers for a
//! chart, and says why when nothing fits (the Silences: its one list is
//! the cuts' ids, repeated); "bigger" emphasizes the numbers; "undo"
//! passes to the host; anything else is no change, for the proposer of
//! surfaces.

// shared with the other test binaries: this one reads less of it.
#[allow(dead_code)]
mod support;

use propose::{
    Adaptation, ProposeError, Rules, Transformed, Transformer, reads_as_change, transform_verified,
};
use sem::{Role, SnapshotBuilder, Value};
use support::{Changing, aside, claude_review, compact, silences, todo};
use uir::{ChartKind, Component, IrNode, Op, Step, TypographyRole};

fn sid(text: &str) -> sem::SemId {
    sem::SemId::parse(text).unwrap()
}

/// What the rules change for `intent`, verified.
fn changed(changing: &Changing, intent: &str) -> (Vec<Op>, uir::Adapted) {
    match transform_verified(&mut Rules, &changing.ask(intent)) {
        Ok(Transformed::Changed { ops, adapted }) => (ops, *adapted),
        other => panic!("\"{intent}\": {other:?}"),
    }
}

fn node<'a>(root: &'a IrNode, id: &str) -> &'a IrNode {
    let mut stack = vec![root];
    while let Some(n) = stack.pop() {
        if n.sem.as_str() == id {
            return n;
        }
        if let Some(r) = &n.repeat {
            stack.push(&r.item);
        }
        stack.extend(n.children.iter());
    }
    panic!("no {id}")
}

/// The acceptance over the Claude's review: the video card and the counts
/// card show only context (no action under them) and go aside; the cut's
/// row runs its range, decision, duration and frame beside three buttons,
/// and the buttons are grouped, for every item; every gap one step
/// denser. The same in portuguese.
#[test]
fn too_much_hides_the_context_groups_the_controls_and_tightens() {
    let changing = Changing::over(silences(), &claude_review());
    let expected = vec![
        Op::Hide {
            id: sid("review.video_card"),
        },
        Op::Hide {
            id: sid("review.counts_card"),
        },
        Op::Group {
            ids: vec![
                sid("review.cut_preview[{item}]"),
                sid("review.cut_apply[{item}]"),
                sid("review.cut_reject[{item}]"),
            ],
            into: sid("review.cut_item[{item}].actions"),
        },
        Op::Density { step: Step::Denser },
    ];
    for intent in [
        "too much information",
        "tem informação demais",
        "it's cluttered",
    ] {
        let (ops, adapted) = changed(&changing, intent);
        assert_eq!(ops, expected, "\"{intent}\"");
        assert_eq!(aside(&adapted), ["review.counts_card", "review.video_card"]);
        let row = node(&adapted.ir().surface.root, "review.cut_item[{item}]");
        let kids: Vec<&str> = row.children.iter().map(|c| c.sem.as_str()).collect();
        assert_eq!(
            kids,
            [
                "review.cut_range[{item}]",
                "review.cut_decision[{item}]",
                "review.cut_duration[{item}]",
                "review.cut_frame[{item}]",
                "review.cut_item[{item}].actions",
            ]
        );
        // the task stays: the detection binds actions, the title binds
        // nothing
        for kept in ["review.title", "review.controls", "review.apply_all"] {
            let _ = node(&adapted.ir().surface.root, kept);
        }
    }
}

/// Over the todo: the counters are the only context, and no row mixes
/// readouts with two controls (the entry is a field and a button, an
/// item its text and one checkbox): hidden, and tighter.
#[test]
fn too_much_over_the_todo_hides_its_counters() {
    let changing = Changing::over(todo(), &compact());
    let (ops, _) = changed(&changing, "too much");
    assert_eq!(
        ops,
        [
            Op::Hide {
                id: sid("todo_panel.counts")
            },
            Op::Density { step: Step::Denser }
        ]
    );
}

/// The Silences cannot compare as a table or a chart: its one list is
/// the cuts' ids, repeated. The rules say so, and nothing changes.
#[test]
fn cant_compare_on_the_silences_says_why() {
    let changing = Changing::over(silences(), &claude_review());
    for intent in ["I can't compare these", "não consigo comparar"] {
        match transform_verified(&mut Rules, &changing.ask(intent)) {
            Err(ProposeError::NoChange(why)) => {
                assert!(
                    why.contains("`review.cuts_list` repeats its items over `silences:cuts`"),
                    "{why}"
                );
                assert!(why.contains("no list of numbers is shown"), "{why}");
            }
            other => panic!("\"{intent}\": {other:?}"),
        }
    }
}

/// A board with a list of points (numbers) and a list of players (the
/// keys of its children: ids).
fn board() -> sem::Snapshot {
    let mut b = SnapshotBuilder::new();
    b.node("board", Role::Region, "board")
        .state(
            "points",
            Value::List(vec!["3".into(), "5".into(), "8".into()]),
        )
        .state("players", Value::List(vec!["1".into(), "2".into()]));
    for p in ["1", "2"] {
        b.node(format!("board.player[{p}]").as_str(), Role::ListItem, p)
            .parent("board")
            .state("name", format!("player {p}"));
    }
    b.finish().unwrap()
}

const BOARD: &str = r#"{"plev_ir": 1, "surface": {"id": "panel", "title": "board",
    "root": {"sem": "panel.root", "children": [
        {"sem": "panel.points", "component": "table", "label": "points",
         "bind": {"state": {"rows": "board:points"}}},
        {"sem": "panel.players", "component": "table", "label": "players",
         "bind": {"state": {"rows": "board:players"}}}
    ]}}}"#;

/// Where a list of numbers is shown as a table, "can't compare" swaps it
/// for a bar chart; the table of the players (their ids, the keys of the
/// board's children) stays. A chart asked again becomes a table.
#[test]
fn cant_compare_swaps_a_table_of_numbers_for_a_chart() {
    let changing = Changing::over(board(), BOARD);
    let (ops, adapted) = changed(&changing, "I can't compare these");
    assert_eq!(
        ops,
        [Op::SwapView {
            id: sid("panel.points"),
            component: Component::Chart(ChartKind::Bar)
        }]
    );
    let root = &adapted.ir().surface.root;
    assert_eq!(
        node(root, "panel.players").component,
        Some(Component::Table)
    );
    let mut again = Changing::over(board(), BOARD);
    again.current = adapted;
    let (back, _) = changed(&again, "difícil comparar");
    assert_eq!(
        back,
        [Op::SwapView {
            id: sid("panel.points"),
            component: Component::Table
        }]
    );
}

const READOUTS: &str = r#"{"plev_ir": 1, "surface": {"id": "panel", "title": "video",
    "root": {"sem": "panel.root", "children": [
        {"sem": "panel.file", "component": "text", "bind": {"state": {"text": "silences.video:file"}}},
        {"sem": "panel.length", "component": "text", "bind": {"state": {"text": "silences.video:duration"}}},
        {"sem": "panel.total", "component": "text", "bind": {"state": {"text": "silences.counts:total"}}}
    ]}}}"#;

/// "bigger numbers": the texts bound to a number, one step up the ramp;
/// the file name is no number. The app's words choose ("bigger
/// durations": the duration alone).
#[test]
fn bigger_numbers_emphasizes_what_shows_a_number() {
    let changing = Changing::over(silences(), READOUTS);
    for intent in [
        "bigger numbers",
        "números maiores",
        "I can't read the numbers",
    ] {
        let (ops, adapted) = changed(&changing, intent);
        assert_eq!(
            ops,
            [
                Op::Emphasize {
                    id: sid("panel.length")
                },
                Op::Emphasize {
                    id: sid("panel.total")
                }
            ],
            "\"{intent}\""
        );
        let root = &adapted.ir().surface.root;
        assert_eq!(
            node(root, "panel.total").props.typography,
            Some(TypographyRole::Title)
        );
        assert_eq!(node(root, "panel.file").props.typography, None);
    }
    let (ops, _) = changed(&changing, "bigger durations");
    assert_eq!(
        ops,
        [Op::Emphasize {
            id: sid("panel.length")
        }]
    );
}

#[test]
fn undo_passes_to_the_host() {
    let changing = Changing::over(todo(), &compact());
    for intent in ["undo", "desfazer", "undo that"] {
        assert_eq!(
            Rules.transform(&changing.ask(intent)),
            Ok(Adaptation::Undo),
            "\"{intent}\""
        );
    }
}

/// An intent for a surface, a comparison of views, a word neither the
/// rules' nor the app's: no change, for the proposer of surfaces.
#[test]
fn a_surface_is_no_change() {
    let changing = Changing::over(silences(), &claude_review());
    for intent in [
        "the list with the video and the counts",
        "compare the original with the cuts",
        "too much zzyzx",
        "two columns",
    ] {
        match Rules.transform(&changing.ask(intent)) {
            Err(ProposeError::NotAChange { .. }) => {}
            other => panic!("\"{intent}\": {other:?}"),
        }
    }
    assert!(reads_as_change("tem informação demais"));
    assert!(!reads_as_change("the list with the video and the counts"));
}
