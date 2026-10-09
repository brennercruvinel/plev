//! Context Menu page: an area to right-click. The view owns the overlay
//! stack and opens the menu (`demo_menu`) at the pointer.

use engine::widgets::prelude::{ContextMenu, EventResult, MenuEntry, Rect, WidgetEvent};
use engine::drawing::compositor::{Compositor, SceneNode};
use engine::drawing::text::TextMeasurer;
use engine::styling::theme::{Intent, Theme};

use super::text;

/// The demo context menu.
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

pub struct ContextMenuPage;

impl ContextMenuPage {
    pub fn new() -> Self {
        Self
    }

    /// The demo area: 60% of the content width, never narrower than the
    /// menu's own width times two (clamped to the content), as tall as
    /// the viewport leaves.
    pub fn area(content: Rect, theme: &Theme) -> Rect {
        Rect::new(
            content.x,
            content.y,
            (content.w * 0.6)
                .max(theme.size.menu_w * 2.0)
                .min(content.w),
            content.h.max(theme.size.card_min_w / 2.0),
        )
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        Self::area(content, theme).h
    }

    pub fn handle_event(
        &mut self,
        _event: &WidgetEvent,
        _content: Rect,
        _theme: &Theme,
    ) -> EventResult {
        EventResult::IGNORED
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let area = Self::area(content, theme);
        c.push(SceneNode::RoundedRect {
            x: area.x,
            y: area.y,
            w: area.w,
            h: area.h,
            color: theme.glass.surface.0,
            corner_radius: theme.radius.lg,
            border_width: theme.control.edge_width,
            border_color: theme.glass.edge_soft.0,
        });
        let hint = "Right-click anywhere in this area";
        let style = theme.typography.base_2r();
        let (hint_w, _) = TextMeasurer::measure_styled(hint, &style, None);
        text(
            c,
            hint,
            style.font_size,
            style.font_weight,
            area.x + (area.w - hint_w) / 2.0,
            area.y + (area.h - style.line_height) / 2.0,
            theme.glass.text_placeholder.0,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The area follows the window and never overflows narrow content.
    #[test]
    fn area_grows_with_wide_content_and_fits_narrow_content() {
        let theme = Theme::hoff();
        let wide = Rect::new(288.0, 80.0, 1272.0, 700.0);
        let area = ContextMenuPage::area(wide, &theme);
        assert!((area.w - wide.w * 0.6).abs() < 0.5);
        let narrow = Rect::new(288.0, 80.0, 400.0, 700.0);
        let area = ContextMenuPage::area(narrow, &theme);
        assert!(area.w <= narrow.w, "narrow content must clamp the area");
    }
}
