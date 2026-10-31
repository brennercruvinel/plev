//! Popover page, the hoff core kit's popovers (Toolbar `ZoomControl` and
//! `ExportFiles`, the header's `NotificationsDropdown`): groups with a
//! title and a rule, cards with a chip naming the states each shows. The
//! widget is `engine::widgets::overlay::popover`; every visual value is
//! `theme.overlay.popover`.
//!
//! "Live" is the one that operates: it loads closed and opens against the
//! visible page; a press outside, a second press of its trigger, a preset
//! picked or Escape closes it, and Escape gives the focus back to its
//! trigger. The other cards show a state of the same widget, forced, as
//! the ButtonsPage shows its states: "Triggers" the panel open from the
//! kit's three triggers (the value pill, a toolbar icon, a button), and
//! "Placement" the panel open below its end, flipped to the top, or
//! shifted back in at either edge. Each of these cards is the container
//! its popover receives: the placement answers the card, never the
//! window.
//!
//! Projected: a group per title, `popover_page.group[<g>]`, and in it
//! each card's trigger, `popover_page.group[<g>].card[<c>]`, described by
//! its chip, with its `expanded`. The live trigger's `activate` opens
//! and closes it, and while open its panel `<card>.panel`, a non-modal dialog,
//! with the presets `<card>.panel[<i>]`; a preset sets the zoom the
//! value pill shows. A forced card's panel is a group of the presets as
//! labels: what it shows, with nothing to run.
//!
//! The live popover keeps its entry in the view's one overlay stack
//! (`sync`, from the page's wake); the page draws the panels on the
//! overlay layer.

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::drawing::compositor::{Compositor, LayerId};
use engine::styling::theme::Theme;
use engine::widgets::overlay::popover::{Align, Popover, PopoverTrigger, Side};
use engine::widgets::prelude::{
    Button, ButtonSize, ButtonVariant, EventResult, IconButton, IconButtonVariant, Rect,
    WidgetEvent,
};

use super::{After, Component, child_id, key_under, root_id, under};

/// The kit's zoom presets (`components/Zoom/items.tsx`), each with the
/// keys its line shows in a cap.
const PRESETS: [(&str, Zoom, &str); 5] = [
    ("Zoom in", Zoom::In, "⌘ +"),
    ("Zoom out", Zoom::Out, "⌘ -"),
    ("Zoom to 100%", Zoom::To(100), "⌘ 0"),
    ("Zoom to 200%", Zoom::To(200), "⌘ 1"),
    ("Zoom to fit", Zoom::Fit, "⌘ 2"),
];

/// The step of a zoom in or out, and its limits, in percent.
const ZOOM_STEP: u32 = 25;
const ZOOM_RANGE: (u32, u32) = (25, 400);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Zoom {
    In,
    Out,
    To(u32),
    Fit,
}

impl Zoom {
    /// Whether its line shows the check at the zoom `now` (`None`, fit):
    /// a step never does.
    fn shows(self, now: Option<u32>) -> bool {
        match self {
            Zoom::To(p) => now == Some(p),
            Zoom::Fit => now.is_none(),
            Zoom::In | Zoom::Out => false,
        }
    }
}

/// What a trigger is: the kit's value pill, a toolbar icon, a button.
enum Trigger {
    Kit(PopoverTrigger),
    Icon(IconButton),
    Button(Button),
}

impl Trigger {
    fn preferred_size(&self, theme: &Theme) -> (f32, f32) {
        match self {
            Trigger::Kit(t) => t.preferred_size(theme),
            Trigger::Icon(t) => t.preferred_size(theme),
            Trigger::Button(t) => t.preferred_size(theme),
        }
    }

    fn handle_event(&mut self, event: &WidgetEvent, bounds: Rect) -> EventResult {
        match self {
            Trigger::Kit(t) => t.handle_event(event, bounds),
            Trigger::Icon(t) => t.handle_event(event, bounds),
            Trigger::Button(t) => t.handle_event(event, bounds),
        }
    }

    fn render(&mut self, c: &mut Compositor, bounds: Rect, open: bool, theme: &Theme) {
        match self {
            Trigger::Kit(t) => t.render(c, LayerId::DEFAULT, bounds, open, theme),
            Trigger::Icon(t) => {
                // The toolbar item, selected while its panel is open.
                t.active = open;
                t.render(c, bounds, theme);
            }
            Trigger::Button(t) => t.render(c, bounds, theme),
        }
    }

    fn sem_id(&self) -> Option<&SemId> {
        match self {
            Trigger::Kit(t) => t.sem_id(),
            Trigger::Icon(t) => t.sem_id(),
            Trigger::Button(t) => t.sem_id(),
        }
    }

