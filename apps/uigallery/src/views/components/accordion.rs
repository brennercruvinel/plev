//! Accordion page, in the mold of the kit's ButtonsPage: groups with a
//! title and a bottom rule, cards in rows, each card a chip naming the
//! states it shows over an accordion of `engine::widgets::content`. The
//! sections are the kit's (3D Objects, Materials) with text bodies; every
//! visual value comes from the theme (`theme.disclosure`).
//!
//! The cards are as tall as what they hold: an accordion opening grows
//! its card and the row, and the groups below move down. Rows drop a
//! column where a card would get narrower than `showcase::CARD_MIN_W`;
//! the title, the card and its chip, and the rule are `showcase`'s.
//!
//! Projected: a group per title, `accordion_page.group[<g>]`; in it each
//! card's accordion, `accordion_page.group[<g>].card[<c>]`, named by the
//! card's chip; in that the triggers, `...card[<c>].item[<i>]`, whose
//! `toggle_expanded` opens or closes the section as a click does, and the
//! open sections' regions (`...item[<i>].panel`). The focus a reader or
//! the keyboard asks for is the page's own (the trigger's ring).

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;
use engine::widgets::content::{Accordion, AccordionMode};
use engine::widgets::{EventResult, Rect, Wake, WidgetEvent};

use super::{After, Component, child_id, root_id, showcase, under};

const OBJECTS: &str = "Meshes, primitives and imported scenes, ready to drop on the canvas.";
const MATERIALS: &str = "Surfaces with their color, roughness and the maps that drive them.";
const LIGHTS: &str = "Area, point and environment lights, with their color temperature.";
const LONG_TITLE: &str = "Materials imported from the shared library of the studio, sorted by the project that uses them";
const LONG_BODY: &str = "A section holds what it holds: its height comes from the text \
     measured at the width the card gets, so a narrow card makes a taller section, and the \
     section opens to exactly that height. Nothing here is cut to fit, nothing is given a \
     fixed height, and the title above wraps the same way when the card gets narrow.";

struct Card {
    /// The chip: which states this card shows.
    title: &'static str,
    accordion: Accordion,
}

struct Group {
    title: &'static str,
    /// Cards per row at full width.
    columns: usize,
    cards: Vec<Card>,
}

/// A group laid out: the title's y, each card's rect with its
/// accordion's, and the rule's y.
struct Placed {
    title_y: f32,
    cards: Vec<(Rect, Rect)>,
    rule_y: f32,
}

pub struct AccordionPage {
    root: Option<SemId>,
    groups: Vec<Group>,
    /// The trigger, `(group, card, item)`, a reader or the keyboard
    /// focused (its ring shows); Escape or a click lets it go.
    focus: Option<(usize, usize, usize)>,
}

/// `accordion_page.group[<g>]`.
fn group_id(root: &SemId, g: usize) -> Option<SemId> {
    child_id(Some(root), "group").map(|family| family.key(g.to_string()))
}

/// `accordion_page.group[<g>].card[<c>]`.
fn card_id(root: &SemId, g: usize, c: usize) -> Option<SemId> {
    child_id(group_id(root, g).as_ref(), "card").map(|family| family.key(c.to_string()))
}

impl AccordionPage {
    pub fn new(theme: &Theme) -> Self {
        let kit = |mode: AccordionMode| {
            Accordion::new(mode)
                .with_motion(&theme.disclosure.motion)
                .item("3D Objects", OBJECTS)
                .item("Materials", MATERIALS)
        };
        let mut groups = vec![
            Group {
                title: "Accordion",
                columns: 2,
                cards: vec![
                    Card {
                        title: "Single: Closed, Open, Focus",
                        accordion: kit(AccordionMode::Single).item("Lights", LIGHTS).opened(0),
                    },
                    Card {
                        title: "Multiple: Open",
                        accordion: kit(AccordionMode::Multiple).opened(0).opened(1),
                    },
                ],
            },
            Group {
                title: "Accordion, edges",
                columns: 2,
                cards: vec![
                    Card {
                        title: "Disabled",
                        accordion: kit(AccordionMode::Multiple)
                            .item("Lights", LIGHTS)
                            .opened(0)
                            .disabled_item(1),
                    },
                    Card {
                        title: "Long content",
                        accordion: Accordion::new(AccordionMode::Single)
                            .with_motion(&theme.disclosure.motion)
                            .item(LONG_TITLE, LONG_BODY)
                            .item("3D Objects", OBJECTS)
                            .opened(0),
                    },
                ],
            },
        ];
        let root = root_id("accordion_page");
        if let Some(root) = &root {
            for (g, group) in groups.iter_mut().enumerate() {
                for (c, card) in group.cards.iter_mut().enumerate() {
                    if let Some(id) = card_id(root, g, c) {
                        card.accordion = std::mem::take(&mut card.accordion).sem(id);
                    }
                }
            }
        }
        Self {
            root,
            groups,
            focus: None,
        }
    }

