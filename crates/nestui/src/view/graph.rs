//! GRAPH screen: the chunk-to-chunk graph on the engine's [`GraphView`]
//! (layout, pan/zoom, hover hit-testing, selection). This screen keeps
//! only the app-level parts: the legend + Fit control, the hover tooltip
//! (chunk id + text preview) and the selected chunk's detail panel.
//!
//! Edge kinds are the nest wire types (NEXT_CHUNK=0, SEMANTIC=1,
//! CITATION=2) and the widget's default tones match the old hand-rolled
//! canvas: dim neutral / accent / info.

use engine::compositor::{Compositor, SceneNode, TextNodeKey};
use engine::graph::GraphScene;
use engine::text::{TextMeasurer, TextStyle};
use engine::theme::Theme;
use engine::ui::widgets::{
    Button, ButtonSize, ButtonVariant, EventResult, GraphView, IconButton, Rect, Spinner,
    SpinnerSize, WidgetEvent, menu_shadow,
};

use crate::model::types::ChunksData;

use super::field::FIELD_H;
use super::{Action, group_label, panel, short_id, text};

const GAP: f32 = 12.0;
const DETAIL_W: f32 = 320.0;

/// Everything the Graph screen needs from the central view state.
pub struct GraphContext<'a> {
    pub chunk_ids: &'a [String],
    pub chunks: Option<&'a ChunksData>,
    /// Canonical text lookup for tooltips/previews (chunk_id → first line).
    pub text_of: &'a dyn Fn(&str) -> Option<String>,
}

pub struct GraphScreen {
    view: GraphView,
    fit_button: Button,
    copy_id: IconButton,
    spinner: Spinner,
    pub loading: bool,
    pub error: String,
}

impl GraphScreen {
    pub fn new() -> Self {
        Self {
            view: GraphView::new(),
            fit_button: Button::new("Fit view")
                .size(ButtonSize::Sm)
                .variant(ButtonVariant::Outline),
            copy_id: IconButton::new("copy").variant(ButtonVariant::Ghost),
            spinner: Spinner::new().size(SpinnerSize::Sm),
            loading: false,
            error: String::new(),
        }
    }

    /// Reset per-database state.
    pub fn reset(&mut self) {
        self.view.clear();
        self.loading = false;
        self.error = String::new();
    }

    /// Fold a worker result into the screen (the layout arrives
    /// precomputed; the engine widget fits it to the canvas).
    pub fn fold_scene(&mut self, result: Result<GraphScene, String>) {
        self.loading = false;
        match result {
            Ok(scene) => {
                self.error = String::new();
                self.view.set_scene(scene);
            }
            Err(e) => self.error = e,
        }
    }

    pub fn has_scene(&self) -> bool {
        self.view.scene().is_some()
    }

    /// Canvas + detail rects (detail appears with a selection when wide).
    fn layout(&self, content: Rect) -> (Rect, Option<Rect>) {
        let canvas = Rect::new(
            content.x,
            content.y + FIELD_H + GAP,
            content.w,
            content.h - FIELD_H - GAP,
        );
        if self.view.selected().is_some() && canvas.w >= 760.0 {
            let detail = Rect::new(canvas.x + canvas.w - DETAIL_W, canvas.y, DETAIL_W, canvas.h);
            (
                Rect::new(canvas.x, canvas.y, canvas.w - DETAIL_W - GAP, canvas.h),
                Some(detail),
            )
        } else {
            (canvas, None)
        }
    }

