//! Field: the kit's Field (`components/Field/index.tsx`), a text field
//! with what a form puts around it: a label row (the label, and at its
//! end a link such as "Forgot password?"), an icon at the start of the
//! box, at its end the check of a validated value or the eye that reveals
//! a password, and a message line under the box (an error, or a helper).
//! Every value is `theme.form`'s (`super::tokens`).
//!
//! The width is always the `Rect`'s, and the height is intrinsic: the
//! label, the link and the message wrap to the width
//! ([`Field::height_for`]), and the label row breaks in two, the link
//! under the label, when the two do not fit side by side. The box keeps
//! the recipe's height, and its text never wraps (see [`TextField`]).
//!
//! The editing is the inner [`TextField`]'s (`field.input`): an app routes
//! characters, editing keys, the clipboard and the input method there, as
//! it would to a bare text field.
//!
//! Projected ([`Field::project`]): the text field, named by the label,
//! described by the message, `invalid` with an error, `valid` when
//! validated, and secret when it is a password (the snapshot keeps its
//! length); beside it, under the same parent, the reveal button
//! (`<id>.reveal`, a toggle) and the link (`<id>.link`).

use accord::{Constraints, Role, SemId, SnapshotBuilder};

use crate::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::drawing::text::{TextMeasurer, TextStyle};
use crate::styling::color::Color;
use crate::styling::theme::Theme;
use crate::widgets::form::label::Label;
use crate::widgets::form::text_field::TextField;
use crate::widgets::form::tokens::Slot;
use crate::widgets::hit::hit_rect;
use crate::widgets::icons;
use crate::widgets::recipe::{focus_ring, focus_ring_inside};
use crate::widgets::widget::{EventResult, Rect, WidgetEvent, with_alpha};

/// The line under the box.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Message {
    /// What is wrong with the value, in the error color.
    Error(String),
    /// A hint about the value.
    Helper(String),
}

impl Message {
    pub fn text(&self) -> &str {
        match self {
            Message::Error(t) | Message::Helper(t) => t,
        }
    }
}

/// The part of a field a point falls on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldPart {
    /// The input box (the reveal button excepted).
    Input,
    /// The button that reveals or hides a password.
    Reveal,
    /// The link at the end of the label row.
    Link,
    /// The label: a press on it focuses the box.
    Label,
}

/// Where each part of a field goes in its bounds. The text rects are
/// the boxes their text wraps in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FieldLayout {
    pub label: Option<Rect>,
    pub link: Option<Rect>,
    pub input: Rect,
    /// The leading icon.
    pub icon: Option<Rect>,
    /// The check of a validated value.
    pub check: Option<Rect>,
    /// The reveal button: the whole slot at the end of the box.
    pub reveal: Option<Rect>,
    pub message: Option<Rect>,
}

impl FieldLayout {
    /// The bottom of the lowest part.
    pub fn bottom(&self) -> f32 {
        self.message.map_or(self.input.bottom(), |m| m.bottom())
    }
}

/// A text field with its label, adornments and message line.
#[derive(Clone, Debug)]
pub struct Field {
    pub input: TextField,
    label: Option<Label>,
    link: Option<String>,
    message: Option<Message>,
    icon: Option<&'static str>,
    password: bool,
    revealed: bool,
    validated: bool,
    link_hovered: bool,
    /// The keyboard focus on the eye or the link (a reader's, or Tab):
    /// its ring shows. The box's own focus is its caret.
    part_focus: Option<FieldPart>,
    sem: Option<SemId>,
}

