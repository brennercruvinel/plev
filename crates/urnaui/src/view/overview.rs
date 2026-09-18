//! OVERVIEW screen: the opened database's identity, manifest, capability
//! chips and section table — a straight rendering of `OpenedDbView`.
//!
//! Layout is a pure function of `(content, db)` ([`layout`]) so hit
//! testing and drawing always agree; heights derive from the data
//! (optional manifest fields add rows), never from viewport constants.

use engine::compositor::Compositor;
use engine::text::{TextMeasurer, TextStyle};
use engine::theme::Theme;
use engine::ui::widgets::{EventResult, IconButton, Rect, WidgetEvent};

use crate::model::types::OpenedDbView;

use super::{Action, fmt_bytes, group_label, panel, text};

const CARD_PAD: f32 = 20.0;
const ROW_H: f32 = 26.0;
const CARD_GAP: f32 = 24.0;
const CHIP_GAP: f32 = 8.0;
/// Key column width as a fraction of the card's inner width.
const KEY_FRAC: f32 = 0.28;

/// One card's rows: (key, value, copy target index when copyable).
struct Card {
    title: &'static str,
    rows: Vec<(String, String, Option<CopyTarget>)>,
}

/// Which hash a copy button copies (buttons are retained; rows are not).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum CopyTarget {
    File,
    Content,
    Model,
}

/// Rects the screen needs for hit testing, produced by [`layout`].
struct Layout {
    identity: Rect,
    manifest: Rect,
    chips: Rect,
    chip_rects: Vec<Rect>,
    sections: Rect,
    copy_buttons: [(CopyTarget, Rect); 3],
    section_rows: Vec<Rect>,
    total_h: f32,
}

/// A capability chip: present = Constructive, absent = Neutral outline
/// (engine `Chip`; static — capabilities are not clickable).
fn chip_for(label: &str, present: bool) -> engine::ui::widgets::Chip {
    engine::ui::widgets::Chip::new(label).intent(if present {
        engine::theme::Intent::Constructive
    } else {
        engine::theme::Intent::Neutral
    })
}

fn chips(db: &OpenedDbView) -> Vec<(&'static str, bool)> {
    let caps = &db.inspect.manifest.capabilities;
    let ext = db.inspect.manifest.capabilities_ext.as_ref();
    vec![
        ("exact", caps.supports_exact),
        ("ann", db.has_ann),
        ("bm25", db.has_bm25),
        ("citations", caps.supports_citations),
        ("reproducible", caps.supports_reproducible_build),
        ("graph", db.has_graph),
        ("blobs", ext.and_then(|e| e.blobs_present).unwrap_or(false)),
        ("spaces", db.has_spaces),
    ]
}

/// Card contents derived from the snapshot. Value column strings are
/// truncated at render time against the real column width.
fn cards(db: &OpenedDbView) -> [Card; 2] {
    let m = &db.inspect.manifest;
    type Row = (String, String, Option<CopyTarget>);
    let row = |k: &str, v: String, c: Option<CopyTarget>| -> Row { (k.to_string(), v, c) };

    let mut manifest_rows = vec![
        row("embedding_model", m.embedding_model.clone(), None),
        row("embedding_dim", m.embedding_dim.to_string(), None),
        row("dtype", m.dtype.clone(), None),
        row("metric", m.metric.clone(), None),
        row("n_chunks", m.n_chunks.to_string(), None),
        row("chunker_version", m.chunker_version.clone(), None),
        row("model_hash", m.model_hash.clone(), Some(CopyTarget::Model)),
    ];
    if let Some(title) = &m.title {
        manifest_rows.push(row("title", title.clone(), None));
    }
    if let Some(version) = &m.version {
        manifest_rows.push(row("version", version.clone(), None));
    }
    if let Some(created) = &m.created {
        manifest_rows.push(row("created", created.clone(), None));
    }
    if db.has_spaces {
        manifest_rows.push(row("spaces", db.space_names.join(", "), None));
    }

    [
        Card {
            title: "IDENTITY",
            rows: vec![
                row("path", db.path.display().to_string(), None),
                row("file size", fmt_bytes(db.inspect.file_size), None),
                row(
                    "file_hash",
                    db.inspect.file_hash.clone(),
                    Some(CopyTarget::File),
                ),
                row(
                    "content_hash",
                    db.inspect.content_hash.clone(),
                    Some(CopyTarget::Content),
                ),
                row("simd backend", db.inspect.simd_backend.clone(), None),
            ],
        },
        Card {
            title: "MANIFEST",
            rows: manifest_rows,
        },
    ]
}