    fn set_focused(&mut self, on: bool) {
        match self {
            Trigger::Kit(t) => t.set_focused(on),
            Trigger::Icon(t) => t.set_focused(on),
            Trigger::Button(t) => t.set_focused(on),
        }
    }

    fn project(&self, b: &mut SnapshotBuilder, id: &SemId, parent: &SemId, states: &str) {
        let mut node = match self {
            Trigger::Kit(t) => t.project(b, id, Some(parent)),
            Trigger::Icon(t) => t.project(b, id, "zoom", Some(parent)),
            Trigger::Button(t) => t.project(b, id, Some(parent)),
        };
        node.description(states);
    }
}

/// Where a card holds its trigger.
#[derive(Clone, Copy)]
enum Spot {
    /// Centered: the live card, whose panel opens against the page.
    Center,
    TopStart,
    TopEnd,
    BottomCenter,
}

struct Card {
    /// The chip: which states this card shows.
    title: &'static str,
    trigger: Trigger,
    popover: Popover,
    spot: Spot,
    /// The presets in the panel, as buttons.
    presets: Vec<Button>,
}

struct Group {
    title: &'static str,
    /// Cards per row at full width.
    columns: usize,
    /// The cards show their panel open, forced, each card the container
    /// its popover receives; otherwise they operate.
    pinned: bool,
    cards: Vec<Card>,
}

/// Where the keyboard focus is: a card's trigger or one of its presets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum At {
    Trigger(usize, usize),
    Preset(usize, usize, usize),
}

pub struct PopoverPage {
    root: Option<SemId>,
    groups: Vec<Group>,
    zoom: Option<u32>,
    focus: Option<At>,
}

/// `popover_page.group[<g>]`.
fn group_id(root: &SemId, g: usize) -> Option<SemId> {
    child_id(Some(root), "group").map(|family| family.key(g.to_string()))
}

/// `popover_page.group[<g>].card[<c>]`.
fn card_id(root: &SemId, g: usize, c: usize) -> Option<SemId> {
    child_id(group_id(root, g).as_ref(), "card").map(|family| family.key(c.to_string()))
}

fn zoom_label(zoom: Option<u32>) -> String {
    zoom.map_or_else(|| "Fit".to_owned(), |p| format!("{p}%"))
}

