//! `AgentPort`, the unix socket an app opens for agents, and the only way
//! in from outside the process. off by default.
//!
//! threads: one accepts connections; per connection one reads lines,
//! decodes them and queues each message for the app's thread, and one
//! writes the replies. a thread never touches the app. it queues and
//! wakes the loop through the closure the shell gave (its event loop
//! proxy); the shell then calls `drain` on the app's thread, which
//! projects, dispatches and answers there, through `Host`. the same wake
//! is called on the app's thread when the port's sink hears a stage of a
//! request an agent asked for here after the drain that dispatched it,
//! so a stage that ends outside a drain goes out too.
//!
//! what a client cannot do to the app: block it (the app never waits on a
//! socket: a client that does not read its replies is cut off, when its
//! queue is full or one write waits `WRITE_STALL`, and a flood waits in a
//! bounded queue and is served some messages per turn), crash it (every
//! line is decoded on the reader's thread and a bad one answers an error),
//! or grow it without bound (a line holds at most `MAX_LINE` bytes, a port
//! at most `MAX_CONNS` connections). a client that stops sending still
//! gets every reply queued for it before its connection ends.
//!
//! a watch of a window's scene (task-79) is served the same way: the app
//! offers each frame it draws (`offer_scene`), on its thread, and the
//! lines go to the writers' queues, never waited on. a watcher that does
//! not read its frames is cut off when the lines waiting for it pass
//! `SCENE_QUEUE` bytes, so a camera's frames cannot pile up in the app.

use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::{self, Permissions};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::Shutdown;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender, TrySendError};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::codec::{Refusal, decode_client, encode_server};
use crate::error::WireError;
use crate::place::{check_app, clear_stale, io, private_dir, socket_dir, socket_path};
use crate::proto::{Ask, ClientMsg, ProtoError, Reply};
use crate::ring::DEFAULT_RING;
use crate::scene::{MAX_LINE, Offer, SceneWatch};
use crate::server::{ConnId, Host, Outgoing, Server, Traffic};

/// the variable that opens the port in a debug build: `PLEV_AGENT=1`.
pub const ENV: &str = "PLEV_AGENT";

/// how many connections a port holds at once.
pub const MAX_CONNS: usize = 16;

/// messages waiting for the app's thread, all connections together.
const QUEUE: usize = 256;

/// lines waiting to be written to one client.
const OUTBOX: usize = 1024;

/// the bytes one connection may have waiting to be written when the app
/// queues a line of a scene: past it, the watcher is not reading and its
/// connection ends (16 MiB: some thirteen camera frames of 640x360).
pub const SCENE_QUEUE: usize = 16 << 20;

/// messages one `drain` serves before it lets the loop turn.
const PER_DRAIN: usize = 64;

/// how long one write to a client may wait before the client counts as
/// not reading and its connection ends, so a writer thread never outlives
/// a client that stopped reading.
const WRITE_STALL: Duration = Duration::from_secs(10);

/// what the shell passes to wake its loop.
pub type Wake = Box<dyn Fn() + Send>;

/// where and how a port opens.
#[derive(Clone, Debug)]
pub struct Config {
    /// the app's name, the socket's file name: `[A-Za-z0-9_-]`, 1 to 32.
    pub app: String,
    /// the socket's directory; `$TMPDIR/plev` by default.
    pub dir: PathBuf,
    /// how many served revisions the port keeps.
    pub ring: usize,
}

impl Config {
    pub fn new(app: impl Into<String>) -> Self {
        Config {
            app: app.into(),
            dir: socket_dir(),
            ring: DEFAULT_RING,
        }
    }
}

/// true when `PLEV_AGENT` is `1` and this is a debug build. a release
/// build opens a port only when the app opts in and calls `open`.
pub fn enabled() -> bool {
    cfg!(debug_assertions) && env_says(std::env::var_os(ENV).as_deref())
}

fn env_says(value: Option<&OsStr>) -> bool {
    value == Some(OsStr::new("1"))
}

/// the agent port of one app. the shell keeps it on its own thread and
/// calls `drain` after the wake; dropping it closes every connection and
/// removes the socket.
pub struct AgentPort {
    path: PathBuf,
    server: Server,
    events: Receiver<Event>,
    conns: HashMap<ConnId, Conn>,
    shared: Arc<Shared>,
}

