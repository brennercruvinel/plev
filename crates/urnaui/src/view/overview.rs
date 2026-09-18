//! OVERVIEW screen: the opened database's identity (with the validate
//! button), manifest, capability chips, multimodal spaces, media blobs
//! (with export) and the section table — a straight rendering of
//! `OpenedDbView`.
//!
//! Layout is a pure function of `(content, db)` ([`layout`]) so hit
//! testing and drawing always agree; heights derive from the data
//! (optional manifest fields, spaces and blobs add rows), never from
//! viewport constants.

use engine::compositor::Compositor;
use engine::text::{TextMeasurer, TextStyle};
use engine::theme::Theme;
use engine::ui::widgets::{Button, EventResult, IconButton, Rect, WidgetEvent};

use crate::model::types::OpenedDbView;

use super::{Action, fmt_bytes, group_label, panel, text};

const CARD_PAD: f32 = 20.0;
const ROW_H: f32 = 26.0;
const CARD_GAP: f32 = 24.0;
const CHIP_GAP: f32 = 8.0;
/// Key column width as a fraction of the card's inner width.
const KEY_FRAC: f32 = 0.28;
/// Row button footprint (ButtonSize::Sm square); the ghost variant has no
/// fill, so overhanging the 26px row is invisible.
const BTN: f32 = 40.0;

/// Everything the Overview screen needs from the central view state.
pub struct OverviewContext<'a> {
    pub db: &'a OpenedDbView,
    /// Last `Validate` outcome (`Ok(ms)` or the reader's error).
    pub validation: Option<&'a Result<f64, String>>,
    pub validating: bool,
}

/// One card's rows: (key, value, row button).
struct Card {
    title: &'static str,
    rows: Vec<(String, String, Option<RowButton>)>,
}

/// A retained per-row button: copy a hash, or export a blob.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
enum RowButton {
    CopyFile,
    CopyContent,
    CopyModel,
    CopySpace(usize),
    CopyBlob(usize),
    ExportBlob(usize),
}

/// Rects the screen needs for hit testing, produced by [`layout`].
struct Layout {
    cards: Vec<Rect>,
    validate: Rect,
    chips: Rect,
    chip_rects: Vec<Rect>,
    sections: Rect,
    buttons: Vec<(RowButton, Rect)>,
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
        ("inlined media", db.has_blob_data),
        ("spaces", db.has_spaces),
    ]
}

