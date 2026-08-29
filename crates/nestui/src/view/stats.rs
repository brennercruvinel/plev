//! STATS screen: file/section size breakdown plus the latency benchmark
//! (port of `nest benchmark`'s methodology — see `model::bench`).
//!
//! Sections render as horizontal bars proportional to byte size. The
//! benchmark runs on the worker; this screen owns the controls, the
//! progress counter and the last `BenchmarkView`.

use engine::compositor::Compositor;
use engine::text::{TextMeasurer, TextStyle};
use engine::theme::Theme;
use engine::ui::widgets::{Button, EventResult, Rect, Slider, WidgetEvent, rounded_rect};

use crate::model::bench::{BenchmarkView, LatencyStats};
use crate::model::types::OpenedDbView;

use super::{Action, fmt_bytes, group_label, panel, text, truncate_to_width};

const GAP: f32 = 16.0;
const CARD_PAD: f32 = 20.0;
const ROW_H: f32 = 26.0;
const BAR_H: f32 = 18.0;

pub struct StatsScreen {
    n_slider: Slider,
    k_slider: Slider,
    run_button: Button,
    pub running: bool,
    /// (done, total) while the benchmark runs.
    pub progress: (usize, usize),
    pub result: Option<BenchmarkView>,
    pub error: String,
}

impl StatsScreen {
    pub fn new() -> Self {
        Self {
            n_slider: Slider::new(10.0, 500.0, 50.0).step(10.0),
            k_slider: Slider::new(1.0, 100.0, 10.0).step(1.0),
            run_button: Button::new("Run benchmark").icon("play"),
            running: false,
            progress: (0, 0),
            result: None,
            error: String::new(),
        }
    }

    /// Reset per-database state (benchmark numbers are file-specific).
    pub fn reset(&mut self) {
        self.running = false;
        self.progress = (0, 0);
        self.result = None;
        self.error = String::new();
    }

    pub fn fold_result(&mut self, result: Result<BenchmarkView, String>) {
        self.running = false;
        match result {
            Ok(r) => {
                self.error = String::new();
                self.result = Some(r);
            }
            Err(e) => self.error = e,
        }
    }

    /// Controls row layout: two sliders with labels + the run button.
    fn controls(&self, content: Rect) -> (Rect, Rect, Rect) {
        let slider_w = ((content.w - CARD_PAD * 2.0 - 160.0) / 2.0).max(120.0);
        let n = Rect::new(
            content.x + CARD_PAD + 64.0,
            content.y + 12.0,
            slider_w - 64.0,
            20.0,
        );
        let k = Rect::new(n.x + n.w + 80.0, n.y, slider_w - 64.0, 20.0);
        let (bw, bh) = self.run_button.preferred_size();
        let run = Rect::new(content.x + content.w - CARD_PAD - bw, content.y, bw, bh);
        (n, k, run)
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        has_db: bool,
    ) -> (EventResult, Action) {
        self.run_button.disabled = !has_db || self.running;
        let (n, k, run) = self.controls(content);
        let mut result = self.n_slider.handle_event(event, n);
        result = result.merge(self.k_slider.handle_event(event, k));
        let r = self.run_button.handle_event(event, run);
        if r.clicked {
            self.running = true;
            self.error = String::new();
            self.progress = (0, self.n_slider.value() as usize);
            return (
                r,
                Action::RunBenchmark {
                    n_queries: self.n_slider.value() as usize,
                    k: self.k_slider.value() as i32,
                },
            );
        }
        (result.merge(r), Action::None)
    }

