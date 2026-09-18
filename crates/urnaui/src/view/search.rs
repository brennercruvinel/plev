//! SEARCH screen: query input (free text via the offline embedder, or a
//! pasted JSON vector), path select (exact / ann / graph / hybrid, plus
//! one `space: <name>` entry per multimodal band), k slider, virtualized
//! results with score bars and text previews, and the explain panel
//! (route, candidates, recall, the rerank-source honesty marker) with the
//! selected hit's citation, canonical text and, for media corpora, the
//! decoded frame.
//!
//! The screen owns widget state and the last result; it never talks to
//! the worker directly — submissions bubble up as [`Action::RunSearch`]
//! and the shell validates against the open database.

use engine::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use engine::text::{TextMeasurer, TextStyle};
use engine::theme::Theme;
use engine::ui::widgets::{
    Button, EventResult, IconButton, Rect, Select, Slider, Spinner, SpinnerSize, Tabs, VirtualList,
    WidgetEvent,
};

use crate::model::types::{OpenedDbView, SearchMode, SearchResultsView};

use super::field::{FIELD_H, Field};
use super::{Action, ChunkLookup, EditKey, group_label, panel, short_id, span_label, text};

const GAP: f32 = 12.0;
const ROW_H: f32 = 56.0;
const KIND_W: f32 = 200.0;
const KIND_H: f32 = 36.0;
const MODE_W: f32 = 200.0;
const SLIDER_W: f32 = 200.0;
/// The four text-slab paths that precede the per-space entries in the
/// mode select.
const TEXT_MODES: [&str; 4] = ["exact", "ann", "graph", "hybrid"];
/// Frame preview box inside the explain panel.
const FRAME_H: f32 = 200.0;

/// How the query string is interpreted (the "text | vector" toggle).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QueryKind {
    Text,
    Vector,
}

/// Everything the Search screen needs from the central view state.
pub struct SearchContext<'a> {
    /// Embedding dim of the open db (for the vector hint); 0 when no db.
    pub dim: usize,
    /// Chunk texts, spans and decoded frames for previews.
    pub lookup: &'a ChunkLookup<'a>,
    /// Embedder availability (drives the text-mode hint).
    pub embedder: Option<&'a Result<String, String>>,
}

pub struct SearchScreen {
    kind_tabs: Tabs,
    query: Field,
    mode: Select,
    /// Space names behind the text modes in `mode` (same order).
    spaces: Vec<String>,
    k: Slider,
    search_button: Button,
    results: VirtualList,
    copy_citation: IconButton,
    spinner: Spinner,
    pub result: Option<SearchResultsView>,
    pub error: String,
    pub pending: bool,
}

impl SearchScreen {
    pub fn new(theme: &Theme) -> Self {
        Self {
            kind_tabs: Tabs::new(["text", "vector"]),
            query: Field::new("ask the corpus…", theme),
            mode: Select::new(TEXT_MODES, 0),
            spaces: Vec::new(),
            k: Slider::new(1.0, 100.0, 10.0).step(1.0),
            search_button: Button::new("Search").icon("search"),
            results: VirtualList::new(ROW_H),
            copy_citation: IconButton::new("copy")
                .variant(engine::ui::widgets::ButtonVariant::Ghost),
            spinner: Spinner::new().size(SpinnerSize::Sm),
            result: None,
            error: String::new(),
            pending: false,
        }
    }

    /// Reset per-database state and rebuild the path select for the new
    /// db's spaces (called when a new db opens).
    pub fn reset(&mut self, db: &OpenedDbView) {
        self.result = None;
        self.error = String::new();
        self.pending = false;
        self.results.selected = None;
        self.results.set_item_count(0);
        self.spaces = db.space_names.clone();
        let options = TEXT_MODES
            .iter()
            .map(|m| m.to_string())
            .chain(self.spaces.iter().map(|s| format!("space: {s}")));
        self.mode = Select::new(options, 0);
    }

