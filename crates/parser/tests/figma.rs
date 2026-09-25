//! The Figma importer (task-71 phase 2): one screen of a real design file
//! (the "settings" frame of the dark theme), exported by
//! `script/figma-fixture` into the REST API's shape (`fixtures/figma`);
//! its IR, report and builder code as goldens; every node and every key of
//! the export in the report, by a census read straight from the parse; the
//! IR verified with nothing declared, since a design names no capability.
//! The test never reads the .fig, nor the network.
//!
//! To write the goldens again after an intentional change:
//! UPDATE_GOLDEN=1 cargo test -p parser --test figma

// shared with the other test binaries: this one reads the figma fixture.
#[allow(dead_code)]
mod support;

use parser::Category;
use parser::figma::FigNode;
use support::checks::*;
use support::*;
use uir::{Capabilities, GenerationPolicy, verify};

const FILE: &str = FIGMA;

fn source() -> String {
    fixture("figma/settings.json")
}

/// Every layer (at its `id`) and every key of it but `name` and `type`.
fn census(src: &str) -> Vec<Site> {
    fn walk(n: &FigNode, out: &mut Vec<Site>) {
        out.push(site(FILE, n.line, n.kind.clone()));
        for (k, _, line) in &n.keys {
            out.push(site(FILE, *line, k.clone()));
        }
        for c in &n.children {
            walk(c, out);
        }
    }
    let doc = parser::figma::parse_figma(src).expect("figma");
    let mut out = Vec::new();
    walk(&doc.document, &mut out);
    out
}

#[test]
fn the_figma_settings_ir_its_report_and_its_code() {
    let imported = import_figma();
    goldens(&imported, "figma");
    let code = parser::transpile_figma((FILE, &source())).expect("figma");
    check_golden("figma/expected.rs", &code.code);
    let r = &imported.report;
    for c in Category::ALL {
        assert!(r.count(c) > 0, "no {c:?} entry");
    }
    let kinds = |what: &str| -> Vec<Category> {
        r.entries
            .iter()
            .filter(|e| e.construct.contains(what))
            .map(|e| e.category())
            .collect()
    };
    // auto layout is the stack; a vector and an ellipse have no IR form
    assert!(
        kinds("layoutMode")
            .iter()
            .all(|c| *c == Category::Supported)
    );
    assert!(kinds("ELLIPSE").iter().all(|c| *c == Category::Impossible));
    assert!(kinds("VECTOR").iter().all(|c| *c == Category::Impossible));
    // the screen's children are placed by position: the rule stacks them
    // in y order and says by how much each one moves
    assert!(kinds("absoluteBoundingBox at").contains(&Category::Approximated));
    // what the export could not apply is reported, never lost
    assert!(
        kinds("unappliedOverrides")
            .iter()
            .all(|c| *c == Category::Impossible)
    );
    assert!(!kinds("unappliedOverrides").is_empty());
}

#[test]
fn nothing_of_the_figma_settings_is_lost_or_invented() {
    let imported = import_figma();
    let src = source();
    nothing_is_lost(&imported.report, &census(&src), &[(FILE, &src)]);
    nothing_is_invented(&imported);
}

#[test]
fn a_design_names_no_capability_so_the_ir_verifies_as_it_is() {
    let imported = import_figma();
    assert!(imported.report.bindings.is_empty());
    let verified = verify(
        &imported.ir,
        &Capabilities::default(),
        &GenerationPolicy::default(),
    )
    .unwrap_or_else(|e| panic!("{e:#?}"));
    assert!(verified.focus_order().is_empty(), "no control is invented");
}

#[test]
fn a_figma_import_is_deterministic_and_a_broken_one_an_error() {
    assert_eq!(import_figma(), import_figma());
    let broken = parser::import_figma(("x.json", "{\"document\": 1}"), &hoff());
    assert!(matches!(broken, Err(parser::ParserError::Parse(_))));
}
