//! Charts section: line with axes/grid/dots, bars, stacked area and donut
//! with legend, all drawn from the pure tested geometry core in
//! `showcase::model::charts` (backend before ui). The 2x2 grid is
//! content-driven and stacks to one column in narrow windows (cards.rs
//! pattern). A Tween reveal in the style of the old makepad_charts demo
//! (1.5s ease-out cubic) replays on click; while it runs, `tick()` keeps
//! requesting frames, so render on demand never freezes mid-animation.

mod draw;
#[cfg(test)]
mod tests;

use engine::animation::{Easing, Tween};
use engine::compositor::Compositor;
use engine::theme::Theme;
use engine::ui::widgets::{EventResult, Rect, WidgetEvent};

use super::{group_label, panel, text};

const GAP: f32 = 16.0;
const LABEL_H: f32 = 24.0;
/// Inner padding between a panel edge and its plot.
const PAD: f32 = 20.0;
/// Vertical space inside a panel for its title row.
const HEAD_H: f32 = 48.0;
/// Minimum readable chart panel width; the column count derives from it.
const PANEL_MIN_W: f32 = 320.0;
/// Readability clamp so panels do not degenerate on ultra-wide windows.
const PANEL_MAX_W: f32 = 640.0;
/// Hint line below the grid.
const HINT_H: f32 = 20.0;
/// Reveal timing of the old makepad_charts demo.
const REVEAL_S: f32 = 1.5;

pub struct ChartsSection {
    /// 0..1 draw progress. Constructed settled at 1.0 so an idle section
    /// re-renders identically; clicking a chart replays from 0.
    reveal: Tween<f32>,
    line_data: Vec<f32>,
    bar_data: Vec<f32>,
    area_a: Vec<f32>,
    area_b: Vec<f32>,
    donut_items: Vec<(&'static str, f32)>,
}

impl ChartsSection {
    pub fn new() -> Self {
        Self {
            reveal: Tween::new(1.0, REVEAL_S, Easing::EaseOutCubic),
            line_data: vec![
                0.3, 0.5, 0.4, 0.7, 0.6, 0.9, 0.75, 0.85, 0.65, 0.95, 0.8, 0.92,
            ],
            bar_data: vec![0.6, 0.8, 0.45, 0.9, 0.55, 0.75, 0.65, 0.85],
            area_a: vec![0.2, 0.35, 0.3, 0.5, 0.45, 0.6, 0.55, 0.7, 0.65, 0.75],
            area_b: vec![0.1, 0.2, 0.15, 0.3, 0.25, 0.35, 0.3, 0.4, 0.35, 0.45],
            donut_items: vec![
                ("Search", 35.0),
                ("Direct", 25.0),
                ("Social", 20.0),
                ("Email", 12.0),
                ("Ads", 8.0),
            ],
        }
    }

    /// Content-driven 2x2 grid: two columns when `content.w` affords two
    /// readable panels, one stacked column otherwise (cards.rs pattern).
    fn layout(&self, content: Rect) -> Vec<Rect> {
        let cols = (((content.w + GAP) / (PANEL_MIN_W + GAP)).floor() as usize).clamp(1, 2);
        let col_w = ((content.w - (cols as f32 - 1.0) * GAP) / cols as f32).min(PANEL_MAX_W);
        let panel_h = (col_w * 0.58).clamp(200.0, 300.0);
        (0..4)
            .map(|i| {
                let (row, col) = (i / cols, i % cols);
                Rect::new(
                    content.x + col as f32 * (col_w + GAP),
                    content.y + row as f32 * (LABEL_H + panel_h + GAP) + LABEL_H,
                    col_w,
                    panel_h,
                )
            })
            .collect()
    }

    /// Natural height of the laid-out grid plus the replay hint.
    pub fn content_height(&self, content: Rect) -> f32 {
        let bottom = self
            .layout(content)
            .iter()
            .map(|r| r.y + r.h)
            .fold(content.y, f32::max);
        bottom - content.y + GAP + HINT_H
    }

    /// Clicking any chart replays the reveal from zero.
    pub fn handle_event(&mut self, event: &WidgetEvent, content: Rect) -> EventResult {
        if let WidgetEvent::MouseDown { x, y } = *event
            && self.layout(content).iter().any(|r| r.contains(x, y))
        {
            self.reveal.reset(0.0);
            self.reveal.set_target(1.0);
            return EventResult::clicked();
        }
        EventResult::IGNORED
    }

    /// Advance the reveal. Returns `true` while it still needs frames.
    pub fn tick(&mut self, dt: f32) -> bool {
        self.reveal.tick(dt);
        self.reveal.is_animating()
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let r = self.reveal.get().clamp(0.0, 1.0);
        let rects = self.layout(content);
        let titles = [
            ("LINE", "Sessions per week"),
            ("BARS", "Deploys per day"),
            ("STACKED AREA", "Bandwidth by region"),
            ("DONUT", "Traffic sources"),
        ];
        for ((label, title), rect) in titles.iter().zip(&rects) {
            group_label(c, label, rect.x, rect.y - LABEL_H + 2.0, theme);
            panel(c, *rect, theme);
            text(
                c,
                title,
                14.0,
                600,
                rect.x + PAD,
                rect.y + 16.0,
                theme.colors.text.0,
            );
        }
        // Plot area inside a panel: under the title row, padded all around.
        let inner = |p: &Rect| {
            let (w, h) = ((p.w - PAD * 2.0).max(0.0), (p.h - HEAD_H - PAD).max(0.0));
            Rect::new(p.x + PAD, p.y + HEAD_H, w, h)
        };
        draw::line(c, &self.line_data, inner(&rects[0]), theme, r);
        draw::bars(c, &self.bar_data, inner(&rects[1]), theme, r);
        draw::area(c, &self.area_a, &self.area_b, inner(&rects[2]), theme, r);
        draw::donut(c, &self.donut_items, inner(&rects[3]), theme, r);

        let bottom = rects.iter().map(|p| p.y + p.h).fold(content.y, f32::max);
        text(
            c,
            "Click any chart to replay the reveal",
            12.0,
            400,
            content.x,
            bottom + GAP,
            theme.glass.text_placeholder.0,
        );
    }
}
