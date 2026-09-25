//! The workbench's side of the generated surface (task-96,
//! `crate::generated`): the block turned on; the workspace's own
//! projection apart from it, the registry a proposal binds and the state a
//! surface shows; what a request asks of the line, run first by `execute`
//! (propose, close, the field, the proposer); a request aimed at a node of
//! an open surface routed to the capability the IR bound before the
//! dispatcher (`route`), so a person's click, an agent through the port
//! and a test reach the same one; and the window laid out with the
//! column. A person's pointer and keys on the column, and its drawing, are
//! `column.rs`.
//!
//! The rules propose inside `execute`, in a fraction of a millisecond, and
//! the request ends at once, `completed` with the surface and how many
//! nodes it has, or `failed` with why there is none (the line says it
//! too). Only a `VerifiedIr` opens: `propose::propose_verified` is the way.

use propose::{Ask, Rules, propose_verified};
use sem::{Actor, Execution, Outcome, Request, SemErrors, SemId, Snapshot, SnapshotBuilder, Value};
use surface::SurfaceHost;
use uir::{Capabilities, GenerationPolicy, VerifiedIr};

use super::Workbench;
use crate::generated::{Generated, Op, Proposer, Said};

/// The outcome of a proposal shown: the surface and how many nodes it
/// has.
fn shown(surface: &SemId, nodes: usize) -> Outcome {
    let mut outcome = Outcome::default();
    outcome
        .result
        .insert("surface".to_owned(), Value::Text(surface.to_string()));
    let nodes = i64::try_from(nodes).unwrap_or(i64::MAX);
    outcome.result.insert("nodes".to_owned(), Value::Int(nodes));
    outcome
}

impl Workbench {
    /// Turns on the intent line and the host of generated surfaces, in a
    /// column at the right of the window, beside the fixed views. Off by
    /// default: a workbench without it projects, draws and runs as
    /// before. The ide's window turns it on.
    pub fn host_surfaces(&mut self) {
        if self.generated.is_none() {
            self.generated = Some(Box::new(Generated::new()));
            self.lay_out();
        }
    }

    /// Whether the line and the host are on.
    pub fn hosts_surfaces(&self) -> bool {
        self.generated.is_some()
    }

    /// The surfaces open, when the block is on.
    pub fn surfaces(&self) -> Option<&SurfaceHost> {
        self.generated.as_ref().map(|g| &g.host)
    }

    /// The id of the surface open, when one is.
    pub fn surface_open(&self) -> Option<&SemId> {
        self.generated.as_ref().and_then(|g| g.open.as_ref())
    }

    /// The workspace's own projection, without the line and the surface:
    /// the registry every proposal is verified against
    /// (`uir::Capabilities::from_snapshot`) and the state a surface shows.
    /// With the block off it is [`Workbench::snapshot`].
    pub fn app_snapshot(&self) -> Result<Snapshot, SemErrors> {
        let mut b = SnapshotBuilder::new();
        self.project_app(&mut b);
        b.finish()
    }

    /// The window is `w` by `h` logical pixels: the workspace takes it,
    /// less the column when the block is on.
    pub fn resize(&mut self, w: f32, h: f32) {
        self.window = (w, h);
        match self.generated {
            Some(_) => self.lay_out(),
            None => self.workspace.resize(w, h),
        }
    }

    /// The workspace gets the window less the column, when the block is
    /// on; nothing moves otherwise.
    pub(super) fn lay_out(&mut self) {
        let Some(g) = &self.generated else {
            return;
        };
        let (w, h) = self.window;
        let ws = (w - g.column_w(w, self.workspace.theme())).max(0.0);
        if (self.workspace.vw, self.workspace.vh) != (ws, h) {
            self.workspace.resize(ws, h);
        }
    }

    /// `req` as the capability it asks for when it aims at a node of an
    /// open surface; as it is otherwise.
    pub(super) fn route(&self, req: Request) -> Request {
        match &self.generated {
            Some(g) => g.host.route(req),
            None => req,
        }
    }

    /// Every open surface reads the workspace again when it moved.
    pub(super) fn sync_surfaces(&mut self) {
        if self.generated.as_ref().is_none_or(|g| g.host.is_empty()) {
            return;
        }
        match self.app_snapshot() {
            Ok(app) => {
                if let Some(g) = self.generated.as_mut() {
                    g.host.sync(&app);
                }
            }
            Err(e) => log::warn!("ide: the projection does not validate: {e}"),
        }
    }

