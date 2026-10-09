//! Pure geometry for the Silences page: no GPU, no window. Two cards on
//! top (the video with its timeline; the detection controls), the cuts
//! card under them; stacked when the content is narrower than two cards.
//! Every text is measured with the style it is drawn with; constants
//! exist only as ratios and the theme's tokens.

use engine::widgets::prelude::{Badge, Button, ButtonSize, Rect, Stat, StatSize};
use engine::drawing::text::{TextMeasurer, TextStyle};
use engine::styling::theme::{ControlSize, Theme};
use crate::domain::silences::Segment;

use super::super::SilencesPage;

/// The frame's aspect, width over height: the test video's, and the
/// common one.
const ASPECT: f32 = 16.0 / 9.0;

/// One frame of geometry.
pub(crate) struct Layout {
    /// The whole page, the region's binding.
    pub region: Rect,
    pub video_card: Rect,
    /// The frame box.
    pub video: Rect,
    /// The timeline band under it.
    pub timeline: Rect,
    /// The y of the timecodes under the band.
    pub times_y: f32,
    pub side_card: Rect,
    /// The origin of the side card's title.
    pub side_title: (f32, f32),
    pub threshold: SliderRow,
    pub min_duration: SliderRow,
    pub detect: Rect,
    /// The status text's box (wrapped inside its width).
    pub status: Rect,
    pub cuts_card: Rect,
    /// One rect per stat of the counts, in `stats` order.
    pub stats: Vec<Rect>,
    /// The counts' binding: around the stats.
    pub counts: Rect,
    pub apply_all: Rect,
    /// One rect per cut, in the model's order.
    pub rows: Vec<Rect>,
    /// The rows hold their controls under the texts (narrow).
    pub stacked: bool,
    /// The origin of the line shown when there are no cuts.
    pub empty: (f32, f32),
    /// The whole height, for the page scroll.
    pub height: f32,
}

/// A labelled slider: the label's origin, the right edge the value is
/// aligned to, and the slider's bounds.
pub(crate) struct SliderRow {
    pub label: (f32, f32),
    pub value_right: f32,
    pub slider: Rect,
}

/// The parts of one cut's row.
pub(crate) struct RowParts {
    pub thumb: Rect,
    /// The origin of the range text; the detail line goes under it.
    pub text: (f32, f32),
    pub badge: Rect,
    pub apply: Rect,
    pub reject: Rect,
}

pub(super) fn label_style(theme: &Theme) -> TextStyle {
    theme.typography.caption_sm()
}

pub(super) fn value_style(theme: &Theme) -> TextStyle {
    theme.typography.caption_r()
}

pub(super) fn range_style(theme: &Theme) -> TextStyle {
    theme.typography.base_m()
}

pub(super) fn detail_style(theme: &Theme) -> TextStyle {
    theme.typography.caption_r()
}

pub(super) fn status_style(theme: &Theme) -> TextStyle {
    theme.typography.caption_r()
}

pub(super) fn time_style(theme: &Theme) -> TextStyle {
    theme.typography.small_r()
}

/// The band's height: a control's, so a mark is a target.
pub(super) fn band_h(theme: &Theme) -> f32 {
    theme.control.height(ControlSize::Xs)
}

/// A cut's thumbnail: as tall as the tallest control, 16 by 9.
pub(super) fn thumb_size(theme: &Theme) -> (f32, f32) {
    let h = theme.control.height(ControlSize::Xl);
    ((h * ASPECT).round(), h)
}

/// The width the badge and the two buttons of a row take, with their
/// gaps (the widest badge, "pending").
fn controls_w(theme: &Theme) -> f32 {
    let badge = Badge::tag("pending").preferred_size(theme).0;
    let apply = Button::new("apply")
        .size(ButtonSize::Sm)
        .preferred_size(theme)
        .0;
    let reject = Button::new("reject")
        .size(ButtonSize::Sm)
        .preferred_size(theme)
        .0;
    badge + theme.spacing.md + apply + theme.spacing.sm + reject
}

/// True when a row of `cuts_w` cannot hold the texts beside the thumbnail
/// and the controls in one line: the controls then go under the texts.
pub(super) fn stacked_rows(cuts_w: f32, theme: &Theme) -> bool {
    let (tw, _) = thumb_size(theme);
    let one_line =
        tw + theme.spacing.md + theme.size.field_min_w + theme.spacing.md + controls_w(theme);
    cuts_w < one_line
}

