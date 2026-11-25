//! Hover Card page: a person's card that opens while the pointer rests on
//! their handle, and stays while it moves into the card. The widget is
//! `engine::widgets::overlay::hover_card`, the kit's popover opened by the
//! tooltip's gesture; every visual value is the theme's (the popover's
//! panel, the avatar, the type ramp).
//!
//! "Live" operates: it waits `open_delay` under a resting pointer, closes
//! `close_delay` after the pointer leaves handle and card, opens at once on
//! a press (where nothing hovers: a tap, Enter or Space on the focused
//! handle, a reader's press), and Escape closes it. "States" shows the card open, forced, each card the
//! container it receives: below its handle, and flipped to the top.
//!
//! Projected: `hover_card_page.group[<g>].card[<c>]`, the handle, a button
//! with its `expanded`, described by its chip; while open, its card
//! `<handle>.panel`, a group named by the person and described by the bio.

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use engine::drawing::text::{TextMeasurer, TextStyle};
use engine::styling::theme::TypeRole;
use engine::styling::theme::{Intent, Theme};
use engine::widgets::overlay::OverlayManager;
use engine::widgets::overlay::hover_card::HoverCard;
use engine::widgets::overlay::popover::{Popover, render_panel};
use engine::widgets::prelude::{
    Avatar, AvatarSize, Button, ButtonSize, ButtonVariant, EventResult, Rect, WidgetEvent,
};

use super::showcase::{self, GroupLayout};
use super::{After, Component, child_id, root_id, under};

/// Who the card shows.
const NAME: &str = "Brenner Cruvinel";
const HANDLE: &str = "@plev";
const BIO: &str = "Builds plev, a native engine where a capability keeps its identity while its \
                   look is born, fades or changes in real time.";

struct Card {
    chip: &'static str,
    trigger: Button,
    card: HoverCard,
    /// Where the handle sits in the card: its top, or its bottom.
    bottom: bool,
}

struct Group {
    title: &'static str,
    columns: usize,
    /// Forced open, each card the container it receives.
    pinned: bool,
    cards: Vec<Card>,
}

pub struct HoverCardPage {
    root: Option<SemId>,
    groups: Vec<Group>,
    /// The handle a reader focused, `(group, card)`.
    focus: Option<(usize, usize)>,
}

fn card_id(root: &SemId, g: usize, c: usize) -> Option<SemId> {
    let group = child_id(Some(root), "group")?.key(g.to_string());
    child_id(Some(&group), "card").map(|family| family.key(c.to_string()))
}

