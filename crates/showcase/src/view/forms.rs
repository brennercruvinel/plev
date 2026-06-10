//! Forms section: checkbox, switch, slider, progress, select, tabs.

use plev::compositor::{Compositor, LayerId};
use plev::theme::{Intent, Theme};
use plev::ui::widgets::{
    Checkbox, EventResult, ProgressBar, Rect, Select, Slider, Switch, Tabs, WidgetEvent,
};

use super::{group_label, text};

const COL_W: f32 = 320.0;
const COL_GAP: f32 = 60.0;
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
            tabs: Tabs::new(["Account", "Appearance", "Keybindings", "About"]),
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

    fn layout(&self, content: Rect) -> Layout {
        let (x, y) = (content.x, content.y);
        let col_b = x + COL_W + COL_GAP;

        // Column A: tabs, checkboxes, switches.
        let tabs = Rect::new(x, y + LABEL_H, COL_W, 34.0);
        let cb_y = tabs.y + tabs.h + GROUP_GAP + LABEL_H;
        let checkboxes = [
            Rect::new(x, cb_y, COL_W, ROW_H),
            Rect::new(x, cb_y + ROW_H, COL_W, ROW_H),
            Rect::new(x, cb_y + ROW_H * 2.0, COL_W, ROW_H),
        ];
        let sw_y = cb_y + ROW_H * 3.0 + GROUP_GAP + LABEL_H;
        let switches = [
            Rect::new(x, sw_y, 44.0, ROW_H),
            Rect::new(x, sw_y + ROW_H + 4.0, 44.0, ROW_H),
            Rect::new(x, sw_y + (ROW_H + 4.0) * 2.0, 44.0, ROW_H),
        ];

        // Column B: sliders, progress, select.
        let sl_y = y + LABEL_H;
        let sliders = [
            Rect::new(col_b, sl_y, COL_W, ROW_H),
            Rect::new(col_b, sl_y + ROW_H + 14.0, COL_W, ROW_H),
            Rect::new(col_b, sl_y + (ROW_H + 14.0) * 2.0, COL_W, ROW_H),
        ];
        let pr_y = sliders[2].y + ROW_H + GROUP_GAP + LABEL_H;
        let progresses = [
            Rect::new(col_b, pr_y, COL_W, 18.0),
            Rect::new(col_b, pr_y + 26.0, COL_W, 18.0),
            Rect::new(col_b, pr_y + 52.0, COL_W, 18.0),
        ];
        let select = Rect::new(
            col_b,
            progresses[2].y + 26.0 + GROUP_GAP + LABEL_H,
            240.0,
            32.0,
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

        group_label(c, "SLIDERS", layout.sliders[0].x, content.y, theme);
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