impl PopoverPage {
    pub fn new() -> Self {
        let root = root_id("popover_page");
        let card = |title, trigger, popover, spot| Card {
            title,
            trigger,
            popover,
            spot,
            presets: PRESETS
                .iter()
                .map(|(label, ..)| {
                    Button::new(*label)
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::Sm)
                })
                .collect(),
        };
        let pill = || Trigger::Kit(PopoverTrigger::new(zoom_label(Some(100))));
        let mut groups = vec![
            Group {
                title: "Live",
                columns: 1,
                pinned: false,
                cards: vec![card(
                    "Default, Hover, Open, Focus",
                    pill(),
                    Popover::new(),
                    Spot::Center,
                )],
            },
            Group {
                title: "Triggers",
                columns: 3,
                pinned: true,
                cards: vec![
                    card("Open", pill(), Popover::new(), Spot::TopEnd),
                    card(
                        "Toolbar icon, open",
                        Trigger::Icon(
                            IconButton::new("layers").variant(IconButtonVariant::Toolbar),
                        ),
                        Popover::new(),
                        Spot::TopEnd,
                    ),
                    card(
                        "Button, open",
                        Trigger::Button(Button::new("Export")),
                        Popover::new(),
                        Spot::TopEnd,
                    ),
                ],
            },
            Group {
                title: "Placement",
                columns: 2,
                pinned: true,
                cards: vec![
                    card("Bottom end", pill(), Popover::new(), Spot::TopEnd),
                    card(
                        "Flips to the top",
                        pill(),
                        Popover::new().align(Align::Center),
                        Spot::BottomCenter,
                    ),
                    card(
                        "Shifts in at the start",
                        pill(),
                        Popover::new(),
                        Spot::TopStart,
                    ),
                    card(
                        "Shifts in at the end",
                        pill(),
                        Popover::new().side(Side::Bottom).align(Align::Start),
                        Spot::TopEnd,
                    ),
                ],
            },
        ];
        if let Some(root) = root.as_ref() {
            for (g, group) in groups.iter_mut().enumerate() {
                let pinned = group.pinned;
                for (c, card) in group.cards.iter_mut().enumerate() {
                    let Some(id) = card_id(root, g, c) else {
                        continue;
                    };
                    card.trigger = match std::mem::replace(&mut card.trigger, pill()) {
                        Trigger::Kit(t) => Trigger::Kit(t.sem(id.clone())),
                        Trigger::Icon(t) => Trigger::Icon(t.sem(id.clone())),
                        Trigger::Button(t) => Trigger::Button(t.sem(id.clone())),
                    };
                    // A forced card's presets are labels, never bound.
                    let Some(panel) = Popover::panel_id(&id).filter(|_| !pinned) else {
                        continue;
                    };
                    for (i, preset) in card.presets.iter_mut().enumerate() {
                        *preset = std::mem::replace(preset, Button::new(""))
                            .sem(panel.key(i.to_string()));
                    }
                }
            }
        }
        Self {
            root,
            groups,
            zoom: Some(100),
            focus: None,
        }
    }

    fn card(&self, g: usize, c: usize) -> Option<&Card> {
        self.groups.get(g)?.cards.get(c)
    }

    /// Whether `(g, c)` shows a forced state rather than operating.
    fn pinned(&self, g: usize) -> bool {
        self.groups.get(g).is_some_and(|group| group.pinned)
    }

    /// The card and the preset `id` names, if it is one of the page's.
    fn at(&self, id: &SemId) -> Option<At> {
        let root = self.root.as_ref()?;
        for (g, group) in self.groups.iter().enumerate() {
            for c in 0..group.cards.len() {
                let trigger = card_id(root, g, c)?;
                if *id == trigger {
                    return Some(At::Trigger(g, c));
                }
                let panel = Popover::panel_id(&trigger)?;
                if let Some(i) = key_under(&panel, id).and_then(|k| k.parse().ok()) {
                    return Some(At::Preset(g, c, i));
                }
            }
        }
        None
    }

    fn set_focus(&mut self, at: Option<At>) {
        self.focus = at;
        for (g, group) in self.groups.iter_mut().enumerate() {
            for (c, card) in group.cards.iter_mut().enumerate() {
                card.trigger.set_focused(at == Some(At::Trigger(g, c)));
                for (i, preset) in card.presets.iter_mut().enumerate() {
                    preset.set_focused(at == Some(At::Preset(g, c, i)));
                }
            }
        }
    }

    /// Opens the popover of the live card `(g, c)`.
    fn open(&mut self, g: usize, c: usize) {
        if self.pinned(g) {
            return;
        }
        if let Some(card) = self.groups.get_mut(g).and_then(|gr| gr.cards.get_mut(c)) {
            card.popover.open();
        }
    }

    /// Closes the popover of card `(g, c)`; the focus, if it was in its
    /// panel, goes back to its trigger.
    fn close(&mut self, g: usize, c: usize) {
        if let Some(card) = self.groups.get_mut(g).and_then(|gr| gr.cards.get_mut(c)) {
            card.popover.close();
        }
        if matches!(self.focus, Some(At::Preset(fg, fc, _)) if (fg, fc) == (g, c)) {
            self.set_focus(Some(At::Trigger(g, c)));
        }
    }

    /// A press of the trigger of `(g, c)`, by the pointer or the dispatcher.
    fn press(&mut self, g: usize, c: usize) {
        if self.card(g, c).is_some_and(|card| card.popover.is_open()) {
            self.close(g, c);
        } else {
            self.open(g, c);
        }
    }

    /// Preset `i` of `(g, c)` picked: the zoom changes, the popover
    /// closes, the focus goes back to the trigger.
    fn pick(&mut self, g: usize, c: usize, i: usize) {
        let Some((_, zoom, _)) = PRESETS.get(i) else {
            return;
        };
        let (lo, hi) = ZOOM_RANGE;
        let now = self.zoom.unwrap_or(100);
        self.zoom = match *zoom {
            Zoom::In => Some((now + ZOOM_STEP).min(hi)),
            Zoom::Out => Some(now.saturating_sub(ZOOM_STEP).max(lo)),
            Zoom::To(p) => Some(p),
            Zoom::Fit => None,
        };
        let label = zoom_label(self.zoom);
        for group in &mut self.groups {
            for card in &mut group.cards {
                if let Trigger::Kit(t) = &mut card.trigger {
                    t.label.clone_from(&label);
                }
            }
        }
        if let Some(card) = self.groups.get_mut(g).and_then(|gr| gr.cards.get_mut(c)) {
            card.popover.close();
        }
        if self.focus.is_some() {
            self.set_focus(Some(At::Trigger(g, c)));
        }
    }

    /// The focus a reader left, gone. `true` when there was one.
    fn blur_focus(&mut self) -> bool {
        let had = self.focus.is_some();
        self.set_focus(None);
        had
    }

    /// The open popover, if one is.
    fn open_card(&self) -> Option<(usize, usize)> {
        self.groups.iter().enumerate().find_map(|(g, group)| {
            group
                .cards
                .iter()
                .position(|card| card.popover.is_open())
                .map(|c| (g, c))
        })
    }
}

