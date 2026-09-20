//! Table: a header row plus virtualized data rows in columns whose
//! widths derive from the available width (weights, clamped by each
//! column's minimum), never from fixed pixels. The rows are a
//! [`VirtualList`] (scroll, hover, selection, fading scrollbar); the
//! cells are the caller's through a render closure, so a table never
//! knows what a cell holds, only where it goes.
//!
//! Narrow tables drop their lowest-priority columns first, so a phone
//! sees the two columns that matter instead of six unreadable slivers.

use engine::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use engine::text::{TextMeasurer, TextStyle};
use engine::theme::{ControlSize, Theme};

use crate::content::VirtualList;
use crate::core::{EventResult, Rect, WidgetEvent};
use crate::recipe::rounded_rect;

/// Horizontal alignment of a column's cells.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Align {
    #[default]
    Start,
    End,
}

#[derive(Clone, Debug)]
pub struct Column {
    pub title: String,
    /// Share of the free width this column takes (flex-like).
    pub weight: f32,
    /// Narrowest the column may get before it is dropped.
    pub min_w: f32,
    /// Columns drop in ascending priority when the width runs out; the
    /// highest priority never drops.
    pub priority: u8,
    pub align: Align,
}

impl Column {
    /// A column with weight 1 and the field minimum width.
    pub fn new(title: impl Into<String>, theme: &Theme) -> Self {
        Self {
            title: title.into(),
            weight: 1.0,
            min_w: theme.size.field_min_w,
            priority: u8::MAX,
            align: Align::Start,
        }
    }

    pub fn weight(mut self, weight: f32) -> Self {
        self.weight = weight.max(0.0);
        self
    }

    pub fn min_w(mut self, min_w: f32) -> Self {
        self.min_w = min_w.max(0.0);
        self
    }

    pub fn priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }
}

/// One frame of column geometry: the visible columns (by index into
/// `Table::columns`) and their x spans.
#[derive(Clone, Debug, PartialEq)]
pub struct ColumnLayout {
    pub visible: Vec<usize>,
    pub spans: Vec<(f32, f32)>,
}

#[derive(Debug)]
pub struct Table {
    pub columns: Vec<Column>,
    pub rows: VirtualList,
}

impl Table {
    /// Rows are one `Lg` control tall.
    pub fn new(columns: Vec<Column>, theme: &Theme) -> Self {
        Self {
            columns,
            rows: VirtualList::new(Self::row_h(theme)),
        }
    }

    pub fn row_h(theme: &Theme) -> f32 {
        theme.control.height(ControlSize::Lg)
    }

    /// Header height: one `Xs` control.
    pub fn header_h(theme: &Theme) -> f32 {
        theme.control.height(ControlSize::Xs)
    }

    pub fn set_row_count(&mut self, count: usize) {
        self.rows.set_item_count(count);
    }

    pub fn row_count(&self) -> usize {
        self.rows.item_count()
    }

    pub fn selected(&self) -> Option<usize> {
        self.rows.selected
    }

    /// Cell padding: the `md` step.
    pub fn cell_pad(theme: &Theme) -> f32 {
        theme.spacing.md
    }

    fn header_style(theme: &Theme) -> TextStyle {
        theme.typography.caption_sm()
    }

    /// Cell text style: base-2r.
    pub fn cell_style(theme: &Theme) -> TextStyle {
        theme.typography.base_2r()
    }

    /// Which columns fit in `width`, and where. Columns drop lowest
    /// priority first until the minimums fit; the free width above the
    /// minimums is shared by weight.
    pub fn column_layout(&self, width: f32, theme: &Theme) -> ColumnLayout {
        let gap = theme.spacing.sm;
        let mut visible: Vec<usize> = (0..self.columns.len()).collect();
        let fits = |v: &[usize]| {
            let mins: f32 = v.iter().map(|i| self.columns[*i].min_w).sum();
            mins + gap * (v.len().saturating_sub(1)) as f32 <= width
        };
        while visible.len() > 1 && !fits(&visible) {
            let drop = visible
                .iter()
                .copied()
                .min_by_key(|i| self.columns[*i].priority)
                .unwrap();
            visible.retain(|i| *i != drop);
        }
        let mins: f32 = visible.iter().map(|i| self.columns[*i].min_w).sum();
        let free = (width - mins - gap * (visible.len().saturating_sub(1)) as f32).max(0.0);
        let total_w: f32 = visible.iter().map(|i| self.columns[*i].weight).sum();
        let mut x = 0.0;
        let spans = visible
            .iter()
            .map(|i| {
                let col = &self.columns[*i];
                let share = if total_w > 0.0 {
                    col.weight / total_w
                } else {
                    1.0 / visible.len() as f32
                };
                let w = col.min_w + free * share;
                let span = (x, w);
                x += w + gap;
                span
            })
            .collect();
        ColumnLayout { visible, spans }
    }

    /// The rows' rect inside `bounds` (under the header).
    pub fn rows_rect(bounds: Rect, theme: &Theme) -> Rect {
        let h = Self::header_h(theme);
        Rect::new(bounds.x, bounds.y + h, bounds.w, (bounds.h - h).max(0.0))
    }