impl Field {
    pub fn new(placeholder: impl AsRef<str>) -> Self {
        Self {
            input: TextField::new(placeholder),
            label: None,
            link: None,
            message: None,
            icon: None,
            password: false,
            revealed: false,
            validated: false,
            link_hovered: false,
            part_focus: None,
            sem: None,
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(Label::new(label).disabled(self.input.disabled));
        self
    }

    /// A link at the end of the label row ("Forgot password?"); the owner
    /// runs it ([`FieldPart::Link`]).
    pub fn link(mut self, link: impl Into<String>) -> Self {
        self.link = Some(link.into());
        self
    }

    pub fn error(mut self, error: impl Into<String>) -> Self {
        self.message = Some(Message::Error(error.into()));
        self
    }

    pub fn helper(mut self, helper: impl Into<String>) -> Self {
        self.message = Some(Message::Helper(helper.into()));
        self
    }

    pub fn set_message(&mut self, message: Option<Message>) {
        self.message = message;
    }

    /// A Lucide icon at the start of the box.
    pub fn icon(mut self, icon: &'static str) -> Self {
        self.icon = Some(icon);
        self.fit_slots();
        self
    }

    /// A password: masked, with the eye that reveals it (unless
    /// validated, where the check takes the place, as in the kit).
    pub fn password(mut self, password: bool) -> Self {
        self.password = password;
        self.revealed = false;
        self.input.set_masked(password);
        self.fit_slots();
        self
    }

    /// The value was checked and is good: a check at the end of the box.
    pub fn validated(mut self, validated: bool) -> Self {
        self.set_validated(validated);
        self
    }

    pub fn set_validated(&mut self, validated: bool) {
        self.validated = validated;
        self.fit_slots();
    }

    pub fn with_text(mut self, text: &str) -> Self {
        self.input.set_text(text);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.input.disabled = disabled;
        if let Some(label) = &mut self.label {
            label.set_disabled(disabled);
        }
        self
    }

    /// The id the field's box binds to and projects as; its reveal button
    /// and its link are `<id>.reveal` and `<id>.link`.
    pub fn sem(mut self, id: SemId) -> Self {
        self.input = self.input.sem(id.clone());
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    pub fn reveal_id(&self) -> Option<SemId> {
        self.sem.as_ref()?.child("reveal").ok()
    }

    pub fn link_id(&self) -> Option<SemId> {
        self.sem.as_ref()?.child("link").ok()
    }

    pub fn label_text(&self) -> Option<&str> {
        self.label.as_ref().map(Label::text)
    }

    pub fn message(&self) -> Option<&Message> {
        self.message.as_ref()
    }

    pub fn is_password(&self) -> bool {
        self.password
    }

    pub fn is_revealed(&self) -> bool {
        self.revealed
    }

    /// The keyboard focus on the eye or the link, or off both (the box's
    /// focus is the inner field's).
    pub fn set_part_focus(&mut self, part: Option<FieldPart>) {
        self.part_focus = part.filter(|p| matches!(p, FieldPart::Reveal | FieldPart::Link));
    }

    pub fn part_focus(&self) -> Option<FieldPart> {
        self.part_focus
    }

    pub fn is_validated(&self) -> bool {
        self.validated
    }

    /// The eye is there: a password not marked validated.
    fn has_reveal(&self) -> bool {
        self.password && !self.validated
    }

    /// Reveal a hidden password or hide a revealed one. `false` when
    /// there is no eye (not a password, or validated) or disabled.
    pub fn toggle_reveal(&mut self) -> bool {
        if !self.has_reveal() || self.input.disabled {
            return false;
        }
        self.revealed = !self.revealed;
        self.input.set_masked(!self.revealed);
        true
    }

    /// What takes the ends of the box, as the kit pads them: `pl-10` with
    /// an icon; `pr-12` on a password, `pr-10` when validated.
    fn fit_slots(&mut self) {
        let start = if self.icon.is_some() {
            Slot::Icon
        } else {
            Slot::Pad
        };
        let end = if self.password {
            Slot::Reveal
        } else if self.validated {
            Slot::Check
        } else {
            Slot::Pad
        };
        self.input.set_slots(start, end);
    }

    /// A text wrapped in a box `avail` wide: the box it takes, at `(x,
    /// y)`, as wide as its widest line.
    fn text_rect(text: &str, style: &TextStyle, x: f32, y: f32, avail: f32) -> Rect {
        let (w, h) = TextMeasurer::measure_styled(text, style, Some(avail));
        Rect::new(x, y, w.ceil().min(avail), h)
    }

    /// `text` reads right to left (its first strong letter, as css
    /// `dir="auto"`): it hangs from the end of its row, not the start.
    fn reads_rtl(text: &str, style: &TextStyle) -> bool {
        TextMeasurer::clusters_styled(text, style, None)
            .first()
            .is_some_and(|c| c.paragraph_rtl)
    }

    /// `r` moved to the end of the row from `x`, `w` wide, when `rtl`.
    fn hang(r: Rect, x: f32, w: f32, rtl: bool) -> Rect {
        if rtl {
            Rect::new(x + w - r.w, r.y, r.w, r.h)
        } else {
            r
        }
    }

    /// The label row at `(x, y)`, `w` wide: the label and the link side
    /// by side when both fit on one line, the link under the label, at
    /// the end, when not; a label that reads right to left mirrors the
    /// row (the label at the right, the link at the left). Returns the
    /// two rects and the row's height.
    fn label_row(
        &self,
        x: f32,
        y: f32,
        w: f32,
        theme: &Theme,
    ) -> (Option<Rect>, Option<Rect>, f32) {
        let scale = &theme.typography;
        let f = &theme.form.field;
        // One line's width, rounded up as the rects are.
        let one_line = |text: &str, style: &TextStyle| {
            TextMeasurer::measure_styled(text, style, None).0.ceil()
        };
        let label = self.label.as_ref();
        let link = self.link.as_deref();
        let side_by_side = match (label, link) {
            (Some(label), Some(link)) => {
                label.size(None, theme).0 + one_line(link, &scale.run(f.link)) <= w
            }
            _ => true,
        };
        let link_w = match link {
            Some(link) if side_by_side => one_line(link, &scale.run(f.link)).min(w),
            _ => 0.0,
        };
        let rtl = label.is_some_and(|l| Self::reads_rtl(l.text(), &l.style(theme)));
        let label = label.map(|l| {
            let (lw, lh) = l.size(Some(w - link_w), theme);
            Self::hang(Rect::new(x, y, lw, lh), x, w, rtl)
        });
        let below = if side_by_side {
            y
        } else {
            label.map_or(y, |r| r.bottom())
        };
        let link = link.map(|t| {
            let r = Self::text_rect(t, &scale.run(f.link), x, below, w);
            Self::hang(r, x, w, !rtl)
        });
        let bottom = [label, link]
            .into_iter()
            .flatten()
            .fold(y, |b, r| b.max(r.bottom()));
        (label, link, bottom - y)
    }

    /// Where every part goes in `bounds` (its height is not read: the
    /// field takes [`height_for`](Self::height_for) of the width).
    pub fn layout(&self, bounds: Rect, theme: &Theme) -> FieldLayout {
        let form = &theme.form;
        let (x, w) = (bounds.x, bounds.w);
        let (label, link, row_h) = self.label_row(x, bounds.y, w, theme);
        let top = if row_h > 0.0 {
            bounds.y + row_h + form.field.label_gap
        } else {
            bounds.y
        };
        let input = Rect::new(x, top, w, TextField::height(theme));
        let a = &form.adorn;
        let centered = |slot_x: f32, slot_w: f32, size: f32| {
            Rect::new(
                slot_x + (slot_w - size) / 2.0,
                input.y + (input.h - size) / 2.0,
                size,
                size,
            )
        };
        let icon = self
            .icon
            .map(|_| centered(input.x, a.lead_slot, a.lead_icon));
        let check = self.validated.then(|| {
            Rect::new(
                input.right() - a.check_inset - a.check_icon,
                input.y + (input.h - a.check_icon) / 2.0,
                a.check_icon,
                a.check_icon,
            )
        });
        let reveal = self.has_reveal().then(|| {
            Rect::new(
                input.right() - a.reveal_slot,
                input.y,
                a.reveal_slot,
                input.h,
            )
        });
        let message = self.message.as_ref().map(|m| {
            let style = &theme.typography.run(form.field.message);
            let r = Self::text_rect(
                m.text(),
                style,
                x,
                input.bottom() + form.field.message_gap,
                w,
            );
            Self::hang(r, x, w, Self::reads_rtl(m.text(), style))
        });
        FieldLayout {
            label,
            link,
            input,
            icon,
            check,
            reveal,
            message,
        }
    }

    /// The field's height at width `w`.
    pub fn height_for(&self, w: f32, theme: &Theme) -> f32 {
        self.layout(Rect::new(0.0, 0.0, w, 0.0), theme).bottom()
    }

    /// The part of the field at `(x, y)`, laid out in `bounds`.
    pub fn part_at(&self, x: f32, y: f32, bounds: Rect, theme: &Theme) -> Option<FieldPart> {
        let l = self.layout(bounds, theme);
        if l.reveal.is_some_and(|r| r.contains(x, y)) {
            Some(FieldPart::Reveal)
        } else if l.link.is_some_and(|r| hit_rect(r, theme).contains(x, y)) {
            Some(FieldPart::Link)
        } else if l.label.is_some_and(|r| r.contains(x, y)) {
            Some(FieldPart::Label)
        } else if l.input.contains(x, y) {
            Some(FieldPart::Input)
        } else {
            None
        }
    }

    /// The pointer: hover on the box and on the link; a press on the eye
    /// reveals or hides the password (the box keeps its focus), a press
    /// on the box focuses it and places the caret, a press anywhere else
    /// blurs it. A press on the link is `clicked`; the owner asks
    /// [`part_at`](Self::part_at) what it was and runs it.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        theme: &Theme,
    ) -> EventResult {
        let l = self.layout(bounds, theme);
        if self.input.disabled {
            let was = std::mem::take(&mut self.link_hovered);
            let r = self.input.handle_event(event, l.input, theme);
            return if was {
                r.merge(EventResult::changed())
            } else {
                r
            };
        }
        match *event {
            WidgetEvent::MouseMove { x, y } => {
                let over = l.link.is_some_and(|r| hit_rect(r, theme).contains(x, y));
                let mut r = self.input.handle_event(event, l.input, theme);
                if over != self.link_hovered {
                    self.link_hovered = over;
                    r = r.merge(EventResult::changed());
                }
                r
            }
            WidgetEvent::MouseDown { x, y } => match self.part_at(x, y, bounds, theme) {
                Some(FieldPart::Reveal) => {
                    self.toggle_reveal();
                    EventResult::clicked()
                }
                Some(FieldPart::Link) => {
                    let r = self.input.handle_event(event, l.input, theme);
                    r.merge(EventResult::clicked())
                }
                Some(FieldPart::Label) => {
                    self.input.focus();
                    EventResult::clicked()
                }
                _ => self.input.handle_event(event, l.input, theme),
            },
            _ => self.input.handle_event(event, l.input, theme),
        }
    }

    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        self.render_to_layer(c, LayerId::DEFAULT, bounds, theme);
    }

