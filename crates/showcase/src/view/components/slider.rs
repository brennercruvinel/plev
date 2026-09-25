//! Slider page: continuous, stepped and disabled, with the live value
//! beside each. Tab moves the focus ring over the enabled ones.
//!
//! Projected (task-75 phase 2b): each slider, `slider_page.slider[<i>]`, as
//! the comps projects it ("continuous", "stepped", "disabled"): `set_value`
//! moves it where a drag would, and the stepped one also takes
//! `increment` and `decrement`. The focus a reader asks for is the page's
//! own (the ring Tab moves).

use comps::prelude::{EventResult, Rect, Slider, WidgetEvent};
use engine::compositor::Compositor;
use engine::theme::{ControlSize, Theme};
use sem::{Request, SemId, SnapshotBuilder, Value};

use super::{After, Component, EditKey, Focus, child_id, root_id, text, under};

/// What each slider is called: what the page shows it for.
const LABELS: [&str; 3] = ["continuous", "stepped", "disabled"];

pub struct SliderPage {
    root: Option<SemId>,
    sliders: [Slider; 3],
    focus: Focus,
}

impl SliderPage {
    pub fn new() -> Self {
        let root = root_id("slider_page");
        let family = child_id(root.as_ref(), "slider");
        let sliders = [
            Slider::new(0.0, 100.0, 65.0),
            Slider::new(0.0, 10.0, 4.0).step(1.0),
            Slider::new(0.0, 100.0, 30.0).disabled(true),
        ];
        let mut i = 0;
        let sliders = sliders.map(|s| {
            let id = family.as_ref().map(|f| f.key(i.to_string()));
            i += 1;
            match id {
                Some(id) => s.sem(id),
                None => s,
            }
        });
        Self {
            root,
            sliders,
            focus: Focus::default(),
        }
    }

    /// The slider behind `id`.
    fn at(&self, id: &SemId) -> Option<usize> {
        self.sliders.iter().position(|s| s.sem_id() == Some(id))
    }

    /// The value caption's gutter to the right of every slider.
    fn gutter(theme: &Theme) -> f32 {
        theme.spacing.xxl * 2.0
    }

    fn rects(content: Rect, theme: &Theme) -> [Rect; 3] {
        let h = theme.control.height(ControlSize::Xs);
        let row = h + theme.spacing.md;
        let w = (content.w.min(theme.size.readable_max_w / 2.0) - Self::gutter(theme)).max(0.0);
        std::array::from_fn(|i| Rect::new(content.x, content.y + i as f32 * row, w, h))
    }

    pub fn content_height(&self, _content: Rect, theme: &Theme) -> f32 {
        (theme.control.height(ControlSize::Xs) + theme.spacing.md) * 3.0
    }

    fn set_focus(&mut self, slot: Option<usize>) {
        self.focus.slot = slot;
        self.sliders[0].set_focused(slot == Some(0));
        self.sliders[1].set_focused(slot == Some(1));
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
        for (s, rect) in self.sliders.iter_mut().zip(Self::rects(content, theme)) {
            r = r.merge(s.handle_event(event, rect, theme));
        }
        r
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let style = theme.typography.caption_r();
        let captions = [
            format!("{:.0}", self.sliders[0].value()),
            format!("step 1: {:.0}", self.sliders[1].value()),
            "disabled".to_string(),
        ];
        for ((s, rect), caption) in self
            .sliders
            .iter()
            .zip(Self::rects(content, theme))
            .zip(captions)
        {
            s.render(c, rect, theme);
            text(
                c,
                &caption,
                style.font_size,
                style.font_weight,
                rect.right() + theme.spacing.md,
                rect.y + (rect.h - style.line_height) / 2.0,
                theme.colors.text_mid.0,
            );
        }
    }
}

impl Component for SliderPage {
    fn root(&self) -> Option<&SemId> {
        self.root.as_ref()
    }

    fn project(&self, b: &mut SnapshotBuilder, root: &SemId) {
        for (slider, label) in self.sliders.iter().zip(LABELS) {
            if let Some(id) = slider.sem_id() {
                slider.project(b, id, label, Some(root));
            }
        }
        let focused = self.focus.slot.and_then(|i| self.sliders.get(i));
        if let Some(id) = focused.and_then(Slider::sem_id) {
            b.focus(id);
        }
    }

    /// `set_value` moves the slider to the value (in its range, on its
    /// step), where a drag would leave it; `increment` and `decrement` move
    /// the stepped one a step.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        let Some(i) = self.at(&req.target) else {
            return Err(format!("the slider page has no {}", req.target));
        };
        let slider = &mut self.sliders[i];
        match (req.action.as_str(), &req.param) {
            ("set_value", Some(Value::Num(v))) => slider.set_value(*v as f32),
            ("increment", None) => {
                slider.increment();
            }
            ("decrement", None) => {
                slider.decrement();
            }
            _ => {
                return Err(format!(
                    "the slider page has no {} on {}",
                    req.action, req.target
                ));
            }
        }
        Ok(After::Nothing)
    }

    /// An enabled slider takes the page's focus (its ring, the slot Tab
    /// moves from); another node of the page takes it from the sliders.
    fn focus(&mut self, id: &SemId) -> bool {
        if !self.root.as_ref().is_some_and(|root| under(root, id)) {
            return false;
        }
        let slot = self.at(id).filter(|i| !self.sliders[*i].disabled);
        self.set_focus(slot);
        true
    }
}
