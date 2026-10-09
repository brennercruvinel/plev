//! Editing the title in the window writes the `.rs` (task-65 phase c),
//! with no window and never on the repository: a copy of the gallery's
//! `table.rs` in a temporary git repository is the only source this
//! process reads, so the Todo title's one literal is the copy's. The
//! title in the staged frame leads to it, the function the field's Enter
//! calls writes over it as the person, `git diff` shows that one string,
//! and the next frame draws it with no rebuild. Then an agent is refused,
//! and so is a write after the file's code changed, the file untouched.
//! Only a desktop debug build has any of this (the engine's `hot_reload`),
//! so the file is empty everywhere else.
#![cfg(all(
    debug_assertions,
    any(target_os = "macos", target_os = "linux", target_os = "windows")
))]

use std::path::{Path, PathBuf};
use std::process::Command;

use engine::compositor::{Compositor, SceneNode};
use engine::headless::Viewport;
use engine::hot_reload::literals::{self, Change, Refused};
use engine::hot_reload::project_root;
use engine::hot_reload::view_source::{Origin, origin_at};
use sem::Actor;
use showcase::app::stage_page;

/// The gallery's page table, where the Todo title is written.
const TABLE: &str = "crates/showcase/src/view/pages/table.rs";

fn git(repo: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .expect("git runs");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

/// The unstaged change, as a person reads it: the lines taken out and put in.
fn diff(repo: &Path) -> (String, Vec<String>) {
    let numstat = git(repo, &["diff", "--no-ext-diff", "--numstat"]);
    let patch = git(repo, &["diff", "--no-ext-diff", "--no-color", "-U0"]);
    let changed = patch
        .lines()
        .filter(|l| {
            (l.starts_with('-') || l.starts_with('+'))
                && !l.starts_with("---")
                && !l.starts_with("+++")
        })
        .map(str::to_owned)
        .collect();
    (numstat, changed)
}

/// A git repository holding a copy of `table.rs`, staged, and the
/// literal baseline of it alone.
fn repository() -> (PathBuf, PathBuf) {
    let repo = std::env::temp_dir().join(format!("plev-edit-literal-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&repo);
    let file = repo.join(TABLE);
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::copy(project_root().join(TABLE), &file).unwrap();
    git(&repo, &["init", "-q"]);
    git(&repo, &["add", TABLE]);
    literals::take_baseline(&[repo.join("crates")]);
    (repo, file)
}

/// A point inside the first text run drawn with `text`.
fn on_text(c: &Compositor, text: &str) -> Option<(f32, f32)> {
    c.layers()
        .iter()
        .flat_map(|l| l.nodes())
        .find_map(|n| match n {
            SceneNode::Text { key, x, y, .. } if key.text == text => Some((x + 3.0, y + 5.0)),
            _ => None,
        })
}

#[test]
fn editing_the_title_writes_the_rs_and_git_diff_shows_only_that_string() {
    let (repo, file) = repository();
    let viewport = Viewport::new(1200.0, 800.0, 1.0);
    let page = stage_page(viewport, "Todo", "core", None).unwrap();
    let (x, y) = on_text(page.compositor(), "Todo").expect("the title is drawn");
    let origin = origin_at(page.compositor(), page.semantics().ok().as_ref(), x, y);
    let Origin::Literal { text, sites } = &origin else {
        panic!("{origin}");
    };
    assert_eq!((text.as_str(), sites.len()), ("Todo", 1), "{origin}");
    let site = &sites[0];
    assert_eq!(site.file, file, "the copy's literal");

    // Enter in the field over the title
    assert_eq!(
        literals::write(site, "Todo", "Todo list", &Actor::Human),
        Ok(Change::Literals(1))
    );
    let (numstat, changed) = diff(&repo);
    assert_eq!(numstat, format!("1\t1\t{TABLE}\n"));
    assert_eq!(changed.len(), 2, "{changed:?}");
    let (taken, put) = (&changed[0][1..], &changed[1][1..]);
    assert!(taken.contains("\"Todo\""), "{taken}");
    assert_eq!(put, taken.replacen("\"Todo\"", "\"Todo list\"", 1));
    println!("git diff:\n{}", changed.join("\n"));

    // the next frame draws it, the binary as it was
    let page = stage_page(viewport, "Todo", "core", None).unwrap();
    assert!(on_text(page.compositor(), "Todo list").is_some());
    let now = literals::sites_of("Todo list");
    assert_eq!(now.len(), 1);

    // an agent is refused, and nothing moves
    let before = std::fs::read(&file).unwrap();
    let agent = Actor::Agent {
        name: "claude".into(),
    };
    assert_eq!(
        literals::write(&now[0], "Todo list", "an agent's", &agent),
        Err(Refused::Agent)
    );
    assert_eq!(std::fs::read(&file).unwrap(), before);

    // the person changes code the build does not have: refused, untouched
    let mut changed_code = before;
    changed_code.extend_from_slice(b"\nfn not_built_yet() {}\n");
    std::fs::write(&file, &changed_code).unwrap();
    assert_eq!(
        literals::write(&now[0], "Todo list", "Todo again", &Actor::Human),
        Err(Refused::Code(file.clone()))
    );
    assert_eq!(std::fs::read(&file).unwrap(), changed_code);
    assert_eq!(
        std::fs::read_dir(file.parent().unwrap()).unwrap().count(),
        1
    );

    let _ = std::fs::remove_dir_all(&repo);
}
