//! The tree builder: roles, labels, states, bounds, order, focus, and ids
//! that survive a layout change.

use accesskit::NodeId;
use rustc_hash::FxHashSet;
use sem::{Constraints, ParamSpec, SnapshotBuilder, ValueKind};

use super::*;
use crate::compositor::SceneNode;

fn sid(text: &str) -> SemId {
    match SemId::parse(text) {
        Ok(id) => id,
        Err(e) => panic!("{text}: {e}"),
    }
}

fn finish(b: SnapshotBuilder) -> Snapshot {
    match b.finish() {
        Ok(s) => s,
        Err(e) => panic!("{e}"),
    }
}

const CHROME: Chrome<'static> = Chrome {
    title: "plev test",
    scale: 1.0,
    size: (800, 600),
};

/// A frame with nothing bound.
fn unbound() -> Compositor {
    let mut c = Compositor::new();
    c.begin_frame();
    c
}

fn node<'a>(u: &'a TreeUpdate, ids: &NodeIds, id: &str) -> &'a AkNode {
    let Some(want) = ids.node(&sid(id)) else {
        panic!("{id} has no node id");
    };
    match u.nodes.iter().find(|(n, _)| *n == want) {
        Some((_, out)) => out,
        None => panic!("{id} is not in the update"),
    }
}

fn root(u: &TreeUpdate) -> &AkNode {
    match u.nodes.iter().find(|(n, _)| *n == ROOT) {
        Some((_, out)) => out,
        None => panic!("no root"),
    }
}

/// Every node once, the root first, each other node the child of exactly
/// one node, the focus among them: a full update AccessKit accepts.
fn assert_valid(u: &TreeUpdate) {
    assert_eq!(u.tree.as_ref().map(|t| t.root), Some(ROOT));
    assert_eq!(u.nodes.first().map(|(n, _)| *n), Some(ROOT));
    let listed: FxHashSet<NodeId> = u.nodes.iter().map(|(n, _)| *n).collect();
    assert_eq!(listed.len(), u.nodes.len(), "a node id is listed twice");
    let mut parented = FxHashSet::default();
    for (_, n) in &u.nodes {
        for child in n.children() {
            assert!(listed.contains(child), "{child:?} is not listed");
            assert!(parented.insert(*child), "{child:?} has two parents");
        }
    }
    assert_eq!(parented.len(), u.nodes.len() - 1, "a node is unreachable");
    assert!(listed.contains(&u.focus));
}

fn rect(n: &AkNode) -> [f64; 4] {
    match n.bounds() {
        Some(b) => [b.x0, b.y0, b.x1, b.y1],
        None => panic!("no bounds"),
    }
}

#[test]
fn every_role_is_projected_with_its_label() {
    let mut b = SnapshotBuilder::new();
    for (i, role) in Role::ALL.iter().enumerate() {
        b.node(format!("n{i}").as_str(), *role, role.as_str());
    }
    let (u, ids) = build(&finish(b), &unbound(), &CHROME);
    assert_valid(&u);
    for (i, role) in Role::ALL.iter().enumerate() {
        let n = node(&u, &ids, &format!("n{i}"));
        assert_eq!(n.role(), role::of(*role, false), "{}", role.as_str());
        assert_eq!(n.label(), Some(role.as_str()));
    }
    assert_eq!(root(&u).role(), accesskit::Role::Window);
    assert_eq!(root(&u).label(), Some("plev test"));
}

