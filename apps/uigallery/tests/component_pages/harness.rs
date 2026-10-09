//! The harness: the gallery on a page, and the person, the reader and an
//! agent through a real port acting on it.

use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use comps::prelude::WidgetEvent;
use engine::accessibility::accesskit::{Action, ActionData, ActionRequest, TreeId, TreeUpdate};
use engine::accessibility::{self, Assist, Chrome, Shown};
use engine::compositor::Compositor;
use sem::{ActionName, Node, SemId, Stage, StateRef, Value};
use showcase::view::ShowcaseView;
use wire::{Act, AgentPort, Client, Config, Observe, Reply};

pub(crate) const CHROME: Chrome<'static> = Chrome {
    title: "plev, design system showcase",
    scale: 2.0,
    size: (2560, 1600),
};

pub(crate) fn sid(text: &str) -> SemId {
    SemId::parse(text).unwrap_or_else(|e| panic!("{text}: {e}"))
}

/// The gallery on one page, with the frame on screen.
pub(crate) struct Gallery {
    pub(crate) view: ShowcaseView,
    pub(crate) c: Compositor,
}

pub(crate) fn gallery(page: &str) -> Gallery {
    let mut view = ShowcaseView::new(1280.0, 800.0);
    view.jump_to_page(page);
    assert_eq!(view.page_title(), page);
    let mut g = Gallery {
        view,
        c: Compositor::new(),
    };
    g.frame();
    g
}

impl Gallery {
    pub(crate) fn frame(&mut self) {
        self.view.render(&mut self.c);
    }

    /// What the page means now, its nodes: the state the three compare
    /// (the focus aside, which a click into a field takes and a request
    /// does not).
    pub(crate) fn state(&self) -> Vec<Node> {
        let snapshot = self.view.semantics().unwrap_or_default();
        snapshot.nodes().values().cloned().collect()
    }

    pub(crate) fn value(&self, at: &str) -> Option<Value> {
        let at = StateRef::parse(at).unwrap_or_else(|e| panic!("{at}: {e}"));
        self.view.semantics()?.value(&at).cloned()
    }

    /// The person clicks the middle of where `id` is bound.
    pub(crate) fn click(&mut self, id: &str) {
        let (x, y) = self.middle(id);
        self.view.handle_event(&WidgetEvent::MouseDown { x, y });
        self.view.handle_event(&WidgetEvent::MouseUp { x, y });
        self.frame();
    }

    /// The person drags from the middle of `id` by `dx`.
    pub(crate) fn drag(&mut self, id: &str, dx: f32) {
        let (x, y) = self.middle(id);
        self.view.handle_event(&WidgetEvent::MouseDown { x, y });
        self.view
            .handle_event(&WidgetEvent::MouseMove { x: x + dx, y });
        self.view
            .handle_event(&WidgetEvent::MouseUp { x: x + dx, y });
        self.frame();
    }

    /// The person types `text` into what has the keys.
    pub(crate) fn type_in(&mut self, text: &str) {
        for ch in text.chars() {
            assert!(self.view.handle_key(&ch.to_string()), "{ch} is taken");
        }
        self.frame();
    }

    pub(crate) fn middle(&self, id: &str) -> (f32, f32) {
        let Some([x, y, w, h]) = self.c.binding(&sid(id)).and_then(|b| b.visible_rect()) else {
            panic!("{id} is not bound where a person sees it");
        };
        (x + w / 2.0, y + h / 2.0)
    }

    /// What the bridge shows a reader now.
    pub(crate) fn shown(&self) -> (TreeUpdate, Shown) {
        let snapshot = self.view.reader_semantics().unwrap_or_default();
        let (update, ids) = accessibility::build(&snapshot, &self.c, &CHROME);
        (update, Shown { snapshot, ids })
    }

