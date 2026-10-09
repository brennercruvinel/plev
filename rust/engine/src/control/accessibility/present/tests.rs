//! The controls the tree presents inside an item, and a select pressed
//! open and set by its value: built into a tree and read back through the
//! app's one dispatcher as a screen reader's requests.

use accesskit::{Action, ActionData, ActionRequest, Node as AkNode, Role as Ak, Toggled, TreeId};
use accord::{
    Actor, Constraints, Dispatcher, Execution, Operable, Outcome, ParamSpec, Policy, Request,
    RequestId, Role, SemId, Semantics, Snapshot, SnapshotBuilder, Stage, Value, ValueKind,
};

use super::super::action::{Assist, Unmapped, primary, setter, translate};
use super::super::ids::{NodeIds, ROOT};
use super::super::tree::{self, Chrome, Shown};
use crate::drawing::compositor::Compositor;

fn sid(text: &str) -> SemId {
    match SemId::parse(text) {
        Ok(id) => id,
        Err(e) => panic!("{text}: {e}"),
    }
}

/// A list of two rows (a toggle and a remove each), a cut with three
/// actions of which one is off, a row whose remove is a control the page
/// projects itself, and a proposer select.
#[derive(Default)]
struct Board {
    done: [bool; 2],
    removed: Vec<usize>,
    decision: &'static str,
    chosen: &'static str,
    open: bool,
}

impl Semantics for Board {
    fn project(&self, b: &mut SnapshotBuilder) {
        b.node("list", Role::List, "tasks");
        for (at, done) in self.done.iter().enumerate() {
            if self.removed.contains(&at) {
                continue;
            }
            let id = format!("list.row[{at}]");
            let mut row = b.node(id.as_str(), Role::ListItem, format!("task {at}"));
            row.parent("list").state("completed", *done);
            row.action("toggle")
                .reads(format!("{id}:completed").as_str());
            row.action("remove").policy(Policy {
                destructive: true,
                ..Policy::default()
            });
        }
        let mut cut = b.node("cut", Role::ListItem, "0:03.0 to 0:05.0");
        cut.state("decision", self.decision);
        cut.action("preview_segment");
        cut.action("apply_cut")
            .enabled(self.decision != "apply")
            .reads("cut:decision");
        cut.action("reject_cut").reads("cut:decision");
        let mut own = b.node("own", Role::ListItem, "own");
        own.action("toggle");
        own.action("remove");
        b.node("own[remove]", Role::Button, "remove")
            .parent("own")
            .action("activate");
        let mut select = b.node("pick", Role::Select, "proposer");
        select
            .state("selected", self.chosen)
            .state("expanded", self.open)
            .constraints(Constraints {
                editable: true,
                options: Some(vec!["rules".to_owned(), "claude".to_owned()]),
                ..Constraints::default()
            });
        select
            .action("select")
            .param(ParamSpec {
                kind: ValueKind::Text,
                constrained: true,
            })
            .reads("pick:selected");
        select.action("open").reads("pick:expanded");
        select.action("close").reads("pick:expanded");
    }
}

impl Operable for Board {
    fn execute(&mut self, _id: RequestId, req: &Request) -> Execution {
        let row = |t: &str| {
            t.strip_prefix("list.row[")
                .and_then(|r| r.strip_suffix(']'))
                .and_then(|r| r.parse::<usize>().ok())
        };
        match (req.target.as_str(), req.action.as_str(), &req.param) {
            (t, "toggle", None) if row(t).is_some() => {
                let at = row(t).unwrap_or_default();
                self.done[at] = !self.done[at];
            }
            (t, "remove", None) if row(t).is_some() => {
                self.removed.push(row(t).unwrap_or_default())
            }
            ("cut", "reject_cut", None) => self.decision = "reject",
            ("pick", "open", None) => self.open = true,
            ("pick", "close", None) => self.open = false,
            ("pick", "select", Some(Value::Text(t))) if t == "claude" => self.chosen = "claude",
            _ => return Execution::Failed(format!("no {} on {}", req.action, req.target)),
        }
        Execution::Done(Outcome::default())
    }
}

impl Board {
    fn new() -> Board {
        Board {
            decision: "pending",
            chosen: "rules",
            ..Board::default()
        }
    }

    fn snapshot(&self) -> Snapshot {
        let mut b = SnapshotBuilder::new();
        self.project(&mut b);
        match b.finish() {
            Ok(s) => s,
            Err(e) => panic!("{e}"),
        }
    }

