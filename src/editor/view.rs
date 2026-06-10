//! `EditorView`: state, layout/virtualization math and scene emission.

use std::ops::Range;

use editor_core::{Document, GoalColumn, Selection};

use crate::compositor::{Compositor, SceneNode, TextNodeKey};
use crate::layout::ComputedBounds;
use crate::scroll::ScrollState;
use crate::text::TextMeasurer;

use super::clipboard::{ClipboardProvider, default_clipboard};
use super::config::{EditorConfig, EditorTheme};

/// Horizontal padding inside the gutter, each side of the line numbers.
const GUTTER_PAD: f32 = 10.0;
/// Gap between the gutter separator and the first text column.
const TEXT_PAD_X: f32 = 8.0;
/// Caret width in logical pixels.
const CURSOR_W: f32 = 2.0;
/// Height of the IME preedit underline.
const PREEDIT_UNDERLINE_H: f32 = 1.0;

/// Active IME composition, rendered inline at the primary cursor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Preedit {
    pub text: String,
    /// Byte range of the caret inside `text`, as reported by winit.
    pub cursor: Option<(usize, usize)>,
}

/// Pressed-mouse state: the byte position the drag started from.
#[derive(Clone, Copy, Debug)]
pub(super) struct DragState {
    pub(super) anchor: usize,
}

/// Previous click, used to detect double/triple clicks.
#[derive(Clone, Copy, Debug)]
pub(super) struct ClickRecord {
    pub(super) at: web_time::Instant,
    pub(super) x: f32,
    pub(super) y: f32,
    pub(super) count: u8,
}

/// Multi-line, multi-cursor text editor widget.
///
/// Owns an [`editor_core::Document`] and renders it as compositor scene
/// nodes. Rendering is virtualized: only the lines intersecting the viewport
/// (plus [`EditorConfig::overscan_lines`]) are shaped and emitted, so
/// documents with hundreds of thousands of lines render in constant time.
pub struct EditorView {
    pub document: Document,
    pub scroll: ScrollState,
    pub config: EditorConfig,
    /// Bounds from the last `render`/`set_bounds`; mouse and IME geometry
    /// are resolved against these.
    pub(super) bounds: ComputedBounds,
    pub(super) cursor_visible: bool,
    pub(super) blink_timer: f32,
    /// One goal column per selection, kept across consecutive vertical
    /// moves and cleared by any horizontal movement or edit.
    pub(super) goal: Vec<GoalColumn>,
    pub(super) preedit: Option<Preedit>,
    pub(super) clipboard: Box<dyn ClipboardProvider>,
    pub(super) drag: Option<DragState>,
    pub(super) last_click: Option<ClickRecord>,
}

impl EditorView {
    pub fn new(document: Document) -> Self {
        Self {
            document,
            scroll: ScrollState::new(),
            config: EditorConfig::default(),
            bounds: ComputedBounds::default(),
            cursor_visible: true,
            blink_timer: 0.0,
            goal: Vec::new(),
            preedit: None,
            clipboard: default_clipboard(),
            drag: None,
            last_click: None,
        }
    }

    pub fn with_config(mut self, config: EditorConfig) -> Self {
        self.config = config;
        self
    }

    /// Replace the clipboard backend (tests inject a [`LocalClipboard`]
    /// (super::LocalClipboard) so they never touch the OS clipboard).
    pub fn with_clipboard(mut self, clipboard: Box<dyn ClipboardProvider>) -> Self {
        self.clipboard = clipboard;
        self
    }

    // -- geometry ----------------------------------------------------------

    /// Update layout bounds and the scroll viewport/content extents.
    /// Called by `render`; call directly to hit-test before the first frame.
    pub fn set_bounds(&mut self, bounds: ComputedBounds) {
        self.bounds = bounds;
        self.scroll.set_viewport(bounds.height);
        let content_h = self.document.len_lines() as f32 * self.config.line_height;
        self.scroll.set_content(content_h);
    }

