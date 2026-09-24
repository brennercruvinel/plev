//! The workbench through the agent port (task-86): what the ide's window
//! shell serves when `PLEV_AGENT=1` in a debug build, and what any host of
//! a `Workbench` serves the same way. Unix only, like the port.
//!
//! An agent reaches the workbench through the hooks a person's click
//! already goes through: the projection (`Workbench::snapshot`), the one
//! dispatcher (`Workbench::dispatch`, stamped with the workbench's clock)
//! and a sink on it. Where each id sits is the frame on screen, read from
//! the compositor that drew it.
//!
//! A stage, an unstage or a commit ends on the git worker, so its last
//! stage exists only once the worker's answer is applied
//! (`Workbench::apply`). The shell serves the port after the port's wake
//! and again after every worker event it applies. A stage that ends
//! outside a drain (a person's answer in the modal, a worker's answer)
//! also wakes the port itself (task-92), so it never waits for the
//! agent's next message.
//!
//! When the shell records the session (`crate::record`), it serves through
//! `serve_recorded`: every observation served is recorded with its actor,
//! scope and revision, on the workbench's clock, and the state an act left
//! right after the drain that ran it.

use engine::compositor::Compositor;
use rec::Recorder;
use sem::{Actor, ReceiptSink, Request, RequestId, Revision, SemId, Snapshot};
use wire::{AgentPort, Bound, Host};

use crate::record::Recording;
use crate::workbench::Workbench;

/// The name the ide's port opens under: its socket is
/// `$TMPDIR/plev/ide-<pid>.sock`, and `plev --app ide` finds it.
pub const APP: &str = "ide";

/// The ide's agent port when `PLEV_AGENT=1` in a debug build; `None`
/// otherwise (nothing is created), or when it cannot open (logged).
/// `wake` makes the shell serve it.
pub fn port(wake: impl Fn() + Send + 'static) -> Option<AgentPort> {
    AgentPort::from_env(APP, wake)
        .inspect_err(|e| log::error!("agent port: {e}"))
        .ok()
        .flatten()
}

/// What agents asked through `port` since the last call, served on this
/// thread through `bench`, with `compositor` holding the frame on screen,
/// and every stage that ended since sent. True when an act was
/// dispatched, so the workbench may have changed and the shell draws;
/// an observation changes nothing.
pub fn serve(port: &mut AgentPort, bench: &mut Workbench, compositor: &Compositor) -> bool {
    serve_recorded(port, bench, compositor, None)
}

/// [`serve`], with `recording` (when there is one) keeping who observed
/// what and, when an act ran, the state it left.
pub fn serve_recorded(
    port: &mut AgentPort,
    bench: &mut Workbench,
    compositor: &Compositor,
    recording: Option<&Recording>,
) -> bool {
    let recorder = recording.map(Recording::recorder);
    let acted = port.drain(&mut Served {
        bench,
        compositor,
        recorder,
    });
    if let Some(recording) = recording.filter(|_| acted) {
        recording.state(bench);
    }
    acted
}

/// The workbench as the port serves it for one drain.
pub struct Served<'a> {
    pub bench: &'a mut Workbench,
    /// The frame on screen: where each bound id sits.
    pub compositor: &'a Compositor,
    /// Records every observation served, when the session is recorded.
    pub recorder: Option<&'a Recorder>,
}

impl Host for Served<'_> {
    fn project(&self) -> Option<Snapshot> {
        self.bench
            .snapshot()
            .inspect_err(|e| log::warn!("agent port: the projection does not validate: {e}"))
            .ok()
    }

    /// The visible part of each binding of the frame on screen.
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
        Some(self.bench.dispatch(req))
    }

    fn add_sink(&mut self, sink: Box<dyn ReceiptSink>) {
        self.bench.add_sink(sink);
    }

    /// An observation was served: recorded at the workbench's clock.
    fn observed(&mut self, actor: &Actor, scope: Option<&SemId>, revision: Revision) {
        if let Some(rec) = self.recorder {
            rec.observed(self.bench.now(), actor.clone(), scope.cloned(), revision);
        }
    }
}