/// A form with one control for each kind of state.
fn form() -> Snapshot {
    let mut b = SnapshotBuilder::new();
    b.node("form", Role::Region, "form")
        .description("the settings form");
    b.node("form.name", Role::TextInput, "name")
        .parent("form")
        .state("text", "ada");
    b.node("form.pin", Role::TextInput, "pin")
        .parent("form")
        .state("text", "1234")
        .constraints(Constraints {
            secret: true,
            editable: true,
            ..Constraints::default()
        });
    let mut volume = b.node("form.volume", Role::Slider, "volume");
    volume
        .parent("form")
        .state("value", 0.25)
        .constraints(Constraints {
            editable: true,
            range: Some((0.0, 1.0)),
            step: Some(0.05),
            ..Constraints::default()
        });
    volume
        .action("set_value")
        .param(ParamSpec {
            kind: ValueKind::Num,
            constrained: true,
        })
        .reads("form.volume:value");
    volume.action("increment").reads("form.volume:value");
    volume.action("decrement").reads("form.volume:value");
    let mut gift = b.node("form.gift", Role::Checkbox, "gift");
    gift.parent("form").state("checked", true);
    gift.action("toggle").reads("form.gift:checked");
    let mut task = b.node("form.task", Role::ListItem, "write docs");
    task.parent("form").state("completed", false);
    task.action("toggle").reads("form.task:completed");
    b.node("form.wifi", Role::Toggle, "wifi")
        .parent("form")
        .state("on", true);
    let mut size = b.node("form.size", Role::Select, "size");
    size.parent("form")
        .state("selected", "large")
        .state("expanded", false);
    size.action("open").reads("form.size:expanded");
    b.node("form.save", Role::Button, "save")
        .parent("form")
        .enabled(false)
        .action("activate");
    b.node("form.files", Role::Tree, "files").parent("form");
    b.node("form.files[a]", Role::TreeItem, "a")
        .parent("form.files")
        .state("selected", true);
    b.node("form.tabs", Role::TabList, "tabs")
        .parent("form")
        .state("selected", "b");
    for key in ["a", "b"] {
        let id = format!("form.tabs[{key}]");
        let mut tab = b.node(id.as_str(), Role::Tab, key);
        tab.parent("form.tabs");
        tab.action("select").reads("form.tabs:selected");
    }
    finish(b)
}

#[test]
fn states_become_values_toggles_and_flags() {
    let (u, ids) = build(&form(), &unbound(), &CHROME);
    assert_valid(&u);
    let n = |id: &str| node(&u, &ids, id);

    assert_eq!(n("form").description(), Some("the settings form"));
    assert_eq!(n("form.name").value(), Some("ada"));
    assert_eq!(n("form.pin").role(), accesskit::Role::PasswordInput);
    assert_eq!(n("form.pin").value(), None, "a secret has no value");
    assert_eq!(n("form.pin").numeric_value(), None, "not even its length");

    let volume = n("form.volume");
    assert_eq!(volume.numeric_value(), Some(0.25));
    assert_eq!(volume.min_numeric_value(), Some(0.0));
    assert_eq!(volume.max_numeric_value(), Some(1.0));
    assert_eq!(volume.numeric_value_step(), Some(0.05));
    for a in [Action::SetValue, Action::Increment, Action::Decrement] {
        assert!(volume.supports_action(a), "{a:?}");
    }
    assert!(!volume.supports_action(Action::Click), "no primary action");

    assert_eq!(n("form.gift").toggled(), Some(Toggled::True));
    assert!(n("form.gift").supports_action(Action::Click));
    // an item that toggles holds its toggle as a checkbox (`present`)
    assert_eq!(n("form.task").toggled(), None);
    assert_eq!(
        n("form.task[toggle]").toggled(),
        Some(Toggled::False),
        "the bool its toggle reads"
    );
    assert_eq!(n("form.wifi").toggled(), Some(Toggled::True), "on");
    assert_eq!(n("form.name").toggled(), None);

    assert_eq!(n("form.size").value(), Some("large"));
    assert_eq!(n("form.size").is_expanded(), Some(false));
    assert!(n("form.save").is_disabled());
    assert!(
        !n("form.save").supports_action(Action::Click),
        "declared, and not offered while off"
    );
    assert!(!n("form.gift").is_disabled());
    assert_eq!(n("form.files[a]").is_selected(), Some(true));
    assert_eq!(n("form.tabs[b]").is_selected(), Some(true));
    assert_eq!(n("form.tabs[a]").is_selected(), Some(false));
    assert_eq!(n("form.tabs").is_selected(), None);
    assert_eq!(n("form.name").is_selected(), None);
}