    /// Gutter width adapted to the digit count of the last line number.
    pub fn gutter_width(&self) -> f32 {
        if !self.config.show_gutter {
            return 0.0;
        }
        let digits = self.document.len_lines().max(1).ilog10() + 1;
        let digit_w = TextMeasurer::measure_styled("0", &self.config.text_style(), None).0;
        digits.max(2) as f32 * digit_w + 2.0 * GUTTER_PAD
    }

    /// X of the first text column, in window coordinates.
    pub fn text_origin_x(&self) -> f32 {
        self.bounds.x + self.gutter_width() + TEXT_PAD_X
    }

    /// Range of lines to shape and emit for the current scroll position.
    pub fn visible_lines(&self) -> Range<usize> {
        visible_line_range(
            self.scroll.offset(),
            self.bounds.height,
            self.config.line_height,
            self.document.len_lines(),
            self.config.overscan_lines,
        )
    }

    /// Top Y of `line` in window coordinates.
    fn line_top(&self, line: usize) -> f32 {
        self.bounds.y + line as f32 * self.config.line_height - self.scroll.offset()
    }

    /// Line content without its trailing newline.
    pub(super) fn line_text(&self, line: usize) -> String {
        let mut s = self.document.rope().line(line).to_string();
        if s.ends_with('\n') {
            s.pop();
        }
        s
    }

    /// Caret x (relative to the text origin) for a byte offset in a line.
    pub(super) fn caret_x(&self, line_text: &str, byte_in_line: usize) -> f32 {
        TextMeasurer::cursor_x_styled(line_text, &self.config.text_style(), None, byte_in_line)
    }

    /// Window point -> byte offset in the document, honoring scroll.
    pub fn hit_test_point(&self, x: f32, y: f32) -> usize {
        let rope = self.document.rope();
        let lh = self.config.line_height;
        let rel_y = y - self.bounds.y + self.scroll.offset();
        let line = ((rel_y / lh).floor().max(0.0) as usize).min(rope.len_lines() - 1);
        let text = self.line_text(line);
        let rel_x = (x - self.text_origin_x()).max(0.0);
        let byte_in_line = if text.is_empty() {
            0
        } else {
            TextMeasurer::hit_test_styled(&text, &self.config.text_style(), None, rel_x, lh * 0.5)
        };
        rope.line_to_byte(line) + byte_in_line.min(text.len())
    }

    /// Scroll the minimum amount that brings the primary cursor into view.
    pub fn scroll_to_cursor(&mut self) {
        let head = self.document.selections().primary().head;
        let line = self.document.rope().byte_to_line(head);
        let lh = self.config.line_height;
        let top = line as f32 * lh;
        let bottom = top + lh;
        let offset = self.scroll.offset();
        if top < offset {
            self.scroll.scroll_to(top);
        } else if bottom > offset + self.bounds.height {
            self.scroll.scroll_to(bottom - self.bounds.height);
        }
    }

    // -- cursor blink ------------------------------------------------------

    /// Advance the blink clock. Returns true when cursor visibility toggled
    /// (the caller should request a redraw).
    pub fn tick(&mut self, dt: f32) -> bool {
        self.blink_timer += dt;
        if self.blink_timer >= self.config.cursor_blink_interval {
            self.blink_timer %= self.config.cursor_blink_interval;
            self.cursor_visible = !self.cursor_visible;
            return true;
        }
        false
    }

    /// Make the cursor solid and restart the blink interval (called on any
    /// input so the caret never blinks away mid-typing).
    pub fn reset_blink(&mut self) {
        self.cursor_visible = true;
        self.blink_timer = 0.0;
    }

    // -- IME geometry ------------------------------------------------------

    /// Active preedit, if composing.
    pub fn preedit(&self) -> Option<&Preedit> {
        self.preedit.as_ref()
    }