    /// App ordinal of a scene node (identity unless subsampled).
    fn chunk_of(&self, scene_node: usize) -> Option<usize> {
        self.view
            .scene()
            .and_then(|s| s.node_to.get(scene_node).map(|c| *c as usize))
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        ctx: &GraphContext,
    ) -> (EventResult, Action) {
        let (canvas, detail) = self.layout(content);

        // Fit button (controls row).
        let r = self.fit_button.handle_event(event, self.fit_rect(content));
        if r.clicked {
            self.view.fit_view();
            return (r, Action::None);
        }
        let mut result = r;

        // Detail panel copy button.
        if let (Some(detail), Some(sel)) = (detail, self.view.selected()) {
            let r = self.copy_id.handle_event(event, self.copy_rect(detail));
            if r.clicked
                && let Some(chunk) = self.chunk_of(sel)
                && let Some(id) = ctx.chunk_ids.get(chunk)
            {
                return (
                    r,
                    Action::Copy {
                        text: id.clone(),
                        what: "chunk id".to_string(),
                    },
                );
            }
            result = result.merge(r);
        }

        result = result.merge(self.view.handle_event(event, canvas));
        (result, Action::None)
    }

    fn fit_rect(&self, content: Rect) -> Rect {
        let (w, h) = self.fit_button.preferred_size();
        Rect::new(content.x + content.w - w, content.y, w, h)
    }

    fn copy_rect(&self, detail: Rect) -> Rect {
        Rect::new(
            detail.x + detail.w - 16.0 - 40.0,
            detail.y + 8.0,
            40.0,
            40.0,
        )
    }

    /// The spinner animates only while a layout is in flight.
    pub fn tick(&mut self, dt: f32) -> bool {
        self.loading && self.spinner.tick(dt)
    }

    pub fn render(&mut self, c: &mut Compositor, content: Rect, theme: &Theme, ctx: &GraphContext) {
        // Controls row: legend + fit button. The legend mirrors the
        // widget's default EdgeTones (0 dim, 1 accent, 2 info).
        let legend_y = content.y + FIELD_H / 2.0 - 6.0;
        let mut lx = content.x;
        let label_style = TextStyle::new(11.0).with_weight(500);
        for (label, color) in [
            ("next chunk", theme.glass.edge.0),
            ("semantic", theme.colors.accent.0),
            ("citation", theme.colors.info.0),
        ] {
            c.push(engine::ui::widgets::rounded_rect(
                lx,
                legend_y + 2.0,
                10.0,
                4.0,
                2.0,
                color,
            ));
            text(
                c,
                label,
                11.0,
                500,
                lx + 16.0,
                legend_y,
                theme.colors.text_dim.0,
            );
            lx += 16.0 + TextMeasurer::measure_styled(label, &label_style, None).0 + 20.0;
        }
        self.fit_button.render(c, self.fit_rect(content), theme);

        let (canvas, detail) = self.layout(content);

        if !self.error.is_empty() {
            let msg = self.error.clone();
            text(
                c,
                &msg,
                13.0,
                500,
                canvas.x,
                canvas.y + 8.0,
                theme.colors.danger.0,
            );
            return;
        }
        if self.loading {
            self.spinner
                .render(c, Rect::new(canvas.x, canvas.y + 4.0, 16.0, 16.0), theme);
            text(
                c,
                "laying out the graph on the worker…",
                13.0,
                400,
                canvas.x + 24.0,
                canvas.y + 8.0,
                theme.colors.text_dim.0,
            );
            return;
        }
        let Some(scene) = self.view.scene() else {
            return;
        };
        if scene.subsampled {
            let note = format!(
                "showing {} of {} nodes (BFS neighborhood from the busiest node)",
                scene.graph.n_nodes,
                ctx.chunk_ids.len()
            );
            text(
                c,
                &note,
                11.0,
                400,
                canvas.x,
                canvas.y - 2.0,
                theme.colors.text_dim.0,
            );
        }

        panel(c, canvas, theme);
        self.view.render(c, canvas, theme);

        if let (Some(detail), Some(sel)) = (detail, self.view.selected()) {
            self.render_detail(c, detail, theme, sel, ctx);
        }
        // Hover tooltip floats above the canvas.
        if let Some(hover) = self.view.hovered() {
            self.render_tooltip(c, canvas, theme, hover, ctx);
        }
    }

