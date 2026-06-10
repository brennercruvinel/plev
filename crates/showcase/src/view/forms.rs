//! Forms section: checkbox, switch, slider, progress, select, tabs.

use plev::compositor::{Compositor, LayerId};
use plev::text::TextMeasurer;
use plev::theme::{Intent, Theme, TypographyScale};
use plev::ui::widgets::{
    Checkbox, EventResult, ProgressBar, Rect, Select, Slider, Switch, Tabs, WidgetEvent,
};

use super::{group_label, text};

/// Minimum readable column width: below `2 * COL_MIN_W + COL_GAP` of
/// content the two columns stack instead of cropping column B.
const COL_MIN_W: f32 = 300.0;
/// Maximum column width: controls stay legible on ultra-wide windows.
const COL_MAX_W: f32 = 480.0;
const COL_GAP: f32 = 60.0;
/// Minimum horizontal slack around each tab label inside its segment —
/// the folgado pill the GOLDEN_SPEC calls for (>= 32px breathing room
/// plus the strip's own container padding).
const TAB_SEG_SLACK: f32 = 48.0;
/// Gutter to the right of the sliders, reserved for their live value
/// captions ("65", "step 1 — 4") so the captions never spill past the
/// content rect.
const SLIDER_VALUE_GUTTER: f32 = 72.0;
/// Maximum select pill width; it otherwise follows the column.
const SELECT_MAX_W: f32 = 280.0;
const ROW_H: f32 = 32.0;
const LABEL_H: f32 = 24.0;
const GROUP_GAP: f32 = 26.0;

pub struct FormsSection {
    tabs: Tabs,
    autosave: Checkbox,
    telemetry: Checkbox,
    locked: Checkbox,
    focus_mode: Switch,
    wrap_lines: Switch,
    locked_switch: Switch,
    volume: Slider,
    steps: Slider,
    disabled_slider: Slider,
    progress: ProgressBar,
    progress_ok: ProgressBar,
    progress_err: ProgressBar,
    select: Select,
}

struct Layout {
    tabs: Rect,
    checkboxes: [Rect; 3],
    switches: [Rect; 3],
    sliders: [Rect; 3],
    progresses: [Rect; 3],
    select: Rect,
}

impl FormsSection {
    pub fn new(theme: &Theme) -> Self {
        Self {
            // Three roomy segments: the reference keeps each label folgado
            // inside its pill (GOLDEN_SPEC) — the strip is sized from the
            // measured labels (see `tab_strip_w`) so every one fits.
            tabs: Tabs::new(["Account", "Appearance", "About"]),
            autosave: Checkbox::new(true).label("Autosave on focus loss"),
            telemetry: Checkbox::new(false).label("Share anonymous usage data"),
            locked: Checkbox::new(true)
                .label("Managed by your organization")
                .disabled(true),
            focus_mode: Switch::new(true).with_motion(&theme.motion),
            wrap_lines: Switch::new(false).with_motion(&theme.motion),
            locked_switch: Switch::new(false).with_motion(&theme.motion).disabled(true),
            volume: Slider::new(0.0, 100.0, 65.0),
            steps: Slider::new(0.0, 10.0, 4.0).step(1.0),
            disabled_slider: Slider::new(0.0, 100.0, 30.0).disabled(true),
            progress: ProgressBar::new(0.65),
            progress_ok: ProgressBar::new(1.0).intent(Intent::Constructive),
            progress_err: ProgressBar::new(0.35).intent(Intent::Destructive),
            select: Select::new(
                ["System default", "Always dark", "Always light", "Scheduled"],
                0,
            ),
        }
    }

    /// Tab strip width derived from the measured labels (base-2sm, the
    /// style `Tabs::render` uses): every segment keeps `TAB_SEG_SLACK` of
    /// breathing room, clamped to `max_w` so it never overflows.
    fn tab_strip_w(&self, max_w: f32) -> f32 {
        let style = TypographyScale::hoff().base_2sm();
        let widest = self
            .tabs
            .labels
            .iter()
            .map(|l| TextMeasurer::measure_styled(l, &style, None).0)
            .fold(0.0, f32::max);
        ((widest + TAB_SEG_SLACK) * self.tabs.labels.len() as f32).min(max_w)
    }