    /// Window-coordinate caret rect for `Window::set_ime_cursor_area`,
    /// tracking the preedit caret while composing.
    pub fn ime_cursor_rect(&self) -> ComputedBounds {
        let head = self.document.selections().primary().head;
        let rope = self.document.rope();
        let line = rope.byte_to_line(head.min(rope.len_bytes()));
        let rel = head - rope.line_to_byte(line);
        let (text, caret_byte) = match self.preedit {
            Some(ref p) => {
                let caret = rel + p.cursor.map_or(p.text.len(), |(start, _)| start);
                (self.compose_preedit_line(line, rel, p).0, caret)
            }
            None => (self.line_text(line), rel),
        };
        ComputedBounds {
            x: self.text_origin_x() + self.caret_x(&text, caret_byte),
            y: self.line_top(line),
            width: CURSOR_W,
            height: self.config.line_height,
        }
    }

    /// Line text with the preedit spliced in at `rel`; returns the composed
    /// string and the byte range the preedit occupies in it.
    fn compose_preedit_line(&self, line: usize, rel: usize, p: &Preedit) -> (String, Range<usize>) {
        let text = self.line_text(line);
        let rel = rel.min(text.len());
        let composed = format!("{}{}{}", &text[..rel], p.text, &text[rel..]);
        (composed, rel..rel + p.text.len())
    }

    // -- rendering ---------------------------------------------------------

    /// Emit the editor scene into `compositor`. Only visible lines (plus
    /// overscan) are shaped; everything else is skipped entirely.
    pub fn render(
        &mut self,
        compositor: &mut Compositor,
        bounds: ComputedBounds,
        theme: &EditorTheme,
    ) {
        self.set_bounds(bounds);

        compositor.push(SceneNode::Rect {
            x: bounds.x,
            y: bounds.y,
            w: bounds.width,
            h: bounds.height,
            color: theme.background,
        });

        let gutter_w = self.gutter_width();
        if self.config.show_gutter {
            compositor.push(SceneNode::Rect {
                x: bounds.x,
                y: bounds.y,
                w: gutter_w,
                h: bounds.height,
                color: theme.gutter_background,
            });
            compositor.push(SceneNode::Rect {
                x: bounds.x + gutter_w,
                y: bounds.y,
                w: 1.0,
                h: bounds.height,
                color: theme.gutter_separator,
            });
        }

        let lh = self.config.line_height;
        let style = self.config.text_style();
        let text_x = self.text_origin_x();
        let range = self.visible_lines();

        let selections = self.document.selections().clone();
        let primary = selections.primary();
        let rope_len = self.document.len_bytes();
        let preedit_line = self.preedit.as_ref().map(|_| {
            self.document
                .rope()
                .byte_to_line(primary.head.min(rope_len))
        });

        for line in range.clone() {
            let y = self.line_top(line);
            let text = self.line_text(line);

            if self.config.show_gutter {
                let num = (line + 1).to_string();
                let num_w = TextMeasurer::measure_styled(&num, &style, None).0;
                compositor.push(SceneNode::Text {
                    key: self.text_key(&num),
                    x: bounds.x + gutter_w - GUTTER_PAD - num_w,
                    y,
                    color: theme.gutter_text,
                });
            }

            if Some(line) == preedit_line {
                self.render_preedit_line(compositor, line, y, theme);
                continue;
            }

            for sel in selections.iter() {
                if let Some((x0, w)) = self.selection_rect_on_line(sel, line, &text) {
                    compositor.push(SceneNode::Rect {
                        x: text_x + x0,
                        y,
                        w,
                        h: lh,
                        color: theme.selection,
                    });
                }
            }

            if !text.is_empty() {
                compositor.push(SceneNode::Text {
                    key: self.text_key(&text),
                    x: text_x,
                    y,
                    color: theme.text,
                });
            }
        }

        // Carets: every cursor gets a 2px rect; only the primary blinks.
        let rope = self.document.rope();
        for (i, sel) in selections.iter().enumerate() {
            let line = rope.byte_to_line(sel.head);
            if !range.contains(&line) || Some(line) == preedit_line {
                continue;
            }
            if i == selections.primary_index() && !self.cursor_visible {
                continue;
            }
            let text = self.line_text(line);
            let rel = sel.head - rope.line_to_byte(line);
            compositor.push(SceneNode::Rect {
                x: text_x + self.caret_x(&text, rel),
                y: self.line_top(line),
                w: CURSOR_W,
                h: lh,
                color: theme.cursor,
            });
        }
    }