    fn seen(&self) -> (Vec<(accesskit::NodeId, AkNode)>, Shown) {
        let snapshot = self.snapshot();
        let mut c = Compositor::new();
        c.begin_frame();
        let chrome = Chrome {
            title: "board",
            scale: 1.0,
            size: (0, 0),
        };
        let (u, ids) = tree::build(&snapshot, &c, &chrome);
        (u.nodes, Shown { snapshot, ids })
    }
}

fn find<'a>(nodes: &'a [(accesskit::NodeId, AkNode)], ids: &NodeIds, id: &str) -> &'a AkNode {
    let want = ids.node(&sid(id));
    match nodes.iter().find(|(n, _)| Some(*n) == want) {
        Some((_, n)) => n,
        None => panic!("{id} is not in the tree"),
    }
}

fn ask(
    shown: &Shown,
    id: &str,
    action: Action,
    data: Option<ActionData>,
) -> Result<Assist, Unmapped> {
    let req = ActionRequest {
        action,
        target_tree: TreeId::ROOT,
        target_node: shown.ids.node(&sid(id)).unwrap_or(ROOT),
        data,
    };
    translate(&req, shown)
}

fn request(assist: Result<Assist, Unmapped>) -> Request {
    match assist {
        Ok(Assist::Request(r)) => r,
        other => panic!("expected a request, got {other:?}"),
    }
}

#[test]
fn an_item_holds_its_toggle_as_a_checkbox_and_its_other_actions_as_buttons() {
    let board = Board::new();
    let (nodes, shown) = board.seen();
    let n = |id: &str| find(&nodes, &shown.ids, id);
    let ids = |texts: &[&str]| -> Vec<_> {
        texts
            .iter()
            .filter_map(|t| shown.ids.node(&sid(t)))
            .collect()
    };

    let row = n("list.row[0]");
    assert_eq!(row.role(), Ak::ListItem);
    assert_eq!(row.toggled(), None, "the checkbox carries it");
    assert!(
        !row.supports_action(Action::Click),
        "the checkbox is pressed"
    );
    assert_eq!(
        row.children(),
        ids(&["list.row[0][toggle]", "list.row[0][remove]"])
    );
    let check = n("list.row[0][toggle]");
    assert_eq!(check.role(), Ak::CheckBox);
    assert_eq!(check.label(), Some("completed"), "the key its toggle reads");
    assert_eq!(check.toggled(), Some(Toggled::False));
    assert!(check.supports_action(Action::Click));
    let remove = n("list.row[0][remove]");
    assert_eq!(
        (remove.role(), remove.label()),
        (Ak::Button, Some("remove"))
    );
    assert!(remove.supports_action(Action::Click));
    for p in [check, remove] {
        assert!(!p.supports_action(Action::Focus), "walked to, not focused");
        assert_eq!(p.bounds(), None, "nothing binds an action");
    }

    // a cut with no click of its own: three buttons, the one off disabled
    let cut = n("cut");
    assert_eq!(
        cut.children(),
        ids(&["cut[preview_segment]", "cut[apply_cut]", "cut[reject_cut]"])
    );
    assert_eq!(n("cut[preview_segment]").label(), Some("preview segment"));
    assert_eq!(n("cut[apply_cut]").label(), Some("apply cut"));
    // the cut's decision is not "apply" yet: every button is on
    assert!(n("cut[apply_cut]").supports_action(Action::Click));

    // a control the page projects is the page's: nothing in its place
    assert_eq!(
        n("own").children(),
        ids(&["own[remove]", "own[toggle]"]),
        "the page's own button, once, then the checkbox"
    );
    assert_eq!(shown.ids.presented(&sid("own[remove]")), None);
    assert!(n("own[toggle]").supports_action(Action::Click));

    // what each stands for
    assert_eq!(
        shown.ids.presented(&sid("list.row[1][remove]")),
        Some((&sid("list.row[1]"), "remove"))
    );
    assert_eq!(shown.ids.presented(&sid("list.row[1]")), None);
    // a select is no item: nothing presented
    assert!(n("pick").children().is_empty());
}

#[test]
fn an_action_that_is_off_is_a_disabled_control_with_no_press() {
    let mut board = Board::new();
    board.decision = "apply";
    let (nodes, shown) = board.seen();
    let apply = find(&nodes, &shown.ids, "cut[apply_cut]");
    assert!(apply.is_disabled());
    assert!(!apply.supports_action(Action::Click));
    assert!(find(&nodes, &shown.ids, "cut[reject_cut]").supports_action(Action::Click));
}