/// a connection as the app's thread holds it.
struct Conn {
    outbox: SyncSender<String>,
    stream: UnixStream,
    /// bytes queued and not yet written, counted where a line is queued
    /// and where the writer wrote it.
    queued: Arc<AtomicUsize>,
}

/// what the socket threads tell the app's thread.
enum Event {
    Open {
        conn: ConnId,
        outbox: SyncSender<String>,
        stream: UnixStream,
        queued: Arc<AtomicUsize>,
    },
    Msg {
        conn: ConnId,
        msg: ClientMsg,
    },
    Closed {
        conn: ConnId,
    },
}

/// what the threads share with the port.
struct Shared {
    closing: AtomicBool,
    /// a wake was sent and no drain started since: wakes coalesce.
    pending: AtomicBool,
    live: AtomicUsize,
    wake: Mutex<Wake>,
    /// the transport's side of `Traffic`, counted on the socket threads.
    conns: AtomicU64,
    msgs_in: AtomicU64,
    msgs_out: AtomicU64,
    bytes_in: AtomicU64,
    bytes_out: AtomicU64,
    /// lines refused before the app (not utf-8, not JSON, another
    /// version, over the limit).
    refused: AtomicU64,
}

impl Shared {
    fn wake(&self) {
        if !self.pending.swap(true, Ordering::SeqCst) {
            if let Ok(wake) = self.wake.lock() {
                wake();
            }
        }
    }

    /// one line went out to a client, as written on the socket.
    fn wrote(&self, line: &str) {
        self.msgs_out.fetch_add(1, Ordering::Relaxed);
        self.bytes_out.fetch_add(
            u64::try_from(line.len()).unwrap_or(u64::MAX),
            Ordering::Relaxed,
        );
    }
}

impl AgentPort {
    /// opens the port of `app` in `$TMPDIR/plev` when `enabled()`;
    /// `Ok(None)` otherwise, and then nothing is created. its scenes are
    /// shared when `PLEV_MIRROR=1` too (`scene::mirror_enabled`).
    pub fn from_env(
        app: &str,
        wake: impl Fn() + Send + 'static,
    ) -> Result<Option<AgentPort>, WireError> {
        if !enabled() {
            return Ok(None);
        }
        let mut port = AgentPort::open(app, wake)?;
        port.share_scenes(crate::scene::mirror_enabled());
        Ok(Some(port))
    }

    /// opens the port of `app` in `$TMPDIR/plev`, whatever the build: the
    /// app's explicit opt-in.
    pub fn open(app: &str, wake: impl Fn() + Send + 'static) -> Result<AgentPort, WireError> {
        AgentPort::open_with(Config::new(app), wake)
    }

    /// opens a port as `config` says: the directory made private, a socket
    /// that ended with a process of our pid cleared, the socket bound 0600
    /// and the accepting thread started.
    pub fn open_with(
        config: Config,
        wake: impl Fn() + Send + 'static,
    ) -> Result<AgentPort, WireError> {
        check_app(&config.app)?;
        private_dir(&config.dir)?;
        let path = socket_path(&config.dir, &config.app, std::process::id());
        clear_stale(&path)?;
        let listener =
            UnixListener::bind(&path).map_err(|e| io(format!("bind {}", path.display()), e))?;
        let opened = fs::set_permissions(&path, Permissions::from_mode(0o600))
            .map_err(|e| io(format!("chmod {}", path.display()), e));
        let (tx, events) = mpsc::sync_channel(QUEUE);
        let shared = Arc::new(Shared {
            closing: AtomicBool::new(false),
            pending: AtomicBool::new(false),
            live: AtomicUsize::new(0),
            wake: Mutex::new(Box::new(wake)),
            conns: AtomicU64::new(0),
            msgs_in: AtomicU64::new(0),
            msgs_out: AtomicU64::new(0),
            bytes_in: AtomicU64::new(0),
            bytes_out: AtomicU64::new(0),
            refused: AtomicU64::new(0),
        });
        let accepting = Arc::clone(&shared);
        let started = opened.and_then(|()| {
            thread::Builder::new()
                .name("plev-agent-accept".into())
                .spawn(move || accept(&listener, &tx, &accepting))
                .map_err(|e| io("start the accepting thread", e))
        });
        if let Err(e) = started {
            let _ = fs::remove_file(&path);
            return Err(e);
        }
        let mut server = Server::new(config.app, config.ring);
        let waking = Arc::clone(&shared);
        server.wake_with(move || waking.wake());
        Ok(AgentPort {
            path,
            server,
            events,
            conns: HashMap::new(),
            shared,
        })
    }

