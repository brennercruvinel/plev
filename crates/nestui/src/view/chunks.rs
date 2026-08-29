//! CHUNKS screen: virtualized chunk list (index, short id, text preview)
//! with a detail panel for the selected chunk — full id (copyable),
//! source span and the full canonical text in a scrollable viewport.
//!
//! Data flows in from the shell: `ids` (always, from the open snapshot)
//! and `ChunksData` (texts + spans, loaded on first entry). The list owns
//! rendering-window state only (`VirtualList`); nothing here re-reads the
//! file.

use engine::compositor::{Compositor, SceneNode, TextNodeKey};
use engine::input::scroll::ScrollState;
use engine::text::{TextMeasurer, TextStyle};
use engine::theme::Theme;
use engine::ui::widgets::{
    Button, ButtonSize, ButtonVariant, EventResult, Rect, Scrollbar, VirtualList, WidgetEvent,
};

use crate::model::types::ChunksData;

use super::{Action, group_label, panel, short_id, text, truncate_to_width};

const ROW_H: f32 = 48.0;
const GAP: f32 = 16.0;
const DETAIL_PAD: f32 = 16.0;

pub struct ChunksScreen {
    list: VirtualList,
    detail_scroll: ScrollState,
    detail_scrollbar: Scrollbar,
    copy_id: Button,
}

impl ChunksScreen {
    pub fn new() -> Self {
        Self {
            list: VirtualList::new(ROW_H),
            detail_scroll: ScrollState::new(),
            detail_scrollbar: Scrollbar::new(),
            copy_id: Button::new("copy id")
                .size(ButtonSize::Sm)
                .variant(ButtonVariant::Ghost)
                .icon("copy"),
        }
    }

    /// Reset per-database state (called when a new db opens).
    pub fn reset(&mut self) {
        self.list.selected = None;
        self.list.set_item_count(0);
        self.detail_scroll = ScrollState::new();
    }

    /// List rect + detail rect (when a chunk is selected and wide enough;
    /// below ~720px of content width the detail replaces the list).
    fn layout(&self, content: Rect) -> (Rect, Option<Rect>) {
        if self.list.selected.is_none() {
            return (content, None);
        }
        if content.w < 720.0 {
            return (Rect::new(content.x, content.y, 0.0, 0.0), Some(content));
        }
        let detail_w = (content.w * 0.38).clamp(280.0, 420.0);
        let list = Rect::new(content.x, content.y, content.w - detail_w - GAP, content.h);
        let detail = Rect::new(list.x + list.w + GAP, content.y, detail_w, content.h);
        (list, Some(detail))
    }

    /// Height of the wrapped canonical text inside the detail panel.
    fn detail_text_height(&self, data: &ChunksData, idx: usize, width: f32) -> f32 {
        let style = TextStyle::new(13.0).with_line_height(13.0 * 1.5);
        let Some(text) = data.texts.get(idx) else {
            return 0.0;
        };
        TextMeasurer::measure_styled(text, &style, Some(width)).1
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        ids: &[String],
    ) -> (EventResult, Action) {
        self.list.set_item_count(ids.len());
        let (list_rect, detail) = self.layout(content);

        // Copy button inside the detail panel.
        if let (Some(detail), Some(sel)) = (detail, self.list.selected) {
            let r = self.copy_id.handle_event(event, self.copy_rect(detail));
            if r.clicked {
                if let Some(id) = ids.get(sel) {
                    return (
                        r,
                        Action::Copy {
                            text: id.clone(),
                            what: "chunk id".to_string(),
                        },
                    );
                }
            }
            // Detail scroll.
            if let WidgetEvent::Scroll { x, y, delta } = *event
                && self.text_area(detail).contains(x, y)
            {
                let old = self.detail_scroll.offset();
                self.detail_scroll.scroll_by(delta);
                self.detail_scrollbar.notify_scroll();
                if self.detail_scroll.offset() != old {
                    return (EventResult::changed(), Action::None);
                }
            }
            // Selection change resets the detail scroll.
            let before = self.list.selected;
            let r = self.list.handle_event(event, list_rect);
            if self.list.selected != before {
                self.detail_scroll = ScrollState::new();
            }
            return (r, Action::None);
        }

        let r = self.list.handle_event(event, list_rect);
        (r, Action::None)
    }

