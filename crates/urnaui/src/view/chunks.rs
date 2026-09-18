//! CHUNKS screen: a filter field (text substring, chunk id, or a
//! `urna://` citation), the virtualized chunk list (index, short id, text
//! preview) and a detail panel for the selected chunk — full id and
//! citation (copyable), source span (`frame N` for media chunks), the
//! decoded frame when the corpus inlines its media, and the full
//! canonical text in a scrollable viewport.
//!
//! Data flows in from the shell through [`ChunksContext`]; the screen owns
//! widget state and the filter's index list only. Nothing here re-reads
//! the file.

use engine::compositor::{Compositor, SceneNode, TextNodeKey};
use engine::input::scroll::ScrollState;
use engine::text::{TextMeasurer, TextStyle};
use engine::theme::Theme;
use engine::ui::widgets::{
    EventResult, IconButton, Rect, Scrollbar, Spinner, SpinnerSize, VirtualList, WidgetEvent,
};

use super::field::{FIELD_H, Field};
use super::search::render_frame_box;
use super::{
    Action, ChunkLookup, EditKey, group_label, panel, parse_citation, short_id, span_label, text,
};

const ROW_H: f32 = 48.0;
const GAP: f32 = 16.0;
const DETAIL_PAD: f32 = 16.0;
/// Frame preview box inside the detail panel.
const FRAME_H: f32 = 220.0;

/// Everything the Chunks screen needs from the central view state.
pub struct ChunksContext<'a> {
    pub lookup: &'a ChunkLookup<'a>,
    /// The open file's content_hash: a pasted citation must carry it to
    /// resolve here (a citation from another build never matches).
    pub content_hash: &'a str,
    /// The first text decode is in flight.
    pub loading: bool,
}

pub struct ChunksScreen {
    filter: Field,
    /// Ordinals matching the filter, in file order (`None` = no filter,
    /// every chunk).
    filtered: Option<Vec<usize>>,
    /// The filter text the `filtered` list was built from.
    filter_key: String,
    /// One-line note under the field (citation mismatch, no match…).
    filter_note: String,
    list: VirtualList,
    detail_scroll: ScrollState,
    detail_scrollbar: Scrollbar,
    copy_id: IconButton,
    copy_citation: IconButton,
    spinner: Spinner,
    loading: bool,
}

impl ChunksScreen {
    pub fn new(theme: &Theme) -> Self {
        let ghost = || IconButton::new("copy").variant(engine::ui::widgets::ButtonVariant::Ghost);
        Self {
            filter: Field::new("filter by text, chunk id or urna:// citation", theme),
            filtered: None,
            filter_key: String::new(),
            filter_note: String::new(),
            list: VirtualList::new(ROW_H),
            detail_scroll: ScrollState::new(),
            detail_scrollbar: Scrollbar::new(),
            copy_id: ghost(),
            copy_citation: ghost(),
            spinner: Spinner::new().size(SpinnerSize::Sm),
            loading: false,
        }
    }

    /// Reset per-database state (called when a new db opens).
    pub fn reset(&mut self) {
        self.list.selected = None;
        self.list.set_item_count(0);
        self.detail_scroll = ScrollState::new();
        self.filtered = None;
        self.filter_key.clear();
        self.filter_note.clear();
    }

    /// The selected chunk's ordinal (the list index maps through the
    /// filter).
    pub fn selected_ordinal(&self) -> Option<usize> {
        let row = self.list.selected?;
        match &self.filtered {
            Some(f) => f.get(row).copied(),
            None => Some(row),
        }
    }

    fn row_count(&self, total: usize) -> usize {
        self.filtered.as_ref().map_or(total, Vec::len)
    }

    /// The frame to decode for the selected chunk, when it is a media
    /// chunk whose frame is not loaded yet.
    pub fn wanted_frame(&self, lookup: &ChunkLookup) -> Option<usize> {
        let ordinal = self.selected_ordinal()?;
        (lookup.is_media(ordinal) && lookup.frame(ordinal).is_none()).then_some(ordinal)
    }

