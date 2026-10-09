//! `Server`, protocol v1 against one app, with no transport: messages in,
//! replies out, addressed to the connection that asked. the port runs it
//! on the app's thread; a test runs it in process.
//!
//! the app is reached through `Host` only: its projection, where its bound
//! nodes sit, its one dispatcher and a `ReceiptSink` on it. nothing here
//! runs app code, reads a file or calls a backend; an act becomes a
//! `sem::Request` with `Actor::Agent` and goes through the dispatcher a
//! click goes through, and its stages come back through the sink.
//!
//! an act on `from_revision` rests on what its actor was shown, and on
//! nothing else (`Shown`): the revision must have been given to that
//! actor, the action's declaration comes from the revision, and the value
//! of each state it reads comes from the last observation up to that
//! revision that showed it to the actor. a read never shown is refused
//! before the dispatcher (`ProtoError::NotObserved`).
//!
//! a stale refusal of an act the port dispatched names what ran since its
//! actor looked (task-91). the port's sink follows the dispatcher's order
//! (`sem::Mark`) and marks each revision it gives an actor; an act rests
//! on the oldest observation its reads come from (with `observed` given,
//! on the last giving of the revision it names), and the sink hands that
//! look to the dispatcher when it fills `acted` (`ReceiptSink::looked`).
//!
//! a secret never leaves through the port. the request as dispatched and
//! every stage of it are reduced by `sem::redact`, the one rule the
//! recorder applies too, with the secret nodes read from the live snapshot
//! at dispatch (a node it does not hold counts as secret).

use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

use sem::redact::Secrets;
use sem::{
    Actor, Change, Mark, Node, ReceiptSink, Rejection, Request, RequestId, Revision, SemId,
    Snapshot, Stage, StateRef,
};

use crate::proto::{
    Act, Ask, Bound, ClientMsg, DeltaReply, Observe, ProtoError, Reply, ServerMsg, SnapshotReply,
    StageReply,
};
use crate::ring::Ring;
use crate::scene::{Offer, SceneAsk, SceneWatch, Watches};
use crate::shown::{Looks, Part, Shown, keeps};
use crate::sink::{Heard, PortSink, Waits};
pub use crate::traffic::Traffic;

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

    /// adds the port's sink to that dispatcher. called once, at the first
    /// observation served to an actor or the first act; every stage of
    /// every receipt, now or later, must reach it. through it the port
    /// also follows the dispatcher's order, so a stale refusal of its acts
    /// names what ran since the agent looked; a sink that is dropped leaves
    /// that to the rule without a look. a stage it hears of an agent's
    /// request that ends later wakes the port (`Server::wake_with`).
    fn add_sink(&mut self, sink: Box<dyn ReceiptSink>);

    /// `actor` was just served an observation: the whole snapshot (`scope`
    /// none) or the subtree under `scope`, at `revision`, the revision of
    /// the whole snapshot at that instant. called right after the reply is
    /// built, on the app's thread, only for a connection that said hello
    /// (before it there is no actor). an app records it
    /// (`rec::Recorder::observed`) or reacts to it; nothing by default.
    /// the revision a terminal stage carries is not an observation (the
    /// agent is given its name, not what it holds) and is not reported
    /// here.
    fn observed(&mut self, actor: &Actor, scope: Option<&SemId>, revision: Revision) {
        let _ = (actor, scope, revision);
    }
}

/// one reply for one connection.
#[derive(Clone, Debug, PartialEq)]
pub struct Outgoing {
    pub conn: ConnId,
    pub msg: ServerMsg,
}

/// the protocol state of one port: the actor of each connection, the
/// revisions served, what each actor was shown, and the requests its
/// agents issued, whose stages it forwards to the connection that asked.
pub struct Server {
    app: String,
    ring: Ring,
    shown: Shown,
    names: HashMap<ConnId, String>,
    issued: HashMap<RequestId, Issued>,
    heard: Rc<RefCell<VecDeque<Heard>>>,
    looks: Rc<RefCell<Looks>>,
    waits: Rc<Waits>,
    listening: bool,
    /// what this server answered; the transport fills the rest.
    traffic: Traffic,
    /// the connections that watch a window's scene (task-79).
    scenes: Watches,
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

impl Server {
    /// a server for `app` keeping the last `ring` revisions it serves,
    /// and the last `ring` revisions it gave each actor.
    pub fn new(app: impl Into<String>, ring: usize) -> Self {
        let ring = Ring::new(ring);
        Server {
            app: app.into(),
            shown: Shown::new(ring.cap()),
            ring,
            names: HashMap::new(),
            issued: HashMap::new(),
            heard: Rc::new(RefCell::new(VecDeque::new())),
            looks: Rc::new(RefCell::new(Looks::default())),
            waits: Rc::new(Waits::default()),
            listening: false,
            traffic: Traffic::default(),
            scenes: Watches::default(),
        }
    }

