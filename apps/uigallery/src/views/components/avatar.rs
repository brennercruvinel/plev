//! Avatar page: initials in three sizes, one online.

use comps::content::{Avatar, AvatarSize};
use comps::prelude::{EventResult, Rect, WidgetEvent};
use engine::compositor::Compositor;
use engine::theme::Theme;

use super::flow;

pub struct AvatarPage {
    avatars: [Avatar; 3],
}

impl AvatarPage {
    pub fn new() -> Self {
        Self {
            avatars: [
                Avatar::new("ann").size(AvatarSize::Sm),
                Avatar::new("brenner").online(true),
                Avatar::new("hoff").size(AvatarSize::Lg),
            ],
        }
    }

    fn layout(&self, content: Rect, theme: &Theme) -> (Vec<Rect>, f32) {
        flow(
            content,
            content.y,
            theme.spacing.md,
            self.avatars.iter().map(|a| a.preferred_size(theme)),
        )
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        self.layout(content, theme).1
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
        let (rects, _) = self.layout(content, theme);
        for (avatar, rect) in self.avatars.iter().zip(rects) {
            avatar.render(c, rect, theme);
        }
    }
}
