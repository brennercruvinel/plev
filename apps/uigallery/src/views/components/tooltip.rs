//! Tooltip page, the hoff core kit's tooltip (`components/Tooltip`, the
//! ButtonsPage's "Tooltip" group, the toolbar's `Action`). "Live" is the
//! toolbar's Undo: its tooltip waits for a resting pointer, or shows at
//! once for the keyboard's focus, above the button, or below it when
//! there is no room, on the topmost layer. "Tooltip" shows the kit's two
//! bubbles, forced: the text alone (`ButtonsPage/index.tsx:405`) and the
//! text with the key that runs it (`Toolbar/index.tsx:131`).
//!
//! Projected (task-75 phase 2b): the button, `tooltip_page.button`, as the
//! comps projects it, named by its tooltip and described by its key (what
//! the tooltip says is what the button is, heard whether or not the
//! pointer hovers). The button does nothing but press, so its `activate`
//! changes nothing. A kit bubble is a label with its text, described by
//! its key.

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::drawing::compositor::{Compositor, LayerId};
use engine::styling::theme::Theme;
use engine::widgets::prelude::{
    EventResult, IconButton, IconButtonVariant, Rect, Tooltip, WidgetEvent,
};

use super::showcase::{self, GroupLayout};
use super::{After, Component, child_id, root_id, under};

const LIVE_CHIP: &str = "Default, Hover, Focus";
const GROUPS: [&str; 2] = ["Live", "Tooltip"];
/// The kit's bubbles: each card's chip, its text and its key.
const KIT: [(&str, &str, Option<&str>); 2] = [
    ("Text", "Copy prompt", None),
    ("Shortcut", "Redo", Some("R")),
];

pub struct TooltipPage {
    root: Option<SemId>,
    button: IconButton,
    tooltip: Tooltip,
    /// The kit's bubbles, shown in their cards.
    shown: Vec<Tooltip>,
    /// The keyboard's focus is on the button (its ring shows, and its
    /// tooltip); Escape or a click lets it go.
    focus: bool,
}

impl TooltipPage {
    pub fn new() -> Self {
        let root = root_id("tooltip_page");
        // The toolbar's Undo (`Toolbar/index.tsx:121`).
        let button = IconButton::new("undo").variant(IconButtonVariant::Toolbar);
        let button = match child_id(root.as_ref(), "button") {
            Some(id) => button.sem(id),
            None => button,
        };
        let shown = KIT
            .iter()
            .map(|(_, text, key)| {
                let tip = Tooltip::new(*text);
                match key {
                    Some(k) => tip.key(*k),
                    None => tip,
                }
            })
            .collect();
        Self {
            root,
            button,
            tooltip: Tooltip::new("Undo").key("U"),
            shown,
            focus: false,
        }
    }

    /// The keyboard's focus on the button, or off it: its tooltip goes
    /// with it.
    fn set_focus(&mut self, on: bool) {
        self.focus = on;
        self.button.set_focused(on);
        if !on {
            self.tooltip.hide();
        }
    }

    /// The live card, as tall as the button needs; the kit's cards, as
    /// tall as their bubbles need, two to a row.
    fn layout(&self, content: Rect, theme: &Theme) -> Vec<GroupLayout> {
        let button_h = self.button.preferred_size(theme).1;
        let bubble_h = self
            .shown
            .iter()
            .map(|t| t.size(theme).1)
            .fold(0.0, f32::max);
        let kit = showcase::card_height(bubble_h, theme);
        showcase::layout_sized(
            [
                (1, vec![showcase::card_height(button_h, theme)]),
                (2, vec![kit; self.shown.len()]),
            ],
            content,
            theme,
        )
    }

    fn rect(&self, content: Rect, theme: &Theme) -> Rect {
        let card = self.layout(content, theme)[0].cards[0];
        showcase::centered(self.button.preferred_size(theme), card)
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        showcase::content_height(&self.layout(content, theme), content)
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
    ) -> EventResult {
        let blurred = matches!(event, WidgetEvent::MouseDown { .. }) && self.blur();
        let rect = self.rect(content, theme);
        let mut r = self.button.handle_event(event, rect);
        if blurred {
            r = r.merge(EventResult::changed());
        }
        if matches!(event, WidgetEvent::MouseMove { .. })
            && !self.focus
            && self.tooltip.set_hover(self.button.is_hovered(), rect)
        {
            r = r.merge(EventResult::changed());
        }
        r
    }

    /// Advance the tooltip delay. `true` while it needs frames.
    pub fn tick(&mut self, dt: f32) -> bool {
        self.tooltip.tick(dt)
    }

    /// The groups, the button (its tooltip shown while it has the
    /// focus), the kit's bubbles centered in their cards.
    pub fn render(&mut self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let layout = self.layout(content, theme);
        for (g, at) in layout.iter().enumerate() {
            showcase::title(c, GROUPS[g], content, at.title_y, theme);
            if g + 1 != layout.len() {
                showcase::rule(c, content, at.rule_y, theme);
            }
        }
        showcase::card(c, layout[0].cards[0], LIVE_CHIP, theme);
        let rect = self.rect(content, theme);
        if self.focus {
            self.tooltip.show(rect);
        }
        self.button.render(c, rect, theme);
        for ((tip, card), (chip, ..)) in self.shown.iter().zip(&layout[1].cards).zip(KIT) {
            showcase::card(c, *card, chip, theme);
            let at = showcase::centered(tip.size(theme), *card);
            tip.render_at(c, LayerId::DEFAULT, theme, at);
        }
    }

