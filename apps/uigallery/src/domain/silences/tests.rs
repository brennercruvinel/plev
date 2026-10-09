//! tests for the silences domain model: the parser over the real stderr of
//! the probe and its edges, the decisions, the counts, "apply all", the
//! shown instant, the ids across detections. no gpu, no ffmpeg.

use super::{
    Applied, Counts, Decision, PROBE_STDERR, Segment, Settings, SilencesModel, TEST_VIDEO, Video,
    parse,
};

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-6
}

fn video(duration: f64) -> Video {
    Video {
        name: "clip.mp4".to_owned(),
        duration,
    }
}

fn seg(start: f64, end: f64) -> Segment {
    Segment { start, end }
}

// -- the parser ----------------------------------------------------------

#[test]
fn the_probe_stderr_gives_the_duration_and_the_two_silences() {
    let d = parse(PROBE_STDERR);
    assert_eq!(d.duration, Some(12.0));
    assert_eq!(d.segments.len(), 2);
    assert!(close(d.segments[0].start, 3.018594));
    assert!(close(d.segments[0].end, 5.01551));
    assert!(close(d.segments[1].start, 8.010884));
    assert!(close(d.segments[1].end, 10.0078));
    // the duration ffmpeg printed is end minus start
    assert!(close(d.segments[0].duration(), 1.996916));
    assert!(close(d.segments[1].middle(), 9.009342));
}

#[test]
fn a_stderr_without_silence_lines_has_the_duration_and_no_segments() {
    let quiet: String = PROBE_STDERR
        .lines()
        .filter(|l| !l.contains("silence_"))
        .map(|l| format!("{l}\n"))
        .collect();
    let d = parse(&quiet);
    assert_eq!(d.duration, Some(12.0));
    assert!(d.segments.is_empty());
}

#[test]
fn a_silence_to_the_end_without_silence_end_closes_at_the_duration() {
    let text = "  Duration: 00:00:06.00, start: 0.000000, bitrate: 70 kb/s\n\
                [Parsed_silencedetect_0 @ 0xadec749c0] silence_start: 4.016961\n";
    let d = parse(text);
    assert_eq!(d.segments, [seg(4.016961, 6.0)]);

    // without a duration there is no end to close at: dropped
    let d = parse("[Parsed_silencedetect_0 @ 0x1] silence_start: 4.016961\n");
    assert_eq!(d.duration, None);
    assert!(d.segments.is_empty());

    // a start past the duration is nonsense, not a segment
    let d = parse("  Duration: 00:00:03.00, start: 0\n[x] silence_start: 4\n");
    assert!(d.segments.is_empty());
}

#[test]
fn strange_text_is_ignored_and_the_keys_are_found_anywhere_in_a_line() {
    let text = "garbage\n\
                silence_end: 2.0 | silence_duration: 2.0\n\
                [Parsed_silencedetect_0 @ 0x1] [info] silence_start: abc\n\
                [Parsed_silencedetect_0 @ 0x1] [info] silence_start: 1.5\n\
                silence_start: 2.5\n\
                prefix silence_end: 3.25 | silence_duration: 0.75\n\
                  Duration: N/A, start: 0.000000, bitrate: N/A\n\
                silence_end: 9\n";
    let d = parse(text);
    // the end before any start is dropped; the second start replaces the
    // first; N/A is no duration; the trailing end has no open start
    assert_eq!(d.duration, None);
    assert_eq!(d.segments, [seg(2.5, 3.25)]);
    assert_eq!(parse(""), super::Detected::default());
    assert_eq!(parse("  Duration: 01:02:03.50, x").duration, Some(3723.5));
    assert_eq!(parse("  Duration: 12, x").duration, None);
}

// -- the seed ------------------------------------------------------------

#[test]
fn the_seed_is_the_probe_on_the_test_video_with_everything_pending() {
    let m = SilencesModel::seeded();
    assert_eq!(
        m.video(),
        Some(&Video {
            name: TEST_VIDEO.to_owned(),
            duration: 12.0
        })
    );
    assert_eq!(m.duration(), 12.0);
    assert_eq!(m.shown(), 0.0);
    assert_eq!(m.detected_with(), Settings::default());
    let ids: Vec<u64> = m.cuts().iter().map(|c| c.id()).collect();
    assert_eq!(ids, [1, 2]);
    assert!(m.cuts().iter().all(|c| c.decision() == Decision::Pending));
    assert_eq!(
        m.counts(),
        Counts {
            total: 2,
            pending: 2,
            ..Counts::default()
        }
    );
    assert_eq!(m.cut_at(4.0).map(|c| c.id()), Some(1));
    assert_eq!(m.cut_at(9.0).map(|c| c.id()), Some(2));
    assert_eq!(m.cut_at(6.0), None);
    assert_eq!(SilencesModel::new().cuts().len(), 0);
    assert_eq!(SilencesModel::new().duration(), 0.0);
}

// -- decisions and counts ------------------------------------------------

