//! The fixed screen's width sweep (task-104g): from 280 to 1600 every
//! 8 px, in core and hoff, with the recorded run's cuts, every card in
//! the page, every part in its card, the video and the thumbnails at the
//! video's shape, and no two cards over each other.

use engine::styling::theme::Theme;
use engine::widgets::prelude::Rect;

use super::super::fixed_page::layout::{ASPECT, compute, row_parts};
use super::{frame, setup};

fn inside(inner: Rect, outer: Rect) -> bool {
    const E: f32 = 0.5;
    inner.x >= outer.x - E
        && inner.y >= outer.y - E
        && inner.right() <= outer.right() + E
        && inner.bottom() <= outer.bottom() + E
}

fn apart(a: Rect, b: Rect) -> bool {
    a.right() <= b.x + 0.5
        || b.right() <= a.x + 0.5
        || a.bottom() <= b.y + 0.5
        || b.bottom() <= a.y + 0.5
}

#[test]
fn nothing_cuts_or_overlaps_from_280_to_1600() {
    for theme in [Theme::hoff_core(), Theme::hoff()] {
        let (_todo, mut page, _) = setup(false);
        // the recorded run's cuts, mirrored into rows by a frame
        let _ = frame(&mut page, Rect::new(0.0, 0.0, 1200.0, 800.0), &theme);
        assert!(!page.rows.is_empty(), "the recorded run's cuts");
        for w in (280..=1600).step_by(8) {
            let content = Rect::new(0.0, 0.0, w as f32, 800.0);
            let l = compute(&page, content, &theme);
            let cards = [l.video_card, l.side_card, l.cuts_card];
            for card in cards {
                assert!(
                    card.x >= content.x && card.right() <= content.right() + 0.5,
                    "{w}: {card:?}"
                );
            }
            assert!(
                apart(l.video_card, l.side_card) && apart(l.side_card, l.cuts_card),
                "{w}"
            );
            assert!(apart(l.video_card, l.cuts_card), "{w}");
            assert!(
                inside(l.video, l.video_card) && inside(l.timeline, l.video_card),
                "{w}"
            );
            assert!(
                (l.video.w / l.video.h - ASPECT).abs() < 0.05,
                "{w}: the video's shape"
            );
            for r in [
                l.threshold.slider,
                l.min_duration.slider,
                l.detect,
                l.status,
            ] {
                assert!(inside(r, l.side_card), "{w}: {r:?}");
            }
            for r in l.stats.iter().chain(&l.rows).chain([&l.apply_all]) {
                assert!(inside(*r, l.cuts_card), "{w}: {r:?}");
            }
            for (row, at) in page.rows.iter().zip(&l.rows) {
                let parts = row_parts(*at, &theme, &row.apply, &row.reject, 0.0, l.stacked);
                assert!(inside(parts.thumb, *at), "{w}: thumb");
                assert!(
                    inside(parts.apply, *at) && inside(parts.reject, *at),
                    "{w}: buttons"
                );
            }
        }
    }
}
