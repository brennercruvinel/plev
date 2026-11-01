//! Collapsible page: the kit's HomeSidebar Dropdown in the showcase mold.
//! Each card holds a collapsible (an icon tile, the label, a counter, the
//! chevron) and the list it reveals (links under a rail, the current one
//! inked). The widgets are `engine::widgets::content`'s; every visual
//! value comes from the theme (`theme.disclosure.collapsible`).
//!
//! The cards are as tall as what they hold, so a collapsible opening
//! grows its card, its row and moves the groups below; the rows drop a
//! column where a card would get narrower than the theme's least card width (`showcase::card_min_w`).
//!
//! Projected: a group per title, `collapsible_page.group[<g>]`; in it each
//! card's trigger, `...card[<c>]`, a button with `expanded` whose
//! `toggle_expanded` opens or closes it as a click does, described by the
//! card's chip; while open, its region `...card[<c>].panel` and in it the
//! links `...card[<c>].links[<i>]`, whose `activate` makes one current.

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;
use engine::widgets::content::{Collapsible, DisclosureList};
use engine::widgets::{EventResult, Rect, Wake, WidgetEvent};

use super::accordion::{Fitted, fit_layout};
use super::{After, Component, child_id, root_id, showcase, under};

const LONG_TITLE: &str = "Materials imported from the shared library of the studio";
const LONG_LINKS: [&str; 3] = [
    "Brushed metals, sorted by the project that uses them",
    "Fabrics",
    "Glass, water and everything that bends the light behind it",
];

struct Card {
    /// The chip: which states this card shows.
    title: &'static str,
    trigger: Collapsible,
    links: DisclosureList,
}

struct Group {
    title: &'static str,
    columns: usize,
    cards: Vec<Card>,
}

/// What has the page's focus: a card's trigger or one of its links.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Focus {
    Trigger(usize, usize),
    Link(usize, usize, usize),
}

pub struct CollapsiblePage {
    root: Option<SemId>,
    groups: Vec<Group>,
    focus: Option<Focus>,
}

/// `collapsible_page.group[<g>]`.
fn group_id(root: &SemId, g: usize) -> Option<SemId> {
    child_id(Some(root), "group").map(|family| family.key(g.to_string()))
}

/// `collapsible_page.group[<g>].card[<c>]`.
fn card_id(root: &SemId, g: usize, c: usize) -> Option<SemId> {
    child_id(group_id(root, g).as_ref(), "card").map(|family| family.key(c.to_string()))
}

impl CollapsiblePage {
    pub fn new(theme: &Theme) -> Self {
        let motion = &theme.disclosure.motion;
        let card = |title, trigger: Collapsible, links| Card {
            title,
            trigger: trigger.with_motion(motion),
            links,
        };
        let mut groups = vec![
            Group {
                title: "Collapsible",
                columns: 2,
                cards: vec![
                    card(
                        "Closed, Hover, Focus",
                        Collapsible::new("Assets").icon("layers").counter("112"),
                        DisclosureList::new(["3D Objects", "Materials"]),
                    ),
                    card(
                        "Open, Current",
                        Collapsible::new("Explore")
                            .icon("layout-grid")
                            .current(true)
                            .open(true),
                        DisclosureList::new(["Designs", "Animations"]).current(0),
                    ),
                ],
            },
            Group {
                title: "Collapsible, edges",
                columns: 2,
                cards: vec![
                    card(
                        "Disabled",
                        Collapsible::new("Likes")
                            .icon("heart")
                            .open(true)
                            .disabled(true),
                        DisclosureList::new(["Scenes", "Comments"]).disabled(true),
                    ),
                    card(
                        "Long content",
                        Collapsible::new(LONG_TITLE)
                            .icon("folder")
                            .counter("1,024")
                            .open(true),
                        DisclosureList::new(LONG_LINKS).current(2),
                    ),
                ],
            },
        ];
        let root = root_id("collapsible_page");
        if let Some(root) = &root {
            for (g, group) in groups.iter_mut().enumerate() {
                for (c, card) in group.cards.iter_mut().enumerate() {
                    let Some(id) = card_id(root, g, c) else {
                        continue;
                    };
                    if let Ok(links) = id.child("links") {
                        card.links = std::mem::take(&mut card.links).sem(links);
                    }
                    card.trigger = card.trigger.clone().sem(id);
                }
            }
        }
        Self {
            root,
            groups,
            focus: None,
        }
    }