/// A row's height: the thumbnail with its gaps, plus a line of controls
/// when stacked.
pub(super) fn row_h(theme: &Theme, stacked: bool) -> f32 {
    let th = thumb_size(theme).1 + theme.spacing.sm * 2.0;
    if stacked {
        th + ButtonSize::Sm.height(theme) + theme.spacing.sm
    } else {
        th
    }
}

/// The counts as four readouts.
pub(super) fn stats(page: &SilencesPage) -> [Stat; 4] {
    let c = page.model.counts();
    let word = if c.total == 1 { "silence" } else { "silences" };
    [
        Stat::new(c.total.to_string(), word).size(StatSize::Sm),
        Stat::new(c.applied.to_string(), "applied").size(StatSize::Sm),
        Stat::new(c.rejected.to_string(), "rejected").size(StatSize::Sm),
        Stat::new(format!("{:.1}", c.cut_secs), "s cut").size(StatSize::Sm),
    ]
}

pub(crate) fn compute(page: &SilencesPage, content: Rect, theme: &Theme) -> Layout {
    let pad = theme.spacing.lg;
    let gap = theme.spacing.xl;
    let md = theme.spacing.md;
    let wide = content.w >= theme.size.card_min_w * 2.0 + gap;
    let side_w = if wide {
        theme
            .size
            .card_w
            .min(content.w - gap - theme.size.card_min_w)
            .max(theme.size.card_min_w)
    } else {
        content.w
    };
    let video_w = if wide {
        content.w - gap - side_w
    } else {
        content.w
    };

    // the video card: the frame box, the band, the timecodes
    let video = Rect::new(
        content.x + pad,
        content.y + pad,
        video_w - pad * 2.0,
        ((video_w - pad * 2.0) / ASPECT).round(),
    );
    let timeline = Rect::new(video.x, video.bottom() + md, video.w, band_h(theme));
    let times_y = timeline.bottom() + theme.spacing.xs;
    let video_h = times_y + time_style(theme).line_height + pad - content.y;

    // the side card: title, two labelled sliders, the detect button, the
    // status
    let side_x = if wide {
        content.x + video_w + gap
    } else {
        content.x
    };
    let side_y = if wide {
        content.y
    } else {
        content.y + video_h + gap
    };
    let inner_x = side_x + pad;
    let inner_w = side_w - pad * 2.0;
    let mut y = side_y + pad;
    let side_title = (inner_x, y);
    y += label_style(theme).line_height + md;
    let slider_row = |y: &mut f32| {
        let label = (inner_x, *y);
        *y += label_style(theme).line_height + theme.spacing.xs;
        let slider = Rect::new(inner_x, *y, inner_w, theme.control.height(ControlSize::Xs));
        *y += slider.h + md;
        SliderRow {
            label,
            value_right: inner_x + inner_w,
            slider,
        }
    };
    let threshold = slider_row(&mut y);
    let min_duration = slider_row(&mut y);
    let detect = Rect::new(inner_x, y, inner_w, page.detect.preferred_size(theme).1);
    y = detect.bottom() + md;
    let (message, _, _) = page.message();
    let (_, status_h) = TextMeasurer::measure_styled(&message, &status_style(theme), Some(inner_w));
    let status = Rect::new(inner_x, y, inner_w, status_h);
    let side_h = status.bottom() + pad - side_y;

    let top_h = if wide { video_h.max(side_h) } else { video_h };
    let video_card = Rect::new(content.x, content.y, video_w, top_h);
    let side_card = Rect::new(side_x, side_y, side_w, if wide { top_h } else { side_h });

    // the cuts card: the counts and apply all in the head, one row per cut
    let cuts_y = side_card.bottom() + gap;
    let cuts_x = content.x + pad;
    let cuts_w = content.w - pad * 2.0;
    let (apply_w, apply_h) = page.apply_all.preferred_size(theme);
    let head_y = cuts_y + pad;
    let stats_w = if cuts_w >= apply_w * 3.0 {
        cuts_w - apply_w - gap
    } else {
        cuts_w
    };
    let sizes = stats(page).map(|s| s.preferred_size(theme));
    let (stats, stats_h) = flow(Rect::new(cuts_x, head_y, stats_w, 0.0), gap, &sizes);
    let counts = Rect::new(cuts_x, head_y, stats_w, stats_h);
    let apply_all = if stats_w < cuts_w {
        Rect::new(cuts_x + cuts_w - apply_w, head_y, apply_w, apply_h)
    } else {
        Rect::new(cuts_x, head_y + stats_h + md, apply_w, apply_h)
    };
    let mut y = counts.bottom().max(apply_all.bottom()) + md;
    let stacked = stacked_rows(cuts_w, theme);
    let rows: Vec<Rect> = (0..page.model.cuts().len())
        .map(|_| {
            let r = Rect::new(cuts_x, y, cuts_w, row_h(theme, stacked));
            y += r.h;
            r
        })
        .collect();
    let empty = (cuts_x, y);
    if rows.is_empty() {
        y += detail_style(theme).line_height;
    }
    let cuts_card = Rect::new(content.x, cuts_y, content.w, y + pad - cuts_y);
    let height = cuts_card.bottom() - content.y;
    Layout {
        region: Rect::new(content.x, content.y, content.w, height),
        video_card,
        video,
        timeline,
        times_y,
        side_card,
        side_title,
        threshold,
        min_duration,
        detect,
        status,
        cuts_card,
        stats,
        counts,
        apply_all,
        rows,
        stacked,
        empty,
        height,
    }
}