    pub fn render_to_layer(&self, c: &mut Compositor, layer: LayerId, bounds: Rect, theme: &Theme) {
        let scale = &theme.typography;
        let l = self.layout(bounds, theme);
        let form = &theme.form;
        let f = &form.field;
        let alpha = if self.input.disabled {
            form.input.disabled_alpha
        } else {
            1.0
        };
        if let (Some(label), Some(r)) = (&self.label, l.label) {
            label.draw(c, layer, r.x, r.y, Some(r.w), theme);
        }
        let fade = |color: Color, a: f32| with_alpha(color, color.0[3] * a * alpha);
        let mut text = |s: &str, style: &TextStyle, r: Rect, color: [f32; 4]| {
            c.push_to_layer(
                layer,
                SceneNode::Text {
                    key: TextNodeKey::from_style(s, style, Some(r.w)),
                    x: r.x,
                    y: r.y,
                    color,
                },
            );
        };
        if let (Some(link), Some(r)) = (&self.link, l.link) {
            let a = if self.link_hovered {
                f.link_hover_alpha
            } else {
                f.link_alpha
            };
            text(link, &scale.run(f.link), r, fade(f.link_color, a));
        }
        if let (Some(message), Some(r)) = (&self.message, l.message) {
            let color = match message {
                Message::Error(_) => f.error_color,
                Message::Helper(_) => f.helper_color,
            };
            text(message.text(), &scale.run(f.message), r, fade(color, 1.0));
        }

        self.input.render_to_layer(c, layer, l.input, theme);

        let tint = fade(form.adorn.icon_color, 1.0);
        let mut icon = |name: &str, r: Rect| {
            if let Some(node) = icons::icon_at(name, r.w, tint, r.x, r.y) {
                c.push_to_layer(layer, node);
            }
        };
        if let (Some(name), Some(r)) = (self.icon, l.icon) {
            icon(name, r);
        }
        if let Some(r) = l.check {
            icon("circle-check", r);
        }
        if let Some(slot) = l.reveal {
            // The eye while hidden (reveal it), the struck eye while shown.
            let name = if self.revealed { "eye-off" } else { "eye" };
            let size = form.adorn.reveal_icon;
            let r = Rect::new(
                slot.x + (slot.w - size) / 2.0,
                slot.y + (slot.h - size) / 2.0,
                size,
                size,
            );
            icon(name, r);
            if let Some(id) = self.reveal_id() {
                c.bind_to_layer(layer, &id, [slot.x, slot.y, slot.w, slot.h]);
            }
        }
        if let (Some(id), Some(r)) = (self.link_id(), l.link) {
            // a line of text presses in the minimum target (`widgets::hit`)
            let r = hit_rect(r, theme);
            c.bind_to_layer(layer, &id, [r.x, r.y, r.w, r.h]);
        }
        // The keyboard focus on the eye (inside the box's end) or the link.
        match (self.part_focus, l.reveal, l.link) {
            (Some(FieldPart::Reveal), Some(r), _) => {
                let radius = form.input.radius.min(r.h / 2.0);
                c.push_to_layer(layer, focus_ring_inside(r, radius, theme));
            }
            (Some(FieldPart::Link), _, Some(r)) => {
                c.push_to_layer(layer, focus_ring(r, theme.shape.micro, theme));
            }
            _ => {}
        }
    }