    /// What `id` is on this page: a trigger or a link.
    fn at(&self, id: &SemId) -> Option<Focus> {
        self.groups.iter().enumerate().find_map(|(g, group)| {
            group.cards.iter().enumerate().find_map(|(c, card)| {
                if card.trigger.sem_id() == Some(id) {
                    Some(Focus::Trigger(g, c))
                } else {
                    card.links.position(id).map(|i| Focus::Link(g, c, i))
                }
            })
        })
    }

    fn set_focus(&mut self, at: Option<Focus>) {
        self.focus = at;
        for (g, group) in self.groups.iter_mut().enumerate() {
            for (c, card) in group.cards.iter_mut().enumerate() {
                card.trigger.set_focused(at == Some(Focus::Trigger(g, c)));
                card.links.set_focused(match at {
                    Some(Focus::Link(fg, fc, i)) if (fg, fc) == (g, c) => Some(i),
                    _ => None,
                });
            }
        }
    }

    fn card_height(card: &Card, w: f32, theme: &Theme) -> f32 {
        card.trigger.height(w, card.links.height(w, theme), theme)
    }

    /// Every group placed in `content`: cards as tall as their chip and
    /// their widget now, a row as tall as its tallest card.
    fn layout(&self, content: Rect, theme: &Theme) -> Vec<Fitted> {
        let spans: Vec<_> = self
            .groups
            .iter()
            .map(|g| (g.columns, g.cards.len()))
            .collect();
        fit_layout(&spans, content, theme, |g, c, w| {
            Self::card_height(&self.groups[g].cards[c], w, theme)
        })
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        self.layout(content, theme)
            .last()
            .map_or(0.0, |placed| placed.rule_y - content.y)
    }

    /// Every collapsible's motion, merged.
    pub fn wake(&mut self, dt: f32) -> Wake {
        let mut wake = Wake::idle();
        for group in &mut self.groups {
            for card in &mut group.cards {
                wake = wake.merge(card.trigger.wake(dt));
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
                result = result.merge(Self::card_event(card, event, *widget, theme));
            }
        }
        result
    }

    /// The trigger takes the event; the links take it only where they
    /// show (a release always, to end a press), and lose their hover
    /// elsewhere.
    fn card_event(
        card: &mut Card,
        event: &WidgetEvent,
        widget: Rect,
        theme: &Theme,
    ) -> EventResult {
        let links_h = card.links.height(widget.w, theme);
        let shown = card.trigger.shown_rect(widget, theme);
        let at = card.trigger.content_rect(widget, links_h, theme);
        let (x, y) = event.pos();
        let reaches = card.trigger.is_open()
            && (shown.contains(x, y) || matches!(event, WidgetEvent::MouseUp { .. }));
        let mut r = card.trigger.handle_event(event, widget, theme);
        if reaches {
            r = r.merge(card.links.handle_event(event, at, theme));
        } else if matches!(event, WidgetEvent::MouseMove { .. }) {
            let away = WidgetEvent::MouseMove {
                x: f32::NEG_INFINITY,
                y: f32::NEG_INFINITY,
            };
            r = r.merge(card.links.handle_event(&away, at, theme));
        }
        r
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
                let links_h = card.links.height(widget.w, theme);
                card.trigger.render(c, *widget, links_h, theme, |c, at| {
                    card.links.render(c, at, theme);
                });
            }
            if i != last {
                showcase::rule(c, content, placed.rule_y, theme);
            }
        }
    }
}