    /// Content-driven layout: two columns that stretch with `content.w`
    /// (clamped to `COL_MAX_W` for legibility) and stack into a single
    /// column when the content is too narrow for both.
    fn layout(&self, content: Rect) -> Layout {
        let (x, y) = (content.x, content.y);
        let two_cols = content.w >= COL_MIN_W * 2.0 + COL_GAP;
        let col_w = if two_cols {
            ((content.w - COL_GAP) / 2.0).min(COL_MAX_W)
        } else {
            content.w.min(COL_MAX_W)
        };

        // Column A: tabs, checkboxes, switches. HOFF tabs: 44px strip sized
        // from its measured labels so each one stays folgado in its segment.
        // With two columns it may borrow half the inter-column gap (empty
        // space) — the original design did the same — but never reaches
        // column B.
        let tab_max = if two_cols {
            col_w + COL_GAP / 2.0
        } else {
            col_w
        };
        let tabs = Rect::new(x, y + LABEL_H, self.tab_strip_w(tab_max), 44.0);
        let cb_y = tabs.y + tabs.h + GROUP_GAP + LABEL_H;
        let checkboxes = [
            Rect::new(x, cb_y, col_w, ROW_H),
            Rect::new(x, cb_y + ROW_H, col_w, ROW_H),
            Rect::new(x, cb_y + ROW_H * 2.0, col_w, ROW_H),
        ];
        let sw_y = cb_y + ROW_H * 3.0 + GROUP_GAP + LABEL_H;
        let switches = [
            Rect::new(x, sw_y, 44.0, ROW_H),
            Rect::new(x, sw_y + ROW_H + 4.0, 44.0, ROW_H),
            Rect::new(x, sw_y + (ROW_H + 4.0) * 2.0, 44.0, ROW_H),
        ];

        // Column B: sliders, progress, select — beside column A when the
        // content fits both, stacked below it otherwise.
        let (col_b, col_b_y) = if two_cols {
            (x + col_w + COL_GAP, y)
        } else {
            (x, switches[2].y + switches[2].h + GROUP_GAP)
        };
        let slider_w = (col_w - SLIDER_VALUE_GUTTER).max(0.0);
        let sl_y = col_b_y + LABEL_H;
        let sliders = [
            Rect::new(col_b, sl_y, slider_w, ROW_H),
            Rect::new(col_b, sl_y + ROW_H + 14.0, slider_w, ROW_H),
            Rect::new(col_b, sl_y + (ROW_H + 14.0) * 2.0, slider_w, ROW_H),
        ];
        let pr_y = sliders[2].y + ROW_H + GROUP_GAP + LABEL_H;
        let progresses = [
            Rect::new(col_b, pr_y, col_w, 18.0),
            Rect::new(col_b, pr_y + 26.0, col_w, 18.0),
            Rect::new(col_b, pr_y + 52.0, col_w, 18.0),
        ];
        // HOFF select: 44px pill control, following its column.
        let select = Rect::new(
            col_b,
            progresses[2].y + 26.0 + GROUP_GAP + LABEL_H,
            col_w.min(SELECT_MAX_W),
            44.0,
        );

        Layout {
            tabs,
            checkboxes,
            switches,
            sliders,
            progresses,
            select,
        }
    }

    /// Natural height of both form columns (page scrolling needs it).
    pub fn content_height(&self, content: Rect) -> f32 {
        let l = self.layout(content);
        let col_a = l.switches[2].y + l.switches[2].h;
        let col_b = l.select.y + l.select.h;
        col_a.max(col_b) - content.y + GROUP_GAP
    }

    pub fn select_is_open(&self) -> bool {
        self.select.is_open()
    }

    pub fn close_select(&mut self) {
        self.select.close();
    }

