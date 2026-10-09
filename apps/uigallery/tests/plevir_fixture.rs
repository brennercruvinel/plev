//! The todo page's projection as the plev IR fixtures read it (task-68).
//!
//! The page staged headless as `showcase --render Todo --semantics` stages
//! it, projected and written in the form `plev observe` serves
//! (`uigallery::shell::semantics_json`, without `source`) to
//! `rust/plevir/tests/fixtures/uigallery-todo.snapshot.json`. The todo IR fixture
//! builds its capability registry from it (`plevir::Capabilities::from_snapshot`):
//! the registry is the page's own projection, never a second declaration
//! by hand.
//!
//! The file is a golden. This test fails when the projection moves away
//! from it; `UIR_BLESS=1 cargo test -p uigallery --test uir_fixture` writes
//! it again, and the uir tests then say whether the IR fixture still
//! verifies. Desktop only, like the headless staging it uses.
//!
//! The Silences page (task-69 phase 4a) has its golden the same way,
//! `rust/plevir/tests/fixtures/uigallery-silences.snapshot.json`, kept by a test
//! of the page itself (`view::apps::silences::tests::the_app_projection_is_the_uir_golden`):
//! since phase 4b the page's semantics carry the proposing row too, and
//! the golden is the app alone (the fixed screen as it opens, seeded, no
//! ffmpeg run), the registry the generated surface verifies against.
#![cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]

use std::path::PathBuf;

use accord::Snapshot;
use engine::drawing::headless::Viewport;
use uigallery::shell::{semantics_json, stage_page};

const FIXTURE: &str = "../../rust/plevir/tests/fixtures/uigallery-todo.snapshot.json";

#[test]
fn the_todo_projection_is_the_uir_fixture() {
    let page = stage_page(Viewport::new(1200.0, 800.0, 1.0), "Todo", "core", None)
        .expect("the todo page stages");
    let snap = page.semantics().expect("the todo page projects");
    print!("{}", capabilities(&snap));
    let mut text = semantics_json(&snap, false).expect("json prints");
    text.push('\n');
    golden(FIXTURE, &text);
}

/// What the probe of task-68 reads: every state with its kind and every
/// action with its parameter and policy, one per line.
fn capabilities(snap: &Snapshot) -> String {
    let mut out = String::new();
    for node in snap.nodes().values() {
        for (key, value) in &node.state {
            let kind = value.kind().as_str();
            out.push_str(&format!("data {}:{key} {kind}\n", node.id));
        }
        for action in &node.actions {
            let param = action.param.map_or("none".to_owned(), |p| format!("{p:?}"));
            let policy = format!("{:?}", action.policy);
            let name = &action.name;
            out.push_str(&format!(
                "action {} {name} param={param} policy={policy}\n",
                node.id
            ));
        }
    }
    out
}

/// Compares `text` with the fixture at `rel`, or writes it with
/// `UIR_BLESS=1`.
fn golden(rel: &str, text: &str) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    if std::env::var_os("UIR_BLESS").is_some() {
        std::fs::write(&path, text).expect("the fixture writes");
        return;
    }
    let kept = std::fs::read_to_string(&path).unwrap_or_default();
    assert!(
        kept == text,
        "the projection moved away from {rel}; regenerate it with \
         UIR_BLESS=1 cargo test -p uigallery --test plevir_fixture, then run cargo test -p plevir"
    );
}