/// Card contents derived from the snapshot. Value column strings are
/// truncated at render time against the real column width. Cards with
/// nothing to show (no spaces, no blobs) are omitted.
fn cards(db: &OpenedDbView) -> Vec<Card> {
    let m = &db.inspect.manifest;
    type Row = (String, String, Option<RowButton>);
    let row = |k: &str, v: String, b: Option<RowButton>| -> Row { (k.to_string(), v, b) };

    let mut manifest_rows = vec![
        row("embedding_model", m.embedding_model.clone(), None),
        row("embedding_dim", m.embedding_dim.to_string(), None),
        row("dtype", m.dtype.clone(), None),
        row("metric", m.metric.clone(), None),
        row("index_type", m.index_type.clone(), None),
        row("n_chunks", m.n_chunks.to_string(), None),
        row("chunker_version", m.chunker_version.clone(), None),
        row(
            "model_hash",
            m.model_hash.clone(),
            Some(RowButton::CopyModel),
        ),
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

    let mut out = vec![
        Card {
            title: "IDENTITY",
            rows: vec![
                row("path", db.path.display().to_string(), None),
                row("magic", db.inspect.magic.clone(), None),
                row("file size", fmt_bytes(db.inspect.file_size), None),
                row(
                    "file_hash",
                    db.inspect.file_hash.clone(),
                    Some(RowButton::CopyFile),
                ),
                row(
                    "content_hash",
                    db.inspect.content_hash.clone(),
                    Some(RowButton::CopyContent),
                ),
                row("simd backend", db.inspect.simd_backend.clone(), None),
            ],
        },
        Card {
            title: "MANIFEST",
            rows: manifest_rows,
        },
    ];

    // One row per multimodal space: what `search-space` can score.
    if !db.inspect.spaces.is_empty() {
        let rows = db
            .inspect
            .spaces
            .iter()
            .enumerate()
            .map(|(i, s)| {
                row(
                    &s.name,
                    format!(
                        "{} vectors × {} {} · band {} · {}",
                        s.n_vectors,
                        s.dim,
                        s.dtype,
                        fmt_bytes(s.band_bytes),
                        s.model_hash
                    ),
                    Some(RowButton::CopySpace(i)),
                )
            })
            .collect();
        out.push(Card {
            title: "SPACES",
            rows,
        });
    }

    // One row per media blob; inlined ones export straight off the file.
    if !db.inspect.blobs.is_empty() {
        let rows = db
            .inspect
            .blobs
            .iter()
            .enumerate()
            .map(|(i, b)| {
                let button = if b.inlined && db.has_blob_data {
                    RowButton::ExportBlob(i)
                } else {
                    RowButton::CopyBlob(i)
                };
                row(
                    b.original_uri.trim_start_matches("media://"),
                    format!(
                        "{} · {} · {}",
                        fmt_bytes(b.byte_len),
                        if b.inlined { "inlined" } else { "sidecar" },
                        b.content_hash
                    ),
                    Some(button),
                )
            })
            .collect();
        out.push(Card {
            title: "MEDIA",
            rows,
        });
    }
    out
}

/// Pure layout: card rects, row-button rects, section row rects and the
/// total content height, all derived from `content` and the data.
fn layout(content: Rect, db: &OpenedDbView) -> Layout {
    let card_list = cards(db);
    let card_h = |card: &Card| CARD_PAD * 2.0 + 24.0 + card.rows.len() as f32 * ROW_H;

    let mut cards_rects = Vec::with_capacity(card_list.len());
    let mut buttons = Vec::new();
    let mut y = content.y;
    for card in &card_list {
        let rect = Rect::new(content.x, y, content.w, card_h(card));
        for (i, (_, _, button)) in card.rows.iter().enumerate() {
            if let Some(b) = button {
                let row_y = rect.y + CARD_PAD + 24.0 + i as f32 * ROW_H;
                buttons.push((
                    *b,
                    Rect::new(rect.x + rect.w - CARD_PAD - BTN, row_y - 7.0, BTN, BTN),
                ));
            }
        }
        cards_rects.push(rect);
        y += rect.h + CARD_GAP;
    }
    // The validate button sits in the IDENTITY card's header row.
    let identity = cards_rects[0];
    let validate = Rect::new(
        identity.x + identity.w - CARD_PAD - 120.0,
        identity.y + CARD_PAD - 8.0,
        120.0,
        32.0,
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
    let chips_y = y + 24.0;
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

    Layout {
        cards: cards_rects,
        validate,
        chips: chips_rect,
        chip_rects,
        sections,
        buttons,
        section_rows,
        total_h: sections.y + sections.h - content.y,
    }
}

pub struct OverviewScreen {
    /// Retained row buttons, keyed by what they act on; rebuilt per db.
    buttons: Vec<(RowButton, IconButton)>,
    validate: Button,
    /// Page scroll: the overview is taller than the viewport (cards +
    /// section table), so it scrolls like a HOFF page.
    scroll: engine::input::scroll::ScrollState,
}

impl OverviewScreen {
    pub fn new() -> Self {
        Self {
            buttons: Vec::new(),
            validate: Button::new("Validate")
                .icon("eye")
                .size(engine::ui::widgets::ButtonSize::Sm)
                .variant(engine::ui::widgets::ButtonVariant::Outline),
            scroll: engine::input::scroll::ScrollState::new(),
        }
    }

    /// Rebuild the row buttons for a freshly opened db (spaces and blobs
    /// change the set) and reset the page scroll.
    pub fn reset(&mut self, db: &OpenedDbView) {
        self.scroll = engine::input::scroll::ScrollState::new();
        self.rebuild_buttons(db);
    }

    fn rebuild_buttons(&mut self, db: &OpenedDbView) {
        self.buttons = cards(db)
            .iter()
            .flat_map(|card| card.rows.iter().filter_map(|(_, _, b)| *b))
            .map(|b| {
                let icon = match b {
                    RowButton::ExportBlob(_) => "save",
                    _ => "copy",
                };
                (
                    b,
                    IconButton::new(icon).variant(engine::ui::widgets::ButtonVariant::Ghost),
                )
            })
            .collect();
    }

    /// Clamp the scroll offset to the current viewport/content (resize can
    /// shrink content; the offset must follow).
    fn sync_scroll(&mut self, viewport: Rect, db: &OpenedDbView) {
        self.scroll.set_viewport(viewport.h);
        self.scroll.set_content(layout(viewport, db).total_h);
        // A db set without `reset` (tests, or a snapshot swap) still gets
        // its buttons.
        if self.buttons.is_empty() {
            self.rebuild_buttons(db);
        }
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

    fn button_for(&mut self, target: RowButton) -> Option<&mut IconButton> {
        self.buttons
            .iter_mut()
            .find(|(b, _)| *b == target)
            .map(|(_, w)| w)
    }

    fn action_for(db: &OpenedDbView, target: RowButton) -> Action {
        let copy = |text: String, what: &str| Action::Copy {
            text,
            what: what.to_string(),
        };
        match target {
            RowButton::CopyFile => copy(db.inspect.file_hash.clone(), "file hash"),
            RowButton::CopyContent => copy(db.inspect.content_hash.clone(), "content hash"),
            RowButton::CopyModel => copy(db.inspect.manifest.model_hash.clone(), "model hash"),
            RowButton::CopySpace(i) => db
                .inspect
                .spaces
                .get(i)
                .map(|s| copy(s.model_hash.clone(), "space model hash"))
                .unwrap_or(Action::None),
            RowButton::CopyBlob(i) => db
                .inspect
                .blobs
                .get(i)
                .map(|b| copy(b.content_hash.clone(), "blob content hash"))
                .unwrap_or(Action::None),
            RowButton::ExportBlob(i) => Action::ExportBlob(i),
        }
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        ctx: &OverviewContext,
    ) -> (EventResult, Action) {
        let db = ctx.db;
        self.sync_scroll(content, db);
        let l = layout(self.scrolled(content), db);
        let mut result = EventResult::IGNORED;

        self.validate.disabled = ctx.validating;
        let r = self.validate.handle_event(event, l.validate);
        if r.clicked {
            return (r, Action::Validate);
        }
        result = result.merge(r);

        for (target, rect) in &l.buttons {
            let Some(button) = self.button_for(*target) else {
                continue;
            };
            let r = button.handle_event(event, *rect);
            if r.clicked {
                return (r, Self::action_for(db, *target));
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

    pub fn render(
        &mut self,
        c: &mut Compositor,
        viewport: Rect,
        theme: &Theme,
        ctx: &OverviewContext,
    ) {
        let db = ctx.db;
        self.sync_scroll(viewport, db);
        let content = self.scrolled(viewport);
        let l = layout(content, db);
        let card_list = cards(db);
        let value_style = TextStyle::new(13.0).with_weight(400);

        for (card, rect) in card_list.iter().zip(&l.cards) {
            panel(c, *rect, theme);
            group_label(c, card.title, rect.x + CARD_PAD, rect.y + CARD_PAD, theme);
            for (i, (key, value, button)) in card.rows.iter().enumerate() {
                let row_y = rect.y + CARD_PAD + 24.0 + i as f32 * ROW_H;
                let key_w = (rect.w - CARD_PAD * 2.0) * KEY_FRAC;
                let key = TextMeasurer::truncate_to_width(key, &value_style, key_w - 8.0);
                text(
                    c,
                    &key,
                    13.0,
                    600,
                    rect.x + CARD_PAD,
                    row_y,
                    theme.colors.text_dim.0,
                );
                // Rows with a button reserve room for it.
                let value_w = if button.is_some() {
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
        for (target, rect) in &l.buttons {
            if let Some(button) = self.button_for(*target) {
                button.render(c, *rect, theme);
            }
        }

        // Validate: the button plus its last outcome, in the IDENTITY
        // card's header row.
        self.validate.disabled = ctx.validating;
        self.validate.label = if ctx.validating {
            "Validating…"
        } else {
            "Validate"
        }
        .to_string();
        self.validate.render(c, l.validate, theme);
        let (status, color) = match ctx.validation {
            Some(Ok(ms)) => (format!("integrity ok · {ms:.0} ms"), theme.colors.success.0),
            Some(Err(e)) => (format!("failed: {e}"), theme.colors.danger.0),
            None => (
                "checksums, hashes and contract were verified at open".to_string(),
                theme.colors.text_dim.0,
            ),
        };
        let status_w = l.validate.x - (l.cards[0].x + CARD_PAD + 90.0) - 12.0;
        if status_w > 80.0 {
            let status = TextMeasurer::truncate_to_width(&status, &TextStyle::new(12.0), status_w);
            text(
                c,
                &status,
                12.0,
                400,
                l.validate.x
                    - 12.0
                    - TextMeasurer::measure_styled(&status, &TextStyle::new(12.0), None).0,
                l.cards[0].y + CARD_PAD + 1.0,
                color,
            );
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

    fn ctx(db: &OpenedDbView) -> OverviewContext<'_> {
        OverviewContext {
            db,
            validation: None,
            validating: false,
        }
    }

    fn click(
        screen: &mut OverviewScreen,
        content: Rect,
        ctx: &OverviewContext,
        rect: Rect,
    ) -> (EventResult, Action) {
        let (x, y) = rect.center();
        screen.handle_event(&WidgetEvent::MouseDown { x, y }, content, ctx);
        screen.handle_event(&WidgetEvent::MouseUp { x, y }, content, ctx)
    }

    #[test]
    fn copy_button_copies_the_file_hash_at_two_widths() {
        let db = fixtures::fake_db();
        let ctx = ctx(&db);
        for (w, h) in [(800.0, 600.0), (1600.0, 1000.0)] {
            let mut screen = OverviewScreen::new();
            let content = Rect::new(40.0, 128.0, w - 80.0, h - 128.0 - 40.0);
            let rect = layout(content, &db).buttons[0].1;
            let (r, action) = click(&mut screen, content, &ctx, rect);
            assert!(r.clicked);
            match action {
                Action::Copy { text, .. } => assert_eq!(text, db.inspect.file_hash),
                other => panic!("expected copy action, got {other:?}"),
            }
        }
    }

    #[test]
    fn validate_button_raises_the_action_unless_running() {
        let db = fixtures::fake_db();
        let mut screen = OverviewScreen::new();
        let content = Rect::new(40.0, 128.0, 1200.0, 700.0);
        let rect = layout(content, &db).validate;
        let (r, action) = click(&mut screen, content, &ctx(&db), rect);
        assert!(r.clicked);
        assert_eq!(action, Action::Validate);
        let running = OverviewContext {
            db: &db,
            validation: None,
            validating: true,
        };
        let (_, action) = click(&mut screen, content, &running, rect);
        assert_eq!(action, Action::None);
    }

    #[test]
    fn media_db_lists_spaces_and_blobs_with_export() {
        let db = fixtures::fake_media_db();
        let titles: Vec<&str> = cards(&db).iter().map(|c| c.title).collect();
        assert_eq!(titles, ["IDENTITY", "MANIFEST", "SPACES", "MEDIA"]);
        let mut screen = OverviewScreen::new();
        let content = Rect::new(40.0, 128.0, 1200.0, 700.0);
        let l = layout(content, &db);
        let export = l
            .buttons
            .iter()
            .find(|(b, _)| *b == RowButton::ExportBlob(0))
            .expect("inlined blob gets an export button");
        let (_, action) = click(&mut screen, content, &ctx(&db), export.1);
        assert_eq!(action, Action::ExportBlob(0));
        let space = l
            .buttons
            .iter()
            .find(|(b, _)| *b == RowButton::CopySpace(0))
            .unwrap();
        match click(&mut screen, content, &ctx(&db), space.1).1 {
            Action::Copy { text, .. } => assert_eq!(text, db.inspect.spaces[0].model_hash),
            other => panic!("expected copy, got {other:?}"),
        }
        // A sidecar blob only copies its hash.
        let mut sidecar = fixtures::fake_media_db();
        sidecar.has_blob_data = false;
        assert!(
            layout(content, &sidecar)
                .buttons
                .iter()
                .any(|(b, _)| *b == RowButton::CopyBlob(0))
        );
    }

    #[test]
    fn page_scroll_clamps_to_the_content() {
        let db = fixtures::fake_db();
        let ctx = ctx(&db);
        let mut screen = OverviewScreen::new();
        let content = Rect::new(40.0, 128.0, 720.0, 200.0);
        // A huge wheel delta must clamp at max offset, not run away.
        let scroll = WidgetEvent::Scroll {
            x: 100.0,
            y: 200.0,
            delta: 100_000.0,
        };
        let (r, _) = screen.handle_event(&scroll, content, &ctx);
        assert!(r.changed);
        let offset = screen.scroll.offset();
        assert!(offset > 0.0, "content is taller than the viewport");
        assert!(screen.scroll.is_scrollable());
        // Scrolling again does not move: the offset is at the clamp.
        let (r2, _) = screen.handle_event(&scroll, content, &ctx);
        assert!(!r2.changed);
        assert_eq!(screen.scroll.offset(), offset);
    }

    #[test]
    fn renders_at_narrow_and_wide() {
        let theme = Theme::hoff();
        for db in [fixtures::fake_db(), fixtures::fake_media_db()] {
            let mut screen = OverviewScreen::new();
            let validated = Ok(12.0);
            let ctx = OverviewContext {
                db: &db,
                validation: Some(&validated),
                validating: false,
            };
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
}