impl Component for CollapsiblePage {
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
                let Some(id) = card.trigger.sem_id() else {
                    continue;
                };
                card.trigger
                    .project(b, id, Some(&at))
                    .description(card.title);
                let open = card.trigger.project_panel(b, id, Some(&at)).is_some();
                if open && let Ok(panel) = id.child("panel") {
                    card.links.project(b, Some(&panel));
                }
            }
        }
        let focused = self.focus.and_then(|f| match f {
            Focus::Trigger(g, c) => self.groups.get(g)?.cards.get(c)?.trigger.sem_id().cloned(),
            Focus::Link(g, c, i) => self.groups.get(g)?.cards.get(c)?.links.link_id(i),
        });
        if let Some(id) = focused {
            b.focus(&id);
        }
    }

    /// A trigger's `toggle_expanded` opens or closes it, a link's
    /// `activate` makes it current, as their clicks do; a disabled card
    /// refuses both, and a closed one has no link to activate.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        let done = match (self.at(&req.target), req.action.as_str()) {
            (Some(Focus::Trigger(g, c)), "toggle_expanded") => {
                self.groups[g].cards[c].trigger.toggle()
            }
            (Some(Focus::Link(g, c, i)), "activate") => {
                let card = &mut self.groups[g].cards[c];
                card.trigger.is_open() && card.links.activate(i)
            }
            _ => {
                return Err(format!(
                    "the collapsible page has no {} on {}",
                    req.action, req.target
                ));
            }
        };
        if done {
            Ok(After::Nothing)
        } else {
            Err(format!("{} is disabled or hidden", req.target))
        }
    }

    /// An enabled trigger or link takes the page's focus (its ring);
    /// another node of the page takes it from them.
    fn focus(&mut self, id: &SemId) -> bool {
        if !self.root.as_ref().is_some_and(|root| under(root, id)) {
            return false;
        }
        let at = self.at(id).filter(|f| match *f {
            Focus::Trigger(g, c) => !self.groups[g].cards[c].trigger.is_disabled(),
            Focus::Link(g, c, _) => !self.groups[g].cards[c].links.is_disabled(),
        });
        self.set_focus(at);
        true
    }

    fn blur(&mut self) -> bool {
        let had = self.focus.is_some();
        self.set_focus(None);
        had
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

    fn id(text: &str) -> SemId {
        SemId::parse(text).expect("the id parses")
    }

    fn seen(page: &CollapsiblePage) -> accord::Snapshot {
        let root = page.root.clone().expect("the root parses");
        let mut b = SnapshotBuilder::new();
        b.node(&root, Role::Region, "Collapsible");
        page.project(&mut b, &root);
        b.finish().expect("the page projects a valid tree")
    }

    fn ask(page: &mut CollapsiblePage, at: &str, action: &str) -> Result<After, String> {
        let req = Request {
            actor: accord::Actor::Test {
                name: "collapsible".into(),
            },
            target: id(at),
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

    /// The width sweep, 280 to 1600 every 8 px, under core and hoff: each
    /// card in the content, its widget in it, the triggers and the links
    /// that show in the widget, no two cards of a row overlapping.
    #[test]
    fn nothing_is_cut_or_overlaps_from_280_to_1600() {
        for theme in [Theme::hoff_core(), Theme::hoff()] {
            let page = CollapsiblePage::new(&theme);
            for w in (280..=1600).step_by(8) {
                let content = Rect::new(0.0, 0.0, w as f32, 800.0);
                let mut floor = content.y;
                for (group, placed) in page.groups.iter().zip(page.layout(content, &theme)) {
                    assert!(placed.title_y >= floor, "{w}: {}", group.title);
                    for (k, (card, (rect, widget))) in
                        group.cards.iter().zip(&placed.cards).enumerate()
                    {
                        let what = format!("{w}px, {}", card.title);
                        assert!(
                            rect.x >= 0.0 && rect.right() <= content.right() + 0.01,
                            "{what}"
                        );
                        assert!(inside(*widget, *rect), "{what}: widget");
                        let trigger = card.trigger.trigger_rect(*widget, &theme);
                        assert!(inside(trigger, *widget), "{what}: trigger");
                        if card.trigger.is_open() {
                            let h = card.links.height(widget.w, &theme);
                            let at = card.trigger.content_rect(*widget, h, &theme);
                            for link in card.links.link_rects(at, &theme) {
                                assert!(inside(link, *widget), "{what}: link");
                            }
                        }
                        for (other, _) in &placed.cards[k + 1..] {
                            assert!(apart(*rect, *other), "{what}: overlaps");
                        }
                    }
                    floor = placed.rule_y;
                }
            }
        }
    }

    /// Closed, the trigger says so and nothing is under it; open, its
    /// region holds the links, the current one active; a disabled trigger
    /// and its links are not enabled.
    #[test]
    fn the_projection_shows_the_links_only_while_open() {
        let page = CollapsiblePage::new(&Theme::hoff_core());
        let seen = seen(&page);
        let state = |at: &str, key: &str| seen.get(&id(at)).and_then(|n| n.state.get(key).cloned());
        let closed = "collapsible_page.group[0].card[0]";
        assert_eq!(state(closed, "expanded"), Some(accord::Value::Bool(false)));
        assert!(seen.get(&id(&format!("{closed}.links[0]"))).is_none());
        let open = "collapsible_page.group[0].card[1]";
        assert_eq!(state(open, "expanded"), Some(accord::Value::Bool(true)));
        let link = seen.get(&id(&format!("{open}.links[0]"))).expect("a link");
        assert_eq!(link.role, Role::Link);
        assert_eq!(link.parent, Some(id(&format!("{open}.panel"))));
        assert_eq!(
            state(&format!("{open}.links[0]"), "active"),
            Some(accord::Value::Bool(true))
        );
        let disabled = "collapsible_page.group[1].card[0]";
        assert!(
            !seen
                .get(&id(disabled))
                .expect("the disabled trigger")
                .enabled
        );
        assert!(
            !seen
                .get(&id(&format!("{disabled}.links[0]")))
                .expect("its link")
                .enabled
        );
    }

    #[test]
    fn requests_toggle_and_activate_and_refuse_what_is_disabled_or_hidden() {
        let mut page = CollapsiblePage::new(&Theme::hoff_core());
        let open = "collapsible_page.group[0].card[1]";
        assert_eq!(
            ask(&mut page, &format!("{open}.links[1]"), "activate"),
            Ok(After::Nothing)
        );
        assert_eq!(page.groups[0].cards[1].links.current_link(), Some(1));
        assert_eq!(ask(&mut page, open, "toggle_expanded"), Ok(After::Nothing));
        assert!(!page.groups[0].cards[1].trigger.is_open());
        let disabled = "collapsible_page.group[1].card[0]";
        assert!(ask(&mut page, disabled, "toggle_expanded").is_err());
        assert!(ask(&mut page, &format!("{disabled}.links[1]"), "activate").is_err());
        assert!(ask(&mut page, open, "activate").is_err());
    }

    /// A click on a link of an open card makes it current; on a closed
    /// card the link is not there to click.
    #[test]
    fn a_click_reaches_the_links_only_where_they_show() {
        let theme = Theme::hoff_core();
        let mut page = CollapsiblePage::new(&theme);
        let content = Rect::new(0.0, 0.0, 1200.0, 800.0);
        let (_, widget) = page.layout(content, &theme)[0].cards[1];
        let card = &page.groups[0].cards[1];
        let at = card
            .trigger
            .content_rect(widget, card.links.height(widget.w, &theme), &theme);
        let (x, y) = card.links.link_rects(at, &theme)[1].center();
        page.handle_event(&WidgetEvent::MouseDown { x, y }, content, &theme);
        let r = page.handle_event(&WidgetEvent::MouseUp { x, y }, content, &theme);
        assert!(r.clicked);
        assert_eq!(page.groups[0].cards[1].links.current_link(), Some(1));
        let (_, closed) = page.layout(content, &theme)[0].cards[0];
        let card = &page.groups[0].cards[0];
        let at = card
            .trigger
            .content_rect(closed, card.links.height(closed.w, &theme), &theme);
        let (x, y) = card.links.link_rects(at, &theme)[0].center();
        page.handle_event(&WidgetEvent::MouseDown { x, y }, content, &theme);
        page.handle_event(&WidgetEvent::MouseUp { x, y }, content, &theme);
        assert_eq!(page.groups[0].cards[0].links.current_link(), None);
    }

    #[test]
    fn with_reduced_motion_a_card_opens_at_once_and_asks_for_nothing() {
        let mut page = CollapsiblePage::new(&Theme::hoff_core());
        assert_eq!(page.wake(1.0 / 60.0), Wake::idle());
        set_reduced_motion(true);
        let toggled = ask(
            &mut page,
            "collapsible_page.group[0].card[0]",
            "toggle_expanded",
        );
        let wake = page.wake(1.0 / 60.0);
        set_reduced_motion(false);
        assert_eq!(toggled, Ok(After::Nothing));
        assert_eq!(wake, Wake::idle());
        assert_eq!(page.groups[0].cards[0].trigger.revealed(), 1.0);
    }

    /// The kit's long title beside its counter, at 280 and 360 and at
    /// the person's text sizes up to twice: it wraps between words (its
    /// rule is `BreakWord`, whose min-content is the longest word) and
    /// never leaves a line of one letter.
    #[test]
    fn the_long_title_breaks_between_words_beside_its_counter() {
        use engine::drawing::compositor::{LayerId, SceneNode};
        use engine::drawing::text::{OverflowWrap, TextMeasurer, TextStyle};
        use engine::styling::theme::{Prefs, TextScale};
        let words: Vec<&str> = LONG_TITLE.split(' ').collect();
        for scale in [1.0, 1.5, 2.0] {
            let prefs = Prefs::default().with_text_scale(TextScale::new(scale));
            let theme = Theme::hoff_core().adapted(&prefs);
            let page = CollapsiblePage::new(&theme);
            for w in [280.0, 360.0] {
                let mut c = Compositor::new();
                c.begin_frame();
                page.render(&mut c, Rect::new(0.0, 0.0, w, 2000.0), &theme);
                let nodes = c.layer(LayerId::DEFAULT).unwrap().nodes().to_vec();
                let key = nodes
                    .iter()
                    .find_map(|n| match n {
                        SceneNode::Text { key, .. } if key.text == LONG_TITLE => Some(key.clone()),
                        _ => None,
                    })
                    .expect("the long title is drawn whole, wrapped");
                assert_eq!(key.overflow_wrap, OverflowWrap::BreakWord);
                let mut style = TextStyle::new(f32::from_bits(key.font_size_bits))
                    .with_line_height(f32::from_bits(key.line_height_bits))
                    .with_weight(key.font_weight)
                    .with_letter_spacing(f32::from_bits(key.letter_spacing_bits))
                    .with_overflow_wrap(key.overflow_wrap);
                if let Some(family) = &key.font_family {
                    style = style.with_family(family);
                }
                let max_w = key.max_width_bits.map(f32::from_bits).expect("wrapped");
                let lines = TextMeasurer::lines(LONG_TITLE, &style, max_w);
                let at = format!("x{scale} @{w}: {lines:?} in {max_w}");
                assert!(lines.len() > 1, "{at}: it wraps");
                for (line, _) in &lines {
                    assert!(line.chars().count() > 1, "{at}: a line of one letter");
                    assert!(
                        line.split(' ').all(|word| words.contains(&word)),
                        "{at}: a word broken"
                    );
                }
            }
        }
    }

    #[test]
    fn a_disabled_trigger_takes_no_ring_and_a_link_does() {
        let mut page = CollapsiblePage::new(&Theme::hoff_core());
        assert!(page.focus(&id("collapsible_page.group[1].card[0]")));
        assert_eq!(page.focus, None);
        assert!(page.focus(&id("collapsible_page.group[0].card[1].links[1]")));
        assert_eq!(page.focus, Some(Focus::Link(0, 1, 1)));
        assert!(page.blur());
        assert!(!page.blur());
    }
}