    pub fn render(&mut self, c: &mut Compositor, content: Rect, theme: &Theme, db: &OpenedDbView) {
        self.run_button.disabled = self.running;

        // Controls row.
        let (n, k, run) = self.controls(content);
        text(
            c,
            "queries",
            12.0,
            600,
            content.x + CARD_PAD,
            n.y + 2.0,
            theme.colors.text_dim.0,
        );
        text(
            c,
            &format!("{}", self.n_slider.value() as usize),
            12.0,
            500,
            n.x + n.w + 8.0,
            n.y + 2.0,
            theme.colors.text_mid.0,
        );
        self.n_slider.render(c, n, theme);
        text(
            c,
            "k",
            12.0,
            600,
            k.x - 16.0,
            k.y + 2.0,
            theme.colors.text_dim.0,
        );
        text(
            c,
            &format!("{}", self.k_slider.value() as i32),
            12.0,
            500,
            k.x + k.w + 8.0,
            k.y + 2.0,
            theme.colors.text_mid.0,
        );
        self.k_slider.render(c, k, theme);
        self.run_button.label = if self.running {
            "Running…".to_string()
        } else {
            "Run benchmark".to_string()
        };
        self.run_button.render(c, run, theme);

        let mut y = content.y + 44.0 + GAP;

        // Status: progress or error.
        if self.running {
            text(
                c,
                &format!(
                    "benchmark: {}/{} queries…",
                    self.progress.0, self.progress.1
                ),
                13.0,
                400,
                content.x,
                y,
                theme.colors.text_dim.0,
            );
            y += 28.0;
        } else if !self.error.is_empty() {
            let style = TextStyle::new(13.0);
            let msg = truncate_to_width(&self.error, content.w, &style);
            text(c, &msg, 13.0, 500, content.x, y, theme.colors.danger.0);
            y += 28.0;
        }

        // Two cards side by side when wide, stacked when narrow.
        let wide = content.w >= 900.0;
        let card_w = if wide {
            (content.w - GAP) / 2.0
        } else {
            content.w
        };
        let kv_rect = Rect::new(content.x, y, card_w, CARD_PAD * 2.0 + 24.0 + ROW_H * 6.0);
        self.render_kv(c, kv_rect, theme, db);

        let sections = &db.inspect.sections;
        let sec_rows = sections.len() as f32;
        let sec_h = CARD_PAD * 2.0 + 24.0 + sec_rows * (BAR_H + 8.0);
        let sec_rect = if wide {
            Rect::new(content.x + card_w + GAP, y, card_w, sec_h)
        } else {
            Rect::new(content.x, y + kv_rect.h + GAP, card_w, sec_h)
        };
        self.render_sections(c, sec_rect, theme, db);

        // Benchmark results below the cards.
        if let Some(res) = &self.result {
            let bench_y = if wide {
                y + kv_rect.h.max(sec_rect.h) + GAP
            } else {
                sec_rect.y + sec_rect.h + GAP
            };
            self.render_benchmark(
                c,
                Rect::new(content.x, bench_y, content.w, 300.0),
                theme,
                res,
            );
        }
    }

    /// File facts: dtype, metric, simd, counts.
    fn render_kv(&self, c: &mut Compositor, rect: Rect, theme: &Theme, db: &OpenedDbView) {
        panel(c, rect, theme);
        group_label(c, "FILE", rect.x + CARD_PAD, rect.y + CARD_PAD, theme);
        let i = &db.inspect;
        let rows = [
            ("dtype", i.manifest.dtype.clone()),
            ("metric", i.manifest.metric.clone()),
            ("simd backend", i.simd_backend.clone()),
            ("chunks", i.n_chunks.to_string()),
            ("embeddings", i.n_embeddings.to_string()),
            ("file size", fmt_bytes(i.file_size)),
        ];
        for (row, (key, value)) in rows.iter().enumerate() {
            let y = rect.y + CARD_PAD + 24.0 + row as f32 * ROW_H;
            text(
                c,
                key,
                13.0,
                600,
                rect.x + CARD_PAD,
                y,
                theme.colors.text_dim.0,
            );
            text(
                c,
                value,
                13.0,
                400,
                rect.x + CARD_PAD + 128.0,
                y,
                theme.colors.text_mid.0,
            );
        }
    }

    /// Horizontal bars of section byte sizes (largest = full width).
    fn render_sections(&self, c: &mut Compositor, rect: Rect, theme: &Theme, db: &OpenedDbView) {
        panel(c, rect, theme);
        group_label(
            c,
            "SECTION SIZES",
            rect.x + CARD_PAD,
            rect.y + CARD_PAD,
            theme,
        );
        let max = db
            .inspect
            .sections
            .iter()
            .map(|s| s.size)
            .max()
            .unwrap_or(1)
            .max(1) as f32;
        let name_w = 160.0;
        let size_w = 88.0;
        let bar_w = (rect.w - CARD_PAD * 2.0 - name_w - size_w - 16.0).max(40.0);
        let label_style = TextStyle::new(12.0);
        for (row, section) in db.inspect.sections.iter().enumerate() {
            let y = rect.y + CARD_PAD + 24.0 + row as f32 * (BAR_H + 8.0);
            let name = truncate_to_width(&section.name, name_w - 8.0, &label_style);
            text(
                c,
                &name,
                12.0,
                500,
                rect.x + CARD_PAD,
                y + 3.0,
                theme.colors.text_mid.0,
            );
            let frac = section.size as f32 / max;
            c.push(rounded_rect(
                rect.x + CARD_PAD + name_w,
                y,
                bar_w,
                BAR_H,
                4.0,
                theme.glass.surface_active.0,
            ));
            c.push(rounded_rect(
                rect.x + CARD_PAD + name_w,
                y,
                bar_w * frac,
                BAR_H,
                4.0,
                theme.colors.accent.0,
            ));
            text(
                c,
                &fmt_bytes(section.size),
                12.0,
                400,
                rect.x + CARD_PAD + name_w + bar_w + 8.0,
                y + 3.0,
                theme.colors.text_dim.0,
            );
        }
    }

