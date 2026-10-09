//! What `GitRepo::log(100)` costs on this repository (plev), opened fresh
//! (cold: a new `GitRepo`, nothing cached) and called again on the same
//! `GitRepo` (warm: what every refresh after the first pays), and whether
//! the files of every commit here are the ones `git diff-tree` lists. Both
//! read this repository's history, which moves, so they are ignored by
//! default:
//!
//! `cargo test -p git --test log_cost -- --ignored --nocapture --test-threads=1`
//!
//! (one at a time: the check runs a `git` per commit, which would weigh on
//! the timing.)

use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

use gitdrv::GitRepo;

/// The limit the ide's stacks panel asks for.
const LIMIT: usize = 100;

/// Runs per measurement; the report is the median, with min and max.
const RUNS: usize = 21;

#[test]
#[ignore = "a measurement on this repository, run by hand"]
fn log_of_this_repository_cold_and_warm() {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));

    let mut cold = Vec::with_capacity(RUNS);
    let mut warm = Vec::with_capacity(RUNS);
    let mut log = Vec::new();
    for _ in 0..RUNS {
        let repo = GitRepo::open(here).expect("this repository opens");
        let at = Instant::now();
        log = repo.log(LIMIT).expect("the log reads");
        cold.push(at.elapsed());

        let at = Instant::now();
        let again = repo.log(LIMIT).expect("the log reads again");
        warm.push(at.elapsed());
        assert_eq!(again, log, "the same log twice");
    }
    let paths: Vec<&String> = log.iter().flat_map(|c| &c.files).collect();
    let bytes: usize = paths.iter().map(|p| p.len()).sum();
    println!("log({LIMIT}) over {} commits, {RUNS} runs each", log.len());
    println!("files: {} paths, {bytes} bytes of path", paths.len());
    println!("cold (a new GitRepo): {}", spread(&mut cold));
    println!(
        "warm (the same GitRepo, called again): {}",
        spread(&mut warm)
    );
}

/// `median (min .. max)` in milliseconds.
fn spread(samples: &mut [Duration]) -> String {
    samples.sort();
    let ms = |d: Duration| d.as_secs_f64() * 1000.0;
    format!(
        "median {:.2} ms (min {:.2} .. max {:.2})",
        ms(samples[samples.len() / 2]),
        ms(samples[0]),
        ms(samples[samples.len() - 1])
    )
}

#[test]
#[ignore = "one git process per commit of this repository, run by hand"]
fn the_files_of_every_commit_here_are_what_git_diff_tree_lists() {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo = GitRepo::open(here).expect("this repository opens");
    let log = repo.log(usize::MAX).expect("the whole log reads");
    let mut merges = 0;
    for commit in &log {
        let out = Command::new("git")
            .arg("-C")
            .arg(here)
            .args(["diff-tree", "--no-commit-id", "--name-only", "-r", "-z"])
            .args(["--no-renames", "--root", "--diff-merges=first-parent"])
            .arg(&commit.sha)
            .output()
            .expect("git runs");
        let listed: Vec<String> = String::from_utf8_lossy(&out.stdout)
            .split('\0')
            .filter(|p| !p.is_empty())
            .map(str::to_owned)
            .collect();
        assert_eq!(
            commit.files, listed,
            "{} {}",
            commit.short_sha, commit.message
        );
        let fresh = GitRepo::open(here).expect("this repository opens");
        assert_eq!(
            fresh.commit_files(&commit.sha).expect("the commit's files"),
            listed,
            "a commit's files read alone, as its receipt reads them"
        );
        let parents = Command::new("git")
            .arg("-C")
            .arg(here)
            .args(["rev-list", "--parents", "-n", "1", &commit.sha])
            .output()
            .expect("git runs");
        merges += usize::from(String::from_utf8_lossy(&parents.stdout).split(' ').count() > 2);
    }
    println!(
        "{} commits ({merges} merges): every one lists what git diff-tree lists",
        log.len()
    );
}
