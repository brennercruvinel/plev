//! Spinner page: three sizes. They spin only while toggled on by the
//! button, so an idle page settles (render-on-demand).

use comps::prelude::{
    Button, ButtonSize, ButtonVariant, EventResult, Rect, Spinner, SpinnerSize, WidgetEvent,
};
use engine::compositor::Compositor;
use engine::theme::{ControlSize, Theme};

pub struct SpinnerPage {
    spinners: [Spinner; 3],
    spinning: bool,
    toggle: Button,
}

struct Layout {
    spinners: [Rect; 3],
    toggle: Rect,
    h: f32,
}

impl SpinnerPage {
    pub fn new() -> Self {
        Self {
            spinners: [
                Spinner::new().size(SpinnerSize::Sm),
                Spinner::new(),
                Spinner::new().size(SpinnerSize::Lg),
            ],
            spinning: false,
            toggle: Button::new("Start spinners")
                .size(ButtonSize::Sm)
                .variant(ButtonVariant::Outline)
                .icon("play"),
        }
    }

    fn layout(&self, content: Rect, theme: &Theme) -> Layout {
        let slot = theme.control.height(ControlSize::Md);
        let step = slot + theme.spacing.lg;
        let spinners = std::array::from_fn(|i| {
            let px = self.spinners[i].size.px(theme);
            let x = content.x + i as f32 * step;
            Rect::new(x + (slot - px) / 2.0, content.y + (slot - px) / 2.0, px, px)
        });
        let (tw, th) = self.toggle.preferred_size(theme);
        let toggle = Rect::new(
            content.x + 3.0 * step,
            content.y + (slot - th) / 2.0,
            tw,
            th,
        );
        Layout {
            spinners,
            toggle,
            h: slot.max(th),
        }
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        self.layout(content, theme).h
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
    ) -> EventResult {
        let l = self.layout(content, theme);
        let r = self.toggle.handle_event(event, l.toggle);
        if r.clicked {
            self.spinning = !self.spinning;
            return EventResult::clicked();
        }
        r
    }

    pub fn tick(&mut self, dt: f32) -> bool {
        if !self.spinning {
            return false;
        }
        for s in &mut self.spinners {
            s.tick(dt);
        }
        true
    }

    pub fn render(&mut self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let l = self.layout(content, theme);
        for (spinner, rect) in self.spinners.iter().zip(l.spinners) {
            spinner.render(c, rect, theme);
        }
        self.toggle.label = if self.spinning {
            "Stop spinners".to_string()
        } else {
            "Start spinners".to_string()
        };
        self.toggle.render(c, l.toggle, theme);
    }
}