    fn render_tooltip(
        &self,
        c: &mut Compositor,
        canvas: Rect,
        theme: &Theme,
        node: usize,
        ctx: &GraphContext,
    ) {
        let Some(chunk) = self.chunk_of(node) else {
            return;
        };
        let id = ctx.chunk_ids.get(chunk).map(String::as_str).unwrap_or("");
        let preview = (ctx.text_of)(id).unwrap_or_default();
        let line1 = format!("#{chunk}  {}", short_id(id));
        let style = TextStyle::new(12.0);
        let w1 = TextMeasurer::measure_styled(&line1, &style, None).0;
        let preview = TextMeasurer::truncate_to_width(&preview, &style, 240.0);
        let w = (w1.max(TextMeasurer::measure_styled(&preview, &style, None).0) + 24.0).max(80.0);
        let h = if preview.is_empty() { 40.0 } else { 58.0 };

        let Some((sx, sy)) = self.view.node_screen_pos(node) else {
            return;
        };
        // Keep the tooltip inside the canvas.
        let x = (sx + 12.0).min(canvas.x + canvas.w - w - 4.0);
        let y = (sy - h - 10.0).max(canvas.y + 4.0);
        let rect = Rect::new(x, y, w, h);
        c.push(menu_shadow(rect, theme.radius.md));
        for node in engine::ui::widgets::glass_pill(
            rect,
            theme.radius.md,
            theme.glass.edge_soft.0,
            1.0,
            theme.glass.popover.0,
        ) {
            c.push(node);
        }
        text(c, &line1, 12.0, 600, x + 12.0, y + 8.0, theme.colors.text.0);
        if !preview.is_empty() {
            text(
                c,
                &preview,
                12.0,
                400,
                x + 12.0,
                y + 28.0,
                theme.colors.text_mid.0,
            );
        }
    }

    fn render_detail(
        &mut self,
        c: &mut Compositor,
        detail: Rect,
        theme: &Theme,
        node: usize,
        ctx: &GraphContext,
    ) {
        panel(c, detail, theme);
        group_label(c, "CHUNK", detail.x + 16.0, detail.y + 16.0, theme);
        self.copy_id.render(c, self.copy_rect(detail), theme);

        let Some(chunk) = self.chunk_of(node) else {
            return;
        };
        let id = ctx.chunk_ids.get(chunk).map(String::as_str).unwrap_or("");
        let id_style = TextStyle::new(13.0).with_weight(500);
        let short = TextMeasurer::truncate_to_width(id, &id_style, detail.w - 32.0 - 48.0);
        text(
            c,
            &short,
            13.0,
            500,
            detail.x + 16.0,
            detail.y + 16.0 + 24.0,
            theme.colors.text.0,
        );

        let degree = self.view.scene().map(|s| s.graph.degree(node)).unwrap_or(0);
        text(
            c,
            &format!("{degree} out-edges · corpus chunk #{chunk}"),
            12.0,
            400,
            detail.x + 16.0,
            detail.y + 16.0 + 48.0,
            theme.colors.text_dim.0,
        );
        let (uri, offsets) = match ctx.chunks.and_then(|d| d.metas.get(chunk)) {
            Some(m) => (
                m.source_uri.clone(),
                format!("bytes {}–{}", m.offset_start, m.offset_end),
            ),
            None => ("(spans not loaded)".to_string(), String::new()),
        };
        let meta_style = TextStyle::new(12.0);
        let uri = TextMeasurer::truncate_to_width(&uri, &meta_style, detail.w - 32.0);
        text(
            c,
            &uri,
            12.0,
            400,
            detail.x + 16.0,
            detail.y + 16.0 + 70.0,
            theme.colors.text_mid.0,
        );
        text(
            c,
            &offsets,
            12.0,
            400,
            detail.x + 16.0,
            detail.y + 16.0 + 92.0,
            theme.colors.text_dim.0,
        );

        // Text preview (first lines; full text lives on the Chunks tab).
        if let Some(full) = ctx.chunks.and_then(|d| d.texts.get(chunk)) {
            let area = Rect::new(
                detail.x + 16.0,
                detail.y + 16.0 + 118.0,
                detail.w - 32.0,
                detail.h - 16.0 - 118.0 - 12.0,
            );
            let style = TextStyle::new(13.0).with_line_height(13.0 * 1.5);
            c.push(SceneNode::PushClip {
                x: area.x,
                y: area.y,
                w: area.w,
                h: area.h,
            });
            c.push(SceneNode::Text {
                key: TextNodeKey::from_style(full, &style, Some(area.w)),
                x: area.x,
                y: area.y,
                color: theme.colors.text_mid.0,
            });
            c.push(SceneNode::PopClip);
        }
    }
}