/// The acceptance: a reader presses the presented remove and the
/// checkbox of a row; each reads back as the row's own action, asked by
/// assistive technology, and the one dispatcher runs it.
#[test]
fn a_click_on_a_presented_control_is_the_items_action_through_the_dispatcher() {
    let mut board = Board::new();
    let mut d = Dispatcher::new();

    let (_, shown) = board.seen();
    let req = request(ask(&shown, "list.row[0][toggle]", Action::Click, None));
    assert_eq!(
        (req.target.as_str(), req.action.as_str(), &req.actor),
        ("list.row[0]", "toggle", &Actor::Assistive)
    );
    let id = d.dispatch(&mut board, req, 1);
    assert!(matches!(
        d.receipt(id).and_then(|r| r.stage()),
        Some(Stage::Completed(_))
    ));
    assert_eq!(board.done, [true, false]);

    let (nodes, shown) = board.seen();
    assert_eq!(
        find(&nodes, &shown.ids, "list.row[0][toggle]").toggled(),
        Some(Toggled::True),
        "the next frame says it"
    );
    let req = request(ask(&shown, "list.row[1][remove]", Action::Click, None));
    assert_eq!(
        (req.target.as_str(), req.action.as_str()),
        ("list.row[1]", "remove")
    );
    let id = d.dispatch(&mut board, req, 2);
    assert!(matches!(
        d.receipt(id).and_then(|r| r.stage()),
        Some(Stage::Completed(_))
    ));
    assert_eq!(board.removed, [1]);
    let (_, shown) = board.seen();
    assert_eq!(
        shown.ids.node(&sid("list.row[1][remove]")),
        None,
        "gone with its row"
    );

    // the reject, which reads the decision: observed over what was shown
    let req = request(ask(&shown, "cut[reject_cut]", Action::Click, None));
    assert_eq!(
        req.observed
            .values
            .first()
            .map(|(at, v)| (at.to_string(), v.clone())),
        Some((
            "cut:decision".to_owned(),
            Some(Value::Text("pending".into()))
        ))
    );

    // a presented control takes no focus and no value
    assert!(matches!(
        ask(&shown, "list.row[0][remove]", Action::Focus, None),
        Err(Unmapped::NotFocusable(_))
    ));
    assert!(matches!(
        ask(&shown, "list.row[0][remove]", Action::SetValue, None),
        Err(Unmapped::Unsupported(Action::SetValue))
    ));
}

/// A select is pressed open, pressed closed while open, and set by its
/// value: its `select` with the option.
#[test]
fn a_select_is_pressed_open_and_shut_and_set_by_its_value() {
    let mut board = Board::new();
    let mut d = Dispatcher::new();
    let s = board.snapshot();
    let pick = s.get(&sid("pick"));
    assert_eq!(
        pick.and_then(primary).map(|a| a.name.as_str()),
        Some("open")
    );
    assert_eq!(
        pick.and_then(setter).map(|a| a.name.as_str()),
        Some("select")
    );

    let (nodes, shown) = board.seen();
    let node = find(&nodes, &shown.ids, "pick");
    assert_eq!(node.role(), Ak::ComboBox);
    for a in [Action::Click, Action::SetValue] {
        assert!(node.supports_action(a), "{a:?}");
    }
    let req = request(ask(&shown, "pick", Action::Click, None));
    assert_eq!(req.action.as_str(), "open");
    d.dispatch(&mut board, req, 1);
    assert!(board.open);

    let (_, shown) = board.seen();
    let req = request(ask(&shown, "pick", Action::Click, None));
    assert_eq!(req.action.as_str(), "close", "open, the press shuts it");
    d.dispatch(&mut board, req, 2);
    assert!(!board.open);

    let (_, shown) = board.seen();
    let text = Some(ActionData::Value("claude".into()));
    let req = request(ask(&shown, "pick", Action::SetValue, text));
    assert_eq!(
        (req.action.as_str(), &req.param),
        ("select", &Some(Value::Text("claude".into())))
    );
    let id = d.dispatch(&mut board, req, 3);
    assert!(matches!(
        d.receipt(id).and_then(|r| r.stage()),
        Some(Stage::Completed(_))
    ));
    assert_eq!(board.chosen, "claude");
}
