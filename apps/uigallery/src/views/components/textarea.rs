//! Textarea page: the hoff core kit's Field with `textarea`
//! (`components/Field/index.tsx:55`), as the kit's showcase draws a
//! component (groups with a title and a rule, cards behind a chip naming
//! their states), and the one that grows with its text as the kit's
//! prompt and comment boxes do. The widget is
//! `engine::widgets::form::textarea::TextArea`.
//!
//! The page is a form page (`field::FormPage`): its ids under
//! `textarea_page`, an area projected as a multiline text field. Enter
//! breaks the line, Up and Down move a line, in the focused area.

#[cfg(test)]
use std::any::Any;

use accord::{SemId, SnapshotBuilder};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;
use engine::widgets::form::textarea::TextArea;
use engine::widgets::form::{EditKey, TextField};
use engine::widgets::prelude::{EventResult, Rect, WidgetEvent};

use super::field::spec::{CardWidget, FIELD_W, Part};
use super::field::{FormPage, Group, card};

/// The live card's chip, the Field's.
const LIVE: &str = "Default, Click on, Focus, Hover, Typing";

const NOTE: &str = "Bring the signed consent forms.\nThe lab opens at nine.";

fn message() -> TextArea {
    TextArea::new("Write a message")
}

/// The Textarea tab.
pub fn page() -> FormPage {
    let groups = vec![
        Group {
            title: "Textarea",
            columns: 3,
            cards: vec![
                card(LIVE, message()),
                card("Typed", message().with_text(NOTE)),
                card(
                    "Label, Message",
                    message()
                        .label("Notes")
                        .with_text("Too short.")
                        .error("Write at least twenty characters."),
                ),
            ],
        },
        Group {
            title: "Grows",
            columns: 2,
            cards: vec![
                card("Default, Typing", message().grows(5)),
                card(
                    "Five lines, then scrolls",
                    message().grows(5).with_text(
                        "One line.\nTwo lines.\nThree lines.\nFour lines.\nFive lines.\nSix lines.",
                    ),
                ),
            ],
        },
        Group {
            title: "Disabled",
            columns: 2,
            cards: vec![
                card("Disabled", message().disabled(true)),
                card("Disabled, Typed", message().with_text(NOTE).disabled(true)),
            ],
        },
        Group {
            title: "Content",
            columns: 3,
            cards: vec![
                card(
                    "Long text, Scrolled",
                    message().with_text(
                        "A summary of the visit: the participant arrived on time, the \
                         questionnaire took twenty minutes, and the second sample was \
                         taken after the break. Follow up next week about the results.",
                    ),
                ),
                card(
                    "Long label, Helper",
                    TextArea::new("Beschreiben Sie Ihr Anliegen")
                        .label("Beschreibung des Anliegens für die Forschungsgruppe")
                        .helper("Wir antworten innerhalb von zwei Arbeitstagen."),
                ),
                card(
                    "Right to left",
                    TextArea::new("כתבו הודעה").with_text("שלום עולם\nשורה שנייה"),
                ),
            ],
        },
    ];
    FormPage::new("textarea_page", groups)
}

impl CardWidget for TextArea {
    fn with_sem(self: Box<Self>, id: SemId) -> Box<dyn CardWidget> {
        Box::new((*self).sem(id))
    }

    /// The kit page's `w-60`, narrower when the card is.
    fn size(&self, room: f32, theme: &Theme) -> (f32, f32) {
        let w = FIELD_W.min(room);
        (w, self.height_for(w, theme))
    }

    fn render(&self, c: &mut Compositor, rect: Rect, theme: &Theme) {
        TextArea::render(self, c, rect, theme);
    }

    fn project(&self, b: &mut SnapshotBuilder, parent: &SemId) {
        if let Some(id) = self.sem_id() {
            TextArea::project(self, b, id, Some(parent));
        }
    }

    fn handle_event(&mut self, event: &WidgetEvent, rect: Rect, theme: &Theme) -> EventResult {
        TextArea::handle_event(self, event, rect, theme)
    }

    fn text(&self) -> Option<&TextField> {
        Some(&self.input)
    }

    fn text_mut(&mut self) -> Option<&mut TextField> {
        Some(&mut self.input)
    }

    /// Over the lines on screen.
    fn edit(&mut self, key: EditKey) -> bool {
        TextArea::edit(self, key)
    }

    /// A line break.
    fn enter(&mut self) -> bool {
        self.newline()
    }

    /// A line up or down.
    fn nav(&mut self, down: bool) -> bool {
        self.vertical(down, false)
    }

