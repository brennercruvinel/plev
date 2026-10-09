//! The `code` component (task-97): a block of text a row per line, each
//! line with its kind when the app says it. It verifies by the binding
//! rules every component follows (a state it shows, of a kind it takes; a
//! label that names it; no children), round trips, and the schema knows
//! it. An IR written before it existed reads, verifies and writes back to
//! the same bytes.

// the validator for the keywords the schema uses (no schema dependency).
mod jsonschema;

use std::path::PathBuf;

use accord::{Role, Snapshot, SnapshotBuilder};
use serde_json::{Value, json};
use plevir::{
    Capabilities, Component, GenerationPolicy, Ir, IrError, LineKind, Rule, parse, schema, verify,
};

/// A diff as an app projects it: its lines, the kind of each, what it
/// shows the diff of, and a text that is not a list.
fn app() -> Snapshot {
    let mut b = SnapshotBuilder::new();
    let lines = vec![
        "src/lib.rs: @@ -1,2 +1,2 @@".to_owned(),
        " fn main() {".to_owned(),
        "-    old();".to_owned(),
        "+    new();".to_owned(),
    ];
    let kinds: Vec<String> = [
        LineKind::Header,
        LineKind::Context,
        LineKind::Removed,
        LineKind::Added,
    ]
    .iter()
    .map(|k| k.as_str().to_owned())
    .collect();
    b.node("diff", Role::Region, "diff")
        .state("target", "stacks.commit[c1]")
        .state("lines", lines)
        .state("kinds", kinds)
        .state("count", 4i64);
    b.node("file", Role::Region, "file")
        .state("text", "fn main() {}\n");
    b.finish().expect("the projection validates")
}

fn caps() -> Capabilities {
    Capabilities::from_snapshot(&app())
}

/// A surface with one `code` node, `code` merged into it.
fn doc(code: Value) -> Value {
    let mut node = json!({ "sem": "review.lines", "component": "code", "label": "diff" });
    if let (Some(node), Some(extra)) = (node.as_object_mut(), code.as_object()) {
        for (k, v) in extra {
            node.insert(k.clone(), v.clone());
        }
    }
    json!({
        "plev_ir": 1,
        "surface": {
            "id": "review",
            "title": "the diff",
            "root": { "sem": "review.root", "children": [node] }
        }
    })
}

fn verified(v: &Value) -> Result<plevir::VerifiedIr, Vec<IrError>> {
    let ir: Ir = serde_json::from_value(v.clone()).expect("it reads");
    verify(&ir, &caps(), &GenerationPolicy::default())
}

fn refused(v: &Value, rule: Rule, pointer: &str, reason: &str) {
    let errors = verified(v).expect_err("refused");
    assert!(
        errors
            .iter()
            .any(|e| e.rule == rule && e.pointer == pointer && e.reason.contains(reason)),
        "{rule:?} at {pointer} ({reason}): {errors:#?}"
    );
}

#[test]
fn a_code_bound_to_the_lines_and_their_kinds_verifies_and_round_trips() {
    let v = doc(json!({
        "size": { "max": "dropdown_max_h" },
        "bind": { "state": { "lines": "diff:lines", "kinds": "diff:kinds" } }
    }));
    let ok = verified(&v).unwrap_or_else(|e| panic!("{e:#?}"));
    assert!(
        ok.focus_order().is_empty(),
        "a code is read, not operated: no stop of the focus order"
    );
    let ir: Ir = serde_json::from_value(v.clone()).unwrap();
    let code = &ir.surface.root.children[0];
    assert_eq!(code.component, Some(Component::Code));
    assert_eq!(
        serde_json::to_value(&ir).unwrap(),
        v,
        "it writes what it read"
    );
    let text = serde_json::to_string(&ir).unwrap();
    assert_eq!(parse(&text).unwrap(), ir);
    assert_eq!(jsonschema::validate(&schema(), &v), Vec::<String>::new());
}

#[test]
fn a_code_shows_one_text_as_its_lines_and_goes_without_kinds() {
    let v = doc(json!({ "bind": { "state": { "lines": "file:text" } } }));
    assert!(verified(&v).is_ok(), "{:#?}", verified(&v));
}

