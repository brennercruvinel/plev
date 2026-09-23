//! Dispatch to the active pane: its natural height, its share of an
//! event, its visible-only tick and its drawing. One match per verb over
//! `pages::Pane`, so the rest of the view never names a pane's field.

use comps::prelude::{EmptyState, EventResult, Rect, Wake, WidgetEvent};
use engine::compositor::Compositor;
use engine::theme::Theme;
use sem::{Receipt, ReceiptSink, Request, SemId, Snapshot};

use super::components::toast;
use super::{Layers, Pane, ShowcaseView, panel};

impl ShowcaseView {
    /// Natural (unclipped) height of the active page's content.
    pub(super) fn page_content_height(&self, content: Rect) -> f32 {
        let t = &self.theme;
        let body = match self.pane() {
            None => t.size.card_min_w / 2.0,
            Some(pane) => match pane {
                Pane::Theme => self.panes.themes.content_height(content, t),
                Pane::Typography => self.panes.typography.content_height(content),
                Pane::Icons => self.panes.icons.content_height(content),
                Pane::Effects => self.panes.effects.content_height(content, t),
                Pane::Avatar => self.panes.avatar.content_height(content, t),
                Pane::Badge => self.panes.badge.content_height(content, t),
                Pane::Breadcrumb => self.panes.breadcrumb.content_height(content, t),
                Pane::Button => self.panes.button.content_height(content, t),
                Pane::Card => self.panes.card.content_height(content, t),
                Pane::Chart => self.panes.chart.content_height(content),
                Pane::Checkbox => self.panes.checkbox.content_height(content, t),
                Pane::Chip => self.panes.chip.content_height(content, t),
                Pane::CodeBlock => self.panes.code_block.content_height(content, t),
                Pane::ContextMenu => self.panes.context_menu.content_height(content, t),
                Pane::Dialog => self.panes.dialog.content_height(content, t),
                Pane::Empty => self.panes.empty.content_height(content, t),
                Pane::Graph => self.panes.graph.content_height(content, t),
                Pane::IconButton => self.panes.icon_button.content_height(content, t),
                Pane::Input => self.panes.input.content_height(content, t),
                Pane::NavigationMenu => self.panes.navigation_menu.content_height(content, t),
                Pane::Panel => self.panes.panel.content_height(content, t),
                Pane::PanelHeader => self.panes.panel_header.content_height(content, t),
                Pane::Progress => self.panes.progress.content_height(content, t),
                Pane::Resizable => self.panes.resizable.content_height(content, t),
                Pane::ScrollArea => self.panes.scroll_area.content_height(content, t),
                Pane::Select => self.panes.select.content_height(content, t),
                Pane::Separator => self.panes.separator.content_height(content, t),
                Pane::Sidebar => self.panes.sidebar.content_height(content, t),
                Pane::Skeleton => self.panes.skeleton.content_height(content, t),
                Pane::Slider => self.panes.slider.content_height(content, t),
                Pane::Spinner => self.panes.spinner.content_height(content, t),
                Pane::Stat => self.panes.stat.content_height(content, t),
                Pane::Switch => self.panes.switch.content_height(content, t),
                Pane::Table => self.panes.table.content_height(content, t),
                Pane::Tabs => self.panes.tabs.content_height(content, t),
                Pane::Toast => self.panes.toast.content_height(content, t),
                Pane::Tooltip => self.panes.tooltip.content_height(content, t),
                Pane::Tree => self.panes.tree.content_height(content, t),
                Pane::Todo => self.panes.todo.content_height(content),
                Pane::Dock => self.panes.dock.content_height(content),
                Pane::Editor => self.panes.editor.content_height(content),
                Pane::Git => self.panes.git.content_height(content),
                Pane::Builder => self.panes.builder.content_height(content),
            },
        };
        body + self.note_h()
    }

