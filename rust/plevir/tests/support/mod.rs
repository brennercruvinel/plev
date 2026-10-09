//! A small app projection shaped like the ide's, its registry, and a valid
//! IR over it that each rule test breaks in one place, and the JSON
//! schema validator (`jsonschema`). Each test binary uses its own part.
#![allow(dead_code)]

pub mod jsonschema;

use std::collections::BTreeSet;

use accord::{Actors, Confirm, Constraints, ParamSpec, Policy, Role, SnapshotBuilder, ValueKind};
use serde_json::{Value, json};
use plevir::{Capabilities, GenerationPolicy, IrError, Rule};

/// The registry of a projection like the ide's (changes with two files,
/// the commit form), plus the odd capabilities the policy rules need.
pub fn caps() -> Capabilities {
    let mut b = SnapshotBuilder::new();
    let files = vec!["a.rs".to_owned(), "b.rs".to_owned()];
    b.node("changes", Role::List, "changes")
        .state("files", files)
        .state("staged", BTreeSet::from(["a.rs".to_owned()]));
    let discard = Policy {
        confirm: Confirm::ForAgents,
        destructive: true,
        ..Policy::default()
    };
    for path in ["a.rs", "b.rs"] {
        let id = format!("changes.file[{path}]");
        let mut file = b.node(id.as_str(), Role::ListItem, path);
        file.parent("changes")
            .state("status", "modified")
            .state("staged", path == "a.rs");
        file.action("stage");
        file.action("unstage");
        file.action("discard").policy(discard);
    }
    b.node("commit.form", Role::Group, "commit")
        .state("open", true)
        .action("cancel");
    b.node("commit.message", Role::TextInput, "commit message")
        .parent("commit.form")
        .state("text", "fix parser")
        .constraints(Constraints {
            editable: true,
            ..Constraints::default()
        })
        .action("set_value")
        .param(ParamSpec {
            kind: ValueKind::Text,
            constrained: true,
        });
    b.node("commit.submit", Role::Button, "commit")
        .parent("commit.form")
        .action("submit");
    b.node("counter", Role::Status, "2 files")
        .state("count", 2i64);
    b.node("admin", Role::Button, "reset everything")
        .action("reset")
        .policy(Policy {
            actors: Actors {
                agent: false,
                ..Actors::default()
            },
            ..Policy::default()
        });
    b.node("trash", Role::Button, "empty the trash")
        .action("empty")
        .policy(Policy {
            confirm: Confirm::Always,
            destructive: true,
            ..Policy::default()
        });
    b.node("odd[x]", Role::ListItem, "x").state("v", true);
    b.node("odd[y]", Role::ListItem, "y").state("v", 1i64);
    b.node("half[x]", Role::ListItem, "x").action("go");
    b.node("half[y]", Role::ListItem, "y").state("v", true);
    Capabilities::from_snapshot(&b.finish().expect("the projection validates"))
}

/// A surface that verifies against `caps()`: a list of the changed files
/// with a status and a stage button per file, the message and the commit.
pub fn base() -> Value {
    json!({
        "plev_ir": 1,
        "surface": {
            "id": "review",
            "title": "review the commit",
            "root": {
                "sem": "review.root",
                "layout": { "stack": { "axis": "vertical", "gap": "md" } },
                "children": [
                    {
                        "sem": "review.files",
                        "component": "list",
                        "label": "files",
                        "repeat": {
                            "each": "changes:files",
                            "item": {
                                "sem": "review.file[{item}]",
                                "layout": { "stack": { "axis": "horizontal", "gap": "sm" } },
                                "children": [
                                    {
                                        "sem": "review.file[{item}].status",
                                        "component": "badge",
                                        "bind": { "state": { "text": "changes.file[{item}]:status" } }
                                    },
                                    {
                                        "sem": "review.file[{item}].stage",
                                        "component": "button",
                                        "label": "stage",
                                        "bind": { "actions": { "activate": {
                                            "target": "changes.file[{item}]", "action": "stage"
                                        } } }
                                    }
                                ]
                            }
                        }
                    },
                    {
                        "sem": "review.message",
                        "component": "text_field",
                        "label": "commit message",
                        "bind": {
                            "state": { "text": "commit.message:text" },
                            "actions": { "set_value": {
                                "target": "commit.message", "action": "set_value"
                            } }
                        }
                    },
                    {
                        "sem": "review.commit",
                        "component": "button",
                        "label": "commit",
                        "bind": { "actions": { "activate": {
                            "target": "commit.submit", "action": "submit"
                        } } }
                    }
                ]
            }
        }
    })
}

/// Where the parts of `base()` sit.
pub const FILES: &str = "/surface/root/children/0";
pub const ITEM: &str = "/surface/root/children/0/repeat/item";
pub const STATUS: &str = "/surface/root/children/0/repeat/item/children/0";
pub const STAGE: &str = "/surface/root/children/0/repeat/item/children/1";
pub const MESSAGE: &str = "/surface/root/children/1";
pub const COMMIT: &str = "/surface/root/children/2";

/// `doc` with `value` at `pointer`: an existing value is replaced, a new
/// key is added to its object, one past the end of a list is pushed.
pub fn with(mut doc: Value, pointer: &str, value: Value) -> Value {
    if let Some(slot) = doc.pointer_mut(pointer) {
        *slot = value;
        return doc;
    }
    let (parent, key) = pointer.rsplit_once('/').expect("a pointer with a key");
    match doc.pointer_mut(parent) {
        Some(Value::Object(object)) => {
            object.insert(key.to_owned(), value);
        }
        Some(Value::Array(list)) if key == list.len().to_string() => list.push(value),
        _ => panic!("nothing to add {key} to at {parent}"),
    }
    doc
}

/// `doc` without the key at `pointer`.
pub fn without(mut doc: Value, pointer: &str) -> Value {
    let (parent, key) = pointer.rsplit_once('/').expect("a pointer with a key");
    doc.pointer_mut(parent)
        .and_then(Value::as_object_mut)
        .unwrap_or_else(|| panic!("no object at {parent}"))
        .remove(key);
    doc
}

/// The errors of `doc` under `policy`, read and verified.
pub fn errors_under(doc: &Value, policy: &GenerationPolicy) -> Vec<IrError> {
    match plevir::verify_json(&doc.to_string(), &caps(), policy) {
        Ok(_) => Vec::new(),
        Err(errors) => errors,
    }
}

pub fn errors(doc: &Value) -> Vec<IrError> {
    errors_under(doc, &GenerationPolicy::default())
}

/// `doc` fails with an error of `rule` at `pointer` whose reason says
/// `says`.
#[track_caller]
pub fn refused(doc: &Value, rule: Rule, pointer: &str, says: &str) {
    refused_under(doc, &GenerationPolicy::default(), rule, pointer, says);
}

#[track_caller]
pub fn refused_under(
    doc: &Value,
    policy: &GenerationPolicy,
    rule: Rule,
    pointer: &str,
    says: &str,
) {
    let errors = errors_under(doc, policy);
    let hit = errors
        .iter()
        .any(|e| e.rule == rule && e.pointer == pointer && e.reason.contains(says));
    assert!(
        hit,
        "expected {rule:?} at {pointer:?} saying {says:?}, got:\n{errors:#?}"
    );
}
