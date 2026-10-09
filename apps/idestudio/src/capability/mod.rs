//! The git workspace as a capability: the workbench owns the views
//! (`WorkspaceView`), the git side (`GitSession`) and the one
//! `sem::Dispatcher` every actor goes through. It says what the workspace
//! means (`sem::Semantics`, `project.rs`) and runs what it offers
//! (`sem::Operable`, `execute.rs`). The ide binary and the showcase's Git
//! page both hold one, so neither carries its own copy of the wiring.
//!
//! A person's click or key runs the view first (hit test, cursor, menus),
//! which queues what was asked for; the workbench then observes the
//! workspace at that instant and asks the dispatcher as `Actor::Human`.
//! An agent or a test builds its own `Request` from a snapshot it took
//! and calls [`Workbench::dispatch`]. Either way the effect happens once,
//! in `execute`. What the dispatcher refuses shows as a toast, and a
//! request that waits for a person opens a modal (`person.rs`), or the
//! platform's own dialog when the app chose it
//! ([`Workbench::confirm_natively`]); either answers the same `confirm`
//! or `decline` as the person.
//!
//! With [`Workbench::host_surfaces`] (task-96, `generated.rs`, `column.rs`) the
//! workbench also holds an intent line, in a strip under the fixed views,
//! and the host of generated surfaces, in a column beside them while one
//! is open: projected after the workspace, run by the same dispatcher, and
//! a request aimed at a node of a surface routed to the capability it
//! stands for before the dispatcher takes it.

#[cfg(unix)]
pub mod agent;
mod execute;
pub(super) mod ids;
mod notice;
mod person;
mod project;
#[cfg(test)]
mod tests;
mod truth;

use std::collections::VecDeque;
use std::path::Path;
use std::time::Instant;

use accord::{
    Actor, Dispatcher, Millis, Receipt, ReceiptSink, Request, RequestId, SemError, SemErrors,
    SemId, Semantics, Snapshot, SnapshotBuilder,
};
use engine::drawing::compositor::Compositor;
use engine::runtime::platform::alerts::{Alerts, Parent};
use engine::widgets::prelude::Wake;
use gitdrv::{GitClient, GitEvent};

use crate::generative::Generated;
use crate::interfaces::panels::layout::{UiRequest, WorkspaceView};
use crate::repository::git::GitSession;
use truth::Inflight;

pub use project::{DIFF_LINES, FILES_PROJECTED};

/// The platform's confirmation, when the app chose it over the drawn
/// modal: the panels, and the window they open over (the shell's, once
/// it exists; a test has none).
pub(super) struct NativeConfirm {
    pub(super) alerts: Box<dyn Alerts>,
    pub(super) parent: Option<Box<dyn Parent>>,
}

pub struct Workbench {
    pub(super) workspace: WorkspaceView,
    /// The repository, once connected; a workbench without one draws an
    /// empty workspace and refuses what needs git.
    git: Option<GitSession>,
    /// The one path from a request to `execute`, for every actor.
    pub(super) dispatcher: Dispatcher,
    /// The requests whose execution left for the git worker, oldest
    /// first, with what each waits for; the worker's answer ends them.
    inflight: VecDeque<(RequestId, Inflight)>,
    /// Requests waiting for a person to confirm them, oldest first; the
    /// first one is asked in the modal.
    awaiting: VecDeque<RequestId>,
    /// The platform's dialog asks in the modal's place, when the app
    /// chose it (`confirm_natively`); `None` draws the modal.
    native_confirm: Option<NativeConfirm>,
    /// The monotonic clock receipts are stamped with (`sem` never reads
    /// one).
    clock: Instant,
    /// The window, in logical pixels: the workspace takes it, less the
    /// line's strip and an open surface's column when the block is on.
    pub(super) window: (f32, f32),
    /// The intent line and the host of generated surfaces, when turned on
    /// ([`Workbench::host_surfaces`]).
    pub(super) generated: Option<Box<Generated>>,
}