    /// The tooltip itself, on `layer`, positioned against the window.
    pub fn render_tooltip(
        &self,
        c: &mut Compositor,
        layer: LayerId,
        theme: &Theme,
        vw: f32,
        vh: f32,
    ) {
        self.tooltip.render(c, layer, theme, vw, vh);
    }
}

impl Component for TooltipPage {
    fn root(&self) -> Option<&SemId> {
        self.root.as_ref()
    }

    fn project(&self, b: &mut SnapshotBuilder, root: &SemId) {
        let Some(groups) = child_id(Some(root), "group") else {
            return;
        };
        let live = groups.key("0");
        b.node(&live, Role::Group, GROUPS[0]).parent(root);
        if let Some(id) = self.button.sem_id() {
            {
                let mut node = self
                    .button
                    .project(b, id, self.tooltip.text.as_str(), Some(&live));
                if let Some(key) = &self.tooltip.key {
                    node.description(key.as_str());
                }
            }
            if self.focus {
                b.focus(id);
            }
        }
        let kit = groups.key("1");
        b.node(&kit, Role::Group, GROUPS[1]).parent(root);
        let Some(cards) = child_id(Some(&kit), "card") else {
            return;
        };
        for (i, tip) in self.shown.iter().enumerate() {
            let mut node = b.node(cards.key(i.to_string()), Role::Label, tip.text.as_str());
            node.parent(&kit);
            if let Some(key) = &tip.key {
                node.description(key.as_str());
            }
        }
    }

    /// The button's `activate`: it presses, and nothing else happens, as
    /// for a click.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        if self.button.sem_id() == Some(&req.target) && req.action.as_str() == "activate" {
            return Ok(After::Nothing);
        }
        Err(format!(
            "the tooltip page has no {} on {}",
            req.action, req.target
        ))
    }

    fn focus(&mut self, id: &SemId) -> bool {
        if !self.root.as_ref().is_some_and(|root| under(root, id)) {
            return false;
        }
        self.set_focus(self.button.sem_id() == Some(id));
        true
    }

    fn blur(&mut self) -> bool {
        let had = self.focus;
        self.set_focus(false);
        had
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// From 280 to 1600 every 8 px, core and hoff: the live tooltip, shown,
    /// stays in the window clear of its button; every kit bubble fits its
    /// card.
    #[test]
    fn every_bubble_stays_in_its_container_across_the_width_range() {
        for theme in [Theme::hoff_core(), Theme::hoff()] {
            let mut page = TooltipPage::new();
            let mut w = 280.0;
            while w <= 1600.0 {
                let content = Rect::new(0.0, 0.0, w, 10_000.0);
                let rect = page.rect(content, &theme);
                page.tooltip.show(rect);
                let bubble = page.tooltip.placement(&theme, w, 800.0);
                assert!(bubble.x >= 0.0 && bubble.right() <= w + 0.01, "{w}");
                assert!(
                    bubble.bottom() <= rect.y || bubble.y >= rect.bottom(),
                    "{w}: the bubble covers its button"
                );
                for (tip, card) in page
                    .shown
                    .iter()
                    .zip(&page.layout(content, &theme)[1].cards)
                {
                    let (bw, bh) = tip.size(&theme);
                    assert!(bw <= card.w && bh <= card.h, "{w}: {} spills", tip.text);
                }
                w += 8.0;
            }
        }
    }

    /// The kit's key bubble: the key's padding, the key's square beside
    /// the text, as tall as the square and its padding.
    #[test]
    fn the_key_bubble_is_the_toolbar_s() {
        let theme = Theme::hoff_core();
        let page = TooltipPage::new();
        let k = &theme.overlay.tooltip.key;
        let (_, h) = page.shown[1].size(&theme);
        assert_eq!(h, k.size + k.pad[0] + k.pad[2], "p-0.5 around size-5");
        let (_, plain) = page.shown[0].size(&theme);
        let t = &theme.overlay.tooltip;
        assert_eq!(plain, t.text.line_height + t.pad_y * 2.0, "py-1 around 16");
    }

    /// The keyboard's focus shows the tooltip at once; Escape hides it.
    #[test]
    fn the_focus_shows_the_tooltip_and_escape_hides_it() {
        let theme = Theme::hoff_core();
        let mut page = TooltipPage::new();
        let Some(id) = page.button.sem_id().cloned() else {
            panic!("the button's id parses");
        };
        assert!(page.focus(&id));
        let mut c = Compositor::new();
        c.begin_frame();
        page.render(&mut c, Rect::new(0.0, 0.0, 1000.0, 800.0), &theme);
        assert!(page.tooltip.is_visible(), "no delay for the keyboard");
        assert!(page.blur());
        assert!(!page.tooltip.is_visible());
    }
}
