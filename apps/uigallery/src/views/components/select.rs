//! Select page: the pill select and its dropdown (drawn on the overlay
//! layer so it floats over the page). The open dropdown takes events
//! before everything beneath it; Escape closes it.
//!
//! Projected (task-75 phase 2b): the select, `select_page.select`, named
//! "appearance" (what its options choose), as the comps projects it, with
//! its options while open: `open`, `close`, `select` with an option, and an
//! option's own `select`, each doing what the click on the pill or on the
//! option does. The focus a reader or the keyboard asks for is the
//! pill's, or the open option's, which the panel highlights (task-75
//! phase 2c).

use engine::widgets::prelude::{EventResult, Rect, Select, WidgetEvent};
use engine::drawing::compositor::{Compositor, LayerId};
use engine::styling::theme::{ControlSize, Theme};
use accord::{Request, SemId, SnapshotBuilder, Value};

use super::{After, Component, child_id, key_under, root_id, under};

pub struct SelectPage {
    root: Option<SemId>,
    select: Select,
    /// The open option a screen reader or the keyboard focused; it goes
    /// when the list closes.
    option: Option<usize>,
}

impl SelectPage {
    pub fn new() -> Self {
        let root = root_id("select_page");
        let select = Select::new(
            ["System default", "Always dark", "Always light", "Scheduled"],
            0,
        );
        let select = match child_id(root.as_ref(), "select") {
            Some(id) => select.sem(id),
            None => select,
        };
        Self {
            root,
            select,
            option: None,
        }
    }

    /// The option behind `id`, `select_page.select[<i>]`.
    fn option_at(&self, id: &SemId) -> Option<usize> {
        let key = key_under(self.select.sem_id()?, id)?;
        key.parse().ok().filter(|i| *i < self.select.options.len())
    }

    fn rect(content: Rect, theme: &Theme) -> Rect {
        Rect::new(
            content.x,
            content.y,
            content.w.min(theme.size.menu_w),
            theme.control.height(ControlSize::Md),
        )
    }

    pub fn content_height(&self, _content: Rect, theme: &Theme) -> f32 {
        theme.control.height(ControlSize::Md)
    }

    pub fn is_open(&self) -> bool {
        self.select.is_open()
    }

    /// Escape: close the dropdown, else blur. `false` when neither applied.
    pub fn handle_escape(&mut self) -> bool {
        self.option = None;
        if self.select.is_open() {
            self.select.close();
            true
        } else if self.select.is_focused() {
            self.select.set_focused(false);
            true
        } else {
            false
        }
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
    ) -> EventResult {
        self.select
            .handle_event(event, Self::rect(content, theme), theme)
    }

    pub fn render(&self, c: &mut Compositor, overlay: LayerId, content: Rect, theme: &Theme) {
        let rect = Self::rect(content, theme);
        self.select.render(c, rect, theme);
        self.select.render_dropdown(c, overlay, rect, theme);
    }
}

impl Component for SelectPage {
    fn root(&self) -> Option<&SemId> {
        self.root.as_ref()
    }

    fn project(&self, b: &mut SnapshotBuilder, root: &SemId) {
        let Some(id) = self.select.sem_id() else {
            return;
        };
        self.select.project(b, id, "appearance", Some(root));
        match self.option.filter(|_| self.select.is_open()) {
            Some(i) => b.focus(id.key(i.to_string())),
            None if self.select.is_focused() => b.focus(id),
            None => {}
        }
    }

    /// `open` and `close` open and close the list, as a click on the pill;
    /// `select` with an option, or an open option's own `select`, picks it
    /// and closes the list, as a click on the option.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        let on_select = self.select.sem_id() == Some(&req.target);
        let picked = match (req.action.as_str(), &req.param) {
            ("open", None) if on_select => {
                self.select.open();
                true
            }
            ("close", None) if on_select => {
                self.select.close();
                true
            }
            ("select", Some(Value::Text(label))) if on_select => self.select.select_label(label),
            ("select", None) => self
                .option_at(&req.target)
                .is_some_and(|i| self.select.select_index(i)),
            _ => false,
        };
        if !picked {
            return Err(format!(
                "the select page has no {} on {}",
                req.action, req.target
            ));
        }
        if !self.select.is_open() {
            self.option = None;
        }
        Ok(After::Nothing)
    }

    /// The pill takes the page's focus (its ring); an open option is held
    /// and projected; another node of the page takes it from both.
    fn focus(&mut self, id: &SemId) -> bool {
        if !self.root.as_ref().is_some_and(|root| under(root, id)) {
            return false;
        }
        let pill = self.select.sem_id() == Some(id);
        self.option = self.option_at(id).filter(|_| self.select.is_open());
        self.select.set_focused(pill || self.option.is_some());
        self.select.set_highlighted(self.option);
        true
    }
}