    /// The active pane's share of a pointer event, with the view-level
    /// effects a page asks for (a dialog, a toast, a theme).
    pub(super) fn page_event(&mut self, event: &WidgetEvent) -> EventResult {
        let content = self.page_rect();
        let t = &self.theme;
        match self.pane() {
            None => EventResult::IGNORED,
            Some(pane) => match pane {
                Pane::Theme => {
                    let (r, picked) = self.panes.themes.handle_event(event, content);
                    if let Some(name) = picked {
                        self.set_theme(name);
                    }
                    r
                }
                Pane::Typography => self.panes.typography.handle_event(event, content),
                Pane::Icons => self.panes.icons.handle_event(event, content),
                Pane::Effects => self.panes.effects.handle_event(event, content),
                Pane::Avatar => self.panes.avatar.handle_event(event, content, t),
                Pane::Badge => self.panes.badge.handle_event(event, content, t),
                Pane::Breadcrumb => self.panes.breadcrumb.handle_event(event, content, t),
                Pane::Button => self.panes.button.handle_event(event, content, t),
                Pane::Card => self.panes.card.handle_event(event, content, t),
                Pane::Chart => self.panes.chart.handle_event(event, content),
                Pane::Checkbox => self.panes.checkbox.handle_event(event, content, t),
                Pane::Chip => self.panes.chip.handle_event(event, content, t),
                Pane::CodeBlock => self.panes.code_block.handle_event(event, content, t),
                Pane::ContextMenu => self.panes.context_menu.handle_event(event, content, t),
                Pane::Dialog => {
                    let (r, open) = self.panes.dialog.handle_event(event, content, t);
                    if let Some(destructive) = open {
                        self.open_modal(destructive);
                    }
                    r
                }
                Pane::Empty => self.panes.empty.handle_event(event, content, t),
                Pane::Graph => self.panes.graph.handle_event(event, content, t),
                Pane::IconButton => self.panes.icon_button.handle_event(event, content, t),
                Pane::Input => self.panes.input.handle_event(event, content, t),
                Pane::NavigationMenu => self.panes.navigation_menu.handle_event(event, content, t),
                Pane::Panel => self.panes.panel.handle_event(event, content, t),
                Pane::PanelHeader => self.panes.panel_header.handle_event(event, content, t),
                Pane::Progress => self.panes.progress.handle_event(event, content, t),
                Pane::Resizable => self.panes.resizable.handle_event(event, content, t),
                Pane::ScrollArea => self.panes.scroll_area.handle_event(event, content, t),
                Pane::Select => self.panes.select.handle_event(event, content, t),
                Pane::Separator => self.panes.separator.handle_event(event, content, t),
                Pane::Sidebar => self.panes.sidebar.handle_event(event, content, t),
                Pane::Skeleton => self.panes.skeleton.handle_event(event, content, t),
                Pane::Slider => self.panes.slider.handle_event(event, content, t),
                Pane::Spinner => self.panes.spinner.handle_event(event, content, t),
                Pane::Stat => self.panes.stat.handle_event(event, content, t),
                Pane::Switch => self.panes.switch.handle_event(event, content, t),
                Pane::Table => self.panes.table.handle_event(event, content, t),
                Pane::Tabs => self.panes.tabs.handle_event(event, content, t),
                Pane::Toast => {
                    let (r, queued) = self.panes.toast.handle_event(event, content, t);
                    if let Some(intent) = queued {
                        self.toasts
                            .push(toast::ToastPage::message(intent), intent, &self.theme);
                    }
                    r
                }
                Pane::Tooltip => self.panes.tooltip.handle_event(event, content, t),
                Pane::Tree => self.panes.tree.handle_event(event, content, t),
                Pane::Todo => {
                    let inert = self.overlay_holds_pointer();
                    self.panes.todo.set_inert(inert);
                    self.panes.todo.handle_event(event, content, t)
                }
                Pane::Dock => self.panes.dock.handle_event(event, content),
                Pane::Editor => self.panes.editor.handle_event(event, content),
                Pane::Git => self.panes.git.handle_event(event, content),
                Pane::Builder => self.panes.builder.handle_event(event, content),
            },
        }
    }

