//! Input OTP page: a one-time code, a cell per character, as the kit's
//! showcase draws a component (groups with a title and a rule, cards
//! behind a chip naming their states). The kit has no code input: the
//! widget, `engine::widgets::form::otp::InputOtp`, derives every value
//! from the kit's Field (its box made square, its label and message
//! line), and the chips name the Field's states.
//!
//! The page is a form page (`field::FormPage`): its ids under
//! `input_otp_page`, a code projected as a secret text field at most its
//! cells long, `set_value` refused for what is not the code's.

#[cfg(test)]
use std::any::Any;

use accord::{SemId, SnapshotBuilder};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;
use engine::widgets::form::TextField;
use engine::widgets::form::otp::InputOtp;
use engine::widgets::prelude::{EventResult, Rect, WidgetEvent};

use super::field::spec::{CardWidget, Part};
use super::field::{FormPage, Group, card};

/// The live card's chip, the Field's.
const LIVE: &str = "Default, Click on, Focus, Hover, Typing";

fn six() -> InputOtp {
    InputOtp::new(&[3, 3])
}

/// The Input OTP tab.
pub fn page() -> FormPage {
    let groups = vec![
        Group {
            title: "Six digits",
            columns: 2,
            cards: vec![
                card(LIVE, six()),
                card("Typed", six().with_text("482")),
                card("Filled", six().with_text("482913")),
                card("Masked", six().with_text("482913").masked(true)),
            ],
        },
        Group {
            title: "With label",
            columns: 2,
            cards: vec![
                card(
                    "Label, Helper",
                    six()
                        .label("Verification code")
                        .helper("Enter the code we sent to your phone."),
                ),
                card(
                    "Message",
                    six()
                        .label("Verification code")
                        .with_text("000000")
                        .error("The code is incorrect."),
                ),
            ],
        },
        Group {
            title: "Disabled",
            columns: 2,
            cards: vec![
                card("Disabled", six().disabled(true)),
                card("Disabled, Typed", six().with_text("4829").disabled(true)),
            ],
        },
        Group {
            title: "Content",
            columns: 2,
            cards: vec![
                card("Four digits", InputOtp::new(&[4])),
                card(
                    "Letters and digits",
                    InputOtp::new(&[4, 4]).alphanumeric().with_text("K7Q2"),
                ),
            ],
        },
    ];
    FormPage::new("input_otp_page", groups)
}

impl CardWidget for InputOtp {
    fn with_sem(self: Box<Self>, id: SemId) -> Box<dyn CardWidget> {
        Box::new((*self).sem(id))
    }

    /// As wide as its cells take in the room, a group under the other
    /// where one row does not fit.
    fn size(&self, room: f32, theme: &Theme) -> (f32, f32) {
        let w = self.width_for(room, theme).min(room);
        (w, self.height_for(w, theme))
    }

    fn render(&self, c: &mut Compositor, rect: Rect, theme: &Theme) {
        InputOtp::render(self, c, rect, theme);
    }

    fn project(&self, b: &mut SnapshotBuilder, parent: &SemId) {
        if let Some(id) = self.sem_id() {
            InputOtp::project(self, b, id, "Code", Some(parent));
        }
    }

    fn handle_event(&mut self, event: &WidgetEvent, rect: Rect, theme: &Theme) -> EventResult {
        InputOtp::handle_event(self, event, rect, theme)
    }

    fn text(&self) -> Option<&TextField> {
        Some(&self.input)
    }

    fn text_mut(&mut self) -> Option<&mut TextField> {
        Some(&mut self.input)
    }

    fn part_id(&self, part: Part) -> Option<SemId> {
        (part == Part::Box)
            .then(|| self.sem_id().cloned())
            .flatten()
    }

    fn on_box(&self, x: f32, y: f32, rect: Rect, theme: &Theme) -> bool {
        self.on_row(x, y, rect, theme)
    }

    fn ime_area(&self, rect: Rect, theme: &Theme) -> Option<Rect> {
        Some(InputOtp::ime_area(self, rect, theme))
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

    fn otp(p: &FormPage, at: (usize, usize)) -> &InputOtp {
        p.spec(at).and_then(|s| s.get()).expect("a code")
    }

    /// From 280 to 1600 px every 8, in the core and the HOFF themes:
    /// every code inside its card and under its chip, its cells inside
    /// the widget's rect, apart and square, its digit on one line.
    #[test]
    fn every_code_stays_in_its_card_from_280_to_1600() {
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
                        let code: &InputOtp = c.spec.get().unwrap();
                        let l = code.layout(*rect, &theme);
                        for cell in &l.cells {
                            assert!(
                                cell.x >= rect.x - 0.01 && cell.right() <= rect.right() + 0.5,
                                "{what}: a cell out of the code"
                            );
                            assert!(cell.bottom() <= card.bottom() + 0.5, "{what}: cell");
                            assert!(cell.h >= theme.form.input.text.line_height);
                        }
                        for pair in l.cells.windows(2) {
                            let apart = pair[0].right() <= pair[1].x + 0.01
                                || pair[0].bottom() <= pair[1].y + 0.01;
                            assert!(apart, "{what}: two cells meet");
                        }
                    }
                }
            }
        }
    }

    /// A click on a cell focuses the code and the keys type only digits,
    /// up to its six cells; Escape blurs.
    #[test]
    fn a_click_focuses_the_code_and_the_keys_type_its_digits() {
        let theme = Theme::hoff_core();
        let mut p = page();
        let rect = p.widget_rect((0, 0), content(), &theme).unwrap();
        let cell = otp(&p, (0, 0)).layout(rect, &theme).cells[0];
        let (x, y) = cell.center();
        p.handle_event(&WidgetEvent::MouseDown { x, y }, content(), &theme);
        assert_eq!(p.focused(), Some((0, 0)));
        assert!(p.handle_text("12ab3456789"));
        assert_eq!(otp(&p, (0, 0)).input.text(), "123456");
        assert!(p.ime_area(content(), &theme).is_some());
        assert!(p.handle_escape());
        assert_eq!(p.focused(), None);
    }

    /// A code projects as a secret text field, its length only; a
    /// request's `set_value` puts a code and refuses what is not one.
    #[test]
    fn the_code_projects_secret_and_refuses_what_is_not_a_code() {
        let mut p = page();
        let root = p.root().expect("the root parses").clone();
        let snap = |p: &FormPage| {
            let mut b = SnapshotBuilder::new();
            b.node(&root, Role::Region, "Input OTP");
            p.project(&mut b, &root);
            b.finish().expect("the page projects")
        };
        let id = SemId::parse("input_otp_page.group[0].card[2]").unwrap();
        let s = snap(&p);
        let node = s.get(&id).unwrap();
        assert_eq!(node.role, Role::TextInput);
        assert!(node.constraints.secret);
        assert_eq!(node.constraints.max_len, Some(6));
        assert_eq!(node.state.get("text"), Some(&Value::Int(6)));
        let ask = |value: &str| {
            accord::Request::observe(
                &s,
                accord::Actor::Test { name: "t".into() },
                &id,
                "set_value",
                Some(Value::from(value)),
            )
            .unwrap()
        };
        assert!(p.run(&ask("120934")).is_ok());
        assert_eq!(otp(&p, (0, 2)).input.text(), "120934");
        assert!(p.run(&ask("12a934")).is_err(), "a letter");
        assert_eq!(otp(&p, (0, 2)).input.text(), "120934", "as it was");
    }
}
