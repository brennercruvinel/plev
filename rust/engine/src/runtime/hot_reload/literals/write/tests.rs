//! Writing a literal back, on files of a temporary directory: each test
//! its own directory and its own texts, since the baseline and the
//! overrides are the process's.

use std::path::{Path, PathBuf};

use sem::Actor;

use super::super::{Change, sites_of, snapshot, take_baseline, text};
use super::*;

/// A fresh directory holding `page.rs` with `src`, its baseline taken.
fn page(name: &str, src: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("plev-literal-write-{name}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let file = dir.join("page.rs");
    fs::write(&file, src).unwrap();
    take_baseline(std::slice::from_ref(&dir));
    file
}

/// The one site of `text`.
fn site(text: &str) -> Site {
    let sites = sites_of(text);
    assert_eq!(sites.len(), 1, "{text:?}: {sites:?}");
    sites[0].clone()
}

/// What the directory of `file` holds besides it: nothing, after a write.
fn beside(file: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(file.parent().unwrap())
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n != "page.rs")
        .collect();
    names.sort();
    names
}

const PERSON: Actor = Actor::Human;

#[test]
fn the_literal_s_bytes_are_replaced_and_the_hot_reload_applies_it() {
    let old = "const A: &str = \"write-probe-title\";\n\
               const B: &str = \"write-probe-other\"; // \"write-probe-title\"\n";
    let file = page("span", old);
    let at = site("write-probe-title");
    assert_eq!((at.line, at.column), (1, 17));

    let applied = write(
        &at,
        "write-probe-title",
        "write-probe-title, edited",
        &PERSON,
    );
    assert_eq!(applied, Ok(Change::Literals(1)));
    assert_eq!(
        fs::read_to_string(&file).unwrap(),
        old.replacen("\"write-probe-title\"", "\"write-probe-title, edited\"", 1),
        "that token and nothing else, the comment with the same text kept"
    );
    assert!(beside(&file).is_empty(), "no file left beside it");
    // applied as a save is: the window draws the new text, a click finds it
    assert_eq!(
        text("write-probe-title").as_deref(),
        Some("write-probe-title, edited")
    );
    let now = site("write-probe-title, edited");
    assert_eq!((now.line, now.column), (1, 17));

    // and back, from where the click finds it now
    let back = write(
        &now,
        "write-probe-title, edited",
        "write-probe-title",
        &PERSON,
    );
    assert_eq!(back, Ok(Change::Literals(0)));
    assert_eq!(fs::read_to_string(&file).unwrap(), old);
    assert_eq!(text("write-probe-title"), None);
}

#[test]
fn a_file_whose_code_changed_since_the_baseline_is_refused_untouched() {
    let old = "const A: &str = \"write-probe-code\";\nconst N: u8 = 1;\n";
    let file = page("code", old);
    let at = site("write-probe-code");
    for changed in [
        format!("{old}fn more() {{}}\n"),
        old.replace("u8 = 1", "u8 = 2"),
    ] {
        fs::write(&file, &changed).unwrap();
        let modified = fs::metadata(&file).unwrap().modified().unwrap();
        assert_eq!(
            write(&at, "write-probe-code", "write-probe-code!", &PERSON),
            Err(Refused::Code(file.clone())),
            "{changed}"
        );
        assert_eq!(fs::read_to_string(&file).unwrap(), changed);
        assert_eq!(fs::metadata(&file).unwrap().modified().unwrap(), modified);
        assert!(beside(&file).is_empty());
    }
    let said = Refused::Code(file.clone()).to_string();
    assert!(said.ends_with("page.rs: its code changed since this build; rebuild, then edit"));
}

#[test]
fn a_literal_changed_on_disk_since_the_click_is_refused() {
    let old = "const A: &str = \"write-probe-moved\";\n";
    let file = page("moved", old);
    let at = site("write-probe-moved");
    // saved from the editor, not yet seen by the watcher
    let saved = old.replace("write-probe-moved", "write-probe-moved elsewhere");
    fs::write(&file, &saved).unwrap();
    assert!(matches!(
        write(&at, "write-probe-moved", "x", &PERSON),
        Err(Refused::Moved { .. })
    ));
    assert_eq!(fs::read_to_string(&file).unwrap(), saved);
    // a site the sources never had
    let nowhere = Site { line: 9, ..at };
    assert!(matches!(
        write(&nowhere, "write-probe-moved", "x", &PERSON),
        Err(Refused::Moved { .. })
    ));
}

#[test]
fn quotes_a_backslash_and_unicode_are_written_escaped_and_read_back() {
    let old = "const A: &str = \"write-probe-escape\";\n\
               const R: &str = r#\"write-probe-raw \"q\"\"#;\n";
    let file = page("escape", old);

    let typed = "diga \"oi\" \\ já, ✓ 日本 é\t\u{7}";
    let at = site("write-probe-escape");
    assert_eq!(
        write(&at, "write-probe-escape", typed, &PERSON),
        Ok(Change::Literals(1))
    );
    let src = fs::read_to_string(&file).unwrap();
    assert!(
        src.starts_with("const A: &str = \"diga \\\"oi\\\" \\\\ já, ✓ 日本 é\\t\\u{7}\";\n"),
        "{src}"
    );

    // a raw string stays raw, with the `#`s its new text needs
    let raw = "write-probe-raw \"q\"";
    let typed_raw = "write-probe-raw \"#q\"#";
    assert_eq!(
        write(&site(raw), raw, typed_raw, &PERSON),
        Ok(Change::Literals(2))
    );
    let src = fs::read_to_string(&file).unwrap();
    assert!(
        src.ends_with("const R: &str = r##\"write-probe-raw \"#q\"#\"##;\n"),
        "{src}"
    );

    // the file reads back as the texts that were typed, and still lexes
    let snap = snapshot(&src).unwrap();
    assert_eq!(
        snap.lits,
        vec![Lit::Str(typed.to_owned()), Lit::Str(typed_raw.to_owned())]
    );
    assert_eq!(site(typed).file, file);
}

#[test]
fn an_agent_cannot_write() {
    let old = "const A: &str = \"write-probe-agent\";\n";
    let file = page("agent", old);
    let at = site("write-probe-agent");
    let agent = Actor::Agent {
        name: "claude".into(),
    };
    assert_eq!(
        write(&at, "write-probe-agent", "written by an agent", &agent),
        Err(Refused::Agent)
    );
    assert_eq!(fs::read_to_string(&file).unwrap(), old);
    assert_eq!(text("write-probe-agent"), None);
    assert_eq!(
        Refused::Agent.to_string(),
        "an agent may not write the sources"
    );
    // the person, through a test, writes
    let test = Actor::Test {
        name: "write".into(),
    };
    assert_eq!(
        write(&at, "write-probe-agent", "write-probe-agent!", &test),
        Ok(Change::Literals(1))
    );
}

#[test]
fn a_file_the_baseline_never_read_is_refused() {
    let dir = std::env::temp_dir().join("plev-literal-write-unread");
    fs::create_dir_all(&dir).unwrap();
    let file = dir.join("unread.rs");
    let at = Site {
        file: file.clone(),
        line: 1,
        column: 17,
    };
    assert_eq!(
        write(&at, "a", "b", &PERSON),
        Err(Refused::NoBaseline(file))
    );
}
