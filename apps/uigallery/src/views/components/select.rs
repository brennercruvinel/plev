//! Select page: the hoff core kit's DropdownPage (`templates/
//! DropdownPage`), its Selects in their looks (with an icon, ghost, with
//! a property label, simple, small, minimal) as the kit's showcase draws
//! a component (groups with a title and a rule, cards behind a chip
//! naming their states), and the plev's disabled and long-option ones.
//! Each dropdown is drawn on the overlay layer so it floats over the
//! page; an open one takes every event first; Escape closes it.
//!
//! Projected (task-75 phase 2b): each select under the page, the first
//! `select_page.select` named "appearance" (what its options choose),
//! the others `select_page.<look>` named by their look, as the comps
//! projects a select, with its options while open: `open`, `close`,
//! `select` with an option, and an option's own `select`, each doing what
//! the click on the select or on the option does. The focus a reader or
//! the keyboard asks for is a select's, or its open option's, which the
//! panel highlights (task-75 phase 2c).

use accord::{Request, SemId, SnapshotBuilder, Value};
use engine::drawing::compositor::{Compositor, LayerId};
use engine::styling::theme::Theme;
use engine::widgets::form::tokens::SelectLook;
use engine::widgets::prelude::{EventResult, Rect, Select, WidgetEvent};

use super::showcase;
use super::{After, Component, child_id, key_under, root_id, under};

/// The kit's values.
const VALUES: [&str; 3] = ["Value", "Value 2", "Value 3"];

/// The kit's chip.
const STATES: &str = "Default, Focus, Hover";

/// A card: its chip, its select, its name in the projection, and whether
/// it is the kit's `w-44` or as wide as it measures.
struct Entry {
    chip: &'static str,
    name: &'static str,
    select: Select,
    fixed: bool,
}

/// A group laid out: its title's y, each card with its select's rect,
/// its rule's y.
type Laid = (f32, Vec<(Rect, Rect)>, f32);

struct Group {
    title: &'static str,
    entries: Vec<Entry>,
}

fn entry(chip: &'static str, name: &'static str, select: Select, fixed: bool) -> Entry {
    Entry {
        chip,
        name,
        select,
        fixed,
    }
}

pub struct SelectPage {
    root: Option<SemId>,
    groups: Vec<Group>,
    /// The open option a screen reader or the keyboard focused, with its
    /// select's place; it goes when the list closes.
    option: Option<((usize, usize), usize)>,
}

impl SelectPage {
    pub fn new() -> Self {
        let root = root_id("select_page");
        let values = || Select::new(VALUES, 0);
        let one = |title, chip, name, select, fixed| Group {
            title,
            entries: vec![entry(chip, name, select, fixed)],
        };
        let mut groups = vec![
            one(
                "Dropdown, Icon",
                STATES,
                "select",
                Select::new(
                    ["System default", "Always dark", "Always light", "Scheduled"],
                    0,
                )
                .icon("sun"),
                true,
            ),
            one(
                "Dropdown, Ghost",
                STATES,
                "ghost",
                values().look(SelectLook::Ghost).icon("layers"),
                true,
            ),
            one(
                "Dropdown, with Properties",
                STATES,
                "properties",
                values()
                    .look(SelectLook::Ghost)
                    .icon("send")
                    .label("X Post"),
                true,
            ),
            one("Dropdown, Simple", STATES, "simple", values(), true),
            one(
                "Dropdown, Small",
                STATES,
                "small",
                values().look(SelectLook::Ghost).small(true),
                false,
            ),
            one(
                "Dropdown, Minimal",
                "Default, Focus, Hover, Click",
                "minimal",
                values().look(SelectLook::Minimal).small(true),
                false,
            ),
            Group {
                title: "Disabled, Content",
                entries: vec![
                    entry("Disabled", "disabled", values().disabled(true), true),
                    entry(
                        "Long option",
                        "long",
                        Select::new(
                            [
                                "Department of Molecular and Cellular Biology",
                                "Institute of Neuroscience",
                            ],
                            0,
                        ),
                        true,
                    ),
                ],
            },
        ];
        for group in &mut groups {
            for e in &mut group.entries {
                if let Some(id) = child_id(root.as_ref(), e.name) {
                    e.select = std::mem::replace(&mut e.select, Select::new([""; 0], 0)).sem(id);
                }
            }
        }
        Self {
            root,
            groups,
            option: None,
        }
    }

    fn entries(&self) -> impl Iterator<Item = ((usize, usize), &Entry)> {
        self.groups.iter().enumerate().flat_map(|(g, group)| {
            group
                .entries
                .iter()
                .enumerate()
                .map(move |(c, e)| ((g, c), e))
        })
    }

    fn entries_mut(&mut self) -> impl Iterator<Item = &mut Entry> {
        self.groups.iter_mut().flat_map(|g| g.entries.iter_mut())
    }

