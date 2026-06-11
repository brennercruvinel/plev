//! Golden tests: real corpus inputs (copied under fixtures/) -> frozen
//! expected rust output, compared byte for byte. The droplist is part of
//! the API contract, so these tests also assert it tells the truth: exact
//! counts and the presence of every known-lossy construct.
//!
//! To regenerate after an intentional emitter change:
//! UPDATE_GOLDEN=1 cargo test -p prs --test golden

use std::fs;
use std::path::PathBuf;

fn fixture(rel: &str) -> String {
    fs::read_to_string(fixture_path(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

fn fixture_path(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(rel)
}

fn check_golden(rel: &str, actual: &str) {
    let path = fixture_path(rel);
    if std::env::var("UPDATE_GOLDEN").is_ok() {
        fs::write(&path, actual).unwrap();
        return;
    }
    let expected = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("missing golden {rel} (run with UPDATE_GOLDEN=1): {e}"));
    assert_eq!(expected, actual, "golden mismatch for {rel}");
}

fn react_output() -> prs::Transpiled {
    prs::transpile_react(
        ("index.tsx", &fixture("react/index.tsx")),
        (
            "HoffResearchCard.module.sass",
            &fixture("react/HoffResearchCard.module.sass"),
        ),
        &fixture("react/hoff-research-card-variables.sass"),
    )
    .expect("react transpile")
}

fn gpui_output() -> prs::Transpiled {
    prs::transpile_gpui(("separator.rs", &fixture("gpui/separator.rs"))).expect("gpui transpile")
}

#[test]
#[ignore = "diagnostic: cargo test -p prs --test golden -- --ignored --nocapture"]
fn dump_droplists() {
    let r = react_output();
    eprintln!("react mapped={} dropped={}", r.mapped, r.dropped.len());
    for d in &r.dropped {
        eprintln!("  [{}] {} -- {}", d.at, d.what, d.why);
    }
    let g = gpui_output();
    eprintln!("gpui mapped={} dropped={}", g.mapped, g.dropped.len());
    for d in &g.dropped {
        eprintln!("  [{}] {} -- {}", d.at, d.what, d.why);
    }
}

#[test]
fn react_card_golden_bytes() {
    let out = react_output();
    check_golden("react/expected.rs", &out.code);
}

#[test]
fn gpui_separator_golden_bytes() {
    let out = gpui_output();
    check_golden("gpui/expected.rs", &out.code);
}

#[test]
fn react_card_emit_is_deterministic() {
    assert_eq!(react_output().code, react_output().code);
}

#[test]
fn react_droplist_tells_the_truth() {
    let out = react_output();
    let has = |frag: &str| {
        assert!(
            out.dropped.iter().any(|d| d.what.contains(frag)),
            "droplist missing: {frag}\n{:#?}",
            out.dropped
        );
    };
    // Known-lossy constructs of the card, each with file:line.
    has("buttonCircle> subtree"); // hover animation layer
    has("@keyframes button-circle");
    has("conditional class previewBorder");
    has("mask-image"); // gradient border fades
    has("font-family"); // Rubik is not an embedded face
    has(".hoff-research-cardSquare"); // modifier variants
    has(".hoff-research-cardHorizontal");
    has("@media"); // responsive variants
    has("z-index");
    has("position: relative");
    has("cursor");
    has("opacity");
    for d in &out.dropped {
        assert!(
            d.at.contains(':'),
            "droplist entry without file:line: {d:?}"
        );
        assert!(!d.why.is_empty());
    }
    // Frozen totals: a mapper change must consciously update these.
    assert_eq!(out.mapped, 51, "mapped declarations changed");
    assert_eq!(out.dropped.len(), 38, "droplist length changed");
}

#[test]
fn gpui_droplist_tells_the_truth() {
    let out = gpui_output();
    let has = |frag: &str| {
        assert!(
            out.dropped.iter().any(|d| d.what.contains(frag)),
            "droplist missing: {frag}\n{:#?}",
            out.dropped
        );
    };
    has(".refine_style(..)"); // runtime style merge
    has(".mx_auto()"); // replaced by the flank rewrite
    has(".absolute() line overlay"); // replaced by the flank rewrite
    has("match arm Axis::Vertical"); // variant not transpiled
    has("match arm SeparatorStyle::Dashed"); // canvas path
    has("self.color override");
    for d in &out.dropped {
        assert!(d.at.starts_with("separator.rs:"), "bad location: {d:?}");
    }
    assert_eq!(out.mapped, 18, "mapped builder calls changed");
    assert_eq!(out.dropped.len(), 6, "droplist length changed");
}
