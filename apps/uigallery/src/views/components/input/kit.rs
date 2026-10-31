//! The kit's InputsPage (`templates/InputsPage/index.tsx`) on the Input
//! tab: its groups in its order, each card behind the chip the kit gives
//! it, as a form page (`field::FormPage`) whose ids are under
//! `input_page`. The widgets the Field and Input Group tabs already show
//! are the same widgets here; the avatar upload and the color picker plug
//! in below.
//!
//! The gallery has no files to choose from: an avatar's ask for one is
//! answered at once with the theme's stand-in for a photo.

#[cfg(test)]
use std::any::Any;

use accord::{SemId, SnapshotBuilder, Value};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;
use engine::widgets::form::TextField;
use engine::widgets::form::color_picker::{ColorPart, ColorPicker};
use engine::widgets::form::upload_avatar::UploadAvatar;
use engine::widgets::prelude::{EventResult, Rect, WidgetEvent};

use super::super::field::spec::{CardWidget, Part};
use super::super::field::{FormPage, Group, card};

/// The page root the kit's groups share with the Input group.
pub(super) const ROOT: &str = "input_page";

/// The kit's groups.
pub(super) fn page() -> FormPage {
    let groups = vec![
        Group {
            title: "Replace avatar",
            columns: 2,
            cards: vec![
                card("Default, Hover", UploadAvatar::new().with_photo(None)),
                card("No avatar", UploadAvatar::new()),
            ],
        },
        Group {
            title: "Color picker",
            columns: 1,
            // :167-171, the kit's own color, `F4F4F4` at 100 %.
            cards: vec![card(
                "Default, Hover, Focus",
                ColorPicker::new("F4F4F4", 100),
            )],
        },
    ];
    FormPage::new(ROOT, groups)
}

impl CardWidget for ColorPicker {
    fn with_sem(self: Box<Self>, id: SemId) -> Box<dyn CardWidget> {
        Box::new((*self).sem(id))
    }

    /// The kit's `w-54.5`, never more than the room.
    fn size(&self, room: f32, theme: &Theme) -> (f32, f32) {
        ColorPicker::size(self, room, theme)
    }

    fn render(&self, c: &mut Compositor, rect: Rect, theme: &Theme) {
        ColorPicker::render(self, c, rect, theme);
    }

    fn project(&self, b: &mut SnapshotBuilder, parent: &SemId) {
        if let Some(id) = self.sem_id() {
            ColorPicker::project(self, b, id, "Color", Some(parent));
        }
    }

    fn handle_event(&mut self, event: &WidgetEvent, rect: Rect, theme: &Theme) -> EventResult {
        ColorPicker::handle_event(self, event, rect, theme)
    }

    /// The text with the caret, else the hex.
    fn text(&self) -> Option<&TextField> {
        Some(ColorPicker::text(self))
    }

    fn text_mut(&mut self) -> Option<&mut TextField> {
        Some(ColorPicker::text_mut(self))
    }

    /// The page's caret comes in at the hex; its loss keeps what is a
    /// color.
    fn set_focused(&mut self, focused: bool) {
        if focused {
            self.focus(ColorPart::Hex);
        } else {
            self.blur();
        }
    }

    /// Enter keeps what was typed, the caret staying.
    fn enter(&mut self) -> bool {
        ColorPicker::enter(self)
    }

    /// Escape puts back the color the caret came in to.
    fn escape(&mut self) -> bool {
        ColorPicker::escape(self)
    }

    /// The hex is the box, the opacity the widget's second one.
    fn part(&self, id: &SemId) -> Option<Part> {
        [Part::Box, Part::Alpha]
            .into_iter()
            .find(|p| CardWidget::part_id(self, *p).as_ref() == Some(id))
    }

    fn part_id(&self, part: Part) -> Option<SemId> {
        match part {
            Part::Box => self.text_id(ColorPart::Hex).cloned(),
            Part::Alpha => self.text_id(ColorPart::Alpha).cloned(),
            _ => None,
        }
    }

    /// Tab or a reader to the opacity puts the caret there; the page's
    /// blur takes it off.
    fn set_part_focus(&mut self, part: Option<Part>) {
        if part == Some(Part::Alpha) {
            self.focus(ColorPart::Alpha);
        }
    }

    fn part_focus(&self) -> Option<Part> {
        (self.focused() == Some(ColorPart::Alpha)).then_some(Part::Alpha)
    }

    fn on_box(&self, x: f32, y: f32, rect: Rect, theme: &Theme) -> bool {
        let l = self.layout(rect, theme);
        l.hex.contains(x, y) || l.alpha.contains(x, y)
    }

    fn ime_area(&self, rect: Rect, theme: &Theme) -> Option<Rect> {
        ColorPicker::ime_area(self, rect, theme)
    }

    /// `set_value` on the hex sets the color, on the opacity its percent,
    /// refused where it is not one.
    fn run(&mut self, part: Part, action: &str, param: Option<&Value>) -> bool {
        match (part, action, param) {
            (Part::Box, "set_value", Some(Value::Text(text))) => self.set_hex(text),
            (Part::Alpha, "set_value", Some(Value::Text(text))) => self.set_opacity(text),
            _ => false,
        }
    }

    #[cfg(test)]
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl CardWidget for UploadAvatar {
    fn with_sem(self: Box<Self>, id: SemId) -> Box<dyn CardWidget> {
        Box::new((*self).sem(id))
    }

    /// The kit's `size-12`.
    fn size(&self, room: f32, theme: &Theme) -> (f32, f32) {
        UploadAvatar::size(self, room, theme)
    }

    fn render(&self, c: &mut Compositor, rect: Rect, theme: &Theme) {
        UploadAvatar::render(self, c, rect, theme);
    }

    fn project(&self, b: &mut SnapshotBuilder, parent: &SemId) {
        if let Some(id) = self.sem_id() {
            UploadAvatar::project(self, b, id, Some(parent));
        }
    }

    /// A press asks for a file; the gallery answers at once.
    fn handle_event(&mut self, event: &WidgetEvent, rect: Rect, theme: &Theme) -> EventResult {
        let r = UploadAvatar::handle_event(self, event, rect, theme);
        if self.take_ask() {
            self.chosen(None);
        }
        r
    }

    /// The disc is one button: the page's focus ring and its `activate`.
    fn part_id(&self, part: Part) -> Option<SemId> {
        match part {
            Part::Action => self.sem_id().cloned(),
            _ => None,
        }
    }

    fn set_part_focus(&mut self, part: Option<Part>) {
        self.set_focused(part == Some(Part::Action));
    }

    fn part_focus(&self) -> Option<Part> {
        self.is_focused().then_some(Part::Action)
    }

    /// `activate` asks for a file, as a press does; the gallery answers.
    fn run(&mut self, part: Part, action: &str, param: Option<&Value>) -> bool {
        match (part, action, param) {
            (Part::Action, "activate", None) if self.activate() => {
                self.take_ask();
                self.chosen(None);
                true
            }
            _ => false,
        }
    }

    #[cfg(test)]
    fn as_any(&self) -> &dyn Any {
        self
    }
}
