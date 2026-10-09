//! JSON lines: one message per line, `"v"` and `"id"` first.
//!
//! decoding never panics on any input: a line that is not JSON, not an
//! object, of another version or of an unknown shape comes back as a
//! `Refusal`, which carries the error reply for that line. the version is
//! checked before the shape, so a message of a later version is refused
//! as `BadVersion`, never as `BadMessage`.

mod base64;

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Map, Value as Json};

use crate::protocol::{Ask, ClientMsg, ProtoError, Reply, ServerMsg, VERSION};

/// a line refused before it reached the app, and why. `id` is the line's
/// id when it could be read.
#[derive(Clone, Debug, PartialEq)]
pub struct Refusal {
    pub id: Option<String>,
    pub error: ProtoError,
}

impl Refusal {
    /// the error reply for the refused line.
    pub fn reply(self) -> ServerMsg {
        ServerMsg {
            id: self.id,
            reply: Reply::Error(self.error),
        }
    }
}

/// the envelope written first on every line.
#[derive(Serialize)]
struct Out<'a, T: Serialize> {
    v: u64,
    id: &'a Option<String>,
    #[serde(flatten)]
    body: &'a T,
}

/// one client message as a line, without the newline.
pub fn encode_client(msg: &ClientMsg) -> String {
    encode(&Some(msg.id.clone()), &msg.ask)
}

/// one app message as a line, without the newline.
pub fn encode_server(msg: &ServerMsg) -> String {
    encode(&msg.id, &msg.reply)
}

fn encode<T: Serialize>(id: &Option<String>, body: &T) -> String {
    let out = Out {
        v: VERSION,
        id,
        body,
    };
    match serde_json::to_string(&out) {
        Ok(line) => line,
        // the messages hold only string keys, so serializing them does not
        // fail (serde_json writes a non-finite `Num` as null, which a
        // reader then refuses); should it ever, the line stays well formed.
        Err(e) => {
            let error = Reply::Error(ProtoError::BadMessage {
                reason: format!("the reply does not fit in JSON: {e}"),
            });
            serde_json::to_string(&Out {
                v: VERSION,
                id,
                body: &error,
            })
            .unwrap_or_default()
        }
    }
}

/// reads one line from a client.
pub fn decode_client(line: &str) -> Result<ClientMsg, Refusal> {
    let (id, body) = open(line)?;
    let id = match id {
        Some(id) => id,
        None => return Err(bad(None, "a message needs an \"id\" that is a string")),
    };
    match decode_body::<Ask>(body) {
        Ok(ask) => Ok(ClientMsg { id, ask }),
        Err(reason) => Err(bad(Some(id), reason)),
    }
}

/// reads one line from the app.
pub fn decode_server(line: &str) -> Result<ServerMsg, Refusal> {
    let (id, body) = open(line)?;
    match decode_body::<Reply>(body) {
        Ok(reply) => Ok(ServerMsg { id, reply }),
        Err(reason) => Err(bad(id, reason)),
    }
}

/// parses the line as an object and checks its version; returns its id
/// when it is a string, and the object.
fn open(line: &str) -> Result<(Option<String>, Map<String, Json>), Refusal> {
    let object = match serde_json::from_str::<Json>(line) {
        Ok(Json::Object(object)) => object,
        Ok(_) => return Err(bad(None, "a message is a JSON object")),
        Err(e) => return Err(bad(None, format!("not JSON: {e}"))),
    };
    let id = match object.get("id") {
        Some(Json::String(id)) => Some(id.clone()),
        _ => None,
    };
    match object.get("v") {
        Some(v) if v.as_u64() == Some(VERSION) => Ok((id, object)),
        got => Err(Refusal {
            id,
            error: ProtoError::BadVersion {
                got: got.map_or_else(|| "missing".to_owned(), Json::to_string),
            },
        }),
    }
}

fn decode_body<T: DeserializeOwned>(object: Map<String, Json>) -> Result<T, String> {
    serde_json::from_value(Json::Object(object)).map_err(|e| e.to_string())
}

fn bad(id: Option<String>, reason: impl Into<String>) -> Refusal {
    Refusal {
        id,
        error: ProtoError::BadMessage {
            reason: reason.into(),
        },
    }
}
