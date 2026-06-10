//! Overlays section: modal, toasts, tooltip, context menu.

use plev::compositor::{Compositor, LayerId, SceneNode};
use plev::theme::{Intent, Theme};
use plev::ui::widgets::{
    Button, ButtonVariant, ContextMenu, EventResult, MenuEntry, Rect, Tooltip, WidgetEvent,
};

use super::{group_label, text};

const GAP: f32 = 12.0;
const LABEL_H: f32 = 24.0;
const GROUP_GAP: f32 = 34.0;
/// Minimum demo-area width before the proportional share kicks in (it is
/// still clamped to the content itself on narrow windows).
const MENU_AREA_MIN_W: f32 = 560.0;

pub enum OverlayAction {
    None,
    OpenModal { destructive: bool },
    PushToast(Intent),
}

/// The demo context menu (also used by the view's right-click handler).
pub fn demo_menu() -> ContextMenu {
    ContextMenu::new(vec![
        MenuEntry::item(1, "Stage file").icon("plus"),
        MenuEntry::item(2, "Copy path").icon("clipboard"),
        MenuEntry::item(3, "Reveal in file tree").icon("folder-open"),
        MenuEntry::Separator,
        MenuEntry::item(4, "Locked action").disabled(true),
        MenuEntry::item(5, "Discard changes")
            .icon("trash")
            .intent(Intent::Destructive),
    ])
}

pub fn menu_label(id: u64) -> &'static str {
    match id {
        1 => "Stage file",
        2 => "Copy path",
        3 => "Reveal in file tree",
        5 => "Discard changes",
        _ => "?",
    }
}

pub struct OverlaysSection {
    open_modal: Button,
    delete_modal: Button,
    toast_buttons: Vec<(Intent, Button)>,
    tooltip_button: Button,
    tooltip: Tooltip,
}

struct Layout {
    modal_buttons: [Rect; 2],
    toast_buttons: Vec<Rect>,
    tooltip_button: Rect,
    menu_area: Rect,
}

impl OverlaysSection {
    pub fn new() -> Self {
        Self {
            open_modal: Button::new("Open dialog").variant(ButtonVariant::Outline),
            delete_modal: Button::new("Delete repository…")
                .variant(ButtonVariant::Danger)
                .icon("trash"),
            toast_buttons: vec![
                (Intent::Neutral, Button::new("Neutral")),
                (
                    Intent::Constructive,
                    Button::new("Success").intent(Intent::Constructive),
                ),
                (
                    Intent::Destructive,
                    Button::new("Error").intent(Intent::Destructive),
                ),
                (
                    Intent::Informational,
                    Button::new("Info").intent(Intent::Informational),
                ),
            ],
            tooltip_button: Button::new("Hover me")
                .variant(ButtonVariant::Outline)
                .icon("eye"),
            tooltip: Tooltip::new(
                "Tooltips wait 450 ms, then place themselves above the anchor — or below when there is no room.",
            ),
        }
    }

    fn layout(&self, content: Rect) -> Layout {
        let (x, mut y) = (content.x, content.y);

        y += LABEL_H;
        let (w0, h0) = self.open_modal.preferred_size();
        let (w1, h1) = self.delete_modal.preferred_size();
        let modal_buttons = [Rect::new(x, y, w0, h0), Rect::new(x + w0 + GAP, y, w1, h1)];
        y += h0.max(h1) + GROUP_GAP;

        y += LABEL_H;
        let mut toast_buttons = Vec::with_capacity(self.toast_buttons.len());
        let mut tx = x;
        let mut row_h: f32 = 0.0;
        for (_, b) in &self.toast_buttons {
            let (w, h) = b.preferred_size();
            toast_buttons.push(Rect::new(tx, y, w, h));
            tx += w + GAP;
            row_h = row_h.max(h);
        }
        y += row_h + GROUP_GAP;

        y += LABEL_H;
        let (tw, th) = self.tooltip_button.preferred_size();
        let tooltip_button = Rect::new(x, y, tw, th);
        y += th + GROUP_GAP;

        y += LABEL_H;
        // The demo area grows with the window: 60% of the content width,
        // never below MENU_AREA_MIN_W (clamped to the content on narrow
        // windows instead of overflowing it).
        let menu_area = Rect::new(
            x,
            y,
            (content.w * 0.6).max(MENU_AREA_MIN_W).min(content.w),
            (content.y + content.h - y).max(120.0),
        );

        Layout {
            modal_buttons,
            toast_buttons,
            tooltip_button,
            menu_area,
        }
    }

