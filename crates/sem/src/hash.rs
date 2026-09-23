//! `Revision`, a stable hash of the semantic content of a snapshot.
//!
//! the revision crosses processes (agent, recorder), so it never goes
//! through a std hasher, never depends on pointer width or endianness, and
//! is computed here, by hand: FNV-1a 64 (offset `0xcbf29ce484222325`,
//! prime `0x100000001b3`) over the canonical encoding below. `source` is
//! not part of it. changing the encoding changes every revision: it needs
//! a new version tag and an ADR.
//!
//! ```text
//! u64      8 bytes, little endian
//! i64      8 bytes, little endian, two's complement
//! f64      u64 of f64::to_bits
//! bool     1 byte, 0 or 1
//! len      u64 of the count
//! str      len of the utf-8 bytes, then the bytes
//! opt(x)   1 byte 0, or 1 byte 1 then x
//!
//! snapshot     str("sem.revision.v1") len(nodes) node... opt(str focused)
//!              (nodes in tree order, see below)
//! node         str(id) str(role) str(label) opt(str description)
//!              opt(str parent) len(state) (str(key) value)... (by key)
//!              len(actions) action... constraints
//!              bool(enabled) bool(inert) bool(focusable)
//! value        str(kind) then: bool | i64 | f64 | str | len(items) str(item)...
//! action       str(name) opt(str label) bool(enabled) len(reads) str(read)...
//!              opt(str(param kind) bool(constrained)) policy
//! policy       bool(human) bool(agent) bool(assistive) bool(test)
//!              str(confirm) bool(destructive)
//! constraints  bool(editable) bool(required) opt(f64 low, f64 high)
//!              opt(f64 step) opt(len max_len) opt(len(options) str...)
//!              bool(secret)
//! ```
//!
//! nodes go in tree order: each root in projection order, then its
//! descendants depth first, children in projection order. sibling order is
//! meaning (the order of a list); how unrelated subtrees interleave in the
//! projection is not. state goes sorted by key, since the order keys were
//! declared in is not meaning either; a `Set` goes in its sorted order.
//! role, kind and confirm are their stable names (`Role::as_str`,
//! `ValueKind::as_str`, `Confirm::as_str`); ids and refs are their
//! canonical text.

use std::fmt;

use crate::decl::{ActionDecl, Constraints, Policy};
use crate::id::SemId;
use crate::node::Node;
use crate::value::Value;

const VERSION: &str = "sem.revision.v1";

/// the stable hash of a snapshot's semantic content. two snapshots with the
/// same content have the same revision on every run and every target. it
/// prints, and travels in serde, as 16 lowercase hex digits, because a
/// json number cannot hold every u64.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Revision(u64);

impl Revision {
    pub fn as_u64(self) -> u64 {
        self.0
    }
}

impl fmt::Display for Revision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:016x}", self.0)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Revision {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(self)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Revision {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let text = <String as serde::Deserialize>::deserialize(d)?;
        let hex = text.len() == 16 && text.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'));
        match u64::from_str_radix(&text, 16) {
            Ok(v) if hex => Ok(Revision(v)),
            _ => Err(serde::de::Error::custom(format!(
                "a revision is 16 lowercase hex digits, got {text:?}"
            ))),
        }
    }
}

/// FNV-1a 64.
struct Fnv(u64);

impl Fnv {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;

    fn new() -> Self {
        Fnv(Self::OFFSET)
    }

    fn bytes(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 ^= u64::from(b);
            self.0 = self.0.wrapping_mul(Self::PRIME);
        }
    }

    fn u64(&mut self, v: u64) {
        self.bytes(&v.to_le_bytes());
    }

    fn i64(&mut self, v: i64) {
        self.bytes(&v.to_le_bytes());
    }

    fn f64(&mut self, v: f64) {
        self.u64(v.to_bits());
    }

    fn bool(&mut self, v: bool) {
        self.bytes(&[u8::from(v)]);
    }

    fn len(&mut self, n: usize) {
        self.u64(u64::try_from(n).unwrap_or(u64::MAX));
    }

    fn str(&mut self, s: &str) {
        self.len(s.len());
        self.bytes(s.as_bytes());
    }

    fn opt<T>(&mut self, v: Option<T>, some: impl FnOnce(&mut Self, T)) {
        match v {
            None => self.bytes(&[0]),
            Some(x) => {
                self.bytes(&[1]);
                some(self, x);
            }
        }
    }

    fn strs<'a>(&mut self, items: impl ExactSizeIterator<Item = &'a String>) {
        self.len(items.len());
        for item in items {
            self.str(item);
        }
    }
}

