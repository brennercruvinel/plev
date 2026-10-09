//! `SemId`: canonical form, roundtrip, rejections, escapes.

use proptest::prelude::*;
use accord::{SemError, SemId};

fn id(text: &str) -> SemId {
    SemId::parse(text).expect("valid id")
}

fn reason(text: &str) -> String {
    match SemId::parse(text) {
        Err(SemError::InvalidId { text: t, reason }) => {
            assert_eq!(t, text);
            reason
        }
        other => panic!("{text:?} should be rejected, got {other:?}"),
    }
}

#[test]
fn the_examples_of_the_spec_parse_and_print_back() {
    for text in [
        "commit.submit",
        "changes.file[src/lib.rs]",
        "silence.segment[3].preview",
        "a",
        "a_1.b2[]",
        "list[x][y].z",
    ] {
        assert_eq!(id(text).to_string(), text);
        assert_eq!(id(text).as_str(), text);
    }
}

#[test]
fn builders_produce_the_canonical_form() {
    let changes = SemId::name("changes").unwrap();
    let file = changes.child("file").unwrap().key("src/lib.rs");
    assert_eq!(file.to_string(), "changes.file[src/lib.rs]");
    let preview = SemId::name("silence")
        .unwrap()
        .child("segment")
        .unwrap()
        .key("3")
        .child("preview")
        .unwrap();
    assert_eq!(preview, id("silence.segment[3].preview"));
}

#[test]
fn empty_is_rejected() {
    assert_eq!(reason(""), "empty id");
}

#[test]
fn uppercase_is_rejected() {
    assert!(reason("Commit").contains("'C'"));
    assert!(reason("commit.Submit").contains("'S'"));
    assert!(SemId::name("Commit").is_err());
    assert!(SemId::name("commit").unwrap().child("Submit").is_err());
}

#[test]
fn an_empty_segment_is_rejected() {
    assert_eq!(reason("a..b"), "empty segment");
    assert_eq!(reason(".a"), "empty segment");
    assert_eq!(reason("a."), "empty segment at the end");
    assert_eq!(reason("a[k]."), "empty segment at the end");
}

#[test]
fn a_key_without_its_closing_bracket_is_rejected() {
    assert_eq!(reason("a[b"), "key without a closing ]");
    assert_eq!(reason("a[b\\]"), "key without a closing ]");
    assert_eq!(reason("a[b\\"), "key without a closing ]");
}

#[test]
fn other_malformed_texts_are_rejected() {
    assert_eq!(reason("[a]"), "a key needs a name segment before it");
    assert!(reason("a[b]c").contains("'c'"));
    assert!(reason("a b").contains("' '"));
    assert!(reason("a-b").contains("'-'"));
    assert!(reason("a]").contains("']'"));
    assert!(reason("a[\\x]").contains("invalid escape"));
    assert!(SemId::name("").is_err());
    assert!(SemId::name("a.b").is_err());
    assert!(SemId::name("a").unwrap().child("").is_err());
}

#[test]
fn a_key_escapes_only_the_closing_bracket_and_the_backslash() {
    let k = SemId::name("a").unwrap().key("x]y\\z[w.v:u");
    assert_eq!(k.to_string(), r"a[x\]y\\z[w.v:u]");
    assert_eq!(id(r"a[x\]y\\z[w.v:u]"), k);
    let unicode = SemId::name("a").unwrap().key("ção 日本");
    let text = unicode.to_string();
    assert_eq!(id(&text), unicode);
}

#[test]
fn parent_path_drops_the_last_segment_as_text() {
    assert_eq!(id("commit.submit").parent_path(), Some(id("commit")));
    assert_eq!(
        id("changes.file[src/lib.rs]").parent_path(),
        Some(id("changes.file"))
    );
    assert_eq!(
        id("silence.segment[3].preview").parent_path(),
        Some(id("silence.segment[3]"))
    );
    assert_eq!(id(r"a[x.y\]z[w]").parent_path(), Some(id("a")));
    assert_eq!(id("a").parent_path(), None);
}

#[test]
fn the_order_is_total_and_follows_the_text() {
    let mut ids = [id("b"), id("a.b"), id("a"), id("a[z]"), id("a.a")];
    ids.sort();
    let texts: Vec<&str> = ids.iter().map(SemId::as_str).collect();
    assert_eq!(texts, ["a", "a.a", "a.b", "a[z]", "b"]);
}

#[test]
fn ids_are_cheap_shared_values() {
    fn shared<T: Send + Sync + Clone + Ord + std::hash::Hash>() {}
    shared::<SemId>();
    let a = id("commit.submit");
    let b = a.clone();
    assert_eq!(a, b);
}

#[cfg(feature = "serde")]
#[test]
fn serde_reads_the_canonical_text_and_rejects_the_rest() {
    use serde::Deserialize;
    use serde::de::IntoDeserializer;
    use serde::de::value::{Error, StrDeserializer};

    let ok: StrDeserializer<'_, Error> = "changes.file[src/lib.rs]".into_deserializer();
    assert_eq!(
        SemId::deserialize(ok).unwrap(),
        id("changes.file[src/lib.rs]")
    );
    let bad: StrDeserializer<'_, Error> = "Changes".into_deserializer();
    assert!(SemId::deserialize(bad).is_err());
}

fn name() -> impl Strategy<Value = String> {
    "[a-z0-9_]{1,8}"
}

#[derive(Clone, Debug)]
enum Step {
    Child(String),
    Key(String),
}

fn step() -> impl Strategy<Value = Step> {
    prop_oneof![
        name().prop_map(Step::Child),
        any::<String>().prop_map(Step::Key),
        "[a-z.\\[\\]\\\\:]{0,6}".prop_map(Step::Key),
    ]
}

fn sem_id() -> impl Strategy<Value = SemId> {
    (name(), prop::collection::vec(step(), 0..6)).prop_map(|(root, steps)| {
        let mut id = SemId::name(&root).unwrap();
        for s in steps {
            id = match s {
                Step::Child(n) => id.child(&n).unwrap(),
                Step::Key(k) => id.key(k),
            };
        }
        id
    })
}

proptest! {
    #[test]
    fn every_id_roundtrips_through_its_text(x in sem_id()) {
        let text = x.to_string();
        prop_assert_eq!(SemId::parse(&text), Ok(x.clone()));
        prop_assert!(!x.as_str().is_empty());
    }

    #[test]
    fn a_text_that_parses_prints_back_unchanged(text in "[a-zA-Z0-9_.\\[\\]\\\\ ]{0,16}") {
        if let Ok(x) = SemId::parse(&text) {
            prop_assert_eq!(x.to_string(), text);
        }
    }

    #[test]
    fn parse_never_panics(text in any::<String>()) {
        let _ = SemId::parse(&text);
    }

    #[test]
    fn the_order_matches_the_text_order(a in sem_id(), b in sem_id()) {
        prop_assert_eq!(a.cmp(&b), a.as_str().cmp(b.as_str()));
    }
}
