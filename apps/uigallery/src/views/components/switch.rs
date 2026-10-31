//! Switch page: two live spring toggles and a disabled one, each with a
//! caption. Tab reaches the enabled ones (the view's keyboard, in the
//! order of the projection), Space flips.
//!
//! Projected (task-75 phase 2b): each switch, `switch_page.switch[<i>]`, as
//! the comps projects it, named by its caption; `toggle` flips it (the
//! knob springs) as a click does. The focus a reader or the keyboard
//! asks for is the page's own (the ring).

use accord::{Request, SemId, SnapshotBuilder};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::{ControlSize, Theme};
use engine::widgets::prelude::{EventResult, Rect, Switch, WidgetEvent};

use super::{After, Component, Focus, child_id, root_id, text, under};

const LABELS: [&str; 3] = ["Focus mode", "Wrap long lines", "Disabled switch"];

pub struct SwitchPage {
    root: Option<SemId>,
    switches: [Switch; 3],
    focus: Focus,
}

impl SwitchPage {
    pub fn new(theme: &Theme) -> Self {
        let root = root_id("switch_page");
        let family = child_id(root.as_ref(), "switch");
        let switches = [
            Switch::new(true).with_motion(&theme.motion),
            Switch::new(false).with_motion(&theme.motion),
            Switch::new(false).with_motion(&theme.motion).disabled(true),
        ];
        let mut i = 0;
        let switches = switches.map(|s| {
            let id = family.as_ref().map(|f| f.key(i.to_string()));
            i += 1;
            match id {
                Some(id) => s.sem(id),
                None => s,
            }
        });
        Self {
            root,
            switches,
            focus: Focus::default(),
        }
    }

    /// The switch behind `id`.
    fn at(&self, id: &SemId) -> Option<usize> {
        self.switches.iter().position(|s| s.sem_id() == Some(id))
    }

    fn rects(content: Rect, theme: &Theme) -> [Rect; 3] {
        let row = theme.control.height(ControlSize::Xs) + theme.spacing.xs;
        let [w, h] = theme.control.switch_size;
        std::array::from_fn(|i| Rect::new(content.x, content.y + i as f32 * row, w, h))
    }

    pub fn content_height(&self, _content: Rect, theme: &Theme) -> f32 {
        (theme.control.height(ControlSize::Xs) + theme.spacing.xs) * 3.0
    }

    fn set_focus(&mut self, slot: Option<usize>) {
        self.focus.slot = slot;
        self.switches[0].set_focused(slot == Some(0));
        self.switches[1].set_focused(slot == Some(1));
    }

    pub fn handle_escape(&mut self) -> bool {
        if self.focus.clear() {
            self.set_focus(None);
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
        let mut r = EventResult::IGNORED;
        for (s, rect) in self.switches.iter_mut().zip(Self::rects(content, theme)) {
            r = r.merge(s.handle_event(event, rect));
        }
        r
    }

    /// Knob springs. `true` while any is moving.
    pub fn tick(&mut self, dt: f32) -> bool {
        let mut animating = false;
        for s in &mut self.switches {
            animating |= s.tick(dt);
        }
        animating
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let style = theme.typography.base_2r();
        for ((s, rect), label) in self
            .switches
            .iter()
            .zip(Self::rects(content, theme))
            .zip(LABELS)
        {
            s.render(c, rect, theme);
            text(
                c,
                label,
                style.font_size,
                style.font_weight,
                (
                    rect.right() + theme.spacing.md,
                    rect.y + (rect.h - style.line_height) / 2.0,
                ),
                if s.disabled {
                    theme.colors.text_dim.0
                } else {
                    theme.colors.text.0
                },
                theme,
            );
        }
    }
}

impl Component for SwitchPage {
    fn root(&self) -> Option<&SemId> {
        self.root.as_ref()
    }

    fn project(&self, b: &mut SnapshotBuilder, root: &SemId) {
        for (switch, label) in self.switches.iter().zip(LABELS) {
            if let Some(id) = switch.sem_id() {
                switch.project(b, id, label, Some(root));
            }
        }
        let focused = self.focus.slot.and_then(|i| self.switches.get(i));
        if let Some(id) = focused.and_then(Switch::sem_id) {
            b.focus(id);
        }
    }

    /// A switch's `toggle` flips it, as its click does.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        match (self.at(&req.target), req.action.as_str()) {
            (Some(i), "toggle") => {
                self.switches[i].toggle();
                Ok(After::Nothing)
            }
            _ => Err(format!(
                "the switch page has no {} on {}",
                req.action, req.target
            )),
        }
    }

    /// An enabled switch takes the page's focus (its ring); another node
    /// of the page takes it from the switches.
    fn focus(&mut self, id: &SemId) -> bool {
        if !self.root.as_ref().is_some_and(|root| under(root, id)) {
            return false;
        }
        let slot = self.at(id).filter(|i| !self.switches[*i].disabled);
        self.set_focus(slot);
        true
    }
}