    fn copy_rect(&self, detail: Rect) -> Rect {
        let (w, h) = self.copy_id.preferred_size();
        Rect::new(
            detail.x + detail.w - DETAIL_PAD - w,
            detail.y + DETAIL_PAD - 4.0,
            w,
            h,
        )
    }

    /// Scrollable canonical-text viewport inside the detail panel.
    fn text_area(&self, detail: Rect) -> Rect {
        // Header rows: id label+row, source, offsets, divider.
        let top = DETAIL_PAD + 24.0 + 26.0 * 3.0 + 8.0;
        Rect::new(
            detail.x + DETAIL_PAD,
            detail.y + top,
            detail.w - DETAIL_PAD * 2.0,
            (detail.h - top - DETAIL_PAD).max(40.0),
        )
    }

    pub fn tick(&mut self, dt: f32) -> bool {
        self.list.tick(dt) | self.detail_scrollbar.tick(dt)
    }

    pub fn render(
        &mut self,
        c: &mut Compositor,
        content: Rect,
        theme: &Theme,
        ids: &[String],
        data: Option<&ChunksData>,
        loading: bool,
    ) {
        self.list.set_item_count(ids.len());
        let (list_rect, detail) = self.layout(content);

        if list_rect.w > 0.0 {
            let preview_style = TextStyle::new(12.0);
            let id_style = TextStyle::new(13.0).with_weight(600);
            let preview_of = |i: usize| -> Option<&str> {
                data?.texts.get(i).map(|t| t.lines().next().unwrap_or(""))
            };
            self.list
                .render_with(c, list_rect, theme, |c, i, row, _hov, _sel| {
                    let pad = 12.0;
                    text(
                        c,
                        &format!("#{i}"),
                        11.0,
                        400,
                        row.x + pad,
                        row.y + 7.0,
                        theme.colors.text_dim.0,
                    );
                    let id = short_id(ids.get(i).map(String::as_str).unwrap_or(""));
                    text(
                        c,
                        &id,
                        13.0,
                        600,
                        row.x + pad + 52.0,
                        row.y + 6.0,
                        theme.colors.text.0,
                    );
                    let preview = match (preview_of(i), loading) {
                        (Some(p), _) => {
                            truncate_to_width(p, row.w - pad * 2.0 - 52.0, &preview_style)
                        }
                        (None, true) => "loading texts…".to_string(),
                        (None, false) => String::new(),
                    };
                    text(
                        c,
                        &preview,
                        12.0,
                        400,
                        row.x + pad + 52.0,
                        row.y + 26.0,
                        theme.colors.text_dim.0,
                    );
                    let _ = id_style;
                });
        }

        if let (Some(detail), Some(sel)) = (detail, self.list.selected) {
            self.render_detail(c, detail, theme, ids, data, sel);
        }
    }

