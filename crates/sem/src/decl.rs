//! what bounds a node (`Constraints`) and what it accepts (`ActionDecl`
//! with its `Policy`). all of it is data: applying a policy or checking a
//! precondition belongs to `check` and the `Dispatcher`.
//!
//! the serde form is lean: a field at its default is not written, and a
//! field left out reads as its default, so a JSON that spells every
//! default out (the form before task-57b) reads the same. the helpers at
//! the end say what "default" is for each field; `enabled` is the one
//! flag that is on unless said otherwise.

use crate::name::ActionName;
use crate::value::{StateRef, ValueKind};

/// the conditions that bound a node's value.
///
/// `secret` marks a value that never leaves the process (password, token):
/// the snapshot keeps only its length, so the recorder, the agent and the
/// accessibility tree never see it.
///
/// equality compares the floats by bits, like `Value::Num`.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Constraints {
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "is_false"))]
    pub editable: bool,
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "is_false"))]
    pub required: bool,
    /// inclusive `(low, high)`, both finite, `low <= high`.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub range: Option<(f64, f64)>,
    /// finite and above zero.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub step: Option<f64>,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub max_len: Option<usize>,
    /// the accepted choices, without repeats.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub options: Option<Vec<String>>,
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "is_false"))]
    pub secret: bool,
}

impl Constraints {
    /// every reason these constraints are invalid, in field order.
    pub(crate) fn faults(&self) -> Vec<String> {
        let mut out = Vec::new();
        if let Some((low, high)) = self.range {
            if !(low.is_finite() && high.is_finite()) {
                out.push(format!("range ({low}, {high}) is not finite"));
            } else if low > high {
                out.push(format!("range low {low} is above high {high}"));
            }
        }
        if let Some(step) = self.step {
            if !(step.is_finite() && step > 0.0) {
                out.push(format!("step {step} is not finite and above zero"));
            }
        }
        if let Some(options) = &self.options {
            for (at, option) in options.iter().enumerate() {
                if options.iter().take(at).any(|seen| seen == option) {
                    out.push(format!("option {option:?} appears more than once"));
                }
            }
        }
        out
    }
}

fn bits(pair: Option<(f64, f64)>) -> Option<(u64, u64)> {
    pair.map(|(a, b)| (a.to_bits(), b.to_bits()))
}

impl PartialEq for Constraints {
    fn eq(&self, other: &Self) -> bool {
        self.editable == other.editable
            && self.required == other.required
            && bits(self.range) == bits(other.range)
            && self.step.map(f64::to_bits) == other.step.map(f64::to_bits)
            && self.max_len == other.max_len
            && self.options == other.options
            && self.secret == other.secret
    }
}

impl Eq for Constraints {}

/// who may request an action. the default is everyone: the same interface
/// serves a person, an agent, assistive technology and a test.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Actors {
    pub human: bool,
    pub agent: bool,
    pub assistive: bool,
    pub test: bool,
}

impl Default for Actors {
    fn default() -> Self {
        Actors {
            human: true,
            agent: true,
            assistive: true,
            test: true,
        }
    }
}

/// when an action asks for confirmation before it runs: `ForAgents` asks
/// it of an agent's requests, `Always` of every actor's. a person confirms,
/// at the screen or through assistive technology.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum Confirm {
    #[default]
    Never,
    ForAgents,
    Always,
}

impl Confirm {
    /// the stable name, the same one serde uses.
    pub fn as_str(self) -> &'static str {
        match self {
            Confirm::Never => "never",
            Confirm::ForAgents => "for_agents",
            Confirm::Always => "always",
        }
    }
}

/// who may request an action and under which care. data only here.
///
/// in the serde form `actors` is written whole when any actor is left out,
/// and not at all when everyone may ask.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Policy {
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "is_default"))]
    pub actors: Actors,
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "is_default"))]
    pub confirm: Confirm,
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "is_false"))]
    pub destructive: bool,
}

/// the parameter an action takes: the kind of value expected and whether
/// the node's `Constraints` apply to it (a slider's `set` does, a free
/// `rename` may not).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ParamSpec {
    pub kind: ValueKind,
    pub constrained: bool,
}

/// an action a node accepts.
///
/// `reads` is the state the action depends on: the list a precondition
/// compares. every entry names a node and a key of the same snapshot.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ActionDecl {
    pub name: ActionName,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub label: Option<String>,
    #[cfg_attr(
        feature = "serde",
        serde(default = "yes", skip_serializing_if = "is_true")
    )]
    pub enabled: bool,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub reads: Vec<StateRef>,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub param: Option<ParamSpec>,
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "is_default"))]
    pub policy: Policy,
}

impl ActionDecl {
    /// an enabled action with no label, no reads, no parameter and the
    /// default policy.
    pub fn new(name: ActionName) -> Self {
        ActionDecl {
            name,
            label: None,
            enabled: true,
            reads: Vec::new(),
            param: None,
            policy: Policy::default(),
        }
    }
}

/// true when `v` is its type's default: the serde form leaves it out.
#[cfg(feature = "serde")]
pub(crate) fn is_default<T: Default + PartialEq>(v: &T) -> bool {
    *v == T::default()
}

#[cfg(feature = "serde")]
pub(crate) fn is_false(b: &bool) -> bool {
    !*b
}

#[cfg(feature = "serde")]
pub(crate) fn is_true(b: &bool) -> bool {
    *b
}

/// the default of a flag that is on unless said otherwise (`enabled`).
#[cfg(feature = "serde")]
pub(crate) fn yes() -> bool {
    true
}