    /// The reader asks `action` of the node `id` of the tree: read back,
    /// run through the gallery's dispatcher, and completed.
    pub(crate) fn reader(&mut self, id: &str, action: Action, data: Option<ActionData>) {
        let (_, shown) = self.shown();
        let Some(node) = shown.ids.node(&sid(id)) else {
            panic!("{id} is not in the reader's tree");
        };
        let req = ActionRequest {
            action,
            target_tree: TreeId::ROOT,
            target_node: node,
            data,
        };
        let req = match accessibility::translate(&req, &shown) {
            Ok(Assist::Request(r)) => r,
            other => panic!("{id} {action:?}: {other:?}"),
        };
        assert_eq!(req.actor, sem::Actor::Assistive);
        let receipt = self.view.act(req).expect("the page takes it");
        assert!(
            matches!(receipt.stage(), Some(Stage::Completed(_))),
            "{id} {action:?}: {:?}",
            receipt.stage()
        );
        self.frame();
    }

    pub(crate) fn press(&mut self, id: &str) {
        self.reader(id, Action::Click, None);
    }

    /// Whether the reader's tree offers `action` on `id`.
    pub(crate) fn offers(&self, id: &str, action: Action) -> bool {
        let (u, shown) = self.shown();
        let want = shown.ids.node(&sid(id));
        u.nodes
            .iter()
            .find(|(n, _)| Some(*n) == want)
            .is_some_and(|(_, n)| n.supports_action(action))
    }
}

/// One request of the agent.
pub(crate) struct Ask {
    target: &'static str,
    action: &'static str,
    param: Option<Value>,
}

pub(crate) fn ask(target: &'static str, action: &'static str) -> Ask {
    Ask {
        target,
        action,
        param: None,
    }
}

pub(crate) fn ask_with(target: &'static str, action: &'static str, param: Value) -> Ask {
    Ask {
        target,
        action,
        param: Some(param),
    }
}

/// A private directory for one agent's socket, removed after.
pub(crate) struct Dir(PathBuf);

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// An agent through a real port on the gallery on `page`: it observes
/// and asks each of `asks` from the revision it saw, and every one
/// completes. This thread serves the port as the shell does (the wake,
/// `serve_agent`, a new frame when an act ran).
pub(crate) fn by_agent(page: &str, name: &str, asks: Vec<Ask>) -> Gallery {
    let mut g = gallery(page);
    let dir = Dir(std::env::temp_dir().join(format!("plev-t75c-{}-{name}", std::process::id())));
    let (tx, woke) = mpsc::channel();
    let config = Config {
        app: "showcase".into(),
        dir: dir.0.clone(),
        ring: 16,
    };
    let mut port = AgentPort::open_with(config, move || {
        let _ = tx.send(());
    })
    .expect("the port opens");
    let path = port.path().to_owned();
    let agent = thread::spawn(move || {
        let mut c = Client::connect(&path).expect("the agent connects");
        c.set_timeout(Some(20_000)).expect("a timeout");
        c.hello("claude").expect("hello");
        let mut ends = Vec::new();
        for ask in asks {
            let seen = match c.observe(Observe::default()).expect("an observation") {
                Reply::Snapshot(s) => s.revision,
                other => panic!("a snapshot: {other:?}"),
            };
            let action = ActionName::parse(ask.action).expect("an action name");
            let mut act = Act::from_revision(sid(ask.target), action, seen);
            if let Some(param) = ask.param {
                act = act.with_param(param);
            }
            let stages = c.act(act, |_| {}).expect("the act answers");
            let end = stages.last().map(|s| s.stage.clone());
            ends.push((ask.target, ask.action, end));
        }
        ends
    });
    let deadline = Instant::now() + Duration::from_secs(60);
    while !agent.is_finished() {
        assert!(Instant::now() < deadline, "the agent did not finish");
        let _ = woke.recv_timeout(Duration::from_millis(2));
        if g.view.serve_agent(&mut port, &g.c) {
            g.frame();
        }
    }
    for (target, action, end) in agent.join().expect("the agent ran") {
        assert!(
            matches!(end, Some(Stage::Completed(_))),
            "{target} {action}: {end:?}"
        );
    }
    g
}

/// The three end in the same state, and it is not where they started.
pub(crate) fn same(page: &str, person: &Gallery, reader: &Gallery, agent: &Gallery) {
    let before = gallery(page).state();
    assert_ne!(person.state(), before, "{page}: the person changed it");
    assert_eq!(reader.state(), person.state(), "{page}: the reader");
    assert_eq!(agent.state(), person.state(), "{page}: the agent");
}
