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
//! <- {"v":1,"id":"2","type":"snapshot","revision":"…","nodes":[…],"focused":null}
//! -> {"v":1,"id":"3","type":"act","target":"todo.item[1]","action":"toggle","from_revision":"…"}
//! <- {"v":1,"id":"3","type":"stage","request":"4","stage":"requested","at":…,"asked":{…}}
//! <- {"v":1,"id":"3","type":"stage","request":"4","stage":"accepted","at":…}
//! <- {"v":1,"id":"3","type":"stage","request":"4","stage":"executing","at":…}
//! <- {"v":1,"id":"3","type":"stage","request":"4","stage":{"completed":{"result":{}}},"at":…}
//! ```
//!
//! the messages (`proto`) and their codec (`codec`) are pure and compile
//! everywhere, wasm included. so does the decoding of a snapshot or a
//! stage a client reads.

mod codec;
mod error;
mod proto;

pub use codec::{Refusal, decode_client, decode_server, encode_client, encode_server};
pub use error::WireError;
pub use proto::{
    Act, Ask, Bound, ClientMsg, DeltaReply, Observe, ProtoError, Reply, ServerMsg, SnapshotReply,
    StageReply, VERSION,
};
