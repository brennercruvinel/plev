//! The window's outside actors, desktop only: a screen reader (the
//! AccessKit bridge) and agents (the port), and how their threads wake the
//! event loop. What the screen reader is shown is the view's
//! (`ShowcaseView::reader_semantics`): the page, the toasts and the
//! overlay over it, and what only a reader hears (task-75 phase 2a).

use winit::event_loop::{ActiveEventLoop, EventLoopProxy};
use winit::window::Window;

use super::super::{TITLE, UserEvent};
use crate::views::ShowcaseView;

/// The window's outside actors (a screen reader, agents through the port)
/// and how their threads wake the event loop.
#[derive(Default)]
pub(crate) struct Access {
    pub(super) bridge: Option<engine::control::accessibility::Bridge>,
    pub(crate) agent: Option<socket::AgentPort>,
    pub(crate) proxy: Option<EventLoopProxy<UserEvent>>,
    /// The windows' scenes, written for the port's watches (`mirror`).
    pub(super) scenes: super::mirror::Scenes,
}

impl Access {
    /// Creates the bridge for `window`, which is still hidden. Without a
    /// bridge the window still opens, and the reason is logged.
    pub(crate) fn attach(&mut self, event_loop: &ActiveEventLoop, window: &Window) {
        let proxy = self.proxy.clone();
        let wake = move || {
            if let Some(proxy) = &proxy {
                let _ = proxy.send_event(UserEvent::Access);
            }
        };
        match engine::control::accessibility::Bridge::new(event_loop, window, TITLE, wake) {
            Ok(bridge) => self.bridge = Some(bridge),
            Err(e) => log::error!("accessibility: {e}"),
        }
    }

    /// What the screen reader asked, on the main thread: a request goes
    /// through the active page's dispatcher as `Actor::Assistive`, a focus
    /// move to the page's focus, a scroll into view to the page's scroll.
    pub(crate) fn carry_out(&mut self, view: &mut ShowcaseView) {
        let Some(bridge) = self.bridge.as_mut() else {
            return;
        };
        for assist in bridge.take() {
            match assist {
                engine::control::accessibility::Assist::Request(req) => {
                    let asked = format!("{} {}", req.target, req.action);
                    match view.act(req) {
                        Some(receipt) => {
                            log::debug!("accessibility: {asked}: {:?}", receipt.stage());
                        }
                        None => log::debug!("accessibility: no page takes {asked}"),
                    }
                }
                engine::control::accessibility::Assist::Focus(id) => {
                    if !view.assist_focus(&id) {
                        log::debug!("accessibility: no page takes focus on {id}");
                    }
                }
                engine::control::accessibility::Assist::Reveal(id) => {
                    if !view.assist_reveal(&id) {
                        log::debug!("accessibility: no page scrolls {id} into view");
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    //! What the bridge is shown (`ShowcaseView::reader_semantics`) as the
    //! tree it sends: the toasts, the counter and the shell's notice are
    //! heard where they are, and the focus stays.

    use accord::{Role, SemId, Snapshot};
    use engine::control::accessibility::accesskit::{Live, Node, TreeUpdate};
    use engine::control::accessibility::{self, Chrome, NodeIds};
    use engine::drawing::compositor::Compositor;
    use engine::widgets::prelude::Intent;

    use crate::views::ShowcaseView;

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
        let snapshot = view.reader_semantics().unwrap_or_default();
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
    fn with_nothing_over_the_page_the_reader_is_shown_the_page_as_it_is() {
        // the page as it is, and the sidebar after it (task-104 front H)
        let view = view_on("todo");
        let page = view.semantics().unwrap();
        let seen = view.reader_semantics().unwrap();
        assert_eq!(seen, view.keyboard_semantics().unwrap());
        for (id, node) in page.nodes() {
            assert_eq!(seen.get(id), Some(node), "{id}: the page's own");
        }
        let rest: Vec<&str> = seen
            .nodes()
            .keys()
            .filter(|id| page.get(id).is_none())
            .map(|id| id.as_str())
            .collect();
        assert_eq!(rest, ["shell.sidebar", "shell.sidebar.pin"]);
        let view = view_on("separator");
        let seen = view.reader_semantics().unwrap();
        assert!(
            seen.nodes().keys().all(|id| view.is_sidebar(id)),
            "a page without meaning: the window and the sidebar"
        );
    }

    /// The acceptance of task-75 item 3: a person types in the todo field,
    /// a toast arrives, and the reader hears it (polite; an error,
    /// assertive) while the focus stays in the field. Since phase 2a the
    /// toasts are the view's projection, the one agents read too.
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
        // the toasts are the view's projection: an agent reads them too
        let gallery = view.gallery_semantics().unwrap_or_default();
        assert_eq!(
            gallery.get(&sid("toasts[0]")).map(|n| n.label.as_str()),
            Some("saved")
        );
        assert_eq!(s.focused(), Some(&sid("todo.input")));
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
        let Ok(req) = accord::Request::observe(
            &seen,
            accord::Actor::Assistive,
            "todo.item[1]",
            "toggle",
            None,
        ) else {
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
        let mut view = view_on("separator");
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

    /// The shell's notice (what it told the person on its own, a job that
    /// ended while no window had the focus) is a status in the reader's
    /// tree: heard when it changes, the focus where it was. The port
    /// reads it as the shell's state, and the view's projection does not
    /// carry it.
    #[test]
    fn the_shells_notice_is_a_status_a_reader_hears() {
        let mut view = view_on("todo");
        assert!(view.assist_focus(&sid("todo.input")));
        view.set_notice("silences detected: 2 cuts");
        let (u, ids, s) = tree(&mut view);
        let notice = node(&u, &ids, "shell.notice");
        assert_eq!(notice.live(), Some(Live::Polite));
        assert_eq!(notice.value(), Some("silences detected: 2 cuts"));
        assert_eq!(notice.label(), Some("silences detected: 2 cuts"));
        assert_eq!(Some(u.focus), ids.node(&sid("todo.input")));
        assert!(s.get(&sid("todo")).is_some(), "the page is all there");
        assert!(
            view.semantics()
                .is_some_and(|p| p.get(&sid("shell.notice")).is_none())
        );
        let gallery = view.gallery_semantics().unwrap_or_default();
        let said = gallery
            .value(&accord::StateRef::parse("shell:notice").unwrap_or_else(|e| panic!("{e}")))
            .cloned();
        assert_eq!(said, Some(accord::Value::from("silences detected: 2 cuts")));
        // a page without meaning hears it too
        let mut view = view_on("separator");
        view.set_notice("done");
        let (u, ids, _) = tree(&mut view);
        assert_eq!(node(&u, &ids, "shell.notice").value(), Some("done"));
    }
}