impl Workbench {
    /// A workspace of `vw` by `vh` logical pixels, not connected yet.
    pub fn new(vw: f32, vh: f32) -> Self {
        Self {
            workspace: WorkspaceView::new(vw, vh),
            git: None,
            dispatcher: Dispatcher::new(),
            inflight: VecDeque::new(),
            awaiting: VecDeque::new(),
            native_confirm: None,
            clock: Instant::now(),
            window: (vw, vh),
            generated: None,
        }
    }

    /// From now on a request that waits for a person is asked in the
    /// platform's dialog (`alerts`, over `parent` when there is one)
    /// instead of the drawn modal, with the same words, and the answer
    /// goes through the same `confirm` or `decline` as `Actor::Human`.
    /// The drawn modal stays the default, and comes back on a target
    /// whose dialogs are `Unsupported`. An app's choice, never a
    /// request's: nothing in the contract changes.
    pub fn confirm_natively(&mut self, alerts: Box<dyn Alerts>, parent: Option<Box<dyn Parent>>) {
        self.native_confirm = Some(NativeConfirm { alerts, parent });
    }

    /// Whether requests that wait for a person are asked in the
    /// platform's dialog.
    pub fn confirms_natively(&self) -> bool {
        self.native_confirm.is_some()
    }

    /// Takes the repository behind `client` and starts loading it (the
    /// results come back as the client's events, for [`Workbench::apply`]).
    pub fn connect(&mut self, client: GitClient) {
        let mut git = GitSession::new(client);
        self.workspace.repo_label = git.repo_label();
        git.refresh();
        self.git = Some(git);
    }

    /// The working tree root of the connected repository.
    pub fn workdir(&self) -> Option<&Path> {
        self.git.as_ref().map(|git| git.client().workdir())
    }

    /// Reloads status, log and branches (results stream back as events).
    /// What was read before a mutation that leaves after this call is not
    /// applied; the refresh that mutation's answer sends is.
    pub fn refresh(&mut self) {
        if let Some(git) = &mut self.git {
            git.refresh();
        }
    }

    pub fn workspace(&self) -> &WorkspaceView {
        &self.workspace
    }

    /// The views, for what is not a capability: layout, hover, scroll,
    /// drag, theme, typing into the commit field.
    pub fn workspace_mut(&mut self) -> &mut WorkspaceView {
        &mut self.workspace
    }

    /// The workspace projected now.
    pub fn snapshot(&self) -> Result<Snapshot, SemErrors> {
        let mut b = SnapshotBuilder::new();
        self.project(&mut b);
        b.finish()
    }

    /// Takes `req` through the dispatcher now: checked against the
    /// workspace as it is at this instant, executed if accepted. The
    /// receipt tells the rest; the person sees a refusal as a toast, and
    /// a request that waits for a person as a modal.
    /// A request aimed at a node of a generated surface is the
    /// capability that node stands for, asked by the same actor
    /// (`SurfaceHost::route`), before the dispatcher takes it.
    pub fn dispatch(&mut self, req: Request) -> RequestId {
        let req = self.route(req);
        let now = self.now();
        // `execute` needs the workbench itself, so the dispatcher is lent
        // out for the call (taking it leaves an empty one behind, no
        // allocation) and comes back with the receipt it kept.
        let mut dispatcher = std::mem::take(&mut self.dispatcher);
        let id = dispatcher.dispatch(self, req, now);
        self.dispatcher = dispatcher;
        self.report(id);
        // a cancel of the Claude's proposal ends that proposal now
        self.settle_claude();
        self.sync_surfaces();
        id
    }

    /// A person (`Actor::Human` at the screen, `Actor::Assistive`)
    /// confirms request `id`, which waits for it; checked again against
    /// the workspace now before it runs. An agent or a test cannot. The
    /// modal asking for it closes.
    pub fn confirm(&mut self, id: RequestId, by: Actor) -> Result<(), SemError> {
        let now = self.now();
        let mut dispatcher = std::mem::take(&mut self.dispatcher);
        let result = dispatcher.confirm(self, id, by, now);
        self.dispatcher = dispatcher;
        result?;
        self.awaiting.retain(|r| *r != id);
        self.workspace.close_confirm_modal(id);
        self.report(id);
        self.ask_next_confirmation();
        self.sync_surfaces();
        Ok(())
    }

