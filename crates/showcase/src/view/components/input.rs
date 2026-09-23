//! Input page: three single-line [`comps::form::TextField`]s plus the
//! live value preview line. The page owns the keyboard focus: a click in
//! a field places the caret, Tab cycles the fields, Escape blurs; the
//! characters and editing keys the shell forwards go to the focused one.

use comps::form::TextField;
use comps::prelude::{EventResult, Rect, Wake, WidgetEvent};
use engine::compositor::Compositor;
use engine::theme::Theme;

use super::{EditKey, text};

/// Number of fields on the page.
pub const COUNT: usize = 3;

pub struct InputPage {
    fields: [TextField; COUNT],
}

impl InputPage {
    pub fn new() -> Self {
        let fields = ["Full name", "Email address", "Project codename"].map(TextField::new);
        Self { fields }
    }

    /// Field rects stacked under the content's top-left, `field_max_w`
    /// wide at most, one field height each with the `md` spacing between.
    pub fn rects(content: Rect, theme: &Theme) -> [Rect; COUNT] {
        let h = TextField::height(theme);
        let gap = theme.spacing.md;
        let w = content.w.min(theme.size.readable_max_w / 2.0);
        std::array::from_fn(|i| Rect::new(content.x, content.y + i as f32 * (h + gap), w, h))
    }

