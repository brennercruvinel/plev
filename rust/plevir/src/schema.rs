//! the JSON schema of the IR, for constrained decoding: a model that
//! writes under it cannot write a component, a token, a field or a
//! binding name the vocabulary lacks.
//!
//! it is built from the same tables the types use (`Component::ALL`,
//! each token's `ALL`, the names the specs accept), so it moves with them.
//! `crates/uir/schema/plev-ir-1.json` is this value written out; a test
//! keeps the two equal (`UIR_BLESS=1 cargo test -p uir` writes the file
//! again). the schema checks shape only: ids, references, kinds, policy
//! and accessibility are the verifier's.

use serde_json::{Value, json};

use crate::ir::VERSION;
use crate::token::{
    Align, Axis, ColorRole, ControlSize, Elevation, MAX_FLEX, MAX_RATIO, MIN_RATIO, ShapeRole,
    SizeToken, SpacingToken, TypographyRole,
};
use crate::vocab::{ChartKind, Component, names};

/// an object with exactly these properties, `required` among them.
fn object(required: &[&str], properties: Value) -> Value {
    json!({
        "type": "object",
        "additionalProperties": false,
        "required": required,
        "properties": properties,
    })
}

fn one_of_names(names: impl IntoIterator<Item = &'static str>) -> Value {
    json!({ "enum": names.into_iter().collect::<Vec<_>>() })
}

/// an object with one key, `kind`, holding `fields`.
fn variant(kind: &str, required: &[&str], fields: Value) -> Value {
    let inner = serde_json::Map::from_iter([(kind.to_owned(), object(required, fields))]);
    object(&[kind], Value::Object(inner))
}

fn reference(def: &str) -> Value {
    json!({ "$ref": format!("#/$defs/{def}") })
}

/// the schema of `Ir` version 1 (JSON Schema, draft 2020-12).
pub fn schema() -> Value {
    let (state, events) = names();
    let units = Component::ALL
        .iter()
        .filter(|c| !matches!(c, Component::Chart(_)))
        .map(|c| c.name());
    let charts = ChartKind::ALL.iter().map(|k| k.as_str());
    let node = object(
        &["sem"],
        json!({
            "sem": reference("sem_id"),
            "component": reference("component"),
            "label": { "type": "string" },
            "description": { "type": "string" },
            "props": reference("props"),
            "layout": reference("layout"),
            "size": reference("size"),
            "bind": reference("bind"),
            "children": { "type": "array", "items": reference("node") },
            "repeat": reference("repeat"),
        }),
    );
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "plev IR, version 1",
        "description": "an interface surface described by meaning: identity, a component of a closed vocabulary, props by theme token, layout by primitives, and bindings to the state and the actions the app declares. only what the verifier accepts reaches the runtime.",
        "type": "object",
        "additionalProperties": false,
        "required": ["plev_ir", "surface"],
        "properties": {
            "plev_ir": { "const": VERSION },
            "surface": reference("surface"),
        },
        "$defs": {
            "sem_id": {
                "type": "string",
                "minLength": 1,
                "description": "a SemId in canonical text: name segments [a-z0-9_]+ joined by '.', a segment may be followed by bracketed keys ('changes.file[src/lib.rs]'). inside a repeat, every id carries the key [{item}] once.",
            },
            "state_ref": {
                "type": "string",
                "minLength": 3,
                "description": "a piece of app state, '<sem_id>:<key>' ('commit.message:text').",
            },
            "action_name": {
                "type": "string",
                "minLength": 1,
                "description": "an action name, [a-z0-9_]+.",
            },
            "surface": object(&["id", "title", "root"], json!({
                "id": reference("sem_id"),
                "title": { "type": "string" },
                "root": reference("node"),
            })),
            "node": node,
            "component": {
                "anyOf": [
                    one_of_names(units),
                    object(&["chart"], json!({ "chart": one_of_names(charts) })),
                ],
            },
            "props": object(&[], json!({
                "padding": reference("spacing"),
                "typography": reference("typography"),
                "color": reference("color"),
                "background": reference("color"),
                "elevation": reference("elevation"),
                "control_size": reference("control_size"),
                "shape": reference("shape"),
            })),
            "layout": {
                "anyOf": [
                    { "const": "overlay" },
                    variant("stack", &["axis"], json!({
                        "axis": reference("axis"),
                        "gap": reference("spacing"),
                        "align": reference("align"),
                    })),
                    variant("grid", &["min_col"], json!({
                        "min_col": reference("size_token"),
                        "gap": reference("spacing"),
                    })),
                    variant("split", &["axis", "ratio"], json!({
                        "axis": reference("axis"),
                        "ratio": { "type": "integer", "minimum": MIN_RATIO, "maximum": MAX_RATIO },
                    })),
                ],
            },
            "size": object(&[], json!({
                "min": reference("size_token"),
                "max": reference("size_token"),
                "flex": { "type": "integer", "minimum": 1, "maximum": MAX_FLEX },
            })),
            "bind": object(&[], json!({
                "state": {
                    "type": "object",
                    "propertyNames": one_of_names(state),
                    "additionalProperties": reference("state_ref"),
                },
                "actions": {
                    "type": "object",
                    "propertyNames": one_of_names(events),
                    "additionalProperties": reference("capability"),
                },
            })),
            "capability": object(&["target", "action"], json!({
                "target": reference("sem_id"),
                "action": reference("action_name"),
            })),
            "repeat": object(&["each", "item"], json!({
                "each": reference("state_ref"),
                "item": reference("node"),
            })),
            "spacing": one_of_names(SpacingToken::ALL.iter().map(|t| t.as_str())),
            "typography": one_of_names(TypographyRole::ALL.iter().map(|t| t.as_str())),
            "color": one_of_names(ColorRole::ALL.iter().map(|t| t.as_str())),
            "elevation": one_of_names(Elevation::ALL.iter().map(|t| t.as_str())),
            "control_size": one_of_names(ControlSize::ALL.iter().map(|t| t.as_str())),
            "shape": one_of_names(ShapeRole::ALL.iter().map(|t| t.as_str())),
            "size_token": one_of_names(SizeToken::ALL.iter().map(|t| t.as_str())),
            "axis": one_of_names(Axis::ALL.iter().map(|t| t.as_str())),
            "align": one_of_names(Align::ALL.iter().map(|t| t.as_str())),
        },
    })
}