    /// the socket's path.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// the protocol state (the served revisions among it).
    pub fn server(&self) -> &Server {
        &self.server
    }

    /// what went through this port since it opened: connections,
    /// messages and bytes counted on the socket threads as the lines come
    /// and go (a line with its `\n`), and what the server answered.
    /// `errors` is the server's errors plus the lines refused before it.
    pub fn traffic(&self) -> Traffic {
        let s = &self.shared;
        let mut t = self.server.traffic();
        t.conns = s.conns.load(Ordering::Relaxed);
        t.msgs_in = s.msgs_in.load(Ordering::Relaxed);
        t.msgs_out = s.msgs_out.load(Ordering::Relaxed);
        t.bytes_in = s.bytes_in.load(Ordering::Relaxed);
        t.bytes_out = s.bytes_out.load(Ordering::Relaxed);
        t.errors += s.refused.load(Ordering::Relaxed);
        t
    }

    /// serves what the connections asked since the last drain, on the
    /// calling thread (the app's), through `host`, and sends every stage
    /// that ended since. call it after the port's wake. a stage of a
    /// request an agent asked for here that ends later, outside a drain (a
    /// person's answer to a confirmation, a worker's answer), wakes the
    /// port itself when the sink hears it (task-92), so the next drain
    /// sends it; a shell that also drains right after a worker's answer
    /// sends it sooner. true when an act was dispatched, so the app may
    /// have changed and the shell draws again.
    pub fn drain(&mut self, host: &mut dyn Host) -> bool {
        self.shared.pending.store(false, Ordering::SeqCst);
        let mut acted = false;
        let mut served = 0;
        while served < PER_DRAIN {
            let Ok(event) = self.events.try_recv() else {
                break;
            };
            match event {
                Event::Open {
                    conn,
                    outbox,
                    stream,
                    queued,
                } => {
                    self.conns.insert(
                        conn,
                        Conn {
                            outbox,
                            stream,
                            queued,
                        },
                    );
                }
                Event::Msg { conn, msg } => {
                    served += 1;
                    acted |= matches!(msg.ask, Ask::Act(_));
                    let out = self.server.handle(conn, msg, host);
                    self.send(out);
                }
                Event::Closed { conn } => self.forget(conn),
            }
        }
        if served == PER_DRAIN {
            // more may wait: let the loop draw, then come back.
            self.shared.wake();
        }
        let out = self.server.forward(host);
        self.send(out);
        acted
    }

    /// whether a connection watches the scene of `window` (task-79): the
    /// app writes that window's frames only then.
    pub fn watching(&self, window: &str) -> bool {
        self.server.watching(window)
    }

    /// whether the scenes are shared (`Server::share_scenes`): `from_env`
    /// sets it from `PLEV_MIRROR`; an app that opens the port itself says.
    pub fn share_scenes(&mut self, on: bool) {
        self.server.share_scenes(on);
    }

    /// the watches that began or ended since the last call
    /// (`Server::take_scene_watches`), a cut one among them.
    pub fn take_scene_watches(&mut self) -> Vec<SceneWatch> {
        self.server.take_scene_watches()
    }

    /// `offer`, the frame the app just drew in `window` (or the one it
    /// holds, for a watch that just came), to every watch of that window:
    /// to each, only what changed since what it was sent. on the calling
    /// thread (the app's); never waits on a socket.
    pub fn offer_scene(&mut self, window: &str, offer: &Offer) {
        let out = self.server.offer_scene(window, offer);
        self.send(out);
    }

    fn send(&mut self, out: Vec<Outgoing>) {
        for Outgoing { conn, msg } in out {
            let Some(c) = self.conns.get(&conn) else {
                continue;
            };
            let scene = matches!(msg.reply, Reply::Scene(_) | Reply::SceneBlob(_));
            let mut line = encode_server(&msg);
            line.push('\n');
            let len = line.len();
            if scene && c.queued.load(Ordering::SeqCst).saturating_add(len) > SCENE_QUEUE {
                // a watcher that does not read its frames is cut off
                self.cut(conn);
                continue;
            }
            c.queued.fetch_add(len, Ordering::SeqCst);
            match c.outbox.try_send(line) {
                Ok(()) => {}
                // a client that does not read its replies is cut off;
                // the app never waits on it.
                Err(TrySendError::Full(_) | TrySendError::Disconnected(_)) => self.cut(conn),
            }
        }
    }