/// the revision of `count` nodes given in tree order, and the focus.
pub(crate) fn revision<'a>(
    count: usize,
    tree_order: impl Iterator<Item = &'a Node>,
    focused: Option<&SemId>,
) -> Revision {
    let mut h = Fnv::new();
    h.str(VERSION);
    h.len(count);
    for node in tree_order {
        encode_node(&mut h, node);
    }
    h.opt(focused, |h, id| h.str(id.as_str()));
    Revision(h.0)
}

fn encode_node(h: &mut Fnv, node: &Node) {
    h.str(node.id.as_str());
    h.str(node.role.as_str());
    h.str(&node.label);
    h.opt(node.description.as_deref(), Fnv::str);
    h.opt(node.parent.as_ref(), |h, p| h.str(p.as_str()));
    let mut state: Vec<_> = node.state.iter().collect();
    state.sort_unstable_by(|a, b| a.0.cmp(b.0));
    h.len(state.len());
    for (key, value) in state {
        h.str(key.as_str());
        encode_value(h, value);
    }
    h.len(node.actions.len());
    for action in &node.actions {
        encode_action(h, action);
    }
    encode_constraints(h, &node.constraints);
    h.bool(node.enabled);
    h.bool(node.inert);
    h.bool(node.focusable);
}

fn encode_value(h: &mut Fnv, value: &Value) {
    h.str(value.kind().as_str());
    match value {
        Value::Bool(v) => h.bool(*v),
        Value::Int(v) => h.i64(*v),
        Value::Num(v) => h.f64(*v),
        Value::Text(v) => h.str(v),
        Value::List(items) => h.strs(items.iter()),
        Value::Set(items) => h.strs(items.iter()),
    }
}

fn encode_action(h: &mut Fnv, action: &ActionDecl) {
    h.str(action.name.as_str());
    h.opt(action.label.as_deref(), Fnv::str);
    h.bool(action.enabled);
    h.len(action.reads.len());
    for read in &action.reads {
        h.str(&read.to_string());
    }
    h.opt(action.param, |h, p| {
        h.str(p.kind.as_str());
        h.bool(p.constrained);
    });
    encode_policy(h, &action.policy);
}

fn encode_policy(h: &mut Fnv, policy: &Policy) {
    h.bool(policy.actors.human);
    h.bool(policy.actors.agent);
    h.bool(policy.actors.assistive);
    h.bool(policy.actors.test);
    h.str(policy.confirm.as_str());
    h.bool(policy.destructive);
}

fn encode_constraints(h: &mut Fnv, c: &Constraints) {
    h.bool(c.editable);
    h.bool(c.required);
    h.opt(c.range, |h, (low, high)| {
        h.f64(low);
        h.f64(high);
    });
    h.opt(c.step, Fnv::f64);
    h.opt(c.max_len, Fnv::len);
    h.opt(c.options.as_ref(), |h, options| h.strs(options.iter()));
    h.bool(c.secret);
}

#[cfg(test)]
mod tests {
    use super::Fnv;

    fn fnv(bytes: &[u8]) -> u64 {
        let mut h = Fnv::new();
        h.bytes(bytes);
        h.0
    }

    #[test]
    fn fnv1a_64_matches_the_reference_vectors() {
        assert_eq!(fnv(b""), 0xcbf2_9ce4_8422_2325);
        assert_eq!(fnv(b"a"), 0xaf63_dc4c_8601_ec8c);
        assert_eq!(fnv(b"foobar"), 0x8594_4171_f739_67e8);
    }

    #[test]
    fn numbers_are_little_endian_and_strings_length_prefixed() {
        let mut h = Fnv::new();
        h.u64(0x0102_0304_0506_0708);
        assert_eq!(h.0, fnv(&[8, 7, 6, 5, 4, 3, 2, 1]));
        let mut h = Fnv::new();
        h.str("ab");
        assert_eq!(h.0, fnv(&[2, 0, 0, 0, 0, 0, 0, 0, b'a', b'b']));
        let mut h = Fnv::new();
        h.i64(-1);
        assert_eq!(h.0, fnv(&[0xff; 8]));
    }

    #[test]
    fn adjacent_strings_cannot_alias() {
        let mut ab_c = Fnv::new();
        ab_c.str("ab");
        ab_c.str("c");
        let mut a_bc = Fnv::new();
        a_bc.str("a");
        a_bc.str("bc");
        assert_ne!(ab_c.0, a_bc.0);
    }
}
