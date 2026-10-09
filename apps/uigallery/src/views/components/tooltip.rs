//! Tooltip page: hover the button; the tooltip waits, then places
//! itself above the anchor (or below when there is no room). It draws
//! on the topmost layer so it floats over everything.
//!
//! Projected (task-75 phase 2b): the button, `tooltip_page.button`, as the
//! comps projects it, described by the tooltip's text (the `Tooltip` has no
//! projection of its own: what it says is what the button is about, heard
//! whether or not the pointer hovers). The button does nothing but press,
//! so its `activate` changes nothing.

use comps::prelude::{Button, ButtonVariant, EventResult, Rect, Tooltip, WidgetEvent};
use engine::compositor::{Compositor, LayerId};
use engine::theme::Theme;
use sem::{Request, SemId, SnapshotBuilder};

use super::{After, Component, child_id, root_id, under};

pub struct TooltipPage {
    root: Option<SemId>,
    button: Button,
    tooltip: Tooltip,
    /// A screen reader focused the button (its ring shows); Escape or a
    /// click lets it go.
    focus: bool,
}

impl TooltipPage {
    pub fn new() -> Self {
        let root = root_id("tooltip_page");
        let button = Button::new("Hover me")
            .variant(ButtonVariant::Outline)
            .icon("eye");
        let button = match child_id(root.as_ref(), "button") {
            Some(id) => button.sem(id),
            None => button,
        };
        Self {
            root,
            button,
            tooltip: Tooltip::new(
                "Tooltips wait 450 ms, then place themselves above the anchor, or below when there is no room.",
            ),
            focus: false,
        }
    }

    /// The reader's focus on the button, or off it.
    fn set_focus(&mut self, on: bool) {
        self.focus = on;
        self.button.set_focused(on);
    }

    fn rect(&self, content: Rect, theme: &Theme) -> Rect {
        let (w, h) = self.button.preferred_size(theme);
        Rect::new(content.x, content.y, w, h)
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        self.rect(content, theme).h
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
    ) -> EventResult {
        let blurred = matches!(event, WidgetEvent::MouseDown { .. }) && self.blur();
        let rect = self.rect(content, theme);
        let mut r = self.button.handle_event(event, rect);
        if blurred {
            r = r.merge(EventResult::changed());
        }
        if matches!(event, WidgetEvent::MouseMove { .. })
            && self.tooltip.set_hover(self.button.is_hovered(), rect)
        {
            r = r.merge(EventResult::changed());
        }
        r
    }

    /// Advance the tooltip delay. `true` while it needs frames.
    pub fn tick(&mut self, dt: f32) -> bool {
        self.tooltip.tick(dt)
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        self.button.render(c, self.rect(content, theme), theme);
    }

    /// The tooltip itself, on `layer`, positioned against the window.
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

impl Component for TooltipPage {
    fn root(&self) -> Option<&SemId> {
        self.root.as_ref()
    }

    fn project(&self, b: &mut SnapshotBuilder, root: &SemId) {
        let Some(id) = self.button.sem_id() else {
            return;
        };
        self.button
            .project(b, id, Some(root))
            .description(self.tooltip.text.as_str());
        if self.focus {
            b.focus(id);
        }
    }

    /// The button's `activate`: it presses, and nothing else happens, as
    /// for a click.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        if self.button.sem_id() == Some(&req.target) && req.action.as_str() == "activate" {
            return Ok(After::Nothing);
        }
        Err(format!(
            "the tooltip page has no {} on {}",
            req.action, req.target
        ))
    }

    fn focus(&mut self, id: &SemId) -> bool {
        if !self.root.as_ref().is_some_and(|root| under(root, id)) {
            return false;
        }
        self.set_focus(self.button.sem_id() == Some(id));
        true
    }

    fn blur(&mut self) -> bool {
        let had = self.focus;
        self.set_focus(false);
        had
    }
}
