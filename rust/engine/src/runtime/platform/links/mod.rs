//! Deep links (task-76 item 6): the app's URL scheme as a way in from
//! outside the window. A link names one capability of the app's
//! projection, `<scheme>://<sem id>/<action>[?value=<text>]`, and becomes
//! a request through the app's one dispatcher ([`Link::request`]).
//!
//! A link is not a person. Any web page, mail or other app can open one,
//! so its request is never `Actor::Human`: it is an agent's,
//! `Actor::Agent` named [`ACTOR`] (`link`), the least the contract lets
//! anyone outside ask, and an app answers it only for the capabilities it
//! declares open to links, a list of its own, explicit and small
//! ([`Link::is_in`]). Whatever the list holds, the policy still applies:
//! a capability for a person alone is refused as `NotAuthorized`, and one
//! that asks a person's confirmation of an agent waits for it.
//!
//! The grammar is strict ([`Link::parse`]): at most [`MAX_LEN`] bytes of
//! printable ASCII; the scheme the app's, ASCII case aside; the id
//! percent-decoded and then [`SemId::parse`]d; one `/`; the action a name
//! of `[a-z_]`; at most one query key, `value`, its text percent-decoded
//! as UTF-8, with no control character and no direction override (the
//! app shows a refused link to the person in one line). No fragment, no
//! user, no password, no port, no slash more. [`LinkError`] says why a
//! text is not a link.
//!
//! Who delivers one is a [`LinkSource`] (`source.rs`): the launch
//! argument on every desktop ([`LaunchLinks`]), a fake for the tests
//! ([`FakeLinks`]), none on wasm and mobile ([`NoLinks`]),
//! [`default_links`] choosing. Windows and linux start a new process
//! with the link as its argument; macOS delivers a registered scheme to
//! a bundled app as an Apple event, which is the platform bring-up's
//! (docs/adr/interface/0063-a-link-is-not-a-person.md says how, and why not by an
//! application delegate).

use std::fmt;

use accord::{Actor, Request, SemError, SemId, Snapshot, Value};

mod source;

pub use source::{
    Delivered, FakeLinks, LaunchLinks, LinkSource, NoLinks, Refused, default_links, read,
};

/// The most bytes a link may have. A link that navigates needs well
/// under a hundred; 2048 keeps the parse and the line a refusal shows
/// bounded, below the 2083 characters the Windows shell passes on.
pub const MAX_LEN: usize = 2048;

/// The name of the agent every link's request is made by.
pub const ACTOR: &str = "link";

/// The actor of a link's request: an agent named [`ACTOR`], never a
/// person.
pub fn actor() -> Actor {
    Actor::Agent {
        name: ACTOR.to_owned(),
    }
}

/// A link read: the capability it names, the action it asks, and the
/// text it passes, if any.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Link {
    pub target: SemId,
    /// A name of `[a-z_]`.
    pub action: String,
    pub value: Option<String>,
}

/// Why a text is not a link ([`Link::parse`] says in which order it
/// looks).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LinkError {
    /// Longer than [`MAX_LEN`] bytes.
    TooLong { len: usize },
    /// A character that is not printable ASCII (a space, a control, a
    /// letter beyond ASCII): text travels percent-encoded.
    BadChar { at: usize, found: char },
    /// No `<scheme>://`.
    NotALink,
    /// Another app's scheme.
    OtherScheme { found: String },
    /// A `#`.
    Fragment,
    /// A `@` before the first `/`: a user or a password.
    UserInfo,
    /// A `:` before the first `/`: a port.
    Port,
    /// A `/` after the action.
    ExtraPath,
    /// Nothing between `://` and `/`.
    NoTarget,
    /// The id, decoded, is not a `SemId`.
    BadTarget { reason: String },
    /// Nothing after the `/`.
    NoAction,
    /// An action that is not a name of `[a-z_]`.
    BadAction { found: String },
    /// A query without `value=`.
    NoValue,
    /// A query key other than `value`.
    UnknownKey { key: String },
    /// More than one query key (`&`).
    ExtraKey,
    /// A `%` not followed by two hex digits.
    Percent { escape: String },
    /// Decoded bytes that are not UTF-8.
    Utf8,
    /// Decoded text with a control character or a direction override.
    Control,
}