    pub fn query_kind(&self) -> QueryKind {
        match self.kind_tabs.active {
            0 => QueryKind::Text,
            _ => QueryKind::Vector,
        }
    }

    /// Selected search path, with the CLI's default candidate budget
    /// (`(k * 4).max(64)`).
    fn selected_mode(&self) -> SearchMode {
        let cand = ((self.k.value() as usize) * 4).max(64);
        match self.mode.selected {
            1 => SearchMode::Ann { ef_search: cand },
            2 => SearchMode::Graph { hops: 1, ef: cand },
            3 => SearchMode::Hybrid {
                query_text: String::new(),
                candidates_per_path: cand,
            },
            i if i >= TEXT_MODES.len() => match self.spaces.get(i - TEXT_MODES.len()) {
                Some(name) => SearchMode::Space { name: name.clone() },
                None => SearchMode::Exact,
            },
            _ => SearchMode::Exact,
        }
    }

    pub fn select_is_open(&self) -> bool {
        self.mode.is_open()
    }

    pub fn close_select(&mut self) {
        self.mode.close();
    }

    /// While the dropdown is open it owns every event (priority over
    /// everything beneath it, like the showcase forms).
    pub fn route_select(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        has_result: bool,
    ) -> EventResult {
        self.mode
            .handle_event(event, self.layout(content, has_result).mode_select)
    }

    pub fn fold_result(&mut self, result: Result<SearchResultsView, String>) {
        self.pending = false;
        match result {
            Ok(r) => {
                self.error = String::new();
                self.results.set_item_count(r.hits.len());
                self.results.selected = None;
                self.result = Some(r);
            }
            Err(e) => {
                self.error = e;
            }
        }
    }

    fn submit(&mut self) -> Action {
        if self.pending || self.query.is_empty() {
            return Action::None;
        }
        self.pending = true;
        self.error = String::new();
        Action::RunSearch {
            query: self.query.text().trim().to_string(),
            is_vector: self.query_kind() == QueryKind::Vector,
            mode: self.selected_mode(),
            k: self.k.value() as i32,
        }
    }

    /// Submission failed client-side (bad vector JSON, dim mismatch) —
    /// undo the optimistic pending flag and show the reason inline.
    pub fn reject_submission(&mut self, reason: String) {
        self.pending = false;
        self.error = reason;
    }

    /// The selected hit's chunk ordinal, when texts are indexed.
    fn selected_ordinal(&self, lookup: &ChunkLookup) -> Option<usize> {
        let hit = self.result.as_ref()?.hits.get(self.results.selected?)?;
        lookup.ordinal(&hit.chunk_id)
    }