    /// Rebuild the filtered ordinal list when the filter text changed or
    /// the texts arrived. Three shapes, tried in order: a `urna://`
    /// citation (must match the open file's content_hash), an exact chunk
    /// id, case-insensitive terms that must all occur in the canonical
    /// text.
    fn sync_filter(&mut self, ctx: &ChunksContext) {
        let key = self.filter.text().trim().to_string();
        let texts_ready = ctx.lookup.chunks.is_some();
        // Rebuild when the key changed, or when a text filter waited for
        // the decode.
        let stale = key != self.filter_key
            || (self.filtered.is_none()
                && !key.is_empty()
                && texts_ready
                && self.filter_note == "loading texts…");
        if !stale {
            return;
        }
        self.filter_key = key.clone();
        self.filter_note.clear();
        self.list.selected = None;
        self.detail_scroll = ScrollState::new();
        if key.is_empty() {
            self.filtered = None;
            return;
        }
        if let Some((content_hash, chunk_id)) = parse_citation(&key) {
            if content_hash != ctx.content_hash {
                self.filter_note = "citation points at another build (content_hash differs)".into();
                self.filtered = Some(Vec::new());
                return;
            }
            self.filtered = Some(self.match_id(ctx, chunk_id));
            if self.filtered.as_ref().is_some_and(Vec::is_empty) {
                self.filter_note = "chunk id not in this file".into();
            }
            return;
        }
        if key.starts_with("sha256:") {
            let hits = self.match_id(ctx, &key);
            if !hits.is_empty() {
                self.filtered = Some(hits);
                return;
            }
        }
        let Some(chunks) = ctx.lookup.chunks else {
            self.filtered = None;
            self.filter_note = "loading texts…".into();
            return;
        };
        // Every whitespace-separated term must appear (any order), so
        // "lifelink angel" finds an angel with lifelink.
        let terms: Vec<String> = key.split_whitespace().map(str::to_lowercase).collect();
        let hits: Vec<usize> = chunks
            .texts
            .iter()
            .enumerate()
            .filter(|(_, t)| {
                let lower = t.to_lowercase();
                terms.iter().all(|term| lower.contains(term.as_str()))
            })
            .map(|(i, _)| i)
            .collect();
        if hits.is_empty() {
            self.filter_note = "no chunk matches".into();
        }
        self.filtered = Some(hits);
    }

    /// Ordinals whose id equals `id`, or starts with it when `id` is a
    /// prefix of at least the short form (12 hex chars after `sha256:`).
    fn match_id(&self, ctx: &ChunksContext, id: &str) -> Vec<usize> {
        let index = ctx.lookup.index;
        if let Some(i) = index.and_then(|ix| ix.get(id)) {
            return vec![*i];
        }
        let hex = id.strip_prefix("sha256:").unwrap_or(id);
        if hex.len() < 12 {
            return Vec::new();
        }
        ctx.lookup
            .ids
            .iter()
            .enumerate()
            .filter(|(_, full)| {
                full.strip_prefix("sha256:")
                    .unwrap_or(full)
                    .starts_with(hex)
            })
            .map(|(i, _)| i)
            .collect()
    }

    /// Field rect, list rect + detail rect (when a chunk is selected and
    /// wide enough; below ~720px of content width the detail replaces the
    /// list).
    fn layout(&self, content: Rect) -> Layout {
        let field = Rect::new(content.x, content.y, content.w, FIELD_H);
        let note_y = field.y + field.h + 6.0;
        let body = Rect::new(
            content.x,
            note_y + 22.0,
            content.w,
            (content.y + content.h - note_y - 22.0).max(80.0),
        );
        if self.list.selected.is_none() {
            return Layout {
                field,
                note_y,
                list: body,
                detail: None,
            };
        }
        if content.w < 720.0 {
            return Layout {
                field,
                note_y,
                list: Rect::new(body.x, body.y, 0.0, 0.0),
                detail: Some(body),
            };
        }
        let detail_w = (content.w * 0.4).clamp(300.0, 440.0);
        let list = Rect::new(body.x, body.y, body.w - detail_w - GAP, body.h);
        let detail = Rect::new(list.x + list.w + GAP, body.y, detail_w, body.h);
        Layout {
            field,
            note_y,
            list,
            detail: Some(detail),
        }
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        ctx: &ChunksContext,
    ) -> (EventResult, Action) {
        self.sync_filter(ctx);
        self.list
            .set_item_count(self.row_count(ctx.lookup.ids.len()));
        let l = self.layout(content);

        // Filter field: click focuses; clicks elsewhere blur.
        if let WidgetEvent::MouseDown { x, y } = *event {
            if l.field.contains(x, y) {
                self.filter.click(x - l.field.x);
                return (EventResult::changed(), Action::None);
            }
            if self.filter.input.focused {
                self.filter.unfocus();
            }
        }

        // Buttons + scroll inside the detail panel.
        if let (Some(detail), Some(ordinal)) = (l.detail, self.selected_ordinal()) {
            let r = self.copy_id.handle_event(event, self.copy_rect(detail, 0));
            if r.clicked
                && let Some(id) = ctx.lookup.ids.get(ordinal)
            {
                return (
                    r,
                    Action::Copy {
                        text: id.clone(),
                        what: "chunk id".to_string(),
                    },
                );
            }
            let r = self
                .copy_citation
                .handle_event(event, self.copy_rect(detail, 1));
            if r.clicked
                && let Some(id) = ctx.lookup.ids.get(ordinal)
            {
                return (
                    r,
                    Action::Copy {
                        text: format!("urna://{}/{id}", ctx.content_hash),
                        what: "citation".to_string(),
                    },
                );
            }
            if let WidgetEvent::Scroll { x, y, delta } = *event
                && self
                    .text_area(detail, ctx.lookup.is_media(ordinal))
                    .contains(x, y)
            {
                let old = self.detail_scroll.offset();
                self.detail_scroll.scroll_by(delta);
                self.detail_scrollbar.notify_scroll();
                if self.detail_scroll.offset() != old {
                    return (EventResult::changed(), Action::None);
                }
            }
        }

        // Selection change resets the detail scroll.
        let before = self.list.selected;
        let r = self.list.handle_event(event, l.list);
        if self.list.selected != before {
            self.detail_scroll = ScrollState::new();
        }
        (r, Action::None)
    }

