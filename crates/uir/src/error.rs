//! `IrError`: one reason an IR may not exist, where it is (a JSON pointer
//! into the IR) and which rule of the verifier it breaks. the decoder and
//! the verifier return every error at once, the way the parser's droplist
//! reports every construct it drops.

use std::fmt;

use serde::Serialize;

/// the eight checks of the verifier, in the order it applies them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Rule {
    /// 1: the JSON shape and the version.
    Schema,
    /// 2: every id valid and unique on the surface.
    Identity,
    /// 3: a known component, props by token, layout by primitives.
    Vocabulary,
    /// 4: every bound state exists and its kind fits the component.
    State,
    /// 5: every bound action exists and its parameter fits the event.
    Action,
    /// 6: the generation policy: destructive actions and who may bind.
    Policy,
    /// 7: accessibility: names, focus order, no control that is only
    /// visual.
    Access,
    /// 8: limits on nodes, depth and text.
    Limits,
}

impl Rule {
    /// the number of the rule, 1 to 8.
    pub fn number(self) -> u8 {
        match self {
            Rule::Schema => 1,
            Rule::Identity => 2,
            Rule::Vocabulary => 3,
            Rule::State => 4,
            Rule::Action => 5,
            Rule::Policy => 6,
            Rule::Access => 7,
            Rule::Limits => 8,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Rule::Schema => "schema",
            Rule::Identity => "identity",
            Rule::Vocabulary => "vocabulary",
            Rule::State => "state",
            Rule::Action => "action",
            Rule::Policy => "policy",
            Rule::Access => "access",
            Rule::Limits => "limits",
        }
    }
}

/// one reason an IR may not exist.
///
/// `pointer` is a JSON pointer (RFC 6901) into the IR's JSON form: `""`
/// is the whole document, `/surface/root/children/0/bind/actions/activate`
/// one binding. the serde form is what a model gets back to repair its IR.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct IrError {
    pub rule: Rule,
    pub pointer: String,
    pub reason: String,
}

impl IrError {
    pub(crate) fn new(rule: Rule, at: &Ptr, reason: impl Into<String>) -> IrError {
        IrError {
            rule,
            pointer: at.0.clone(),
            reason: reason.into(),
        }
    }
}

impl fmt::Display for IrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let at = if self.pointer.is_empty() {
            "(the document)"
        } else {
            self.pointer.as_str()
        };
        let (n, name) = (self.rule.number(), self.rule.as_str());
        write!(f, "{at}: {} (rule {n}, {name})", self.reason)
    }
}

impl std::error::Error for IrError {}

/// a JSON pointer being built, one step at a time.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Ptr(String);

impl Ptr {
    /// the whole document.
    pub(crate) fn root() -> Ptr {
        Ptr(String::new())
    }

    /// this pointer followed by an object key, `~` and `/` escaped as
    /// RFC 6901 says.
    pub(crate) fn key(&self, key: &str) -> Ptr {
        let mut out = String::with_capacity(self.0.len() + key.len() + 1);
        out.push_str(&self.0);
        out.push('/');
        for c in key.chars() {
            match c {
                '~' => out.push_str("~0"),
                '/' => out.push_str("~1"),
                c => out.push(c),
            }
        }
        Ptr(out)
    }

    /// this pointer followed by an array index.
    pub(crate) fn index(&self, at: usize) -> Ptr {
        Ptr(format!("{}/{at}", self.0))
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pointer_escapes_its_keys() {
        let at = Ptr::root().key("surface").key("a/b~c").index(3);
        assert_eq!(at.as_str(), "/surface/a~1b~0c/3");
        let e = IrError::new(Rule::Action, &at, "no such action");
        assert_eq!(
            e.to_string(),
            "/surface/a~1b~0c/3: no such action (rule 5, action)"
        );
        let whole = IrError::new(Rule::Schema, &Ptr::root(), "not JSON");
        assert_eq!(
            whole.to_string(),
            "(the document): not JSON (rule 1, schema)"
        );
        assert_eq!(
            serde_json::to_value(&e).unwrap(),
            serde_json::json!({
                "rule": "action",
                "pointer": "/surface/a~1b~0c/3",
                "reason": "no such action"
            })
        );
    }
}
