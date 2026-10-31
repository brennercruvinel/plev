//! Every page of the gallery as a screen reader meets it (task-75 phase
//! 1): the tree the shell's bridge builds from the page's projection,
//! placed by one drawn frame, checked for what a reader needs. Every
//! control has a name and a role that takes focus; Tab and the reader
//! walk the controls in the same order, and the page takes the focus the
//! reader asks for; every action the tree offers reads back as a request
//! the dispatcher admits from assistive technology.
//!
//! Nothing here runs a request: the Files page asks the platform's own
//! dialogs and the Surface page a model, and a test opens neither. What
//! the dispatcher does with a request is `sem`'s, tested there and in the
//! todo page's own screen reader tests; here the tree and the contract
//! are held to each other.
//!
//! What the pages do not give a reader yet is listed, not fixed: the
//! pages that project no meaning, each with its reason (`SILENT`), and
//! what phases 2a and 2b left with theirs. Each list is exact, so a page
//! that starts projecting, or a new gap, shows here. Phase 2a emptied the
//! others: an item's actions its own node does not reach are controls the
//! tree presents inside it (`accessibility::present`), a select is
//! pressed open and set by its value, the pages project the focus they
//! take, and a press is offered only while its action is on. Phase 2b
//! made every component page whose widgets project in comps project what
//! it shows (`view::dispatch::component`).
#![cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]

use accord::{Actor, Request, Role, SemId, StateKey, Value};
use engine::control::accessibility::accesskit::{
    Action, ActionData, ActionRequest, Live, Node as AkNode, NodeId, Role as AkRole, TreeId,
    TreeUpdate,
};
use engine::control::accessibility::{self, Assist, Chrome, Shown, focus};
use engine::drawing::compositor::Compositor;
use uigallery::views::{PAGES, ShowcaseView};

/// The pages that project their meaning, in sidebar order: since phase
/// 2b every component page whose widgets project in comps ("Alert
/// Dialog" shows the Dialog page, "Toggle Group" the todo page's
/// example), and the apps of phase 1. Every other page is in `SILENT`.
const PROJECTING: &[&str] = &[
    "Accordion",
    "Alert",
    "Alert Dialog",
    "Aspect Ratio",
    "Attachment",
    "Avatar",
    "Badge",
    "Breadcrumb",
    "Bubble",
    "Button",
    "Button Group",
    "Calendar",
    "Carousel",
    "Chart",
    "Checkbox",
    "Chip",
    "Code Block",
    "Collapsible",
    "Combobox",
    "Command",
    "Context Menu",
    "Data Table",
    "Date Picker",
    "Dialog",
    "Drawer",
    "Dropdown Menu",
    "Field",
    "Graph",
    "Hover Card",
    "Icon Button",
    "Input",
    "Input Group",
    "Input OTP",
    "Item",
    "Kbd",
    "Label",
    "Marker",
    "Menubar",
    "Menus",
    "Message",
    "Message Scroller",
    "Navigation Menu",
    "Pagination",
    "Panel",
    "Panel Header",
    "Popover",
    "Progress",
    "Radio Group",
    "Resizable",
    "Scroll Area",
    "Select",
    "Sheet",
    "Skeleton",
    "Slider",
    "Spinner",
    "Stat",
    "Switch",
    "Table",
    "Tabs",
    "Textarea",
    "Toast",
    "Toggle",
    "Toggle Group",
    "Tooltip",
    "Topbar",
    "Tree",
    "Todo",
    "Surface",
    "Silences",
    "Files",
    "Capture",
];

