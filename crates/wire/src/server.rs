//! `Server`, protocol v1 against one app, with no transport: messages in,
//! replies out, addressed to the connection that asked. the port runs it
//! on the app's thread; a test runs it in process.
//!
//! the app is reached through `Host` only: its projection, where its bound
//! nodes sit, its one dispatcher and a `ReceiptSink` on it. nothing here
//! runs app code, reads a file or calls a backend; an act becomes a
//! `sem::Request` with `Actor::Agent` and goes through the dispatcher a
//! click goes through, and its stages come back through the sink.

use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

use sem::{Actor, Millis, Receipt, ReceiptSink, Request, RequestId, SemId, Snapshot, Stage};

use crate::proto::{
    Act, Ask, Bound, ClientMsg, DeltaReply, Observe, ProtoError, Reply, ServerMsg, SnapshotReply,
    StageReply,
};
use crate::redact::Secrets;
use crate::ring::Ring;

/// a connection, numbered by the transport.
pub type ConnId = u64;

/// the longest actor name `hello` takes, in characters.
pub const MAX_NAME: usize = 64;

/// what an app gives the port. the shell implements it over what it
/// already has for a person and for assistive technology; nothing else of
/// the app is reachable.
pub trait Host {
    /// what the app means now (the active page, the workbench), projected
    /// from its live state; `None` when nothing projects.
    fn project(&self) -> Option<Snapshot>;

    /// where each bound node sits in the frame on screen, the part inside
    /// its clip, in logical window pixels. none by default.
    fn bounds(&self) -> Vec<Bound> {
        Vec::new()
    }

    /// runs `req` through the app's one dispatcher, stamped with the
    /// dispatcher's own clock (the one a click is stamped with), and
    /// returns its id; `None` when nothing takes requests now.
    fn dispatch(&mut self, req: Request) -> Option<RequestId>;

    /// adds the port's sink to that dispatcher. called once, before the
    /// first dispatch; every stage of every receipt, now or later, must
    /// reach it.
    fn add_sink(&mut self, sink: Box<dyn ReceiptSink>);
}

/// one reply for one connection.
#[derive(Clone, Debug, PartialEq)]
pub struct Outgoing {
    pub conn: ConnId,
    pub msg: ServerMsg,
}

/// the protocol state of one port: the actor of each connection, the
/// revisions served, and the requests its agents issued, whose stages it
/// forwards to the connection that asked.
pub struct Server {
    app: String,
    ring: Ring,
    names: HashMap<ConnId, String>,
    issued: HashMap<RequestId, Issued>,
    heard: Rc<RefCell<VecDeque<Heard>>>,
    listening: bool,
}

/// a request an agent of this port issued, until its terminal stage.
struct Issued {
    conn: ConnId,
    msg: String,
    target: SemId,
    /// the request as dispatched, redacted; sent with `requested`.
    asked: Option<Request>,
    secrets: Secrets,
}

/// a stage the sink heard.
struct Heard {
    id: RequestId,
    stage: Stage,
    at: Millis,
}

/// the port's ear on the dispatcher: every stage of an agent's request,
/// kept until the port forwards the ones it issued.
struct PortSink(Rc<RefCell<VecDeque<Heard>>>);

impl ReceiptSink for PortSink {
    fn on_stage(&mut self, receipt: &Receipt, stage: &Stage) {
        if !matches!(receipt.request.actor, Actor::Agent { .. }) {
            return;
        }
        let at = receipt.stages.last().map_or(0, |(_, at)| *at);
        if let Ok(mut heard) = self.0.try_borrow_mut() {
            heard.push_back(Heard {
                id: receipt.id,
                stage: stage.clone(),
                at,
            });
        }
    }
}

impl Server {
    /// a server for `app` keeping the last `ring` revisions it serves.
    pub fn new(app: impl Into<String>, ring: usize) -> Self {
        Server {
            app: app.into(),
            ring: Ring::new(ring),
            names: HashMap::new(),
            issued: HashMap::new(),
            heard: Rc::new(RefCell::new(VecDeque::new())),
            listening: false,
        }
    }

