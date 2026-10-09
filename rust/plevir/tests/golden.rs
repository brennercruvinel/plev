//! The golden IR of three surfaces, each verified against the capability
//! registry of the real projection it was written for:
//!
//! - `ide-commit-form.ir.json`, the ide's commit form, the fixed screen,
//!   with the ide's own ids: a fixed screen fits the IR too;
//! - `ide-review-commit.ir.json`, a surface generated over the same
//!   capabilities, "review what goes into the commit": the changed files
//!   repeated over `changes:files`, stage and unstage per file, the
//!   message and the commit;
//! - `showcase-todo.ir.json`, a surface over the showcase's todo: the
//!   field and the add button, the items repeated over `todo:items` (each
//!   one's text and a checkbox for its `toggle`), the filter as chips and
//!   the counters.
//!
//! The registries come from `ide-workbench.snapshot.json` and
//! `showcase-todo.snapshot.json`, which the projection tests of the ide and
//! the showcase write (`crates/ide/tests/uir_fixture.rs`,
//! `crates/showcase/tests/uir_fixture.rs`). The schema file is
//! `uir::schema()` written out; `UIR_BLESS=1` writes it again.

// the validator for the keywords the schema uses (no schema dependency).
#[allow(dead_code)]
mod support;

use std::path::PathBuf;

use accord::Snapshot;
use plevir::{Capabilities, GenerationPolicy, Ir, Rule, parse, schema, verify};
use serde_json::{Value, json};

const COMMIT_FORM: &str = "ide-commit-form.ir.json";
const REVIEW: &str = "ide-review-commit.ir.json";
const TODO: &str = "showcase-todo.ir.json";
const IDE: &str = "ide-workbench.snapshot.json";
const SHOWCASE: &str = "showcase-todo.snapshot.json";
const SCHEMA: &str = "assets/plev-ir-1.json";

fn path(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn fixture(name: &str) -> String {
    let at = path(&format!("tests/fixtures/{name}"));
    std::fs::read_to_string(&at).unwrap_or_else(|e| panic!("{}: {e}", at.display()))
}

fn registry(snapshot: &str) -> Capabilities {
    let snap: Snapshot = serde_json::from_str(&fixture(snapshot)).expect("a sem snapshot");
    Capabilities::from_snapshot(&snap)
}

fn ir(name: &str) -> Ir {
    parse(&fixture(name)).unwrap_or_else(|errors| panic!("{name}: {errors:#?}"))
}

/// Each IR fixture, the registry it is written for, and the focus order
/// its verification gives.
fn cases() -> [(&'static str, &'static str, &'static [&'static str]); 3] {
    [
        (
            COMMIT_FORM,
            IDE,
            &["commit.message", "commit.cancel", "commit.submit"],
        ),
        (
            REVIEW,
            IDE,
            &[
                "review.file[{item}].stage",
                "review.file[{item}].unstage",
                "review.message",
                "review.commit",
            ],
        ),
        (
            TODO,
            SHOWCASE,
            &[
                "todo_panel.input",
                "todo_panel.add",
                "todo_panel.item[{item}].done",
                "todo_panel.filter.all",
                "todo_panel.filter.active",
                "todo_panel.filter.completed",
            ],
        ),
    ]
}

#[test]
fn every_fixture_verifies_against_the_projection_it_was_written_for() {
    for (name, snapshot, focus) in cases() {
        let verified = verify(&ir(name), &registry(snapshot), &GenerationPolicy::default())
            .unwrap_or_else(|errors| panic!("{name}: {errors:#?}"));
        let order: Vec<&str> = verified
            .focus_order()
            .iter()
            .map(|id| id.as_str())
            .collect();
        assert_eq!(order, focus, "{name}");
    }
}

#[test]
fn a_fixture_verified_against_the_other_apps_registry_is_refused() {
    let errors = verify(&ir(TODO), &registry(IDE), &GenerationPolicy::default()).unwrap_err();
    assert!(
        errors
            .iter()
            .all(|e| matches!(e.rule, Rule::State | Rule::Action))
    );
    assert!(
        errors
            .iter()
            .any(|e| e.pointer == "/surface/root/children/0/children/1/bind/actions/activate"),
        "{errors:#?}"
    );
    assert!(
        verify(
            &ir(REVIEW),
            &registry(SHOWCASE),
            &GenerationPolicy::default()
        )
        .is_err()
    );
}

#[test]
fn the_review_template_names_one_capability_per_changed_file() {
    let caps = registry(IDE);
    let files: Vec<String> = caps
        .actions()
        .iter()
        .filter(|a| a.action.as_str() == "stage")
        .map(|a| a.target.to_string())
        .collect();
    assert_eq!(
        files,
        [
            "changes.file[src/lexer.rs]",
            "changes.file[notes.md]",
            "changes.file[src/parser.rs]"
        ]
    );
    let each = caps.datum(&"changes:files".parse().unwrap()).unwrap();
    assert_eq!(each.kind, accord::ValueKind::List);
}

#[test]
fn a_fixture_reads_and_writes_back_unchanged() {
    for (name, _, _) in cases() {
        let text = fixture(name);
        let written: Value = serde_json::to_value(ir(name)).unwrap();
        let kept: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(written, kept, "{name}");
        assert_eq!(
            serde_json::from_str::<Ir>(&text).unwrap(),
            ir(name),
            "{name}"
        );
    }
}

#[test]
fn the_schema_accepts_every_fixture() {
    let schema = schema();
    for (name, _, _) in cases() {
        let doc: Value = serde_json::from_str(&fixture(name)).unwrap();
        assert_eq!(
            support::jsonschema::validate(&schema, &doc),
            Vec::<String>::new(),
            "{name}"
        );
    }
}

#[test]
fn the_schema_refuses_a_literal_an_unknown_name_and_a_second_form() {
    let schema = schema();
    let doc: Value = serde_json::from_str(&fixture(COMMIT_FORM)).unwrap();
    let edits = [
        ("/surface/root/props/background", json!("#1a1a1a")),
        ("/surface/root/props/padding", json!(12)),
        ("/surface/root/children/0/component", json!("video")),
        (
            "/surface/root/children/0/bind/state",
            json!({ "colour": "commit.message:text" }),
        ),
        (
            "/surface/root/layout",
            json!({ "split": ["horizontal", 50] }),
        ),
        (
            "/surface/root/layout",
            json!({ "split": { "axis": "horizontal", "ratio": 5 } }),
        ),
        ("/plev_ir", json!(2)),
    ];
    for (at, value) in edits {
        let mut bad = doc.clone();
        *bad.pointer_mut(at).unwrap() = value;
        assert!(
            !support::jsonschema::validate(&schema, &bad).is_empty(),
            "{at}"
        );
    }
}

#[test]
fn the_schema_file_is_what_schema_writes() {
    let mut text = serde_json::to_string_pretty(&schema()).unwrap();
    text.push('\n');
    let at = path(SCHEMA);
    if std::env::var_os("UIR_BLESS").is_some() {
        std::fs::write(&at, &text).unwrap();
        return;
    }
    let kept = std::fs::read_to_string(&at).unwrap_or_default();
    assert!(
        kept == text,
        "{SCHEMA} is not what uir::schema() writes; UIR_BLESS=1 cargo test -p uir writes it again"
    );
}
