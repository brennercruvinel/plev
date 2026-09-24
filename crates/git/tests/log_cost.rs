//! What `GitRepo::log(100)` costs on this repository (plev), opened fresh
//! (cold: a new `GitRepo`, nothing cached) and called again on the same
//! `GitRepo` (warm: what every refresh after the first pays). A
//! measurement, not a check, so it is ignored by default:
//!
//! `cargo test -p git --test log_cost -- --ignored --nocapture`

use std::path::Path;
use std::time::{Duration, Instant};

use git::GitRepo;

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
    let mut commits = 0;
    for _ in 0..RUNS {
        let repo = GitRepo::open(here).expect("this repository opens");
        let at = Instant::now();
        let log = repo.log(LIMIT).expect("the log reads");
        cold.push(at.elapsed());
        commits = log.len();

        let at = Instant::now();
        let again = repo.log(LIMIT).expect("the log reads again");
        warm.push(at.elapsed());
        assert_eq!(again, log, "the same log twice");
    }
    println!("log({LIMIT}) over {commits} commits, {RUNS} runs each");
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