    fn layout(&self, content: Rect, has_result: bool) -> Layout {
        let kind_tabs = Rect::new(content.x, content.y, KIND_W, KIND_H);
        let query_y = content.y + KIND_H + GAP;
        let (bw, bh) = self.search_button.preferred_size();
        let query_field = Rect::new(
            content.x,
            query_y,
            (content.w - bw - GAP).max(120.0),
            FIELD_H.max(bh),
        );
        let search_button = Rect::new(content.x + content.w - bw, query_y, bw, bh);
        let opts_y = query_y + query_field.h + GAP;
        let mode_select = Rect::new(content.x, opts_y, MODE_W, 44.0);
        let slider = Rect::new(
            content.x + MODE_W + GAP * 2.0,
            opts_y + 12.0,
            SLIDER_W.min(content.w - MODE_W - GAP * 2.0 - 60.0),
            20.0,
        );
        let status_y = opts_y + 44.0 + GAP;
        let results_y = status_y + 24.0;
        let results_h = (content.y + content.h - results_y).max(80.0);
        let (results, explain) = if has_result && content.w >= 720.0 {
            let explain_w = (content.w * 0.36).clamp(280.0, 400.0);
            (
                Rect::new(content.x, results_y, content.w - explain_w - GAP, results_h),
                Some(Rect::new(
                    content.x + content.w - explain_w,
                    results_y,
                    explain_w,
                    results_h,
                )),
            )
        } else {
            (Rect::new(content.x, results_y, content.w, results_h), None)
        };
        Layout {
            kind_tabs,
            query_field,
            search_button,
            mode_select,
            slider,
            status_y,
            results,
            explain,
        }
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        has_db: bool,
    ) -> (EventResult, Action) {
        let l = self.layout(content, self.result.is_some());
        self.search_button.disabled = !has_db || self.pending || self.query.is_empty();

        let mut result = EventResult::IGNORED;

        // Controls first (topmost in z): copy citation, search, kind tabs,
        // mode select (closed pill), k slider.
        if let Some(explain) = l.explain
            && self.results.selected.is_some()
        {
            let r = self
                .copy_citation
                .handle_event(event, self.copy_rect(explain));
            if r.clicked
                && let (Some(res), Some(sel)) = (&self.result, self.results.selected)
                && let Some(hit) = res.hits.get(sel)
            {
                return (
                    r,
                    Action::Copy {
                        text: hit.citation_id.clone(),
                        what: "citation".to_string(),
                    },
                );
            }
            result = result.merge(r);
        }

        let r = self.search_button.handle_event(event, l.search_button);
        if r.clicked {
            self.query.unfocus();
            return (r, self.submit());
        }
        result = result.merge(r);

        let r = self.kind_tabs.handle_event(event, l.kind_tabs);
        if r.clicked {
            // Retune the placeholder to the new query kind.
            self.query.input.placeholder = match self.query_kind() {
                QueryKind::Text => "ask the corpus…".to_string(),
                QueryKind::Vector => "[0.012, -0.34, …]".to_string(),
            };
            return (r, Action::None);
        }
        result = result.merge(r);

        result = result.merge(self.mode.handle_event(event, l.mode_select));
        result = result.merge(self.k.handle_event(event, l.slider));

        // Results list.
        if let Some(res) = &self.result {
            self.results.set_item_count(res.hits.len());
        }
        result = result.merge(self.results.handle_event(event, l.results));

        // Query field: click focuses; clicks elsewhere blur.
        if let WidgetEvent::MouseDown { x, y } = *event {
            if l.query_field.contains(x, y) {
                self.query.click(x - l.query_field.x);
                return (EventResult::changed(), Action::None);
            }
            if self.query.input.focused {
                self.query.unfocus();
                return (EventResult::changed(), Action::None);
            }
        }
        (result, Action::None)
    }

    pub fn handle_text(&mut self, s: &str) -> bool {
        self.query.insert(s)
    }

    pub fn handle_edit_key(&mut self, key: EditKey) -> (bool, Action) {
        if key == EditKey::Enter && self.query.input.focused && !self.query.is_empty() {
            return (true, self.submit());
        }
        (self.query.edit(key), Action::None)
    }

    pub fn tick(&mut self, dt: f32) -> bool {
        let spinning = self.pending && self.spinner.tick(dt);
        self.query.tick(dt) | self.results.tick(dt) | spinning
    }

    fn copy_rect(&self, explain: Rect) -> Rect {
        let (w, h) = self.copy_citation.preferred_size();
        Rect::new(explain.x + explain.w - 16.0 - w, explain.y + 8.0, w, h)
    }

    /// The frame to decode for the selected hit, when it is a media chunk
    /// whose frame is not loaded yet. Rendering is the moment the panel
    /// knows what it shows, so the request is raised from there.
    pub fn wanted_frame(&self, lookup: &ChunkLookup) -> Option<usize> {
        let ordinal = self.selected_ordinal(lookup)?;
        (lookup.is_media(ordinal) && lookup.frame(ordinal).is_none()).then_some(ordinal)
    }

