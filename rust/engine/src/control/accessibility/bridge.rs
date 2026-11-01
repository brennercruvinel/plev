//! The one accessibility helper an app shell uses: `Bridge` owns the
//! `accesskit_winit` adapter of a window, hands it the tree projected
//! from the app's meaning each frame a screen reader is listening, and
//! hands the app back what the screen reader asked.
//!
//! The adapter calls its handlers on whatever thread the platform uses.
//! They never touch the app: an action request goes into a channel and
//! the app's `wake` runs, which sends the app's own event through its
//! `EventLoopProxy` (docs/adr/rendering/0026-worker-results-wake-the-event-loop.md). On
//! the main thread the app drains the channel with `take`, and each
//! request goes through its dispatcher like any other actor's. The
//! initial tree is not built on the handler's thread either: activation
//! wakes the app, the platform shows its placeholder, and the next frame
//! sends the full tree.
//!
//! The bridge also carries the system's "reduce motion" to the app's
//! transitions (task-75 phase 1): it reads it when it attaches and each
//! time the window takes the focus again, and sets the event loop
//! thread's motion (`motion::follow_system`), so a shell that has a bridge
//! has it with no line of its own.
//!
//! Native only: AccessKit has no browser adapter, so a wasm build never
//! compiles this file (task-75 owns the web).

use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};

use accesskit::{ActionHandler, ActionRequest, ActivationHandler, DeactivationHandler, TreeUpdate};
use accord::Snapshot;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

use super::action::{self, Assist};
use super::tree::{self, Chrome, Shown};
use crate::drawing::compositor::Compositor;

/// Asks the app's event loop for a turn, from any thread.
pub type Wake = Arc<dyn Fn() + Send + Sync>;

/// Why a `Bridge` was not created.
#[derive(Debug, PartialEq, Eq)]
pub enum BridgeError {
    /// The window is already visible: AccessKit must attach before the
    /// first show. Create it with `with_visible(false)`, create the
    /// bridge, then `set_visible(true)`.
    WindowVisible,
}

impl std::fmt::Display for BridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WindowVisible => f.write_str(
                "the window is already visible; the accessibility adapter must \
                 attach before the first show",
            ),
        }
    }
}

impl std::error::Error for BridgeError {}

/// A window's accessibility, projected from the app's `accord::Snapshot`.
///
/// The shell creates it right after the window, while the window is still
/// hidden, and then:
///
/// 1. passes every window event to `process_event` before handling it;
/// 2. after each frame's scene is built, calls `update` with the
///    compositor that holds this frame's bindings and a closure that
///    projects the app (`None` when the screen has no meaning yet: the
///    tree is then the window alone). The closure runs only while an
///    assistive client is listening, so an app pays nothing otherwise;
/// 3. when `wake` fires, calls `take` on the main thread and sends each
///    `Assist::Request` through its dispatcher (a request aimed at a
///    confirmation it shows goes to `Dispatcher::confirm` with the
///    request's actor), moves its focus for each `Assist::Focus`, and asks
///    for a frame.
pub struct Bridge {
    adapter: accesskit_winit::Adapter,
    inbox: Inbox,
    title: String,
    shown: Shown,
}

impl Bridge {
    /// Attaches to `window`, which must not have been shown yet. `title`
    /// labels the root (the window's own title); `wake` is called, from
    /// any thread, when the screen reader starts listening or asks for an
    /// action. Call it on the event loop's thread: it sets that thread's
    /// motion from the system (see the module docs).
    pub fn new(
        event_loop: &ActiveEventLoop,
        window: &Window,
        title: impl Into<String>,
        wake: impl Fn() + Send + Sync + 'static,
    ) -> Result<Self, BridgeError> {
        if window.is_visible() == Some(true) {
            return Err(BridgeError::WindowVisible);
        }
        super::reduced_motion::follow_system();
        super::preference::follow_contrast();
        let (activation, actions, deactivation, inbox) = handlers(Arc::new(wake));
        let adapter = accesskit_winit::Adapter::with_direct_handlers(
            event_loop,
            window,
            activation,
            actions,
            deactivation,
        );
        Ok(Self {
            adapter,
            inbox,
            title: title.into(),
            shown: Shown::default(),
        })
    }

    /// Lets the adapter see a window event (focus of the window, on some
    /// platforms more). Call it before the app handles the event. The
    /// window taking the focus again reads the system's "reduce motion"
    /// once more: the person may have changed it while away.
    pub fn process_event(&mut self, window: &Window, event: &WindowEvent) {
        if matches!(event, WindowEvent::Focused(true)) {
            super::reduced_motion::follow_system();
            super::preference::follow_contrast();
        }
        self.adapter.process_event(window, event);
    }

    /// Sends the tree of this frame when an assistive client listens:
    /// `project` is called then, and only then, and its snapshot is placed
    /// by `compositor`'s bindings at `window`'s scale.
    pub fn update(
        &mut self,
        window: &Window,
        compositor: &Compositor,
        project: impl FnOnce() -> Option<Snapshot>,
    ) {
        let Self {
            adapter,
            title,
            shown,
            ..
        } = self;
        adapter.update_if_active(|| {
            let snapshot = project().unwrap_or_default();
            let size = window.inner_size();
            let chrome = Chrome {
                title,
                scale: window.scale_factor(),
                size: (size.width, size.height),
            };
            let (update, ids) = tree::build(&snapshot, compositor, &chrome);
            *shown = Shown { snapshot, ids };
            update
        });
    }