#[test]
fn deciding_a_cut_moves_the_counts_and_the_cut_seconds() {
    let mut m = SilencesModel::seeded();
    assert_eq!(m.decide(1, Decision::Apply), Some(true));
    assert_eq!(m.decide(1, Decision::Apply), Some(false), "already applied");
    assert_eq!(m.cut(1).map(|c| c.decision()), Some(Decision::Apply));
    let c = m.counts();
    assert_eq!((c.total, c.applied, c.rejected, c.pending), (2, 1, 0, 1));
    assert!(close(c.cut_secs, 1.996916));

    assert_eq!(m.decide(2, Decision::Reject), Some(true));
    let c = m.counts();
    assert_eq!((c.applied, c.rejected, c.pending), (1, 1, 0));

    // a mind changed: the rejected one is applied after all
    assert_eq!(m.decide(2, Decision::Apply), Some(true));
    let c = m.counts();
    assert_eq!((c.applied, c.rejected, c.pending), (2, 0, 0));
    assert!(close(c.cut_secs, 2.0 * 1.996916));

    // back to pending is a decision too
    assert_eq!(m.decide(2, Decision::Pending), Some(true));
    assert_eq!(m.counts().pending, 1);
}

#[test]
fn deciding_an_unknown_cut_is_none_and_changes_nothing() {
    let mut m = SilencesModel::seeded();
    assert_eq!(m.decide(999, Decision::Apply), None);
    assert_eq!(m.decide(0, Decision::Reject), None);
    assert_eq!(m.counts().pending, 2);
}

#[test]
fn apply_all_applies_the_pending_cuts_and_leaves_the_rejected_ones() {
    let mut m = SilencesModel::seeded();
    m.decide(2, Decision::Reject);
    let applied = m.apply_all();
    assert_eq!(
        applied,
        Applied {
            newly: 1,
            counts: Counts {
                total: 2,
                applied: 1,
                rejected: 1,
                pending: 0,
                cut_secs: applied.counts.cut_secs,
            }
        }
    );
    assert!(close(applied.counts.cut_secs, 1.996916));
    assert_eq!(m.cut(2).map(|c| c.decision()), Some(Decision::Reject));

    // nothing pending: nothing newly applied, the counts as they are
    let again = m.apply_all();
    assert_eq!(again.newly, 0);
    assert_eq!(again.counts, m.counts());

    // on an empty model too
    let mut empty = SilencesModel::new();
    assert_eq!(empty.apply_all().newly, 0);
}

// -- the shown instant ---------------------------------------------------

#[test]
fn show_keeps_the_instant_inside_the_video() {
    let mut m = SilencesModel::seeded();
    assert!(m.show(4.0));
    assert_eq!(m.shown(), 4.0);
    assert!(!m.show(4.0), "the same instant does not move");
    assert!(m.show(99.0));
    assert_eq!(m.shown(), 12.0, "clamped to the end");
    assert!(m.show(-3.0));
    assert_eq!(m.shown(), 0.0);
    assert!(!m.show(f64::NAN), "not a number lands at the start");
    let mut empty = SilencesModel::new();
    assert!(!empty.show(5.0), "no video: the start is the only instant");
}

// -- a new detection -----------------------------------------------------

#[test]
fn a_new_detection_replaces_the_cuts_with_fresh_ids() {
    let mut m = SilencesModel::seeded();
    m.decide(1, Decision::Apply);
    m.show(9.0);
    let with = Settings {
        threshold_db: -40.0,
        min_duration: 1.0,
    };
    m.set_detected(
        video(8.0),
        with,
        &[seg(1.0, 2.0), seg(3.0, 3.0), seg(5.0, 6.5)],
    );
    let ids: Vec<u64> = m.cuts().iter().map(|c| c.id()).collect();
    assert_eq!(ids, [3, 4], "the seed's ids are never reused");
    assert!(m.cut(1).is_none());
    assert!(m.cuts().iter().all(|c| c.decision() == Decision::Pending));
    assert_eq!(m.detected_with(), with);
    assert_eq!(m.duration(), 8.0);
    assert_eq!(
        m.shown(),
        8.0,
        "the shown instant is kept inside the new video"
    );
    let c = m.counts();
    assert_eq!((c.total, c.pending), (2, 2));
}

#[test]
fn settings_are_clamped_to_what_a_screen_offers() {
    let s = Settings {
        threshold_db: 5.0,
        min_duration: 0.0,
    }
    .clamped();
    assert_eq!(s.threshold_db, 0.0);
    assert_eq!(s.min_duration, 0.1);
    let s = Settings {
        threshold_db: -90.0,
        min_duration: 9.0,
    }
    .clamped();
    assert_eq!(s.threshold_db, -60.0);
    assert_eq!(s.min_duration, 3.0);
    let mut m = SilencesModel::new();
    m.set_detected(video(1.0), s, &[]);
    assert_eq!(m.detected_with(), s);
    assert_eq!(Decision::Apply.name(), "apply");
    assert_eq!(Decision::default(), Decision::Pending);
}
