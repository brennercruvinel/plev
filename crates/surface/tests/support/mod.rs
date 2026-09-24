//! What the surface tests share: the fixtures of task-68 (the IR and the
//! projections they are written against, read from `crates/uir/fixtures`).

use std::path::PathBuf;

use sem::{SemId, Snapshot};
use uir::{Capabilities, GenerationPolicy, Ir, VerifiedIr, parse, verify};

pub const COMMIT_FORM: &str = "ide-commit-form.ir.json";
pub const REVIEW: &str = "ide-review-commit.ir.json";
pub const TODO: &str = "showcase-todo.ir.json";
pub const IDE: &str = "ide-workbench.snapshot.json";
pub const SHOWCASE: &str = "showcase-todo.snapshot.json";

/// Each IR fixture and the projection it is written against.
pub const CASES: [(&str, &str); 3] = [(COMMIT_FORM, IDE), (REVIEW, IDE), (TODO, SHOWCASE)];

pub fn fixture(name: &str) -> String {
    let at = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../uir/fixtures")
        .join(name);
    std::fs::read_to_string(&at).unwrap_or_else(|e| panic!("{}: {e}", at.display()))
}

pub fn snapshot(name: &str) -> Snapshot {
    serde_json::from_str(&fixture(name)).expect("a sem snapshot")
}

pub fn ir(name: &str) -> Ir {
    parse(&fixture(name)).unwrap_or_else(|errors| panic!("{name}: {errors:#?}"))
}

/// The fixture `ir` verified against the registry of `app`.
pub fn verified(ir_name: &str, app: &Snapshot) -> VerifiedIr {
    verify(
        &ir(ir_name),
        &Capabilities::from_snapshot(app),
        &GenerationPolicy::default(),
    )
    .unwrap_or_else(|errors| panic!("{ir_name}: {errors:#?}"))
}

pub fn id(text: &str) -> SemId {
    SemId::parse(text).unwrap()
}
