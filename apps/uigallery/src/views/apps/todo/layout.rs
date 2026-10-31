//! Pure geometry for the App tab: no GPU, no window. The footer filters
//! are the kit's toggle group (`engine::widgets::selection::ToggleGroup`,
//! its small step), sized by the widget; the counter is measured with
//! TextMeasurer (one style per run, shared with drawing); constants exist
//! only as min, max and gap.

use crate::domain::todo::Filter;
use engine::drawing::text::{TextMeasurer, TextStyle};
use engine::styling::theme::{Theme, TypographyScale};
use engine::widgets::form::TextField;
use engine::widgets::prelude::Rect;
use engine::widgets::selection::{SegmentSize, ToggleGroup};

pub(super) const PAD: f32 = 24.0;
/// Readability clamp: the app card never stretches past this width.
pub(super) const MAX_W: f32 = 640.0;
/// Below this height the page scroll (view/mod.rs) covers the difference.
pub(super) const MIN_H: f32 = 280.0;
pub(super) const ROW_H: f32 = 44.0;
/// Checkbox box edge (18, see widgets::checkbox) + its 10px label gap.
pub(super) const LABEL_INSET: f32 = 28.0;
/// Delete button hit square.
pub(super) const DEL: f32 = 28.0;
pub(super) const STRIKE_H: f32 = 1.5;
const COUNTER_GAP: f32 = 16.0;

pub(super) fn item_style() -> TextStyle {
    TypographyScale::hoff().base_2r()
}

/// The filters as the kit's small toggle group, one segment per
/// filter in `Filter::ALL` order: what the footer measures and draws.
pub(super) fn filter_group() -> ToggleGroup {
    ToggleGroup::new(Filter::ALL.map(|f| f.label())).size(SegmentSize::Sm)
}

pub(super) fn counter_style() -> TextStyle {
    TypographyScale::hoff().caption_r()
}

/// One frame of geometry: card, add field, scrollable list viewport and
/// the footer (divider, counter origin, the filter group and one segment
/// rect per filter).
pub(super) struct Layout {
    pub panel: Rect,
    pub input: Rect,
    pub list: Rect,
    pub divider_y: f32,
    pub counter: (f32, f32),
    pub filters: Rect,
    pub pills: [Rect; 3],
}

pub(super) fn compute(content: Rect, counter: &str, theme: &Theme) -> Layout {
    let w = content.w.min(MAX_W);
    let panel = Rect::new(
        (content.x + (content.w - w) / 2.0).floor(),
        content.y,
        w,
        content.h.max(MIN_H),
    );
    let input = Rect::new(
        panel.x + PAD,
        panel.y + PAD,
        panel.w - PAD * 2.0,
        TextField::height(theme),
    );

    let group = filter_group();
    let (pills_w, pill_h) = group.preferred_size(theme);
    let pills_w = pills_w.min(panel.w - PAD * 2.0);
    let cstyle = counter_style();
    let counter_w = TextMeasurer::measure_styled(counter, &cstyle, None).0;

    // Counter and pills share one footer row when both fit; on a narrow
    // panel the footer stacks them instead of overlapping.
    let stacked = counter_w + COUNTER_GAP + pills_w > panel.w - PAD * 2.0;
    let footer_h = if stacked {
        pill_h * 2.0 + 30.0
    } else {
        pill_h + 26.0
    };
    let divider_y = panel.y + panel.h - footer_h;

    let pill_y = panel.y + panel.h - pill_h - 13.0;
    let filters = Rect::new(panel.x + panel.w - PAD - pills_w, pill_y, pills_w, pill_h);
    let segments = group.segment_rects(filters, theme);
    let pills = [segments[0], segments[1], segments[2]];
    let counter = (
        panel.x + PAD,
        if stacked {
            divider_y + 8.0
        } else {
            pill_y + TextMeasurer::vertical_center(&cstyle, pill_h)
        },
    );

    let list_top = input.y + input.h + 14.0;
    let list = Rect::new(
        panel.x + PAD,
        list_top,
        panel.w - PAD * 2.0,
        (divider_y - 8.0 - list_top).max(0.0),
    );
    Layout {
        panel,
        input,
        list,
        divider_y,
        counter,
        filters,
        pills,
    }
}

pub(super) fn row_rect(list: Rect, i: usize, offset: f32) -> Rect {
    Rect::new(list.x, list.y + i as f32 * ROW_H - offset, list.w, ROW_H)
}

pub(super) fn delete_rect(row: Rect) -> Rect {
    Rect::new(
        row.x + row.w - DEL - 6.0,
        row.y + (ROW_H - DEL) / 2.0,
        DEL,
        DEL,
    )
}

/// Toggle hit area: box plus label, from the row edge to the delete gap.
pub(super) fn checkbox_rect(row: Rect) -> Rect {
    let del = delete_rect(row);
    Rect::new(
        row.x + 10.0,
        row.y,
        (del.x - 8.0 - row.x - 10.0).max(0.0),
        ROW_H,
    )
}

/// A row's label as it shows in `w`: whole on one line, or cut with an
/// ellipsis; the row is one line tall, and a wrapped second line would
/// spill into the next.
pub(super) fn label_shown(text: &str, w: f32) -> String {
    TextMeasurer::truncate_to_width(text, &item_style(), w)
}

/// Label origin x and the width available before the delete button.
pub(super) fn label_span(row: Rect) -> (f32, f32) {
    let lx = checkbox_rect(row).x + LABEL_INSET;
    (lx, (delete_rect(row).x - 8.0 - lx).max(0.0))
}
