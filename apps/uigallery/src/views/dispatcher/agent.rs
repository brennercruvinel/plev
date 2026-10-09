use engine::compositor::Compositor;
use rec::Recorder;
use sem::{Actor, ReceiptSink, Request, RequestId, Revision, SemId, Snapshot};
use wire::{AgentPort, Bound, Host};

use super::ShowcaseView;

impl ShowcaseView {
    /// The gallery's agent port when `PLEV_AGENT=1` in a debug build;
    /// `None` otherwise, or when it cannot open (logged). `wake` makes
    /// the shell serve it.
    pub fn agent_port(wake: impl Fn() + Send + 'static) -> Option<AgentPort> {
        AgentPort::from_env("showcase", wake)
            .inspect_err(|e| log::error!("agent port: {e}"))
            .ok()
            .flatten()
    }

    /// What agents asked through `port`, served on this thread
    /// through the active page's hooks, `compositor` holding the frame
    /// on screen. True when an act was dispatched.
    pub fn serve_agent(&mut self, port: &mut AgentPort, compositor: &Compositor) -> bool {
        self.serve_agent_recorded(port, compositor, None)
    }

    /// [`serve_agent`](Self::serve_agent), with `recorder` (when the
    /// session is recorded, `crate::app::record`) keeping every
    /// observation served, with its actor, scope and revision, on the
    /// gallery's clock, and the state an act left right after the drain
    /// that ran it.
    pub fn serve_agent_recorded(
        &mut self,
        port: &mut AgentPort,
        compositor: &Compositor,
        recorder: Option<&Recorder>,
    ) -> bool {
        let acted = port.drain(&mut Served {
            view: self,
            compositor,
            recorder,
        });
        if let Some(rec) = recorder.filter(|_| acted)
            && let Some(snapshot) = self.semantics()
        {
            rec.snapshot(self.now(), &snapshot);
        }
        acted
    }
}

/// The gallery as the port serves it: the shell and the active page in
/// one projection, where the page's bound nodes sit, the one dispatcher.
struct Served<'a> {
    view: &'a mut ShowcaseView,
    compositor: &'a Compositor,
    /// Records every observation served, when the session is recorded.
    recorder: Option<&'a Recorder>,
}

impl Host for Served<'_> {
    /// The shell's capabilities and the page's (task-76 item 4): an agent
    /// switches the theme or the page here, and is refused the window.
    fn project(&self) -> Option<Snapshot> {
        self.view
            .gallery_semantics()
            .inspect_err(|e| log::warn!("the gallery projection does not validate: {e}"))
            .ok()
    }

    /// The visible part of each binding (its rect inside its clip).
    fn bounds(&self) -> Vec<Bound> {
        self.compositor
            .visible_bindings()
            .map(|(id, rect)| Bound {
                id: id.clone(),
                rect,
            })
            .collect()
    }

    fn dispatch(&mut self, req: Request) -> Option<RequestId> {
        self.view.act(req).map(|receipt| receipt.id)
    }

    fn add_sink(&mut self, sink: Box<dyn ReceiptSink>) {
        self.view.add_sink(sink);
    }

    /// An observation was served: recorded at the gallery's clock.
    fn observed(&mut self, actor: &Actor, scope: Option<&SemId>, revision: Revision) {
        if let Some(rec) = self.recorder {
            rec.observed(self.view.now(), actor.clone(), scope.cloned(), revision);
        }
    }
}

