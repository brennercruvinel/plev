//! `SemId`, the persistent logical identity of a node. the app or the IR
//! picks it; a counter or a position never does.

use std::borrow::Borrow;
use std::fmt;
use std::iter::Peekable;
use std::str::{CharIndices, FromStr};
use std::sync::Arc;

use crate::failures::SemError;
use crate::notation::{is_name_char, name_fault};
use crate::encoding::text_serde;

/// the logical identity of a node.
///
/// canonical text form: a sequence of segments. a name segment is
/// `[a-z0-9_]+`, and name segments are joined by `.`. a key segment is
/// bracketed text that follows a segment directly, `[` then any text then
/// `]`, where `]` and `\` inside the key are escaped with a `\` and nothing
/// else is escaped. the first segment is always a name.
///
/// `commit.submit`, `changes.file[src/lib.rs]`, `silence.segment[3].preview`
///
/// the value is the canonical text behind an `Arc<str>`, so a clone is a
/// refcount bump. `Eq`, `Ord` and `Hash` follow the text: the order is total
/// and the same on every target. `parse(&id.to_string()) == Ok(id)` for
/// every id, and a text that parses prints back unchanged.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SemId(Arc<str>);

impl SemId {
    /// parses the canonical text form.
    pub fn parse(text: &str) -> Result<SemId, SemError> {
        match scan(text) {
            Ok(()) => Ok(SemId(Arc::from(text))),
            Err(reason) => Err(SemError::InvalidId {
                text: text.to_owned(),
                reason,
            }),
        }
    }

    /// a root id made of one name segment.
    pub fn name(name: &str) -> Result<SemId, SemError> {
        check_name(name)?;
        Ok(SemId(Arc::from(name)))
    }

    /// this id followed by `.name`.
    pub fn child(&self, name: &str) -> Result<SemId, SemError> {
        check_name(name)?;
        let mut text = String::with_capacity(self.0.len() + 1 + name.len());
        text.push_str(&self.0);
        text.push('.');
        text.push_str(name);
        Ok(SemId(Arc::from(text)))
    }

    /// this id followed by the key segment `[key]`. any text is a valid
    /// key, so this cannot fail; `]` and `\` are escaped in the text form.
    pub fn key(&self, key: impl Into<String>) -> SemId {
        let key = key.into();
        let mut text = String::with_capacity(self.0.len() + key.len() + 2);
        text.push_str(&self.0);
        text.push('[');
        for c in key.chars() {
            if c == ']' || c == '\\' {
                text.push('\\');
            }
            text.push(c);
        }
        text.push(']');
        SemId(Arc::from(text))
    }

    /// the id without its last segment, or `None` for a single name. this
    /// is text only: it does not say that a node with that id exists, nor
    /// that it is the parent in the tree (the parent is declared).
    pub fn parent_path(&self) -> Option<SemId> {
        let mut cut = None;
        let mut in_key = false;
        let mut escaped = false;
        for (at, c) in self.0.char_indices() {
            if in_key {
                if escaped {
                    escaped = false;
                } else if c == '\\' {
                    escaped = true;
                } else if c == ']' {
                    in_key = false;
                }
            } else if c == '.' || c == '[' {
                cut = Some(at);
                in_key = c == '[';
            }
        }
        cut.and_then(|at| self.0.get(..at))
            .map(|t| SemId(Arc::from(t)))
    }

    /// the canonical text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn check_name(name: &str) -> Result<(), SemError> {
    match name_fault(name) {
        None => Ok(()),
        Some(reason) => Err(SemError::InvalidId {
            text: name.to_owned(),
            reason,
        }),
    }
}

/// checks the canonical grammar; the error is a short reason.
fn scan(text: &str) -> Result<(), String> {
    if text.is_empty() {
        return Err("empty id".to_owned());
    }
    let mut chars = text.char_indices().peekable();
    loop {
        let mut len = 0usize;
        while chars.next_if(|&(_, c)| is_name_char(c)).is_some() {
            len += 1;
        }
        if len == 0 {
            return Err(match chars.peek() {
                None => "empty segment at the end".to_owned(),
                Some(&(_, '.')) => "empty segment".to_owned(),
                Some(&(_, '[')) => "a key needs a name segment before it".to_owned(),
                Some(&(at, c)) => format!("invalid character {c:?} at byte {at}"),
            });
        }
        while chars.next_if(|&(_, c)| c == '[').is_some() {
            scan_key(&mut chars)?;
        }
        match chars.next() {
            None => return Ok(()),
            Some((_, '.')) => {}
            Some((at, c)) => return Err(format!("invalid character {c:?} at byte {at}")),
        }
    }
}

/// consumes a key body up to and including its closing `]`.
fn scan_key(chars: &mut Peekable<CharIndices<'_>>) -> Result<(), String> {
    while let Some((at, c)) = chars.next() {
        match c {
            ']' => return Ok(()),
            '\\' => match chars.next() {
                Some((_, ']' | '\\')) => {}
                Some((_, e)) => return Err(format!("invalid escape \\{e} at byte {at}")),
                None => break,
            },
            _ => {}
        }
    }
    Err("key without a closing ]".to_owned())
}

impl fmt::Display for SemId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for SemId {
    type Err = SemError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        SemId::parse(text)
    }
}

impl TryFrom<&str> for SemId {
    type Error = SemError;

    fn try_from(text: &str) -> Result<Self, Self::Error> {
        SemId::parse(text)
    }
}

impl TryFrom<String> for SemId {
    type Error = SemError;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        SemId::parse(&text)
    }
}

impl From<&SemId> for SemId {
    fn from(id: &SemId) -> Self {
        id.clone()
    }
}

impl AsRef<str> for SemId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Borrow<str> for SemId {
    fn borrow(&self) -> &str {
        &self.0
    }
}

text_serde!(SemId);
