//! Combobox page: the kit's Select (`components/Select/index.tsx`) with
//! a filter, as the kit's showcase draws a component (groups with a title
//! and a rule, cards behind a chip naming their states, the chips the
//! kit's DropdownPage's). The widget is
//! `engine::widgets::form::combobox::Combobox`; its open list floats on
//! the overlay layer and has the pointer first where it is.
//!
//! The page is a form page (`field::FormPage`): its ids under
//! `combobox_page`, a combobox projected as a select (`select` with an
//! option, `open`, `close`) with its filter as `query`, and its matching
//! options while open (`….combobox[<i>]`, `select`). Typing filters, Up
//! and Down move the highlight, Enter picks it, Escape closes.

#[cfg(test)]
use std::any::Any;

use accord::{SemId, SnapshotBuilder, Value};
use engine::drawing::compositor::{Compositor, LayerId};
use engine::styling::theme::Theme;
use engine::widgets::form::TextField;
use engine::widgets::form::combobox::Combobox;
use engine::widgets::prelude::{EventResult, Rect, WidgetEvent};

use super::field::spec::{CardWidget, Part};
use super::field::{FormPage, Group, card};

const FRAMEWORKS: [&str; 6] = ["Next.js", "SvelteKit", "Nuxt", "Remix", "Astro", "Leptos"];

fn framework() -> Combobox {
    Combobox::new(FRAMEWORKS, "Select framework...")
}

/// The Combobox tab.
pub fn page() -> FormPage {
    let groups = vec![
        Group {
            title: "Combobox",
            columns: 2,
            cards: vec![
                card("Default, Focus, Hover, Typing", framework()),
                card("Chosen", framework().selected(3)),
            ],
        },
        Group {
            title: "Disabled",
            columns: 2,
            cards: vec![
                card("Disabled", framework().disabled(true)),
                card("Disabled, Chosen", framework().selected(1).disabled(true)),
            ],
        },
        Group {
            title: "Content",
            columns: 2,
            cards: vec![
                card(
                    "Many options, Scrolls",
                    Combobox::new(
                        [
                            "Lisbon", "Madrid", "Paris", "Berlin", "Vienna", "Prague", "Warsaw",
                            "Budapest", "Athens", "Rome", "Oslo", "Helsinki",
                        ],
                        "Select city...",
                    ),
                ),
                card(
                    "Long option",
                    Combobox::new(
                        [
                            "Department of Molecular and Cellular Biology",
                            "Institute of Neuroscience",
                        ],
                        "Select department...",
                    )
                    .selected(0)
                    .empty("Nothing matches."),
                ),
            ],
        },
    ];
    FormPage::new("combobox_page", groups)
}

impl CardWidget for Combobox {
    fn with_sem(self: Box<Self>, id: SemId) -> Box<dyn CardWidget> {
        Box::new((*self).sem(id))
    }

    /// The kit's Select, `w-44`; its list floats.
    fn size(&self, room: f32, theme: &Theme) -> (f32, f32) {
        (theme.form.select.width.min(room), Combobox::height(theme))
    }

    fn render(&self, c: &mut Compositor, rect: Rect, theme: &Theme) {
        Combobox::render(self, c, rect, theme);
    }

    fn render_overlay(&self, c: &mut Compositor, layer: LayerId, rect: Rect, theme: &Theme) {
        self.render_dropdown(c, layer, rect, theme);
    }

    fn over_overlay(&self, x: f32, y: f32, rect: Rect, theme: &Theme) -> bool {
        self.dropdown_rect(rect, theme)
            .is_some_and(|r| r.contains(x, y))
    }

    /// Named by its placeholder.
    fn project(&self, b: &mut SnapshotBuilder, parent: &SemId) {
        if let Some(id) = self.sem_id() {
            let name = self.input.input.placeholder.clone();
            Combobox::project(self, b, id, name, Some(parent));
        }
    }

    fn handle_event(&mut self, event: &WidgetEvent, rect: Rect, theme: &Theme) -> EventResult {
        Combobox::handle_event(self, event, rect, theme)
    }

    fn text(&self) -> Option<&TextField> {
        Some(&self.input)
    }

    fn text_mut(&mut self) -> Option<&mut TextField> {
        Some(&mut self.input)
    }

    /// Opens with the text selected whole; closes back to the chosen
    /// option.
    fn set_focused(&mut self, focused: bool) {
        Combobox::set_focused(self, focused);
    }

    fn enter(&mut self) -> bool {
        Combobox::enter(self)
    }

    fn nav(&mut self, down: bool) -> bool {
        Combobox::nav(self, down)
    }

    /// A reader on an option highlights it, the list open.
    fn set_part_focus(&mut self, part: Option<Part>) {
        if let Some(Part::Option(i)) = part {
            self.highlight(i);
        }
    }

    fn part_focus(&self) -> Option<Part> {
        self.focused_option().map(Part::Option)
    }

    fn part_id(&self, part: Part) -> Option<SemId> {
        let id = self.sem_id()?;
        match part {
            Part::Box => Some(id.clone()),
            Part::Option(i) => Some(id.key(i.to_string())),
            _ => None,
        }
    }

    /// The box, or an option the open list shows.
    fn part(&self, id: &SemId) -> Option<Part> {
        if self.sem_id() == Some(id) {
            return Some(Part::Box);
        }
        let open = self.is_open();
        self.matches()
            .into_iter()
            .map(Part::Option)
            .find(|p| open && self.part_id(*p).as_ref() == Some(id))
    }

