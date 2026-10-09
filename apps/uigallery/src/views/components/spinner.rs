//! Spinner page: three sizes. They spin only while toggled on by the
//! button, so an idle page settles (render-on-demand).
//!
//! Projected (task-75 phase 2b): the button, `spinner_page.toggle`, whose
//! `activate` starts or stops the spinners as its click does; its label
//! says which. The spinners themselves have no projection in comps yet.

use engine::widgets::prelude::{
    Button, ButtonSize, ButtonVariant, EventResult, Rect, Spinner, SpinnerSize, WidgetEvent,
};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::{ControlSize, Theme};
use accord::{Request, SemId, SnapshotBuilder};

use super::{After, Component, child_id, root_id, under};

pub struct SpinnerPage {
    root: Option<SemId>,
    spinners: [Spinner; 3],
    spinning: bool,
    toggle: Button,
    /// A screen reader focused the button (its ring shows); Escape or a
    /// click lets it go.
    focus: bool,
}

struct Layout {
    spinners: [Rect; 3],
    toggle: Rect,
    h: f32,
}

impl SpinnerPage {
    pub fn new() -> Self {
        let root = root_id("spinner_page");
        let toggle = Button::new(Self::label(false))
            .size(ButtonSize::Sm)
            .variant(ButtonVariant::Outline)
            .icon("play");
        let toggle = match child_id(root.as_ref(), "toggle") {
            Some(id) => toggle.sem(id),
            None => toggle,
        };
        Self {
            root,
            spinners: [
                Spinner::new().size(SpinnerSize::Sm),
                Spinner::new(),
                Spinner::new().size(SpinnerSize::Lg),
            ],
            spinning: false,
            toggle,
            focus: false,
        }
    }

    /// What the button says while the spinners spin, or not.
    fn label(spinning: bool) -> &'static str {
        if spinning {
            "Stop spinners"
        } else {
            "Start spinners"
        }
    }

    /// Start or stop the spinners, the button saying which: the click's
    /// path and a request's.
    fn set_spinning(&mut self, on: bool) {
        self.spinning = on;
        self.toggle.label = Self::label(on).to_string();
    }

    /// The reader's focus on the button, or off it.
    fn set_focus(&mut self, on: bool) {
        self.focus = on;
        self.toggle.set_focused(on);
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
        let blurred = matches!(event, WidgetEvent::MouseDown { .. }) && self.blur();
        let l = self.layout(content, theme);
        let r = self.toggle.handle_event(event, l.toggle);
        if r.clicked {
            self.set_spinning(!self.spinning);
            return EventResult::clicked();
        }
        if blurred {
            return r.merge(EventResult::changed());
        }
        r
    }

    pub fn tick(&mut self, dt: f32) -> bool {
        if !self.spinning {
            return false;
        }
        // with reduced motion on each stands still and asks for nothing;
        // every one is ticked (no short circuit)
        let mut moved = false;
        for s in &mut self.spinners {
            moved |= s.tick(dt);
        }
        moved
    }

    pub fn render(&mut self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let l = self.layout(content, theme);
        for (spinner, rect) in self.spinners.iter().zip(l.spinners) {
            spinner.render(c, rect, theme);
        }
        self.toggle.render(c, l.toggle, theme);
    }
}

impl Component for SpinnerPage {
    fn root(&self) -> Option<&SemId> {
        self.root.as_ref()
    }

    fn project(&self, b: &mut SnapshotBuilder, root: &SemId) {
        let Some(id) = self.toggle.sem_id() else {
            return;
        };
        self.toggle.project(b, id, Some(root));
        if self.focus {
            b.focus(id);
        }
    }

    /// The button's `activate` starts or stops the spinners, as its click.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        if self.toggle.sem_id() == Some(&req.target) && req.action.as_str() == "activate" {
            self.set_spinning(!self.spinning);
            return Ok(After::Nothing);
        }
        Err(format!(
            "the spinner page has no {} on {}",
            req.action, req.target
        ))
    }

    fn focus(&mut self, id: &SemId) -> bool {
        if !self.root.as_ref().is_some_and(|root| under(root, id)) {
            return false;
        }
        self.set_focus(self.toggle.sem_id() == Some(id));
        true
    }

    fn blur(&mut self) -> bool {
        let had = self.focus;
        self.set_focus(false);
        had
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FRAME: f32 = 1.0 / 60.0;

    /// Started, the page asks for every frame; with the system's "reduce
    /// motion" on (task-75 phase 2a) it asks for none, and asks again once
    /// the preference is off.
    #[test]
    fn the_page_asks_for_no_frame_with_reduced_motion() {
        let mut page = SpinnerPage::new();
        assert!(!page.tick(FRAME), "at rest until the person starts it");
        page.spinning = true;
        assert!(page.tick(FRAME), "with motion");
        engine::compose::animation::set_reduced_motion(true);
        let moved = page.tick(FRAME);
        engine::compose::animation::set_reduced_motion(false);
        assert!(!moved, "reduced: it stands still");
        assert!(page.tick(FRAME), "motion again");
    }
}
