//! A screen reader's requests, from the tree it was shown to the app's
//! one dispatcher: a small shop app, projected, built into a tree, asked
//! through AccessKit action requests, and checked by its receipts.

use accesskit::{Action, ActionData, ActionRequest, NodeId, TreeId};
use sem::{
    Actor, Confirm, Constraints, Dispatcher, Execution, Operable, Outcome, ParamSpec, Policy,
    Rejection, Request, RequestId, Role, SemId, Semantics, Snapshot, SnapshotBuilder, Stage, Value,
    ValueKind,
};

use super::super::ids::ROOT;
use super::super::tree::{self, Chrome, Shown};
use super::*;
use crate::compositor::Compositor;

fn sid(text: &str) -> SemId {
    match SemId::parse(text) {
        Ok(id) => id,
        Err(e) => panic!("{text}: {e}"),
    }
}

/// A shop: a quantity slider, a gift checkbox, a note, a buy button, a
/// purge that always asks a person to confirm, and the confirmation it
/// shows while one waits.
#[derive(Default)]
struct Shop {
    qty: f64,
    gift: bool,
    note: String,
    bought: u32,
    purged: bool,
    inert: bool,
    /// The purge waiting for a person to confirm it.
    pending: Option<RequestId>,
}

impl Semantics for Shop {
    fn project(&self, b: &mut SnapshotBuilder) {
        b.node("shop", Role::Region, "shop").inert(self.inert);
        let mut qty = b.node("shop.qty", Role::Slider, "quantity");
        qty.parent("shop")
            .state("value", self.qty)
            .constraints(Constraints {
                editable: true,
                range: Some((0.0, 10.0)),
                step: Some(1.0),
                ..Constraints::default()
            });
        qty.action("set_value")
            .param(ParamSpec {
                kind: ValueKind::Num,
                constrained: true,
            })
            .reads("shop.qty:value");
        qty.action("increment").reads("shop.qty:value");
        qty.action("decrement").reads("shop.qty:value");
        let mut gift = b.node("shop.gift", Role::Checkbox, "gift wrap");
        gift.parent("shop").state("checked", self.gift);
        gift.action("toggle").reads("shop.gift:checked");
        let mut note = b.node("shop.note", Role::TextInput, "note");
        note.parent("shop")
            .state("text", self.note.as_str())
            .constraints(Constraints {
                editable: true,
                ..Constraints::default()
            });
        note.action("set_value")
            .param(ParamSpec {
                kind: ValueKind::Text,
                constrained: true,
            })
            .reads("shop.note:text");
        b.node("shop.buy", Role::Button, "buy")
            .parent("shop")
            .action("activate");
        b.node("shop.purge", Role::Button, "purge")
            .parent("shop")
            .action("activate")
            .policy(Policy {
                confirm: Confirm::Always,
                destructive: true,
                ..Policy::default()
            });
        if self.pending.is_some() {
            b.node("confirm", Role::Dialog, "purge everything?");
            b.node("confirm[yes]", Role::Button, "purge")
                .parent("confirm")
                .action("activate");
        }
    }
}

impl Operable for Shop {
    fn execute(&mut self, _id: RequestId, req: &Request) -> Execution {
        match (req.target.as_str(), req.action.as_str(), &req.param) {
            ("shop.qty", "set_value", Some(Value::Num(v))) => self.qty = *v,
            ("shop.qty", "increment", None) => self.qty += 1.0,
            ("shop.qty", "decrement", None) => self.qty -= 1.0,
            ("shop.gift", "toggle", None) => self.gift = !self.gift,
            ("shop.note", "set_value", Some(Value::Text(t))) => self.note.clone_from(t),
            ("shop.buy", "activate", None) => self.bought += 1,
            ("shop.purge", "activate", None) => self.purged = true,
            _ => return Execution::Failed(format!("no {} on {}", req.action, req.target)),
        }
        Execution::Done(Outcome::default())
    }
}

impl Shop {
    fn snapshot(&self) -> Snapshot {
        let mut b = SnapshotBuilder::new();
        self.project(&mut b);
        match b.finish() {
            Ok(s) => s,
            Err(e) => panic!("{e}"),
        }
    }

    /// What the screen reader is shown this frame.
    fn shown(&self) -> Shown {
        let snapshot = self.snapshot();
        let mut c = Compositor::new();
        c.begin_frame();
        let chrome = Chrome {
            title: "shop",
            scale: 1.0,
            size: (0, 0),
        };
        let (_, ids) = tree::build(&snapshot, &c, &chrome);
        Shown { snapshot, ids }
    }
}