    /// Benchmark cards: exact latency; ANN latency + recall when present.
    fn render_benchmark(&self, c: &mut Compositor, rect: Rect, theme: &Theme, res: &BenchmarkView) {
        group_label(
            c,
            &format!(
                "BENCHMARK — {} queries · k={} · dim={} · {}",
                res.n_queries, res.k, res.dim, res.dtype
            ),
            rect.x,
            rect.y,
            theme,
        );
        let cards_y = rect.y + 28.0;
        let card_w = ((rect.w - GAP) / 2.0).min(420.0);
        let exact_rect = Rect::new(rect.x, cards_y, card_w, 200.0);
        self.render_stats_card(c, exact_rect, theme, "EXACT (hot)", &res.exact);
        if let Some(ann) = &res.ann {
            let ann_rect = Rect::new(rect.x + card_w + GAP, cards_y, card_w, 200.0);
            self.render_stats_card(c, ann_rect, theme, "ANN / HNSW (hot)", ann);
            if let Some(recall) = res.recall_at_k {
                let text_w = TextMeasurer::measure_styled(
                    &format!("recall@{} vs exact: {:.4}", res.k, recall),
                    &TextStyle::new(13.0).with_weight(600),
                    None,
                )
                .0;
                text(
                    c,
                    &format!("recall@{} vs exact: {:.4}", res.k, recall),
                    13.0,
                    600,
                    ann_rect.x + (card_w - text_w) / 2.0,
                    ann_rect.y + ann_rect.h - 24.0,
                    theme.colors.success.0,
                );
            }
        }
    }

    fn render_stats_card(
        &self,
        c: &mut Compositor,
        rect: Rect,
        theme: &Theme,
        title: &str,
        stats: &LatencyStats,
    ) {
        panel(c, rect, theme);
        group_label(c, title, rect.x + CARD_PAD, rect.y + CARD_PAD, theme);
        let rows = [
            ("mean", stats.mean),
            ("p50", stats.p50),
            ("p95", stats.p95),
            ("p99", stats.p99),
            ("min – max", 0.0),
        ];
        for (row, (key, value)) in rows.iter().enumerate() {
            let y = rect.y + CARD_PAD + 24.0 + row as f32 * ROW_H;
            text(
                c,
                key,
                13.0,
                600,
                rect.x + CARD_PAD,
                y,
                theme.colors.text_dim.0,
            );
            let v = if row == 4 {
                format!("{:.3} – {:.3} ms", stats.min, stats.max)
            } else {
                format!("{value:.3} ms")
            };
            text(
                c,
                &v,
                13.0,
                400,
                rect.x + CARD_PAD + 96.0,
                y,
                theme.colors.text_mid.0,
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Headless stats tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view::fixtures;

    fn content(w: f32, h: f32) -> Rect {
        Rect::new(40.0, 128.0, w - 80.0, h - 168.0)
    }

    fn fake_bench() -> BenchmarkView {
        BenchmarkView {
            n_queries: 50,
            k: 10,
            dim: 4,
            dtype: "float32".into(),
            simd_backend: "neon".into(),
            exact: crate::model::bench::latency_stats(
                &(0..50).map(|i| i as f64 * 0.1).collect::<Vec<_>>(),
            ),
            ann: Some(crate::model::bench::latency_stats(
                &(0..50).map(|i| i as f64 * 0.02).collect::<Vec<_>>(),
            )),
            recall_at_k: Some(0.97),
        }
    }

    #[test]
    fn run_button_emits_the_benchmark_action() {
        let mut screen = StatsScreen::new();
        let content = content(1600.0, 1000.0);
        let (_, _, run) = screen.controls(content);
        let (x, y) = run.center();
        screen.handle_event(&WidgetEvent::MouseDown { x, y }, content, true);
        let (r, action) = screen.handle_event(&WidgetEvent::MouseUp { x, y }, content, true);
        assert!(r.clicked);
        assert_eq!(
            action,
            Action::RunBenchmark {
                n_queries: 50,
                k: 10
            }
        );
        assert!(screen.running);
    }

    #[test]
    fn fold_result_and_render_at_two_widths() {
        let db = fixtures::fake_db();
        let theme = Theme::hoff();
        let mut screen = StatsScreen::new();
        screen.running = true;
        screen.fold_result(Err("boom".to_string()));
        assert!(!screen.running);
        assert_eq!(screen.error, "boom");

        screen.fold_result(Ok(fake_bench()));
        assert!(screen.error.is_empty());
        for (w, h) in [(800.0, 600.0), (1600.0, 1000.0)] {
            let mut c = Compositor::new();
            screen.render(&mut c, content(w, h), &theme, &db);
        }
    }
}
