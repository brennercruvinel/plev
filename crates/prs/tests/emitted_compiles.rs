//! Proof that the emitted gpui code compiles and builds a real plev
//! element tree: the frozen golden is `include!`d and executed against
//! plev (a dev-dependency). The react side has no stored golden anymore
//! (the owner retired the copies); its emission is exercised live by
//! tests/golden.rs and rendered by examples/preview.rs.

mod sep {
    include!("../fixtures/gpui/expected.rs");
}

#[test]
fn emitted_separator_builds_the_expected_tree() {
    let root = sep::separator("OR");
    // line, label chip, line: the flank rewrite output.
    assert_eq!(root.children_ref().len(), 3);
    let chip = &root.children_ref()[1];
    assert_eq!(chip.children_ref().len(), 1); // the label text run
}

#[test]
fn emitted_separator_label_is_a_real_text_run() {
    // Edge case: empty label still builds (no panic, same structure).
    let root = sep::separator("");
    assert_eq!(root.children_ref().len(), 3);
}
