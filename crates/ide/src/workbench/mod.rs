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
//! request that waits for a person opens a modal (`person.rs`).

mod execute;
mod notice;
mod person;
mod project;
#[cfg(test)]
mod tests;
mod truth;

use std::collections::VecDeque;
use std::path::Path;
use std::time::Instant;

use engine::compositor::Compositor;
use git::{GitClient, GitEvent};
use sem::{
    Actor, Dispatcher, Millis, Receipt, ReceiptSink, Request, RequestId, SemError, SemErrors,
    SemId, Semantics, Snapshot, SnapshotBuilder,
};

use crate::session::GitSession;
use crate::views::workspace::{UiRequest, WorkspaceView};
use truth::Inflight;

pub use project::DIFF_LINES;

pub struct Workbench {
    workspace: WorkspaceView,
    /// The repository, once connected; a workbench without one draws an
    /// empty workspace and refuses what needs git.
    git: Option<GitSession>,
    /// The one path from a request to `execute`, for every actor.
    dispatcher: Dispatcher,
    /// The requests whose execution left for the git worker, oldest
    /// first, with what each waits for; the worker's answer ends them.
    inflight: VecDeque<(RequestId, Inflight)>,
    /// Requests waiting for a person to confirm them, oldest first; the
    /// first one is asked in the modal.
    awaiting: VecDeque<RequestId>,
    /// The monotonic clock receipts are stamped with (`sem` never reads
    /// one).
    clock: Instant,
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
            clock: Instant::now(),
        }
    }

    /// Takes the repository behind `client` and starts loading it (the
    /// results come back as the client's events, for [`Workbench::apply`]).
    pub fn connect(&mut self, client: GitClient) {
        let git = GitSession::new(client);
        self.workspace.repo_label = git.repo_label();
        git.refresh();
        self.git = Some(git);
    }

    /// The working tree root of the connected repository.
    pub fn workdir(&self) -> Option<&Path> {
        self.git.as_ref().map(|git| git.client().workdir())
    }

    /// Reloads status, log and branches (results stream back as events).
    pub fn refresh(&self) {
        if let Some(git) = &self.git {
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
    pub fn dispatch(&mut self, req: Request) -> RequestId {
        let now = self.now();
        // `execute` needs the workbench itself, so the dispatcher is lent
        // out for the call (taking it leaves an empty one behind, no
        // allocation) and comes back with the receipt it kept.
        let mut dispatcher = std::mem::take(&mut self.dispatcher);
        let id = dispatcher.dispatch(self, req, now);
        self.dispatcher = dispatcher;
        self.report(id);
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
    /// request it answers (a commit completes with its sha and files, or
    /// is rejected with the delta the index showed), then updates the
    /// views.
    pub fn apply(&mut self, event: GitEvent) {
        self.settle(&event);
        if let Some(git) = &mut self.git {
            git.apply(&mut self.workspace, event);
        }
    }

    /// Advances what animates; `true` while frames are needed.
    pub fn tick(&mut self, dt: f32) -> bool {
        self.workspace.tick(dt)
    }

    /// Draws the workspace into its frame and binds its ids; the host owns
    /// the frame boundary (`Compositor::begin_frame`).
    pub fn render(&mut self, compositor: &mut Compositor) {
        self.workspace.render(compositor);
    }

    // -- a person's pointer and keys -----------------------------------------

    /// A left click at `(x, y)` in window coordinates. `true` when
    /// something changed.
    pub fn click(&mut self, x: f32, y: f32) -> bool {
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