    pub fn render(
        &mut self,
        c: &mut Compositor,
        content: Rect,
        theme: &Theme,
        ctx: &SearchContext,
    ) {
        let has_result = self.result.is_some();
        let l = self.layout(content, has_result);
        self.search_button.disabled = ctx.dim == 0 || self.pending || self.query.is_empty();
        self.search_button.label = if self.pending {
            "Searching…"
        } else {
            "Search"
        }
        .to_string();

        self.kind_tabs.render(c, l.kind_tabs, theme);
        self.query.render(c, l.query_field, theme);
        self.search_button.render(c, l.search_button, theme);
        self.mode.render(c, l.mode_select, theme);
        self.k.render(c, l.slider, theme);
        text(
            c,
            &format!("k = {}", self.k.value() as i32),
            13.0,
            500,
            l.slider.x + l.slider.w + GAP,
            l.slider.y + 2.0,
            theme.colors.text_mid.0,
        );

        // Status line: error (destructive) > pending > mode hint.
        let status_style = TextStyle::new(13.0);
        if !self.error.is_empty() {
            let msg = TextMeasurer::truncate_to_width(&self.error, &status_style, content.w);
            text(
                c,
                &msg,
                13.0,
                500,
                content.x,
                l.status_y,
                theme.colors.danger.0,
            );
        } else if self.pending {
            let what = match (self.query_kind(), self.selected_mode()) {
                (QueryKind::Vector, _) => "searching…".to_string(),
                (QueryKind::Text, SearchMode::Space { name }) => {
                    format!("embedding query with the {name} model (offline)…")
                }
                (QueryKind::Text, _) => "embedding query (offline)…".to_string(),
            };
            text(
                c,
                &what,
                13.0,
                400,
                content.x,
                l.status_y,
                theme.colors.text_dim.0,
            );
        } else if let SearchMode::Space { name } = self.selected_mode() {
            let hint = match self.query_kind() {
                QueryKind::Text => format!(
                    "text-to-{name}: the query goes through that space's model (registry embedder, needs its deps); scores are exact cosine over the band"
                ),
                QueryKind::Vector => format!("vector must have the {name} space's dim"),
            };
            let hint = TextMeasurer::truncate_to_width(&hint, &status_style, content.w);
            text(
                c,
                &hint,
                13.0,
                400,
                content.x,
                l.status_y,
                theme.colors.text_dim.0,
            );
        } else if self.query_kind() == QueryKind::Text
            && let Some(Err(reason)) = ctx.embedder
        {
            let msg = TextMeasurer::truncate_to_width(reason, &status_style, content.w);
            text(
                c,
                &msg,
                13.0,
                400,
                content.x,
                l.status_y,
                theme.colors.text_dim.0,
            );
        }

        if let Some(res) = &self.result {
            self.results.set_item_count(res.hits.len());
            render_results(&mut self.results, c, l.results, theme, res, ctx.lookup);
            if let Some(explain) = l.explain {
                let selected = self.results.selected;
                render_explain(
                    &self.copy_citation,
                    self.copy_rect(explain),
                    c,
                    explain,
                    theme,
                    res,
                    selected,
                    ctx.lookup,
                );
            }
        }
    }

    /// The open dropdown draws above everything, on the overlay layer.
    pub fn render_dropdown(
        &self,
        c: &mut Compositor,
        layer: LayerId,
        content: Rect,
        theme: &Theme,
    ) {
        if self.mode.is_open() {
            let l = self.layout(content, self.result.is_some());
            self.mode.render_dropdown(c, layer, l.mode_select, theme);
        }
    }
}