// ---------------------------------------------------------------------------
// Headless graph tests (over the engine GraphView)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view::fixtures;

    fn content() -> Rect {
        Rect::new(40.0, 128.0, 1520.0, 832.0)
    }

    #[test]
    fn renders_a_loaded_scene_at_narrow_and_wide() {
        let db = fixtures::fake_db();
        let chunks = fixtures::fake_chunks();
        let lookup = |id: &str| {
            db.chunk_ids
                .iter()
                .position(|c| c == id)
                .and_then(|i| chunks.texts.get(i))
                .and_then(|t| t.lines().next())
                .map(str::to_string)
        };
        let ctx = GraphContext {
            chunk_ids: &db.chunk_ids,
            chunks: Some(&chunks),
            text_of: &lookup,
        };
        let theme = Theme::hoff();
        let mut screen = GraphScreen::new();
        screen.fold_scene(Ok(fixtures::fake_graph_scene()));
        assert!(screen.has_scene());
        for (w, h) in [(800.0, 600.0), (1600.0, 1000.0)] {
            let mut c = Compositor::new();
            screen.render(
                &mut c,
                Rect::new(40.0, 128.0, w - 80.0, h - 168.0),
                &theme,
                &ctx,
            );
        }
    }

    #[test]
    fn click_selects_a_node_and_shows_the_detail_panel() {
        let db = fixtures::fake_db();
        let chunks = fixtures::fake_chunks();
        let lookup = |_: &str| Some("preview".to_string());
        let ctx = GraphContext {
            chunk_ids: &db.chunk_ids,
            chunks: Some(&chunks),
            text_of: &lookup,
        };
        let theme = Theme::hoff();
        let mut screen = GraphScreen::new();
        screen.fold_scene(Ok(fixtures::fake_graph_scene()));

        // Render once to settle the fit transform, then click node 0.
        let mut c = Compositor::new();
        screen.render(&mut c, content(), &theme, &ctx);
        let (sx, sy) = screen.view.node_screen_pos(0).unwrap();
        screen.handle_event(&WidgetEvent::MouseMove { x: sx, y: sy }, content(), &ctx);
        assert_eq!(screen.view.hovered(), Some(0));
        screen.handle_event(&WidgetEvent::MouseDown { x: sx, y: sy }, content(), &ctx);
        let (r, _) = screen.handle_event(&WidgetEvent::MouseUp { x: sx, y: sy }, content(), &ctx);
        assert!(r.clicked);
        assert_eq!(screen.view.selected(), Some(0));
        // Detail panel renders alongside.
        let mut c = Compositor::new();
        screen.render(&mut c, content(), &theme, &ctx);
    }

    #[test]
    fn error_and_loading_states_render() {
        let db = fixtures::fake_db();
        let lookup = |_: &str| None;
        let ctx = GraphContext {
            chunk_ids: &db.chunk_ids,
            chunks: None,
            text_of: &lookup,
        };
        let theme = Theme::hoff();
        let mut screen = GraphScreen::new();
        screen.loading = true;
        assert!(screen.tick(0.016), "spinner runs while loading");
        let mut c = Compositor::new();
        screen.render(&mut c, content(), &theme, &ctx);
        screen.fold_scene(Err("no graph section in this file".to_string()));
        assert!(!screen.loading);
        assert!(!screen.tick(0.016), "idle after the error");
        let mut c = Compositor::new();
        screen.render(&mut c, content(), &theme, &ctx);
    }
}
