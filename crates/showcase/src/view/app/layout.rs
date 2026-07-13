//! Pure geometry for the App tab: no GPU, no window. The footer pills and
//! the counter are measured with TextMeasurer (one style per run, shared
//! with drawing); constants exist only as min, max and gap.

use engine::text::{TextMeasurer, TextStyle};
use engine::theme::TypographyScale;
use engine::ui::widgets::Rect;
use showcase::model::todo::Filter;

pub(super) const PAD: f32 = 24.0;
/// Readability clamp: the app card never stretches past this width.
pub(super) const MAX_W: f32 = 640.0;
/// Below this height the page scroll (view/mod.rs) covers the difference.
pub(super) const MIN_H: f32 = 280.0;
/// TextInput field font; the field height is its `font_size * 2` rule.
pub(super) const INPUT_FONT: f32 = 16.0;
/// Inner text padding of TextInput (`build_scene`/`handle_click` pair).
pub(super) const INPUT_PAD: f32 = 8.0;
pub(super) const ROW_H: f32 = 44.0;
/// Checkbox box edge (18, see widgets::checkbox) + its 10px label gap.
pub(super) const LABEL_INSET: f32 = 28.0;
/// Delete button hit square.
pub(super) const DEL: f32 = 28.0;
pub(super) const PILL_H: f32 = 26.0;
pub(super) const STRIKE_H: f32 = 1.5;
const PILL_PAD_X: f32 = 12.0;
const PILL_GAP: f32 = 8.0;
const COUNTER_GAP: f32 = 16.0;

pub(super) fn item_style() -> TextStyle {
    TypographyScale::hoff().base_2r()
}

pub(super) fn pill_style() -> TextStyle {
    TypographyScale::hoff().caption_sm()
}

pub(super) fn counter_style() -> TextStyle {
    TypographyScale::hoff().caption_r()
}

/// One frame of geometry: card, add field, scrollable list viewport and
/// the footer (divider, counter origin, one pill rect per filter).
pub(super) struct Layout {
    pub panel: Rect,
    pub input: Rect,
    pub list: Rect,
    pub divider_y: f32,
    pub counter: (f32, f32),
    pub pills: [Rect; 3],
}

pub(super) fn compute(content: Rect, counter: &str) -> Layout {
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
        INPUT_FONT * 2.0,
    );

    let pstyle = pill_style();
    let pw = Filter::ALL.map(|f| {
        (TextMeasurer::measure_styled(f.label(), &pstyle, None).0 + PILL_PAD_X * 2.0).ceil()
    });
    let pills_w = pw[0] + pw[1] + pw[2] + PILL_GAP * 2.0;
    let cstyle = counter_style();
    let counter_w = TextMeasurer::measure_styled(counter, &cstyle, None).0;

    // Counter and pills share one footer row when both fit; on a narrow
    // panel the footer stacks them instead of overlapping.
    let stacked = counter_w + COUNTER_GAP + pills_w > panel.w - PAD * 2.0;
    let footer_h = if stacked {
        PILL_H * 2.0 + 30.0
    } else {
        PILL_H + 26.0
    };
    let divider_y = panel.y + panel.h - footer_h;

    let pill_y = panel.y + panel.h - PILL_H - 13.0;
    let mut px = panel.x + panel.w - PAD - pills_w;
    let pills = pw.map(|w| {
        let r = Rect::new(px, pill_y, w, PILL_H);
        px += w + PILL_GAP;
        r
    });
    let counter = (
        panel.x + PAD,
        if stacked {
            divider_y + 8.0
        } else {
            pill_y + TextMeasurer::vertical_center(&cstyle, PILL_H)
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

/// Label origin x and the width available before the delete button.
pub(super) fn label_span(row: Rect) -> (f32, f32) {
    let lx = checkbox_rect(row).x + LABEL_INSET;
    (lx, (delete_rect(row).x - 8.0 - lx).max(0.0))
}