    /// A person (`Actor::Human` at the screen, `Actor::Assistive`)
    /// declines request `id`, which waits for it: the receipt ends
    /// rejected (`Rejection::Declined`) and nothing runs. An agent or a
    /// test cannot. The modal asking for it closes, a toast says it was
    /// declined, and the next request waiting for a person is asked.
    pub fn decline(&mut self, id: RequestId, by: Actor) -> Result<(), SemError> {
        let now = self.now();
        self.dispatcher.decline(id, by, now)?;
        self.awaiting.retain(|r| *r != id);
        self.workspace.close_confirm_modal(id);
        self.report(id);
        self.ask_next_confirmation();
        self.sync_surfaces();
        Ok(())
    }

    /// The receipt of `id`, open or among the finished ones kept.
    pub fn receipt(&self, id: RequestId) -> Option<&Receipt> {
        self.dispatcher.receipt(id)
    }

    /// A sink that sees every stage of every receipt as it happens.
    pub fn add_sink(&mut self, sink: Box<dyn ReceiptSink>) {
        self.dispatcher.add_sink(sink);
    }

    /// Applies a result of the git worker: it ends the receipt of the
    /// request it answers (a commit is shown in the views, then completes
    /// with its sha and files; or it is rejected with the delta the index
    /// showed), then updates the views.
    pub fn apply(&mut self, event: GitEvent) {
        self.settle(&event);
        if let Some(git) = &mut self.git {
            git.apply(&mut self.workspace, event);
        }
        self.sync_surfaces();
    }

    /// Advances what animates by `dt` seconds (the real time since the
    /// last frame) and says what it asks of the loop: a frame on every
    /// turn while something moves, the earliest deadline while things
    /// rest (a caret flip, a toast leaving), nothing otherwise.
    /// The Claude's answer, when it arrived, is settled here first.
    pub fn wake(&mut self, dt: f32) -> Wake {
        let answered = self.settle_claude();
        if answered {
            self.sync_surfaces();
        }
        let column = self.generated.as_mut().map_or(Wake::idle(), |g| g.wake(dt));
        let now = if answered { Wake::now() } else { Wake::idle() };
        self.workspace.wake(dt).merge(column).merge(now)
    }

    /// [`Workbench::wake`] as the `bool` a settle loop reads (the
    /// workbench tests let the toasts spring in with it): `true` while
    /// something moves, a frame now or on the very next turn. A focused
    /// caret between flips and a toast at rest are settled.
    pub fn tick(&mut self, dt: f32) -> bool {
        let wake = self.wake(dt);
        wake.now || wake.is_continuous()
    }

    /// Draws the workspace into its frame and binds its ids, then the
    /// line and an open surface's column when the block is on; the host
    /// owns the frame boundary (`Compositor::begin_frame`).
    pub fn render(&mut self, compositor: &mut Compositor) {
        if self.generated.is_some() {
            self.lay_out();
            self.sync_surfaces();
        }
        self.workspace.render(compositor);
        self.render_column(compositor);
    }

    // -- a person's pointer and keys -----------------------------------------

    /// A left click at `(x, y)` in window coordinates: the line's or an
    /// open surface's when it lands on them, else the workspace's. `true`
    /// when something changed.
    pub fn click(&mut self, x: f32, y: f32) -> bool {
        if let Some(changed) = self.click_column(x, y) {
            return changed;
        }
        self.act(|ws| ws.handle_click(x, y))
    }

    /// A right click: the file menu.
    pub fn right_click(&mut self, x: f32, y: f32) -> bool {
        self.act(|ws| ws.handle_right_click(x, y))
    }

    pub fn nav_up(&mut self) -> bool {
        self.act(WorkspaceView::nav_up)
    }

    pub fn nav_down(&mut self) -> bool {
        self.act(WorkspaceView::nav_down)
    }