    /// Motion that would run forever ticks only while visible: the
    /// dock's caret, the chart reveal, spinners, breathing skeletons,
    /// the todo caret, the editor. Continuous motion asks for every
    /// frame; a caret asks for its next flip.
    pub(super) fn page_wake(&mut self, dt: f32) -> Wake {
        match self.pane() {
            Some(Pane::Dock) => Wake::moving(self.panes.dock.tick(dt)),
            Some(Pane::Chart) => Wake::moving(self.panes.chart.tick(dt)),
            Some(Pane::Spinner) => Wake::moving(self.panes.spinner.tick(dt)),
            Some(Pane::Skeleton) => Wake::moving(self.panes.skeleton.tick(dt)),
            Some(Pane::Todo) => self.panes.todo.wake(dt),
            Some(Pane::Editor) => self.panes.editor.wake(dt),
            _ => Wake::idle(),
        }
    }

    /// Draw the active page into `content` (a placeholder names its slot).
    pub(super) fn page_render(
        &mut self,
        c: &mut Compositor,
        layers: Layers,
        content: Rect,
        theme: &Theme,
    ) {
        match self.pane() {
            None => self.render_placeholder(c, content, theme),
            Some(pane) => match pane {
                Pane::Theme => self
                    .panes
                    .themes
                    .render(c, content, theme, &self.theme_name),
                Pane::Typography => self.panes.typography.render(c, content, theme),
                Pane::Icons => self.panes.icons.render(c, content, theme),
                Pane::Effects => self.panes.effects.render(c, content, theme),
                Pane::Avatar => self.panes.avatar.render(c, content, theme),
                Pane::Badge => self.panes.badge.render(c, content, theme),
                Pane::Breadcrumb => self.panes.breadcrumb.render(c, content, theme),
                Pane::Button => self.panes.button.render(c, content, theme),
                Pane::Card => self.panes.card.render(c, content, theme),
                Pane::Chart => self.panes.chart.render(c, content, theme),
                Pane::Checkbox => self.panes.checkbox.render(c, content, theme),
                Pane::Chip => self.panes.chip.render(c, content, theme),
                Pane::CodeBlock => self.panes.code_block.render(c, content, theme),
                Pane::ContextMenu => self.panes.context_menu.render(c, content, theme),
                Pane::Dialog => self.panes.dialog.render(c, content, theme),
                Pane::Empty => self.panes.empty.render(c, content, theme),
                Pane::Graph => self.panes.graph.render(c, content, theme),
                Pane::IconButton => self.panes.icon_button.render(c, content, theme),
                Pane::Input => self.panes.input.render(c, content, theme),
                Pane::NavigationMenu => self.panes.navigation_menu.render(c, content, theme),
                Pane::Panel => self.panes.panel.render(c, content, theme),
                Pane::PanelHeader => self.panes.panel_header.render(c, content, theme),
                Pane::Progress => self.panes.progress.render(c, content, theme),
                Pane::Resizable => self.panes.resizable.render(c, content, theme),
                Pane::ScrollArea => self
                    .panes
                    .scroll_area
                    .render(c, layers.list, content, theme),
                Pane::Select => self.panes.select.render(c, layers.overlay, content, theme),
                Pane::Separator => self.panes.separator.render(c, content, theme),
                Pane::Sidebar => self.panes.sidebar.render(c, content, theme),
                Pane::Skeleton => self.panes.skeleton.render(c, content, theme),
                Pane::Slider => self.panes.slider.render(c, content, theme),
                Pane::Spinner => self.panes.spinner.render(c, content, theme),
                Pane::Stat => self.panes.stat.render(c, content, theme),
                Pane::Switch => self.panes.switch.render(c, content, theme),
                Pane::Table => self.panes.table.render(c, content, theme),
                Pane::Tabs => self.panes.tabs.render(c, content, theme),
                Pane::Toast => self.panes.toast.render(c, content, theme),
                Pane::Tooltip => {
                    self.panes.tooltip.render(c, content, theme);
                    self.panes.tooltip.render_tooltip(
                        c,
                        layers.toast,
                        theme,
                        self.width,
                        self.height,
                    );
                }
                Pane::Tree => self.panes.tree.render(c, content, theme),
                Pane::Todo => {
                    let inert = self.overlay_holds_pointer();
                    self.panes.todo.set_inert(inert);
                    self.panes.todo.render(c, content, theme);
                }
                Pane::Dock => self.panes.dock.render(c, content, theme),
                Pane::Editor => self.panes.editor.render(c, content),
                Pane::Git => self.panes.git.render(c, content, theme),
                Pane::Builder => self.panes.builder.render(c, content, theme),
            },
        }
    }

