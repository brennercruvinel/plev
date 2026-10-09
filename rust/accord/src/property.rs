//! the values a node's state holds, and the reference to one of them.

use std::collections::BTreeSet;
use std::fmt;
use std::str::FromStr;

use crate::encoding::text_serde;
use crate::failures::SemError;
use crate::identity::SemId;
use crate::notation::StateKey;

/// one piece of state.
///
/// equality is exact. `Num` compares by bits (`f64::to_bits`), so `NaN`
/// equals the same `NaN` and `0.0` differs from `-0.0`: the contract is
/// deterministic, never approximate. `List` keeps its order and may repeat;
/// `Set` has no order and no repeats.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(tag = "kind", content = "value", rename_all = "snake_case")
)]
pub enum Value {
    Bool(bool),
    Int(i64),
    Num(f64),
    Text(String),
    List(Vec<String>),
    Set(BTreeSet<String>),
}

impl Value {
    /// which kind of value this is.
    pub fn kind(&self) -> ValueKind {
        match self {
            Value::Bool(_) => ValueKind::Bool,
            Value::Int(_) => ValueKind::Int,
            Value::Num(_) => ValueKind::Num,
            Value::Text(_) => ValueKind::Text,
            Value::List(_) => ValueKind::List,
            Value::Set(_) => ValueKind::Set,
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Num(a), Value::Num(b)) => a.to_bits() == b.to_bits(),
            (Value::Text(a), Value::Text(b)) => a == b,
            (Value::List(a), Value::List(b)) => a == b,
            (Value::Set(a), Value::Set(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for Value {}

impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Value::Bool(v)
    }
}

impl From<i64> for Value {
    fn from(v: i64) -> Self {
        Value::Int(v)
    }
}

impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Value::Num(v)
    }
}

impl From<String> for Value {
    fn from(v: String) -> Self {
        Value::Text(v)
    }
}

impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Value::Text(v.to_owned())
    }
}

impl From<Vec<String>> for Value {
    fn from(v: Vec<String>) -> Self {
        Value::List(v)
    }
}

impl From<BTreeSet<String>> for Value {
    fn from(v: BTreeSet<String>) -> Self {
        Value::Set(v)
    }
}

/// the kind of a `Value`, without its content.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum ValueKind {
    Bool,
    Int,
    Num,
    Text,
    List,
    Set,
}

impl ValueKind {
    /// the stable name, the same one serde uses.
    pub fn as_str(self) -> &'static str {
        match self {
            ValueKind::Bool => "bool",
            ValueKind::Int => "int",
            ValueKind::Num => "num",
            ValueKind::Text => "text",
            ValueKind::List => "list",
            ValueKind::Set => "set",
        }
    }
}

/// one piece of state of one node, text form `<semid>:<key>`:
/// `changes:staged`, `commit.message:text`.
///
/// the last `:` separates the two parts. a key never holds `:`, and an id
/// key segment may, so `file[c:/x]:staged` reads as expected.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StateRef {
    pub node: SemId,
    pub key: StateKey,
}

impl StateRef {
    pub fn new(node: SemId, key: StateKey) -> Self {
        StateRef { node, key }
    }

    /// parses `<semid>:<key>`.
    pub fn parse(text: &str) -> Result<StateRef, SemError> {
        let Some((node, key)) = text.rsplit_once(':') else {
            return Err(SemError::InvalidId {
                text: text.to_owned(),
                reason: "a state ref is <id>:<key> and the ':' is missing".to_owned(),
            });
        };
        Ok(StateRef {
            node: SemId::parse(node)?,
            key: StateKey::parse(key)?,
        })
    }
}

impl fmt::Display for StateRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.node, self.key)
    }
}

impl FromStr for StateRef {
    type Err = SemError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        StateRef::parse(text)
    }
}

impl TryFrom<&str> for StateRef {
    type Error = SemError;

    fn try_from(text: &str) -> Result<Self, Self::Error> {
        StateRef::parse(text)
    }
}

impl TryFrom<String> for StateRef {
    type Error = SemError;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        StateRef::parse(&text)
    }
}

impl From<&StateRef> for StateRef {
    fn from(r: &StateRef) -> Self {
        r.clone()
    }
}

text_serde!(StateRef);
