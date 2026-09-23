//! the agent surface of plev: an agent outside the process observes and
//! operates an app through the semantic contract only. no screenshot, no
//! coordinate, no access to the app's backend: an observation is a
//! projection of the app (`sem::Snapshot`), scoped and incremental, and an
//! action is a `sem::Request` that goes through the same `Dispatcher` a
//! person's click goes through.
//!
//! protocol v1 is JSON lines. every message is one object with `"v": 1`,
//! an `"id"` the client picks and a `"type"`:
//!
//! ```text
//! -> {"v":1,"id":"1","type":"hello","name":"claude"}
//! <- {"v":1,"id":"1","type":"welcome","app":"showcase","actor":{"agent":{"name":"claude"}}}
//! -> {"v":1,"id":"2","type":"observe","scope":"todo"}
//! <- {"v":1,"id":"2","type":"snapshot","revision":"cfc1185078f7c825","nodes":[...],"focused":null}
//! -> {"v":1,"id":"3","type":"act","target":"todo.item[1]","action":"toggle","from_revision":"cfc1185078f7c825"}
//! <- {"v":1,"id":"3","type":"stage","request":"1","stage":"requested","at":745,"asked":{...}}
//! <- {"v":1,"id":"3","type":"stage","request":"1","stage":"accepted","at":745}
//! <- {"v":1,"id":"3","type":"stage","request":"1","stage":"executing","at":745}
//! <- {"v":1,"id":"3","type":"stage","request":"1","stage":{"completed":{"result":{}}},"at":745}
//! ```
//!
//! the messages (`proto`), their codec (`codec`), the ring of served
//! revisions (`Ring`) and the protocol against an app (`Server`, which
//! reaches the app only through `Host`) are pure and compile everywhere,
//! wasm included. the socket is unix only: `AgentPort` (the app's side,
//! off unless `PLEV_AGENT=1` in a debug build or the app opts in) and
//! `Client` (the agent's side, which the `plev` binary uses).

#[cfg(unix)]
mod client;
mod codec;
mod error;
#[cfg(unix)]
mod place;
#[cfg(unix)]
mod port;
mod proto;
mod ring;
mod server;

#[cfg(unix)]
pub use client::Client;
pub use codec::{Refusal, decode_client, decode_server, encode_client, encode_server};
pub use error::WireError;
#[cfg(unix)]
pub use place::{MAX_APP, check_app, find, socket_dir, socket_path};
#[cfg(unix)]
pub use port::{AgentPort, Config, ENV, MAX_CONNS, MAX_LINE, Wake, enabled};
pub use proto::{
    Act, Ask, Bound, ClientMsg, DeltaReply, Observe, ProtoError, Reply, ServerMsg, SnapshotReply,
    StageReply, VERSION,
};
pub use ring::{DEFAULT_RING, Ring};
pub use server::{ConnId, Host, MAX_NAME, Outgoing, Server, Traffic};
