//! Progress: the hoff core kit's video player rail
//! (`components/VideoPlayer`): a thin pill, its filled part a shade
//! deeper, and a head at the fill's end in
//! the ink (or the intent's color, made to read) that marks the value.
//! Every value is `theme.message.progress`.
//!
//! Behavior apart from the look: the bar holds a value from 0 to 1; a
//! reader hears it as a progress bar with that value and its range
//! ([`ProgressBar::project`]). It takes no input.

use accord::{Constraints, NodeBuilder, Role, SemId, SnapshotBuilder, Value};

use crate::drawing::compositor::Compositor;
use crate::styling::theme::{Intent, Theme};
use crate::widgets::recipe::rounded_rect;
use crate::widgets::widget::Rect;

/// Determinate progress bar, colored by intent.
#[derive(Clone, Debug)]
pub struct ProgressBar {
    /// Progress in 0.0..=1.0.
    value: f32,
    pub intent: Intent,
    sem: Option<SemId>,
}

/// Where a bar lands in its bounds: the track, the filled part and the
/// head, vertically centered.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProgressLayout {
    pub track: Rect,
    pub fill: Rect,
    pub head: Rect,
}

impl ProgressBar {
    pub fn new(value: f32) -> Self {
        Self {
            value: value.clamp(0.0, 1.0),
            intent: Intent::Neutral,
            sem: None,
        }
    }

    pub fn intent(mut self, intent: Intent) -> Self {
        self.intent = intent;
        self
    }

    pub fn sem(mut self, id: SemId) -> Self {
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    pub fn value(&self) -> f32 {
        self.value
    }

    pub fn set_value(&mut self, value: f32) {
        self.value = value.clamp(0.0, 1.0);
    }

    /// The narrowest the kit lays the rail out (`min-w-45`), as tall as
    /// the rail.
    pub fn preferred_size(theme: &Theme) -> (f32, f32) {
        (theme.message.progress.min_w, theme.message.progress.height)
    }

    pub fn layout(&self, bounds: Rect, theme: &Theme) -> ProgressLayout {
        let t = &theme.message.progress;
        let h = t.height.min(bounds.h.max(0.0));
        let w = bounds.w.max(0.0);
        let y = bounds.y + (bounds.h - h) / 2.0;
        let track = Rect::new(bounds.x, y, w, h);
        let fill = Rect::new(bounds.x, y, w * self.value, h);
        // the head ends where the fill does (`after:right-0`), and stays
        // on the track at 0
        let hw = t.head_w.min(w);
        let head = Rect::new(bounds.x + (fill.w - hw).max(0.0), y, hw, h);
        ProgressLayout { track, fill, head }
    }

    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        let t = &theme.message.progress;
        let l = self.layout(bounds, theme);
        if let Some(id) = &self.sem {
            c.bind(id, [l.track.x, l.track.y, l.track.w, l.track.h]);
        }
        let pill = |c: &mut Compositor, r: Rect, color: [f32; 4]| {
            if r.w > 0.0 && r.h > 0.0 {
                c.push(rounded_rect(r.x, r.y, r.w, r.h, r.h.min(r.w) / 2.0, color));
            }
        };
        pill(c, l.track, t.track.0);
        pill(c, l.fill, t.fill.of(self.intent).0);
        pill(c, l.head, t.head.of(self.intent).0);
    }

    /// A reader's view: a progress bar named `label`, its value from 0 to
    /// 1 in that range. It declares no action.
    pub fn project<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        parent: Option<&SemId>,
        label: &str,
    ) -> NodeBuilder<'b> {
        let mut node = b.node(id, Role::ProgressBar, label);
        if let Some(parent) = parent {
            node.parent(parent);
        }
        node.state("value", Value::Num(f64::from(self.value)))
            .constraints(Constraints {
                range: Some((0.0, 1.0)),
                ..Constraints::default()
            });
        node
    }
}