    fn on_box(&self, x: f32, y: f32, rect: Rect, theme: &Theme) -> bool {
        y <= rect.y + Combobox::height(theme) && rect.contains(x, y)
    }

    fn ime_area(&self, rect: Rect, theme: &Theme) -> Option<Rect> {
        Some(Combobox::ime_area(self, rect, theme))
    }

    /// `select` with a label, or an option's own `select`, picks it as a
    /// press on its row; `open` and `close` focus and blur the box.
    fn run(&mut self, part: Part, action: &str, param: Option<&Value>) -> bool {
        match (part, action, param) {
            (Part::Box, "select", Some(Value::Text(label))) => self.pick_label(label),
            (Part::Box, "open", None) => {
                Combobox::open(self);
                self.is_open()
            }
            (Part::Box, "close", None) => {
                Combobox::set_focused(self, false);
                true
            }
            (Part::Option(i), "select", None) => self.pick(i),
            _ => false,
        }
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

    fn combo(p: &FormPage, at: (usize, usize)) -> &Combobox {
        p.spec(at).and_then(|s| s.get()).expect("a combobox")
    }

    /// From 280 to 1600 px every 8, in the core and the HOFF themes:
    /// every combobox inside its card and under its chip.
    #[test]
    fn every_combobox_stays_in_its_card_from_280_to_1600() {
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
                    }
                }
            }
        }
    }

    /// A click opens the list with every option; typing filters it, Down
    /// moves the highlight, Enter picks it and closes the list; a click on
    /// a row picks that one, over the card beneath it.
    #[test]
    fn typing_filters_the_arrows_highlight_and_enter_picks() {
        let theme = Theme::hoff_core();
        let mut p = page();
        let rect = p.widget_rect((0, 0), content(), &theme).unwrap();
        let (x, y) = rect.center();
        p.handle_event(&WidgetEvent::MouseDown { x, y }, content(), &theme);
        assert_eq!(p.focused(), Some((0, 0)));
        assert!(combo(&p, (0, 0)).is_open());
        assert_eq!(combo(&p, (0, 0)).matches().len(), FRAMEWORKS.len());
        assert!(p.handle_text("e"));
        let m = combo(&p, (0, 0)).matches();
        assert_eq!(m, [0, 1, 3, 5], "Next.js, SvelteKit, Remix, Leptos");
        assert!(p.handle_nav(true));
        assert_eq!(combo(&p, (0, 0)).highlighted(), Some(1));
        assert!(p.handle_enter());
        assert_eq!(combo(&p, (0, 0)).selected_label(), Some("SvelteKit"));
        assert!(!combo(&p, (0, 0)).is_open());
        assert_eq!(combo(&p, (0, 0)).input.text(), "SvelteKit");

        p.handle_event(&WidgetEvent::MouseDown { x, y }, content(), &theme);
        p.handle_text("zzz");
        assert!(combo(&p, (0, 0)).matches().is_empty(), "no match");
        assert!(p.handle_enter(), "the list stays");
        assert!(combo(&p, (0, 0)).is_open());
        assert!(p.handle_escape());
        assert_eq!(
            combo(&p, (0, 0)).input.text(),
            "SvelteKit",
            "back to the chosen"
        );

        p.handle_event(&WidgetEvent::MouseDown { x, y }, content(), &theme);
        let dd = combo(&p, (0, 0)).dropdown_rect(rect, &theme).unwrap();
        let row_h = theme.form.select.panel.row_h;
        let (rx, ry) = (
            dd.x + 20.0,
            dd.y + theme.form.select.panel.pad + row_h * 4.5,
        );
        p.handle_event(&WidgetEvent::MouseDown { x: rx, y: ry }, content(), &theme);
        assert_eq!(combo(&p, (0, 0)).selected_label(), Some("Astro"));
        assert_eq!(p.focused(), None, "the row, not the card beneath");
    }

    /// A combobox projects as a select with its query; open, its matches
    /// as options; `select` picks by label and an option's `select` picks
    /// it.
    #[test]
    fn the_combobox_projects_as_a_select_and_its_requests_pick() {
        let mut p = page();
        let root = p.root().expect("the root parses").clone();
        let snap = |p: &FormPage| {
            let mut b = SnapshotBuilder::new();
            b.node(&root, Role::Region, "Combobox");
            p.project(&mut b, &root);
            b.finish().expect("the page projects")
        };
        let id = SemId::parse("combobox_page.group[0].card[0]").unwrap();
        let s = snap(&p);
        let node = s.get(&id).unwrap();
        assert_eq!(node.role, Role::Select);
        assert_eq!(node.state.get("expanded"), Some(&Value::Bool(false)));
        let ask = |s: &accord::Snapshot, target: &SemId, action: &str, v: Option<Value>| {
            accord::Request::observe(
                s,
                accord::Actor::Test { name: "t".into() },
                target,
                action,
                v,
            )
            .unwrap()
        };
        assert!(p.run(&ask(&s, &id, "open", None)).is_ok());
        let s = snap(&p);
        let astro = id.key("4");
        assert_eq!(s.get(&astro).map(|n| n.role), Some(Role::Option));
        assert!(p.run(&ask(&s, &astro, "select", None)).is_ok());
        assert_eq!(combo(&p, (0, 0)).selected_label(), Some("Astro"));
        let s = snap(&p);
        assert!(
            p.run(&ask(&s, &id, "select", Some(Value::from("Remix"))))
                .is_ok()
        );
        assert_eq!(combo(&p, (0, 0)).selected_label(), Some("Remix"));
    }
}
