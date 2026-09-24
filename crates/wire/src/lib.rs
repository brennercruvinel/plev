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
//! <- {"v":1,"id":"3","type":"stage","request":"1","stage":{"completed":{"result":{}}},"at":745,"revision":"7a51c0e9d2b8f364"}
//! ```
//!
//! the terminal stage carries the revision the app is at when it goes
//! out, which the next `act` can rest on (`from_revision`) and `observe`
//! can start from (`since`) with no observation in between. an act rests
//! on what its agent was shown, and on nothing else: a stage's revision
//! shows no state, so what the act reads comes from the agent's last
//! observations of it, or the act is refused as `not_observed`.
//!
//! an act may also ask a person first (`"ask"`, task-92): it waits
//! (`awaiting_confirmation`) until a person confirms or declines it at the
//! app, and the port sends that stage when it comes, with no message from
//! the agent.
//!
//! the messages (`proto`), their codec (`codec`), the ring of served
//! revisions (`Ring`) and the protocol against an app (`Server`, which
//! reaches the app only through `Host`) are pure and compile everywhere,
//! wasm included. the socket is unix only: `AgentPort` (the app's side,
//! off unless `PLEV_AGENT=1` in a debug build or the app opts in) and
//! `Client` (the agent's side, which the `plev` binary uses).
//!
//! the same connection also carries a window's scene (task-79,
//! `scene`): a `scene` request watches it, and its frames come back as
//! replies, only the layers that changed, binary parts as base64. the
//! app offers each frame it draws (`AgentPort::offer_scene`); a viewer
//! puts the lines back together (`SceneReader`).

mod b64;
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
mod scene;
mod scene_read;
mod server;
mod shown;
mod sink;

pub use b64::{decode as decode_base64, encode as encode_base64};
#[cfg(unix)]
pub use client::Client;
pub use codec::{Refusal, decode_client, decode_server, encode_client, encode_server};
pub use error::WireError;
#[cfg(unix)]
pub use place::{MAX_APP, check_app, find, socket_dir, socket_path};
#[cfg(unix)]
pub use port::{AgentPort, Config, ENV, MAX_CONNS, SCENE_QUEUE, Wake, enabled};
pub use proto::{
    Act, Ask, Bound, ClientMsg, DeltaReply, MAX_ASK, Observe, ProtoError, Reply, ServerMsg,
    SnapshotReply, StageReply, VERSION,
};
pub use ring::{DEFAULT_RING, Ring};
pub use scene::{
    BlobReply, MAIN_WINDOW, MAX_LINE, MAX_WINDOW, Offer, OfferLayer, SCENE_PART, SceneAsk,
    SceneLayer, SceneReply,
};
pub use scene_read::{LayerBody, MAX_BLOB, SceneFrame, SceneLayerIn, SceneReader};
pub use server::{ConnId, Host, MAX_NAME, Outgoing, Server, Traffic};
pub use shown::MAX_ACTORS;