    /// ends a connection now, whatever is still queued for it.
    fn cut(&mut self, conn: ConnId) {
        if let Some(c) = self.conns.remove(&conn) {
            let _ = c.stream.shutdown(Shutdown::Both);
        }
        self.server.close(conn);
    }

    /// a connection whose reader ended (the client stopped sending, or
    /// broke the line limit): the replies already queued for it, its
    /// refusal among them, still go out, and its writer ends it after the
    /// last one. shutting it down here would race the writer and drop
    /// them.
    fn forget(&mut self, conn: ConnId) {
        self.conns.remove(&conn);
        self.server.close(conn);
    }
}

impl Drop for AgentPort {
    fn drop(&mut self) {
        self.shared.closing.store(true, Ordering::SeqCst);
        for (_, c) in self.conns.drain() {
            let _ = c.stream.shutdown(Shutdown::Both);
        }
        // the accepting thread wakes on this connection, sees the port
        // closing and ends.
        let _ = UnixStream::connect(&self.path);
        while let Ok(event) = self.events.try_recv() {
            if let Event::Open { stream, .. } = event {
                let _ = stream.shutdown(Shutdown::Both);
            }
        }
        let _ = fs::remove_file(&self.path);
    }
}

fn accept(listener: &UnixListener, events: &SyncSender<Event>, shared: &Arc<Shared>) {
    let mut next: ConnId = 0;
    for stream in listener.incoming() {
        if shared.closing.load(Ordering::SeqCst) {
            return;
        }
        let Ok(stream) = stream else {
            // out of descriptors or alike: do not spin.
            thread::sleep(Duration::from_millis(20));
            continue;
        };
        if shared.live.load(Ordering::SeqCst) >= MAX_CONNS {
            refuse(
                stream,
                "this app already holds as many agent connections as it takes",
                shared,
            );
            continue;
        }
        next += 1;
        shared.conns.fetch_add(1, Ordering::Relaxed);
        if start(next, stream, events, shared).is_err() {
            return;
        }
    }
}

/// answers a connection that is not taken and closes it.
fn refuse(mut stream: UnixStream, reason: &str, shared: &Shared) {
    let refusal = Refusal {
        id: None,
        error: ProtoError::NotEnabled {
            reason: reason.to_owned(),
        },
    };
    let mut line = encode_server(&refusal.reply());
    line.push('\n');
    let _ = stream.set_write_timeout(Some(Duration::from_millis(200)));
    if stream.write_all(line.as_bytes()).is_ok() {
        shared.wrote(&line);
        shared.refused.fetch_add(1, Ordering::Relaxed);
    }
    let _ = stream.shutdown(Shutdown::Both);
}

/// starts the two threads of a connection. `Err` when the port is gone.
fn start(
    conn: ConnId,
    stream: UnixStream,
    events: &SyncSender<Event>,
    shared: &Arc<Shared>,
) -> Result<(), ()> {
    let (Ok(reading), Ok(writing), Ok(held)) =
        (stream.try_clone(), stream.try_clone(), stream.try_clone())
    else {
        return Ok(());
    };
    let _ = writing.set_write_timeout(Some(WRITE_STALL));
    let (outbox, lines) = mpsc::sync_channel::<String>(OUTBOX);
    let queued = Arc::new(AtomicUsize::new(0));
    events
        .send(Event::Open {
            conn,
            outbox: outbox.clone(),
            stream: held,
            queued: Arc::clone(&queued),
        })
        .map_err(|_| ())?;
    shared.live.fetch_add(1, Ordering::SeqCst);
    let shared_w = Arc::clone(shared);
    let written = Arc::clone(&queued);
    let writer = thread::Builder::new()
        .name(format!("plev-agent-write-{conn}"))
        .spawn(move || write(writing, &lines, &shared_w, &written));
    let events = events.clone();
    let shared_r = Arc::clone(shared);
    let out = Out { outbox, queued };
    let reader = thread::Builder::new()
        .name(format!("plev-agent-read-{conn}"))
        .spawn(move || read(conn, reading, &events, &out, &shared_r));
    if writer.is_err() || reader.is_err() {
        shared.live.fetch_sub(1, Ordering::SeqCst);
        let _ = stream.shutdown(Shutdown::Both);
    }
    Ok(())
}