    /// Cell rect for column `col` (an index into `columns`) in `row`,
    /// or `None` when the column is dropped at this width.
    pub fn cell_rect(
        &self,
        row: Rect,
        col: usize,
        layout: &ColumnLayout,
        theme: &Theme,
    ) -> Option<Rect> {
        let pos = layout.visible.iter().position(|i| *i == col)?;
        let (x, w) = layout.spans[pos];
        let pad = Self::cell_pad(theme);
        Some(Rect::new(
            row.x + x + pad,
            row.y,
            (w - pad * 2.0).max(0.0),
            row.h,
        ))
    }

    /// Draw `text` in `cell` with the column's alignment, truncated to
    /// the cell width with the cell style.
    pub fn draw_cell(
        c: &mut Compositor,
        layer: LayerId,
        cell: Rect,
        text: &str,
        align: Align,
        color: [f32; 4],
        theme: &Theme,
    ) {
        let style = Self::cell_style(theme);
        let shown = TextMeasurer::truncate_to_width(text, &style, cell.w);
        let (tw, _) = TextMeasurer::measure_styled(&shown, &style, None);
        let x = match align {
            Align::Start => cell.x,
            Align::End => cell.x + cell.w - tw,
        };
        c.push_to_layer(
            layer,
            SceneNode::Text {
                key: TextNodeKey::from_style(&shown, &style, None),
                x,
                y: cell.y + TextMeasurer::vertical_center(&style, cell.h),
                color,
            },
        );
    }

    pub fn tick(&mut self, dt: f32) -> bool {
        self.rows.tick(dt)
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        theme: &Theme,
    ) -> EventResult {
        self.rows
            .handle_event(event, Self::rows_rect(bounds, theme), theme)
    }

    /// Render the header and the visible rows. `cell_fn(compositor,
    /// row_index, cell_rect, column_index)` draws one cell; it is called
    /// once per visible column per visible row.
    pub fn render_with(
        &mut self,
        c: &mut Compositor,
        bounds: Rect,
        theme: &Theme,
        cell_fn: impl FnMut(&mut Compositor, usize, Rect, usize),
    ) {
        self.render_with_to_layer(c, LayerId::DEFAULT, bounds, theme, cell_fn);
    }

    pub fn render_with_to_layer(
        &mut self,
        c: &mut Compositor,
        layer: LayerId,
        bounds: Rect,
        theme: &Theme,
        mut cell_fn: impl FnMut(&mut Compositor, usize, Rect, usize),
    ) {
        let layout = self.column_layout(bounds.w, theme);
        let glass = &theme.glass;

        // Header band: tabs-tone strip, caption-sm titles, a hairline
        // under it.
        let header = Rect::new(bounds.x, bounds.y, bounds.w, Self::header_h(theme));
        c.push_to_layer(
            layer,
            rounded_rect(
                header.x,
                header.y,
                header.w,
                header.h,
                theme.shape.nav.min(header.h / 2.0),
                glass.tabs.0,
            ),
        );
        let style = Self::header_style(theme);
        for (pos, col_i) in layout.visible.iter().enumerate() {
            let col = &self.columns[*col_i];
            if let Some(cell) = self.cell_rect(header, *col_i, &layout, theme) {
                let shown = TextMeasurer::truncate_to_width(&col.title, &style, cell.w);
                let (tw, _) = TextMeasurer::measure_styled(&shown, &style, None);
                let x = match col.align {
                    Align::Start => cell.x,
                    Align::End => cell.x + cell.w - tw,
                };
                c.push_to_layer(
                    layer,
                    SceneNode::Text {
                        key: TextNodeKey::from_style(&shown, &style, None),
                        x,
                        y: cell.y + TextMeasurer::vertical_center(&style, cell.h),
                        color: glass.text_faint.0,
                    },
                );
            }
            let _ = pos;
        }
        c.push_to_layer(
            layer,
            SceneNode::Rect {
                x: bounds.x,
                y: header.y + header.h,
                w: bounds.w,
                h: theme.control.edge_width,
                color: theme.colors.divider.0,
            },
        );

        let rows_rect = Self::rows_rect(bounds, theme);
        let visible = layout.visible.clone();
        let table_layout = layout;
        let columns_len = self.columns.len();
        let cell_rects: Vec<Option<(f32, f32)>> = (0..columns_len)
            .map(|col| {
                table_layout
                    .visible
                    .iter()
                    .position(|i| *i == col)
                    .map(|pos| table_layout.spans[pos])
            })
            .collect();
        let pad = Self::cell_pad(theme);
        self.rows
            .render_with_to_layer(c, layer, rows_rect, theme, |c, index, row, _, _| {
                for col in &visible {
                    if let Some((x, w)) = cell_rects[*col] {
                        let cell =
                            Rect::new(row.x + x + pad, row.y, (w - pad * 2.0).max(0.0), row.h);
                        cell_fn(c, index, cell, *col);
                    }
                }
            });
    }
}
