//! short plain-english forms of what the session holds: a value, a
//! change, a stage, a rejection, a role. the timeline keeps them in its
//! entries and the text views print them.

use sem::{Change, Outcome, Rejection, Role, Stage, Value};

/// a value as a person reads it: text quoted, a list in brackets, a set in
/// braces.
pub(crate) fn value(v: &Value) -> String {
    match v {
        Value::Bool(b) => b.to_string(),
        Value::Int(i) => i.to_string(),
        Value::Num(n) => n.to_string(),
        Value::Text(s) => format!("{s:?}"),
        Value::List(items) => format!("[{}]", items.join(", ")),
        Value::Set(items) => {
            let items: Vec<&str> = items.iter().map(String::as_str).collect();
            format!("{{{}}}", items.join(", "))
        }
    }
}

/// a value that may be absent.
pub(crate) fn maybe(v: Option<&Value>) -> String {
    v.map_or_else(|| "(none)".to_owned(), value)
}

/// what moved in one value: the items that came and went for a list or a
/// set, the value before and after otherwise.
pub(crate) fn moved(
    before: Option<&Value>,
    after: Option<&Value>,
    added: &[String],
    removed: &[String],
) -> String {
    let mut parts = Vec::new();
    if !added.is_empty() {
        parts.push(format!("added {}", added.join(", ")));
    }
    if !removed.is_empty() {
        parts.push(format!("removed {}", removed.join(", ")));
    }
    if parts.is_empty() {
        format!("{} -> {}", maybe(before), maybe(after))
    } else {
        parts.join("; ")
    }
}

/// one change, on one line.
pub(crate) fn change(c: &Change) -> String {
    match c {
        Change::NodeAdded(node) => format!("{} added", node.id),
        Change::NodeRemoved(id) => format!("{id} removed"),
        Change::StateChanged {
            at,
            before,
            after,
            added,
            removed,
        } => format!(
            "{at}: {}",
            moved(before.as_ref(), after.as_ref(), added, removed)
        ),
        Change::LabelChanged {
            node,
            before,
            after,
        } => format!("{node} label {before:?} -> {after:?}"),
        Change::ActionsChanged { node, .. } => format!("{node} actions"),
        Change::FlagsChanged {
            node,
            enabled,
            inert,
            focusable,
        } => {
            let flags: Vec<String> = [
                ("enabled", enabled),
                ("inert", inert),
                ("focusable", focusable),
            ]
            .into_iter()
            .filter_map(|(name, v)| v.map(|on| format!("{name} {}", on_off(on))))
            .collect();
            format!("{node} {}", flags.join(", "))
        }
        Change::ParentChanged { node, after, .. } => match after {
            Some(parent) => format!("{node} moved under {parent}"),
            None => format!("{node} moved to the root"),
        },
        Change::RoleChanged { node, after, .. } => format!("{node} role -> {}", role(*after)),
        Change::DescriptionChanged { node, .. } => format!("{node} description"),
        Change::ConstraintsChanged { node, .. } => format!("{node} constraints"),
        Change::Reordered { parent } => match parent {
            Some(parent) => format!("{parent} children reordered"),
            None => "roots reordered".to_owned(),
        },
        Change::FocusChanged { after, .. } => match after {
            Some(id) => format!("focus -> {id}"),
            None => "focus -> none".to_owned(),
        },
    }
}

fn on_off(on: bool) -> &'static str {
    if on { "on" } else { "off" }
}

/// a role by its serde name (`button`, `text_input`).
pub(crate) fn role(r: Role) -> String {
    match serde_json::to_value(r) {
        Ok(serde_json::Value::String(name)) => name,
        _ => format!("{r:?}"),
    }
}

/// why a request was refused, by the rejection's serde name.
pub(crate) fn rejection(r: &Rejection) -> &'static str {
    match r {
        Rejection::NotFound { .. } => "not_found",
        Rejection::UnknownAction { .. } => "unknown_action",
        Rejection::NotAuthorized { .. } => "not_authorized",
        Rejection::Stale { .. } => "stale",
        Rejection::Inert { .. } => "inert",
        Rejection::Disabled { .. } => "disabled",
        Rejection::InvalidParam { .. } => "invalid_param",
        Rejection::InvalidProjection { .. } => "invalid_projection",
        Rejection::Declined { .. } => "declined",
    }
}

/// an outcome's named results, `name = value`, comma separated.
pub(crate) fn outcome(o: &Outcome) -> String {
    o.result
        .iter()
        .map(|(name, v)| format!("{name} = {}", value(v)))
        .collect::<Vec<_>>()
        .join(", ")
}

/// what a stage says beyond its name: the rejection, the outcome, the
/// error, who confirmed.
pub(crate) fn stage_detail(s: &Stage) -> Option<String> {
    match s {
        Stage::Rejected(r) => Some(rejection(r).to_owned()),
        Stage::Completed(o) if !o.result.is_empty() => Some(outcome(o)),
        Stage::Failed { error } => Some(error.clone()),
        Stage::Confirmed { by } => Some(format!("by {by}")),
        _ => None,
    }
}