/// How the shop's shell carries out what a screen reader asked: a
/// request aimed at the confirmation it shows confirms the waiting purge
/// as the request's actor; any other request is dispatched.
fn route(shop: &mut Shop, d: &mut Dispatcher, assist: Assist) -> RequestId {
    let Assist::Request(req) = assist else {
        panic!("expected a request, got {assist:?}");
    };
    if req.target == sid("confirm[yes]") {
        let Some(id) = shop.pending.take() else {
            panic!("nothing waits for a confirmation");
        };
        if let Err(e) = d.confirm(shop, id, req.actor.clone(), 2) {
            panic!("{e}");
        }
        return id;
    }
    let id = d.dispatch(shop, req, 1);
    if d.receipt(id).and_then(|r| r.stage()) == Some(&Stage::AwaitingConfirmation) {
        shop.pending = Some(id);
    }
    id
}

/// One frame: the shop is shown to the reader, which asks on it.
fn reader(
    shop: &mut Shop,
    d: &mut Dispatcher,
    target: &str,
    action: Action,
    data: Option<ActionData>,
) -> RequestId {
    let assist = ask(&shop.shown(), target, action, data);
    route(shop, d, assist)
}

fn ask(shown: &Shown, target: &str, action: Action, data: Option<ActionData>) -> Assist {
    let Some(node) = shown.ids.node(&sid(target)) else {
        panic!("{target} is not in the tree");
    };
    let req = ActionRequest {
        action,
        target_tree: TreeId::ROOT,
        target_node: node,
        data,
    };
    match translate(&req, shown) {
        Ok(assist) => assist,
        Err(e) => panic!("{target} {action:?}: {e}"),
    }
}

fn stage(d: &Dispatcher, id: RequestId) -> Stage {
    match d.receipt(id).and_then(|r| r.stage()) {
        Some(s) => s.clone(),
        None => panic!("no receipt for {id:?}"),
    }
}

#[test]
fn a_click_asks_for_the_primary_action_as_assistive_technology() {
    let mut shop = Shop::default();
    let mut d = Dispatcher::new();
    let assist = ask(&shop.shown(), "shop.gift", Action::Click, None);
    let Assist::Request(req) = &assist else {
        panic!("{assist:?}");
    };
    assert_eq!(req.actor, Actor::Assistive);
    assert_eq!(req.target, sid("shop.gift"));
    assert_eq!(req.action.as_str(), "toggle");
    assert_eq!(req.param, None);
    assert_eq!(
        req.observed.values,
        vec![(
            sem::StateRef::parse("shop.gift:checked").unwrap_or_else(|e| panic!("{e}")),
            Some(Value::Bool(false))
        )]
    );

    let id = route(&mut shop, &mut d, assist);
    assert!(matches!(stage(&d, id), Stage::Completed(_)));
    assert!(shop.gift);
    let receipt = d.receipt(id).unwrap_or_else(|| panic!("no receipt"));
    assert_eq!(receipt.request.actor, Actor::Assistive);

    // The same capability asked by a person reaches the same state.
    let mut person = Shop::default();
    let mut pd = Dispatcher::new();
    let seen = person.snapshot();
    let req = Request::observe(&seen, Actor::Human, "shop.gift", "toggle", None)
        .unwrap_or_else(|e| panic!("{e}"));
    pd.dispatch(&mut person, req, 1);
    assert_eq!(person.snapshot(), shop.snapshot());
}

#[test]
fn the_primary_action_follows_the_documented_order() {
    let mut b = SnapshotBuilder::new();
    let mut n = b.node("x", Role::Button, "x");
    n.action("select");
    n.action("submit");
    n.action("toggle");
    b.node("y", Role::Select, "y")
        .action("select")
        .param(ParamSpec {
            kind: ValueKind::Text,
            constrained: false,
        });
    let s = match b.finish() {
        Ok(s) => s,
        Err(e) => panic!("{e}"),
    };
    let pick = |id: &str| s.get(&sid(id)).and_then(primary).map(|a| a.name.as_str());
    assert_eq!(pick("x"), Some("toggle"), "toggle before submit and select");
    assert_eq!(pick("y"), None, "a click carries no parameter");
    assert_eq!(PRIMARY, ["activate", "toggle", "submit", "select"]);
}

