//! A semantic change in a few words, and the node it is about.

use accord::{Change, Value};

/// The node a semantic change is about (`None`: the page, for the focus
/// and the order of the roots), and the change in a few words.
pub fn describe(change: &Change) -> (Option<String>, String) {
    let id = |id: &accord::SemId| Some(id.to_string());
    match change {
        Change::NodeAdded(node) => (
            id(&node.id),
            format!("added ({} \"{}\")", node.role.as_str(), node.label),
        ),
        Change::NodeRemoved(node) => (id(node), "removed".to_string()),
        Change::StateChanged {
            at, before, after, ..
        } => (
            id(&at.node),
            format!(
                "{} {} -> {}",
                at.key,
                value(before.as_ref()),
                value(after.as_ref())
            ),
        ),
        Change::LabelChanged {
            node,
            before,
            after,
        } => (id(node), format!("label \"{before}\" -> \"{after}\"")),
        Change::ActionsChanged {
            node,
            before,
            after,
        } => {
            let names = |a: &[accord::ActionDecl]| {
                a.iter()
                    .map(|d| d.name.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            (
                id(node),
                format!("actions [{}] -> [{}]", names(before), names(after)),
            )
        }
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
            .iter()
            .filter_map(|(name, v)| v.map(|v| format!("{name} -> {v}")))
            .collect();
            (id(node), flags.join(", "))
        }
        Change::ParentChanged {
            node,
            before,
            after,
        } => (
            id(node),
            format!("parent {} -> {}", opt(before), opt(after)),
        ),
        Change::RoleChanged {
            node,
            before,
            after,
        } => (
            id(node),
            format!("role {} -> {}", before.as_str(), after.as_str()),
        ),
        Change::DescriptionChanged { node, .. } => (id(node), "description changed".to_string()),
        Change::ConstraintsChanged { node, .. } => (id(node), "constraints changed".to_string()),
        Change::Reordered { parent: Some(p) } => (id(p), "children reordered".to_string()),
        Change::Reordered { parent: None } => (None, "roots reordered".to_string()),
        Change::FocusChanged { before, after } => {
            (None, format!("focus {} -> {}", opt(before), opt(after)))
        }
    }
}

fn opt(id: &Option<accord::SemId>) -> String {
    id.as_ref().map_or("none".to_string(), ToString::to_string)
}

/// A state value as its JSON (`true`, `3`, `"text"`), `none` when absent.
fn value(v: Option<&Value>) -> String {
    let Some(v) = v else {
        return "none".to_string();
    };
    match v {
        Value::Bool(b) => b.to_string(),
        Value::Int(i) => i.to_string(),
        Value::Num(n) => n.to_string(),
        Value::Text(t) => format!("{t:?}"),
        Value::List(items) => format!("{items:?}"),
        Value::Set(items) => format!("{items:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use accord::{SemId, StateRef};

    fn id(text: &str) -> SemId {
        match SemId::parse(text) {
            Ok(id) => id,
            Err(e) => panic!("{e}"),
        }
    }

    #[test]
    fn a_change_is_told_by_the_node_it_is_about() {
        let label = Change::LabelChanged {
            node: id("todo.item[2]"),
            before: "Walk the dog".to_string(),
            after: "Walk the cat".to_string(),
        };
        assert_eq!(
            describe(&label),
            (
                Some("todo.item[2]".to_string()),
                "label \"Walk the dog\" -> \"Walk the cat\"".to_string()
            )
        );
        let at: StateRef = match "todo.item[2]:completed".parse() {
            Ok(at) => at,
            Err(e) => panic!("{e}"),
        };
        let state = Change::StateChanged {
            at,
            before: Some(Value::Bool(false)),
            after: Some(Value::Bool(true)),
            added: vec![],
            removed: vec![],
        };
        assert_eq!(describe(&state).1, "completed false -> true");
        let focus = Change::FocusChanged {
            before: None,
            after: Some(id("todo.input")),
        };
        assert_eq!(
            describe(&focus),
            (None, "focus none -> todo.input".to_string())
        );
    }
}
