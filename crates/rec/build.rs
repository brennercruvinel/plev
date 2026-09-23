//! the build identity a session file carries in its header: the git
//! commit the recorder was built from (`REC_GIT_SHA`), so a report leads
//! back to the code of that version (task-65). `git rev-parse --short
//! HEAD`, or "unknown" when git or the repository is missing; this script
//! never fails the build. the crate version comes from
//! `CARGO_PKG_VERSION`.

use std::path::Path;
use std::process::Command;

/// the trimmed output of `git <args>` when it succeeds with text.
fn git(args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8(out.stdout).ok()?;
    let text = text.trim();
    (!text.is_empty()).then(|| text.to_owned())
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let sha = git(&["rev-parse", "--short", "HEAD"]).unwrap_or_else(|| "unknown".to_owned());
    println!("cargo:rustc-env=REC_GIT_SHA={sha}");
    // rebuild when HEAD moves: the head file of this checkout (a worktree
    // has its own) and the refs it may point at.
    if let Some(dir) = git(&["rev-parse", "--git-dir"]) {
        let head = Path::new(&dir).join("HEAD");
        if head.exists() {
            println!("cargo:rerun-if-changed={}", head.display());
        }
    }
    if let Some(common) = git(&["rev-parse", "--git-common-dir"]) {
        for rel in ["refs/heads", "packed-refs"] {
            let at = Path::new(&common).join(rel);
            if at.exists() {
                println!("cargo:rerun-if-changed={}", at.display());
            }
        }
    }
}