/// The gallery's todo page through a real agent port in a temporary
/// directory, with no window: this thread is the shell's (it drains
/// the port through the view and draws the frame), a spawned thread
/// is the agent.
#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::sync::mpsc::{self, Receiver, Sender};
    use std::thread;
    use std::time::{Duration, Instant};

    use engine::compositor::Compositor;
    use sem::{Actor, Change, Rejection, SemId, Stage, StateRef, Value};
    use wire::{Act, AgentPort, Client, Config, Observe, Reply, ServerMsg, encode_server};

    use crate::view::ShowcaseView;

    fn sid(text: &str) -> SemId {
        SemId::parse(text).unwrap()
    }

    /// What the agent asks the shell's thread to do between two of
    /// its messages, as a person at the window would.
    enum Poke {
        Type(&'static str),
        OpenModal,
        AddItems(usize),
    }

    type Pokes = Sender<(Poke, Sender<()>)>;

    fn poke(pokes: &Pokes, what: Poke) {
        let (done, wait) = mpsc::channel();
        pokes.send((what, done)).unwrap();
        wait.recv().unwrap();
    }

    /// A private directory for one test's socket, removed after.
    struct Dir(PathBuf);

    impl Dir {
        fn new(name: &str) -> Self {
            let dir = std::env::temp_dir().join(format!("plev-t61-{}-{name}", std::process::id()));
            Dir(dir)
        }
    }

    impl Drop for Dir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn frame(view: &mut ShowcaseView) -> Compositor {
        let mut c = Compositor::new();
        view.render(&mut c);
        c
    }

    /// Runs `agent` against the gallery on the todo page, serving the
    /// port the way `app.rs` does: the wake, then `serve_agent`, then
    /// a new frame when an act was dispatched.
    fn gallery<T: Send + 'static>(
        dir: &Path,
        agent: impl FnOnce(PathBuf, Pokes) -> T + Send + 'static,
    ) -> (T, ShowcaseView) {
        let mut view = ShowcaseView::new(1200.0, 800.0);
        view.jump_to_page("todo");
        let mut c = frame(&mut view);
        let (tx, woke): (Sender<()>, Receiver<()>) = mpsc::channel();
        let config = Config {
            app: "showcase".into(),
            dir: dir.to_owned(),
            ring: 16,
        };
        let mut port = AgentPort::open_with(config, move || {
            let _ = tx.send(());
        })
        .unwrap();
        let path = port.path().to_owned();
        let (pokes, poked) = mpsc::channel::<(Poke, Sender<()>)>();
        let agent = thread::spawn(move || agent(path, pokes));
        let deadline = Instant::now() + Duration::from_secs(60);
        while !agent.is_finished() {
            assert!(Instant::now() < deadline, "the agent did not finish");
            let _ = woke.recv_timeout(Duration::from_millis(2));
            if view.serve_agent(&mut port, &c) {
                c = frame(&mut view);
            }
            while let Ok((what, done)) = poked.try_recv() {
                match what {
                    Poke::Type(text) => {
                        assert!(view.assist_focus(&sid("todo.input")));
                        for ch in text.chars() {
                            assert!(view.handle_key(&ch.to_string()));
                        }
                    }
                    Poke::OpenModal => view.open_modal(false),
                    Poke::AddItems(n) => {
                        for i in 0..n {
                            assert!(view.assist_focus(&sid("todo.input")));
                            for ch in format!("item {i}").chars() {
                                view.handle_key(&ch.to_string());
                            }
                            assert!(view.handle_enter());
                        }
                    }
                }
                c = frame(&mut view);
                let _ = done.send(());
            }
        }
        (agent.join().unwrap(), view)
    }

    fn agent(path: &Path) -> Client {
        let mut c = Client::connect(path).unwrap();
        c.set_timeout(Some(20_000)).unwrap();
        c.hello("claude").unwrap();
        c
    }

    fn snapshot(reply: Reply) -> wire::SnapshotReply {
        match reply {
            Reply::Snapshot(s) => s,
            other => panic!("a snapshot: {other:?}"),
        }
    }

    fn act(c: &mut Client, target: &str, action: &str, r: sem::Revision) -> Vec<Stage> {
        let act = Act::from_revision(sid(target), sem::ActionName::parse(action).unwrap(), r);
        c.act(act, |_| {})
            .unwrap()
            .into_iter()
            .map(|s| s.stage)
            .collect()
    }

    fn act_with(
        c: &mut Client,
        target: &str,
        action: &str,
        r: sem::Revision,
        param: Value,
    ) -> Vec<Stage> {
        let act = Act::from_revision(sid(target), sem::ActionName::parse(action).unwrap(), r)
            .with_param(param);
        c.act(act, |_| {})
            .unwrap()
            .into_iter()
            .map(|s| s.stage)
            .collect()
    }

    fn names(stages: &[Stage]) -> Vec<&'static str> {
        stages.iter().map(Stage::name).collect()
    }

    /// A value of the page as it means now.
    fn value(view: &ShowcaseView, at: &str) -> Option<Value> {
        let at = StateRef::parse(at).unwrap();
        view.semantics()?.value(&at).cloned()
    }

    /// The acceptance: observe the todo scope, toggle a row from the
    /// revision seen, and see a submit from an old revision come back
    /// stale after a person typed in the field; with the delta.
    #[test]
    fn an_agent_observes_and_operates_the_todo_page() {
        let dir = Dir::new("todo");
        let ((scoped, toggled, stale, delta, added), view) = gallery(&dir.0, |path, pokes| {
            let mut c = agent(&path);
            let scoped = snapshot(
                c.observe(Observe {
                    scope: Some(sid("todo")),
                    bounds: true,
                    ..Observe::default()
                })
                .unwrap(),
            );
            let toggled = act(&mut c, "todo.item[1]", "toggle", scoped.revision);

            poke(&pokes, Poke::Type("buy milk"));
            let seen = snapshot(c.observe(Observe::default()).unwrap()).revision;
            poke(&pokes, Poke::Type(" and bread"));
            let stale = act(&mut c, "todo.input", "submit", seen);
            let delta = c
                .observe(Observe {
                    scope: Some(sid("todo.input")),
                    since: Some(seen),
                    ..Observe::default()
                })
                .unwrap();
            let now = snapshot(c.observe(Observe::default()).unwrap()).revision;
            let added = act(&mut c, "todo.input", "submit", now);
            (scoped, toggled, stale, delta, added)
        });

        let ids: Vec<&str> = scoped.nodes.iter().map(|n| n.id.as_str()).collect();
        for id in [
            "todo",
            "todo.input",
            "todo.filter",
            "todo.item[1]",
            "todo.counter",
        ] {
            assert!(ids.contains(&id), "{id} in {ids:?}");
        }
        let bounds = scoped.bounds.unwrap();
        assert!(
            bounds
                .iter()
                .any(|b| b.id == sid("todo.item[1]") && b.rect[2] > 0.0)
        );

        assert_eq!(
            names(&toggled),
            ["requested", "accepted", "executing", "completed"]
        );
        assert_eq!(
            value(&view, "todo.item[1]:completed"),
            Some(Value::Bool(true))
        );

        assert_eq!(names(&stale), ["requested", "rejected"]);
        let Stage::Rejected(Rejection::Stale { changes, .. }) = &stale[1] else {
            panic!("stale: {stale:?}");
        };
        assert_eq!(
            changes,
            &[Change::StateChanged {
                at: StateRef::parse("todo.input:text").unwrap(),
                before: Some(Value::Text("buy milk".into())),
                after: Some(Value::Text("buy milk and bread".into())),
                added: Vec::new(),
                removed: Vec::new(),
            }]
        );
        let Reply::Delta(delta) = delta else {
            panic!("a delta: {delta:?}");
        };
        assert!(
            delta.changes.iter().all(
                |c| matches!(c, Change::StateChanged { at, .. } if at.node == sid("todo.input"))
            ),
            "only the field, in the field's scope: {delta:?}"
        );
        assert!(!delta.changes.is_empty());

        assert_eq!(names(&added).last(), Some(&"completed"));
        assert_eq!(value(&view, "todo.counter:active"), Some(Value::Int(2)));
        assert_eq!(value(&view, "todo.counter:done"), Some(Value::Int(2)));
        assert_eq!(
            value(&view, "todo.input:text"),
            Some(Value::Text(String::new()))
        );
        let agent = Actor::Agent {
            name: "claude".into(),
        };
        let Some(receipt) = view.panes.todo.receipt(sem::RequestId::parse("1").unwrap()) else {
            panic!("the page's dispatcher kept the first request");
        };
        assert_eq!(receipt.request.actor, agent);
    }

    /// An open overlay leaves the page inert for an agent as for a
    /// click, and confirming stays a person's.
    #[test]
    fn behind_a_modal_the_agent_is_refused_as_a_click_is() {
        let dir = Dir::new("modal");
        let (stages, _) = gallery(&dir.0, |path, pokes| {
            let mut c = agent(&path);
            let r = snapshot(c.observe(Observe::default()).unwrap()).revision;
            poke(&pokes, Poke::OpenModal);
            let stages = act(&mut c, "todo.item[1]", "toggle", r);
            let refused = c.call(wire::Ask::Confirm {
                request: sem::RequestId::parse("1").unwrap(),
            });
            assert!(matches!(
                refused,
                Err(wire::WireError::Refused(
                    wire::ProtoError::NotEnabled { .. }
                ))
            ));
            stages
        });
        assert_eq!(
            stages.last(),
            Some(&Stage::Rejected(Rejection::Inert {
                target: sid("todo.item[1]")
            }))
        );
    }

    /// The shell through the port (task-76 item 4): the agent sees the
    /// shell's capabilities beside the page's, switches the theme and
    /// the page from the revision it saw, and is refused the page's own
    /// window, which is a person's.
    #[test]
    fn an_agent_switches_the_theme_and_the_page_and_is_refused_the_window() {
        let dir = Dir::new("shell");
        let ((ids, themed, paged, refused), view) = gallery(&dir.0, |path, _pokes| {
            let mut c = agent(&path);
            let whole = snapshot(c.observe(Observe::default()).unwrap());
            let ids: Vec<String> = whole.nodes.iter().map(|n| n.id.to_string()).collect();
            let themed = act_with(
                &mut c,
                "shell.theme",
                "select",
                whole.revision,
                Value::Text("dark".into()),
            );
            let r = snapshot(c.observe(Observe::default()).unwrap()).revision;
            let paged = act_with(
                &mut c,
                "shell.page",
                "select",
                r,
                Value::Text("Card".into()),
            );
            let r = snapshot(c.observe(Observe::default()).unwrap()).revision;
            let refused = act(&mut c, "shell.detach", "toggle", r);
            (ids, themed, paged, refused)
        });
        for id in ["shell", "shell.theme", "shell.page", "shell.detach", "todo"] {
            assert!(ids.contains(&id.to_owned()), "{id} in {ids:?}");
        }
        assert_eq!(
            names(&themed),
            ["requested", "accepted", "executing", "completed"]
        );
        assert_eq!(view.theme_name, "dark");
        assert_eq!(names(&paged).last(), Some(&"completed"));
        assert_eq!(view.page_title(), "Card");
        assert_eq!(names(&refused), ["requested", "rejected"]);
        assert!(matches!(
            &refused[1],
            Stage::Rejected(Rejection::NotAuthorized {
                actor: Actor::Agent { .. }
            })
        ));
        assert!(!view.is_detached());
    }

    fn bytes(reply: Reply) -> usize {
        encode_server(&ServerMsg {
            id: Some("1".into()),
            reply,
        })
        .len()
    }

    /// What an observation and an act cost on the todo page, measured
    /// over the socket: the bytes of a whole observation (as served,
    /// and with each node's `source` asked for), a scoped and an
    /// incremental one, and the round trip of one act. Bytes are the
    /// same in debug; the round trip wants release. Run by hand:
    /// `cargo test -p showcase --release agent_port_cost -- --ignored --nocapture`.
    #[test]
    #[ignore = "a measurement, not a check"]
    fn agent_port_cost() {
        let dir = Dir::new("cost");
        let (lines, _) = gallery(&dir.0, |path, pokes| {
            let mut c = agent(&path);
            let mut lines = Vec::new();
            for (extra, label) in [(0, "3 items"), (100, "103 items")] {
                poke(&pokes, Poke::AddItems(extra));
                let whole = c.observe(Observe::default()).unwrap();
                let r = snapshot(whole.clone()).revision;
                let sourced = c
                    .observe(Observe {
                        source: true,
                        ..Observe::default()
                    })
                    .unwrap();
                let scoped = c
                    .observe(Observe {
                        scope: Some(sid("todo.item[1]")),
                        ..Observe::default()
                    })
                    .unwrap();
                let keyed = c
                    .observe(Observe {
                        keys: Some(vec![sem::StateKey::parse("completed").unwrap()]),
                        ..Observe::default()
                    })
                    .unwrap();
                let mut best = Duration::MAX;
                let mut at = r;
                let mut delta = 0;
                for n in 0..50 {
                    let start = Instant::now();
                    let stages = act(&mut c, "todo.item[1]", "toggle", at);
                    best = best.min(start.elapsed());
                    assert_eq!(names(&stages).last(), Some(&"completed"));
                    if n == 0 {
                        let since = Observe {
                            since: Some(r),
                            ..Observe::default()
                        };
                        delta = bytes(c.observe(since).unwrap());
                    }
                    at = snapshot(
                        c.observe(Observe {
                            scope: Some(sid("todo.item[1]")),
                            ..Observe::default()
                        })
                        .unwrap(),
                    )
                    .revision;
                }
                lines.push(format!(
                    "{label}: whole {} B (with source {} B), one row {} B, keyed to \
                     completed {} B, delta after one toggle {delta} B, act round trip \
                     best {} us",
                    bytes(whole),
                    bytes(sourced),
                    bytes(scoped),
                    bytes(keyed),
                    best.as_micros()
                ));
            }
            lines
        });
        for line in lines {
            println!("{line}");
        }
    }

    /// The line of the delta a new row makes (a person adds one item
    /// through the page), beside the whole observation after it, with
    /// 3 and with 103 items before the one added. Bytes only, so a
    /// debug build measures them. Run by hand:
    /// `cargo test -p showcase agent_port_new_row -- --ignored --nocapture`.
    #[test]
    #[ignore = "a measurement, not a check"]
    fn agent_port_new_row() {
        let dir = Dir::new("row");
        let (lines, _) = gallery(&dir.0, |path, pokes| {
            let mut c = agent(&path);
            let mut lines = Vec::new();
            // the page opens with 3 items; the first round leaves 4.
            for (extra, items) in [(0, 3), (99, 103)] {
                poke(&pokes, Poke::AddItems(extra));
                let r = snapshot(c.observe(Observe::default()).unwrap()).revision;
                poke(&pokes, Poke::AddItems(1));
                let since = Observe {
                    since: Some(r),
                    ..Observe::default()
                };
                let delta = c.observe(since).unwrap();
                let whole = bytes(c.observe(Observe::default()).unwrap());
                let line = encode_server(&ServerMsg {
                    id: Some("1".into()),
                    reply: delta,
                });
                lines.push(format!(
                    "{items} items, one added: delta {} B, whole after it {whole} B\n{line}",
                    line.len()
                ));
            }
            lines
        });
        for line in lines {
            println!("{line}");
        }
    }
}