/// Results list rows: score bar + value, short id, first text line, and
/// the span (`frame N` for media chunks).
fn render_results(
    results: &mut VirtualList,
    c: &mut Compositor,
    bounds: Rect,
    theme: &Theme,
    res: &SearchResultsView,
    lookup: &ChunkLookup,
) {
    let sub_style = TextStyle::new(12.0);
    let title_style = TextStyle::new(13.0).with_weight(500);
    results.render_with(c, bounds, theme, |c, i, row, _hov, _sel| {
        let Some(hit) = res.hits.get(i) else {
            return;
        };
        let pad = 12.0;
        // Score meter (engine charts helper) + numeric value.
        let bar_w = 64.0;
        engine::charts::draw::meter(
            c,
            hit.score,
            Rect::new(row.x + pad, row.y + 10.0, bar_w, 6.0),
            theme,
        );
        text(
            c,
            &format!("{:.4}", hit.score),
            12.0,
            500,
            row.x + pad + bar_w + 8.0,
            row.y + 6.0,
            theme.colors.text.0,
        );
        // First line: the chunk's first text line (card name…), falling
        // back to the short id while texts load.
        let ordinal = lookup.ordinal(&hit.chunk_id);
        let title = ordinal
            .and_then(|o| lookup.first_line(o))
            .map(str::to_string)
            .unwrap_or_else(|| short_id(&hit.chunk_id));
        let title_x = row.x + pad + bar_w + 72.0;
        let title = TextMeasurer::truncate_to_width(&title, &title_style, row.w - (title_x - row.x) - pad);
        text(
            c,
            &title,
            13.0,
            500,
            title_x,
            row.y + 6.0,
            theme.colors.text_mid.0,
        );
        // Second line: short id · span.
        let media = ordinal.is_some_and(|o| lookup.is_media(o));
        let sub = format!(
            "{} · {}",
            short_id(&hit.chunk_id),
            span_label(&hit.source_uri, hit.offset_start, hit.offset_end, media)
        );
        let sub = TextMeasurer::truncate_to_width(&sub, &sub_style, row.w - pad * 2.0);
        text(
            c,
            &sub,
            12.0,
            400,
            row.x + pad,
            row.y + 30.0,
            theme.colors.text_dim.0,
        );
    });
}