    fn part_id(&self, part: Part) -> Option<SemId> {
        (part == Part::Box)
            .then(|| self.sem_id().cloned())
            .flatten()
    }

    fn on_box(&self, x: f32, y: f32, rect: Rect, theme: &Theme) -> bool {
        self.layout(rect, theme).input.contains(x, y)
    }

    fn ime_area(&self, rect: Rect, theme: &Theme) -> Option<Rect> {
        Some(TextArea::ime_area(self, rect, theme))
    }

    #[cfg(test)]
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use accord::{Role, Value};
    use engine::widgets::prelude::{Rect, WidgetEvent};

    use super::*;
    use crate::views::components::Component;
    use crate::views::text_input::host::TextHost;

    fn content() -> Rect {
        Rect::new(288.0, 80.0, 900.0, 2400.0)
    }

    fn area(p: &FormPage, at: (usize, usize)) -> &TextArea {
        p.spec(at).and_then(|s| s.get()).expect("a text area")
    }

    /// From 280 to 1600 px every 8, in the core and the HOFF themes:
    /// every area inside its card and under its chip, its text box in its
    /// box and its message under it.
    #[test]
    fn every_area_stays_in_its_card_from_280_to_1600() {
        let p = page();
        for theme in [Theme::hoff_core(), Theme::hoff()] {
            for w in (280..=1600).step_by(8).map(|w| w as f32) {
                let content = Rect::new(0.0, 0.0, w, 800.0);
                for (laid, group) in p.layout(content, &theme).iter().zip(&p.groups) {
                    for ((card, rect), c) in laid.cards.iter().zip(&group.cards) {
                        let what = format!("{w}: {} / {}", group.title, c.title);
                        assert!(card.right() <= w + 0.5, "{what}: card");
                        assert!(rect.x >= card.x && rect.right() <= card.right() + 0.5);
                        assert!(rect.bottom() <= card.bottom() + 0.5, "{what}: under");
                        let a: &TextArea = c.spec.get().unwrap();
                        let l = a.layout(*rect, &theme);
                        assert!(l.text.x >= l.input.x && l.text.right() <= l.input.right());
                        assert!(l.text.bottom() <= l.input.bottom() + 0.01, "{what}");
                        if let Some(m) = l.message {
                            assert!(m.y >= l.input.bottom() && m.right() <= rect.right() + 0.5);
                        }
                    }
                }
            }
        }
    }

    /// A click focuses an area; the keys type, Enter breaks the line, Up
    /// goes back a line, Escape blurs.
    #[test]
    fn enter_breaks_the_line_and_up_goes_back_one() {
        let theme = Theme::hoff_core();
        let mut p = page();
        let rect = p.widget_rect((0, 0), content(), &theme).unwrap();
        let (x, y) = rect.center();
        p.handle_event(&WidgetEvent::MouseDown { x, y }, content(), &theme);
        assert_eq!(p.focused(), Some((0, 0)));
        assert!(p.handle_text("first"));
        assert!(p.handle_enter());
        assert!(p.handle_text("second"));
        assert_eq!(area(&p, (0, 0)).input.text(), "first\nsecond");
        let mut c = Compositor::new();
        c.begin_frame();
        p.render(
            &mut c,
            engine::drawing::compositor::LayerId::DEFAULT,
            content(),
            &theme,
        );
        assert!(p.handle_nav(false));
        assert!(area(&p, (0, 0)).input.input.buffer.cursor() <= "first".len());
        assert!(p.ime_area(content(), &theme).is_some());
        assert!(p.handle_escape());
        assert!(!p.handle_enter(), "nothing focused: the walk's");
    }

    /// An area projects as a multiline text field; `set_value` puts a
    /// text with its lines.
    #[test]
    fn the_area_projects_multiline_and_takes_a_value_with_lines() {
        let mut p = page();
        let root = p.root().expect("the root parses").clone();
        let mut b = SnapshotBuilder::new();
        b.node(&root, Role::Region, "Textarea");
        p.project(&mut b, &root);
        let snap = b.finish().expect("the page projects");
        let id = SemId::parse("textarea_page.group[0].card[1]").unwrap();
        let node = snap.get(&id).unwrap();
        assert_eq!(node.role, Role::TextInput);
        assert_eq!(node.state.get("multiline"), Some(&Value::Bool(true)));
        let req = accord::Request::observe(
            &snap,
            accord::Actor::Test { name: "t".into() },
            &id,
            "set_value",
            Some(Value::from("a\nb")),
        )
        .unwrap();
        assert!(p.run(&req).is_ok());
        assert_eq!(area(&p, (0, 1)).input.text(), "a\nb");
    }
}
