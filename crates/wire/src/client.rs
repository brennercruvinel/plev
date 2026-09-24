//! `Client`, one connection to an app's agent port: what the `plev`
//! binary and the tests speak through.

use std::io::{BufRead, BufReader, ErrorKind, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::Duration;

use sem::Actor;

use crate::codec::{decode_server, encode_client};
use crate::error::WireError;
use crate::place::io;
use crate::proto::{Act, Ask, ClientMsg, Observe, Reply, ServerMsg, StageReply};

/// a connection to an agent port. ids are numbered from 1.
pub struct Client {
    writer: UnixStream,
    reader: BufReader<UnixStream>,
    next: u64,
    timeout: Option<u64>,
}

impl Client {
    pub fn connect(path: &Path) -> Result<Client, WireError> {
        let writer =
            UnixStream::connect(path).map_err(|e| io(format!("connect {}", path.display()), e))?;
        let reader = writer
            .try_clone()
            .map_err(|e| io("clone the connection", e))?;
        Ok(Client {
            writer,
            reader: BufReader::new(reader),
            next: 0,
            timeout: None,
        })
    }

    /// how long a read waits for a line, in milliseconds; `None` waits
    /// for as long as it takes.
    pub fn set_timeout(&mut self, ms: Option<u64>) -> Result<(), WireError> {
        self.timeout = ms;
        self.reader
            .get_ref()
            .set_read_timeout(ms.map(|ms| Duration::from_millis(ms.max(1))))
            .map_err(|e| io("set the read timeout", e))
    }

    /// sends `ask` under a new id and returns the id.
    pub fn send(&mut self, ask: Ask) -> Result<String, WireError> {
        self.next += 1;
        let id = self.next.to_string();
        let mut line = encode_client(&ClientMsg {
            id: id.clone(),
            ask,
        });
        line.push('\n');
        self.send_raw(&line)?;
        Ok(id)
    }

    /// writes `text` as it is (a test's broken line).
    pub fn send_raw(&mut self, text: &str) -> Result<(), WireError> {
        self.writer
            .write_all(text.as_bytes())
            .map_err(|e| io("write to the app", e))
    }

    /// the next message from the app.
    pub fn recv(&mut self) -> Result<ServerMsg, WireError> {
        let mut line = String::new();
        match self.reader.read_line(&mut line) {
            Ok(0) => Err(WireError::Closed),
            Ok(_) => {
                decode_server(line.trim_end()).map_err(|r| WireError::BadReply(r.error.to_string()))
            }
            Err(e) if matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
                Err(WireError::Timeout(self.timeout.unwrap_or(0)))
            }
            Err(e) => Err(io("read from the app", e)),
        }
    }

    /// the next message answering `id`; messages for other ids are skipped.
    pub fn recv_for(&mut self, id: &str) -> Result<Reply, WireError> {
        loop {
            let msg = self.recv()?;
            if msg.id.as_deref() == Some(id) {
                return Ok(msg.reply);
            }
            if msg.id.is_none() {
                if let Reply::Error(e) = msg.reply {
                    return Err(WireError::Refused(e));
                }
            }
        }
    }

    /// sends `ask` and waits for its one reply; an error reply is `Err`.
    pub fn call(&mut self, ask: Ask) -> Result<Reply, WireError> {
        let id = self.send(ask)?;
        match self.recv_for(&id)? {
            Reply::Error(e) => Err(WireError::Refused(e)),
            reply => Ok(reply),
        }
    }

    /// watches the scene of `window` (task-79): its frames come as
    /// replies to the returned id, the first whole, then what changed,
    /// for as long as the connection lasts; read them with `recv` and put
    /// them back together with a `SceneReader`.
    pub fn watch_scene(&mut self, window: &str) -> Result<String, WireError> {
        self.send(Ask::Scene(crate::scene::SceneAsk {
            window: window.to_owned(),
        }))
    }

    /// names this connection's actor.
    pub fn hello(&mut self, name: &str) -> Result<Actor, WireError> {
        match self.call(Ask::Hello {
            name: name.to_owned(),
        })? {
            Reply::Welcome { actor, .. } => Ok(actor),
            other => Err(unexpected(&other)),
        }
    }

    /// one observation: a `snapshot` or a `delta`.
    pub fn observe(&mut self, observe: Observe) -> Result<Reply, WireError> {
        match self.call(Ask::Observe(observe))? {
            reply @ (Reply::Snapshot(_) | Reply::Delta(_)) => Ok(reply),
            other => Err(unexpected(&other)),
        }
    }

    /// sends an act and reads its stages until a terminal one, handing
    /// each to `each` as it arrives; returns every stage.
    pub fn act(
        &mut self,
        act: Act,
        mut each: impl FnMut(&StageReply),
    ) -> Result<Vec<StageReply>, WireError> {
        let id = self.send(Ask::Act(act))?;
        let mut stages = Vec::new();
        loop {
            match self.recv_for(&id)? {
                Reply::Stage(stage) => {
                    each(&stage);
                    let end = stage.is_terminal();
                    stages.push(stage);
                    if end {
                        return Ok(stages);
                    }
                }
                Reply::Error(e) => return Err(WireError::Refused(e)),
                other => return Err(unexpected(&other)),
            }
        }
    }
}

fn unexpected(reply: &Reply) -> WireError {
    WireError::BadReply(format!("an unexpected reply: {reply:?}"))
}