/// Why a page is the window alone to a reader.
#[derive(Clone, Copy, Debug)]
enum Why {
    /// It shows a widget with no projection in comps: what it would need
    /// (task-75, "fase 2b: a sonda", group 2). The next phase.
    Unprojected(&'static str),
    /// Nothing to operate and nothing to read but a sample of drawing
    /// (group 3): it projects nothing.
    Nothing(&'static str),
}

use Why::{Nothing, Unprojected};

const PLACEHOLDER: Why = Nothing("a placeholder: no widget in comps yet");

/// The pages that project nothing, in sidebar order, each with its
/// reason. With `PROJECTING`, every page of the table.
const SILENT: &[(&str, Why)] = &[
    (
        "Theme",
        Unprojected("the palette cards are drawn by hand: options pressed by shell.theme"),
    ),
    ("Typography", Nothing("a type specimen table")),
    ("Icons", Nothing("a grid of icons")),
    (
        "Effects",
        Nothing("samples of the shadow stacks, blur and gradient"),
    ),
    (
        "Card",
        Unprojected("Card: a group per variant with its words and numbers"),
    ),
    (
        "Empty",
        Unprojected("EmptyState: a group named by its title, its message, its call to action"),
    ),
    ("Questionnaire", PLACEHOLDER),
    ("Separator", Nothing("rules between captions")),
    (
        "Sidebar",
        Unprojected("Sidebar: a navigation region of its links, each group folded by an action"),
    ),
    (
        "Dock",
        Unprojected("the dock is drawn by hand: its contacts and a field, over a stage of bubbles"),
    ),
    (
        "Editor",
        Unprojected("EditorView: a multiline text input with its selection"),
    ),
    (
        "Git",
        Unprojected("the ide's views: the ide is out of this task"),
    ),
    (
        "Builder",
        Unprojected("the builder's elements: a button per hit region, the count as a status"),
    ),
];

/// The actions a reader can ask of a node: what the tree offers.
const OFFERED: [Action; 5] = [
    Action::Click,
    Action::Focus,
    Action::SetValue,
    Action::Increment,
    Action::Decrement,
];

const CHROME: Chrome<'static> = Chrome {
    title: "plev, design system showcase",
    scale: 2.0,
    size: (2560, 1600),
};

/// A page as the shell shows it to a reader: the view on the page, one
/// frame drawn, and the tree of that frame, built from what the bridge is
/// given (`reader_semantics`).
struct Seen {
    view: ShowcaseView,
    update: TreeUpdate,
    shown: Shown,
}

fn see(title: &str) -> Seen {
    let mut view = ShowcaseView::new(1280.0, 800.0);
    view.jump_to_page(title);
    assert_eq!(view.page_title(), title);
    look(view)
}

/// The view as a reader meets it now: one frame drawn, its tree.
fn look(mut view: ShowcaseView) -> Seen {
    let mut c = Compositor::new();
    view.render(&mut c);
    let snapshot = view.reader_semantics().unwrap_or_default();
    let (update, ids) = accessibility::build(&snapshot, &c, &CHROME);
    Seen {
        view,
        update,
        shown: Shown { snapshot, ids },
    }
}

impl Seen {
    fn node(&self, id: NodeId) -> Option<&AkNode> {
        self.update
            .nodes
            .iter()
            .find(|(n, _)| *n == id)
            .map(|(_, n)| n)
    }

    /// Every node as a reader walks the tree: depth first from the
    /// window, children in order, with its `SemId`.
    fn walk(&self) -> Vec<(NodeId, &SemId, &AkNode)> {
        let mut out = Vec::new();
        let mut stack = vec![accessibility::ROOT];
        while let Some(id) = stack.pop() {
            let Some(node) = self.node(id) else {
                panic!("{id:?} is a child and not in the update");
            };
            if let Some(sem) = self.shown.ids.sem(id) {
                out.push((id, sem, node));
            }
            stack.extend(node.children().iter().rev());
        }
        out
    }

    fn sem(&self, id: &SemId) -> &accord::Node {
        match self.shown.snapshot.get(id) {
            Some(n) => n,
            None => panic!("{id} is in the tree and not in the snapshot"),
        }
    }

