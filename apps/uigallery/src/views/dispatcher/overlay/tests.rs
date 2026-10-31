//! The view's overlays as a screen reader and an agent meet them (task-75
//! phase 2a): the modal a dialog with its buttons and the focus, the
//! context menu a menu of items, the toasts statuses a reader or an agent
//! can take away; each answered through the gallery's one dispatcher, the
//! person's click too.

use std::sync::{Arc, Mutex};

use accord::{Actor, Receipt, ReceiptSink, Request, SemId, Stage};
use engine::control::accessibility::accesskit::{
    Action, ActionRequest, Live, Node, Role as AkRole, TreeId, TreeUpdate,
};
use engine::control::accessibility::{self, Assist, Chrome, Shown};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::Intent;
use engine::widgets::prelude::WidgetEvent;

use super::super::reader::NOTICE;
use super::{DIALOG, MENU, TOASTS, id};
use crate::views::{ActiveOverlay, ShowcaseView};

const CHROME: Chrome<'static> = Chrome {
    title: "plev, design system showcase",
    scale: 2.0,
    size: (2560, 1600),
};

fn sid(text: &str) -> SemId {
    match SemId::parse(text) {
        Ok(id) => id,
        Err(e) => panic!("{text}: {e}"),
    }
}

fn view_on(page: &str) -> ShowcaseView {
    let mut view = ShowcaseView::new(1280.0, 800.0);
    view.jump_to_page(page);
    view
}

/// What the bridge shows a reader this frame, and its tree.
fn seen(view: &mut ShowcaseView) -> (TreeUpdate, Shown) {
    let mut c = Compositor::new();
    view.render(&mut c);
    let snapshot = view.reader_semantics().unwrap_or_default();
    let (update, ids) = accessibility::build(&snapshot, &c, &CHROME);
    (update, Shown { snapshot, ids })
}

fn node<'a>(u: &'a TreeUpdate, shown: &Shown, id: &str) -> &'a Node {
    let want = shown.ids.node(&sid(id));
    match u.nodes.iter().find(|(n, _)| Some(*n) == want) {
        Some((_, n)) => n,
        None => panic!("{id} is not in the tree"),
    }
}

/// A reader presses `id`: the request the tree reads back.
fn press(shown: &Shown, id: &str) -> Request {
    let req = ActionRequest {
        action: Action::Click,
        target_tree: TreeId::ROOT,
        target_node: shown.ids.node(&sid(id)).unwrap_or(accessibility::ROOT),
        data: None,
    };
    match accessibility::translate(&req, shown) {
        Ok(Assist::Request(r)) => r,
        other => panic!("{id}: {other:?}"),
    }
}

/// Every request the gallery's dispatcher records: who, what, on what.
type Heard = Arc<Mutex<Vec<(Actor, String, String)>>>;

struct Ear(Heard);

impl ReceiptSink for Ear {
    fn on_stage(&mut self, receipt: &Receipt, stage: &Stage) {
        if let Stage::Completed(_) = stage
            && let Ok(mut heard) = self.0.lock()
        {
            heard.push((
                receipt.request.actor.clone(),
                receipt.request.target.to_string(),
                receipt.request.action.to_string(),
            ));
        }
    }
}

fn ear(view: &mut ShowcaseView) -> Heard {
    let heard = Heard::default();
    view.add_sink(Box::new(Ear(heard.clone())));
    heard
}

fn heard(h: &Heard) -> Vec<(Actor, String, String)> {
    h.lock().map(|v| v.clone()).unwrap_or_default()
}

#[test]
fn the_overlays_ids_parse() {
    for name in [TOASTS, DIALOG, MENU] {
        assert!(id(name).is_some(), "{name}");
    }
    assert!(SemId::parse(NOTICE).is_ok());
}

/// The acceptance: with the modal open over the todo page, a reader is in
/// a dialog named by its title and described by its body, on its cancel;
/// the page is hidden; cancel closes it and the focus is the page's
/// again; confirm closes it and the toast it pushes is heard.
#[test]
fn the_modal_is_a_dialog_a_reader_answers_and_the_focus_comes_back() {
    let mut view = view_on("todo");
    let ear = ear(&mut view);
    assert!(view.assist_focus(&sid("todo.input")));
    view.open_modal(false);
    let (u, shown) = seen(&mut view);
    let dialog = node(&u, &shown, "dialog");
    assert_eq!(dialog.role(), AkRole::Dialog);
    assert_eq!(dialog.label(), Some("Apply settings?"));
    assert!(dialog.description().is_some_and(|d| d.contains("reload")));
    let ids = |texts: &[&str]| -> Vec<_> {
        texts
            .iter()
            .filter_map(|t| shown.ids.node(&sid(t)))
            .collect()
    };
    assert_eq!(
        dialog.children(),
        ids(&["dialog[confirm]", "dialog[cancel]"])
    );
    assert_eq!(node(&u, &shown, "dialog[confirm]").label(), Some("Apply"));
    assert_eq!(Some(u.focus), shown.ids.node(&sid("dialog[cancel]")));
    assert!(node(&u, &shown, "todo").is_hidden(), "the page is inert");

    // a reader moves to confirm: the dialog keeps it
    assert!(view.assist_focus(&sid("dialog[confirm]")));
    let (u, shown) = seen(&mut view);
    assert_eq!(Some(u.focus), shown.ids.node(&sid("dialog[confirm]")));
    assert!(!view.assist_focus(&sid("todo.input")), "the page is inert");

    // cancel: the dialog leaves, the page and its focus come back
    let (_, shown) = seen(&mut view);
    let req = press(&shown, "dialog[cancel]");
    let receipt = view.act(req);
    assert!(matches!(
        receipt.as_ref().and_then(Receipt::stage),
        Some(Stage::Completed(_))
    ));
    let (u, shown) = seen(&mut view);
    assert_eq!(shown.ids.node(&sid("dialog")), None);
    assert!(!node(&u, &shown, "todo").is_hidden());
    assert_eq!(Some(u.focus), shown.ids.node(&sid("todo.input")));

    // confirm: the toast is heard, polite, and nothing takes the focus
    view.open_modal(false);
    let (_, shown) = seen(&mut view);
    let req = press(&shown, "dialog[confirm]");
    assert!(view.act(req).is_some());
    let (u, shown) = seen(&mut view);
    let toast = node(&u, &shown, "toasts[0]");
    assert_eq!(toast.live(), Some(Live::Polite));
    assert_eq!(toast.value(), Some("Settings applied."));
    assert_eq!(Some(u.focus), shown.ids.node(&sid("todo.input")));
    assert_eq!(
        heard(&ear),
        [
            (Actor::Assistive, "dialog[cancel]".into(), "activate".into()),
            (
                Actor::Assistive,
                "dialog[confirm]".into(),
                "activate".into()
            ),
        ]
    );
}