    fn render_detail(
        &mut self,
        c: &mut Compositor,
        detail: Rect,
        theme: &Theme,
        ids: &[String],
        data: Option<&ChunksData>,
        sel: usize,
    ) {
        panel(c, detail, theme);
        group_label(
            c,
            "CHUNK",
            detail.x + DETAIL_PAD,
            detail.y + DETAIL_PAD,
            theme,
        );
        self.copy_id.render(c, self.copy_rect(detail), theme);

        let id_style = TextStyle::new(13.0).with_weight(500);
        let id = ids.get(sel).map(String::as_str).unwrap_or("");
        let id_w = detail.w - DETAIL_PAD * 2.0 - self.copy_rect(detail).w - 8.0;
        let short = truncate_to_width(id, id_w, &id_style);
        text(
            c,
            &short,
            13.0,
            500,
            detail.x + DETAIL_PAD,
            detail.y + DETAIL_PAD + 24.0,
            theme.colors.text.0,
        );

        let meta_y = detail.y + DETAIL_PAD + 24.0 + 26.0;
        let (uri, offsets) = match data.and_then(|d| d.metas.get(sel)) {
            Some(m) => (
                m.source_uri.clone(),
                format!("bytes {}–{}", m.offset_start, m.offset_end),
            ),
            None => ("(spans not loaded)".to_string(), String::new()),
        };
        let meta_style = TextStyle::new(12.0);
        let uri = truncate_to_width(&uri, detail.w - DETAIL_PAD * 2.0, &meta_style);
        text(
            c,
            &uri,
            12.0,
            400,
            detail.x + DETAIL_PAD,
            meta_y,
            theme.colors.text_mid.0,
        );
        text(
            c,
            &offsets,
            12.0,
            400,
            detail.x + DETAIL_PAD,
            meta_y + 26.0,
            theme.colors.text_dim.0,
        );

        // Canonical text, wrapped + scrolled inside its viewport.
        let area = self.text_area(detail);
        let style = TextStyle::new(13.0).with_line_height(13.0 * 1.5);
        if let Some(d) = data
            && let Some(full) = d.texts.get(sel)
        {
            let text_h = self.detail_text_height(d, sel, area.w);
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

// ---------------------------------------------------------------------------
// Headless chunks tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view::fixtures;

    #[test]
    fn clicking_a_row_selects_it_and_shows_the_detail() {
        let db = fixtures::fake_db();
        let data = fixtures::fake_chunks();
        let theme = Theme::hoff();
        let mut screen = ChunksScreen::new();
        let content = Rect::new(40.0, 128.0, 1200.0, 600.0);

        assert_eq!(screen.list.selected, None);
        let (r, _) = screen.handle_event(
            &WidgetEvent::MouseDown {
                x: content.x + 20.0,
                y: content.y + 10.0,
            },
            content,
            &db.chunk_ids,
        );
        assert!(r.clicked);
        assert_eq!(screen.list.selected, Some(0));

        // Wide: list + detail side by side.
        let (list, detail) = screen.layout(content);
        assert!(list.w > 0.0);
        assert!(detail.is_some());

        let mut c = Compositor::new();
        screen.render(&mut c, content, &theme, &db.chunk_ids, Some(&data), false);
    }

    #[test]
    fn narrow_viewports_replace_the_list_with_the_detail() {
        let mut screen = ChunksScreen::new();
        screen.list.selected = Some(1);
        let narrow = Rect::new(40.0, 128.0, 600.0, 400.0);
        let (list, detail) = screen.layout(narrow);
        assert_eq!(list.w, 0.0, "no room for both: the detail wins");
        assert_eq!(detail, Some(narrow));
    }

    #[test]
    fn copy_button_copies_the_full_chunk_id() {
        let db = fixtures::fake_db();
        let mut screen = ChunksScreen::new();
        screen.list.selected = Some(2);
        let content = Rect::new(40.0, 128.0, 1200.0, 600.0);
        let (_, detail) = screen.layout(content);
        let rect = screen.copy_rect(detail.unwrap());
        let (x, y) = rect.center();
        screen.handle_event(&WidgetEvent::MouseDown { x, y }, content, &db.chunk_ids);
        let (r, action) =
            screen.handle_event(&WidgetEvent::MouseUp { x, y }, content, &db.chunk_ids);
        assert!(r.clicked);
        match action {
            Action::Copy { text, .. } => assert_eq!(text, db.chunk_ids[2]),
            other => panic!("expected copy action, got {other:?}"),
        }
    }

    #[test]
    fn renders_without_texts_while_loading() {
        let db = fixtures::fake_db();
        let theme = Theme::hoff();
        let mut screen = ChunksScreen::new();
        for (w, h) in [(800.0, 600.0), (1600.0, 1000.0)] {
            let mut c = Compositor::new();
            screen.render(
                &mut c,
                Rect::new(40.0, 128.0, w - 80.0, h - 168.0),
                &theme,
                &db.chunk_ids,
                None,
                true,
            );
        }
    }
}