    /// Route an event to the open select dropdown (priority path).
    pub fn route_select(&mut self, event: &WidgetEvent, content: Rect) -> EventResult {
        let layout = self.layout(content);
        self.select.handle_event(event, layout.select)
    }

    pub fn handle_event(&mut self, event: &WidgetEvent, content: Rect) -> EventResult {
        let layout = self.layout(content);
        let mut r = EventResult::IGNORED;
        r = r.merge(self.tabs.handle_event(event, layout.tabs));
        r = r.merge(self.autosave.handle_event(event, layout.checkboxes[0]));
        r = r.merge(self.telemetry.handle_event(event, layout.checkboxes[1]));
        r = r.merge(self.locked.handle_event(event, layout.checkboxes[2]));
        r = r.merge(self.focus_mode.handle_event(event, layout.switches[0]));
        r = r.merge(self.wrap_lines.handle_event(event, layout.switches[1]));
        r = r.merge(self.locked_switch.handle_event(event, layout.switches[2]));
        r = r.merge(self.volume.handle_event(event, layout.sliders[0]));
        r = r.merge(self.steps.handle_event(event, layout.sliders[1]));
        r = r.merge(self.disabled_slider.handle_event(event, layout.sliders[2]));
        r = r.merge(self.select.handle_event(event, layout.select));
        // The first progress bar mirrors the volume slider live.
        self.progress.set_value(self.volume.value() / 100.0);
        r
    }

    /// Advance switch springs. Returns `true` while animating.
    pub fn tick(&mut self, dt: f32) -> bool {
        let a = self.focus_mode.tick(dt);
        let b = self.wrap_lines.tick(dt);
        let c = self.locked_switch.tick(dt);
        a || b || c
    }

