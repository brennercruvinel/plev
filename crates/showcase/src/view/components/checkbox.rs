//! Checkbox page: checked, unchecked and disabled. Tab moves the focus
//! ring over the enabled ones, Escape blurs.
//!
//! Projected (task-75 phase 2b): each box, `checkbox_page.box[<i>]`, as the
//! comps projects it, named by its label; `toggle` checks or unchecks it as
//! a click does. The focus a reader asks for is the page's own (the ring
//! Tab moves).

use comps::prelude::{Checkbox, EventResult, Rect, WidgetEvent};
use engine::compositor::Compositor;
use engine::theme::{ControlSize, Theme};
use sem::{Request, SemId, SnapshotBuilder};

use super::{After, Component, EditKey, Focus, child_id, root_id, under};

pub struct CheckboxPage {
    root: Option<SemId>,
    boxes: [Checkbox; 3],
    focus: Focus,
}

impl CheckboxPage {
    pub fn new() -> Self {
        let root = root_id("checkbox_page");
        let family = child_id(root.as_ref(), "box");
        let boxes = [
            Checkbox::new(true).label("Autosave on focus loss"),
            Checkbox::new(false).label("Share anonymous usage data"),
            Checkbox::new(true)
                .label("Managed by your organization")
                .disabled(true),
        ];
        let mut i = 0;
        let boxes = boxes.map(|b| {
            let id = family.as_ref().map(|f| f.key(i.to_string()));
            i += 1;
            match id {
                Some(id) => b.sem(id),
                None => b,
            }
        });
        Self {
            root,
            boxes,
            focus: Focus::default(),
        }
    }

    /// The box behind `id`.
    fn at(&self, id: &SemId) -> Option<usize> {
        self.boxes.iter().position(|b| b.sem_id() == Some(id))
    }

    fn rects(content: Rect, theme: &Theme) -> [Rect; 3] {
        let h = theme.control.height(ControlSize::Xs);
        let w = content.w.min(theme.size.readable_max_w / 2.0);
        std::array::from_fn(|i| Rect::new(content.x, content.y + i as f32 * h, w, h))
    }

    pub fn content_height(&self, _content: Rect, theme: &Theme) -> f32 {
        theme.control.height(ControlSize::Xs) * 3.0
    }

    /// Focus slots: the two enabled boxes.
    fn set_focus(&mut self, slot: Option<usize>) {
        self.focus.slot = slot;
        self.boxes[0].set_focused(slot == Some(0));
        self.boxes[1].set_focused(slot == Some(1));
    }

    pub fn handle_edit_key(&mut self, key: EditKey) -> bool {
        if key == EditKey::Tab {
            self.set_focus(Some(self.focus.next(2)));
            return true;
        }
        false
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
        for (b, rect) in self.boxes.iter_mut().zip(Self::rects(content, theme)) {
            r = r.merge(b.handle_event(event, rect));
        }
        r
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        for (b, rect) in self.boxes.iter().zip(Self::rects(content, theme)) {
            b.render(c, rect, theme);
        }
    }
}

impl Component for CheckboxPage {
    fn root(&self) -> Option<&SemId> {
        self.root.as_ref()
    }

    fn project(&self, b: &mut SnapshotBuilder, root: &SemId) {
        for cb in &self.boxes {
            if let Some(id) = cb.sem_id() {
                let label = cb.label.clone().unwrap_or_default();
                cb.project(b, id, label, Some(root));
            }
        }
        let focused = self.focus.slot.and_then(|i| self.boxes.get(i));
        if let Some(id) = focused.and_then(Checkbox::sem_id) {
            b.focus(id);
        }
    }

    /// A box's `toggle` checks or unchecks it, as its click does.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        match (self.at(&req.target), req.action.as_str()) {
            (Some(i), "toggle") => {
                self.boxes[i].checked = !self.boxes[i].checked;
                Ok(After::Nothing)
            }
            _ => Err(format!(
                "the checkbox page has no {} on {}",
                req.action, req.target
            )),
        }
    }

    /// An enabled box takes the page's focus (its ring, the slot Tab
    /// moves from); another node of the page takes it from the boxes.
    fn focus(&mut self, id: &SemId) -> bool {
        if !self.root.as_ref().is_some_and(|root| under(root, id)) {
            return false;
        }
        let slot = self.at(id).filter(|i| !self.boxes[*i].disabled);
        self.set_focus(slot);
        true
    }
}