    pub fn handle_text(&mut self, s: &str) -> bool {
        self.filter.insert(s)
    }

    pub fn handle_edit_key(&mut self, key: EditKey) -> bool {
        self.filter.edit(key)
    }

    /// Copy buttons stack at the top right of the detail: `slot` 0 is
    /// the chunk id, 1 the citation.
    fn copy_rect(&self, detail: Rect, slot: usize) -> Rect {
        let (w, h) = self.copy_id.preferred_size();
        Rect::new(
            detail.x + detail.w - DETAIL_PAD - w,
            detail.y + DETAIL_PAD - 4.0 + slot as f32 * 26.0,
            w,
            h,
        )
    }

    /// Scrollable canonical-text viewport inside the detail panel: below
    /// the header rows (id, citation, source, offsets) and the frame box
    /// when the chunk is a media one.
    fn text_area(&self, detail: Rect, media: bool) -> Rect {
        let mut top = DETAIL_PAD + 24.0 + 26.0 * 4.0 + 8.0;
        if media {
            top += FRAME_H + GAP;
        }
        Rect::new(
            detail.x + DETAIL_PAD,
            detail.y + top,
            detail.w - DETAIL_PAD * 2.0,
            (detail.h - top - DETAIL_PAD).max(40.0),
        )
    }

    pub fn tick(&mut self, dt: f32) -> bool {
        let spinning = self.loading && self.spinner.tick(dt);
        self.filter.tick(dt) | self.list.tick(dt) | self.detail_scrollbar.tick(dt) | spinning
    }