    /// The trigger behind `id`: `(group, card, item)`.
    fn at(&self, id: &SemId) -> Option<(usize, usize, usize)> {
        self.groups.iter().enumerate().find_map(|(g, group)| {
            group
                .cards
                .iter()
                .enumerate()
                .find_map(|(c, card)| card.accordion.position(id).map(|i| (g, c, i)))
        })
    }

    fn set_focus(&mut self, at: Option<(usize, usize, usize)>) {
        self.focus = at;
        for (g, group) in self.groups.iter_mut().enumerate() {
            for (c, card) in group.cards.iter_mut().enumerate() {
                let item = at.filter(|(fg, fc, _)| (*fg, *fc) == (g, c));
                card.accordion.set_focused(item.map(|(_, _, i)| i));
            }
        }
    }

    /// Every group placed in `content`: cards as tall as their chip and
    /// their accordion now, a row as tall as its tallest card.
    fn layout(&self, content: Rect, theme: &Theme) -> Vec<Placed> {
        let pad = theme.spacing.xxl;
        let gap = theme.spacing.sm;
        let inset = theme.spacing.sm;
        let title_h = mold::title_h(theme);
        let chip_h = mold::chip_h(theme);
        let mut y = content.y;
        let mut out = Vec::with_capacity(self.groups.len());
        for group in &self.groups {
            let title_y = y + pad;
            let cols = mold::columns(group.columns, content.w, gap);
            let card_w = (content.w - gap * (cols as f32 - 1.0)) / cols as f32;
            let widget_w = (card_w - inset * 2.0).max(0.0);
            let mut row_y = title_y + title_h + theme.spacing.lg;
            let mut cards = Vec::with_capacity(group.cards.len());
            for row in group.cards.chunks(cols) {
                let heights: Vec<f32> = row
                    .iter()
                    .map(|card| inset * 3.0 + chip_h + card.accordion.height(widget_w, theme))
                    .collect();
                let row_h = heights.iter().copied().fold(0.0, f32::max);
                for (col, card) in row.iter().enumerate() {
                    let rect = Rect::new(
                        content.x + col as f32 * (card_w + gap),
                        row_y,
                        card_w,
                        row_h,
                    );
                    let widget = Rect::new(
                        rect.x + inset,
                        rect.y + inset * 2.0 + chip_h,
                        widget_w,
                        card.accordion.height(widget_w, theme),
                    );
                    cards.push((rect, widget));
                }
                row_y += row_h + gap;
            }
            let rule_y = row_y - gap + pad;
            out.push(Placed {
                title_y,
                cards,
                rule_y,
            });
            y = rule_y;
        }
        out
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        self.layout(content, theme)
            .last()
            .map_or(0.0, |placed| placed.rule_y - content.y)
    }

    /// Every accordion's motion, merged.
    pub fn wake(&mut self, dt: f32) -> Wake {
        let mut wake = Wake::idle();
        for group in &mut self.groups {
            for card in &mut group.cards {
                wake = wake.merge(card.accordion.wake(dt));
            }
        }
        wake
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
    ) -> EventResult {
        let mut result = EventResult::IGNORED;
        if matches!(event, WidgetEvent::MouseDown { .. }) && self.blur() {
            result = EventResult::changed();
        }
        let layout = self.layout(content, theme);
        for (group, placed) in self.groups.iter_mut().zip(&layout) {
            for (card, (_, widget)) in group.cards.iter_mut().zip(&placed.cards) {
                result = result.merge(card.accordion.handle_event(event, *widget, theme));
            }
        }
        result
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let last = self.groups.len().saturating_sub(1);
        for (i, (group, placed)) in self
            .groups
            .iter()
            .zip(self.layout(content, theme))
            .enumerate()
        {
            showcase::title(c, group.title, content, placed.title_y, theme);
            for (card, (rect, widget)) in group.cards.iter().zip(&placed.cards) {
                showcase::card(c, *rect, card.title, theme);
                card.accordion.render(c, *widget, theme);
            }
            if i != last {
                showcase::rule(c, content, placed.rule_y, theme);
            }
        }
    }
}