    pub fn menu_area(&self, content: Rect) -> Rect {
        self.layout(content).menu_area
    }

    /// Natural height of the section (page scrolling needs it). The menu
    /// area normally fills the remaining viewport; on short windows its
    /// minimum height makes the page overflow (and scroll).
    pub fn content_height(&self, content: Rect) -> f32 {
        let area = self.layout(content).menu_area;
        area.y + area.h - content.y
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
    ) -> (EventResult, OverlayAction) {
        let layout = self.layout(content);
        let mut result = EventResult::IGNORED;
        let mut action = OverlayAction::None;

        let r = self.open_modal.handle_event(event, layout.modal_buttons[0]);
        if r.clicked {
            action = OverlayAction::OpenModal { destructive: false };
        }
        result = result.merge(r);

        let r = self
            .delete_modal
            .handle_event(event, layout.modal_buttons[1]);
        if r.clicked {
            action = OverlayAction::OpenModal { destructive: true };
        }
        result = result.merge(r);

        for ((intent, button), rect) in self.toast_buttons.iter_mut().zip(&layout.toast_buttons) {
            let r = button.handle_event(event, *rect);
            if r.clicked {
                action = OverlayAction::PushToast(*intent);
            }
            result = result.merge(r);
        }

        let r = self
            .tooltip_button
            .handle_event(event, layout.tooltip_button);
        result = result.merge(r);
        if matches!(event, WidgetEvent::MouseMove { .. })
            && self
                .tooltip
                .set_hover(self.tooltip_button.is_hovered(), layout.tooltip_button)
        {
            result = result.merge(EventResult::changed());
        }

        (result, action)
    }

    /// Advance the tooltip delay. Returns `true` while it needs frames.
    pub fn tick(&mut self, dt: f32) -> bool {
        self.tooltip.tick(dt)
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let layout = self.layout(content);

        group_label(
            c,
            "MODALS",
            content.x,
            layout.modal_buttons[0].y - LABEL_H,
            theme,
        );
        self.open_modal.render(c, layout.modal_buttons[0], theme);
        self.delete_modal.render(c, layout.modal_buttons[1], theme);

        group_label(
            c,
            "TOASTS (CLICK TO QUEUE)",
            content.x,
            layout.toast_buttons[0].y - LABEL_H,
            theme,
        );
        for ((_, button), rect) in self.toast_buttons.iter().zip(&layout.toast_buttons) {
            button.render(c, *rect, theme);
        }

        group_label(
            c,
            "TOOLTIP",
            content.x,
            layout.tooltip_button.y - LABEL_H,
            theme,
        );
        self.tooltip_button.render(c, layout.tooltip_button, theme);

        group_label(
            c,
            "CONTEXT MENU",
            content.x,
            layout.menu_area.y - LABEL_H,
            theme,
        );
        let area = layout.menu_area;
        c.push(SceneNode::RoundedRect {
            x: area.x,
            y: area.y,
            w: area.w,
            h: area.h,
            color: theme.glass.surface.0,
            corner_radius: theme.radius.lg,
            border_width: 1.0,
            border_color: theme.glass.edge_soft.0,
        });
        let hint = "Right-click anywhere in this area";
        let (hint_w, _) = plev::text::TextMeasurer::measure(hint, 14.0, None);
        text(
            c,
            hint,
            14.0,
            400,
            area.x + (area.w - hint_w) / 2.0,
            area.y + area.h / 2.0 - 10.0,
            theme.glass.text_placeholder.0,
        );
    }

    pub fn render_tooltip(
        &self,
        c: &mut Compositor,
        layer: LayerId,
        theme: &Theme,
        vw: f32,
        vh: f32,
    ) {
        self.tooltip.render(c, layer, theme, vw, vh);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The context-menu demo area must follow the window instead of
    /// freezing at the old 560px cap — and never overflow narrow content.
    #[test]
    fn menu_area_grows_with_wide_content_and_fits_narrow_content() {
        let section = OverlaysSection::new();

        let wide = Rect::new(288.0, 80.0, 1272.0, 700.0);
        let area = section.menu_area(wide);
        assert!(
            area.w > 560.0,
            "wide content must stretch the demo area, got {}",
            area.w
        );
        assert!((area.w - wide.w * 0.6).abs() < 0.5);

        let narrow = Rect::new(288.0, 80.0, 400.0, 700.0);
        let area = section.menu_area(narrow);
        assert!(area.w <= narrow.w, "narrow content must clamp the area");
    }
}