/// Pure layout: card rects, copy-button rects, section row rects and the
/// total content height, all derived from `content` and the data.
fn layout(content: Rect, db: &OpenedDbView) -> Layout {
    let [identity_card, manifest_card] = cards(db);
    let card_h = |card: &Card| CARD_PAD * 2.0 + 24.0 + card.rows.len() as f32 * ROW_H;

    let identity = Rect::new(content.x, content.y, content.w, card_h(&identity_card));
    let manifest = Rect::new(
        content.x,
        identity.y + identity.h + CARD_GAP,
        content.w,
        card_h(&manifest_card),
    );

    // Chips wrap in rows against the content width, sized by the engine
    // Chip's own measured preferred size.
    let chip_list = chips(db);
    let mut cx = content.x;
    let mut cy_rows = 1usize;
    let mut chip_rects = Vec::with_capacity(chip_list.len());
    let mut row_top = 0.0_f32;
    for &(label, present) in &chip_list {
        let (w, h) = chip_for(label, present).preferred_size();
        if cx + w > content.x + content.w && cx > content.x {
            cy_rows += 1;
            row_top += h + CHIP_GAP;
            cx = content.x;
        }
        chip_rects.push(Rect::new(cx, row_top, w, h));
        cx += w + CHIP_GAP;
    }
    let chips_y = manifest.y + manifest.h + CARD_GAP + 24.0;
    for r in &mut chip_rects {
        r.y += chips_y;
    }
    let chip_h = chip_rects.first().map(|r| r.h).unwrap_or(0.0);
    let chips_rect = Rect::new(
        content.x,
        chips_y,
        content.w,
        cy_rows as f32 * (chip_h + CHIP_GAP),
    );

    let sections_y = chips_rect.y + chips_rect.h + CARD_GAP + 24.0;
    let section_rows: Vec<Rect> = (0..db.inspect.sections.len())
        .map(|i| {
            Rect::new(
                content.x,
                sections_y + 28.0 + i as f32 * ROW_H,
                content.w,
                ROW_H,
            )
        })
        .collect();
    let sections = Rect::new(
        content.x,
        sections_y,
        content.w,
        28.0 + db.inspect.sections.len() as f32 * ROW_H,
    );

    // Copy buttons sit at the right edge of their rows.
    let mut copy_buttons = [
        (CopyTarget::File, Rect::default()),
        (CopyTarget::Content, Rect::default()),
        (CopyTarget::Model, Rect::default()),
    ];
    let place = |card: &Card, rect: Rect, copy_buttons: &mut [(CopyTarget, Rect); 3]| {
        for (i, (_, _, target)) in card.rows.iter().enumerate() {
            if let Some(t) = target {
                let row_y = rect.y + CARD_PAD + 24.0 + i as f32 * ROW_H;
                let slot = copy_buttons.iter_mut().find(|(ct, _)| *ct == *t).unwrap();
                // 40px square (ButtonSize::Sm); the ghost variant has no
                // fill, so overhanging the 26px row is invisible.
                slot.1 = Rect::new(rect.x + rect.w - CARD_PAD - 40.0, row_y - 7.0, 40.0, 40.0);
            }
        }
    };
    place(&identity_card, identity, &mut copy_buttons);
    place(&manifest_card, manifest, &mut copy_buttons);

    Layout {
        identity,
        manifest,
        chips: chips_rect,
        chip_rects,
        sections,
        copy_buttons,
        section_rows,
        total_h: sections.y + sections.h - content.y,
    }
}

pub struct OverviewScreen {
    copy_file: IconButton,
    copy_content: IconButton,
    copy_model: IconButton,
    /// Page scroll: the overview is taller than the viewport (cards +
    /// section table), so it scrolls like a HOFF page.
    scroll: engine::input::scroll::ScrollState,
}