impl Component for AccordionPage {
    fn root(&self) -> Option<&SemId> {
        self.root.as_ref()
    }

    fn project(&self, b: &mut SnapshotBuilder, root: &SemId) {
        for (g, group) in self.groups.iter().enumerate() {
            let Some(at) = group_id(root, g) else {
                continue;
            };
            b.node(&at, Role::Group, group.title).parent(root);
            for card in &group.cards {
                if let Some(id) = card.accordion.sem_id() {
                    card.accordion.project(b, id, card.title, Some(&at));
                }
            }
        }
        let focused = self.focus.and_then(|(g, c, i)| {
            self.groups
                .get(g)?
                .cards
                .get(c)?
                .accordion
                .items()
                .get(i)?
                .sem_id()
        });
        if let Some(id) = focused {
            b.focus(id);
        }
    }

    /// A trigger's `toggle_expanded` opens or closes its section, as its
    /// click does; a disabled one refuses.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        match (self.at(&req.target), req.action.as_str()) {
            (Some((g, c, i)), "toggle_expanded") => {
                if self.groups[g].cards[c].accordion.toggle(i) {
                    Ok(After::Nothing)
                } else {
                    Err(format!("{} is disabled", req.target))
                }
            }
            _ => Err(format!(
                "the accordion page has no {} on {}",
                req.action, req.target
            )),
        }
    }

    /// An enabled trigger takes the page's focus (its ring); another node
    /// of the page takes it from the triggers.
    fn focus(&mut self, id: &SemId) -> bool {
        if !self.root.as_ref().is_some_and(|root| under(root, id)) {
            return false;
        }
        let at = self
            .at(id)
            .filter(|(g, c, i)| !self.groups[*g].cards[*c].accordion.is_disabled(*i));
        self.set_focus(at);
        true
    }

    fn blur(&mut self) -> bool {
        let had = self.focus.is_some();
        self.set_focus(None);
        had
    }
}

/// What the page needs of the showcase mold that `showcase` keeps
/// private: the row's columns, the title's line and the chip's band. The
/// cards here are as tall as their accordion, so the page lays its rows
/// out itself and leaves the drawing (title, card, chip, rule) to
/// `showcase`; these three go when it makes them public.
mod mold {
    use super::showcase::CARD_MIN_W;
    use engine::styling::theme::Theme;

    /// The line of `showcase::title`.
    pub(super) fn title_h(theme: &Theme) -> f32 {
        theme
            .typography
            .exact(theme.typography.title_sm, 28.0, 500)
            .line_height
    }

    /// The height of `showcase::card`'s chip.
    pub(super) fn chip_h(theme: &Theme) -> f32 {
        let line = theme
            .typography
            .exact(theme.typography.caption, 16.0, 500)
            .line_height;
        line + theme.spacing.xs * 2.0
    }