    /// the revisions served and still held.
    pub fn ring(&self) -> &Ring {
        &self.ring
    }

    /// what this server answered so far: observations by kind, acts,
    /// stages sent, stale rejections among them, errors. connections,
    /// messages and bytes are the transport's (`AgentPort::traffic`) and
    /// stay zero here.
    pub fn traffic(&self) -> Traffic {
        self.traffic
    }

    /// takes one message of `conn`; returns the replies, the stages of
    /// an act among them, and any stage that ended since.
    pub fn handle(&mut self, conn: ConnId, msg: ClientMsg, host: &mut dyn Host) -> Vec<Outgoing> {
        let ClientMsg { id, ask } = msg;
        let reply = match ask {
            Ask::Hello { name } => self.hello(conn, name),
            Ask::Observe(o) => self.observe(conn, &o, host),
            Ask::Act(a) => {
                self.traffic.acts += 1;
                match (&a.observed, &a.from_revision) {
                    (None, Some(_)) => self.traffic.acts_from_revision += 1,
                    (Some(_), None) => self.traffic.acts_observed += 1,
                    _ => {}
                }
                self.act(conn, &id, a, host).map(|()| None)
            }
            Ask::Confirm { request } | Ask::Decline { request } => Err(ProtoError::NotEnabled {
                reason: format!(
                    "only a person confirms or declines request {request}, at the screen \
                         or through assistive technology; this connection acts as {}",
                    self.actor(conn)
                        .map_or_else(|| "an agent".to_owned(), |a| a.to_string())
                ),
            }),
            Ask::Scene(s) => self.watch_scene(conn, &id, s, host),
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
            Err(error) => {
                self.traffic.errors += 1;
                out.push(Outgoing {
                    conn,
                    msg: ServerMsg {
                        id: Some(id),
                        reply: Reply::Error(error),
                    },
                });
            }
        }
        out.extend(self.forward(host));
        out
    }

    /// the stages the sink heard for requests this port issued, in the
    /// order they happened, each to the connection that asked. a request
    /// is forgotten after its terminal stage.
    ///
    /// a terminal stage carries the revision the app is at now, projected
    /// through `host` and held in the ring as an observation's revision
    /// is. the app is projected once per call, and only when a terminal
    /// stage goes out: nothing runs on this thread between two stages of
    /// one call, so they all leave it at the same revision. it is not an
    /// observation: the traffic's observations and `Host::observed` do not
    /// count it, and it shows the actor no state.
    pub fn forward(&mut self, host: &mut dyn Host) -> Vec<Outgoing> {
        let heard: Vec<Heard> = match self.heard.try_borrow_mut() {
            Ok(mut q) => q.drain(..).collect(),
            Err(_) => return Vec::new(),
        };
        let ends = heard
            .iter()
            .any(|h| h.stage.is_terminal() && self.issued.contains_key(&h.id));
        let now = if ends {
            host.project().map(|live| self.ring.serve(live))
        } else {
            None
        };
        let mark = self.mark();
        let mut out = Vec::new();
        for h in heard {
            let Some(issued) = self.issued.get_mut(&h.id) else {
                continue;
            };
            let asked = match h.stage {
                Stage::Requested => issued.asked.take(),
                _ => None,
            };
            let revision = if h.stage.is_terminal() { now } else { None };
            if let (Some(r), Some(name)) = (revision, self.names.get(&issued.conn)) {
                self.shown.record(name, r, Part::Nothing, mark);
            }
            self.traffic.stages_out += 1;
            if matches!(h.stage, Stage::Rejected(Rejection::Stale { .. })) {
                self.traffic.rejected_stale += 1;
            }
            out.push(Outgoing {
                conn: issued.conn,
                msg: ServerMsg {
                    id: Some(issued.msg.clone()),
                    reply: Reply::Stage(StageReply {
                        request: h.id,
                        stage: issued.secrets.stage(&issued.target, &h.stage),
                        at: h.at,
                        asked,
                        revision,
                    }),
                },
            });
            if h.stage.is_terminal() {
                self.issued.remove(&h.id);
            }
        }
        self.waiting();
        out
    }