/// Left to right from `at`, wrapping at its right edge: the rects and the
/// height used.
fn flow(at: Rect, gap: f32, sizes: &[(f32, f32)]) -> (Vec<Rect>, f32) {
    let mut rects = Vec::with_capacity(sizes.len());
    let (mut x, mut y, mut line_h) = (at.x, at.y, 0.0f32);
    for &(w, h) in sizes {
        if x > at.x && x + w > at.right() {
            x = at.x;
            y += line_h + gap / 2.0;
            line_h = 0.0;
        }
        rects.push(Rect::new(x, y, w, h));
        x += w + gap;
        line_h = line_h.max(h);
    }
    (rects, y + line_h - at.y)
}

/// The parts of a cut's row: the thumbnail at the left, the texts beside
/// it, the decision badge and the two buttons at the right; stacked, the
/// badge and the buttons go on a line under the thumbnail.
pub(crate) fn row_parts(
    row: Rect,
    theme: &Theme,
    apply: &Button,
    reject: &Button,
    badge_w: f32,
    stacked: bool,
) -> RowParts {
    let (tw, th) = thumb_size(theme);
    let (aw, ah) = apply.preferred_size(theme);
    let (rw, rh) = reject.preferred_size(theme);
    let badge_h = theme.typography.caption_sm().line_height + theme.spacing.xs * 1.5;
    // the band the controls are centered in: the row, or its second line
    let (thumb, controls) = if stacked {
        let thumb = Rect::new(row.x, row.y + theme.spacing.sm, tw, th);
        let y = thumb.bottom() + theme.spacing.sm;
        (thumb, Rect::new(row.x, y, row.w, row.bottom() - y))
    } else {
        (Rect::new(row.x, row.y + (row.h - th) / 2.0, tw, th), row)
    };
    let reject = Rect::new(
        controls.right() - rw,
        controls.y + (controls.h - rh) / 2.0,
        rw,
        rh,
    );
    let apply = Rect::new(
        reject.x - theme.spacing.sm - aw,
        controls.y + (controls.h - ah) / 2.0,
        aw,
        ah,
    );
    let badge = Rect::new(
        apply.x - theme.spacing.md - badge_w,
        controls.y + (controls.h - badge_h) / 2.0,
        badge_w,
        badge_h,
    );
    let two = range_style(theme).line_height + theme.spacing.xs + detail_style(theme).line_height;
    let text = (
        thumb.right() + theme.spacing.md,
        thumb.y + (thumb.h - two) / 2.0,
    );
    RowParts {
        thumb,
        text,
        badge,
        apply,
        reject,
    }
}

/// Where `seg` sits on the band of a video `duration` long.
pub(crate) fn mark_rect(band: Rect, seg: Segment, duration: f64) -> Rect {
    if duration <= 0.0 {
        return Rect::new(band.x, band.y, 0.0, band.h);
    }
    let x = band.x + (seg.start / duration) as f32 * band.w;
    let w = (seg.duration() / duration) as f32 * band.w;
    Rect::new(x, band.y, w.max(1.0), band.h)
}

/// The x of the instant `t` on the band.
pub(super) fn mark_x(band: Rect, t: f64, duration: f64) -> f32 {
    if duration <= 0.0 {
        return band.x;
    }
    band.x + (t / duration).clamp(0.0, 1.0) as f32 * band.w
}