#[test]
fn focusable_nodes_take_focus_and_inert_ones_are_hidden() {
    let mut b = SnapshotBuilder::new();
    b.node("page", Role::Region, "page").inert(true);
    b.node("page.go", Role::Button, "go")
        .parent("page")
        .action("activate");
    b.node("modal", Role::Dialog, "sure?");
    b.node("modal.ok", Role::Button, "ok")
        .parent("modal")
        .action("activate");
    let (u, ids) = build(&finish(b), &unbound(), &CHROME);
    let n = |id: &str| node(&u, &ids, id);
    assert!(n("page").is_hidden());
    assert!(n("page.go").is_hidden(), "inert is inherited");
    assert!(!n("page.go").supports_action(Action::Focus));
    assert!(!n("modal").is_hidden());
    assert!(n("modal.ok").supports_action(Action::Focus));
    assert!(n("modal.ok").supports_action(Action::Click));
    assert!(!n("modal").supports_action(Action::Focus), "not focusable");
}

#[test]
fn children_follow_the_projection_order_and_roots_hang_from_the_window() {
    let mut b = SnapshotBuilder::new();
    b.node("list", Role::List, "list");
    for key in ["c", "a", "b"] {
        b.node(format!("list[{key}]").as_str(), Role::ListItem, key)
            .parent("list");
    }
    b.node("aside", Role::Region, "aside");
    let (u, ids) = build(&finish(b), &unbound(), &CHROME);
    assert_valid(&u);
    let order = |texts: &[&str]| -> Vec<NodeId> {
        texts.iter().filter_map(|t| ids.node(&sid(t))).collect()
    };
    assert_eq!(
        node(&u, &ids, "list").children(),
        order(&["list[c]", "list[a]", "list[b]"])
    );
    assert_eq!(root(&u).children(), order(&["list", "aside"]));
}

#[test]
fn focus_is_the_focused_node_or_the_window() {
    let mut b = SnapshotBuilder::new();
    b.node("name", Role::TextInput, "name");
    b.focus("name");
    let (u, ids) = build(&finish(b), &unbound(), &CHROME);
    assert_eq!(Some(u.focus), ids.node(&sid("name")));

    let mut b = SnapshotBuilder::new();
    b.node("name", Role::TextInput, "name");
    let (u, _) = build(&finish(b), &unbound(), &CHROME);
    assert_eq!(u.focus, ROOT);
}

#[test]
fn bounds_are_the_binding_in_physical_pixels_inside_its_clip() {
    let mut b = SnapshotBuilder::new();
    b.node("bar", Role::Toolbar, "bar");
    b.node("bar.go", Role::Button, "go").parent("bar");
    b.node("bar.stop", Role::Button, "stop").parent("bar");
    b.node("list", Role::List, "list");
    b.node("list[1]", Role::ListItem, "one").parent("list");
    b.node("list[2]", Role::ListItem, "two").parent("list");
    b.node("list[3]", Role::ListItem, "three").parent("list");
    b.node("status", Role::Status, "ready");
    let s = finish(b);

    let mut c = Compositor::new();
    c.begin_frame();
    c.bind(&sid("bar.go"), [10.0, 20.0, 30.0, 40.0]);
    c.bind(&sid("bar.stop"), [50.0, 20.0, 30.0, 40.0]);
    c.push(SceneNode::PushClip {
        x: 0.0,
        y: 100.0,
        w: 200.0,
        h: 50.0,
    });
    c.bind(&sid("list[1]"), [0.0, 90.0, 200.0, 20.0]);
    c.bind(&sid("list[2]"), [0.0, 130.0, 200.0, 20.0]);
    c.bind(&sid("list[3]"), [0.0, 160.0, 200.0, 20.0]);
    c.push(SceneNode::PopClip);

    let hidpi = Chrome {
        scale: 2.0,
        ..CHROME
    };
    let (u, ids) = build(&s, &c, &hidpi);
    let n = |id: &str| node(&u, &ids, id);
    assert_eq!(rect(n("bar.go")), [20.0, 40.0, 80.0, 120.0], "scaled by 2");
    // A container nothing binds: the union of its children.
    assert_eq!(rect(n("bar")), [20.0, 40.0, 160.0, 120.0]);
    // Clipped: the visible part only; wholly clipped: no bounds.
    assert_eq!(rect(n("list[1]")), [0.0, 200.0, 400.0, 220.0]);
    assert_eq!(rect(n("list[2]")), [0.0, 260.0, 400.0, 300.0]);
    assert_eq!(n("list[3]").bounds(), None, "scrolled out");
    assert_eq!(rect(n("list")), [0.0, 200.0, 400.0, 300.0]);
    // Unbound with nothing bound under it: in the tree, without bounds.
    assert_eq!(n("status").bounds(), None);
    assert_eq!(n("status").label(), Some("ready"));
    assert_eq!(rect(root(&u)), [0.0, 0.0, 800.0, 600.0]);
}

