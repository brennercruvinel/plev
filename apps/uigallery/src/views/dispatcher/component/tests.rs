//! The mechanism of the component pages (task-75 phase 2b): every page in
//! the table projects under a region named by its title, is inert while an
//! overlay holds the pointer, and runs what is asked of it through the
//! gallery's one dispatcher, a reader's request and an agent's alike.

use std::sync::{Arc, Mutex};

use sem::{Actor, Receipt, ReceiptSink, Rejection, Request, Role, SemId, Stage};

use super::under;
use crate::view::{PAGES, ShowcaseView};

fn sid(text: &str) -> SemId {
    match SemId::parse(text) {
        Ok(id) => id,
        Err(e) => panic!("{text}: {e}"),
    }
}

fn view_on(page: &str) -> ShowcaseView {
    let mut view = ShowcaseView::new(1280.0, 800.0);
    view.jump_to_page(page);
    assert_eq!(view.page_title(), page);
    view
}

/// Every request the dispatcher took, with how it ended.
type Heard = Arc<Mutex<Vec<(Actor, String, String, &'static str)>>>;

struct Ear(Heard);

impl ReceiptSink for Ear {
    fn on_stage(&mut self, receipt: &Receipt, stage: &Stage) {
        let end = match stage {
            Stage::Completed(_) => "completed",
            Stage::Rejected(_) => "rejected",
            Stage::Failed { .. } => "failed",
            _ => return,
        };
        if let Ok(mut heard) = self.0.lock() {
            heard.push((
                receipt.request.actor.clone(),
                receipt.request.target.to_string(),
                receipt.request.action.to_string(),
                end,
            ));
        }
    }
}

fn ear(view: &mut ShowcaseView) -> Heard {
    let heard = Heard::default();
    view.add_sink(Box::new(Ear(heard.clone())));
    heard
}

fn heard(h: &Heard) -> Vec<(Actor, String, String, &'static str)> {
    h.lock().map(|v| v.clone()).unwrap_or_default()
}

/// A request of `actor` over what the view projects now.
fn ask(view: &ShowcaseView, actor: Actor, target: &str, action: &str) -> Request {
    let seen = view.gallery_semantics().unwrap_or_default();
    match Request::observe(&seen, actor, target, action, None) {
        Ok(req) => req,
        Err(e) => panic!("{target} {action}: {e}"),
    }
}

/// Every page whose pane is in the table: one region, its root, named by
/// the page's title (an alias by its own), and every other node under it.
#[test]
fn every_component_page_projects_under_a_region_named_by_its_title() {
    let mut pages = 0;
    for page in PAGES {
        let view = view_on(page.title);
        let Some(pane) = page.pane() else {
            continue;
        };
        let Some(component) = view.panes.component(pane) else {
            continue;
        };
        let Some(root) = component.root() else {
            panic!("{}: the root parses", page.title);
        };
        let Some(snapshot) = view.semantics() else {
            panic!("{}: it projects", page.title);
        };
        let roots: Vec<&SemId> = snapshot.roots().map(|n| &n.id).collect();
        assert_eq!(roots, [root], "{}", page.title);
        let region = snapshot.get(root).map(|n| (n.role, n.label.as_str()));
        assert_eq!(region, Some((Role::Region, page.title)));
        for id in snapshot.nodes().keys() {
            assert!(under(root, id), "{}: {id}", page.title);
            assert!(view.is_component(id), "{}: {id}", page.title);
        }
        assert!(!view.is_component(&sid("todo.input")));
        assert!(!view.is_component(&sid("shell.page")));
        pages += 1;
    }
    assert!(pages >= 3, "the table holds the component pages");
}

#[test]
fn under_is_the_root_and_its_children_only() {
    let root = sid("toast_page");
    assert!(under(&root, &root));
    assert!(under(&root, &sid("toast_page.button[error]")));
    assert!(under(&root, &sid("toast_page[1]")));
    assert!(!under(&root, &sid("toast_pages")));
    assert!(!under(&root, &sid("toasts[1]")));
}

/// A reader's press and an agent's act on a component page are requests
/// of the gallery's one dispatcher, heard by its sinks; what the page asks
/// of the view (the toast) is done by the time the receipt comes back.
#[test]
fn a_component_page_s_requests_go_through_the_gallery_s_dispatcher() {
    let mut view = view_on("Toast");
    let heard_by = ear(&mut view);
    let req = ask(
        &view,
        Actor::Assistive,
        "toast_page.button[error]",
        "activate",
    );
    let receipt = view.act(req).expect("the page takes it");
    assert!(matches!(receipt.stage(), Some(Stage::Completed(_))));
    let toasts: Vec<String> = view
        .semantics()
        .unwrap_or_default()
        .nodes()
        .values()
        .filter(|n| n.role == Role::Alert)
        .map(|n| n.label.clone())
        .collect();
    assert_eq!(toasts, ["Failed to push: remote rejected."]);

    let agent = Actor::Agent {
        name: "claude".into(),
    };
    let req = ask(&view, agent.clone(), "toast_page.button[info]", "activate");
    assert!(view.act(req).is_some());
    assert_eq!(
        heard(&heard_by),
        [
            (
                Actor::Assistive,
                "toast_page.button[error]".to_owned(),
                "activate".to_owned(),
                "completed"
            ),
            (
                agent,
                "toast_page.button[info]".to_owned(),
                "activate".to_owned(),
                "completed"
            ),
        ]
    );
}

/// With the modal open the Dialog page is inert: its region says so, a
/// reader's focus is refused, and a request comes back rejected, as for a
/// click, which the modal takes before the page.
#[test]
fn a_component_page_is_inert_while_an_overlay_holds_the_pointer() {
    let mut view = view_on("Dialog");
    let req = ask(&view, Actor::Assistive, "dialog_page.open", "activate");
    assert!(view.act(req).is_some());
    let seen = view.semantics().unwrap_or_default();
    assert!(seen.get(&sid("dialog_page")).is_some_and(|n| n.inert));
    assert!(seen.get(&sid("dialog")).is_some(), "the modal is open");
    assert_eq!(seen.focused(), Some(&sid("dialog[cancel]")));
    assert!(!view.assist_focus(&sid("dialog_page.delete")));

    let heard_by = ear(&mut view);
    let req = ask(&view, Actor::Assistive, "dialog_page.delete", "activate");
    let receipt = view.act(req).expect("the page takes it");
    assert!(matches!(
        receipt.stage(),
        Some(Stage::Rejected(Rejection::Inert { .. }))
    ));
    assert_eq!(heard(&heard_by).len(), 1);
    let dialog = view.semantics().unwrap_or_default();
    let title = dialog.get(&sid("dialog")).map(|n| n.label.clone());
    assert_eq!(title.as_deref(), Some("Apply settings?"), "the plain one");
}