    /// The columns of `showcase::layout`: the span until a card would get
    /// narrower than `CARD_MIN_W`.
    pub(super) fn columns(span: usize, w: f32, gap: f32) -> usize {
        (1..=span)
            .rev()
            .find(|n| (w - gap * (*n as f32 - 1.0)) / *n as f32 >= CARD_MIN_W)
            .unwrap_or(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine::compose::animation::set_reduced_motion;

    fn inside(inner: Rect, outer: Rect) -> bool {
        const EPS: f32 = 0.01;
        inner.x >= outer.x - EPS
            && inner.y >= outer.y - EPS
            && inner.right() <= outer.right() + EPS
            && inner.bottom() <= outer.bottom() + EPS
    }

    fn apart(a: Rect, b: Rect) -> bool {
        a.right() <= b.x || b.right() <= a.x || a.bottom() <= b.y || b.bottom() <= a.y
    }

    /// Lets every motion land.
    fn settle(page: &mut AccordionPage) {
        for _ in 0..120 {
            page.wake(1.0 / 60.0);
        }
    }

    fn trigger(g: usize, c: usize, i: usize) -> SemId {
        let root = root_id("accordion_page").expect("the root parses");
        card_id(&root, g, c)
            .and_then(|card| card.child("item").ok())
            .map(|family| family.key(i.to_string()))
            .expect("the trigger's id parses")
    }

    /// The page under its region, as the view projects it; the tree must
    /// be valid.
    fn seen(page: &AccordionPage) -> accord::Snapshot {
        let root = page.root.clone().expect("the root parses");
        let mut b = SnapshotBuilder::new();
        b.node(&root, Role::Region, "Accordion");
        page.project(&mut b, &root);
        b.finish().expect("the page projects a valid tree")
    }

    /// `action` on `at`, asked by a test having seen the page now.
    fn ask(page: &mut AccordionPage, at: SemId, action: &str) -> Result<After, String> {
        let req = Request {
            actor: accord::Actor::Test {
                name: "accordion".into(),
            },
            target: at,
            action: accord::ActionName::parse(action).expect("an action name"),
            param: None,
            observed: accord::Observation {
                revision: seen(page).revision(),
                values: Vec::new(),
            },
            ask: None,
        };
        page.run(&req)
    }

    /// Closed, a trigger says so and offers `toggle_expanded`, with no
    /// region; open, its region is there beside it, described by the
    /// body; a disabled trigger is not enabled.
    #[test]
    fn the_projection_says_expanded_and_shows_the_open_regions() {
        let page = AccordionPage::new(&Theme::hoff_core());
        let seen = seen(&page);
        let expanded = |id: &SemId| seen.get(id).and_then(|n| n.state.get("expanded").cloned());
        let (open, closed) = (trigger(0, 0, 0), trigger(0, 0, 1));
        assert_eq!(expanded(&open), Some(accord::Value::Bool(true)));
        assert_eq!(expanded(&closed), Some(accord::Value::Bool(false)));
        let node = seen.get(&closed).expect("the closed trigger");
        assert_eq!(node.role, Role::Button);
        assert!(
            node.actions
                .iter()
                .any(|a| a.name.as_str() == "toggle_expanded")
        );
        let panel = |id: &SemId| id.child("panel").ok().and_then(|p| seen.get(&p).cloned());
        assert!(panel(&closed).is_none());
        let region = panel(&open).expect("the open region");
        assert_eq!(region.role, Role::Region);
        assert_eq!(region.parent.as_ref(), node.parent.as_ref());
        assert_eq!(region.description.as_deref(), Some(OBJECTS));
        let disabled = seen.get(&trigger(1, 0, 1)).expect("the disabled trigger");
        assert!(!disabled.enabled);
    }

    /// The width sweep, 280 to 1600 every 8 px, under core and hoff, with
    /// the long card open and again with every section open: each card in
    /// the content, each chip and accordion in its card, each section in
    /// its accordion and each trigger in its section, no two cards of a
    /// group overlapping, the groups one under the other.
    #[test]
    fn nothing_is_cut_or_overlaps_from_280_to_1600() {
        for theme in [Theme::hoff_core(), Theme::hoff()] {
            let mut page = AccordionPage::new(&theme);
            for pass in 0..2 {
                if pass == 1 {
                    for group in &mut page.groups {
                        for card in &mut group.cards {
                            for i in 0..card.accordion.len() {
                                if !card.accordion.is_open(i) {
                                    card.accordion.toggle(i);
                                }
                            }
                        }
                    }
                    settle(&mut page);
                }
                for w in (280..=1600).step_by(8) {
                    let content = Rect::new(0.0, 0.0, w as f32, 800.0);
                    let tall = Rect::new(0.0, 0.0, w as f32, f32::MAX / 4.0);
                    let mut floor = content.y;
                    for (group, placed) in page.groups.iter().zip(page.layout(content, &theme)) {
                        assert!(placed.title_y >= floor, "{w}: {}", group.title);
                        for (k, (card, (rect, widget))) in
                            group.cards.iter().zip(&placed.cards).enumerate()
                        {
                            let what = format!("{w}px, {}", card.title);
                            assert!(
                                inside(*rect, tall) && rect.right() <= content.right() + 0.01,
                                "{what}"
                            );
                            assert!(inside(*widget, *rect), "{what}: accordion");
                            for (other, _) in &placed.cards[k + 1..] {
                                assert!(apart(*rect, *other), "{what}: overlaps");
                            }
                            let items = card.accordion.item_rects(*widget, &theme);
                            for (item, at) in card.accordion.items().iter().zip(&items) {
                                assert!(inside(*at, *widget), "{what}: section");
                                assert!(
                                    inside(item.trigger_rect(*at, &theme), *at),
                                    "{what}: trigger"
                                );
                            }
                        }
                        floor = placed.rule_y;
                    }
                }
            }
        }
    }

    #[test]
    fn rows_drop_a_column_when_narrow() {
        let theme = Theme::hoff_core();
        let page = AccordionPage::new(&theme);
        let narrow = page.layout(Rect::new(0.0, 0.0, 400.0, 800.0), &theme);
        assert!(narrow[0].cards.iter().all(|(r, _)| r.x == 0.0));
        let wide = page.layout(Rect::new(0.0, 0.0, 1200.0, 800.0), &theme);
        let [(a, _), (b, _)] = [wide[0].cards[0], wide[0].cards[1]];
        assert_eq!(a.y, b.y);
        assert!(b.x > a.x);
        assert_eq!(a.h, b.h, "a row is as tall as its tallest card");
    }

    /// A click on a closed trigger of the single card opens it and closes
    /// the open one; the page grows or shrinks as they land.
    #[test]
    fn a_click_opens_one_section_and_closes_the_other() {
        let theme = Theme::hoff_core();
        let mut page = AccordionPage::new(&theme);
        let content = Rect::new(0.0, 0.0, 1200.0, 800.0);
        let (_, widget) = page.layout(content, &theme)[0].cards[0];
        let accordion = &page.groups[0].cards[0].accordion;
        let section = accordion.item_rects(widget, &theme)[1];
        let (x, y) = accordion.items()[1].trigger_rect(section, &theme).center();
        page.handle_event(&WidgetEvent::MouseDown { x, y }, content, &theme);
        let r = page.handle_event(&WidgetEvent::MouseUp { x, y }, content, &theme);
        assert!(r.clicked);
        let accordion = &page.groups[0].cards[0].accordion;
        assert!(accordion.is_open(1) && !accordion.is_open(0));
        assert_eq!(page.wake(1.0 / 60.0), Wake::continuous(), "it moves");
        settle(&mut page);
        assert_eq!(page.wake(1.0 / 60.0), Wake::idle(), "and lands");
    }

    #[test]
    fn a_request_toggles_an_enabled_section_and_refuses_a_disabled_one() {
        let mut page = AccordionPage::new(&Theme::hoff_core());
        assert_eq!(
            ask(&mut page, trigger(0, 1, 0), "toggle_expanded"),
            Ok(After::Nothing)
        );
        assert!(!page.groups[0].cards[1].accordion.is_open(0));
        assert!(ask(&mut page, trigger(1, 0, 1), "toggle_expanded").is_err());
        assert!(!page.groups[1].cards[0].accordion.is_open(1));
        assert!(ask(&mut page, trigger(0, 0, 0), "activate").is_err());
        assert!(ask(&mut page, trigger(0, 0, 9), "toggle_expanded").is_err());
    }

    /// At rest the page asks for nothing; with the motion reduced a
    /// section opens at once and still asks for nothing.
    #[test]
    fn the_page_asks_for_no_frame_at_rest_or_with_reduced_motion() {
        let theme = Theme::hoff_core();
        let mut page = AccordionPage::new(&theme);
        assert_eq!(page.wake(1.0 / 60.0), Wake::idle());
        set_reduced_motion(true);
        let toggled = ask(&mut page, trigger(0, 0, 1), "toggle_expanded");
        let wake = page.wake(1.0 / 60.0);
        set_reduced_motion(false);
        assert_eq!(toggled, Ok(After::Nothing));
        assert_eq!(wake, Wake::idle());
        let accordion = &page.groups[0].cards[0].accordion;
        assert_eq!(accordion.items()[1].revealed(), 1.0);
        assert_eq!(accordion.items()[0].revealed(), 0.0);
        let w = 400.0;
        let opened = accordion.item_rects(Rect::new(0.0, 0.0, w, 0.0), &theme)[1];
        assert!(opened.h > accordion.items()[1].trigger_height(w, &theme));
    }

    #[test]
    fn a_disabled_trigger_takes_no_ring_and_escape_lets_the_focus_go() {
        let mut page = AccordionPage::new(&Theme::hoff_core());
        assert!(page.focus(&trigger(1, 0, 1)));
        assert_eq!(page.focus, None);
        assert!(page.focus(&trigger(0, 0, 2)));
        assert_eq!(page.focus, Some((0, 0, 2)));
        assert!(page.groups[0].cards[0].accordion.items()[2].focused());
        assert!(page.blur());
        assert!(!page.blur());
    }
}