/// The same meaning drawn at two widths: every node keeps its id and only
/// the bounds move, so a screen reader on a node stays on it.
#[test]
fn node_ids_are_stable_across_frames_whose_layout_differs() {
    let s = form();
    let place = |x: f32, w: f32| {
        let mut c = Compositor::new();
        c.begin_frame();
        for (i, n) in s.nodes().values().enumerate() {
            c.bind(&n.id, [x, i as f32 * 30.0, w, 28.0]);
        }
        c
    };
    let (narrow, a) = build(&s, &place(0.0, 360.0), &CHROME);
    let (wide, b) = build(&s, &place(240.0, 900.0), &CHROME);
    // every node, and the checkbox the task holds (`present`)
    assert_eq!(a.len(), s.len() + 1);
    assert_eq!(
        a.node(&sid("form.task[toggle]")),
        b.node(&sid("form.task[toggle]"))
    );
    for id in s.nodes().keys() {
        assert_eq!(a.node(id), b.node(id), "{id}");
        let text = id.as_str();
        assert_ne!(
            rect(node(&narrow, &a, text)),
            rect(node(&wide, &b, text)),
            "{id} moved"
        );
    }
}

/// The Surface page's quick todo wraps its field and its counts in groups
/// with no name (the sonda of task-75): a reader would stop on "group"
/// and hear nothing. Such a group is structure, and every other one keeps
/// its role.
#[test]
fn a_group_that_names_and_does_nothing_is_only_structure() {
    let mut b = SnapshotBuilder::new();
    b.node("panel", Role::Region, "panel");
    b.node("panel.entry", Role::Group, "").parent("panel");
    b.node("panel.entry.input", Role::TextInput, "new todo")
        .parent("panel.entry");
    b.node("panel.named", Role::Group, "filter").parent("panel");
    b.node("panel.told", Role::Group, "")
        .parent("panel")
        .description("the counts");
    b.node("panel.acts", Role::Group, "")
        .parent("panel")
        .action("activate");
    b.node("panel.list", Role::List, "").parent("panel");
    let (u, ids) = build(&finish(b), &unbound(), &CHROME);
    assert_valid(&u);
    let role = |id: &str| node(&u, &ids, id).role();
    assert_eq!(role("panel.entry"), accesskit::Role::GenericContainer);
    assert_eq!(
        node(&u, &ids, "panel.entry").children(),
        ids.node(&sid("panel.entry.input"))
            .into_iter()
            .collect::<Vec<_>>(),
        "its children stay under it; the adapter lifts them"
    );
    assert_eq!(role("panel.named"), accesskit::Role::Group);
    assert_eq!(
        role("panel.told"),
        accesskit::Role::Group,
        "it says something"
    );
    assert_eq!(
        role("panel.acts"),
        accesskit::Role::Group,
        "it does something"
    );
    assert_eq!(role("panel.list"), accesskit::Role::List, "a group alone");
}

