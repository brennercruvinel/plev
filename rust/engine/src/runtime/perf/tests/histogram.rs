//! The histogram of one zone's runs: the steps, the ends, the ranks.

use super::super::histogram::Histogram;

#[test]
fn no_run_is_an_empty_histogram() {
    let none = Histogram::of(Vec::new(), 8);
    assert!(none.is_empty());
    assert!(none.bins.is_empty());
    assert_eq!(none.mean_ns(), 0);
    assert!(Histogram::of(vec![1, 2], 0).is_empty());
}

#[test]
fn runs_land_in_equal_steps_and_the_longest_in_the_last() {
    let runs = Histogram::of(vec![99, 0, 10, 20, 30, 40, 50, 60, 70, 80], 10);
    assert_eq!(runs.bins, vec![1; 10]);
    assert_eq!((runs.min_ns, runs.max_ns), (0, 99));
    assert_eq!(runs.count, 10);
    assert_eq!(runs.tallest(), 1);
}

#[test]
fn the_median_and_the_p95_are_the_nearest_ranks() {
    let runs = Histogram::of((1..=100).collect(), 4);
    assert_eq!(runs.median_ns, 51);
    assert_eq!(runs.p95_ns, 95);
    assert_eq!(runs.mean_ns(), 50);
    assert_eq!(runs.bins, vec![25; 4]);
}

#[test]
fn equal_runs_all_land_in_the_first_step() {
    let runs = Histogram::of(vec![7; 5], 8);
    assert_eq!(runs.bins[0], 5);
    assert_eq!(runs.bins.iter().sum::<u32>(), 5);
    assert_eq!((runs.min_ns, runs.median_ns, runs.max_ns), (7, 7, 7));
}
