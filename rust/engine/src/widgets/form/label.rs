//! Label: the text that names a form control (the kit's Field label,
//! `components/Field/index.tsx:36`, 12/16 medium), or a section's caption
//! (the uppercase group heading of the HOFF accordion). Every value is
//! `theme.form.label`'s.
//!
//! A label wraps to the width it is given ([`Label::size`] measures it so)
//! and is drawn with the style it is measured with. It is text only: the
//! control it names keeps its own focus, and the owner decides what a
//! press on it does (a field focuses its box, [`super::field::Field`]).
//!
//! Projected ([`Label::project`]): a `Role::Label` with its text, enabled
//! unless disabled.

use accord::{NodeBuilder, Role, SemId, SnapshotBuilder};

use crate::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::drawing::text::{OverflowWrap, TextMeasurer, TextStyle};
use crate::styling::color::Color;
use crate::styling::theme::Theme;
use crate::widgets::widget::{Rect, with_alpha};

/// What a label is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LabelKind {
    /// Names a control: the kit's form label.
    #[default]
    Field,
    /// Heads a section: the caption over a group.
    Caption,
}

/// A label, retained so a page can bind and project it.
#[derive(Clone, Debug, PartialEq)]
pub struct Label {
    text: String,
    kind: LabelKind,
    disabled: bool,
    sem: Option<SemId>,
}

impl Label {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            kind: LabelKind::Field,
            disabled: false,
            sem: None,
        }
    }

    /// A section's caption instead of a control's label.
    pub fn caption(mut self) -> Self {
        self.kind = LabelKind::Caption;
        self
    }

    /// The label of a disabled control: faded as the control is.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    /// The id the drawn text binds to.
    pub fn sem(mut self, id: SemId) -> Self {
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn kind(&self) -> LabelKind {
        self.kind
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// The style the label is measured and drawn with; wrapped, a word
    /// wider than the line (a larger text, a longer language) breaks
    /// between its letters.
    pub fn style(&self, theme: &Theme) -> TextStyle {
        let l = &theme.form.label;
        theme
            .typography
            .run(match self.kind {
                LabelKind::Field => l.text,
                LabelKind::Caption => l.caption,
            })
            .with_overflow_wrap(OverflowWrap::Anywhere)
    }

    fn color(&self, theme: &Theme) -> [f32; 4] {
        let l = &theme.form.label;
        let color: Color = match self.kind {
            LabelKind::Field => l.color,
            LabelKind::Caption => l.caption_color,
        };
        let alpha = if self.disabled { l.disabled_alpha } else { 1.0 };
        with_alpha(color, color.0[3] * alpha)
    }

    /// The box the label takes wrapped to `max_w` (`None`: one line), as
    /// wide as its widest line, rounded up to whole pixels.
    pub fn size(&self, max_w: Option<f32>, theme: &Theme) -> (f32, f32) {
        let (w, h) = TextMeasurer::measure_styled(&self.text, &self.style(theme), max_w);
        let w = w.ceil();
        (max_w.map_or(w, |m| w.min(m)), h)
    }

    /// Draws the label with its top-left at `(x, y)`, wrapped to `width`
    /// (`None`: one line); binds that box when it has an id.
    pub fn draw(
        &self,
        c: &mut Compositor,
        layer: LayerId,
        x: f32,
        y: f32,
        width: Option<f32>,
        theme: &Theme,
    ) {
        let style = self.style(theme);
        if let Some(id) = &self.sem {
            let (w, h) = self.size(width, theme);
            c.bind_to_layer(layer, id, [x, y, w, h]);
        }
        c.push_to_layer(
            layer,
            SceneNode::Text {
                key: TextNodeKey::from_style(&self.text, &style, width),
                x,
                y,
                color: self.color(theme),
            },
        );
    }

    /// Draws the label in `rect`, wrapped to its width.
    pub fn render(&self, c: &mut Compositor, rect: Rect, theme: &Theme) {
        self.draw(c, LayerId::DEFAULT, rect.x, rect.y, Some(rect.w), theme);
    }

    /// Declares the label as `id`: a `Role::Label` with its text, enabled
    /// unless disabled. Returns the node for the owner to add to.
    #[track_caller]
    pub fn project<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        parent: Option<&SemId>,
    ) -> NodeBuilder<'b> {
        let mut node = b.node(id, Role::Label, self.text.as_str());
        if let Some(parent) = parent {
            node.parent(parent);
        }
        node.enabled(!self.disabled);
        node
    }
}