/// A person's click on the modal's button is the same request, the
/// person's, through the same dispatcher.
#[test]
fn a_persons_click_on_the_modal_is_a_request_too() {
    let mut view = view_on("dialog");
    let ear = ear(&mut view);
    view.open_modal(true);
    let Some(ActiveOverlay::Modal { widget, .. }) = &view.active_overlay else {
        panic!("the modal is open");
    };
    let dialog = widget.dialog_rect(&view.theme, view.width, view.height);
    let (confirm, _) = widget.button_rects(&view.theme, dialog);
    let (x, y) = confirm.center();
    view.handle_event(&WidgetEvent::MouseDown { x, y });
    view.handle_event(&WidgetEvent::MouseUp { x, y });
    assert_eq!(
        heard(&ear),
        [(Actor::Human, "dialog[confirm]".into(), "activate".into())]
    );
    let toasts: Vec<(&str, Intent)> = view
        .toasts
        .visible()
        .map(|t| (t.message.as_str(), t.intent))
        .collect();
    assert_eq!(
        toasts,
        [("File deleted (not really).", Intent::Destructive)]
    );
    assert!(!view.overlay_holds_pointer(), "the modal leaves");
}

/// The Context Menu page's menu, the kit's file menu: a menu of items,
/// each described by its keys, the destructive one's pick so declared,
/// the focus on the first; a reader moves to an item and picks it.
#[test]
fn the_context_menu_is_a_menu_a_reader_picks_from() {
    let mut view = view_on("context menu");
    let ear = ear(&mut view);
    let _ = seen(&mut view);
    let area = view.panes.context_menu.area(view.page_rect(), &view.theme);
    assert!(view.handle_right_click(area.x + area.w / 2.0, area.y + area.h / 2.0));
    let (u, shown) = seen(&mut view);
    let menu = node(&u, &shown, "context_menu");
    assert_eq!(
        (menu.role(), menu.label()),
        (AkRole::Menu, Some("context menu"))
    );
    assert_eq!(
        node(&u, &shown, "context_menu[2]").label(),
        Some("Create new scene...")
    );
    assert_eq!(node(&u, &shown, "context_menu[2]").role(), AkRole::MenuItem);
    assert_eq!(
        node(&u, &shown, "context_menu[2]").description(),
        Some("⌘N")
    );
    assert!(!node(&u, &shown, "context_menu[6]").is_disabled());
    assert!(node(&u, &shown, "context_menu[6]").supports_action(Action::Click));
    assert_eq!(Some(u.focus), shown.ids.node(&sid("context_menu[1]")));

    assert!(view.assist_focus(&sid("context_menu[2]")));
    let (u, shown) = seen(&mut view);
    assert_eq!(Some(u.focus), shown.ids.node(&sid("context_menu[2]")));
    let req = press(&shown, "context_menu[2]");
    assert!(view.act(req).is_some());
    let (u, shown) = seen(&mut view);
    assert_eq!(
        shown.ids.node(&sid("context_menu")),
        None,
        "picked, it leaves"
    );
    assert_eq!(
        node(&u, &shown, "toasts[0]").value(),
        Some("Context menu: Create new scene...")
    );
    assert_eq!(
        heard(&ear),
        [(
            Actor::Assistive,
            "context_menu[2]".into(),
            "activate".into()
        )]
    );
}

/// A toast is the view's projection: an agent through the port reads it
/// and may take it away, as a reader may; it is gone from the next
/// projection. The menu bar leaves the toasts out.
#[test]
fn a_toast_is_read_and_dismissed_by_an_agent_as_by_a_reader() {
    let mut view = view_on("todo");
    let theme = view.theme.clone();
    view.toasts.push("saved", Intent::Constructive, &theme);
    let gallery = view.gallery_semantics().unwrap_or_else(|e| panic!("{e}"));
    let toast = gallery.get(&sid("toasts[0]"));
    assert_eq!(toast.map(|n| n.role), Some(accord::Role::Status));
    assert!(view.is_overlay(&sid("toasts[0]")) && view.is_overlay(&sid("toasts")));
    let agent = Actor::Agent {
        name: "tester".to_owned(),
    };
    let Ok(req) = Request::observe(&gallery, agent, "toasts[0]", "dismiss", None) else {
        panic!("the toast is dismissed");
    };
    let receipt = view.act(req);
    assert!(matches!(
        receipt.as_ref().and_then(Receipt::stage),
        Some(Stage::Completed(_))
    ));
    assert_eq!(
        view.semantics()
            .and_then(|s| s.get(&sid("toasts")).cloned()),
        None
    );
}