    fn entry_mut(&mut self, (g, c): (usize, usize)) -> Option<&mut Entry> {
        self.groups.get_mut(g)?.entries.get_mut(c)
    }

    /// The select behind `id`, or the select an option `id` is under,
    /// with the option.
    fn at(&self, id: &SemId) -> Option<((usize, usize), Option<usize>)> {
        self.entries().find_map(|(at, e)| {
            let sid = e.select.sem_id()?;
            if sid == id {
                return Some((at, None));
            }
            let i = key_under(sid, id)?.parse::<usize>().ok()?;
            (i < e.select.options.len()).then_some((at, Some(i)))
        })
    }

    /// The select's size in a card `card_w` wide.
    fn size(e: &Entry, card_w: f32, theme: &Theme) -> (f32, f32) {
        let room = showcase::room(card_w, theme);
        let w = if e.fixed {
            theme.form.select.width
        } else {
            e.select.width_in(theme)
        };
        (w.min(room), e.select.height_in(theme))
    }

    /// Per group: the title's y, each card with its select's rect, the
    /// rule's y, on the showcase's grid.
    fn layout(&self, content: Rect, theme: &Theme) -> Vec<Laid> {
        let card_w = |n: usize| showcase::card_width(n, content, theme);
        let wanted = self.groups.iter().map(|g| {
            let w = card_w(g.entries.len());
            let h = g
                .entries
                .iter()
                .map(|e| showcase::card_height(Self::size(e, w, theme).1, theme));
            (g.entries.len(), h.collect())
        });
        showcase::layout_sized(wanted, content, theme)
            .into_iter()
            .zip(&self.groups)
            .map(|(grid, g)| {
                let cards = grid
                    .cards
                    .iter()
                    .zip(&g.entries)
                    .map(|(card, e)| {
                        let size = Self::size(e, card.w, theme);
                        (*card, showcase::centered(size, *card))
                    })
                    .collect();
                (grid.title_y, cards, grid.rule_y)
            })
            .collect()
    }

    /// Each select with its rect.
    fn rects(&self, content: Rect, theme: &Theme) -> Vec<((usize, usize), Rect)> {
        self.layout(content, theme)
            .into_iter()
            .enumerate()
            .flat_map(|(g, (_, cards, _))| {
                cards
                    .into_iter()
                    .enumerate()
                    .map(move |(c, (_, rect))| ((g, c), rect))
            })
            .collect()
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        self.layout(content, theme)
            .last()
            .map_or(0.0, |(_, _, rule)| rule - content.y)
    }

    pub fn is_open(&self) -> bool {
        self.entries().any(|(_, e)| e.select.is_open())
    }

    /// Escape: close an open dropdown, else blur. `false` when neither
    /// applied.
    pub fn handle_escape(&mut self) -> bool {
        self.option = None;
        let mut did = false;
        for e in self.entries_mut() {
            if e.select.is_open() {
                e.select.close();
                return true;
            }
        }
        for e in self.entries_mut() {
            if e.select.is_focused() {
                e.select.set_focused(false);
                did = true;
            }
        }
        did
    }

    /// An open dropdown takes every event (a press off it closes it);
    /// otherwise each select takes its own.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
    ) -> EventResult {
        let rects = self.rects(content, theme);
        let open = rects
            .iter()
            .find(|(at, _)| self.groups[at.0].entries[at.1].select.is_open());
        if let Some(&(at, rect)) = open {
            return self.entry_mut(at).map_or(EventResult::IGNORED, |e| {
                e.select.handle_event(event, rect, theme)
            });
        }
        let mut result = EventResult::IGNORED;
        for (at, rect) in rects {
            if let Some(e) = self.entry_mut(at) {
                result = result.merge(e.select.handle_event(event, rect, theme));
            }
        }
        result
    }

    pub fn render(&self, c: &mut Compositor, overlay: LayerId, content: Rect, theme: &Theme) {
        let layout = self.layout(content, theme);
        let last = self.groups.len().saturating_sub(1);
        for (i, (group, (title_y, cards, rule_y))) in self.groups.iter().zip(&layout).enumerate() {
            showcase::title(c, group.title, content, *title_y, theme);
            for (e, (card, rect)) in group.entries.iter().zip(cards) {
                showcase::card(c, *card, e.chip, theme);
                e.select.render(c, *rect, theme);
                e.select.render_dropdown(c, overlay, *rect, theme);
            }
            if i != last {
                showcase::rule(c, content, *rule_y, theme);
            }
        }
    }
}

impl Component for SelectPage {
    fn root(&self) -> Option<&SemId> {
        self.root.as_ref()
    }