impl Component for PopoverPage {
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
                card.trigger.project(b, id, &at, card.title);
                if group.pinned {
                    project_forced(b, id, &at);
                    continue;
                }
                let Some(_) = card.popover.project(b, id, "Zoom", Some(&at)) else {
                    continue;
                };
                let Some(panel) = Popover::panel_id(id) else {
                    continue;
                };
                for (preset, (.., keys)) in card.presets.iter().zip(PRESETS) {
                    if let Some(pid) = preset.sem_id() {
                        preset.project(b, pid, Some(&panel)).description(keys);
                    }
                }
            }
        }
        let focused = match self.focus {
            Some(At::Trigger(g, c)) => self.card(g, c).and_then(|card| card.trigger.sem_id()),
            Some(At::Preset(g, c, i)) => self
                .card(g, c)
                .filter(|card| card.popover.is_open())
                .and_then(|card| card.presets.get(i)?.sem_id()),
            None => None,
        };
        if let Some(id) = focused {
            b.focus(id);
        }
    }

    /// A trigger's `activate` (a press); a panel's
    /// `close`; a preset's `activate` (a pick): what a click does.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        let action = req.action.as_str();
        let panel_of = |page: &Self| {
            page.root.as_ref().and_then(|root| {
                page.groups.iter().enumerate().find_map(|(g, group)| {
                    (0..group.cards.len()).find_map(|c| {
                        let panel = Popover::panel_id(&card_id(root, g, c)?)?;
                        (panel == req.target).then_some((g, c))
                    })
                })
            })
        };
        let open = |page: &Self, g, c| page.card(g, c).is_some_and(|card| card.popover.is_open());
        match (self.at(&req.target), action) {
            // A forced card's trigger presses and changes nothing.
            (Some(At::Trigger(g, _)), "activate") if self.pinned(g) => {}
            (Some(At::Trigger(g, c)), "activate") => self.press(g, c),
            (Some(At::Preset(g, c, i)), "activate") if open(self, g, c) => self.pick(g, c, i),
            (None, "close") => match panel_of(self) {
                Some((g, c)) if open(self, g, c) => self.close(g, c),
                _ => return Err(format!("{} is not an open panel", req.target)),
            },
            _ => {
                return Err(format!(
                    "the popover page has no {action} on {}",
                    req.target
                ));
            }
        }
        Ok(After::Nothing)
    }

    fn focus(&mut self, id: &SemId) -> bool {
        if !self.root.as_ref().is_some_and(|root| under(root, id)) {
            return false;
        }
        let at = self.at(id).filter(|at| match *at {
            At::Preset(g, c, _) => self.card(g, c).is_some_and(|card| card.popover.is_open()),
            At::Trigger(..) => true,
        });
        self.set_focus(at);
        true
    }

    /// Escape: the open popover closes and the focus goes back to its
    /// trigger; with none open, the focus goes.
    fn blur(&mut self) -> bool {
        if let Some((g, c)) = self.open_card() {
            self.close(g, c);
            if self.focus.is_some() {
                self.set_focus(Some(At::Trigger(g, c)));
            }
            return true;
        }
        self.blur_focus()
    }
}

impl super::super::ShowcaseView {
    /// The container a page's anchored overlay receives (the popover, the
    /// hover card): the visible page under
    /// the header in the main window (`content` scrolls past it), the
    /// page itself in a detached one.
    pub(in crate::views) fn anchored_bounds(&self, content: Rect) -> Rect {
        if content == self.page_rect() {
            self.content_rect()
        } else {
            content
        }
    }
}

/// A forced card's trigger, drawn open, and its panel as what it shows:
/// a group of the presets as labels, beside the trigger, under `parent`.
fn project_forced(b: &mut SnapshotBuilder, trigger: &SemId, parent: &SemId) {
    if let Some(mut node) = b.reopen(trigger) {
        node.state("expanded", true);
    }
    let Some(panel) = Popover::panel_id(trigger) else {
        return;
    };
    b.node(&panel, Role::Group, "Zoom").parent(parent);
    for (i, (label, _, keys)) in PRESETS.iter().enumerate() {
        b.node(panel.key(i.to_string()), Role::Label, *label)
            .parent(&panel)
            .description(*keys);
    }
}

mod layout;

#[cfg(test)]
mod tests;