/// A page with a counter, a status line that keeps its words in
/// `message`, a toast and an error, and a field with the focus.
fn announcing(counter: &str, message: &str, focus: bool) -> Snapshot {
    let mut b = SnapshotBuilder::new();
    b.node("page", Role::Region, "page");
    b.node("page.name", Role::TextInput, "name")
        .parent("page")
        .state("text", "ada");
    b.node("page.counter", Role::Status, counter)
        .parent("page")
        .state("done", 1_i64);
    b.node("page.status", Role::Status, "status")
        .parent("page")
        .state("message", message)
        .state("busy", false);
    b.node("toasts", Role::Region, "notifications");
    b.node("toasts[0]", Role::Status, "saved")
        .parent("toasts")
        .action("dismiss");
    b.node("toasts[1]", Role::Alert, "push failed")
        .parent("toasts")
        .action("dismiss");
    b.node("page.level", Role::Status, "level")
        .parent("page")
        .state("value", 0.5);
    b.node("page.form", Role::Group, "form").parent("page");
    if focus {
        b.focus("page.name");
    }
    finish(b)
}

#[test]
fn a_status_is_a_polite_live_region_and_an_alert_an_assertive_one() {
    let (u, ids) = build(
        &announcing("2 active", "detecting", true),
        &unbound(),
        &CHROME,
    );
    assert_valid(&u);
    let n = |id: &str| node(&u, &ids, id);
    for id in ["page.counter", "page.status", "toasts[0]", "page.level"] {
        assert_eq!(n(id).live(), Some(Live::Polite), "{id}");
    }
    assert_eq!(n("toasts[1]").live(), Some(Live::Assertive));
    for id in ["page", "page.name", "toasts", "page.form"] {
        assert_eq!(n(id).live(), None, "{id} is not live");
    }
    // what a reader hears: the label, or the message a status keeps
    assert_eq!(n("page.counter").value(), Some("2 active"));
    assert_eq!(
        n("page.counter").label(),
        Some("2 active"),
        "the name stays"
    );
    assert_eq!(n("page.status").value(), Some("detecting"));
    assert_eq!(
        n("page.status").label(),
        Some("detecting"),
        "the name follows the message: Windows and AT-SPI announce the name"
    );
    assert_eq!(n("toasts[0]").value(), Some("saved"));
    assert_eq!(n("toasts[1]").value(), Some("push failed"));
    // a status with a value of its own keeps it
    assert_eq!(n("page.level").numeric_value(), Some(0.5));
    assert_eq!(n("page.level").value(), None);
}

#[test]
fn an_empty_message_says_nothing() {
    let (u, ids) = build(&announcing("2 active", "", false), &unbound(), &CHROME);
    let status = node(&u, &ids, "page.status");
    assert_eq!(status.live(), Some(Live::Polite));
    assert_eq!(status.value(), None, "no words, nothing to announce");
    assert_eq!(
        status.label(),
        Some("status"),
        "the page's name, while silent"
    );
}

/// The name follows the message frame to frame, where Windows and AT-SPI
/// announce a live node (on a change of its name), and the value where
/// macOS does (on a change of its value): the same words on all three.
#[test]
fn a_status_message_is_its_name_and_its_value_frame_to_frame() {
    let say = |message: &str| {
        let (u, ids) = build(&announcing("2 active", message, false), &unbound(), &CHROME);
        let n = node(&u, &ids, "page.status");
        (n.label().map(str::to_owned), n.value().map(str::to_owned))
    };
    assert_eq!(
        say("detecting"),
        (Some("detecting".into()), Some("detecting".into()))
    );
    assert_eq!(
        say("2 silences in 40 ms"),
        (
            Some("2 silences in 40 ms".into()),
            Some("2 silences in 40 ms".into())
        )
    );
    assert_eq!(say(""), (Some("status".into()), None));
}

/// A live region is heard where it is: a status that changes, or a
/// toast that arrives, leaves the focus on the field the person is in,
/// and the field's node is the same one.
#[test]
fn an_announcement_never_moves_the_focus() {
    let (before, a) = build(&announcing("2 active", "", true), &unbound(), &CHROME);
    let (after, b) = build(
        &announcing("3 active", "done: 2 cuts", true),
        &unbound(),
        &CHROME,
    );
    assert_eq!(Some(before.focus), a.node(&sid("page.name")));
    assert_eq!(after.focus, before.focus);
    assert_eq!(b.node(&sid("page.name")), a.node(&sid("page.name")));
    assert_eq!(node(&after, &b, "page.counter").value(), Some("3 active"));
    assert_eq!(
        node(&after, &b, "page.status").value(),
        Some("done: 2 cuts")
    );
    let (none, _) = build(&announcing("3 active", "done", false), &unbound(), &CHROME);
    assert_eq!(
        none.focus, ROOT,
        "nothing focused: the window, not a status"
    );
}

