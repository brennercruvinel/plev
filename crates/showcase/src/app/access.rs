//! The window's outside actors, desktop only: a screen reader (the
//! AccessKit bridge) and agents (the port), and how their threads wake the
//! event loop; and what the screen reader is shown (`shown`).

use sem::{SemId, Snapshot, SnapshotBuilder};
use winit::event_loop::{ActiveEventLoop, EventLoopProxy};
use winit::window::Window;

use super::{TITLE, UserEvent};
use crate::view::ShowcaseView;

/// What the screen reader is shown this frame: the page's meaning
/// ([`ShowcaseView::semantics`], what agents, tests and the baselines
/// read) and, while a toast is on screen, the gallery's toasts as `comps`
/// projects them ([`comps::prelude::ToastManager::project`]: `toasts`, a
/// region, with a `Status` per toast, or an `Alert` for a destructive
/// one, named by its message). The engine makes a status a polite live
/// region and an alert an assertive one, so a reader hears a toast
/// arrive and the focus stays where it is (task-75 phase 1). With no
/// toast on screen the snapshot is the page's, untouched.
///
/// The toasts belong in the view's own projection, where an agent would
/// read them too; until the pages open (phase 2) the shell adds them
/// here, the one place the reader's tree is asked for.
pub(super) fn shown(view: &ShowcaseView) -> Option<Snapshot> {
    let page = view.semantics();
    if view.toasts.visible().all(|t| t.is_closing()) {
        return page;
    }
    let id = match SemId::name("toasts") {
        Ok(id) => id,
        Err(e) => {
            log::warn!("accessibility: the toasts' id: {e}");
            return page;
        }
    };
    let mut b = SnapshotBuilder::new();
    if let Some(page) = &page {
        redeclare(&mut b, page);
    }
    view.toasts.project(&mut b, &id, "notifications", None);
    match b.finish() {
        Ok(snapshot) => Some(snapshot),
        Err(e) => {
            log::warn!("accessibility: the page with its toasts does not validate: {e}");
            page
        }
    }
}

/// Every node of `page` declared again into `b`, with its parent,
/// description, state, constraints, flags, actions and the focus (the
/// view's `detached::extend` does the same for the page's own window;
/// `sem` has no way to carry a finished snapshot into another). What the
/// rebuild loses is each node's `source`, which the reader never sees.
fn redeclare(b: &mut SnapshotBuilder, page: &Snapshot) {
    for node in page.nodes().values() {
        let mut n = b.node(&node.id, node.role, node.label.clone());
        if let Some(parent) = &node.parent {
            n.parent(parent);
        }
        if let Some(text) = &node.description {
            n.description(text.clone());
        }
        for (key, value) in &node.state {
            n.state(key, value.clone());
        }
        n.constraints(node.constraints.clone())
            .enabled(node.enabled)
            .inert(node.inert)
            .focusable(node.focusable);
        for decl in &node.actions {
            let mut a = n.action(&decl.name);
            if let Some(label) = &decl.label {
                a.label(label.clone());
            }
            a.enabled(decl.enabled).policy(decl.policy);
            for at in &decl.reads {
                a.reads(at.clone());
            }
            if let Some(spec) = &decl.param {
                a.param(*spec);
            }
        }
    }
    if let Some(focused) = page.focused() {
        b.focus(focused);
    }
}

/// The window's outside actors (a screen reader, agents through the port)
/// and how their threads wake the event loop.
#[derive(Default)]
pub(super) struct Access {
    pub(super) bridge: Option<engine::accessibility::Bridge>,
    pub(super) agent: Option<wire::AgentPort>,
    pub(super) proxy: Option<EventLoopProxy<UserEvent>>,
    /// The windows' scenes, written for the port's watches (`mirror`).
    pub(super) scenes: super::mirror::Scenes,
}

impl Access {
    /// Creates the bridge for `window`, which is still hidden. Without a
    /// bridge the window still opens, and the reason is logged.
    pub(super) fn attach(&mut self, event_loop: &ActiveEventLoop, window: &Window) {
        let proxy = self.proxy.clone();
        let wake = move || {
            if let Some(proxy) = &proxy {
                let _ = proxy.send_event(UserEvent::Access);
            }
        };
        match engine::accessibility::Bridge::new(event_loop, window, TITLE, wake) {
            Ok(bridge) => self.bridge = Some(bridge),
            Err(e) => log::error!("accessibility: {e}"),
        }
    }