    pub fn render(&self, c: &mut Compositor, overlay: LayerId, content: Rect, theme: &Theme) {
        let layout = self.layout(content);
        let dim = theme.colors.text_mid.0;

        group_label(c, "TABS", content.x, content.y, theme);
        self.tabs.render(c, layout.tabs, theme);

        group_label(
            c,
            "CHECKBOXES",
            content.x,
            layout.checkboxes[0].y - LABEL_H,
            theme,
        );
        self.autosave.render(c, layout.checkboxes[0], theme);
        self.telemetry.render(c, layout.checkboxes[1], theme);
        self.locked.render(c, layout.checkboxes[2], theme);

        group_label(
            c,
            "SWITCHES",
            content.x,
            layout.switches[0].y - LABEL_H,
            theme,
        );
        let switch_labels = ["Focus mode", "Wrap long lines", "Disabled switch"];
        for ((switch, rect), label) in [&self.focus_mode, &self.wrap_lines, &self.locked_switch]
            .iter()
            .zip(layout.switches)
            .zip(switch_labels)
        {
            switch.render(c, rect, theme);
            text(
                c,
                label,
                13.0,
                400,
                rect.x + rect.w + 12.0,
                rect.y + (rect.h - 13.0 * 1.3) / 2.0,
                if switch.disabled {
                    theme.colors.text_dim.0
                } else {
                    theme.colors.text.0
                },
            );
        }

        group_label(
            c,
            "SLIDERS",
            layout.sliders[0].x,
            layout.sliders[0].y - LABEL_H,
            theme,
        );
        self.volume.render(c, layout.sliders[0], theme);
        text(
            c,
            &format!("{:.0}", self.volume.value()),
            12.0,
            500,
            layout.sliders[0].x + layout.sliders[0].w + 12.0,
            layout.sliders[0].y + 7.0,
            dim,
        );
        self.steps.render(c, layout.sliders[1], theme);
        text(
            c,
            &format!("step 1 — {:.0}", self.steps.value()),
            12.0,
            500,
            layout.sliders[1].x + layout.sliders[1].w + 12.0,
            layout.sliders[1].y + 7.0,
            dim,
        );
        self.disabled_slider.render(c, layout.sliders[2], theme);

        group_label(
            c,
            "PROGRESS",
            layout.progresses[0].x,
            layout.progresses[0].y - LABEL_H,
            theme,
        );
        self.progress.render(c, layout.progresses[0], theme);
        self.progress_ok.render(c, layout.progresses[1], theme);
        self.progress_err.render(c, layout.progresses[2], theme);

        group_label(
            c,
            "SELECT",
            layout.select.x,
            layout.select.y - LABEL_H,
            theme,
        );
        self.select.render(c, layout.select, theme);
        self.select
            .render_dropdown(c, overlay, layout.select, theme);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every rect a layout hands out, flattened (for bounds checks).
    fn all_rects(l: &Layout) -> Vec<Rect> {
        let mut v = vec![l.tabs, l.select];
        v.extend(l.checkboxes);
        v.extend(l.switches);
        v.extend(l.sliders);
        v.extend(l.progresses);
        v
    }

    /// Narrow viewport (~600px window): the columns must stack instead of
    /// cropping column B, and nothing may reach past `content.w`.
    #[test]
    fn forms_columns_stack_in_narrow_content_without_overflow() {
        let theme = Theme::hoff();
        let section = FormsSection::new(&theme);
        let content = Rect::new(288.0, 80.0, 600.0, 700.0);
        let layout = section.layout(content);

        // Column B starts at the left edge, below column A.
        assert_eq!(layout.sliders[0].x, content.x, "column B must stack left");
        assert!(
            layout.sliders[0].y > layout.switches[2].y + layout.switches[2].h,
            "stacked column B must start below column A"
        );

        let right = content.x + content.w;
        for r in all_rects(&layout) {
            assert!(
                r.x + r.w <= right + 0.5,
                "rect {:?} overflows content right edge {right}",
                (r.x, r.y, r.w, r.h)
            );
        }
    }

    /// Wide viewport (~1600px window): the two columns must actually use
    /// the available width instead of huddling at fixed offsets.
    #[test]
    fn forms_columns_spread_in_wide_content() {
        let theme = Theme::hoff();
        let section = FormsSection::new(&theme);
        let content = Rect::new(288.0, 80.0, 1272.0, 700.0);
        let layout = section.layout(content);

        // Two real columns, side by side.
        assert!(
            layout.sliders[0].x > content.x,
            "wide content must keep column B beside column A"
        );
        assert_eq!(layout.sliders[0].y - LABEL_H, content.y);

        // The columns span well past the old fixed 700px footprint.
        let span = layout.progresses[0].x + layout.progresses[0].w - content.x;
        assert!(
            span >= content.w * 0.6,
            "columns use only {span:.0}px of {:.0}px content",
            content.w
        );
        // …but each column stays clamped for legibility.
        assert!(layout.progresses[0].w <= COL_MAX_W);
        // The select follows its column instead of the old fixed x+380.
        assert_eq!(layout.select.x, layout.sliders[0].x);
        assert!(layout.select.w <= layout.progresses[0].w);
    }

    /// Every tab label must sit FOLGADO inside its segment: the GOLDEN_SPEC
    /// flags cramped tabs as the broken state. We require at least 16px of
    /// horizontal breathing room on each side (text centered in the segment).
    #[test]
    fn forms_tabs_keep_every_label_folgado() {
        let theme = Theme::hoff();
        let section = FormsSection::new(&theme);
        // A representative content rect (matches the page layout origin).
        let content = Rect::new(288.0, 80.0, 760.0, 700.0);
        let layout = section.layout(content);
        let rects = section.tabs.item_rects(layout.tabs);
        let style = TypographyScale::hoff().base_2sm();

        assert_eq!(rects.len(), section.tabs.labels.len());
        for (label, rect) in section.tabs.labels.iter().zip(&rects) {
            let (text_w, _) = TextMeasurer::measure_styled(label, &style, None);
            let slack = rect.w - text_w;
            assert!(
                slack >= 32.0,
                "tab '{label}' is cramped: segment {:.1}px holds {:.1}px of text ({:.1}px slack, need >=32)",
                rect.w,
                text_w,
                slack
            );
        }
    }
}
