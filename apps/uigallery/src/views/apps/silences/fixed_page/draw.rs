//! Scene building for the Silences page. State and events live in the
//! parent module; everything here only reads it and pushes nodes. What
//! the page draws for a capability it also binds, beside the scene, with
//! the rect it hit-tests with: the page as `silences`, the frame box as
//! `silences.video`, each row as its `silences.cut[<id>]`, the counts and
//! the status; the sliders and the buttons bind themselves. The row and
//! the generated surface draw themselves (`generated.rs`, the `surface`
//! crate); `render` puts them where the halves say.

use engine::widgets::prelude::{Badge, ButtonVariant, Intent, Rect, rounded_rect};
use engine::drawing::compositor::{Compositor, SceneNode, TextNodeKey};
use engine::drawing::gpu::image::ImageHandle;
use engine::drawing::text::TextMeasurer;
use engine::styling::theme::Theme;
use crate::domain::silences::Decision;

use screen::{Frames, NoFrames};

use super::jobs::PageFrames;
use super::layout::{
    Layout, RowParts, compute, detail_style, label_style, mark_rect, mark_x, range_style,
    row_parts, stats, status_style, time_style, value_style,
};
use super::semantics::{clock, ms};
use super::super::{SilencesPage, panel};

/// `(x, y, w, h)`, the form `Compositor::bind` takes.
pub(super) fn bounds(r: Rect) -> [f32; 4] {
    [r.x, r.y, r.w, r.h]
}

/// The badge of a decision.
pub(crate) fn badge(decision: Decision) -> Badge {
    let (text, intent) = match decision {
        Decision::Pending => ("pending", Intent::Neutral),
        Decision::Apply => ("cut", Intent::Destructive),
        Decision::Reject => ("kept", Intent::Constructive),
    };
    Badge::tag(text).intent(intent)
}

/// The color a decision marks the band with: amber for a decision to
/// take, red for a stretch that goes, faint for one that stays.
fn mark_color(decision: Decision, theme: &Theme) -> [f32; 4] {
    match decision {
        Decision::Pending => theme.colors.warning.0,
        Decision::Apply => theme.colors.danger.0,
        Decision::Reject => theme.glass.text_placeholder.0,
    }
}

/// A frame fitted inside `box_`, centered, never scaled up.
fn fit(c: &mut Compositor, box_: Rect, handle: ImageHandle, radius: f32) {
    let (iw, ih) = (handle.width as f32, handle.height as f32);
    let scale = (box_.w / iw).min(box_.h / ih).min(1.0);
    let (w, h) = (iw * scale, ih * scale);
    c.draw_image(
        box_.x + (box_.w - w) / 2.0,
        box_.y + (box_.h - h) / 2.0,
        w,
        h,
        handle,
        radius,
    );
}

impl SilencesPage {
    /// The row, the fixed screen and, when one is open, the generated
    /// surface beside it with the frames the page holds; the surface
    /// reads the app again first when it moved.
    pub fn render(&mut self, c: &mut Compositor, content: Rect, theme: &Theme) {
        self.sync_rows();
        self.sync_surface();
        let h = self.halves(content, theme);
        self.generated.render_row(c, &h, theme);
        self.render_fixed(c, h.fixed, theme);
        let frames = self.ids.as_ref().map(|ids| PageFrames {
            video: &ids.video,
            frames: &self.frames,
        });
        let frames: &dyn Frames = match &frames {
            Some(f) => f,
            None => &NoFrames,
        };
        self.generated.host.draw_closing_with(c, theme, frames);
        if let (Some(rect), Some(id)) = (h.surface, &self.generated.id)
            && let Some(s) = self.generated.host.get_mut(id)
        {
            s.render_with(c, rect, theme, frames);
        }
        self.generated.proposing.render_open(c, &h.row, theme);
    }

    /// The fixed screen in `content`: the three cards.
    fn render_fixed(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let l = compute(self, content, theme);
        if let Some(ids) = &self.ids {
            c.bind(&ids.region, bounds(l.region));
        }
        self.render_video(c, &l, theme);
        self.render_side(c, &l, theme);
        self.render_cuts(c, &l, theme);
    }

