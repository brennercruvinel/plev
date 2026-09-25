//! What a screen reader asks, read back into the contract: an AccessKit
//! `ActionRequest` becomes a `sem::Request` with `Actor::Assistive`, which
//! the app sends through the same `Dispatcher` a click, an agent and a
//! test use. Nothing here runs app logic; there is no second path.
//!
//! - `Click` asks for the node's primary action: the first of
//!   `activate`, `toggle`, `submit` and `select` it declares without a
//!   parameter (a click carries none). A button, a link or a menu item
//!   that declares none of them and exactly one action without a
//!   parameter (`propose`, `open_file`, `detect_silence`) is pressed by
//!   that one, as a person's click presses it; with two or more, a click
//!   would have to guess, and it has none.
//! - `SetValue` asks for `set_value` with the value as the parameter, in
//!   the kind the action declares (a number for a slider, a text for a
//!   field).
//! - `Increment` and `Decrement` ask for `increment` and `decrement`.
//! - `Focus` is not a capability: moving keyboard focus is editing, as a
//!   click into a field is for a person (see the ADR
//!   semantic-binding-beside-the-scene), so it comes back as
//!   `Assist::Focus` for the app to move its own focus, the way it answers
//!   Tab. Only a focusable node that is not inert takes it.
//!
//! The request is observed over the snapshot the screen reader was shown
//! (`Shown`): a person using a screen reader decides on what they heard,
//! so when that state moved since, the dispatcher answers `Stale` with
//! the delta, as it does for any actor. A confirmation the app asks for
//! is answered by a person, and a person using a screen reader is one:
//! the app hands a request aimed at its confirmation to
//! `Dispatcher::confirm` with the request's actor, `Actor::Assistive`.

use std::fmt;

use accesskit::{Action, ActionData, ActionRequest, NodeId, TreeId};
use sem::{ActionDecl, Actor, Node, Request, Role, SemError, SemId, Value, ValueKind};

use super::tree::Shown;

/// The actions a click may stand for, in the order the primary one is
/// picked.
pub const PRIMARY: [&str; 4] = ["activate", "toggle", "submit", "select"];

/// What the screen reader asked, for the app to carry out.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Assist {
    /// A capability, to go through the app's dispatcher.
    Request(Request),
    /// Keyboard focus to this node: the app moves its own focus there,
    /// as it does for Tab.
    Focus(SemId),
}

/// Why a screen reader's request carries nothing to the app. None of
/// these is an error of the app: the request is logged and dropped, as
/// AccessKit asks of an unsupported one.
#[derive(Debug, PartialEq)]
pub enum Unmapped {
    /// The request names a tree this app did not build.
    OtherTree(TreeId),
    /// The window itself: it has no capability.
    Root,
    /// No node of the tree shown has this id.
    UnknownNode(NodeId),
    /// A click on a node with no primary action.
    NoPrimaryAction(SemId),
    /// Focus on a node that is not focusable, or is inert.
    NotFocusable(SemId),
    /// `SetValue` without a value to set.
    NoValue(SemId),
    /// An action the contract has no counterpart for yet.
    Unsupported(Action),
    /// The node does not declare the action, or a read has no value in
    /// the snapshot shown.
    Contract(SemError),
}