    /// The item and the action a control the tree presents stands for;
    /// `None` for a node of the snapshot.
    fn presented(&self, id: &SemId) -> Option<(&SemId, &str)> {
        self.shown.ids.presented(id)
    }
}

fn projecting() -> impl Iterator<Item = Seen> {
    PROJECTING.iter().map(|title| see(title))
}

/// Every node once, the window first, each other node the child of one
/// node, the focus among them: a full update AccessKit accepts.
fn assert_valid(title: &str, u: &TreeUpdate) {
    assert_eq!(u.nodes.first().map(|(n, _)| *n), Some(accessibility::ROOT));
    let mut listed: Vec<NodeId> = u.nodes.iter().map(|(n, _)| *n).collect();
    listed.sort();
    listed.dedup();
    assert_eq!(listed.len(), u.nodes.len(), "{title}: a node twice");
    let mut parented: Vec<NodeId> = u
        .nodes
        .iter()
        .flat_map(|(_, n)| n.children().iter().copied())
        .collect();
    parented.sort();
    let children = parented.len();
    parented.dedup();
    assert_eq!(parented.len(), children, "{title}: a node with two parents");
    assert_eq!(children, u.nodes.len() - 1, "{title}: a node unreachable");
    assert!(listed.binary_search(&u.focus).is_ok(), "{title}: focus");
}

/// Every page is a valid tree; the pages that project are `PROJECTING`,
/// and the ones a reader meets empty are `SILENT`, each with its reason.
/// Phase 1 counted 76 silent; phase 2b leaves the ones whose widgets have
/// no projection in comps, and the samples with nothing to operate.
#[test]
fn every_page_is_a_valid_tree_and_the_silent_ones_are_the_known_ones() {
    let mut projected = Vec::new();
    let mut silent = Vec::new();
    for page in PAGES {
        let seen = see(page.title);
        assert_valid(page.title, &seen.update);
        if seen.shown.snapshot.is_empty() {
            assert_eq!(seen.update.nodes.len(), 1, "{}: the window", page.title);
            silent.push(page.title);
        } else {
            projected.push(page.title);
        }
    }
    assert_eq!(projected, PROJECTING);
    let known: Vec<&str> = SILENT.iter().map(|(title, _)| *title).collect();
    assert_eq!(silent, known);
    for (title, why) in SILENT {
        let (Unprojected(reason) | Nothing(reason)) = why;
        assert!(!reason.is_empty(), "{title}: say why");
    }
}

#[test]
fn every_control_has_a_name_and_takes_focus_and_no_group_is_empty() {
    for seen in projecting() {
        let title = seen.view.page_title();
        for (_, id, n) in seen.walk() {
            if seen.presented(id).is_some() {
                assert!(
                    n.label().is_some_and(|l| !l.trim().is_empty()),
                    "{title}: {id} is a presented control without a name"
                );
                continue;
            }
            let node = seen.sem(id);
            let control = node.role.is_interactive() || n.supports_action(Action::Focus);
            if control {
                assert!(
                    n.label().is_some_and(|l| !l.trim().is_empty()),
                    "{title}: {id} is a control without a name"
                );
            }
            if node.role.is_interactive() && node.enabled && !node.inert {
                assert!(
                    n.supports_action(Action::Focus),
                    "{title}: {id} is a {} that takes no focus",
                    node.role.as_str()
                );
            }
            if n.role() == AkRole::Group {
                assert!(
                    n.label().is_some() || n.description().is_some(),
                    "{title}: {id} is a group a reader stops on and hears nothing"
                );
            }
        }
    }
}

/// The focus a page takes and does not project: the reader moved it,
/// the page agreed, and the tree still says the window has it. Phase 1
/// listed 13 (the todo's filter tabs, the quick todo's filters and
/// checkboxes, the proposer selects, the Silences page's detect and apply
/// all); since phase 2a every page projects the focus it takes (the todo
/// keeps its tab, the Surface and Silences pages the surface's node, the
/// proposing row its select and buttons, the Silences page its buttons).
const FOCUS_NOT_PROJECTED: &[&str] = &[];

#[test]
fn tab_and_the_reader_walk_the_same_order_and_the_page_takes_the_focus() {
    let mut unprojected: Vec<String> = Vec::new();
    for seen in projecting() {
        let title = seen.view.page_title();
        let walked: Vec<&SemId> = seen
            .walk()
            .into_iter()
            .filter(|(_, _, n)| {
                n.supports_action(Action::Focus) && !n.is_disabled() && !n.is_hidden()
            })
            .map(|(_, id, _)| id)
            .collect();
        let tab = focus::order(&seen.shown.snapshot);
        assert_eq!(tab, walked, "{title}: Tab and the reader disagree");
        for id in tab {
            let mut fresh = see(title);
            assert!(fresh.view.assist_focus(id), "{title}: {id} refused");
            let now = fresh.view.semantics().unwrap_or_default();
            if now.focused() != Some(id) {
                unprojected.push(id.to_string());
            }
        }
    }
    unprojected.sort();
    unprojected.dedup();
    assert_eq!(unprojected, FOCUS_NOT_PROJECTED);
}

/// The value a reader's `SetValue` carries in this test: the node's own,
/// so the request is the one a reader would make without changing it.
fn same_value(n: &AkNode) -> ActionData {
    match n.numeric_value() {
        Some(v) => ActionData::NumericValue(v),
        None => ActionData::Value(n.value().unwrap_or("").into()),
    }
}

/// The presses a page offers while the action behind them is disabled.
/// Phase 1 listed the todo field's and the quick todo field's `submit`
/// while the field is empty: the tree offered the press and the
/// dispatcher refused it, so a reader heard nothing happen. Since phase
/// 2a a press is offered only while its action is on; the field stays
/// enabled (it edits), and the VoiceOver path of task-60 (focus, type,
/// press) finds the press once the text turns the `submit` on.
const OFFERED_DISABLED: &[&str] = &[];

#[test]
fn every_action_the_tree_offers_is_a_request_the_dispatcher_admits() {
    let mut refused = Vec::new();
    for seen in projecting() {
        let title = seen.view.page_title();
        for (at, id, n) in seen.walk() {
            for action in OFFERED.into_iter().filter(|a| n.supports_action(*a)) {
                let req = ActionRequest {
                    action,
                    target_tree: TreeId::ROOT,
                    target_node: at,
                    data: (action == Action::SetValue).then(|| same_value(n)),
                };
                match accessibility::translate(&req, &seen.shown) {
                    Ok(Assist::Request(r)) => {
                        if !admitted(title, &seen, &r) {
                            refused.push(format!("{title}: {} {}", r.target, r.action));
                        }
                    }
                    Ok(Assist::Focus(to)) => assert_eq!(&to, id, "{title}"),
                    Err(e) => panic!("{title}: {id} offers {action:?} and it maps to nothing: {e}"),
                }
            }
        }
    }
    refused.sort();
    assert_eq!(refused, OFFERED_DISABLED);
}

/// The request is the reader's, on the node it names, of an action the
/// node declares and opens to assistive technology; `false` when the
/// action is disabled on a node the tree tells enabled.
fn admitted(title: &str, seen: &Seen, r: &Request) -> bool {
    assert_eq!(r.actor, Actor::Assistive);
    let node = seen.sem(&r.target);
    let Some(decl) = node.actions.iter().find(|d| d.name == r.action) else {
        panic!("{title}: {} does not declare {}", r.target, r.action);
    };
    assert!(
        decl.policy.actors.assistive,
        "{title}: {} {} is offered to a reader and refused to one",
        r.target, r.action
    );
    decl.enabled || told_disabled(seen, &r.target)
}

/// Whether the tree tells the reader the node is disabled.
fn told_disabled(seen: &Seen, id: &SemId) -> bool {
    seen.shown
        .ids
        .node(id)
        .and_then(|n| seen.node(n))
        .is_some_and(AkNode::is_disabled)
}

/// The actions the pages declare that no action of the tree reaches. Phase
/// 1 listed 22: a row's `remove` beside its `toggle`, the three actions
/// of a cut, a select's `open`, `close` and `select`, and the file drop.
/// Phase 2a reaches all but the drop: a row's and a cut's actions are
/// controls the tree presents inside them, a select is pressed open (and
/// shut, while open) and set by its value. The drop stays: a reader does
/// not drag a file, and `files.open` reaches the same files. Phase 2b
/// adds the Resizable page's split: a group whose `set_ratio` no action of
/// the tree reaches (a value is set through `set_value` or a select's
/// `select`, and the contract has no splitter role, AccessKit's
/// `Splitter`); an agent asks it, and a person drags. Task-104 put a
/// stacked split beside it, the same.
const UNREACHABLE: &[&str] = &[
    "Files: files.drop drop_files",
    "Resizable: resizable_page.group[1].card[0] set_ratio",
    "Resizable: resizable_page.split set_ratio",
];

/// Whether the node's press is off now: its primary action, or the node,
/// is disabled, so the tree does not offer the press until it is on (a
/// field with no text, a propose with no intent). The press is there the
/// moment the action is.
fn off(node: &accord::Node) -> bool {
    accessibility::primary(node).is_some_and(|d| !d.enabled || !node.enabled)
}

/// The node with its list `expanded` the other way: a select's press is
/// `open` while shut and `close` while open, so both are reached by it.
fn flipped(node: &accord::Node) -> accord::Node {
    let mut other = node.clone();
    let open = matches!(node.state.get("expanded"), Some(Value::Bool(true)));
    if let Ok(key) = StateKey::parse("expanded") {
        other.state.insert(key, Value::Bool(!open));
    }
    other
}

#[test]
fn the_actions_no_reader_can_reach_are_the_known_ones() {
    let mut unreachable = Vec::new();
    for seen in projecting() {
        let title = seen.view.page_title();
        for (_, id, n) in seen.walk() {
            if seen.presented(id).is_some() {
                continue;
            }
            let node = seen.sem(id);
            let other = flipped(node);
            let mut reached: Vec<&str> = [
                (
                    Action::Click,
                    accessibility::primary(node).map(|d| d.name.as_str()),
                ),
                (
                    Action::Click,
                    accessibility::primary(&other).map(|d| d.name.as_str()),
                ),
                (
                    Action::SetValue,
                    accessibility::setter(node).map(|d| d.name.as_str()),
                ),
                (Action::Increment, Some("increment")),
                (Action::Decrement, Some("decrement")),
            ]
            .into_iter()
            .filter(|(a, _)| n.supports_action(*a) || *a == Action::Click && off(node))
            .filter_map(|(_, name)| name)
            .collect();
            // the controls the tree presents inside it, each its action
            for child in n.children() {
                let owned = seen
                    .shown
                    .ids
                    .sem(*child)
                    .and_then(|c| seen.presented(c))
                    .filter(|(item, _)| *item == id);
                if let Some((_, action)) = owned {
                    reached.push(action);
                }
            }
            for decl in &node.actions {
                if !reached.contains(&decl.name.as_str()) {
                    unreachable.push(format!("{title}: {id} {}", decl.name));
                }
            }
        }
    }
    unreachable.sort();
    assert_eq!(unreachable, UNREACHABLE);
}

/// Every status line of every page is a polite live region, and an
/// alert an assertive one, that says its words as its value (what macOS
/// announces) and as its name (what Windows and AT-SPI announce): the
/// message a status keeps (`silences.status`, `surface_page.status`, the
/// Files and Capture lines), or its label (the todo's counter, the
/// Silences counts).
#[test]
fn every_status_line_is_a_live_region_that_says_its_words() {
    let mut lines = Vec::new();
    for seen in projecting() {
        let title = seen.view.page_title();
        for (_, id, n) in seen.walk() {
            if seen.presented(id).is_some() {
                assert_eq!(n.live(), None, "{title}: {id} is not live");
                continue;
            }
            let node = seen.sem(id);
            let politeness = match node.role {
                Role::Status => Live::Polite,
                Role::Alert => Live::Assertive,
                _ => {
                    assert_eq!(n.live(), None, "{title}: {id} is not live");
                    continue;
                }
            };
            assert_eq!(n.live(), Some(politeness), "{title}: {id}");
            let words = match node.state.get("message") {
                Some(Value::Text(m)) if !m.is_empty() => Some(m.as_str()),
                Some(_) => None,
                None => Some(node.label.as_str()),
            };
            assert_eq!(n.value(), words, "{title}: {id} says its words");
            // the name too: Windows and AT-SPI announce the name (phase 2a)
            let name = words.unwrap_or(node.label.as_str());
            assert_eq!(n.label(), Some(name), "{title}: {id} is named by its words");
            lines.push(id.to_string());
        }
    }
    lines.sort();
    lines.dedup();
    assert_eq!(
        lines,
        [
            // task-104 front C: the inline alerts, by intent (#156)
            "alert_page.group[0].card[0].alert[0]",
            "alert_page.group[0].card[1].alert[0]",
            "alert_page.group[0].card[2].alert[0]",
            "alert_page.group[0].card[3].alert[0]",
            "alert_page.group[1].card[0].alert[0]",
            "alert_page.group[1].card[1].alert[0]",
            "alert_page.group[1].card[2].alert[0]",
            "alert_page.group[1].card[3].alert[0]",
            "alert_page.group[2].card[0].alert[0]",
            "alert_page.group[2].card[0].alert[1]",
            "alert_page.group[2].card[0].alert[2]",
            "alert_page.group[2].card[0].alert[3]",
            "alert_page.group[3].card[0].alert[0]",
            "alert_page.group[3].card[1].alert[0]",
            "capture.status",
            "files.status",
            "silences.counts",
            "silences.status",
            "silences_surface.status",
            "surface_page.status",
            "todo.counter",
        ]
    );
}

/// The focus a reader leaves on a control the page draws no focus of its
/// own for (a filter tab, a checkbox of the quick todo, the proposer, the
/// detect button) is the page's until something else takes it: Escape
/// lets it go, a click of the person moves it where the click went.
#[test]
fn the_focus_a_reader_leaves_goes_with_escape_and_with_a_click() {
    for (page, id) in [
        ("Todo", "todo.filter[active]"),
        ("Surface", "todo_panel.filter.active"),
        ("Surface", "todo_panel.item[2].done"),
        ("Surface", "surface_page.proposer"),
        ("Silences", "silences.detect"),
        ("Silences", "silences_surface.proposer"),
    ] {
        let sid = |t: &str| SemId::parse(t).unwrap_or_else(|e| panic!("{t}: {e}"));
        let focused = |view: &ShowcaseView| {
            view.semantics()
                .and_then(|s| s.focused().map(ToString::to_string))
        };
        let mut seen = see(page);
        assert!(seen.view.assist_focus(&sid(id)), "{page}: {id}");
        assert_eq!(
            focused(&seen.view).as_deref(),
            Some(id),
            "{page}: projected"
        );
        assert!(seen.view.close_top_overlay(), "{page}: Escape lets it go");
        assert_eq!(focused(&seen.view), None, "{page}: {id} after Escape");

        assert!(seen.view.assist_focus(&sid(id)));
        let mut c = Compositor::new();
        seen.view.render(&mut c);
        let field = match page {
            "Silences" => "silences_surface.intent",
            "Surface" => "surface_page.intent",
            _ => "todo.input",
        };
        let Some([x, y, w, h]) = c.binding(&sid(field)).map(|b| b.rect) else {
            panic!("{page}: {field} is bound");
        };
        let (x, y) = (x + w / 2.0, y + h / 2.0);
        seen.view
            .handle_event(&engine::widgets::prelude::WidgetEvent::MouseDown { x, y });
        seen.view
            .handle_event(&engine::widgets::prelude::WidgetEvent::MouseUp { x, y });
        assert_eq!(
            focused(&seen.view).as_deref(),
            Some(field),
            "{page}: the click moved it to the field"
        );
    }
}

/// The roles phase 2a set right: the todo row holds a checkbox (the
/// `toggle` presented by the tree, the row neither toggled nor pressed),
/// each item of the quick todo is named by its text and not by its key,
/// and the Capture page's level bar says its value.
#[test]
fn the_row_holds_a_checkbox_the_quick_todo_says_its_items_and_the_level_its_value() {
    let seen = see("Surface");
    let node = |id: &str| {
        let sid = SemId::parse(id).unwrap_or_else(|e| panic!("{id}: {e}"));
        let at = seen.shown.ids.node(&sid);
        match at.and_then(|n| seen.node(n)) {
            Some(n) => n,
            None => panic!("{id} is not in the tree"),
        }
    };
    let row = node("todo.item[1]");
    assert_eq!((row.role(), row.toggled()), (AkRole::ListItem, None));
    assert!(!row.supports_action(Action::Click));
    let check = node("todo.item[1][toggle]");
    assert_eq!(check.role(), AkRole::CheckBox);
    assert!(check.toggled().is_some());
    for (key, text) in [
        ("1", "Absorb the plev demos into the showcase"),
        ("2", "Measure text, never estimate it"),
        ("3", "Port the todo domain (tested first)"),
    ] {
        let item = node(&format!("todo_panel.item[{key}]"));
        assert_eq!(item.label(), Some(text), "todo_panel.item[{key}]");
    }

    let seen = see("Capture");
    let sid = SemId::parse("capture.level").unwrap_or_else(|e| panic!("{e}"));
    let Some(level) = seen.shown.ids.node(&sid).and_then(|n| seen.node(n)) else {
        panic!("capture.level is not in the tree");
    };
    assert_eq!(level.role(), AkRole::ProgressIndicator);
    assert_eq!(level.numeric_value(), Some(0.0), "nothing captured yet");
    assert_eq!(
        (level.min_numeric_value(), level.max_numeric_value()),
        (Some(0.0), Some(1.0))
    );
}

/// A reader writes an intent the rules do not understand and presses
/// propose, all through the tree: the status line says why, as its value
/// (macOS) and as its name (Windows and AT-SPI announce the name), and the
/// focus stays on the field.
#[test]
fn a_status_says_its_new_message_as_its_name_and_its_value() {
    let sid = |t: &str| SemId::parse(t).unwrap_or_else(|e| panic!("{t}: {e}"));
    let ask = |seen: &Seen, id: &str, action: Action, data: Option<ActionData>| {
        let req = ActionRequest {
            action,
            target_tree: TreeId::ROOT,
            target_node: seen.shown.ids.node(&sid(id)).unwrap_or(accessibility::ROOT),
            data,
        };
        match accessibility::translate(&req, &seen.shown) {
            Ok(Assist::Request(r)) => r,
            other => panic!("{id} {action:?}: {other:?}"),
        }
    };
    let mut seen = see("Surface");
    assert!(seen.view.assist_focus(&sid("surface_page.intent")));
    let words = Some(ActionData::Value("xyzzy plugh".into()));
    let req = ask(&seen, "surface_page.intent", Action::SetValue, words);
    assert!(seen.view.act(req).is_some());
    let mut seen = look(seen.view);
    let req = ask(&seen, "surface_page.propose", Action::Click, None);
    assert_eq!(req.action.as_str(), "propose");
    assert!(seen.view.act(req).is_some());

    let seen = look(seen.view);
    let message = match seen.sem(&sid("surface_page.status")).state.get("message") {
        Some(Value::Text(m)) if !m.is_empty() => m.clone(),
        other => panic!("the status says why: {other:?}"),
    };
    let status = seen
        .shown
        .ids
        .node(&sid("surface_page.status"))
        .and_then(|n| seen.node(n));
    let Some(status) = status else {
        panic!("the status is in the tree");
    };
    assert_eq!(
        status.label(),
        Some(message.as_str()),
        "named by its message"
    );
    assert_eq!(status.value(), Some(message.as_str()));
    assert_eq!(status.live(), Some(Live::Polite));
    assert_eq!(
        Some(seen.update.focus),
        seen.shown.ids.node(&sid("surface_page.intent")),
        "the focus stays on the field"
    );
}