    /// What the screen reader asked, on the main thread: a request goes
    /// through the active page's dispatcher as `Actor::Assistive`, a focus
    /// move to the page's focus.
    pub(super) fn carry_out(&mut self, view: &mut ShowcaseView) {
        let Some(bridge) = self.bridge.as_mut() else {
            return;
        };
        for assist in bridge.take() {
            match assist {
                engine::accessibility::Assist::Request(req) => {
                    let asked = format!("{} {}", req.target, req.action);
                    match view.act(req) {
                        Some(receipt) => {
                            log::debug!("accessibility: {asked}: {:?}", receipt.stage());
                        }
                        None => log::debug!("accessibility: no page takes {asked}"),
                    }
                }
                engine::accessibility::Assist::Focus(id) => {
                    if !view.assist_focus(&id) {
                        log::debug!("accessibility: no page takes focus on {id}");
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use comps::prelude::Intent;
    use engine::accessibility::accesskit::{Live, Node, TreeUpdate};
    use engine::accessibility::{self, Chrome, NodeIds};
    use engine::compositor::Compositor;
    use sem::{Role, SemId, Snapshot};

    use super::shown;
    use crate::view::ShowcaseView;

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

    /// The tree the bridge would send this frame.
    fn tree(view: &mut ShowcaseView) -> (TreeUpdate, NodeIds, Snapshot) {
        let mut c = Compositor::new();
        view.render(&mut c);
        let snapshot = shown(view).unwrap_or_default();
        let (u, ids) = accessibility::build(&snapshot, &c, &CHROME);
        (u, ids, snapshot)
    }

    fn node<'a>(u: &'a TreeUpdate, ids: &NodeIds, id: &str) -> &'a Node {
        let want = ids.node(&sid(id));
        match u.nodes.iter().find(|(n, _)| Some(*n) == want) {
            Some((_, n)) => n,
            None => panic!("{id} is not in the tree"),
        }
    }

    /// The toasts of the gallery, as the tree carries them: each id with
    /// its politeness and what a reader hears.
    fn toasts(u: &TreeUpdate, ids: &NodeIds, s: &Snapshot) -> Vec<(String, Option<Live>, String)> {
        s.children(&sid("toasts"))
            .map(|t| {
                let n = node(u, ids, t.id.as_str());
                (
                    t.id.to_string(),
                    n.live(),
                    n.value().unwrap_or_default().to_owned(),
                )
            })
            .collect()
    }

    #[test]
    fn with_no_toast_the_reader_is_shown_the_page_as_it_is() {
        let view = view_on("todo");
        assert_eq!(shown(&view), view.semantics());
        let view = view_on("button");
        assert_eq!(shown(&view), None, "a page without meaning: the window");
    }

    /// The acceptance of task-75 item 3: a person types in the todo field,
    /// a toast arrives, and the reader hears it (polite; an error,
    /// assertive) while the focus stays in the field.
    #[test]
    fn a_toast_is_announced_and_the_focus_stays_in_the_field() {
        let mut view = view_on("todo");
        assert!(view.assist_focus(&sid("todo.input")));
        let (before, ids, _) = tree(&mut view);
        let field = ids.node(&sid("todo.input"));
        assert_eq!(Some(before.focus), field, "the field has the focus");

        let theme = view.theme.clone();
        view.toasts.push("saved", Intent::Constructive, &theme);
        view.toasts.push("push failed", Intent::Destructive, &theme);
        let (after, ids, s) = tree(&mut view);
        assert_eq!(Some(after.focus), ids.node(&sid("todo.input")));
        assert_eq!(ids.node(&sid("todo.input")), field, "the same node");
        assert_eq!(
            toasts(&after, &ids, &s),
            [
                (
                    "toasts[0]".to_owned(),
                    Some(Live::Polite),
                    "saved".to_owned()
                ),
                (
                    "toasts[1]".to_owned(),
                    Some(Live::Assertive),
                    "push failed".to_owned()
                ),
            ]
        );
        assert_eq!(s.get(&sid("toasts[1]")).map(|n| n.role), Some(Role::Alert));
        // the page is all there, as the page projects it
        let page = view.semantics().unwrap_or_default();
        for n in page.nodes().values() {
            assert_eq!(s.get(&n.id), Some(n), "{} is the page's", n.id);
        }
        assert_eq!(s.focused(), page.focused());
    }

    /// The counter under the list is a status too: when a row is done,
    /// what a reader hears is the new count, and nothing takes the focus.
    #[test]
    fn the_todo_counter_says_its_new_count() {
        let mut view = view_on("todo");
        let (u, ids, _) = tree(&mut view);
        let counter = node(&u, &ids, "todo.counter");
        assert_eq!(counter.live(), Some(Live::Polite));
        assert_eq!(counter.value(), Some("2 active · 1 done"));
        let seen = view.semantics().unwrap_or_default();
        let Ok(req) =
            sem::Request::observe(&seen, sem::Actor::Assistive, "todo.item[1]", "toggle", None)
        else {
            panic!("the row toggles");
        };
        assert!(view.act(req).is_some());
        let (u, ids, _) = tree(&mut view);
        assert_eq!(
            node(&u, &ids, "todo.counter").value(),
            Some("1 active · 2 done")
        );
        assert_eq!(u.focus, accessibility::ROOT);
    }

    #[test]
    fn a_toast_on_a_page_without_meaning_is_heard_too() {
        let mut view = view_on("button");
        let theme = view.theme.clone();
        view.toasts.push("copied", Intent::Informational, &theme);
        let (u, ids, s) = tree(&mut view);
        assert_eq!(
            toasts(&u, &ids, &s),
            [(
                "toasts[0]".to_owned(),
                Some(Live::Polite),
                "copied".to_owned()
            )]
        );
        assert_eq!(node(&u, &ids, "toasts").label(), Some("notifications"));
        assert_eq!(u.focus, accessibility::ROOT);
    }
}
