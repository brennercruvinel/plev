//! Input Group page: the hoff core kit's Search (`components/Search/
//! index.tsx`), a text field with an icon button at its start and a
//! shortcut hint at its end, and the addons derived from it (a prefix, a
//! suffix), as the kit's showcase draws a component: groups with a title
//! and a rule, cards behind a chip naming their states. The widget is
//! `engine::widgets::form::input_group::InputGroup`.
//!
//! The kit's Invite (`components/Invite/index.tsx`) is a group too: the
//! inset box with a select at its end, whose list floats over the page.
//!
//! The page is a form page (`field::FormPage`): its ids under
//! `input_group_page`, a group projected as its text field (named by its
//! placeholder, described by its hint) with its icon button
//! (`….action`) and its select (`….choice`) beside it.

use engine::widgets::form::Select;
use engine::widgets::form::input_group::{Addon, InputGroup};
use engine::widgets::form::tokens::Look;

use super::field::{FormPage, Group, card};

/// The kit's Invite: who, in the inset box, and what they can do, in the
/// select at its end.
fn invite() -> InputGroup {
    InputGroup::new("Email, name...")
        .look(Look::Inset)
        .end_select(Select::new(["can view", "can edit"], 0))
}

fn search() -> InputGroup {
    InputGroup::new("Search...")
        .start(Addon::Icon("search"))
        .end(Addon::Hint("⌘ K".into()))
}

/// The Input Group tab.
pub fn page() -> FormPage {
    let groups = vec![
        Group {
            title: "Search",
            columns: 1,
            cards: vec![card("Default, Hover, Focus", search())],
        },
        Group {
            title: "Invite",
            columns: 1,
            cards: vec![card("Default, Hover, Focus, Dropdown", invite())],
        },
        Group {
            title: "Addons",
            columns: 3,
            cards: vec![
                card(
                    "Prefix",
                    InputGroup::new("hoff.dev").start(Addon::Text("https://".into())),
                ),
                card(
                    "Suffix",
                    InputGroup::new("sophie").end(Addon::Text("@hoff.dev".into())),
                ),
                card(
                    "Icon",
                    InputGroup::new("Search...").start(Addon::Icon("search")),
                ),
            ],
        },
        Group {
            title: "Disabled",
            columns: 1,
            cards: vec![card("Disabled", search().disabled(true))],
        },
        Group {
            title: "Content",
            columns: 2,
            cards: vec![
                card(
                    "Long value",
                    search().with_text("an invoice from the first quarter of the research grant"),
                ),
                card("Right to left", search().with_text("חיפוש במסמכים")),
            ],
        },
    ];
    FormPage::new("input_group_page", groups)
}

#[cfg(test)]
mod tests {
    use accord::{Role, SemId, SnapshotBuilder};
    use engine::styling::theme::Theme;
    use engine::widgets::prelude::{Rect, WidgetEvent};

    use super::*;
    use crate::views::components::Component;
    use crate::views::text_input::host::TextHost;

    fn content() -> Rect {
        Rect::new(288.0, 80.0, 900.0, 2400.0)
    }

    /// From 280 to 1600 px every 8, in the core and the HOFF themes:
    /// every group inside its card and under its chip, its addons inside
    /// its box and the typed text's box between them.
    #[test]
    fn every_group_stays_in_its_card_from_280_to_1600() {
        let p = page();
        for theme in [Theme::hoff_core(), Theme::hoff()] {
            for w in (280..=1600).step_by(8).map(|w| w as f32) {
                let content = Rect::new(0.0, 0.0, w, 800.0);
                for (laid, group) in p.layout(content, &theme).iter().zip(&p.groups) {
                    for ((card, rect), spec) in laid.cards.iter().zip(&group.cards) {
                        assert!(card.right() <= w + 0.5, "{w}: card");
                        assert!(rect.x >= card.x && rect.right() <= card.right() + 0.5);
                        assert!(rect.bottom() <= card.bottom() + 0.5);
                        let g: &InputGroup = spec.spec.get().expect("a group");
                        let l = g.layout(*rect, &theme);
                        for addon in [l.start, l.end].into_iter().flatten() {
                            assert!(addon.x >= l.input.x && addon.right() <= l.input.right());
                        }
                        if let (Some(a), Some(b)) = (l.start, l.end) {
                            assert!(a.right() <= b.x, "{w}: the addons meet");
                        }
                        if let Some(c) = l.choice {
                            assert!(c.x >= l.input.x && c.right() <= l.input.right());
                            assert!(c.y >= l.input.y && c.bottom() <= l.input.bottom());
                        }
                    }
                }
            }
        }
    }

