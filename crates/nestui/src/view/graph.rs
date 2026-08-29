//! GRAPH screen: the chunk-to-chunk CSR rendered as a force-directed
//! node-link canvas with pan (drag) and zoom (wheel), hover tooltips and
//! a selected-node detail panel.
//!
//! The layout arrives ready-made from the worker (`GraphScene`, world
//! space 1000×1000); this screen owns only the [`ViewTransform`], pointer
//! state and buttons. Rendering batches edges into one path per edge
//! type (stroke color documents the type: NEXT_CHUNK = neutral edge-light,
//! SEMANTIC = accent, CITATION = informational — HOFF tokens, low alpha
//! so dense clusters stay readable; the selected node's incident edges go
//! full alpha).

use engine::compositor::{Compositor, SceneNode, TextNodeKey};
use engine::path::PathBuilder;
use engine::text::{TextMeasurer, TextStyle};
use engine::theme::Theme;
use engine::ui::widgets::{Button, ButtonSize, ButtonVariant, EventResult, Rect, WidgetEvent};

use crate::model::graph::{EDGE_CITATION, EDGE_NEXT_CHUNK, GraphScene, ViewTransform};
use crate::model::types::ChunksData;

use super::field::FIELD_H;
use super::{Action, group_label, panel, short_id, text, truncate_to_width};

const GAP: f32 = 12.0;
/// World-space node radius; screen radius is clamped per zoom level.
const NODE_R: f32 = 5.0;
const DETAIL_W: f32 = 320.0;

/// Everything the Graph screen needs from the central view state.
pub struct GraphContext<'a> {
    pub chunk_ids: &'a [String],
    pub chunks: Option<&'a ChunksData>,
    /// Canonical text lookup for tooltips/previews (chunk_id → first line).
    pub text_of: &'a dyn Fn(&str) -> Option<String>,
}

pub struct GraphScreen {
    scene: Option<GraphScene>,
    transform: ViewTransform,
    /// Fit the world into the viewport on the next render (after load or
    /// the Fit button).
    needs_fit: bool,
    hover: Option<usize>,
    selected: Option<usize>,
    /// Last cursor position while panning.
    drag: Option<(f32, f32)>,
    /// Distance dragged since MouseDown (click vs. pan discrimination).
    drag_dist: f32,
    fit_button: Button,
    copy_id: Button,
    pub loading: bool,
    pub error: String,
}

impl GraphScreen {
    pub fn new() -> Self {
        Self {
            scene: None,
            transform: ViewTransform::default(),
            needs_fit: true,
            hover: None,
            selected: None,
            drag: None,
            drag_dist: 0.0,
            fit_button: Button::new("Fit view")
                .size(ButtonSize::Sm)
                .variant(ButtonVariant::Outline),
            copy_id: Button::new("copy id")
                .size(ButtonSize::Sm)
                .variant(ButtonVariant::Ghost)
                .icon("copy"),
            loading: false,
            error: String::new(),
        }
    }

    /// Reset per-database state.
    pub fn reset(&mut self) {
        self.scene = None;
        self.hover = None;
        self.selected = None;
        self.loading = false;
        self.error = String::new();
        self.needs_fit = true;
    }

    /// Fold a worker result into the screen.
    pub fn fold_scene(&mut self, result: Result<GraphScene, String>) {
        self.loading = false;
        match result {
            Ok(scene) => {
                self.error = String::new();
                self.scene = Some(scene);
                self.needs_fit = true;
                self.hover = None;
                self.selected = None;
            }
            Err(e) => self.error = e,
        }
    }

    pub fn has_scene(&self) -> bool {
        self.scene.is_some()
    }

    /// Canvas + detail rects.
    fn layout(&self, content: Rect) -> (Rect, Option<Rect>) {
        let canvas = Rect::new(
            content.x,
            content.y + FIELD_H + GAP,
            content.w,
            content.h - FIELD_H - GAP,
        );
        if self.selected.is_some() && canvas.w >= 760.0 {
            let detail = Rect::new(canvas.x + canvas.w - DETAIL_W, canvas.y, DETAIL_W, canvas.h);
            (
                Rect::new(canvas.x, canvas.y, canvas.w - DETAIL_W - GAP, canvas.h),
                Some(detail),
            )
        } else {
            (canvas, None)
        }
    }

    /// Node radius in screen px at the current zoom (clamped so nodes
    /// stay visible zoomed out and stop growing zoomed in).
    fn node_r(&self) -> f32 {
        (NODE_R * self.transform.scale).clamp(2.0, 9.0)
    }