    /// What the screen reader asked since the last call, read against the
    /// tree it was shown, in the order it asked.
    pub fn take(&mut self) -> Vec<Assist> {
        self.inbox.drain(&self.shown)
    }
}

/// The main thread's end of the handlers' channel.
pub(crate) struct Inbox {
    rx: Receiver<ActionRequest>,
}

impl Inbox {
    /// Every request queued so far, translated; one that maps to nothing
    /// is logged and dropped (AccessKit asks an app to ignore what it does
    /// not support).
    pub(crate) fn drain(&self, shown: &Shown) -> Vec<Assist> {
        let mut out = Vec::new();
        while let Ok(req) = self.rx.try_recv() {
            match action::translate(&req, shown) {
                Ok(assist) => out.push(assist),
                Err(why) => log::debug!("accessibility: {:?} dropped: {why}", req.action),
            }
        }
        out
    }
}

/// The three handlers the adapter calls, and the inbox they feed.
pub(crate) fn handlers(wake: Wake) -> (Activation, Actions, Deactivation, Inbox) {
    let (tx, rx) = mpsc::channel();
    (
        Activation { wake: wake.clone() },
        Actions { tx, wake },
        Deactivation,
        Inbox { rx },
    )
}

/// A client started listening: the app draws a frame, which sends the
/// full tree. The platform shows its placeholder until then.
pub(crate) struct Activation {
    wake: Wake,
}

impl ActivationHandler for Activation {
    fn request_initial_tree(&mut self) -> Option<TreeUpdate> {
        (self.wake)();
        None
    }
}

/// An action request: queued for the main thread, then the app is woken.
pub(crate) struct Actions {
    tx: Sender<ActionRequest>,
    wake: Wake,
}

impl ActionHandler for Actions {
    fn do_action(&mut self, request: ActionRequest) {
        if self.tx.send(request).is_ok() {
            (self.wake)();
        }
    }
}

/// The client left. The adapter stops asking for updates by itself, so
/// there is nothing to undo.
pub(crate) struct Deactivation;

impl DeactivationHandler for Deactivation {
    fn deactivate_accessibility(&mut self) {
        log::debug!("accessibility: the assistive client left");
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use accesskit::{Action, TreeId};
    use accord::{Actor, Role, SemId, SnapshotBuilder};

    use super::*;

    /// The adapter's threads never touch the app: an action request from
    /// another thread is queued and wakes the app, and the main thread
    /// reads it back, in order, against the tree it was shown.
    #[test]
    fn a_request_from_another_thread_waits_for_the_main_thread() {
        let wakes = Arc::new(AtomicUsize::new(0));
        let counter = Arc::clone(&wakes);
        let (mut activation, actions, _, inbox) = handlers(Arc::new(move || {
            counter.fetch_add(1, Ordering::SeqCst);
        }));

        assert!(
            activation.request_initial_tree().is_none(),
            "no tree off thread"
        );
        assert_eq!(
            wakes.load(Ordering::SeqCst),
            1,
            "activation asks for a frame"
        );

        let mut b = SnapshotBuilder::new();
        b.node("go", Role::Button, "go").action("activate");
        b.node("stop", Role::Button, "stop").action("activate");
        let snapshot = match b.finish() {
            Ok(s) => s,
            Err(e) => panic!("{e}"),
        };
        let mut c = Compositor::new();
        c.begin_frame();
        let chrome = Chrome {
            title: "t",
            scale: 1.0,
            size: (0, 0),
        };
        let (_, ids) = tree::build(&snapshot, &c, &chrome);
        let shown = Shown { snapshot, ids };
        let click = |id: &str| ActionRequest {
            action: Action::Click,
            target_tree: TreeId::ROOT,
            target_node: shown
                .ids
                .node(&SemId::parse(id).unwrap_or_else(|e| panic!("{e}")))
                .unwrap_or(super::super::ids::ROOT),
            data: None,
        };
        let (go, stop) = (click("go"), click("stop"));

        let mut actions = actions;
        let worker = std::thread::spawn(move || {
            actions.do_action(go);
            actions.do_action(stop);
        });
        if worker.join().is_err() {
            panic!("the handler thread panicked");
        }
        assert_eq!(wakes.load(Ordering::SeqCst), 3, "one wake per request");

        let asked = inbox.drain(&shown);
        let targets: Vec<(&str, &Actor)> = asked
            .iter()
            .filter_map(|a| match a {
                Assist::Request(r) => Some((r.target.as_str(), &r.actor)),
                Assist::Focus(_) | Assist::Reveal(_) => None,
            })
            .collect();
        assert_eq!(
            targets,
            [("go", &Actor::Assistive), ("stop", &Actor::Assistive)]
        );
        assert!(inbox.drain(&shown).is_empty(), "drained once");
    }

    #[test]
    fn a_request_that_maps_to_nothing_is_dropped() {
        let (_, mut actions, _, inbox) = handlers(Arc::new(|| {}));
        actions.do_action(ActionRequest {
            action: Action::Click,
            target_tree: TreeId::ROOT,
            target_node: super::super::ids::ROOT,
            data: None,
        });
        assert!(inbox.drain(&Shown::default()).is_empty());
    }
}