impl OverviewScreen {
    pub fn new() -> Self {
        let copy = || IconButton::new("copy").variant(engine::ui::widgets::ButtonVariant::Ghost);
        Self {
            copy_file: copy(),
            copy_content: copy(),
            copy_model: copy(),
            scroll: engine::input::scroll::ScrollState::new(),
        }
    }

    /// Clamp the scroll offset to the current viewport/content (resize can
    /// shrink content; the offset must follow).
    fn sync_scroll(&mut self, viewport: Rect, db: &OpenedDbView) {
        self.scroll.set_viewport(viewport.h);
        self.scroll.set_content(layout(viewport, db).total_h);
    }

    /// The layout rect shifted up by the scroll offset. Layout, hit
    /// testing and rendering all run against this rect so events and
    /// pixels always agree; the shell clips it back to the viewport.
    fn scrolled(&self, viewport: Rect) -> Rect {
        Rect::new(
            viewport.x,
            viewport.y - self.scroll.offset(),
            viewport.w,
            viewport.h,
        )
    }

    fn button_for(&mut self, target: CopyTarget) -> &mut IconButton {
        match target {
            CopyTarget::File => &mut self.copy_file,
            CopyTarget::Content => &mut self.copy_content,
            CopyTarget::Model => &mut self.copy_model,
        }
    }

    fn copy_text(db: &OpenedDbView, target: CopyTarget) -> String {
        match target {
            CopyTarget::File => db.inspect.file_hash.clone(),
            CopyTarget::Content => db.inspect.content_hash.clone(),
            CopyTarget::Model => db.inspect.manifest.model_hash.clone(),
        }
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        db: &OpenedDbView,
    ) -> (EventResult, Action) {
        self.sync_scroll(content, db);
        let l = layout(self.scrolled(content), db);
        let mut result = EventResult::IGNORED;
        for (target, rect) in &l.copy_buttons {
            let r = self.button_for(*target).handle_event(event, *rect);
            if r.clicked {
                return (
                    r,
                    Action::Copy {
                        text: Self::copy_text(db, *target),
                        what: match target {
                            CopyTarget::File => "file hash",
                            CopyTarget::Content => "content hash",
                            CopyTarget::Model => "model hash",
                        }
                        .to_string(),
                    },
                );
            }
            result = result.merge(r);
        }

        // Page scroll: the wheel scrolls the overview itself (clamped by
        // ScrollState; only an actual offset change requests a frame).
        if let WidgetEvent::Scroll { x, y, delta } = *event
            && content.contains(x, y)
        {
            let old = self.scroll.offset();
            self.scroll.scroll_by(delta);
            if self.scroll.offset() != old {
                result = result.merge(EventResult::changed());
            } else {
                result = result.merge(EventResult {
                    handled: true,
                    ..EventResult::IGNORED
                });
            }
        }
        (result, Action::None)
    }