    fn render_video(&self, c: &mut Compositor, l: &Layout, theme: &Theme) {
        panel(c, l.video_card, theme);
        if let Some(ids) = &self.ids {
            c.bind(&ids.video, bounds(l.video));
        }
        c.push(rounded_rect(
            l.video.x,
            l.video.y,
            l.video.w,
            l.video.h,
            theme.radius.md,
            theme.colors.bg_panel.0,
        ));
        let shown = self.model.shown();
        match self.frames.get(&ms(shown)) {
            Some(Ok(handle)) => fit(c, l.video, *handle, theme.radius.md),
            other => {
                let note = match other {
                    Some(Err(why)) => format!("no frame: {why}"),
                    _ => "no frame decoded yet: detect, or preview a cut, runs ffmpeg".to_owned(),
                };
                let style = detail_style(theme);
                let w = l.video.w - theme.spacing.lg * 2.0;
                let (_, h) = TextMeasurer::measure_styled(&note, &style, Some(w));
                c.push(SceneNode::Text {
                    key: TextNodeKey::from_style(&note, &style, Some(w)),
                    x: l.video.x + theme.spacing.lg,
                    y: l.video.y + (l.video.h - h) / 2.0,
                    color: theme.glass.text_faint.0,
                });
            }
        }
        // the clock over the frame, bottom left
        let time = format!("{} / {}", clock(shown), clock(self.model.duration()));
        let style = time_style(theme);
        let (tw, th) = TextMeasurer::measure_styled(&time, &style, None);
        let (px, py) = (theme.spacing.sm, theme.spacing.xs);
        let pill = Rect::new(
            l.video.x + theme.spacing.sm,
            l.video.bottom() - theme.spacing.sm - th - py * 2.0,
            tw + px * 2.0,
            th + py * 2.0,
        );
        c.push(rounded_rect(
            pill.x,
            pill.y,
            pill.w,
            pill.h,
            pill.h / 2.0,
            theme.glass.popover.0,
        ));
        c.push(SceneNode::Text {
            key: TextNodeKey::from_style(&time, &style, None),
            x: pill.x + px,
            y: pill.y + py,
            color: theme.colors.text.0,
        });

        // the band: every cut marked by its decision, the instant shown
        let band = l.timeline;
        c.push(rounded_rect(
            band.x,
            band.y,
            band.w,
            band.h,
            theme.radius.sm,
            theme.glass.field.0,
        ));
        let duration = self.model.duration();
        for cut in self.model.cuts() {
            let mark = mark_rect(band, cut.segment(), duration);
            let mut color = mark_color(cut.decision(), theme);
            if self.hover_mark != Some(cut.id()) {
                color[3] *= 0.75;
            }
            c.push(rounded_rect(
                mark.x,
                mark.y + theme.spacing.xs,
                mark.w,
                mark.h - theme.spacing.xs * 2.0,
                theme.radius.sm / 2.0,
                color,
            ));
        }
        let head = theme.control.focus_ring_width;
        c.push(SceneNode::Rect {
            x: mark_x(band, shown, duration) - head / 2.0,
            y: band.y,
            w: head,
            h: band.h,
            color: theme.colors.accent.0,
        });
        let (start, end) = (clock(0.0), clock(duration));
        let (ew, _) = TextMeasurer::measure_styled(&end, &style, None);
        for (text, x) in [(start.as_str(), band.x), (end.as_str(), band.right() - ew)] {
            c.push(SceneNode::Text {
                key: TextNodeKey::from_style(text, &style, None),
                x,
                y: l.times_y,
                color: theme.glass.text_faint.0,
            });
        }
    }

    fn render_side(&self, c: &mut Compositor, l: &Layout, theme: &Theme) {
        panel(c, l.side_card, theme);
        let label = label_style(theme);
        c.push(SceneNode::Text {
            key: TextNodeKey::from_style("detection", &label, None),
            x: l.side_title.0,
            y: l.side_title.1,
            color: theme.glass.text_placeholder.0,
        });
        let value = value_style(theme);
        let rows = [
            (
                "threshold",
                format!("{:.0} dB", self.threshold.value()),
                &l.threshold,
                &self.threshold,
            ),
            (
                "minimum silence",
                format!("{:.1} s", self.min_duration.value()),
                &l.min_duration,
                &self.min_duration,
            ),
        ];
        for (name, text, row, slider) in rows {
            c.push(SceneNode::Text {
                key: TextNodeKey::from_style(name, &label, None),
                x: row.label.0,
                y: row.label.1,
                color: theme.colors.text_mid.0,
            });
            let (vw, _) = TextMeasurer::measure_styled(&text, &value, None);
            c.push(SceneNode::Text {
                key: TextNodeKey::from_style(&text, &value, None),
                x: row.value_right - vw,
                y: row.label.1,
                color: theme.colors.text.0,
            });
            slider.render(c, row.slider, theme);
        }
        self.detect.render(c, l.detect, theme);
        let (message, _, failed) = self.message();
        let style = status_style(theme);
        if let Some(ids) = &self.ids {
            c.bind(&ids.status, bounds(l.status));
        }
        c.push(SceneNode::Text {
            key: TextNodeKey::from_style(&message, &style, Some(l.status.w)),
            x: l.status.x,
            y: l.status.y,
            color: if failed {
                theme.colors.danger.0
            } else {
                theme.glass.text_faint.0
            },
        });
    }