    fn project(&self, b: &mut SnapshotBuilder, root: &SemId) {
        for (at, e) in self.entries() {
            let Some(id) = e.select.sem_id() else {
                continue;
            };
            let name = if e.name == "select" {
                "appearance"
            } else {
                e.name
            };
            e.select.project(b, id, name, Some(root));
            match self.option.filter(|(o, _)| *o == at && e.select.is_open()) {
                Some((_, i)) => b.focus(id.key(i.to_string())),
                None if e.select.is_focused() => b.focus(id),
                None => {}
            }
        }
    }

    /// `open` and `close` open and close a list, as a click on its
    /// select; `select` with an option, or an open option's own `select`,
    /// picks it and closes the list, as a click on the option.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        let refused = || format!("the select page has no {} on {}", req.action, req.target);
        let (at, option) = self.at(&req.target).ok_or_else(refused)?;
        let select = &mut self.entry_mut(at).ok_or_else(refused)?.select;
        let picked = match (req.action.as_str(), &req.param, option) {
            ("open", None, None) => {
                select.open();
                true
            }
            ("close", None, None) => {
                select.close();
                true
            }
            ("select", Some(Value::Text(label)), None) => select.select_label(label),
            ("select", None, Some(i)) => select.select_index(i),
            _ => false,
        };
        if !picked {
            return Err(refused());
        }
        if !select.is_open() {
            self.option = None;
        }
        Ok(After::Nothing)
    }

    /// A select takes the page's focus (its ring); an open option is held
    /// and projected; another node of the page takes it from all.
    fn focus(&mut self, id: &SemId) -> bool {
        if !self.root.as_ref().is_some_and(|root| under(root, id)) {
            return false;
        }
        let hit = self.at(id);
        self.option = hit.and_then(|(at, o)| {
            let open = self.groups[at.0].entries[at.1].select.is_open();
            o.filter(|_| open).map(|i| (at, i))
        });
        let option = self.option;
        for (g, group) in self.groups.iter_mut().enumerate() {
            for (c, e) in group.entries.iter_mut().enumerate() {
                // Its select, or an open option of it (an option of a
                // closed list focuses nothing).
                let here = hit.is_some_and(|(at, o)| {
                    at == (g, c) && (o.is_none() || option.is_some_and(|(oa, _)| oa == at))
                });
                e.select.set_focused(here);
                e.select
                    .set_highlighted(option.filter(|(at, _)| *at == (g, c)).map(|(_, i)| i));
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// From 280 to 1600 px every 8, in the core and the HOFF themes:
    /// every select inside its card and under its chip, the kit's `w-44`
    /// or as wide as it measures, never past its card.
    #[test]
    fn every_select_stays_in_its_card_from_280_to_1600() {
        let p = SelectPage::new();
        for theme in [Theme::hoff_core(), Theme::hoff()] {
            for w in (280..=1600).step_by(8).map(|w| w as f32) {
                let content = Rect::new(0.0, 0.0, w, 800.0);
                for (group, (_, cards, _)) in p.groups.iter().zip(p.layout(content, &theme)) {
                    for (e, (card, rect)) in group.entries.iter().zip(cards) {
                        let what = format!("{w}: {} / {}", group.title, e.chip);
                        assert!(card.right() <= w + 0.5, "{what}: card");
                        assert!(
                            rect.x >= card.x && rect.right() <= card.right() + 0.5,
                            "{what}"
                        );
                        assert!(rect.bottom() <= card.bottom() + 0.5, "{what}: under");
                        assert_eq!(rect.h, e.select.height_in(&theme), "{what}: its height");
                    }
                }
            }
        }
    }

    /// The first select keeps its id and options (the keyboard and the
    /// three actors read them); a click opens it, the open list takes the
    /// next press, which picks.
    #[test]
    fn the_first_select_opens_and_picks_over_the_page() {
        let theme = Theme::hoff_core();
        let content = Rect::new(288.0, 80.0, 900.0, 2400.0);
        let mut p = SelectPage::new();
        let rect = p.rects(content, &theme)[0].1;
        let first = &p.groups[0].entries[0].select;
        assert_eq!(
            first.sem_id().map(|s| s.as_str()),
            Some("select_page.select")
        );
        let (x, y) = rect.center();
        p.handle_event(&WidgetEvent::MouseDown { x, y }, content, &theme);
        assert!(p.is_open());
        let dd = p.groups[0].entries[0].select.dropdown_rect(rect, &theme);
        let row = theme.form.select.panel.row_h;
        let (rx, ry) = (dd.x + 10.0, dd.y + theme.form.select.panel.pad + row * 2.5);
        p.handle_event(&WidgetEvent::MouseDown { x: rx, y: ry }, content, &theme);
        assert!(!p.is_open());
        assert_eq!(
            p.groups[0].entries[0].select.selected_label(),
            Some("Always light")
        );
    }
}