/// The explain panel: route, candidate counts, recall and the
/// rerank-source honesty line, plus the selected hit's citation, frame
/// preview (media corpora) and canonical text.
#[allow(clippy::too_many_arguments)]
fn render_explain(
    copy_citation: &IconButton,
    copy_rect: Rect,
    c: &mut Compositor,
    rect: Rect,
    theme: &Theme,
    res: &SearchResultsView,
    selected: Option<usize>,
    lookup: &ChunkLookup,
) {
    panel(c, rect, theme);
    group_label(c, "EXPLAIN", rect.x + 16.0, rect.y + 16.0, theme);

    let recall = if res.recall.is_nan() {
        "not computed (never claimed)".to_string()
    } else {
        format!("{:.3}", res.recall)
    };
    let mut rows = vec![
        ("index_type", res.index_type.clone()),
        ("route", res.route.clone()),
        ("query_time", format!("{:.3} ms", res.query_time_ms)),
        ("k", format!("{}/{}", res.k_returned, res.k_requested)),
        ("truncated", res.truncated.to_string()),
        ("recall", recall),
    ];
    if res.exact_candidates > 0 {
        rows.push(("exact candidates", res.exact_candidates.to_string()));
    }
    if res.ann_candidates > 0 {
        rows.push(("ann candidates", res.ann_candidates.to_string()));
    }
    if res.bm25_candidates > 0 {
        rows.push(("bm25 candidates", res.bm25_candidates.to_string()));
    }
    if res.graph_candidates > 0 {
        rows.push(("graph candidates", res.graph_candidates.to_string()));
    }
    if res.fusion_mode != "none" {
        rows.push(("fusion", res.fusion_mode.clone()));
    }
    rows.push(("scores", res.rerank_disclosure.clone()));

    let inner_w = rect.w - 32.0;
    let mut y = rect.y + 16.0 + 24.0;
    for (key, value) in rows {
        text(c, key, 12.0, 600, rect.x + 16.0, y, theme.colors.text_dim.0);
        let style = TextStyle::new(12.0);
        let v = TextMeasurer::truncate_to_width(&value, &style, inner_w - 128.0);
        text(
            c,
            &v,
            12.0,
            400,
            rect.x + 16.0 + 128.0,
            y,
            theme.colors.text_mid.0,
        );
        y += 22.0;
    }

    // Selected hit: citation (copyable), frame, canonical text.
    let Some(hit) = selected.and_then(|sel| res.hits.get(sel)) else {
        return;
    };
    y += 12.0;
    group_label(c, "SELECTED", rect.x + 16.0, y, theme);
    copy_citation.render(c, copy_rect, theme);
    let style = TextStyle::new(12.0);
    let id = TextMeasurer::truncate_to_width(&hit.citation_id, &style, inner_w);
    text(
        c,
        &id,
        12.0,
        400,
        rect.x + 16.0,
        y + 20.0,
        theme.colors.text_mid.0,
    );
    y += 20.0 + 24.0;

    let Some(ordinal) = lookup.ordinal(&hit.chunk_id) else {
        return;
    };
    if lookup.is_media(ordinal) {
        y += render_frame_box(c, Rect::new(rect.x + 16.0, y, inner_w, FRAME_H), theme, lookup, ordinal);
        y += GAP;
    }
    if let Some(full) = lookup.text(ordinal) {
        let area = Rect::new(
            rect.x + 16.0,
            y,
            inner_w,
            (rect.y + rect.h - 16.0 - y).max(0.0),
        );
        if area.h > 16.0 {
            let body = TextStyle::new(13.0).with_line_height(13.0 * 1.5);
            c.push(SceneNode::PushClip {
                x: area.x,
                y: area.y,
                w: area.w,
                h: area.h,
            });
            c.push(SceneNode::Text {
                key: TextNodeKey::from_style(full, &body, Some(area.w)),
                x: area.x,
                y: area.y,
                color: theme.colors.text_mid.0,
            });
            c.push(SceneNode::PopClip);
        }
    }
}

/// Draw the decoded frame of `ordinal` fitted inside `bounds` (or the
/// loading / failure note). Returns the height used.
pub(crate) fn render_frame_box(
    c: &mut Compositor,
    bounds: Rect,
    theme: &Theme,
    lookup: &ChunkLookup,
    ordinal: usize,
) -> f32 {
    match lookup.frame(ordinal) {
        Some(Ok(handle)) => {
            // Fit the atlas image inside the box, keep aspect, centered.
            let (iw, ih) = (handle.width as f32, handle.height as f32);
            let scale = (bounds.w / iw).min(bounds.h / ih).min(1.0);
            let (w, h) = (iw * scale, ih * scale);
            c.draw_image(
                bounds.x + (bounds.w - w) / 2.0,
                bounds.y,
                w,
                h,
                *handle,
                theme.radius.sm,
            );
            h
        }
        Some(Err(reason)) => {
            let style = TextStyle::new(12.0);
            let msg = TextMeasurer::truncate_to_width(
                &format!("no frame preview: {reason}"),
                &style,
                bounds.w,
            );
            text(c, &msg, 12.0, 400, bounds.x, bounds.y, theme.colors.text_dim.0);
            20.0
        }
        None => {
            text(
                c,
                "decoding frame…",
                12.0,
                400,
                bounds.x,
                bounds.y,
                theme.colors.text_dim.0,
            );
            20.0
        }
    }
}

/// Rects the screen needs for hit testing and drawing, from [`layout`].
struct Layout {
    kind_tabs: Rect,
    query_field: Rect,
    search_button: Rect,
    mode_select: Rect,
    slider: Rect,
    status_y: f32,
    results: Rect,
    explain: Option<Rect>,
}

