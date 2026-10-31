//! Graph page tests: the canvas a reader hears, a node's `select` and
//! its click, the focus ring, what is refused, and the width sweep.

use accord::{Request, Role, SemId, SnapshotBuilder, Value};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;
use engine::widgets::prelude::{Rect, WidgetEvent};

use super::{After, Component, GraphPage};

fn sid(text: &str) -> SemId {
    SemId::parse(text).expect("the id parses")
}

fn seen(page: &GraphPage) -> accord::Snapshot {
    let root = page.root.clone().expect("the root parses");
    let mut b = SnapshotBuilder::new();
    b.node(&root, Role::Region, "Graph");
    page.project(&mut b, &root);
    b.finish().expect("the page projects a valid tree")
}

fn ask(page: &mut GraphPage, target: &str, action: &str) -> Result<After, String> {
    let req = Request {
        actor: accord::Actor::Test {
            name: "graph".into(),
        },
        target: sid(target),
        action: accord::ActionName::parse(action).expect("an action name"),
        param: None,
        observed: accord::Observation {
            revision: seen(page).revision(),
            values: Vec::new(),
        },
        ask: None,
    };
    page.run(&req)
}

const CANVAS: &str = "graph_page.group[0].card[0]";

fn content() -> Rect {
    Rect::new(288.0, 118.0, 952.0, 682.0)
}

/// Every canvas is a canvas with its selection, over a list item per
/// node that offers `select`; one of them is a stop of Tab.
#[test]
fn every_canvas_projects_its_nodes_and_selection() {
    let page = GraphPage::new();
    let snap = seen(&page);
    let canvas = snap.get(&sid(CANVAS)).expect("the canvas");
    assert_eq!(
        (canvas.role, canvas.label.as_str()),
        (Role::Canvas, "Notes and their links")
    );
    assert_eq!(
        canvas.state.get("selected"),
        Some(&Value::Text(String::new()))
    );
    let nodes: Vec<_> = snap.children(&canvas.id).collect();
    assert_eq!(nodes.len(), 12);
    assert!(nodes.iter().all(|n| n.role == Role::ListItem));
    assert!(
        nodes
            .iter()
            .all(|n| n.actions.iter().any(|a| a.name.as_str() == "select"))
    );
    assert_eq!(nodes.iter().filter(|n| n.focusable).count(), 1);
    let other = snap.get(&sid("graph_page.group[1].card[0]")).unwrap();
    assert_eq!(snap.children(&other.id).count(), 8);
}

/// `select` selects the node, as the click does; a node not there, or
/// any other action, is refused.
#[test]
fn select_takes_the_node_and_the_rest_is_refused() {
    let mut page = GraphPage::new();
    assert!(ask(&mut page, &format!("{CANVAS}[3]"), "select").is_ok());
    assert_eq!(page.groups[0].cards[0].view.selected(), Some(3));
    let snap = seen(&page);
    let canvas = snap.get(&sid(CANVAS)).unwrap();
    assert_eq!(canvas.state.get("selected"), Some(&Value::Text("3".into())));
    assert!(ask(&mut page, &format!("{CANVAS}[30]"), "select").is_err());
    assert!(ask(&mut page, &format!("{CANVAS}[3]"), "activate").is_err());
    assert!(ask(&mut page, CANVAS, "select").is_err());
}

/// A person's click on a node, where it is drawn, selects it as the
/// request does.
#[test]
fn a_click_on_a_node_selects_it() {
    for theme in [Theme::hoff_core(), Theme::hoff()] {
        let mut page = GraphPage::new();
        let mut c = Compositor::new();
        page.render(&mut c, content(), &theme);
        let [x, y, w, h] = c.binding(&sid(&format!("{CANVAS}[5]"))).unwrap().rect;
        let (cx, cy) = (x + w / 2.0, y + h / 2.0);
        page.handle_event(&WidgetEvent::MouseDown { x: cx, y: cy }, content(), &theme);
        let r = page.handle_event(&WidgetEvent::MouseUp { x: cx, y: cy }, content(), &theme);
        assert!(r.clicked);
        assert_eq!(page.groups[0].cards[0].view.selected(), Some(5));
    }
}

/// The focus a reader brings rings its node on its canvas alone; blur
/// lets it go; an id off the page is not the page's.
#[test]
fn the_focus_rings_one_node() {
    let mut page = GraphPage::new();
    assert!(page.focus(&sid("graph_page.group[1].card[0][2]")));
    assert_eq!(page.groups[1].cards[0].view.focused(), Some(2));
    assert_eq!(page.groups[0].cards[0].view.focused(), None);
    assert!(page.blur());
    assert_eq!(page.groups[1].cards[0].view.focused(), None);
    assert!(!page.focus(&sid("chart_page.group[0]")));
}

/// The width sweep of the epic, 280 to 1600 every 8 px, under the core
/// and the hoff themes: every card inside the content and above its
/// rule, every canvas inside its card and below its chip, and every node
/// it binds inside its canvas.
#[test]
fn nothing_cuts_or_overlaps_from_280_to_1600() {
    for theme in [Theme::hoff_core(), Theme::hoff()] {
        for w in (280..=1600).step_by(8) {
            let mut page = GraphPage::new();
            let content = Rect::new(0.0, 0.0, w as f32, 800.0);
            let mut c = Compositor::new();
            page.render(&mut c, content, &theme);
            let mut prev_rule = content.y;
            for (group, at) in page.groups.iter().zip(page.layout(content, &theme)) {
                assert!(at.title_y >= prev_rule, "{w}: groups in order");
                prev_rule = at.rule_y;
                for (card, r) in group.cards.iter().zip(&at.cards) {
                    assert!(
                        r.x >= content.x && r.right() <= content.right() + 0.5,
                        "{w}"
                    );
                    assert!(r.bottom() <= at.rule_y, "{w}: card above its rule");
                    let v = GraphPage::canvas_rect(*r, &theme);
                    let chip = theme.spacing.sm * 2.0 + theme.spacing.xs * 2.0;
                    assert!(v.x >= r.x && v.right() <= r.right() + 0.5, "{w}");
                    assert!(v.y >= r.y + chip && v.bottom() <= r.bottom(), "{w}");
                    let id = card.view.sem_id().unwrap();
                    for i in 0..card.labels.len() {
                        let b = c.binding(&id.key(i.to_string())).map(|b| b.rect);
                        let [x, y, bw, bh] = b.unwrap_or_else(|| panic!("{w}: node {i}"));
                        let (cx, cy) = (x + bw / 2.0, y + bh / 2.0);
                        assert!(v.contains(cx, cy), "{w}: node {i} on its canvas");
                    }
                }
            }
        }
    }
}