    pub fn render(&mut self, c: &mut Compositor, viewport: Rect, theme: &Theme, db: &OpenedDbView) {
        self.sync_scroll(viewport, db);
        let content = self.scrolled(viewport);
        let l = layout(content, db);
        let [identity_card, manifest_card] = cards(db);
        let value_style = TextStyle::new(13.0).with_weight(400);

        for (card, rect) in [(&identity_card, l.identity), (&manifest_card, l.manifest)] {
            panel(c, rect, theme);
            group_label(c, card.title, rect.x + CARD_PAD, rect.y + CARD_PAD, theme);
            for (i, (key, value, target)) in card.rows.iter().enumerate() {
                let row_y = rect.y + CARD_PAD + 24.0 + i as f32 * ROW_H;
                let key_w = (rect.w - CARD_PAD * 2.0) * KEY_FRAC;
                text(
                    c,
                    key,
                    13.0,
                    600,
                    rect.x + CARD_PAD,
                    row_y,
                    theme.colors.text_dim.0,
                );
                // Copyable rows reserve room for their button.
                let value_w = if target.is_some() {
                    rect.w - CARD_PAD * 2.0 - key_w - 76.0
                } else {
                    rect.w - CARD_PAD * 2.0 - key_w
                };
                let value = TextMeasurer::truncate_to_width(value, &value_style, value_w);
                text(
                    c,
                    &value,
                    13.0,
                    400,
                    rect.x + CARD_PAD + key_w,
                    row_y,
                    theme.colors.text_mid.0,
                );
            }
        }
        for (target, rect) in &l.copy_buttons {
            self.button_for(*target).render(c, *rect, theme);
        }

        // Capability chips (engine Chip): present = constructive, absent
        // = neutral dim outline.
        group_label(c, "CAPABILITIES", content.x, l.chips.y - 24.0, theme);
        for ((label, present), rect) in chips(db).iter().zip(&l.chip_rects) {
            chip_for(label, *present).render(c, *rect, theme);
        }

        // Section table: id, name, size.
        group_label(c, "SECTIONS", content.x, l.sections.y - 24.0, theme);
        panel(c, l.sections, theme);
        let id_w = 72.0;
        let size_w = 96.0;
        for (section, rect) in db.inspect.sections.iter().zip(&l.section_rows) {
            text(
                c,
                &format!("0x{:02X}", section.section_id),
                12.0,
                400,
                rect.x + CARD_PAD,
                rect.y + 6.0,
                theme.colors.text_dim.0,
            );
            let name = TextMeasurer::truncate_to_width(
                &section.name,
                &value_style,
                rect.w - CARD_PAD * 2.0 - id_w - size_w,
            );
            text(
                c,
                &name,
                13.0,
                500,
                rect.x + CARD_PAD + id_w,
                rect.y + 6.0,
                theme.colors.text_mid.0,
            );
            text(
                c,
                &fmt_bytes(section.size),
                12.0,
                400,
                rect.x + rect.w - CARD_PAD - size_w,
                rect.y + 6.0,
                theme.colors.text_dim.0,
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Headless overview tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view::fixtures;

    #[test]
    fn copy_button_copies_the_file_hash_at_two_widths() {
        let db = fixtures::fake_db();
        for (w, h) in [(800.0, 600.0), (1600.0, 1000.0)] {
            let mut screen = OverviewScreen::new();
            let content = Rect::new(40.0, 128.0, w - 80.0, h - 128.0 - 40.0);
            let rect = layout(content, &db).copy_buttons[0].1;
            let (x, y) = rect.center();
            screen.handle_event(&WidgetEvent::MouseDown { x, y }, content, &db);
            let (r, action) = screen.handle_event(&WidgetEvent::MouseUp { x, y }, content, &db);
            assert!(r.clicked);
            match action {
                Action::Copy { text, .. } => assert_eq!(text, db.inspect.file_hash),
                other => panic!("expected copy action, got {other:?}"),
            }
        }
    }

    #[test]
    fn page_scroll_clamps_to_the_content() {
        let db = fixtures::fake_db();
        let mut screen = OverviewScreen::new();
        let content = Rect::new(40.0, 128.0, 720.0, 200.0);
        // A huge wheel delta must clamp at max offset, not run away.
        let scroll = WidgetEvent::Scroll {
            x: 100.0,
            y: 200.0,
            delta: 100_000.0,
        };
        let (r, _) = screen.handle_event(&scroll, content, &db);
        assert!(r.changed);
        let offset = screen.scroll.offset();
        assert!(offset > 0.0, "content is taller than the viewport");
        assert!(screen.scroll.is_scrollable());
        // Scrolling again does not move: the offset is at the clamp.
        let (r2, _) = screen.handle_event(&scroll, content, &db);
        assert!(!r2.changed);
        assert_eq!(screen.scroll.offset(), offset);
    }

    #[test]
    fn renders_at_narrow_and_wide() {
        let db = fixtures::fake_db();
        let theme = Theme::hoff();
        let mut screen = OverviewScreen::new();
        for (w, h) in [(800.0, 600.0), (1600.0, 1000.0)] {
            let mut c = Compositor::new();
            screen.render(
                &mut c,
                Rect::new(40.0, 128.0, w - 80.0, h - 168.0),
                &theme,
                &db,
            );
        }
    }
}