    /// Declares the field as `id` (the id given to [`Field::sem`]) under
    /// `parent`: the text field as [`TextField::project`] declares it,
    /// named by the label (the placeholder without one) and described by
    /// the message, with `invalid` and `valid`, secret when a password;
    /// then, under the same parent, the reveal button (`<id>.reveal`, a
    /// toggle with its `on`, the password shown) and the link
    /// (`<id>.link`, `activate`). It only reads the widget: the owner runs
    /// `set_value`, `toggle` ([`toggle_reveal`](Self::toggle_reveal)) and
    /// the link.
    #[track_caller]
    pub fn project(&self, b: &mut SnapshotBuilder, id: &SemId, parent: Option<&SemId>) {
        let name = self.label.as_ref().map_or_else(
            || self.input.input.placeholder.clone(),
            |l| l.text().to_owned(),
        );
        let mut node = self.input.project(b, id, name, parent);
        node.state("invalid", matches!(self.message, Some(Message::Error(_))))
            .state("valid", self.validated);
        if let Some(message) = &self.message {
            node.description(message.text());
        }
        if self.password {
            node.constraints(Constraints {
                editable: true,
                secret: true,
                ..Constraints::default()
            });
        }
        let beside = parent.unwrap_or(id);
        let enabled = !self.input.disabled;
        if let Some(reveal) = self.reveal_id().filter(|_| self.has_reveal()) {
            let mut node = b.node(&reveal, Role::Toggle, "show password");
            node.parent(beside);
            node.state("on", self.revealed).enabled(enabled);
            node.action("toggle").reads(format!("{reveal}:on"));
        }
        if let (Some(link_id), Some(link)) = (self.link_id(), &self.link) {
            let mut node = b.node(&link_id, Role::Link, link.as_str());
            node.parent(beside);
            node.enabled(enabled);
            node.action("activate");
        }
    }
}
