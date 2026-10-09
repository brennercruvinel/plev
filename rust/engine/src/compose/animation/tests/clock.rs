//! The frame clock: `dt` is clamped for motion, `since_last` is the real
//! time for deadlines.

use std::thread::sleep;
use std::time::Duration;

use super::FrameClock;

#[test]
fn since_last_is_the_real_gap_and_dt_stays_clamped() {
    let mut clock = FrameClock::new();
    let first = clock.tick();
    assert_eq!(first.since_last, 0.0, "nothing before the first tick");
    sleep(Duration::from_millis(120));
    let second = clock.tick();
    assert!(
        second.since_last >= 0.12,
        "the whole sleep, got {}",
        second.since_last
    );
    assert!(second.dt <= 0.1, "clamped, got {}", second.dt);
    assert!(second.elapsed >= second.since_last);
}

#[test]
fn a_short_gap_reads_the_same_on_both() {
    let mut clock = FrameClock::new();
    clock.tick();
    sleep(Duration::from_millis(5));
    let t = clock.tick();
    assert_eq!(t.dt, t.since_last, "under the clamp the two agree");
    assert!(t.since_last >= 0.005);
}