    /// the revisions served and still held.
    pub fn ring(&self) -> &Ring {
        &self.ring
    }

    /// takes one message of `conn`; returns the replies, the stages of
    /// an act among them, and any stage that ended since.
    pub fn handle(&mut self, conn: ConnId, msg: ClientMsg, host: &mut dyn Host) -> Vec<Outgoing> {
        let ClientMsg { id, ask } = msg;
        let reply = match ask {
            Ask::Hello { name } => self.hello(conn, name),
            Ask::Observe(o) => self.observe(&o, host),
            Ask::Act(a) => self.act(conn, &id, a, host).map(|()| None),
            Ask::Confirm { request } | Ask::Decline { request } => Err(ProtoError::NotEnabled {
                reason: format!(
                    "only a person confirms or declines request {request}, at the screen \
                         or through assistive technology; this connection acts as {}",
                    self.actor(conn)
                        .map_or_else(|| "an agent".to_owned(), |a| a.to_string())
                ),
            }),
        };
        let mut out = Vec::new();
        match reply {
            Ok(Some(reply)) => out.push(Outgoing {
                conn,
                msg: ServerMsg {
                    id: Some(id),
                    reply,
                },
            }),
            Ok(None) => {}
            Err(error) => out.push(Outgoing {
                conn,
                msg: ServerMsg {
                    id: Some(id),
                    reply: Reply::Error(error),
                },
            }),
        }
        out.extend(self.forward());
        out
    }

    /// the stages the sink heard for requests this port issued, in the
    /// order they happened, each to the connection that asked. a request
    /// is forgotten after its terminal stage.
    pub fn forward(&mut self) -> Vec<Outgoing> {
        let heard: Vec<Heard> = match self.heard.try_borrow_mut() {
            Ok(mut q) => q.drain(..).collect(),
            Err(_) => return Vec::new(),
        };
        let mut out = Vec::new();
        for h in heard {
            let Some(issued) = self.issued.get_mut(&h.id) else {
                continue;
            };
            let asked = match h.stage {
                Stage::Requested => issued.asked.take(),
                _ => None,
            };
            out.push(Outgoing {
                conn: issued.conn,
                msg: ServerMsg {
                    id: Some(issued.msg.clone()),
                    reply: Reply::Stage(StageReply {
                        request: h.id,
                        stage: issued.secrets.stage(&issued.target, &h.stage),
                        at: h.at,
                        asked,
                    }),
                },
            });
            if h.stage.is_terminal() {
                self.issued.remove(&h.id);
            }
        }
        out
    }

    /// forgets a connection that ended, with the requests it was waiting
    /// on (they still run; nobody listens).
    pub fn close(&mut self, conn: ConnId) {
        self.names.remove(&conn);
        self.issued.retain(|_, issued| issued.conn != conn);
    }

    /// the actor of `conn`, once it said hello.
    pub fn actor(&self, conn: ConnId) -> Option<Actor> {
        self.names
            .get(&conn)
            .map(|name| Actor::Agent { name: name.clone() })
    }

    fn hello(&mut self, conn: ConnId, name: String) -> Result<Option<Reply>, ProtoError> {
        if let Some(actor) = self.actor(conn) {
            return Err(bad(format!(
                "this connection already said hello, as {actor}"
            )));
        }
        let chars = name.chars().count();
        if name.trim().is_empty() || chars > MAX_NAME || name.chars().any(char::is_control) {
            return Err(bad(format!(
                "an agent's name is 1 to {MAX_NAME} characters, not blank, with no control \
                 character; got {chars} characters"
            )));
        }
        self.names.insert(conn, name.clone());
        Ok(Some(Reply::Welcome {
            app: self.app.clone(),
            actor: Actor::Agent { name },
        }))
    }