    /// forgets a connection that ended, with the requests it was waiting
    /// on (they still run; nobody listens). what its actor was shown
    /// stays: another connection under the same name is the same actor.
    pub fn close(&mut self, conn: ConnId) {
        self.names.remove(&conn);
        self.scenes.close(conn);
        self.issued.retain(|_, issued| issued.conn != conn);
        self.waiting();
    }

    /// whether a connection watches the scene of `window`: an app writes
    /// that window's frames only then (task-79).
    pub fn watching(&self, window: &str) -> bool {
        self.scenes.watching(window)
    }

    /// whether the person who launched the app shares its windows'
    /// scenes (`PLEV_MIRROR=1`); off, a `scene` request is refused with
    /// `scene_not_shared`. off by default.
    pub fn share_scenes(&mut self, on: bool) {
        self.scenes.share(on);
    }

    /// the watches that began or ended since the last call, in order,
    /// each with its actor and window: for the app to say who watches.
    pub fn take_scene_watches(&mut self) -> Vec<SceneWatch> {
        self.scenes.take_events()
    }

    /// a watch of a window's scene: refused unless the scenes are shared,
    /// and before `hello`. it shows its actor the whole window from now
    /// on, so the host is told of it as of an observation of the whole
    /// app at the revision it projects now (`Host::observed`).
    fn watch_scene(
        &mut self,
        conn: ConnId,
        msg: &str,
        ask: SceneAsk,
        host: &mut dyn Host,
    ) -> Result<Option<Reply>, ProtoError> {
        let actor = self.actor(conn);
        self.scenes.watch(conn, msg, ask, actor.clone())?;
        if let (Some(actor), Some(now)) = (actor, host.project()) {
            host.observed(&actor, None, now.revision());
        }
        Ok(None)
    }

    /// the lines each watch of `window` needs of `offer`, the frame the
    /// app drew there: none for a watch it shows nothing new.
    pub fn offer_scene(&mut self, window: &str, offer: &Offer) -> Vec<Outgoing> {
        self.scenes.offer(window, offer)
    }

    /// the wake of the app's loop, the one the port's reader threads call
    /// (task-92). from then on a stage of a request this server issued,
    /// heard after the act that dispatched it, calls it, so a stage that
    /// ends outside a drain (a person's answer to a confirmation, a
    /// worker's answer) goes out in the drain that follows. a server
    /// without one leaves the stages queued until `forward` is called.
    pub fn wake_with(&mut self, wake: impl Fn() + 'static) {
        if let Ok(mut slot) = self.waits.wake.try_borrow_mut() {
            *slot = Some(Box::new(wake));
        }
    }

    /// the sink's copy of the requests this server forwards, after they
    /// changed.
    fn waiting(&self) {
        if let Ok(mut ids) = self.waits.ids.try_borrow_mut() {
            *ids = self.issued.keys().copied().collect();
        }
    }

    /// adds the port's sink to the app's dispatcher, once.
    fn listen(&mut self, host: &mut dyn Host) {
        if !self.listening {
            host.add_sink(Box::new(PortSink {
                heard: Rc::clone(&self.heard),
                looks: Rc::clone(&self.looks),
                waits: Rc::clone(&self.waits),
            }));
            self.listening = true;
        }
    }

    /// the dispatcher's mark now, as the port's sink follows it.
    fn mark(&self) -> Option<Mark> {
        self.looks.try_borrow().ok()?.now()
    }