#[test]
fn a_code_is_held_to_the_binding_rules_of_every_component() {
    // `kinds` is a list; a number is not
    refused(
        &doc(json!({ "bind": { "state": { "lines": "diff:lines", "kinds": "diff:count" } } })),
        Rule::State,
        "/surface/root/children/0/bind/state/kinds",
        "a code's `kinds` takes list",
    );
    // a state it does not show, and one the app does not declare
    refused(
        &doc(json!({ "bind": { "state": { "rows": "diff:lines" } } })),
        Rule::State,
        "/surface/root/children/0/bind/state/rows",
        "a code shows no `rows`; it shows `lines`, `kinds`",
    );
    refused(
        &doc(json!({ "bind": { "state": { "lines": "diff:gone" } } })),
        Rule::State,
        "/surface/root/children/0/bind/state/lines",
        "the app declares no state `diff:gone`",
    );
    // it sends nothing
    refused(
        &doc(json!({ "bind": { "actions": { "activate": { "target": "diff", "action": "x" } } } })),
        Rule::Action,
        "/surface/root/children/0/bind/actions/activate",
        "a code sends no `activate`; it sends nothing",
    );
    // a label names it, and it holds no children
    let mut unnamed = doc(json!({ "bind": { "state": { "lines": "diff:lines" } } }));
    if let Some(node) = unnamed.pointer_mut("/surface/root/children/0") {
        node.as_object_mut().map(|n| n.remove("label"));
    }
    refused(
        &unnamed,
        Rule::Access,
        "/surface/root/children/0/label",
        "a code needs a label",
    );
    refused(
        &doc(
            json!({ "children": [{ "sem": "review.lines.x", "component": "text", "label": "x" }] }),
        ),
        Rule::Vocabulary,
        "/surface/root/children/0/children",
        "a code holds no children",
    );
}

#[test]
fn the_schema_knows_the_code_and_its_two_names() {
    let s = schema();
    let components = &s["$defs"]["component"]["anyOf"][0]["enum"];
    assert_eq!(
        components.as_array().and_then(|a| a.last()),
        Some(&json!("code"))
    );
    let names = s["$defs"]["bind"]["properties"]["state"]["propertyNames"]["enum"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    assert_eq!(names[names.len() - 2..], [json!("lines"), json!("kinds")]);
    let bad = doc(json!({ "bind": { "state": { "diff": "diff:lines" } } }));
    assert!(!jsonschema::validate(&s, &bad).is_empty());
}

fn path(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

/// Every IR fixture written before the `code`, in this crate and in the
/// surface's tests.
fn fixtures_of_before() -> Vec<PathBuf> {
    let mut out = Vec::new();
    for dir in ["tests/fixtures", "../screen/tests/fixtures"] {
        let Ok(entries) = std::fs::read_dir(path(dir)) else {
            continue;
        };
        for entry in entries.flatten() {
            let p = entry.path();
            if p.to_string_lossy().ends_with(".ir.json") {
                out.push(p);
            }
        }
    }
    out.sort();
    out
}

/// `v` written with every object's keys in order, whatever order the
/// reader kept them in: two documents that mean the same are the same
/// bytes here.
fn canonical(v: &Value) -> String {
    match v {
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            let fields: Vec<String> = keys
                .into_iter()
                .map(|k| format!("{}:{}", Value::String(k.clone()), canonical(&map[k])))
                .collect();
            format!("{{{}}}", fields.join(","))
        }
        Value::Array(items) => {
            let items: Vec<String> = items.iter().map(canonical).collect();
            format!("[{}]", items.join(","))
        }
        other => other.to_string(),
    }
}

#[test]
fn an_ir_of_before_reads_and_writes_back_the_same_document() {
    let found = fixtures_of_before();
    assert_eq!(found.len(), 8, "{found:?}");
    for at in &found {
        let text = std::fs::read_to_string(at).expect("the fixture");
        assert!(
            !text.contains("\"code\""),
            "{} is of before the code",
            at.display()
        );
        let ir = parse(&text).unwrap_or_else(|e| panic!("{}: {e:#?}", at.display()));
        let kept: Value = serde_json::from_str(&text).expect("json");
        let written = serde_json::to_value(&ir).expect("it writes");
        assert!(
            canonical(&written) == canonical(&kept),
            "{} writes back another document",
            at.display()
        );
        assert_eq!(
            jsonschema::validate(&schema(), &kept),
            Vec::<String>::new(),
            "{}",
            at.display()
        );
    }
    // the three of this crate are in the writer's own order: byte a byte
    for name in [
        "ide-commit-form.ir.json",
        "ide-review-commit.ir.json",
        "showcase-todo.ir.json",
    ] {
        let text = std::fs::read_to_string(path(&format!("tests/fixtures/{name}"))).expect("it");
        let mut written = serde_json::to_string_pretty(&parse(&text).expect("reads")).unwrap();
        written.push('\n');
        assert!(written == text, "{name} writes back other bytes");
    }
}