    /// The primary cursor's line while composing: line text with the preedit
    /// spliced in, an underline beneath the preedit span and a solid caret at
    /// the preedit cursor.
    fn render_preedit_line(
        &self,
        compositor: &mut Compositor,
        line: usize,
        y: f32,
        theme: &EditorTheme,
    ) {
        let Some(ref p) = self.preedit else { return };
        let head = self.document.selections().primary().head;
        let rel = head - self.document.rope().line_to_byte(line);
        let (composed, span) = self.compose_preedit_line(line, rel, p);
        let lh = self.config.line_height;
        let text_x = self.text_origin_x();

        if !composed.is_empty() {
            compositor.push(SceneNode::Text {
                key: self.text_key(&composed),
                x: text_x,
                y,
                color: theme.text,
            });
        }

        let x0 = self.caret_x(&composed, span.start);
        let x1 = self.caret_x(&composed, span.end);
        compositor.push(SceneNode::Rect {
            x: text_x + x0,
            y: y + lh - PREEDIT_UNDERLINE_H - 1.0,
            w: (x1 - x0).max(1.0),
            h: PREEDIT_UNDERLINE_H,
            color: theme.preedit_underline,
        });

        let caret_byte = span.start + p.cursor.map_or(p.text.len(), |(start, _)| start);
        compositor.push(SceneNode::Rect {
            x: text_x + self.caret_x(&composed, caret_byte),
            y,
            w: CURSOR_W,
            h: lh,
            color: theme.cursor,
        });
    }

    fn text_key(&self, text: &str) -> TextNodeKey {
        let key = TextNodeKey::new(text, self.config.font_size, self.config.line_height, None);
        match self.config.font_family {
            Some(ref family) => key.with_family(family),
            None => key,
        }
    }

    /// `(x, width)` of the part of `sel` lying on `line` (text-origin
    /// relative), or `None` when the selection does not touch the line.
    /// A selection crossing the line break gets a small extra width to make
    /// the newline visible.
    fn selection_rect_on_line(
        &self,
        sel: &Selection,
        line: usize,
        text: &str,
    ) -> Option<(f32, f32)> {
        if sel.is_caret() {
            return None;
        }
        let rope = self.document.rope();
        let lstart = rope.line_to_byte(line);
        let lnext = if line + 1 < rope.len_lines() {
            rope.line_to_byte(line + 1)
        } else {
            rope.len_bytes()
        };
        let (smin, smax) = (sel.min(), sel.max());
        if smin >= lnext || smax <= lstart {
            return None;
        }
        let content_end = lstart + text.len();
        let seg_start = smin.max(lstart).min(content_end) - lstart;
        let seg_end = smax.min(content_end) - lstart;
        let x0 = self.caret_x(text, seg_start);
        let mut w = self.caret_x(text, seg_end) - x0;
        if smax >= lnext && line + 1 < rope.len_lines() {
            // Selection includes this line's newline.
            w += self.config.font_size * 0.5;
        }
        (w > 0.0).then_some((x0, w))
    }
}

/// Pure virtualization math: the range of lines to shape for a scroll
/// offset and viewport, padded by `overscan` and clamped to the document.
pub(crate) fn visible_line_range(
    scroll_offset: f32,
    viewport_h: f32,
    line_height: f32,
    total_lines: usize,
    overscan: usize,
) -> Range<usize> {
    if line_height <= 0.0 || total_lines == 0 {
        return 0..0;
    }
    let first = (scroll_offset / line_height).floor().max(0.0) as usize;
    let last = ((scroll_offset + viewport_h) / line_height).ceil().max(0.0) as usize;
    let start = first.saturating_sub(overscan);
    let end = last.saturating_add(overscan).min(total_lines);
    start..end.max(start)
}