    /// the look the next request of this actor rests on, for the sink to
    /// file when the dispatcher takes it; `None` expects nothing.
    fn expect(&self, next: Option<(Actor, Mark)>) {
        if let Ok(mut looks) = self.looks.try_borrow_mut() {
            looks.expect(next);
        }
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

    /// serves an observation and, when `conn` has an actor, remembers what
    /// it showed that actor and tells the host who was served what
    /// (`Host::observed`), after the reply is built.
    fn observe(
        &mut self,
        conn: ConnId,
        o: &Observe,
        host: &mut dyn Host,
    ) -> Result<Option<Reply>, ProtoError> {
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
        match base {
            Some(_) => self.traffic.observes_delta += 1,
            None => self.traffic.observes_snapshot += 1,
        }
        let reply = match base {
            Some((base, seen)) => Reply::Delta(DeltaReply {
                base,
                revision,
                changes: Snapshot::diff(&seen, &now)
                    .into_iter()
                    .filter_map(|c| match c {
                        Change::StateChanged { ref at, .. } if !keeps(&o.keys, &at.key) => None,
                        Change::NodeAdded(mut node) => {
                            serve(&mut node, o);
                            Some(Change::NodeAdded(node))
                        }
                        other => Some(other),
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
                        serve(&mut node, o);
                        node
                    })
                    .collect(),
                focused: now.focused().cloned(),
                bounds,
            }),
        };
        if self.names.contains_key(&conn) {
            self.listen(host);
        }
        let mark = self.mark();
        if let Some(name) = self.names.get(&conn) {
            let part = match &reply {
                Reply::Delta(d) => Part::moved(&d.changes, o.keys.clone()),
                _ => Part::Cut {
                    scope: o.scope.clone(),
                    keys: o.keys.clone(),
                },
            };
            self.shown.record(name, revision, part, mark);
        }
        if let Some(actor) = self.actor(conn) {
            host.observed(&actor, o.scope.as_ref(), revision);
        }
        Ok(Some(reply))
    }

    fn act(
        &mut self,
        conn: ConnId,
        msg: &str,
        mut a: Act,
        host: &mut dyn Host,
    ) -> Result<(), ProtoError> {
        let Some(actor) = self.actor(conn) else {
            return Err(ProtoError::NotEnabled {
                reason: "say hello first: an agent acts under the name it declares".to_owned(),
            });
        };
        if let Some(fault) = a.ask_fault() {
            return Err(bad(fault));
        }
        let (req, look) = match (a.observed.take(), a.from_revision) {
            (Some(observed), None) => {
                let name = self.names.get(&conn).map_or("", String::as_str);
                let look = self.shown.looked(name, observed.revision);
                let req = Request {
                    actor,
                    target: a.target,
                    action: a.action,
                    param: a.param,
                    observed,
                    ask: a.ask,
                };
                (req, look)
            }
            (None, Some(revision)) => self.rest(conn, actor, a, revision)?,
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
        self.listen(host);
        self.expect(look.map(|look| (req.actor.clone(), look)));
        let id = host.dispatch(req);
        self.expect(None);
        let Some(id) = id else {
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
        self.waiting();
        Ok(())
    }

    /// the request of an act on `revision`: the revision must have been
    /// given to the actor of `conn` and still be held; the action and what
    /// it reads come from the revision's snapshot, and the value of each
    /// read from what the actor was shown up to that revision. the
    /// observation keeps `revision`, to trace the request. with it, the
    /// look it rests on: the oldest mark of the observations its values
    /// came from.
    fn rest(
        &self,
        conn: ConnId,
        actor: Actor,
        a: Act,
        revision: Revision,
    ) -> Result<(Request, Option<Mark>), ProtoError> {
        let name = self.names.get(&conn).map_or("", String::as_str);
        let seen = self
            .ring
            .get(revision)
            .filter(|_| self.shown.given(name, revision));
        let Some(seen) = seen else {
            return Err(ProtoError::UnknownRevision { revision });
        };
        let mut req = Request::observe(seen, actor, a.target, a.action, a.param)
            .map_err(|e| bad(format!("in revision {revision}: {e}")))?;
        req.ask = a.ask;
        let reads: Vec<StateRef> = req.observed.values.drain(..).map(|(r, _)| r).collect();
        let rested = self
            .shown
            .values(name, revision, &reads, &self.ring)
            .map_err(|reads| ProtoError::NotObserved { reads })?;
        req.observed.values = reads.into_iter().zip(rested.values).collect();
        Ok((req, rested.look))
    }
}

/// a node as an observation serves it, whether a snapshot holds it or a
/// delta adds it: its state cut to the keys asked for, and without
/// `source` unless asked.
fn serve(node: &mut Node, o: &Observe) {
    node.state.retain(|key, _| keeps(&o.keys, key));
    if !o.source {
        node.source = None;
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