    pub fn render(
        &mut self,
        c: &mut Compositor,
        content: Rect,
        theme: &Theme,
        ctx: &ChunksContext,
    ) {
        self.sync_filter(ctx);
        let total = ctx.lookup.ids.len();
        self.list.set_item_count(self.row_count(total));
        self.loading = ctx.loading;
        let l = self.layout(content);

        self.filter.render(c, l.field, theme);
        let note = if !self.filter_note.is_empty() {
            self.filter_note.clone()
        } else if let Some(f) = &self.filtered {
            format!("{} of {total} chunks", f.len())
        } else {
            format!("{total} chunks")
        };
        text(
            c,
            &note,
            12.0,
            400,
            content.x,
            l.note_y,
            theme.colors.text_dim.0,
        );

        // While the first decode is in flight, center a spinner + note
        // over the list area.
        if ctx.loading && ctx.lookup.chunks.is_none() && l.list.w > 0.0 {
            self.spinner.render(
                c,
                Rect::new(
                    l.list.x + (l.list.w - 24.0) / 2.0,
                    l.list.y + 60.0,
                    24.0,
                    24.0,
                ),
                theme,
            );
            text(
                c,
                "loading chunk texts…",
                13.0,
                400,
                l.list.x + (l.list.w - 24.0) / 2.0 - 56.0,
                l.list.y + 96.0,
                theme.colors.text_dim.0,
            );
        }

        if l.list.w > 0.0 {
            let preview_style = TextStyle::new(12.0);
            let filtered = self.filtered.as_deref();
            let loading = ctx.loading;
            let lookup = ctx.lookup;
            self.list
                .render_with(c, l.list, theme, |c, row, rect, _hov, _sel| {
                    let i = filtered.map_or(row, |f| f.get(row).copied().unwrap_or(row));
                    let pad = 12.0;
                    text(
                        c,
                        &format!("#{i}"),
                        11.0,
                        400,
                        rect.x + pad,
                        rect.y + 7.0,
                        theme.colors.text_dim.0,
                    );
                    let id = short_id(lookup.ids.get(i).map(String::as_str).unwrap_or(""));
                    text(
                        c,
                        &id,
                        13.0,
                        600,
                        rect.x + pad + 64.0,
                        rect.y + 6.0,
                        theme.colors.text.0,
                    );
                    let preview = match (lookup.first_line(i), loading) {
                        (Some(p), _) => TextMeasurer::truncate_to_width(
                            p,
                            &preview_style,
                            rect.w - pad * 2.0 - 64.0,
                        ),
                        (None, true) => "loading texts…".to_string(),
                        (None, false) => String::new(),
                    };
                    text(
                        c,
                        &preview,
                        12.0,
                        400,
                        rect.x + pad + 64.0,
                        rect.y + 26.0,
                        theme.colors.text_dim.0,
                    );
                });
        }

        if let (Some(detail), Some(ordinal)) = (l.detail, self.selected_ordinal()) {
            self.render_detail(c, detail, theme, ctx, ordinal);
        }
    }

    fn render_detail(
        &mut self,
        c: &mut Compositor,
        detail: Rect,
        theme: &Theme,
        ctx: &ChunksContext,
        ordinal: usize,
    ) {
        panel(c, detail, theme);
        group_label(
            c,
            &format!("CHUNK #{ordinal}"),
            detail.x + DETAIL_PAD,
            detail.y + DETAIL_PAD,
            theme,
        );
        self.copy_id.render(c, self.copy_rect(detail, 0), theme);
        self.copy_citation
            .render(c, self.copy_rect(detail, 1), theme);

        let id = ctx
            .lookup
            .ids
            .get(ordinal)
            .map(String::as_str)
            .unwrap_or("");
        let line_w = detail.w - DETAIL_PAD * 2.0 - self.copy_rect(detail, 0).w - 8.0;
        let id_style = TextStyle::new(13.0).with_weight(500);
        let meta_style = TextStyle::new(12.0);
        let mut y = detail.y + DETAIL_PAD + 24.0;
        text(
            c,
            &TextMeasurer::truncate_to_width(id, &id_style, line_w),
            13.0,
            500,
            detail.x + DETAIL_PAD,
            y,
            theme.colors.text.0,
        );
        y += 26.0;
        let citation = format!("urna://{}/{id}", ctx.content_hash);
        text(
            c,
            &TextMeasurer::truncate_to_width(&citation, &meta_style, line_w),
            12.0,
            400,
            detail.x + DETAIL_PAD,
            y,
            theme.colors.text_mid.0,
        );
        y += 26.0;

        let media = ctx.lookup.is_media(ordinal);
        let (uri, offsets) = match ctx.lookup.meta(ordinal) {
            Some(m) => (
                span_label(&m.source_uri, m.offset_start, m.offset_end, media),
                if media {
                    format!("blob {} · overlay span", m.blob.map_or(0, |b| b.blob_index))
                } else {
                    format!("bytes {}–{}", m.offset_start, m.offset_end)
                },
            ),
            None => ("(spans not loaded)".to_string(), String::new()),
        };
        text(
            c,
            &TextMeasurer::truncate_to_width(&uri, &meta_style, detail.w - DETAIL_PAD * 2.0),
            12.0,
            400,
            detail.x + DETAIL_PAD,
            y,
            theme.colors.text_mid.0,
        );
        y += 26.0;
        text(
            c,
            &offsets,
            12.0,
            400,
            detail.x + DETAIL_PAD,
            y,
            theme.colors.text_dim.0,
        );
        y += 26.0 + 8.0;

        if media {
            render_frame_box(
                c,
                Rect::new(
                    detail.x + DETAIL_PAD,
                    y,
                    detail.w - DETAIL_PAD * 2.0,
                    FRAME_H,
                ),
                theme,
                ctx.lookup,
                ordinal,
            );
        }

        // Canonical text, wrapped + scrolled inside its viewport.
        let area = self.text_area(detail, media);
        let style = TextStyle::new(13.0).with_line_height(13.0 * 1.5);
        if let Some(full) = ctx.lookup.text(ordinal) {
            let text_h = TextMeasurer::measure_styled(full, &style, Some(area.w)).1;
            self.detail_scroll.set_viewport(area.h);
            self.detail_scroll.set_content(text_h);
            c.push(SceneNode::PushClip {
                x: area.x,
                y: area.y,
                w: area.w,
                h: area.h,
            });
            c.push(SceneNode::Text {
                key: TextNodeKey::from_style(full, &style, Some(area.w)),
                x: area.x,
                y: area.y - self.detail_scroll.offset(),
                color: theme.colors.text_mid.0,
            });
            c.push(SceneNode::PopClip);
            let mut scratch = Vec::new();
            self.detail_scrollbar
                .render_nodes(&mut scratch, area, &self.detail_scroll, theme);
            for node in scratch {
                c.push(node);
            }
        }
    }
}