impl fmt::Display for LinkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LinkError::TooLong { len } => {
                write!(
                    f,
                    "the link has {len} bytes, above the {MAX_LEN} a link may have"
                )
            }
            LinkError::BadChar { at, found } => write!(
                f,
                "'{}' at byte {at} is not printable ASCII (a link carries its text \
                 percent-encoded)",
                found.escape_default()
            ),
            LinkError::NotALink => f.write_str("not a link: no scheme:// before the capability"),
            LinkError::OtherScheme { found } => write!(f, "the link is for {found}://"),
            LinkError::Fragment => f.write_str("a link has no fragment (#)"),
            LinkError::UserInfo => f.write_str("a link has no user or password (@)"),
            LinkError::Port => f.write_str("a link has no port (:)"),
            LinkError::ExtraPath => f.write_str("a link is <id>/<action>, with one slash"),
            LinkError::NoTarget => f.write_str("the link names no capability"),
            LinkError::BadTarget { reason } => write!(f, "the link's id is not an id: {reason}"),
            LinkError::NoAction => f.write_str("the link names no action"),
            LinkError::BadAction { found } => {
                write!(f, "the link's action {found:?} is not a name of [a-z_]")
            }
            LinkError::NoValue => f.write_str("the link's query has no value="),
            LinkError::UnknownKey { key } => {
                write!(f, "a link takes value= alone, not {key:?}")
            }
            LinkError::ExtraKey => f.write_str("a link takes one value= and nothing more (&)"),
            LinkError::Percent { escape } => {
                write!(f, "{escape:?} is not a percent escape of two hex digits")
            }
            LinkError::Utf8 => f.write_str("the link's text is not UTF-8 once decoded"),
            LinkError::Control => {
                f.write_str("the link's text holds a control or a direction character")
            }
        }
    }
}

impl std::error::Error for LinkError {}

impl Link {
    /// Reads `url` as a link of `scheme` (`[a-z][a-z0-9+.-]*`, the app's).
    /// See the module doc for the grammar. The error is the first that
    /// holds, looking in this order: the length, the characters, the
    /// scheme, a fragment, a user or a port before the first `/`, a slash
    /// more, the id (empty, then its decoding, then `SemId::parse`), the
    /// action, the query (its key, then a second key, then the value's
    /// decoding). A decoding refuses a bad `%`, then bytes that are not
    /// UTF-8, then a control or a direction character.
    pub fn parse(url: &str, scheme: &str) -> Result<Link, LinkError> {
        if url.len() > MAX_LEN {
            return Err(LinkError::TooLong { len: url.len() });
        }
        if let Some((at, found)) = url.char_indices().find(|&(_, c)| !c.is_ascii_graphic()) {
            return Err(LinkError::BadChar { at, found });
        }
        let Some((given, rest)) = url.split_once("://") else {
            return Err(LinkError::NotALink);
        };
        if !given.eq_ignore_ascii_case(scheme) {
            return Err(LinkError::OtherScheme {
                found: excerpt(given),
            });
        }
        if rest.contains('#') {
            return Err(LinkError::Fragment);
        }
        let (path, query) = match rest.split_once('?') {
            Some((path, query)) => (path, Some(query)),
            None => (rest, None),
        };
        let mut parts = path.split('/');
        let id = parts.next().unwrap_or_default();
        let action = parts.next();
        if id.contains('@') {
            return Err(LinkError::UserInfo);
        }
        if id.contains(':') {
            return Err(LinkError::Port);
        }
        if parts.next().is_some() {
            return Err(LinkError::ExtraPath);
        }
        if id.is_empty() {
            return Err(LinkError::NoTarget);
        }
        let target = SemId::parse(&decode(id)?).map_err(|e| LinkError::BadTarget {
            reason: match e {
                SemError::InvalidId { reason, .. } => reason,
                other => other.to_string(),
            },
        })?;
        let action = match action {
            None | Some("") => return Err(LinkError::NoAction),
            Some(name) if name.bytes().all(|b| b.is_ascii_lowercase() || b == b'_') => {
                name.to_owned()
            }
            Some(other) => {
                return Err(LinkError::BadAction {
                    found: excerpt(other),
                });
            }
        };
        let value = query.map(value).transpose()?;
        Ok(Link {
            target,
            action,
            value,
        })
    }

    /// The parameter the request carries: the value as text.
    pub fn param(&self) -> Option<Value> {
        self.value.clone().map(Value::Text)
    }

    /// Whether the app opened this capability to links: `open` lists the
    /// `(id, action)` pairs a link may ask, the app's own list.
    pub fn is_in(&self, open: &[(&str, &str)]) -> bool {
        open.iter()
            .any(|(id, action)| self.target.as_str() == *id && self.action == *action)
    }