    /// What the active page means, when it projects it (the todo page,
    /// today): the snapshot assistive technology is shown. A page without
    /// a projection has no tree yet.
    pub fn semantics(&self) -> Option<Snapshot> {
        match self.pane() {
            Some(Pane::Todo) => self
                .panes
                .todo
                .snapshot()
                .inspect_err(|e| log::warn!("todo: the projection does not validate: {e}"))
                .ok(),
            _ => None,
        }
    }

    /// A request from an actor outside the page's own input (assistive
    /// technology, an agent through the port), through the active page's
    /// one dispatcher (an open overlay leaves the page inert, as for a
    /// click); the receipt as it stands after the dispatch. `None` when
    /// the page takes none.
    pub fn act(&mut self, req: Request) -> Option<Receipt> {
        match self.pane() {
            Some(Pane::Todo) => {
                let inert = self.overlay_holds_pointer();
                self.panes.todo.set_inert(inert);
                let id = self.panes.todo.dispatch(req);
                self.panes.todo.receipt(id).cloned()
            }
            _ => None,
        }
    }

    /// Adds a sink to the dispatcher `act` goes through, so every stage of
    /// every receipt reaches it, later ones included (the todo page's is
    /// the only dispatcher of the gallery today).
    pub fn add_sink(&mut self, sink: Box<dyn ReceiptSink>) {
        self.panes.todo.add_sink(sink);
    }

    /// Keyboard focus from assistive technology to a node of the active
    /// page; `false` when the page does not take it.
    pub fn assist_focus(&mut self, id: &SemId) -> bool {
        match self.pane() {
            Some(Pane::Todo) => {
                let inert = self.overlay_holds_pointer();
                self.panes.todo.set_inert(inert);
                self.panes.todo.focus(id)
            }
            _ => false,
        }
    }

    /// A placeholder page: an empty state on a panel, naming the slot.
    fn render_placeholder(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let rect = Rect::new(
            content.x,
            content.y,
            content.w.min(theme.size.readable_max_w),
            theme.size.card_min_w / 2.0,
        );
        panel(c, rect, theme);
        EmptyState::new(
            self.current().title,
            "Not in comps yet. The slot is here so the tree reads complete.",
        )
        .render(c, rect, theme);
    }
}

/// The gallery through the agent port (task-61): the same hooks a screen
/// reader goes through, desktop only.
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
mod agent {
    use engine::compositor::{Compositor, intersect_rects};
    use sem::{ReceiptSink, Request, RequestId, Snapshot};
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
            port.drain(&mut Served {
                view: self,
                compositor,
            })
        }
    }

    /// The active page as the port serves it: its projection, where its
    /// bound nodes sit, its dispatcher.
    struct Served<'a> {
        view: &'a mut ShowcaseView,
        compositor: &'a Compositor,
    }

    impl Host for Served<'_> {
        fn project(&self) -> Option<Snapshot> {
            self.view.semantics()
        }

        /// The visible part of each binding (its rect inside its clip).
        fn bounds(&self) -> Vec<Bound> {
            let visible = |(id, b): (&sem::SemId, &engine::compositor::Binding)| {
                let [x, y, w, h] = b.clip.map_or(b.rect, |clip| intersect_rects(b.rect, clip));
                (w > 0.0 && h > 0.0).then(|| Bound {
                    id: id.clone(),
                    rect: [x, y, w, h],
                })
            };
            self.compositor.bindings().filter_map(visible).collect()
        }

        fn dispatch(&mut self, req: Request) -> Option<RequestId> {
            self.view.act(req).map(|receipt| receipt.id)
        }

        fn add_sink(&mut self, sink: Box<dyn ReceiptSink>) {
            self.view.add_sink(sink);
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
                let dir =
                    std::env::temp_dir().join(format!("plev-t61-{}-{name}", std::process::id()));
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
            let Stage::Rejected(Rejection::Stale { changes }) = &stale[1] else {
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
}