    /// A click on the box focuses it and the keys type; a click on the
    /// icon button presses it and leaves the box's focus where it was.
    #[test]
    fn the_box_types_and_the_icon_button_presses() {
        let theme = Theme::hoff_core();
        let mut p = page();
        let rect = p.widget_rect((0, 0), content(), &theme).unwrap();
        let (x, y) = (rect.x + rect.w / 2.0, rect.y + rect.h / 2.0);
        p.handle_event(&WidgetEvent::MouseDown { x, y }, content(), &theme);
        assert_eq!(p.focused(), Some((0, 0)));
        assert!(p.handle_text("invoice"));
        let icon = (rect.x + 6.0, y);
        let r = p.handle_event(
            &WidgetEvent::MouseDown {
                x: icon.0,
                y: icon.1,
            },
            content(),
            &theme,
        );
        assert!(r.clicked, "the icon button presses");
        assert_eq!(p.focused(), Some((0, 0)), "the box keeps the caret");
        assert!(p.ime_area(content(), &theme).is_some());
    }

    /// The search projects as a text field described by its hint, its
    /// icon button beside it, which a request presses.
    #[test]
    fn the_search_projects_with_its_hint_and_button() {
        let mut p = page();
        let root = p.root().expect("the root parses").clone();
        let mut b = SnapshotBuilder::new();
        b.node(&root, Role::Region, "Input Group");
        p.project(&mut b, &root);
        let snap = b.finish().expect("the page projects");
        let id = SemId::parse("input_group_page.group[0].card[0]").unwrap();
        let node = snap.get(&id).unwrap();
        assert_eq!(node.role, Role::TextInput);
        assert_eq!(node.description.as_deref(), Some("⌘ K"));
        let action = SemId::parse("input_group_page.group[0].card[0].action").unwrap();
        assert_eq!(snap.get(&action).map(|n| n.role), Some(Role::Button));
        let req = accord::Request::observe(
            &snap,
            accord::Actor::Test { name: "t".into() },
            &action,
            "activate",
            None,
        )
        .unwrap();
        assert!(p.run(&req).is_ok());
    }

    /// The Invite: a press on its select opens the list over the cards
    /// beneath, a press on a row picks it and closes the list; the box
    /// keeps typing beside it.
    #[test]
    fn the_invite_s_select_opens_over_the_page_and_picks() {
        let theme = Theme::hoff_core();
        let mut p = page();
        let rect = p.widget_rect((1, 0), content(), &theme).unwrap();
        let g: &InputGroup = p.spec((1, 0)).and_then(|s| s.get()).unwrap();
        let choice = g.layout(rect, &theme).choice.expect("a select");
        assert!(choice.w >= 102.0, "never under the kit's w-25.5");
        let (x, y) = choice.center();
        p.handle_event(&WidgetEvent::MouseDown { x, y }, content(), &theme);
        let g: &InputGroup = p.spec((1, 0)).and_then(|s| s.get()).unwrap();
        let dd = g.dropdown_rect(rect, &theme).expect("open");
        let row = theme.form.select.panel.row_h;
        let (rx, ry) = (dd.x + 10.0, dd.y + theme.form.select.panel.pad + row * 1.5);
        p.handle_event(&WidgetEvent::MouseDown { x: rx, y: ry }, content(), &theme);
        let g: &InputGroup = p.spec((1, 0)).and_then(|s| s.get()).unwrap();
        let c = g.choice.as_ref().unwrap();
        assert_eq!((c.selected_label(), c.is_open()), (Some("can edit"), false));
        let (bx, by) = (rect.x + 20.0, rect.y + rect.h / 2.0);
        p.handle_event(&WidgetEvent::MouseDown { x: bx, y: by }, content(), &theme);
        assert_eq!(p.focused(), Some((1, 0)));
        assert!(p.handle_text("ada@hoff.dev"));
    }
}
