//! Label page: the kit's form label (`components/Field/index.tsx:36`),
//! alone and over a field, as the hoff core kit's showcase draws a
//! component: groups with a title and a rule, cards behind a chip naming
//! their states. The widget is `engine::widgets::form::label::Label`; the
//! caption kind is the one the gallery's own group headings are drawn
//! with (`compositor/draw.rs`, `group_label`).
//!
//! A press on a field's label focuses the field, as an HTML label does.
//! The page is a form page (`field::FormPage`): its ids under
//! `label_page`, a label projected as a `Role::Label`, a field as the
//! Field page projects it.

use engine::widgets::form::field::Field;
use engine::widgets::form::label::Label;

use super::field::{FormPage, Group, card};

/// The Label tab.
pub fn page() -> FormPage {
    let email = || Field::new("Enter email").label("Email");
    let groups = vec![
        Group {
            title: "Label",
            columns: 3,
            cards: vec![
                card("Default", Label::new("Email address")),
                card("Disabled", Label::new("Email address").disabled(true)),
                card("Caption", Label::new("ACCOUNT").caption()),
            ],
        },
        Group {
            title: "With a field",
            columns: 2,
            cards: vec![
                card("Default, Click on the label", email()),
                card("Disabled", email().disabled(true)),
            ],
        },
        Group {
            title: "Content",
            columns: 2,
            cards: vec![
                card(
                    "Long label, Wrapped",
                    Label::new(
                        "Geschäftliche E-Mail-Adresse für Rechnungen, Belege und \
                         Hinweise zu Ihrem Arbeitsbereich",
                    ),
                ),
                card("Right to left", Label::new("כתובת דוא״ל")),
            ],
        },
    ];
    FormPage::new("label_page", groups)
}

#[cfg(test)]
mod tests {
    use accord::{Role, SemId, SnapshotBuilder};
    use engine::styling::theme::Theme;
    use engine::widgets::prelude::{Rect, WidgetEvent};

    use super::*;
    use crate::views::components::Component;

    /// From 280 to 1600 px every 8, in the core and the HOFF themes:
    /// every label and field inside its card and under its chip.
    #[test]
    fn every_label_stays_in_its_card_from_280_to_1600() {
        let p = page();
        for theme in [Theme::hoff_core(), Theme::hoff()] {
            let chip_band = theme.spacing.sm * 2.0;
            for w in (280..=1600).step_by(8).map(|w| w as f32) {
                let content = Rect::new(0.0, 0.0, w, 800.0);
                for laid in p.layout(content, &theme) {
                    for (card, widget) in &laid.cards {
                        assert!(card.right() <= w + 0.5, "{w}: card");
                        assert!(widget.x >= card.x && widget.right() <= card.right() + 0.5);
                        assert!(widget.y >= card.y + chip_band, "{w}: under the chip");
                        assert!(widget.bottom() <= card.bottom() + 0.5, "{w}: in the card");
                    }
                }
            }
        }
    }

    /// A press on a field's label gives the field the caret.
    #[test]
    fn a_press_on_the_label_focuses_its_field() {
        let theme = Theme::hoff_core();
        let mut p = page();
        let content = Rect::new(288.0, 80.0, 900.0, 2400.0);
        let rect = p.widget_rect((1, 0), content, &theme).expect("the card");
        let field = p.field((1, 0)).expect("a field");
        let (x, y) = field.layout(rect, &theme).label.expect("a label").center();
        p.handle_event(&WidgetEvent::MouseDown { x, y }, content, &theme);
        assert_eq!(p.focused(), Some((1, 0)));
        let disabled = p.widget_rect((1, 1), content, &theme).unwrap();
        let field = p.field((1, 1)).unwrap();
        let (x, y) = field.layout(disabled, &theme).label.unwrap().center();
        p.handle_event(&WidgetEvent::MouseDown { x, y }, content, &theme);
        assert_eq!(p.focused(), None, "a disabled field takes no caret");
    }

    /// Each label projects as a label, the disabled one disabled; the
    /// field by its label.
    #[test]
    fn the_labels_project_as_labels() {
        let p = page();
        let root = p.root().expect("the root parses").clone();
        let mut b = SnapshotBuilder::new();
        b.node(&root, Role::Region, "Label");
        p.project(&mut b, &root);
        let snap = b.finish().expect("the page projects");
        let node = |id: &str| snap.get(&SemId::parse(id).unwrap()).cloned().unwrap();
        let first = node("label_page.group[0].card[0]");
        assert_eq!(
            (first.role, first.label.as_str()),
            (Role::Label, "Email address")
        );
        assert!(!node("label_page.group[0].card[1]").enabled);
        let field = node("label_page.group[1].card[0]");
        assert_eq!(
            (field.role, field.label.as_str()),
            (Role::TextInput, "Email")
        );
    }
}
