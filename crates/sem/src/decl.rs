//! what bounds a node (`Constraints`) and what it accepts (`ActionDecl`
//! with its `Policy`). all of it is data: applying a policy or checking a
//! precondition belongs to `check` and the `Dispatcher`.

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
    pub editable: bool,
    pub required: bool,
    /// inclusive `(low, high)`, both finite, `low <= high`.
    pub range: Option<(f64, f64)>,
    /// finite and above zero.
    pub step: Option<f64>,
    pub max_len: Option<usize>,
    /// the accepted choices, without repeats.
    pub options: Option<Vec<String>>,
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

/// when an action asks for confirmation before it runs.
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
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Policy {
    pub actors: Actors,
    pub confirm: Confirm,
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
    pub label: Option<String>,
    pub enabled: bool,
    pub reads: Vec<StateRef>,
    pub param: Option<ParamSpec>,
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