impl HoverCardPage {
    pub fn new() -> Self {
        let root = root_id("hover_card_page");
        let handle = || {
            Button::new(HANDLE)
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm)
        };
        let card = |chip, bottom| Card {
            chip,
            trigger: handle(),
            card: HoverCard::new(),
            bottom,
        };
        let mut groups = vec![
            Group {
                title: "Live",
                columns: 1,
                pinned: false,
                cards: vec![card("Default, Hover, Open, Focus", false)],
            },
            Group {
                title: "States",
                columns: 2,
                pinned: true,
                cards: vec![card("Open", false), card("Flips to the top", true)],
            },
        ];
        if let Some(root) = root.as_ref() {
            for (g, group) in groups.iter_mut().enumerate() {
                for (c, card) in group.cards.iter_mut().enumerate() {
                    if let Some(id) = card_id(root, g, c) {
                        card.trigger = std::mem::replace(&mut card.trigger, handle()).sem(id);
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

    fn name_style(theme: &Theme) -> TextStyle {
        theme.typography.run(TypeRole::Caption.strong())
    }

    fn text_style(theme: &Theme) -> TextStyle {
        theme.typography.run(TypeRole::Caption.regular())
    }

    /// The card's size at `max_w`: the avatar, the name and handle beside
    /// it, the bio under them, wrapped, inside the panel's padding.
    fn panel_size(theme: &Theme, max_w: f32) -> (f32, f32) {
        let pad = theme.overlay.popover.pad * 2.0;
        let w = theme.overlay.hover_card.width.min(max_w);
        let inner = (w - pad * 2.0).max(0.0);
        let avatar = AvatarSize::Md.px(theme);
        let gap = theme.spacing.sm;
        let (_, bio_h) = TextMeasurer::measure_styled(BIO, &Self::text_style(theme), Some(inner));
        (w, pad * 2.0 + avatar + gap + bio_h)
    }

    /// The tallest the card gets in a forced group: the chip band, the
    /// handle, the gap and the panel.
    fn card_h(&self, g: usize, content: Rect, theme: &Theme) -> f32 {
        if !self.groups[g].pinned {
            // the live stage: its panel floats over the page
            return showcase::card_min_h(theme);
        }
        let frame = showcase::body_height(0.0, theme);
        let handle = ButtonSize::Sm.height(theme);
        let gap = theme.overlay.popover.clearance.gap;
        let narrowest = content.w - frame;
        frame + handle + gap + Self::panel_size(theme, narrowest).1
    }

    fn layout(&self, content: Rect, theme: &Theme) -> Vec<GroupLayout> {
        let spans = self.groups.iter().enumerate().map(|(g, group)| {
            let h = self.card_h(g, content, theme);
            (group.title, group.columns, group.cards.len(), h)
        });
        let groups = spans.map(|(title, span, n, h)| (title, span, vec![h; n]));
        showcase::layout_sized(groups, content, theme)
    }

    /// Every card's handle and where its panel goes: in its card's body
    /// when forced, in `viewport` for the live one.
    fn places(
        &self,
        content: Rect,
        viewport: Rect,
        theme: &Theme,
    ) -> Vec<(usize, usize, Rect, Rect)> {
        let mut out = Vec::new();
        for (g, (group, at)) in self
            .groups
            .iter()
            .zip(self.layout(content, theme))
            .enumerate()
        {
            for (c, (card, rect)) in group.cards.iter().zip(at.cards).enumerate() {
                let size = card.trigger.preferred_size(theme);
                let (trigger, bounds) = if group.pinned {
                    let body = showcase::body(rect, theme);
                    let y = if card.bottom {
                        body.bottom() - size.1
                    } else {
                        body.y
                    };
                    let x = body.x + (body.w - size.0) / 2.0;
                    (Rect::new(x.round(), y.round(), size.0, size.1), body)
                } else {
                    (showcase::centered(size, rect), viewport)
                };
                let panel = Self::panel_size(theme, bounds.w);
                let placed = card.card.popover.placement(trigger, panel, bounds, theme);
                out.push((g, c, trigger, placed.rect));
            }
        }
        out
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        showcase::content_height(&self.layout(content, theme), content)
    }

    fn live(&mut self, g: usize, c: usize) -> Option<&mut Card> {
        let group = self.groups.get_mut(g).filter(|group| !group.pinned)?;
        group.cards.get_mut(c)
    }

    fn set_focus(&mut self, at: Option<(usize, usize)>) {
        self.focus = at;
        for (g, group) in self.groups.iter_mut().enumerate() {
            for (c, card) in group.cards.iter_mut().enumerate() {
                card.trigger.set_focused(at == Some((g, c)));
            }
        }
    }

    fn at(&self, id: &SemId) -> Option<(usize, usize)> {
        let root = self.root.as_ref()?;
        self.groups.iter().enumerate().find_map(|(g, group)| {
            (0..group.cards.len())
                .find_map(|c| (card_id(root, g, c).as_ref() == Some(id)).then_some((g, c)))
        })
    }

    /// The delays and the stack, each frame of the page's wake.
    pub fn wake(
        &mut self,
        dt: f32,
        mgr: &mut OverlayManager,
        content: Rect,
        viewport: Rect,
        theme: &Theme,
    ) -> bool {
        let t = &theme.overlay.hover_card;
        let motion = theme.intent_motion(Intent::Neutral);
        let places = self.places(content, viewport, theme);
        let mut moving = false;
        for (g, c, _, panel) in places {
            let Some(card) = self.live(g, c) else {
                continue;
            };
            moving |= card.card.tick(dt, t.open_delay, t.close_delay);
            moving |= card.card.popover.sync(mgr, panel, &motion);
        }
        moving
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        viewport: Rect,
        theme: &Theme,
    ) -> EventResult {
        let mut result = EventResult::IGNORED;
        if matches!(event, WidgetEvent::MouseDown { .. }) && self.focus.is_some() {
            self.set_focus(None);
            result = EventResult::changed();
        }
        let (px, py) = event.pos();
        for (g, c, trigger, panel) in self.places(content, viewport, theme) {
            let Some(card) = self.live(g, c) else {
                continue;
            };
            let r = card.trigger.handle_event(event, trigger);
            if matches!(event, WidgetEvent::MouseMove { .. })
                && card
                    .card
                    .hover(card.trigger.is_hovered(), panel.contains(px, py))
            {
                result = result.merge(EventResult::changed());
            }
            if r.clicked {
                card.card.toggle();
            } else if card.card.popover.dismiss(event, trigger, panel) {
                card.card.close();
                result = result.merge(EventResult::changed());
            }
            result = result.merge(r);
        }
        result
    }

    /// The groups, the handles, the forced cards open in theirs; the live
    /// card, while it shows, on `anchored` at its fade.
    pub fn render(
        &self,
        c: &mut Compositor,
        anchored: LayerId,
        content: Rect,
        viewport: Rect,
        theme: &Theme,
        mgr: &OverlayManager,
    ) {
        let layout = self.layout(content, theme);
        let last = layout.len().saturating_sub(1);
        for (g, (group, at)) in self.groups.iter().zip(&layout).enumerate() {
            showcase::title(c, group.title, content, at.title_y, theme);
            for (card, rect) in group.cards.iter().zip(&at.cards) {
                showcase::card(c, *rect, card.chip, theme);
            }
            if g != last {
                showcase::rule(c, content, at.rule_y, theme);
            }
        }
        for (g, ci, trigger, panel) in self.places(content, viewport, theme) {
            let card = &self.groups[g].cards[ci];
            card.trigger.render(c, trigger, theme);
            let layer = if self.groups[g].pinned {
                LayerId::DEFAULT
            } else {
                let Some(shown) = card.card.popover.shown(mgr) else {
                    continue;
                };
                c.set_layer_opacity(anchored, shown);
                anchored
            };
            Self::render_card(c, layer, panel, theme);
        }
    }

    /// The person: the avatar, the name and handle beside it, the bio.
    fn render_card(c: &mut Compositor, layer: LayerId, panel: Rect, theme: &Theme) {
        render_panel(c, layer, panel, theme);
        let inner = panel.inset(theme.overlay.popover.pad * 2.0);
        let size = AvatarSize::Md.px(theme);
        let gap = theme.spacing.sm;
        c.push_to_layer(
            layer,
            SceneNode::PushClip {
                x: panel.x,
                y: panel.y,
                w: panel.w,
                h: panel.h,
            },
        );
        let avatar = Rect::new(inner.x, inner.y, size, size);
        Avatar::new(NAME)
            .size(AvatarSize::Md)
            .render_to_layer(c, layer, avatar, theme);
        let (name, text) = (Self::name_style(theme), Self::text_style(theme));
        let x = avatar.right() + gap;
        let lines = name.line_height + text.line_height;
        let y = inner.y + (size - lines) / 2.0;
        let w = (inner.right() - x).max(0.0);
        for (words, style, color, at) in [
            (NAME, &name, theme.colors.text, y),
            (HANDLE, &text, theme.colors.text_mid, y + name.line_height),
        ] {
            c.push_to_layer(
                layer,
                SceneNode::Text {
                    key: TextNodeKey::from_style(words, style, Some(w)),
                    x,
                    y: at,
                    color: color.0,
                },
            );
        }
        c.push_to_layer(
            layer,
            SceneNode::Text {
                key: TextNodeKey::from_style(BIO, &text, Some(inner.w)),
                x: inner.x,
                y: avatar.bottom() + gap,
                color: theme.colors.text_mid.0,
            },
        );
        c.push_to_layer(layer, SceneNode::PopClip);
    }
}

impl Component for HoverCardPage {
    fn root(&self) -> Option<&SemId> {
        self.root.as_ref()
    }

    fn project(&self, b: &mut SnapshotBuilder, root: &SemId) {
        let Some(groups) = child_id(Some(root), "group") else {
            return;
        };
        for (g, group) in self.groups.iter().enumerate() {
            let at = groups.key(g.to_string());
            b.node(&at, Role::Group, group.title).parent(root);
            for card in &group.cards {
                let Some(id) = card.trigger.sem_id() else {
                    continue;
                };
                card.trigger
                    .project(b, id, Some(&at))
                    .description(card.chip);
                if group.pinned {
                    if let Some(mut node) = b.reopen(id) {
                        node.state("expanded", true);
                    }
                    if let Some(panel) = Popover::panel_id(id) {
                        b.node(&panel, Role::Group, NAME)
                            .parent(&at)
                            .description(BIO);
                    }
                    continue;
                }
                if let Some(mut panel) =
                    card.card
                        .popover
                        .project_as(b, id, Role::Group, NAME, Some(&at))
                {
                    panel.description(BIO);
                }
            }
        }
        let focused = self
            .focus
            .and_then(|(g, c)| self.groups.get(g)?.cards.get(c)?.trigger.sem_id());
        if let Some(id) = focused {
            b.focus(id);
        }
    }

    /// A handle's `activate`: the live one opens or closes its card (a
    /// press where nothing hovers); a forced one changes nothing. The
    /// card's `close`.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        let action = req.action.as_str();
        if let Some((g, c)) = self.at(&req.target) {
            if action != "activate" {
                return Err(format!("{} has no {action}", req.target));
            }
            if let Some(card) = self.live(g, c) {
                card.card.toggle();
            }
            return Ok(After::Nothing);
        }
        let root = self.root.clone();
        for (g, group) in self.groups.iter_mut().enumerate() {
            for (c, card) in group.cards.iter_mut().enumerate() {
                let panel = root
                    .as_ref()
                    .and_then(|r| Popover::panel_id(&card_id(r, g, c)?));
                if panel.as_ref() == Some(&req.target) && action == "close" && card.card.is_open() {
                    card.card.close();
                    return Ok(After::Nothing);
                }
            }
        }
        Err(format!(
            "the hover card page has no {action} on {}",
            req.target
        ))
    }

    /// A reader's or the keyboard's focus on a handle: its ring. Enter or
    /// Space then opens the card (no hover to wait for).
    fn focus(&mut self, id: &SemId) -> bool {
        if !self.root.as_ref().is_some_and(|root| under(root, id)) {
            return false;
        }
        let at = self.at(id);
        self.set_focus(at);
        true
    }

    /// Escape: the open card closes, the focus stays on its handle; with
    /// none open, the focus goes.
    fn blur(&mut self) -> bool {
        for group in self.groups.iter_mut().filter(|g| !g.pinned) {
            for card in &mut group.cards {
                if card.card.is_open() {
                    card.card.close();
                    return true;
                }
            }
        }
        let had = self.focus.is_some();
        self.set_focus(None);
        had
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inside(inner: Rect, outer: Rect) -> bool {
        const E: f32 = 0.01;
        inner.x >= outer.x - E
            && inner.y >= outer.y - E
            && inner.right() <= outer.right() + E
            && inner.bottom() <= outer.bottom() + E
    }

    /// From 280 to 1600 every 8 px, core and hoff: every card stays in its
    /// container and never covers its handle; the forced one flips above.
    #[test]
    fn every_card_stays_in_its_container_across_the_width_range() {
        for theme in [Theme::hoff_core(), Theme::hoff()] {
            let page = HoverCardPage::new();
            let mut w = 280.0;
            while w <= 1600.0 {
                let content = Rect::new(0.0, 0.0, w, 10_000.0);
                let viewport = Rect::new(0.0, 0.0, w, 800.0);
                let layout = page.layout(content, &theme);
                for (g, c, trigger, panel) in page.places(content, viewport, &theme) {
                    let container = if page.groups[g].pinned {
                        showcase::body(layout[g].cards[c], &theme)
                    } else {
                        viewport
                    };
                    assert!(
                        inside(panel, container),
                        "{w}: ({g}, {c}) leaves its container"
                    );
                    let covers = panel.x < trigger.right()
                        && trigger.x < panel.right()
                        && panel.y < trigger.bottom()
                        && trigger.y < panel.bottom();
                    assert!(!covers, "{w}: ({g}, {c}) covers its handle");
                    if (g, c) == (1, 1) {
                        assert!(panel.bottom() <= trigger.y, "{w}: flipped above");
                    }
                }
                w += 8.0;
            }
        }
    }

    #[test]
    fn a_resting_pointer_opens_it_after_the_delay_and_leaving_closes_it() {
        let theme = Theme::hoff_core();
        let t = theme.overlay.hover_card.clone();
        let mut page = HoverCardPage::new();
        let card = &mut page.groups[0].cards[0].card;
        card.hover(true, false);
        card.tick(t.open_delay / 2.0, t.open_delay, t.close_delay);
        assert!(!card.is_open(), "still waiting");
        card.tick(t.open_delay, t.open_delay, t.close_delay);
        assert!(card.is_open());
        card.hover(false, true);
        card.tick(t.close_delay * 2.0, t.open_delay, t.close_delay);
        assert!(card.is_open(), "the pointer is in the card");
        card.hover(false, false);
        card.tick(t.close_delay * 2.0, t.open_delay, t.close_delay);
        assert!(!card.is_open(), "it left both");
    }

    #[test]
    fn a_press_opens_it_at_once_and_escape_closes_it_keeping_the_focus() {
        let mut page = HoverCardPage::new();
        let id = card_id(page.root.as_ref().unwrap(), 0, 0).unwrap();
        assert!(page.focus(&id));
        assert!(
            !page.groups[0].cards[0].card.is_open(),
            "the focus alone opens nothing"
        );
        let req = Request::observe(
            &{
                let mut b = SnapshotBuilder::new();
                b.node(page.root.clone().unwrap(), Role::Region, "Hover Card");
                page.project(&mut b, page.root.as_ref().unwrap());
                b.finish().unwrap()
            },
            accord::Actor::Human,
            &id,
            "activate",
            None,
        )
        .unwrap();
        page.run(&req).unwrap();
        assert!(page.groups[0].cards[0].card.is_open());
        assert!(page.blur());
        assert!(!page.groups[0].cards[0].card.is_open());
        assert_eq!(page.focus, Some((0, 0)));
        assert!(page.blur());
        assert_eq!(page.focus, None);
    }
}