// ---------------------------------------------------------------------------
// Headless search tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::types::SearchHitView;
    use crate::view::fixtures;
    use std::collections::HashMap;

    fn harness() -> (SearchScreen, Theme) {
        let theme = Theme::hoff();
        (SearchScreen::new(&theme), theme)
    }

    fn fake_results() -> SearchResultsView {
        SearchResultsView {
            hits: vec![SearchHitView {
                chunk_id: format!("sha256:{}{}", 0, "0".repeat(63)),
                score: 0.91,
                source_uri: "corpus.txt".into(),
                offset_start: 0,
                offset_end: 16,
                citation_id: format!("urna://{}/chunk", "2".repeat(64)),
                reranked: true,
            }],
            query_time_ms: 1.5,
            index_type: "hnsw".into(),
            recall: f32::NAN,
            truncated: true,
            k_requested: 10,
            k_returned: 1,
            route: "hnsw".into(),
            exact_candidates: 0,
            ann_candidates: 40,
            bm25_candidates: 0,
            graph_candidates: 0,
            fusion_mode: "none".into(),
            rerank_disclosure: "real cosine".into(),
            recall_estimate: f32::NAN,
        }
    }

    #[test]
    fn typing_then_enter_submits_a_text_search() {
        let (mut screen, _) = harness();
        let content = Rect::new(40.0, 128.0, 1200.0, 600.0);
        // Unfocused fields swallow nothing.
        assert!(!screen.handle_text("hello"));
        // Click the query field to focus it.
        let field = screen.layout(content, false).query_field;
        screen.handle_event(
            &WidgetEvent::MouseDown {
                x: field.x + 20.0,
                y: field.y + 10.0,
            },
            content,
            true,
        );
        assert!(screen.handle_text("hello corpus"));
        let (handled, action) = screen.handle_edit_key(EditKey::Enter);
        assert!(handled);
        match action {
            Action::RunSearch {
                query,
                is_vector,
                mode,
                k,
            } => {
                assert_eq!(query, "hello corpus");
                assert!(!is_vector);
                assert_eq!(mode, SearchMode::Exact);
                assert_eq!(k, 10);
            }
            other => panic!("expected RunSearch, got {other:?}"),
        }
        assert!(screen.pending);
    }

    #[test]
    fn kind_toggle_switches_to_vector_mode() {
        let (mut screen, _) = harness();
        let content = Rect::new(40.0, 128.0, 1200.0, 600.0);
        assert_eq!(screen.query_kind(), QueryKind::Text);
        let tabs = screen.layout(content, false).kind_tabs;
        // Second segment ("vector").
        let rect = screen.kind_tabs.item_rects(tabs)[1];
        let (x, y) = rect.center();
        let (r, _) = screen.handle_event(&WidgetEvent::MouseDown { x, y }, content, true);
        assert!(r.clicked);
        assert_eq!(screen.query_kind(), QueryKind::Vector);
        assert!(screen.query.input.placeholder.starts_with('['));
    }

    #[test]
    fn mode_select_maps_to_search_modes() {
        let (mut screen, _) = harness();
        screen.mode.selected = 1;
        assert!(matches!(screen.selected_mode(), SearchMode::Ann { .. }));
        screen.mode.selected = 2;
        assert!(matches!(screen.selected_mode(), SearchMode::Graph { .. }));
        screen.mode.selected = 3;
        assert!(matches!(screen.selected_mode(), SearchMode::Hybrid { .. }));
    }

    #[test]
    fn spaces_append_to_the_mode_select() {
        let (mut screen, _) = harness();
        screen.reset(&fixtures::fake_media_db());
        assert_eq!(screen.mode.options.len(), TEXT_MODES.len() + 1);
        assert_eq!(screen.mode.options[4], "space: clip-vit-b32");
        screen.mode.selected = 4;
        assert_eq!(
            screen.selected_mode(),
            SearchMode::Space {
                name: "clip-vit-b32".into()
            }
        );
        // A text-only db drops them again.
        screen.reset(&fixtures::fake_db());
        assert_eq!(screen.mode.options.len(), TEXT_MODES.len());
        assert_eq!(screen.mode.selected, 0);
    }

    #[test]
    fn fold_result_updates_state_and_renders_at_two_widths() {
        let (mut screen, theme) = harness();
        screen.pending = true;
        screen.fold_result(Err("embedder exploded".to_string()));
        assert!(!screen.pending);
        assert_eq!(screen.error, "embedder exploded");

        screen.fold_result(Ok(fake_results()));
        assert!(screen.error.is_empty());
        assert_eq!(screen.result.as_ref().unwrap().hits.len(), 1);

        let db = fixtures::fake_media_db();
        let chunks = fixtures::fake_media_chunks();
        let index: HashMap<String, usize> = db
            .chunk_ids
            .iter()
            .enumerate()
            .map(|(i, id)| (id.clone(), i))
            .collect();
        let mut frames = HashMap::new();
        frames.insert(0usize, Err("no ffmpeg".to_string()));
        let lookup = ChunkLookup {
            ids: &db.chunk_ids,
            index: Some(&index),
            chunks: Some(&chunks),
            frames: &frames,
        };
        screen.results.selected = Some(0);
        assert_eq!(screen.wanted_frame(&lookup), None, "a failed decode is not re-requested");
        for (w, h) in [(800.0, 600.0), (1600.0, 1000.0)] {
            let mut c = Compositor::new();
            screen.render(
                &mut c,
                Rect::new(40.0, 128.0, w - 80.0, h - 168.0),
                &theme,
                &SearchContext {
                    dim: 4,
                    lookup: &lookup,
                    embedder: None,
                },
            );
        }
    }

    #[test]
    fn selecting_a_media_hit_wants_its_frame() {
        let (mut screen, _) = harness();
        screen.fold_result(Ok(fake_results()));
        let db = fixtures::fake_media_db();
        let chunks = fixtures::fake_media_chunks();
        let index: HashMap<String, usize> = db
            .chunk_ids
            .iter()
            .enumerate()
            .map(|(i, id)| (id.clone(), i))
            .collect();
        let frames = HashMap::new();
        let lookup = ChunkLookup {
            ids: &db.chunk_ids,
            index: Some(&index),
            chunks: Some(&chunks),
            frames: &frames,
        };
        assert_eq!(screen.wanted_frame(&lookup), None);
        screen.results.selected = Some(0);
        assert_eq!(screen.wanted_frame(&lookup), Some(0));
        // Text-only chunks never ask for a frame.
        let text_chunks = fixtures::fake_chunks();
        let lookup = ChunkLookup {
            chunks: Some(&text_chunks),
            ..lookup
        };
        assert_eq!(screen.wanted_frame(&lookup), None);
    }

    #[test]
    fn selecting_a_hit_enables_citation_copy() {
        let (mut screen, _) = harness();
        screen.fold_result(Ok(fake_results()));
        let content = Rect::new(40.0, 128.0, 1200.0, 600.0);
        // Click the first result row.
        let results = screen.layout(content, true).results;
        screen.handle_event(
            &WidgetEvent::MouseDown {
                x: results.x + 20.0,
                y: results.y + 10.0,
            },
            content,
            true,
        );
        assert_eq!(screen.results.selected, Some(0));
        // Click the copy-citation button in the explain panel.
        let explain = screen.layout(content, true).explain.unwrap();
        let rect = screen.copy_rect(explain);
        let (x, y) = rect.center();
        screen.handle_event(&WidgetEvent::MouseDown { x, y }, content, true);
        let (r, action) = screen.handle_event(&WidgetEvent::MouseUp { x, y }, content, true);
        assert!(r.clicked);
        match action {
            Action::Copy { text, .. } => assert!(text.starts_with("urna://")),
            other => panic!("expected copy action, got {other:?}"),
        }
    }
}