fn write(mut stream: UnixStream, lines: &Receiver<String>, shared: &Shared, queued: &AtomicUsize) {
    while let Ok(line) = lines.recv() {
        if stream.write_all(line.as_bytes()).is_err() {
            break;
        }
        queued.fetch_sub(line.len(), Ordering::SeqCst);
        shared.wrote(&line);
    }
    let _ = stream.shutdown(Shutdown::Both);
}

/// a connection's queue as its reader thread writes refusals into it,
/// counted like the app's lines.
struct Out {
    outbox: SyncSender<String>,
    queued: Arc<AtomicUsize>,
}

impl Out {
    fn send(&self, line: String) -> Result<(), ()> {
        self.queued.fetch_add(line.len(), Ordering::SeqCst);
        self.outbox.send(line).map_err(|_| ())
    }
}

/// reads lines until the client leaves or breaks the line limit. a line
/// that does not decode answers its error from here; a message goes to
/// the app's thread, which the wake calls.
fn read(
    conn: ConnId,
    stream: UnixStream,
    events: &SyncSender<Event>,
    outbox: &Out,
    shared: &Shared,
) {
    let mut reader = BufReader::new(stream);
    let mut buf = Vec::new();
    loop {
        buf.clear();
        let limit = u64::try_from(MAX_LINE)
            .unwrap_or(u64::MAX)
            .saturating_add(1);
        match (&mut reader).take(limit).read_until(b'\n', &mut buf) {
            Ok(0) | Err(_) => break,
            Ok(_) => {}
        }
        shared.bytes_in.fetch_add(
            u64::try_from(buf.len()).unwrap_or(u64::MAX),
            Ordering::Relaxed,
        );
        if buf.last() != Some(&b'\n') && buf.len() > MAX_LINE {
            shared.msgs_in.fetch_add(1, Ordering::Relaxed);
            shared.refused.fetch_add(1, Ordering::Relaxed);
            let reason = format!("a line holds at most {MAX_LINE} bytes; the connection ends");
            let _ = outbox.send(refusal_line(None, reason));
            break;
        }
        let line = trim_end(&buf);
        let text = match std::str::from_utf8(line) {
            Ok(text) => text,
            Err(e) => {
                shared.msgs_in.fetch_add(1, Ordering::Relaxed);
                shared.refused.fetch_add(1, Ordering::Relaxed);
                if outbox
                    .send(refusal_line(None, format!("not utf-8: {e}")))
                    .is_err()
                {
                    break;
                }
                continue;
            }
        };
        if text.trim().is_empty() {
            continue;
        }
        shared.msgs_in.fetch_add(1, Ordering::Relaxed);
        match decode_client(text) {
            Ok(msg) => {
                if events.send(Event::Msg { conn, msg }).is_err() {
                    break;
                }
                shared.wake();
            }
            Err(refusal) => {
                shared.refused.fetch_add(1, Ordering::Relaxed);
                let mut line = encode_server(&refusal.reply());
                line.push('\n');
                if outbox.send(line).is_err() {
                    break;
                }
            }
        }
    }
    shared.live.fetch_sub(1, Ordering::SeqCst);
    if events.send(Event::Closed { conn }).is_ok() {
        shared.wake();
    }
}

fn refusal_line(id: Option<String>, reason: String) -> String {
    let refusal = Refusal {
        id,
        error: ProtoError::BadMessage { reason },
    };
    let mut line = encode_server(&refusal.reply());
    line.push('\n');
    line
}

/// the line without its `\n` or `\r\n`.
fn trim_end(buf: &[u8]) -> &[u8] {
    let buf = buf.strip_suffix(b"\n").unwrap_or(buf);
    buf.strip_suffix(b"\r").unwrap_or(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_one_opens_the_port() {
        assert!(env_says(Some(OsStr::new("1"))));
        for no in [
            None,
            Some("0"),
            Some(""),
            Some("true"),
            Some("yes"),
            Some(" 1"),
        ] {
            assert!(!env_says(no.map(OsStr::new)), "{no:?}");
        }
    }

    #[test]
    fn a_line_ends_without_its_terminator() {
        assert_eq!(trim_end(b"abc\r\n"), b"abc");
        assert_eq!(trim_end(b"abc\n"), b"abc");
        assert_eq!(trim_end(b"abc"), b"abc");
        assert_eq!(trim_end(b"\n"), b"");
    }
}
