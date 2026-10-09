//! the `plev-inspect` binary over a session file written the way an app
//! writes one: every mode, text and json, and the exit codes. the file is
//! the same bytes after every run.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use crate::thesis::record;

type Fallible = Result<(), Box<dyn std::error::Error>>;

fn run(args: &[&str]) -> Result<Output, std::io::Error> {
    Command::new(env!("CARGO_BIN_EXE_plev-inspect"))
        .args(args)
        .output()
}

fn text(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// the thesis session written to a file in `dir`.
fn written(dir: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let path = dir.join("session.monster");
    record().session.write(&path)?;
    Ok(path)
}

#[test]
fn every_mode_answers_and_the_file_stays_as_it_was() -> Fallible {
    let dir = tempfile::tempdir()?;
    let path = written(dir.path())?;
    let file = path.to_str().ok_or("a utf-8 path")?;
    let bytes = std::fs::read(&path)?;

    let summary = run(&[file])?;
    assert_eq!(summary.status.code(), Some(0));
    assert!(text(&summary).contains("refusals  1"), "{}", text(&summary));

    let why = run(&[file, "--why", "2"])?;
    assert_eq!(why.status.code(), Some(0));
    for says in [
        "request 2 (origin 0): submit on commit.submit by agent:reviewer",
        "by human, request 1: unstage on changes",
        "what changed  changes:staged: removed B",
    ] {
        assert!(text(&why).contains(says), "{says:?} in\n{}", text(&why));
    }

    let done = run(&[file, "--why", "3"])?;
    assert!(text(&done).contains("result        completed at +0.500s: commit = "));

    let control = run(&[
        file,
        "--control",
        "commit.submit",
        "--at",
        "300",
        "--timeline",
    ])?;
    assert_eq!(control.status.code(), Some(0));
    let out = text(&control);
    assert!(out.contains("kinds     observed 2"), "{out}");
    assert!(out.contains("control commit.submit at +0.300s"), "{out}");

    let json = run(&[file, "--why", "2", "--control", "changes", "--json"])?;
    assert_eq!(json.status.code(), Some(0));
    let value: serde_json::Value = serde_json::from_slice(&json.stdout)?;
    assert_eq!(value["why"]["id"], "2");
    assert_eq!(value["why"]["ending"]["kind"], "rejected");
    assert_eq!(
        value["why"]["between"]["reads"][0]["moves"][0]["by"]["cause"]["actor"],
        "human"
    );
    assert_eq!(
        value["control"]["receipts"].as_array().map(Vec::len),
        Some(3)
    );
    assert!(value.get("summary").is_none(), "only what was asked");

    assert_eq!(std::fs::read(&path)?, bytes, "reading never writes");
    Ok(())
}

#[test]
fn the_exit_code_says_how_it_ended() -> Fallible {
    let dir = tempfile::tempdir()?;
    let path = written(dir.path())?;
    let file = path.to_str().ok_or("a utf-8 path")?;
    assert_eq!(run(&[file, "--why", "9"])?.status.code(), Some(1));
    assert_eq!(
        run(&[file, "--why", "2", "--origin", "4"])?.status.code(),
        Some(1)
    );
    assert_eq!(run(&[file, "--why", "x"])?.status.code(), Some(2));
    assert_eq!(run(&[file, "--at", "3"])?.status.code(), Some(2));
    assert_eq!(run(&["--timeline"])?.status.code(), Some(2));

    let missing = dir.path().join("none.monster");
    let missing = missing.to_str().ok_or("a utf-8 path")?;
    assert_eq!(run(&[missing])?.status.code(), Some(3));
    let foreign = dir.path().join("foreign.monster");
    std::fs::write(&foreign, b"not a session at all")?;
    let out = run(&[foreign.to_str().ok_or("a utf-8 path")?])?;
    assert_eq!(out.status.code(), Some(3));
    assert!(String::from_utf8_lossy(&out.stderr).contains("not a session"));
    let bytes = std::fs::read(&path)?;
    let cut = dir.path().join("cut.monster");
    std::fs::write(&cut, bytes.get(..bytes.len() / 2).ok_or("half")?)?;
    assert_eq!(
        run(&[cut.to_str().ok_or("a utf-8 path")?])?.status.code(),
        Some(3)
    );

    let help = run(&[])?;
    assert_eq!(help.status.code(), Some(0));
    assert!(text(&help).contains("usage:"));
    Ok(())
}
