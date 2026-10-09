//! Icon Button page: solid, outline, ghost and toolbar variants, two
//! sizes, one disabled. Projected (task-75 phase 2b): each button,
//! `icon_button_page.button[<icon>]`, named by its icon; it does nothing
//! but press, so its `activate` changes nothing.

use comps::prelude::{
    EventResult, IconButton, IconButtonSize, IconButtonVariant, Rect, WidgetEvent,
};
use engine::compositor::Compositor;
use engine::theme::Theme;
use sem::{Request, SemId, SnapshotBuilder};

use super::{After, Component, child_id, flow, root_id, under};

pub struct IconButtonPage {
    root: Option<SemId>,
    buttons: Vec<IconButton>,
    /// The button a screen reader focused (its ring shows); Escape or a
    /// click lets it go.
    focus: Option<usize>,
}

impl IconButtonPage {
    pub fn new() -> Self {
        let root = root_id("icon_button_page");
        let family = child_id(root.as_ref(), "button");
        let buttons = vec![
            IconButton::new("copy"),
            IconButton::new("search").variant(IconButtonVariant::Outline),
            IconButton::new("settings").variant(IconButtonVariant::Ghost),
            IconButton::new("trash")
                .variant(IconButtonVariant::Toolbar)
                .active(true),
            IconButton::new("save").size(IconButtonSize::Sm),
            IconButton::new("play").size(IconButtonSize::Md),
            IconButton::new("undo").disabled(true),
        ];
        let buttons = buttons
            .into_iter()
            .map(|b| match &family {
                Some(f) => {
                    let id = f.key(b.icon);
                    b.sem(id)
                }
                None => b,
            })
            .collect();
        Self {
            root,
            buttons,
            focus: None,
        }
    }

    fn layout(&self, content: Rect, theme: &Theme) -> (Vec<Rect>, f32) {
        flow(
            content,
            content.y,
            theme.spacing.md,
            self.buttons.iter().map(|b| b.preferred_size(theme)),
        )
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        self.layout(content, theme).1
    }

    /// The button behind `id`.
    fn at(&self, id: &SemId) -> Option<usize> {
        self.buttons.iter().position(|b| b.sem_id() == Some(id))
    }

    /// The reader's focus on the button at `at`, or on none.
    fn set_focus(&mut self, at: Option<usize>) {
        self.focus = at;
        for (i, b) in self.buttons.iter_mut().enumerate() {
            b.set_focused(at == Some(i));
        }
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
    ) -> EventResult {
        let mut result = EventResult::IGNORED;
        if matches!(event, WidgetEvent::MouseDown { .. }) && self.blur() {
            result = EventResult::changed();
        }
        let (rects, _) = self.layout(content, theme);
        for (button, rect) in self.buttons.iter_mut().zip(rects) {
            result = result.merge(button.handle_event(event, rect));
        }
        result
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let (rects, _) = self.layout(content, theme);
        for (button, rect) in self.buttons.iter().zip(rects) {
            button.render(c, rect, theme);
        }
    }
}

impl Component for IconButtonPage {
    fn root(&self) -> Option<&SemId> {
        self.root.as_ref()
    }

    fn project(&self, b: &mut SnapshotBuilder, root: &SemId) {
        for button in &self.buttons {
            if let Some(id) = button.sem_id() {
                button.project(b, id, button.icon, Some(root));
            }
        }
        let focused = self.focus.and_then(|i| self.buttons.get(i));
        if let Some(id) = focused.and_then(IconButton::sem_id) {
            b.focus(id);
        }
    }

    /// A button's `activate`: it presses, and nothing else happens, as
    /// for a click.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        match (self.at(&req.target), req.action.as_str()) {
            (Some(_), "activate") => Ok(After::Nothing),
            _ => Err(format!(
                "the icon button page has no {} on {}",
                req.action, req.target
            )),
        }
    }

    fn focus(&mut self, id: &SemId) -> bool {
        if !self.root.as_ref().is_some_and(|root| under(root, id)) {
            return false;
        }
        self.set_focus(self.at(id));
        true
    }

    fn blur(&mut self) -> bool {
        let had = self.focus.is_some();
        self.set_focus(None);
        had
    }
}
