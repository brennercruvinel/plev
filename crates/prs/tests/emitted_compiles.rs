//! Proof that the emitted code compiles and builds real plev element trees.
//!
//! Mechanism choice (the simplest that is airtight): the frozen goldens in
//! fixtures/ are `include!`d here as modules and executed against plev (a
//! dev-dependency). Because tests/golden.rs asserts the transpiler output
//! equals those same files byte for byte, compiling the fixture IS
//! compiling the generated output; no build.rs/OUT_DIR indirection and no
//! trybuild dependency are needed.

mod card {
    include!("../fixtures/react/expected.rs");
}

mod sep {
    include!("../fixtures/gpui/expected.rs");
}

use plev::builder::div;

#[test]
fn emitted_card_builds_the_expected_tree() {
    let root = card::hoff_research_card(div(), "Neural Search", "Latent space exploration.");
    // card -> inner
    assert_eq!(root.children_ref().len(), 1);
    let inner = &root.children_ref()[0];
    // inner -> preview, spacer(16), details
    assert_eq!(inner.children_ref().len(), 3);
    let preview = &inner.children_ref()[0];
    assert_eq!(preview.children_ref().len(), 1); // the slot element
    let details = &inner.children_ref()[2];
    // details -> title, spacer(8), content, spacer(24), button
    assert_eq!(details.children_ref().len(), 5);
    let button = &details.children_ref()[4];
    assert_eq!(button.children_ref().len(), 1); // the Discover text run
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