    fn observe(&mut self, o: &Observe, host: &mut dyn Host) -> Result<Option<Reply>, ProtoError> {
        let Some(live) = host.project() else {
            return Err(ProtoError::NotEnabled {
                reason: "the app projects nothing now".to_owned(),
            });
        };
        let base = o.since.and_then(|r| {
            self.ring
                .get(r)
                .map(|seen| (r, scoped(seen, o.scope.as_ref())))
        });
        let now = scoped(&live, o.scope.as_ref());
        let revision = self.ring.serve(live);
        let bounds = o.bounds.then(|| {
            let all = host.bounds();
            all.into_iter()
                .filter(|b| now.get(&b.id).is_some())
                .collect()
        });
        let keep = |key: &sem::StateKey| o.keys.as_ref().is_none_or(|keys| keys.contains(key));
        let reply = match base {
            Some((base, seen)) => Reply::Delta(DeltaReply {
                base,
                revision,
                changes: Snapshot::diff(&seen, &now)
                    .into_iter()
                    .filter(|c| match c {
                        sem::Change::StateChanged { at, .. } => keep(&at.key),
                        _ => true,
                    })
                    .collect(),
                bounds,
            }),
            None => Reply::Snapshot(SnapshotReply {
                revision,
                nodes: now
                    .nodes()
                    .values()
                    .map(|node| {
                        let mut node = node.clone();
                        node.state.retain(|key, _| keep(key));
                        if !o.source {
                            node.source = None;
                        }
                        node
                    })
                    .collect(),
                focused: now.focused().cloned(),
                bounds,
            }),
        };
        Ok(Some(reply))
    }

    fn act(
        &mut self,
        conn: ConnId,
        msg: &str,
        a: Act,
        host: &mut dyn Host,
    ) -> Result<(), ProtoError> {
        let Some(actor) = self.actor(conn) else {
            return Err(ProtoError::NotEnabled {
                reason: "say hello first: an agent acts under the name it declares".to_owned(),
            });
        };
        let req = match (a.observed, a.from_revision) {
            (Some(observed), None) => Request {
                actor,
                target: a.target,
                action: a.action,
                param: a.param,
                observed,
            },
            (None, Some(revision)) => {
                let Some(seen) = self.ring.get(revision) else {
                    return Err(ProtoError::UnknownRevision { revision });
                };
                Request::observe(seen, actor, a.target, a.action, a.param)
                    .map_err(|e| bad(format!("in revision {revision}: {e}")))?
            }
            (Some(_), Some(_)) => {
                return Err(bad(
                    "an act rests on observed or on from_revision, not on both",
                ));
            }
            (None, None) => {
                return Err(bad(
                    "an act rests on the values it observed or on from_revision",
                ));
            }
        };
        let secrets = Secrets::of(&req, host.project().as_ref());
        let asked = secrets.request(&req);
        let target = req.target.clone();
        if !self.listening {
            host.add_sink(Box::new(PortSink(Rc::clone(&self.heard))));
            self.listening = true;
        }
        let Some(id) = host.dispatch(req) else {
            return Err(ProtoError::NotEnabled {
                reason: "nothing takes requests now".to_owned(),
            });
        };
        let heard = self
            .heard
            .try_borrow()
            .is_ok_and(|q| q.iter().any(|h| h.id == id));
        if !heard {
            return Err(ProtoError::NotEnabled {
                reason: format!(
                    "request {id} was dispatched, but the app's dispatcher reports no stage \
                     to the port; how it ends is unknown"
                ),
            });
        }
        self.issued.insert(
            id,
            Issued {
                conn,
                msg: msg.to_owned(),
                target,
                asked: Some(asked),
                secrets,
            },
        );
        Ok(())
    }
}

/// `scope` and its descendants, or the whole snapshot.
fn scoped(snapshot: &Snapshot, scope: Option<&SemId>) -> Snapshot {
    match scope {
        Some(id) => snapshot.subtree(id),
        None => snapshot.clone(),
    }
}

fn bad(reason: impl Into<String>) -> ProtoError {
    ProtoError::BadMessage {
        reason: reason.into(),
    }
}