    /// The diff of the file under the cursor.
    pub fn show_selected_diff(&mut self) -> bool {
        self.act(WorkspaceView::show_selected_diff)
    }

    /// Stage the file under the cursor.
    pub fn stage_selected(&mut self) -> bool {
        self.act(WorkspaceView::stage_selected)
    }

    /// Discard the file under the cursor, after the person confirms.
    pub fn discard_selected(&mut self) -> bool {
        self.act(WorkspaceView::discard_selected)
    }

    /// Open the commit form, or cancel it when open.
    pub fn toggle_commit_form(&mut self) -> bool {
        self.act(WorkspaceView::toggle_commit_form)
    }

    pub fn cancel_commit_form(&mut self) -> bool {
        self.act(WorkspaceView::cancel_commit_form)
    }

    pub fn submit_commit(&mut self) -> bool {
        self.act(WorkspaceView::submit_commit)
    }

    /// Escape on an overlay.
    pub fn close_top_overlay(&mut self) -> bool {
        self.act(WorkspaceView::close_top_overlay)
    }

    /// Runs a person's interaction on the view, then asks the dispatcher
    /// for everything it queued, observed as the workspace is now.
    fn act(&mut self, interact: impl FnOnce(&mut WorkspaceView) -> bool) -> bool {
        let changed = interact(&mut self.workspace);
        let mut asked = false;
        for ask in self.workspace.take_requests() {
            asked |= self.ask(ask);
        }
        // A modal that closed leaves room for a request still waiting.
        self.ask_next_confirmation();
        changed || asked
    }

    /// One thing a person asked for, as a request through the dispatcher
    /// (or, from the confirmation modal, an answer to one). `false` when
    /// it could not be observed (its target left the workspace, or the
    /// projection does not validate).
    fn ask(&mut self, ask: UiRequest) -> bool {
        match ask {
            UiRequest::Confirm { request } => {
                self.confirmed_by_person(request);
                return true;
            }
            UiRequest::Decline { request } => {
                self.declined_by_person(request);
                return true;
            }
            _ => {}
        }
        let Some((target, action)) = self.aim(ask) else {
            return false;
        };
        let seen = match self.snapshot() {
            Ok(seen) => seen,
            Err(errors) => {
                log::warn!("ide: the projection does not validate: {errors}");
                return false;
            }
        };
        match Request::observe(&seen, Actor::Human, &target, action, None) {
            Ok(req) => {
                let id = self.dispatch(req);
                log::debug!("ide: {target} {action}: {:?}", self.receipt(id));
                true
            }
            Err(e) => {
                log::warn!("ide: {e}");
                false
            }
        }
    }

    /// The capability and action a person's ask names.
    fn aim(&self, ask: UiRequest) -> Option<(SemId, &'static str)> {
        let ids = self.workspace.ids.as_ref()?;
        Some(match ask {
            UiRequest::ShowFileDiff { path } => (ids.file(&path), "show_diff"),
            UiRequest::ShowCommitDiff { sha } => (ids.commit(&sha), "show_diff"),
            UiRequest::Stage { path } => (ids.file(&path), "stage"),
            UiRequest::Unstage { path } => (ids.file(&path), "unstage"),
            UiRequest::Discard { path } => (ids.file(&path), "discard"),
            UiRequest::Ignore { path } => (ids.file(&path), "ignore"),
            UiRequest::OpenForm => (ids.form.clone(), "open"),
            UiRequest::CancelForm => (ids.form.clone(), "cancel"),
            UiRequest::Submit => (ids.submit.clone(), "submit"),
            UiRequest::DismissNotice { id } => (ids.notices.key(id.to_string()), "dismiss"),
            UiRequest::Confirm { .. } | UiRequest::Decline { .. } => return None,
        })
    }

    /// Milliseconds since the workbench was made: the clock every receipt
    /// is stamped with, and so the clock anything recorded beside them
    /// (a `rec::Recorder`) must use, or its events and the receipts
    /// would not line up.
    pub fn now(&self) -> Millis {
        u64::try_from(self.clock.elapsed().as_millis()).unwrap_or(u64::MAX)
    }
}