struct Layout {
    field: Rect,
    note_y: f32,
    list: Rect,
    detail: Option<Rect>,
}

// ---------------------------------------------------------------------------
// Headless chunks tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::types::{ChunksData, OpenedDbView};
    use crate::view::fixtures;
    use std::collections::HashMap;

    struct Fix {
        db: Box<OpenedDbView>,
        chunks: ChunksData,
        index: HashMap<String, usize>,
        frames: HashMap<usize, Result<engine::gpu::image::ImageHandle, String>>,
    }

    impl Fix {
        fn text() -> Self {
            Self::from(fixtures::fake_db(), fixtures::fake_chunks())
        }

        fn media() -> Self {
            Self::from(fixtures::fake_media_db(), fixtures::fake_media_chunks())
        }

        fn from(db: Box<OpenedDbView>, chunks: ChunksData) -> Self {
            let index = db
                .chunk_ids
                .iter()
                .enumerate()
                .map(|(i, id)| (id.clone(), i))
                .collect();
            Self {
                db,
                chunks,
                index,
                frames: HashMap::new(),
            }
        }

        fn lookup(&self) -> ChunkLookup<'_> {
            ChunkLookup {
                ids: &self.db.chunk_ids,
                index: Some(&self.index),
                chunks: Some(&self.chunks),
                frames: &self.frames,
            }
        }
    }

    fn ctx<'a>(lookup: &'a ChunkLookup<'a>, fix: &'a Fix) -> ChunksContext<'a> {
        ChunksContext {
            lookup,
            content_hash: &fix.db.inspect.content_hash,
            loading: false,
        }
    }

    fn harness() -> (ChunksScreen, Theme) {
        let theme = Theme::hoff();
        (ChunksScreen::new(&theme), theme)
    }

    #[test]
    fn clicking_a_row_selects_it_and_shows_the_detail() {
        let fix = Fix::text();
        let lookup = fix.lookup();
        let ctx = ctx(&lookup, &fix);
        let (mut screen, theme) = harness();
        let content = Rect::new(40.0, 128.0, 1200.0, 600.0);

        assert_eq!(screen.list.selected, None);
        let list = screen.layout(content).list;
        let (r, _) = screen.handle_event(
            &WidgetEvent::MouseDown {
                x: list.x + 20.0,
                y: list.y + 10.0,
            },
            content,
            &ctx,
        );
        assert!(r.clicked);
        assert_eq!(screen.selected_ordinal(), Some(0));

        // Wide: list + detail side by side.
        let l = screen.layout(content);
        assert!(l.list.w > 0.0);
        assert!(l.detail.is_some());

        let mut c = Compositor::new();
        screen.render(&mut c, content, &theme, &ctx);
    }

    #[test]
    fn narrow_viewports_replace_the_list_with_the_detail() {
        let (mut screen, _) = harness();
        screen.list.selected = Some(1);
        let narrow = Rect::new(40.0, 128.0, 600.0, 400.0);
        let l = screen.layout(narrow);
        assert_eq!(l.list.w, 0.0, "no room for both: the detail wins");
        assert_eq!(l.detail.map(|d| d.w), Some(narrow.w));
    }

    #[test]
    fn copy_buttons_copy_the_id_and_the_citation() {
        let fix = Fix::text();
        let lookup = fix.lookup();
        let ctx = ctx(&lookup, &fix);
        let (mut screen, _) = harness();
        screen.list.selected = Some(2);
        let content = Rect::new(40.0, 128.0, 1200.0, 600.0);
        let detail = screen.layout(content).detail.unwrap();
        for (slot, expect_prefix) in [(0, "sha256:"), (1, "urna://")] {
            let (x, y) = screen.copy_rect(detail, slot).center();
            screen.handle_event(&WidgetEvent::MouseDown { x, y }, content, &ctx);
            let (r, action) = screen.handle_event(&WidgetEvent::MouseUp { x, y }, content, &ctx);
            assert!(r.clicked);
            match action {
                Action::Copy { text, .. } => {
                    assert!(text.starts_with(expect_prefix));
                    assert!(text.ends_with(&fix.db.chunk_ids[2]));
                }
                other => panic!("expected copy action, got {other:?}"),
            }
        }
    }

    #[test]
    fn filter_matches_text_id_and_citation() {
        let fix = Fix::text();
        let lookup = fix.lookup();
        let ctx = ctx(&lookup, &fix);
        let (mut screen, _) = harness();

        // terms, case-insensitive, any order, all required
        screen.filter.input.focused = true;
        screen.filter.insert("BETA");
        screen.sync_filter(&ctx);
        assert_eq!(screen.filtered, Some(vec![1]));
        screen.filter.input.buffer.set_text("");
        screen.filter.insert("text chunk");
        screen.sync_filter(&ctx);
        assert_eq!(screen.filtered, Some(vec![0, 1, 2]));
        screen.filter.input.buffer.set_text("");
        screen.filter.insert("chunk delta");
        screen.sync_filter(&ctx);
        assert_eq!(screen.filtered, Some(vec![]));

        // exact chunk id
        screen.filter.input.buffer.set_text("");
        screen.filter.insert(&fix.db.chunk_ids[2]);
        screen.sync_filter(&ctx);
        assert_eq!(screen.filtered, Some(vec![2]));

        // citation with the file's content_hash
        screen.filter.input.buffer.set_text("");
        screen.filter.insert(&format!(
            "urna://{}/{}",
            fix.db.inspect.content_hash, fix.db.chunk_ids[0]
        ));
        screen.sync_filter(&ctx);
        assert_eq!(screen.filtered, Some(vec![0]));

        // citation from another build
        screen.filter.input.buffer.set_text("");
        screen.filter.insert(&format!(
            "urna://sha256:{}/{}",
            "9".repeat(64),
            fix.db.chunk_ids[0]
        ));
        screen.sync_filter(&ctx);
        assert_eq!(screen.filtered, Some(vec![]));
        assert!(screen.filter_note.contains("another build"));

        // cleared: everything again, selection reset
        screen.filter.input.buffer.set_text("");
        screen.sync_filter(&ctx);
        assert_eq!(screen.filtered, None);
        assert_eq!(screen.selected_ordinal(), None);
    }

    #[test]
    fn selected_row_maps_through_the_filter() {
        let fix = Fix::text();
        let lookup = fix.lookup();
        let ctx = ctx(&lookup, &fix);
        let (mut screen, _) = harness();
        screen.filter.input.focused = true;
        screen.filter.insert("gamma");
        screen.sync_filter(&ctx);
        screen.list.selected = Some(0);
        assert_eq!(screen.selected_ordinal(), Some(2));
    }

    #[test]
    fn media_chunks_want_a_frame_and_render_the_box() {
        let fix = Fix::media();
        let lookup = fix.lookup();
        let ctx = ctx(&lookup, &fix);
        let (mut screen, theme) = harness();
        screen.list.selected = Some(1);
        assert_eq!(screen.wanted_frame(&lookup), Some(1));
        let mut c = Compositor::new();
        screen.render(&mut c, Rect::new(40.0, 128.0, 1200.0, 600.0), &theme, &ctx);
    }

    #[test]
    fn renders_without_texts_while_loading() {
        let db = fixtures::fake_db();
        let frames = HashMap::new();
        let lookup = ChunkLookup {
            ids: &db.chunk_ids,
            index: None,
            chunks: None,
            frames: &frames,
        };
        let ctx = ChunksContext {
            lookup: &lookup,
            content_hash: &db.inspect.content_hash,
            loading: true,
        };
        let (mut screen, theme) = harness();
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
}