    /// Bottom edge of the page (fields plus the preview line).
    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        let rects = Self::rects(content, theme);
        let last = rects[COUNT - 1];
        last.y + last.h + theme.spacing.sm + theme.typography.caption_sm().line_height - content.y
    }

    pub fn focused(&self) -> Option<usize> {
        self.fields.iter().position(|f| f.is_focused())
    }

    /// Buffer contents, exposed for the tests.
    #[cfg(test)]
    pub fn value(&self, i: usize) -> &str {
        self.fields[i].text()
    }

    /// Caret position (byte index), exposed for the tests.
    #[cfg(test)]
    pub fn cursor(&self, i: usize) -> usize {
        self.fields[i].input.buffer.cursor()
    }

    /// Focus exactly `target` (or nothing), unfocusing the rest.
    pub fn set_focused(&mut self, target: Option<usize>) {
        for (i, field) in self.fields.iter_mut().enumerate() {
            field.set_focused(Some(i) == target);
        }
    }

    /// A press inside a field focuses it and places the caret; a press
    /// anywhere else blurs (and is not swallowed).
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
    ) -> EventResult {
        let rects = Self::rects(content, theme);
        if let WidgetEvent::MouseDown { x, y } = *event {
            if let Some(i) = rects.iter().position(|f| f.contains(x, y)) {
                self.set_focused(Some(i));
                self.fields[i].handle_event(event, rects[i], theme);
                return EventResult::changed();
            }
            if self.focused().is_some() {
                self.set_focused(None);
                return EventResult::changed();
            }
        }
        let mut r = EventResult::IGNORED;
        for (field, rect) in self.fields.iter_mut().zip(rects) {
            r = r.merge(field.handle_event(event, rect, theme));
        }
        r
    }

    /// Type `s` into the focused field. `false` when nothing is focused.
    pub fn handle_text(&mut self, s: &str) -> bool {
        match self.focused() {
            Some(i) => self.fields[i].insert(s),
            None => false,
        }
    }

    /// Tab cycles the fields; the other editing keys edit the focused one.
    pub fn handle_edit_key(&mut self, key: EditKey) -> bool {
        if key == EditKey::Tab {
            let next = self.focused().map_or(0, |i| (i + 1) % COUNT);
            self.set_focused(Some(next));
            return true;
        }
        match self.focused() {
            Some(i) => self.fields[i].edit(key.into()),
            None => false,
        }
    }

    /// Escape blurs the focused field. `false` when none was.
    pub fn handle_escape(&mut self) -> bool {
        if self.focused().is_some() {
            self.set_focused(None);
            true
        } else {
            false
        }
    }

    /// Advance the cursor blink of the focused field: a frame on a flip,
    /// the next flip as the deadline; nothing while no field is focused.
    pub fn wake(&mut self, dt: f32) -> Wake {
        match self.focused() {
            Some(i) => self.fields[i].wake(dt),
            None => Wake::idle(),
        }
    }

    /// The live preview line under the fields.
    pub fn preview(&self) -> String {
        let filled: Vec<&str> = self
            .fields
            .iter()
            .map(|f| f.text())
            .filter(|t| !t.is_empty())
            .collect();
        if filled.is_empty() {
            "live value: (empty)".to_string()
        } else {
            format!("live value: {}", filled.join(" / "))
        }
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let rects = Self::rects(content, theme);
        for (field, rect) in self.fields.iter().zip(rects) {
            field.render(c, rect, theme);
        }
        let last = rects[COUNT - 1];
        let style = theme.typography.caption_sm();
        text(
            c,
            &self.preview(),
            style.font_size,
            style.font_weight,
            last.x,
            last.y + last.h + theme.spacing.sm,
            theme.colors.text_dim.0,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine::compositor::SceneNode;
    use engine::text::TextMeasurer;

    fn content() -> Rect {
        Rect::new(288.0, 80.0, 760.0, 700.0)
    }

    fn texts(page: &InputPage) -> Vec<String> {
        let theme = Theme::hoff();
        let mut c = Compositor::new();
        c.begin_frame();
        page.render(&mut c, content(), &theme);
        c.layer(engine::compositor::LayerId::DEFAULT)
            .unwrap()
            .nodes()
            .iter()
            .filter_map(|n| match n {
                SceneNode::Text { key, .. } => Some(key.text.clone()),
                _ => None,
            })
            .collect()
    }

    /// Tab walks the fields and wraps, Escape blurs.
    #[test]
    fn tab_cycles_the_fields_and_wraps() {
        let mut p = InputPage::new();
        assert_eq!(p.focused(), None);
        assert!(p.handle_edit_key(EditKey::Tab));
        assert_eq!(p.focused(), Some(0));
        p.handle_edit_key(EditKey::Tab);
        p.handle_edit_key(EditKey::Tab);
        assert_eq!(p.focused(), Some(2));
        p.handle_edit_key(EditKey::Tab);
        assert_eq!(p.focused(), Some(0), "Tab wraps to the first field");
        assert!(p.handle_escape());
        assert_eq!(p.focused(), None);
        assert!(!p.handle_escape());
    }

    /// A click inside a field focuses it and places the caret on the
    /// clicked glyph (real shaping via TextMeasurer, not chars * factor).
    #[test]
    fn click_focuses_field_and_places_caret() {
        let theme = Theme::hoff();
        let mut p = InputPage::new();
        let field = InputPage::rects(content(), &theme)[1];

        let (cx, cy) = field.center();
        let r = p.handle_event(&WidgetEvent::MouseDown { x: cx, y: cy }, content(), &theme);
        assert!(r.changed);
        assert_eq!(p.focused(), Some(1));

        assert!(p.handle_text("Brenner"));
        assert_eq!(p.value(1), "Brenner");
        let style = TextField::text_style(&theme);
        let x =
            field.x + theme.spacing.md + TextMeasurer::cursor_x_styled("Brenner", &style, None, 4);
        p.handle_event(&WidgetEvent::MouseDown { x, y: cy }, content(), &theme);
        assert_eq!(p.cursor(1), 4);

        // A click outside every field blurs but is not swallowed.
        let r = p.handle_event(
            &WidgetEvent::MouseDown {
                x: content().x + 1.0,
                y: content().y + content().h - 1.0,
            },
            content(),
            &theme,
        );
        assert_eq!(p.focused(), None);
        assert!(r.changed);
    }

    /// Characters only reach a field while one is focused, and editing
    /// keys edit the focused buffer.
    #[test]
    fn typing_requires_focus_and_editing_keys_edit() {
        let mut p = InputPage::new();
        assert!(!p.handle_text("x"), "no focus: the shell keeps the key");
        p.handle_edit_key(EditKey::Tab);
        p.handle_text("abc");
        p.handle_edit_key(EditKey::Left);
        p.handle_edit_key(EditKey::Backspace);
        assert_eq!(p.value(0), "ac");
        p.handle_edit_key(EditKey::End);
        p.handle_edit_key(EditKey::Delete);
        assert_eq!(p.value(0), "ac");
        p.handle_escape();
        assert!(
            !p.handle_edit_key(EditKey::Backspace),
            "blurred: not consumed"
        );
    }

    /// The scene mirrors the typed value live (field text + preview).
    #[test]
    fn scene_shows_typed_value_and_live_preview() {
        let mut p = InputPage::new();
        let before = texts(&p);
        assert!(
            before.iter().any(|t| t == "Full name"),
            "placeholder while empty"
        );
        assert!(before.iter().any(|t| t == "live value: (empty)"));
        p.handle_edit_key(EditKey::Tab);
        p.handle_text("Ada");
        let after = texts(&p);
        assert!(after.iter().any(|t| t == "Ada"), "typed value rendered");
        assert!(
            after.iter().any(|t| t == "live value: Ada"),
            "preview mirrors the buffer"
        );
    }

    /// The accent focus ring sits 4px (offset + stroke) outside the
    /// focused field, and nowhere when nothing is focused.
    #[test]
    fn focus_ring_tracks_the_focused_field() {
        let theme = Theme::hoff();
        let mut p = InputPage::new();
        let rings = |p: &InputPage| -> Vec<(f32, f32)> {
            let accent = theme.colors.accent.0;
            let mut c = Compositor::new();
            c.begin_frame();
            p.render(&mut c, content(), &theme);
            c.layer(engine::compositor::LayerId::DEFAULT)
                .unwrap()
                .nodes()
                .iter()
                .filter_map(|n| match *n {
                    SceneNode::RoundedRect {
                        x,
                        y,
                        color,
                        border_width,
                        border_color,
                        ..
                    } if color[3] == 0.0 && border_width == 2.0 && border_color == accent => {
                        Some((x, y))
                    }
                    _ => None,
                })
                .collect()
        };
        assert!(rings(&p).is_empty(), "no focus, no ring");
        p.handle_edit_key(EditKey::Tab);
        let f = InputPage::rects(content(), &theme)[0];
        assert_eq!(rings(&p), vec![(f.x - 4.0, f.y - 4.0)]);
    }

    /// While a field is focused the page asks for the caret's next flip,
    /// not for a frame every tick; idle it asks for nothing.
    #[test]
    fn focused_field_asks_for_the_next_flip() {
        let mut p = InputPage::new();
        assert_eq!(p.wake(0.016), Wake::idle(), "idle page requests no frames");
        p.handle_edit_key(EditKey::Tab);
        let w = p.wake(0.016);
        assert!(!w.now && !w.is_continuous(), "no flip yet, no frame: {w:?}");
        assert!(w.next.is_some(), "the next flip is pending");
    }
}