/// A secret is never read out, not even by a live region.
#[test]
fn a_secret_status_is_not_announced() {
    let mut b = SnapshotBuilder::new();
    b.node("pin", Role::Status, "1234")
        .constraints(Constraints {
            secret: true,
            ..Constraints::default()
        });
    let (u, ids) = build(&finish(b), &unbound(), &CHROME);
    let pin = node(&u, &ids, "pin");
    assert_eq!(pin.value(), None);
    assert_eq!(pin.live(), None);
}

#[test]
fn with_no_meaning_the_tree_is_the_window() {
    let (u, ids) = build(&Snapshot::default(), &unbound(), &CHROME);
    assert_valid(&u);
    assert_eq!(u.nodes.len(), 1);
    assert!(ids.is_empty());
    assert_eq!(root(&u).label(), Some("plev test"));
    let (u, _) = build(
        &Snapshot::default(),
        &unbound(),
        &Chrome {
            size: (0, 0),
            ..CHROME
        },
    );
    assert_eq!(root(&u).bounds(), None);
}

/// A press the app would refuse is not offered: a field whose `submit` is
/// off (empty) keeps its focus and its value and loses its press until
/// the text turns it on; an action off on an enabled button, the same.
#[test]
fn a_press_is_offered_only_while_its_action_is_on() {
    let field = |text: &str| {
        let mut b = SnapshotBuilder::new();
        let mut f = b.node("todo.input", Role::TextInput, "new todo");
        f.state("text", text);
        f.action("submit")
            .enabled(!text.is_empty())
            .reads("todo.input:text");
        let mut go = b.node("go", Role::Button, "go");
        go.action("activate").enabled(false);
        finish(b)
    };
    let (u, ids) = build(&field(""), &unbound(), &CHROME);
    let input = node(&u, &ids, "todo.input");
    assert!(!input.supports_action(Action::Click), "empty: no press");
    assert!(input.supports_action(Action::Focus));
    assert!(!input.is_disabled(), "the field still edits");
    let go = node(&u, &ids, "go");
    assert!(!go.supports_action(Action::Click) && !go.is_disabled());
    let (u, ids) = build(&field("buy milk"), &unbound(), &CHROME);
    assert!(node(&u, &ids, "todo.input").supports_action(Action::Click));
}

/// A group named by nothing but its key, as a generated surface names an
/// instance of a repeat, is named by the labels inside it; a group named
/// by words keeps them, and a keyed group with no label inside keeps its
/// key.
#[test]
fn a_group_named_by_its_key_is_named_by_the_text_inside_it() {
    let mut b = SnapshotBuilder::new();
    b.node("panel", Role::List, "items");
    b.node("panel.item[1]", Role::Group, "1").parent("panel");
    b.node("panel.item[1].text", Role::Label, "buy milk")
        .parent("panel.item[1]");
    b.node("panel.item[1].done", Role::Checkbox, "done")
        .parent("panel.item[1]")
        .state("checked", false);
    b.node("panel.item[2]", Role::Group, "2").parent("panel");
    b.node("panel.item[2].done", Role::Checkbox, "done")
        .parent("panel.item[2]")
        .state("checked", true);
    b.node("panel.named", Role::Group, "named").parent("panel");
    b.node("panel.named.text", Role::Label, "inside")
        .parent("panel.named");
    let (u, ids) = build(&finish(b), &unbound(), &CHROME);
    assert_valid(&u);
    assert_eq!(node(&u, &ids, "panel.item[1]").label(), Some("buy milk"));
    assert_eq!(
        node(&u, &ids, "panel.item[1]").role(),
        accesskit::Role::Group
    );
    assert_eq!(
        node(&u, &ids, "panel.item[2]").label(),
        Some("2"),
        "no text inside"
    );
    assert_eq!(node(&u, &ids, "panel.named").label(), Some("named"));
}