    /// Nearest node to a screen point within its click radius.
    fn node_at(&self, scene: &GraphScene, x: f32, y: f32) -> Option<usize> {
        let r = self.node_r() + 3.0;
        let mut best: Option<(usize, f32)> = None;
        for (i, &(wx, wy)) in scene.positions.iter().enumerate() {
            let (sx, sy) = self.transform.world_to_screen(wx, wy);
            let d = ((sx - x).powi(2) + (sy - y).powi(2)).sqrt();
            if d <= r && best.is_none_or(|(_, bd)| d < bd) {
                best = Some((i, d));
            }
        }
        best.map(|(i, _)| i)
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
            self.needs_fit = true;
            return (r, Action::None);
        }
        let mut result = r;

        // Detail panel copy button.
        if let (Some(detail), Some(sel)) = (detail, self.selected)
            && let Some(scene) = &self.scene
        {
            let r = self.copy_id.handle_event(event, self.copy_rect(detail));
            if r.clicked {
                let chunk = scene.node_to_chunk[sel] as usize;
                if let Some(id) = ctx.chunk_ids.get(chunk) {
                    return (
                        r,
                        Action::Copy {
                            text: id.clone(),
                            what: "chunk id".to_string(),
                        },
                    );
                }
            }
            result = result.merge(r);
        }

        if !canvas.contains(event.pos().0, event.pos().1) && self.drag.is_none() {
            return (result, Action::None);
        }
        let Some(scene) = &self.scene else {
            return (result, Action::None);
        };