impl fmt::Display for Unmapped {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OtherTree(tree) => write!(f, "the request names another tree ({tree:?})"),
            Self::Root => f.write_str("the window has no action"),
            Self::UnknownNode(node) => write!(f, "no node {node:?} in the tree shown"),
            Self::NoPrimaryAction(id) => write!(f, "{id} has no primary action to click"),
            Self::NotFocusable(id) => write!(f, "{id} does not take focus"),
            Self::NoValue(id) => write!(f, "set value on {id} carries no value"),
            Self::Unsupported(action) => write!(f, "{action:?} has no counterpart yet"),
            Self::Contract(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for Unmapped {}

/// The action a click on `node` asks for: the first of `PRIMARY` it
/// declares without a parameter, else, on a node whose role exists to be
/// pressed, its one action without a parameter.
pub fn primary(node: &Node) -> Option<&ActionDecl> {
    let named = PRIMARY.iter().find_map(|name| {
        node.actions
            .iter()
            .find(|a| a.name.as_str() == *name && a.param.is_none())
    });
    named.or_else(|| sole(node))
}

/// The one action without a parameter of a button, a link or a menu
/// item; `None` for any other role, or when there are two or more.
fn sole(node: &Node) -> Option<&ActionDecl> {
    if !matches!(node.role, Role::Button | Role::Link | Role::MenuItem) {
        return None;
    }
    let mut bare = node.actions.iter().filter(|a| a.param.is_none());
    match (bare.next(), bare.next()) {
        (Some(one), None) => Some(one),
        _ => None,
    }
}

/// Reads `req` back against what the screen reader was shown.
pub fn translate(req: &ActionRequest, shown: &Shown) -> Result<Assist, Unmapped> {
    if req.target_tree != TreeId::ROOT {
        return Err(Unmapped::OtherTree(req.target_tree));
    }
    if req.target_node == super::ids::ROOT {
        return Err(Unmapped::Root);
    }
    let id = shown
        .ids
        .sem(req.target_node)
        .ok_or(Unmapped::UnknownNode(req.target_node))?;
    let node = shown
        .snapshot
        .get(id)
        .ok_or(Unmapped::UnknownNode(req.target_node))?;
    let (action, param) = match req.action {
        Action::Click => {
            let decl = primary(node).ok_or_else(|| Unmapped::NoPrimaryAction(id.clone()))?;
            (decl.name.as_str(), None)
        }
        Action::Focus => {
            return if node.focusable && !node.inert {
                Ok(Assist::Focus(id.clone()))
            } else {
                Err(Unmapped::NotFocusable(id.clone()))
            };
        }
        Action::SetValue => ("set_value", Some(value(node, req.data.as_ref(), id)?)),
        Action::Increment => ("increment", None),
        Action::Decrement => ("decrement", None),
        other => return Err(Unmapped::Unsupported(other)),
    };
    Request::observe(&shown.snapshot, Actor::Assistive, id, action, param)
        .map(Assist::Request)
        .map_err(Unmapped::Contract)
}

/// The parameter of a `SetValue`: in the kind `set_value` declares when
/// the value converts without loss, else as it came (the dispatcher
/// refuses a kind that does not fit, with the reason in the receipt).
fn value(node: &Node, data: Option<&ActionData>, id: &SemId) -> Result<Value, Unmapped> {
    let kind = node
        .actions
        .iter()
        .find(|a| a.name.as_str() == "set_value")
        .and_then(|a| a.param)
        .map(|p| p.kind);
    Ok(match (data, kind) {
        (Some(ActionData::Value(text)), Some(ValueKind::Num)) => match text.trim().parse() {
            Ok(n) => Value::Num(n),
            Err(_) => Value::Text(text.to_string()),
        },
        (Some(ActionData::Value(text)), Some(ValueKind::Int)) => match text.trim().parse() {
            Ok(n) => Value::Int(n),
            Err(_) => Value::Text(text.to_string()),
        },
        (Some(ActionData::Value(text)), _) => Value::Text(text.to_string()),
        (Some(ActionData::NumericValue(n)), Some(ValueKind::Int)) => whole(*n),
        (Some(ActionData::NumericValue(n)), Some(ValueKind::Text)) => Value::Text(n.to_string()),
        (Some(ActionData::NumericValue(n)), _) => Value::Num(*n),
        _ => return Err(Unmapped::NoValue(id.clone())),
    })
}

/// A whole number as an `Int`, anything else as the `Num` it is.
fn whole(n: f64) -> Value {
    // i64 holds every whole f64 in this range exactly.
    const LIMIT: f64 = 9_007_199_254_740_992.0;
    if n.fract() == 0.0 && n.abs() <= LIMIT {
        Value::Int(n as i64)
    } else {
        Value::Num(n)
    }
}

#[cfg(test)]
mod tests;
