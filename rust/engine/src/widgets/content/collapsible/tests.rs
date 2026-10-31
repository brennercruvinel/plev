//! The Dropdown's parts of the collapsible: the counter beside the label
//! and the list it reveals (task-104d, the Collapsible tab).

use accord::{Role, SemId, SnapshotBuilder, Value};

use super::{Collapsible, DisclosureList};
use crate::drawing::compositor::{Compositor, LayerId, SceneNode};
use crate::styling::theme::Theme;
use crate::widgets::widget::{Rect, WidgetEvent};

fn core() -> Theme {
    Theme::hoff_core()
}

fn list() -> DisclosureList {
    DisclosureList::new(["Designs", "Animations"])
        .current(0)
        .sem(SemId::parse("d.links").unwrap())
}

#[test]
fn a_counter_keeps_the_trigger_s_height_and_takes_room_from_the_label() {
    let theme = core();
    let plain = Collapsible::new("Assets").icon("folder");
    let counted = Collapsible::new("Assets").icon("folder").counter("112");
    assert_eq!(counted.trigger_height(240.0, &theme), 40.0);
    let long = "Assets imported from the shared library";
    let narrow = |c: Collapsible| c.trigger_height(170.0, &theme);
    assert!(
        narrow(Collapsible::new(long).icon("folder").counter("112"))
            >= narrow(Collapsible::new(long).icon("folder")),
        "the counter leaves the label less room"
    );
    assert_eq!(plain.trigger_height(240.0, &theme), 40.0);
}

#[test]
fn the_list_is_as_tall_as_its_links_and_a_long_one_wraps() {
    let theme = core();
    let rail = theme.disclosure.collapsible.rail.unwrap();
    assert_eq!(list().height(240.0, &theme), rail.link_h * 2.0);
    let rects = list().link_rects(Rect::new(0.0, 10.0, 240.0, 0.0), &theme);
    assert_eq!(rects[0].y, 10.0);
    assert_eq!(rects[1].y, rects[0].bottom());
    let long = DisclosureList::new(["Materials imported from the shared library of the studio"]);
    assert!(long.height(120.0, &theme) > rail.link_h);
}

#[test]
fn a_click_on_a_link_makes_it_current_and_a_disabled_list_ignores_it() {
    let theme = core();
    let bounds = Rect::new(0.0, 0.0, 240.0, 80.0);
    let mut l = list();
    let (x, y) = l.link_rects(bounds, &theme)[1].center();
    assert!(
        l.handle_event(&WidgetEvent::MouseMove { x, y }, bounds, &theme)
            .changed
    );
    l.handle_event(&WidgetEvent::MouseDown { x, y }, bounds, &theme);
    assert!(
        l.handle_event(&WidgetEvent::MouseUp { x, y }, bounds, &theme)
            .clicked
    );
    assert_eq!(l.current_link(), Some(1));
    l.handle_event(&WidgetEvent::MouseDown { x, y }, bounds, &theme);
    let r = l.handle_event(&WidgetEvent::MouseUp { x, y: y + 500.0 }, bounds, &theme);
    assert!(!r.clicked, "released away: nothing");
    let mut off = list().disabled(true);
    off.handle_event(&WidgetEvent::MouseDown { x, y }, bounds, &theme);
    off.handle_event(&WidgetEvent::MouseUp { x, y }, bounds, &theme);
    assert_eq!(off.current_link(), Some(0));
    assert!(!off.activate(1) && !list().activate(9));
}

#[test]
fn each_link_projects_as_a_link_with_active_and_binds_where_it_draws() {
    let theme = core();
    let l = list();
    let mut b = SnapshotBuilder::new();
    l.project(&mut b, None);
    let seen = b.finish().unwrap();
    let first = l.link_id(0).unwrap();
    let node = seen.get(&first).unwrap();
    assert_eq!(node.role, Role::Link);
    assert_eq!(node.state.get("active"), Some(&Value::Bool(true)));
    assert!(node.actions.iter().any(|a| a.name.as_str() == "activate"));
    assert_eq!(l.position(&first), Some(0));

    let mut c = Compositor::new();
    c.begin_frame();
    let bounds = Rect::new(0.0, 0.0, 240.0, 80.0);
    l.render(&mut c, bounds, &theme);
    let at = l.link_rects(bounds, &theme)[1];
    assert_eq!(
        c.binding(&l.link_id(1).unwrap()).unwrap().rect,
        [at.x, at.y, at.w, at.h]
    );
    let nodes = c.layer(LayerId::DEFAULT).unwrap().nodes().to_vec();
    let rail = theme.disclosure.collapsible.rail.unwrap();
    assert!(
        nodes
            .iter()
            .any(|n| matches!(n, SceneNode::Rect { w, .. } if *w == rail.width)),
        "the rail is drawn"
    );
}