    fn render_cuts(&self, c: &mut Compositor, l: &Layout, theme: &Theme) {
        panel(c, l.cuts_card, theme);
        if let Some(ids) = &self.ids {
            c.bind(&ids.counts, bounds(l.counts));
        }
        for (stat, at) in stats(self).iter().zip(&l.stats) {
            stat.render(c, *at, theme);
        }
        self.apply_all.render(c, l.apply_all, theme);
        if l.rows.is_empty() {
            let style = detail_style(theme);
            c.push(SceneNode::Text {
                key: TextNodeKey::from_style(
                    "no silences at these settings: lower the threshold or the minimum and detect again",
                    &style,
                    Some(l.cuts_card.w - theme.spacing.lg * 2.0),
                ),
                x: l.empty.0,
                y: l.empty.1,
                color: theme.glass.text_faint.0,
            });
            return;
        }
        let shown = self.model.cut_at(self.model.shown()).map(|c| c.id());
        for ((cut, row), at) in self.model.cuts().iter().zip(&self.rows).zip(&l.rows) {
            let id = cut.id();
            if let Some(ids) = &self.ids {
                c.bind(&ids.cut(id), bounds(*at));
            }
            if self.hover_cut == Some(id) {
                c.push(rounded_rect(
                    at.x,
                    at.y,
                    at.w,
                    at.h,
                    theme.radius.sm,
                    theme.glass.surface.0,
                ));
            }
            if shown == Some(id) {
                c.push(SceneNode::Rect {
                    x: at.x - theme.spacing.sm,
                    y: at.y + theme.spacing.sm,
                    w: theme.control.focus_ring_width,
                    h: at.h - theme.spacing.sm * 2.0,
                    color: theme.colors.accent.0,
                });
            }
            let tag = badge(cut.decision());
            let (bw, _) = tag.preferred_size(theme);
            let parts: RowParts = row_parts(*at, theme, &row.apply, &row.reject, bw, l.stacked);
            self.render_thumb(c, parts.thumb, cut.segment().middle(), theme);
            let seg = cut.segment();
            let range = format!("{} to {}", clock(seg.start), clock(seg.end));
            let rstyle = range_style(theme);
            c.push(SceneNode::Text {
                key: TextNodeKey::from_style(&range, &rstyle, None),
                x: parts.text.0,
                y: parts.text.1,
                color: theme.colors.text.0,
            });
            let detail = format!("{:.1} s of silence", seg.duration());
            let dstyle = detail_style(theme);
            c.push(SceneNode::Text {
                key: TextNodeKey::from_style(&detail, &dstyle, None),
                x: parts.text.0,
                y: parts.text.1 + rstyle.line_height + theme.spacing.xs,
                color: theme.glass.text_faint.0,
            });
            tag.render(c, parts.badge, theme);
            row.apply.render(c, parts.apply, theme);
            row.reject.render(c, parts.reject, theme);
        }
    }

    /// The frame of a cut, or a dark tile with its instant.
    fn render_thumb(&self, c: &mut Compositor, at: Rect, middle: f64, theme: &Theme) {
        c.push(rounded_rect(
            at.x,
            at.y,
            at.w,
            at.h,
            theme.radius.sm,
            theme.colors.bg_panel.0,
        ));
        if let Some(Ok(handle)) = self.frames.get(&ms(middle)) {
            fit(c, at, *handle, theme.radius.sm);
            return;
        }
        let text = clock(middle);
        let style = time_style(theme);
        let (tw, th) = TextMeasurer::measure_styled(&text, &style, None);
        c.push(SceneNode::Text {
            key: TextNodeKey::from_style(&text, &style, None),
            x: at.x + (at.w - tw) / 2.0,
            y: at.y + (at.h - th) / 2.0,
            color: theme.glass.text_faint.0,
        });
    }

    /// The variant of a row's buttons: the decision taken reads dark.
    pub(crate) fn variant(taken: bool) -> ButtonVariant {
        if taken {
            ButtonVariant::Secondary
        } else {
            ButtonVariant::Outline
        }
    }
}