    /// The request this link makes, having seen `seen`:
    /// `Request::observe` by [`actor`], an agent named `link`, never a
    /// person, so the dispatcher checks it as it checks an agent's (the
    /// policy, stale, inert, disabled, the parameter, a confirmation).
    pub fn request(&self, seen: &Snapshot) -> Result<Request, SemError> {
        Request::observe(
            seen,
            actor(),
            &self.target,
            self.action.as_str(),
            self.param(),
        )
    }

    /// The link as a URL of `scheme`, in the form [`Link::parse`] reads
    /// back: the id's brackets kept, anything else beyond `[A-Za-z0-9._~-]`
    /// percent-encoded, so the text is one line of printable ASCII.
    pub fn url(&self, scheme: &str) -> String {
        let mut out = format!("{scheme}://");
        encode(self.target.as_str(), b"[]", &mut out);
        out.push('/');
        out.push_str(&self.action);
        if let Some(value) = &self.value {
            out.push_str("?value=");
            encode(value, b"", &mut out);
        }
        out
    }
}

/// Whether `text` starts as a link of `scheme` does (`<scheme>://`, the
/// scheme in any ASCII case): what an app takes as a link rather than as
/// another argument. It says nothing of the rest; [`Link::parse`] does.
pub fn is_link(text: &str, scheme: &str) -> bool {
    let n = scheme.len();
    text.as_bytes()
        .get(..n)
        .is_some_and(|head| head.eq_ignore_ascii_case(scheme.as_bytes()))
        && text.get(n..).is_some_and(|rest| rest.starts_with("://"))
}

/// A short form of `text` for a line a person reads: printable ASCII as
/// it is, anything else escaped, cut after 96 characters.
pub fn excerpt(text: &str) -> String {
    const SHOWN: usize = 96;
    let mut out = String::new();
    for (n, c) in text.chars().enumerate() {
        if n == SHOWN {
            out.push_str("...");
            break;
        }
        if c == ' ' || c.is_ascii_graphic() {
            out.push(c);
        } else {
            out.extend(c.escape_default());
        }
    }
    out
}

/// The value of a query: exactly `value=<text>`, decoded.
fn value(query: &str) -> Result<String, LinkError> {
    let mut pairs = query.split('&');
    let pair = pairs.next().unwrap_or_default();
    let (key, text) = match pair.split_once('=') {
        Some(kv) => kv,
        None if pair.is_empty() || pair == "value" => return Err(LinkError::NoValue),
        None => {
            return Err(LinkError::UnknownKey { key: excerpt(pair) });
        }
    };
    if key != "value" {
        return Err(LinkError::UnknownKey { key: excerpt(key) });
    }
    if pairs.next().is_some() {
        return Err(LinkError::ExtraKey);
    }
    decode(text)
}

/// `raw` with every `%XX` decoded (a `+` stays a plus), as UTF-8 text
/// with no control character and no direction override.
fn decode(raw: &str) -> Result<String, LinkError> {
    let bytes = raw.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut at = 0;
    while let Some(&b) = bytes.get(at) {
        if b != b'%' {
            out.push(b);
            at += 1;
            continue;
        }
        let byte = match bytes.get(at + 1..at + 3) {
            Some(&[hi, lo]) => hex(hi).zip(hex(lo)).map(|(hi, lo)| (hi << 4) | lo),
            _ => None,
        };
        let Some(byte) = byte else {
            let end = (at + 3).min(raw.len());
            return Err(LinkError::Percent {
                escape: raw.get(at..end).unwrap_or("%").to_owned(),
            });
        };
        out.push(byte);
        at += 3;
    }
    let text = String::from_utf8(out).map_err(|_| LinkError::Utf8)?;
    if text.chars().any(unshown) {
        return Err(LinkError::Control);
    }
    Ok(text)
}

fn hex(digit: u8) -> Option<u8> {
    char::from(digit)
        .to_digit(16)
        .and_then(|d| u8::try_from(d).ok())
}

/// A character a one-line refusal must not carry: a control (a newline
/// would forge a second line) or a direction mark, override or isolate
/// (it would reorder the line a person reads).
fn unshown(c: char) -> bool {
    c.is_control()
        || matches!(
            c,
            '\u{200e}' | '\u{200f}' | '\u{061c}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}'
        )
}

/// Appends `text` percent-encoded: `[A-Za-z0-9._~-]` and the bytes in
/// `keep` as they are, every other byte of its UTF-8 as `%XX`.
fn encode(text: &str, keep: &[u8], out: &mut String) {
    for b in text.bytes() {
        if b.is_ascii_alphanumeric() || b"._~-".contains(&b) || keep.contains(&b) {
            out.push(char::from(b));
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
}

#[cfg(test)]
mod tests;