/// The buttons the sonda of task-75 found a screen reader could focus
/// and not press (`propose`, `detect_silence`, `open_file`): a button, a
/// link or a menu item with one action without a parameter is pressed by
/// it. Two such actions, or another role, and a click has no action.
#[test]
fn a_button_with_one_action_of_its_own_is_pressed_by_it() {
    let mut b = SnapshotBuilder::new();
    b.node("propose", Role::Button, "propose").action("propose");
    b.node("docs", Role::Link, "docs").action("follow");
    b.node("quit", Role::MenuItem, "quit").action("quit");
    let mut both = b.node("both", Role::Button, "both");
    both.action("apply");
    both.action("reject");
    let mut named = b.node("named", Role::Button, "named");
    named.action("preview");
    named.action("activate");
    let mut param = b.node("param", Role::Button, "param");
    param.action("rename").param(ParamSpec {
        kind: ValueKind::Text,
        constrained: false,
    });
    param.action("go");
    b.node("cut", Role::ListItem, "cut").action("apply_cut");
    let s = match b.finish() {
        Ok(s) => s,
        Err(e) => panic!("{e}"),
    };
    let pick = |id: &str| s.get(&sid(id)).and_then(primary).map(|a| a.name.as_str());
    assert_eq!(pick("propose"), Some("propose"));
    assert_eq!(pick("docs"), Some("follow"));
    assert_eq!(pick("quit"), Some("quit"));
    assert_eq!(pick("both"), None, "two of its own: a click would guess");
    assert_eq!(pick("named"), Some("activate"), "the named one first");
    assert_eq!(pick("param"), Some("go"), "the one without a parameter");
    assert_eq!(pick("cut"), None, "a list item is not pressed");

    // what the tree offers is what the request carries
    let mut c = Compositor::new();
    c.begin_frame();
    let chrome = Chrome {
        title: "t",
        scale: 1.0,
        size: (0, 0),
    };
    let (u, ids) = tree::build(&s, &c, &chrome);
    let offers = |id: &str| {
        let node = ids.node(&sid(id)).unwrap_or(ROOT);
        u.nodes
            .iter()
            .find(|(n, _)| *n == node)
            .is_some_and(|(_, n)| n.supports_action(Action::Click))
    };
    assert!(offers("propose") && !offers("both") && !offers("cut"));
    let shown = Shown { snapshot: s, ids };
    let req = ActionRequest {
        action: Action::Click,
        target_tree: TreeId::ROOT,
        target_node: shown.ids.node(&sid("propose")).unwrap_or(ROOT),
        data: None,
    };
    match translate(&req, &shown) {
        Ok(Assist::Request(r)) => {
            assert_eq!(r.action.as_str(), "propose");
            assert_eq!(r.actor, Actor::Assistive);
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn set_value_increment_and_decrement_reach_the_app() {
    let mut shop = Shop::default();
    let mut d = Dispatcher::new();
    let number = Some(ActionData::NumericValue(3.0));
    let id = reader(&mut shop, &mut d, "shop.qty", Action::SetValue, number);
    assert!(matches!(stage(&d, id), Stage::Completed(_)));
    assert_eq!(shop.qty, 3.0);
    reader(&mut shop, &mut d, "shop.qty", Action::Increment, None);
    reader(&mut shop, &mut d, "shop.qty", Action::Increment, None);
    reader(&mut shop, &mut d, "shop.qty", Action::Decrement, None);
    assert_eq!(shop.qty, 4.0);

    // A number typed as text is a number for a numeric action.
    let typed = Some(ActionData::Value("7".into()));
    reader(&mut shop, &mut d, "shop.qty", Action::SetValue, typed);
    assert_eq!(shop.qty, 7.0);

    let text = Some(ActionData::Value("leave at the door".into()));
    let id = reader(&mut shop, &mut d, "shop.note", Action::SetValue, text);
    assert!(matches!(stage(&d, id), Stage::Completed(_)));
    assert_eq!(shop.note, "leave at the door");

    // Out of range: the dispatcher refuses it, with the reason.
    let far = Some(ActionData::NumericValue(99.0));
    let id = reader(&mut shop, &mut d, "shop.qty", Action::SetValue, far);
    assert!(matches!(
        stage(&d, id),
        Stage::Rejected(Rejection::InvalidParam { .. })
    ));
    assert_eq!(shop.qty, 7.0);
}

#[test]
fn focus_comes_back_as_a_focus_move_not_a_request() {
    let shown = Shop::default().shown();
    assert_eq!(
        ask(&shown, "shop.buy", Action::Focus, None),
        Assist::Focus(sid("shop.buy"))
    );
    let region = shown.ids.node(&sid("shop")).unwrap_or(ROOT);
    let req = ActionRequest {
        action: Action::Focus,
        target_tree: TreeId::ROOT,
        target_node: region,
        data: None,
    };
    assert_eq!(
        translate(&req, &shown),
        Err(Unmapped::NotFocusable(sid("shop")))
    );

    let inert = Shop {
        inert: true,
        ..Shop::default()
    }
    .shown();
    let buy = inert.ids.node(&sid("shop.buy")).unwrap_or(ROOT);
    let req = ActionRequest {
        target_node: buy,
        ..req
    };
    assert_eq!(
        translate(&req, &inert),
        Err(Unmapped::NotFocusable(sid("shop.buy")))
    );
}

#[test]
fn a_request_that_maps_to_nothing_says_why() {
    let shown = Shop::default().shown();
    let at = |node: NodeId, action: Action, data: Option<ActionData>| {
        translate(
            &ActionRequest {
                action,
                target_tree: TreeId::ROOT,
                target_node: node,
                data,
            },
            &shown,
        )
    };
    let node = |id: &str| shown.ids.node(&sid(id)).unwrap_or(ROOT);
    assert_eq!(at(ROOT, Action::Click, None), Err(Unmapped::Root));
    assert_eq!(
        at(NodeId(12345), Action::Click, None),
        Err(Unmapped::UnknownNode(NodeId(12345)))
    );
    assert_eq!(
        at(node("shop"), Action::Click, None),
        Err(Unmapped::NoPrimaryAction(sid("shop")))
    );
    assert_eq!(
        at(node("shop.qty"), Action::SetValue, None),
        Err(Unmapped::NoValue(sid("shop.qty")))
    );
    assert_eq!(
        at(node("shop.qty"), Action::ScrollIntoView, None),
        Err(Unmapped::Unsupported(Action::ScrollIntoView))
    );
    assert!(matches!(
        at(node("shop.buy"), Action::Increment, None),
        Err(Unmapped::Contract(_))
    ));
    let other = accesskit::TreeId(accesskit::Uuid::from_u128(7));
    let req = ActionRequest {
        action: Action::Click,
        target_tree: other,
        target_node: node("shop.buy"),
        data: None,
    };
    assert_eq!(translate(&req, &shown), Err(Unmapped::OtherTree(other)));
}

/// The screen reader decided on what it was shown; the state moved under
/// it, so the dispatcher answers `Stale` with the delta, as for any actor.
#[test]
fn a_request_on_state_that_moved_since_it_was_shown_is_stale() {
    let mut shop = Shop::default();
    let mut d = Dispatcher::new();
    let shown = shop.shown();
    let seen = shop.snapshot();
    let human = Request::observe(&seen, Actor::Human, "shop.gift", "toggle", None)
        .unwrap_or_else(|e| panic!("{e}"));
    d.dispatch(&mut shop, human, 1);
    assert!(shop.gift);

    let id = route(
        &mut shop,
        &mut d,
        ask(&shown, "shop.gift", Action::Click, None),
    );
    assert!(matches!(
        stage(&d, id),
        Stage::Rejected(Rejection::Stale { .. })
    ));
    assert!(shop.gift, "nothing ran");
}

#[test]
fn an_inert_target_comes_back_inert() {
    let mut shop = Shop {
        inert: true,
        ..Shop::default()
    };
    let mut d = Dispatcher::new();
    let id = reader(&mut shop, &mut d, "shop.buy", Action::Click, None);
    assert_eq!(
        stage(&d, id),
        Stage::Rejected(Rejection::Inert {
            target: sid("shop.buy")
        })
    );
    assert_eq!(shop.bought, 0);
}

/// A person using a screen reader confirms like a person at the screen:
/// the confirmation the app shows, clicked through AccessKit, reaches
/// `Dispatcher::confirm` as `Actor::Assistive`.
#[test]
fn a_confirmation_through_the_screen_reader_confirms_as_assistive() {
    let mut shop = Shop::default();
    let mut d = Dispatcher::new();
    let id = reader(&mut shop, &mut d, "shop.purge", Action::Click, None);
    assert_eq!(stage(&d, id), Stage::AwaitingConfirmation);
    assert!(!shop.purged);

    // The next frame shows the confirmation; the reader clicks it.
    let shown = shop.shown();
    let confirmed = route(
        &mut shop,
        &mut d,
        ask(&shown, "confirm[yes]", Action::Click, None),
    );
    assert_eq!(confirmed, id);
    assert!(shop.purged);
    let stages: Vec<Stage> = d
        .receipt(id)
        .map(|r| r.stages.iter().map(|(s, _)| s.clone()).collect())
        .unwrap_or_default();
    assert_eq!(stages.first(), Some(&Stage::Requested));
    assert!(stages.contains(&Stage::Confirmed {
        by: Actor::Assistive
    }));
    assert!(matches!(stages.last(), Some(Stage::Completed(_))));
}