        match *event {
            WidgetEvent::MouseMove { x, y } => {
                if let Some((lx, ly)) = self.drag {
                    // Pan follows the cursor 1:1 (screen px).
                    self.transform.pan_by(x - lx, y - ly);
                    self.drag = Some((x, y));
                    self.drag_dist += (x - lx).abs() + (y - ly).abs();
                    return (EventResult::changed(), Action::None);
                }
                let hit = self.node_at(scene, x, y);
                if hit != self.hover {
                    self.hover = hit;
                    result = result.merge(EventResult::changed());
                }
            }
            WidgetEvent::MouseDown { x, y } => {
                self.drag = Some((x, y));
                self.drag_dist = 0.0;
                result = result.merge(EventResult {
                    handled: true,
                    ..EventResult::IGNORED
                });
            }
            WidgetEvent::MouseUp { x, y } => {
                if self.drag.take().is_some() && self.drag_dist < 4.0 {
                    // A click, not a pan: toggle the node selection.
                    let hit = self.node_at(scene, x, y);
                    if hit != self.selected {
                        self.selected = hit;
                        result = result.merge(EventResult::changed());
                    }
                }
            }
            WidgetEvent::Scroll { x, y, delta } => {
                // Wheel zoom, anchored at the cursor. Trackpad pixel deltas
                // are small, line deltas large; exp() normalizes both.
                self.transform.zoom_at((x, y), (-delta * 0.002).exp());
                result = result.merge(EventResult::changed());
            }
        }
        (result, Action::None)
    }

    fn fit_rect(&self, content: Rect) -> Rect {
        let (w, h) = self.fit_button.preferred_size();
        Rect::new(content.x + content.w - w, content.y, w, h)
    }

    fn copy_rect(&self, detail: Rect) -> Rect {
        let (w, h) = self.copy_id.preferred_size();
        Rect::new(detail.x + detail.w - 16.0 - w, detail.y + 12.0, w, h)
    }

    /// Keyboard/pointer idle: nothing animates on this screen. Kept for
    /// symmetry with the other screens (the explorer ticks all of them).
    pub fn tick(&mut self, _dt: f32) -> bool {
        false
    }

    pub fn render(&mut self, c: &mut Compositor, content: Rect, theme: &Theme, ctx: &GraphContext) {
        // Controls row: legend + fit button.
        let legend_y = content.y + FIELD_H / 2.0 - 6.0;
        let mut lx = content.x;
        for (label, color) in self.legend(theme) {
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
            lx += 16.0
                + TextMeasurer::measure_styled(label, &TextStyle::new(11.0).with_weight(500), None)
                    .0
                + 20.0;
        }
        self.fit_button.render(c, self.fit_rect(content), theme);

        let (canvas, detail) = self.layout(content);

        if !self.error.is_empty() {
            text(
                c,
                &self.error.clone(),
                13.0,
                500,
                canvas.x,
                canvas.y + 8.0,
                theme.colors.danger.0,
            );
            return;
        }
        if self.loading {
            text(
                c,
                "laying out the graph on the worker…",
                13.0,
                400,
                canvas.x,
                canvas.y + 8.0,
                theme.colors.text_dim.0,
            );
            return;
        }
        let Some(scene) = self.scene.clone() else {
            return;
        };
        if self.needs_fit {
            self.transform = ViewTransform::fit(1000.0, 1000.0, canvas.w, canvas.h, 24.0);
            self.needs_fit = false;
        }
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

        self.render_canvas(c, canvas, theme, &scene);
        if let (Some(detail), Some(sel)) = (detail, self.selected) {
            self.render_detail(c, detail, theme, &scene, sel, ctx);
        }
        // Hover tooltip floats above the canvas.
        if let Some(hover) = self.hover {
            self.render_tooltip(c, canvas, theme, &scene, hover, ctx);
        }
    }

    /// Legend entries: (label, color) per edge type, from theme tokens.
    fn legend(&self, theme: &Theme) -> [(&'static str, [f32; 4]); 3] {
        [
            ("next chunk", theme.glass.edge.0),
            ("semantic", theme.colors.accent.0),
            ("citation", theme.colors.info.0),
        ]
    }

    fn edge_color(&self, edge_type: u8, theme: &Theme, hot: bool) -> [f32; 4] {
        let base = match edge_type {
            EDGE_NEXT_CHUNK => theme.glass.edge.0,
            EDGE_CITATION => theme.colors.info.0,
            _ => theme.colors.accent.0, // semantic
        };
        // Cool pass at low alpha so dense clusters stay readable; the
        // selected node's incident edges go nearly opaque.
        let alpha = if hot { 0.9 } else { 0.35 };
        [base[0], base[1], base[2], alpha]
    }

    fn render_canvas(&self, c: &mut Compositor, canvas: Rect, theme: &Theme, scene: &GraphScene) {
        panel(c, canvas, theme);
        c.push(SceneNode::PushClip {
            x: canvas.x,
            y: canvas.y,
            w: canvas.w,
            h: canvas.h,
        });

        let r = self.node_r();
        let edge_w = (self.transform.scale * 1.2).clamp(0.5, 2.5);

        // Edges: one batched path per type, cool pass (all edges) then a
        // hot pass (the selected node's incident edges at full alpha).
        let selected = self.selected;
        let hot = |i: usize, j: u32| selected.is_some_and(|s| s == i || s as u32 == j);
        for pass_hot in [false, true] {
            // Per-type builders, created lazily.
            let mut builders: [Option<PathBuilder>; 3] = [None, None, None];
            for i in 0..scene.graph.n_nodes {
                let (x1, y1) = self
                    .transform
                    .world_to_screen(scene.positions[i].0, scene.positions[i].1);
                for (k, &j) in scene.graph.neighbors(i).iter().enumerate() {
                    let j = j as usize;
                    if j >= scene.graph.n_nodes || j < i {
                        continue; // draw each pair once
                    }
                    if hot(i, j as u32) != pass_hot {
                        continue;
                    }
                    let t = scene.graph.edge_type(i, k).unwrap_or(0).min(2) as usize;
                    let (x2, y2) = self
                        .transform
                        .world_to_screen(scene.positions[j].0, scene.positions[j].1);
                    let b = builders[t].get_or_insert_with(PathBuilder::new);
                    // Lyon requires end() between sub-paths; each edge is
                    // its own open sub-path in the batched stroke.
                    *b = std::mem::take(b).move_to(x1, y1).line_to(x2, y2).end_open();
                }
            }
            for (t, builder) in builders.into_iter().enumerate() {
                if let Some(b) = builder {
                    let color = self.edge_color(t as u8, theme, pass_hot);
                    // Every sub-path already ended (`end_open` per edge).
                    c.draw_path(b.stroke(color, edge_w));
                }
            }
        }

        // Nodes: accent dots; hovered/selected get a ring and full alpha.
        let node_color = theme.colors.accent.0;
        for (i, &(wx, wy)) in scene.positions.iter().enumerate() {
            let (sx, sy) = self.transform.world_to_screen(wx, wy);
            let is_sel = self.selected == Some(i);
            let is_hov = self.hover == Some(i);
            let alpha = if is_sel || is_hov { 1.0 } else { 0.65 };
            let color = [node_color[0], node_color[1], node_color[2], alpha];
            c.draw_path(PathBuilder::circle(sx, sy, r).fill(color));
            if is_sel || is_hov {
                c.draw_path(PathBuilder::circle(sx, sy, r + 3.0).stroke(theme.colors.text.0, 1.0));
            }
        }
        c.push(SceneNode::PopClip);
    }

    fn render_tooltip(
        &self,
        c: &mut Compositor,
        canvas: Rect,
        theme: &Theme,
        scene: &GraphScene,
        node: usize,
        ctx: &GraphContext,
    ) {
        let chunk = scene.node_to_chunk[node] as usize;
        let id = ctx.chunk_ids.get(chunk).map(String::as_str).unwrap_or("");
        let preview = (ctx.text_of)(id).unwrap_or_default();
        let line1 = format!("#{chunk}  {}", short_id(id));
        let style = TextStyle::new(12.0);
        let w1 = TextMeasurer::measure_styled(&line1, &style, None).0;
        let preview = truncate_to_width(&preview, 240.0, &style);
        let w = (w1.max(TextMeasurer::measure_styled(&preview, &style, None).0) + 24.0).max(80.0);
        let h = if preview.is_empty() { 40.0 } else { 58.0 };

        let (sx, sy) = self
            .transform
            .world_to_screen(scene.positions[node].0, scene.positions[node].1);
        // Keep the tooltip inside the canvas.
        let x = (sx + 12.0).min(canvas.x + canvas.w - w - 4.0);
        let y = (sy - h - 10.0).max(canvas.y + 4.0);
        let rect = Rect::new(x, y, w, h);
        c.push(engine::ui::widgets::menu_shadow(rect, theme.radius.md));
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
        scene: &GraphScene,
        node: usize,
        ctx: &GraphContext,
    ) {
        panel(c, detail, theme);
        group_label(c, "CHUNK", detail.x + 16.0, detail.y + 16.0, theme);
        self.copy_id.render(c, self.copy_rect(detail), theme);

        let chunk = scene.node_to_chunk[node] as usize;
        let id = ctx.chunk_ids.get(chunk).map(String::as_str).unwrap_or("");
        let id_style = TextStyle::new(13.0).with_weight(500);
        let short = truncate_to_width(
            id,
            detail.w - 32.0 - self.copy_rect(detail).w - 8.0,
            &id_style,
        );
        text(
            c,
            &short,
            13.0,
            500,
            detail.x + 16.0,
            detail.y + 16.0 + 24.0,
            theme.colors.text.0,
        );

        let degree = scene.graph.degree(node);
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
        let uri = truncate_to_width(&uri, detail.w - 32.0, &meta_style);
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
// Headless graph tests
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
    fn hover_selects_and_click_pan_zoom_work() {
        let db = fixtures::fake_db();
        let chunks = fixtures::fake_chunks();
        let lookup = |_: &str| Some("preview".to_string());
        let _ = &chunks;
        let ctx = GraphContext {
            chunk_ids: &db.chunk_ids,
            chunks: Some(&chunks),
            text_of: &lookup,
        };
        let theme = Theme::hoff();
        let mut screen = GraphScreen::new();
        screen.fold_scene(Ok(fixtures::fake_graph_scene()));

        // Render once to settle the fit transform.
        let mut c = Compositor::new();
        screen.render(&mut c, content(), &theme, &ctx);

        // Node 0's screen position.
        let scene = screen.scene.as_ref().unwrap();
        let (sx, sy) = screen
            .transform
            .world_to_screen(scene.positions[0].0, scene.positions[0].1);

        // Hover.
        let (r, _) = screen.handle_event(&WidgetEvent::MouseMove { x: sx, y: sy }, content(), &ctx);
        assert!(r.changed);
        assert_eq!(screen.hover, Some(0));

        // Click selects (down + up without drag).
        screen.handle_event(&WidgetEvent::MouseDown { x: sx, y: sy }, content(), &ctx);
        screen.handle_event(&WidgetEvent::MouseUp { x: sx, y: sy }, content(), &ctx);
        assert_eq!(screen.selected, Some(0));
        // The detail panel renders alongside.
        let mut c = Compositor::new();
        screen.render(&mut c, content(), &theme, &ctx);

        // Pan: drag from empty space.
        let before = screen.transform.offset;
        screen.handle_event(
            &WidgetEvent::MouseDown { x: 60.0, y: 200.0 },
            content(),
            &ctx,
        );
        screen.handle_event(
            &WidgetEvent::MouseMove { x: 110.0, y: 240.0 },
            content(),
            &ctx,
        );
        screen.handle_event(
            &WidgetEvent::MouseUp { x: 110.0, y: 240.0 },
            content(),
            &ctx,
        );
        assert_eq!(
            screen.transform.offset,
            (before.0 + 50.0, before.1 + 40.0),
            "pan follows the cursor"
        );

        // Zoom in anchored at the cursor.
        let scale = screen.transform.scale;
        let (r, _) = screen.handle_event(
            &WidgetEvent::Scroll {
                x: 400.0,
                y: 400.0,
                delta: -100.0,
            },
            content(),
            &ctx,
        );
        assert!(r.changed);
        assert!(screen.transform.scale > scale);
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
        let mut c = Compositor::new();
        screen.render(&mut c, content(), &theme, &ctx);
        screen.fold_scene(Err("no graph section in this file".to_string()));
        assert!(!screen.loading);
        let mut c = Compositor::new();
        screen.render(&mut c, content(), &theme, &ctx);
    }
}