    /// What `req` asks of the line, run now; `None` when it aims
    /// elsewhere (the workspace, whose `execute` runs it).
    pub(super) fn execute_generated(&mut self, req: &Request) -> Option<Execution> {
        let g = self.generated.as_mut()?;
        let op = g.op(req)?;
        let done = Execution::Done(Outcome::default());
        Some(match op {
            Op::Write(text) => {
                g.intent.set_text(&text);
                done
            }
            Op::Propose => self.propose(&req.actor),
            Op::Choose(name) => {
                if g.proposer.select_label(&name) {
                    done
                } else {
                    Execution::Failed(format!("no proposer is named {name:?}"))
                }
            }
            Op::ChooseAt(at) => {
                if g.proposer.select_index(at) {
                    done
                } else {
                    Execution::Failed(format!("no proposer at {at}"))
                }
            }
            Op::Unfold => {
                g.proposer.open();
                done
            }
            Op::Fold => {
                g.proposer.close();
                done
            }
            Op::Dismiss => self.dismiss(),
        })
    }

    /// A proposal for the intent, as `author` wrote it, over the
    /// workspace as it is (its own registry, never the line's) and the
    /// surface shown: verified, shown, and the request done.
    fn propose(&mut self, author: &Actor) -> Execution {
        let app = match self.app_snapshot() {
            Ok(app) => app,
            Err(e) => return Execution::Failed(e.to_string()),
        };
        let Some(g) = self.generated.as_ref() else {
            return Execution::Failed("no surface is hosted here".to_owned());
        };
        let intent = g.intent.text().trim().to_owned();
        let caps = Capabilities::from_snapshot(&app);
        let policy = GenerationPolicy::by(author.clone());
        let current: Option<VerifiedIr> = g
            .open
            .as_ref()
            .and_then(|id| g.host.current(id))
            .map(|asked| asked.verified().clone());
        let proposed = match g.chosen() {
            Proposer::Rules => {
                let ask = Ask {
                    intent: &intent,
                    caps: &caps,
                    app: &app,
                    current: current.as_ref(),
                    policy: &policy,
                };
                propose_verified(&mut Rules, &ask).map_err(|e| e.to_string())
            }
        };
        match proposed.and_then(|ir| self.show(ir, policy, &app)) {
            Ok((surface, nodes)) => {
                self.say(Said::Proposed(intent));
                Execution::Done(shown(&surface, nodes))
            }
            Err(why) => {
                self.say(Said::Refused(why.clone()));
                Execution::Failed(why)
            }
        }
    }

    fn say(&mut self, said: Said) {
        if let Some(g) = self.generated.as_mut() {
            g.said = Some(said);
        }
    }

    /// Shows `ir`, verified under `policy`, over `app`: the surface
    /// opens, or takes the form in place when its id is open (reconciled
    /// by `SemId`), and the workspace makes room for it. Its id and how
    /// many nodes it has.
    fn show(
        &mut self,
        ir: VerifiedIr,
        policy: GenerationPolicy,
        app: &Snapshot,
    ) -> Result<(SemId, usize), String> {
        let g = self.generated.as_mut().ok_or("no surface is hosted here")?;
        let open = g.host.get(&ir.ir().surface.id).is_some();
        let surface = if open {
            g.host.transform_under(ir, app, policy)
        } else {
            g.host.open_under(ir, app, policy)
        };
        let (id, nodes) = surface
            .map(|s| (s.id().clone(), s.nodes().count()))
            .map_err(|e| e.to_string())?;
        g.open = Some(id.clone());
        self.lay_out();
        Ok((id, nodes))
    }

    /// The surface is dismissed: out of the projection, the routing and
    /// the hit test at once, its picture fading where it was
    /// (`SurfaceHost::close`), the workspace back to its width, and the
    /// line says nothing until the next proposal.
    fn dismiss(&mut self) -> Execution {
        let Some(g) = self.generated.as_mut() else {
            return Execution::Failed("no surface is hosted here".to_owned());
        };
        let Some(id) = g.open.take() else {
            return Execution::Failed("no surface is open".to_owned());
        };
        g.host.close(&id);
        g.said = None;
        self.lay_out();
        let mut outcome = Outcome::default();
        outcome
            .result
            .insert("closed".to_owned(), Value::Text(id.to_string()));
        Execution::Done(outcome)
    }
}
