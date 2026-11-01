//! Spinner page, in the showcase mold: the spinner the kit's progress
//! rail derives (the rail wound into a ring, `theme.message.spinner`) at
//! its three sizes, then the button that starts and stops the motion.
//! They spin only while toggled on, so an idle page settles
//! (render-on-demand); with reduced motion they stand still.
//!
//! Projected: a group per title, `spinner_page.group[<g>]`; in it each
//! card as a group named by its chip, `…card[<c>]`; a spinner in it,
//! `…card[<c>].spinner`, a progress bar with no value (indeterminate,
//! busy; not a live region); and the button,
//! `spinner_page.toggle`, whose `activate` starts or stops the spinners
//! as its click does; its label says which.

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;
use engine::widgets::prelude::{
    Button, ButtonSize, ButtonVariant, EventResult, Rect, Spinner, SpinnerSize, WidgetEvent,
};

use super::{After, Component, child_id, root_id, showcase, under};

/// The cards: one per size, then the button's.
const SIZES: [(&str, SpinnerSize); 3] = [
    ("Sm, inline", SpinnerSize::Sm),
    ("Md", SpinnerSize::Md),
    ("Lg, page", SpinnerSize::Lg),
];
const GROUPS: [(&str, usize); 2] = [("Size", 3), ("Motion", 1)];

pub struct SpinnerPage {
    root: Option<SemId>,
    spinners: [Spinner; 3],
    ids: [Option<SemId>; 3],
    spinning: bool,
    toggle: Button,
    /// A screen reader focused the button (its ring shows); Escape or a
    /// click lets it go.
    focus: bool,
}

struct Layout {
    groups: Vec<showcase::GroupLayout>,
    spinners: [Rect; 3],
    toggle: Rect,
}

fn card_id(root: Option<&SemId>, g: usize, c: usize) -> Option<SemId> {
    let group = child_id(root, "group").map(|f| f.key(g.to_string()));
    child_id(group.as_ref(), "card").map(|f| f.key(c.to_string()))
}

impl SpinnerPage {
    pub fn new() -> Self {
        let root = root_id("spinner_page");
        let toggle = Button::new(Self::label(false))
            .size(ButtonSize::Sm)
            .variant(ButtonVariant::Outline)
            .icon("play");
        let toggle = match child_id(root.as_ref(), "toggle") {
            Some(id) => toggle.sem(id),
            None => toggle,
        };
        Self {
            ids: std::array::from_fn(|c| {
                child_id(card_id(root.as_ref(), 0, c).as_ref(), "spinner")
            }),
            root,
            spinners: SIZES.map(|(_, size)| Spinner::new().size(size)),
            spinning: false,
            toggle,
            focus: false,
        }
    }

    /// What the button says while the spinners spin, or not.
    fn label(spinning: bool) -> &'static str {
        if spinning {
            "Stop spinners"
        } else {
            "Start spinners"
        }
    }

    /// Start or stop the spinners, the button saying which: the click's
    /// path and a request's.
    fn set_spinning(&mut self, on: bool) {
        self.spinning = on;
        self.toggle.label = Self::label(on).to_string();
    }

    /// The reader's focus on the button, or off it.
    fn set_focus(&mut self, on: bool) {
        self.focus = on;
        self.toggle.set_focused(on);
    }

    fn layout(&self, content: Rect, theme: &Theme) -> Layout {
        let groups = showcase::layout_measured(
            [
                (GROUPS[0].0, GROUPS[0].1, SIZES.len()),
                (GROUPS[1].0, GROUPS[1].1, 1),
            ],
            content,
            theme,
            |g, c, _| match g {
                0 => self.spinners[c].size.px(theme),
                _ => self.toggle.preferred_size(theme).1,
            },
        );
        let spinners = std::array::from_fn(|i| {
            let px = self.spinners[i].size.px(theme);
            showcase::centered((px, px), groups[0].cards[i])
        });
        let toggle = showcase::centered(self.toggle.preferred_size(theme), groups[1].cards[0]);
        Layout {
            groups,
            spinners,
            toggle,
        }
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        showcase::content_height(&self.layout(content, theme).groups, content)
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
    ) -> EventResult {
        let blurred = matches!(event, WidgetEvent::MouseDown { .. }) && self.blur();
        let l = self.layout(content, theme);
        let r = self.toggle.handle_event(event, l.toggle);
        if r.clicked {
            self.set_spinning(!self.spinning);
            return EventResult::clicked();
        }
        if blurred {
            return r.merge(EventResult::changed());
        }
        r
    }

    pub fn tick(&mut self, dt: f32) -> bool {
        if !self.spinning {
            return false;
        }
        // with reduced motion on each stands still and asks for nothing;
        // every one is ticked (no short circuit)
        let mut moved = false;
        for s in &mut self.spinners {
            moved |= s.tick(dt);
        }
        moved
    }

    pub fn render(&mut self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let l = self.layout(content, theme);
        let last = l.groups.len().saturating_sub(1);
        for (g, placed) in l.groups.iter().enumerate() {
            showcase::title(c, GROUPS[g].0, content, placed.title_y, theme);
            if g != last {
                showcase::rule(c, content, placed.rule_y, theme);
            }
        }
        for (((spinner, rect), card), (chip, _)) in self
            .spinners
            .iter()
            .zip(l.spinners)
            .zip(&l.groups[0].cards)
            .zip(SIZES)
        {
            showcase::card(c, *card, chip, theme);
            spinner.render(c, rect, theme);
        }
        for (id, rect) in self.ids.iter().zip(l.spinners) {
            if let Some(id) = id {
                c.bind(id, [rect.x, rect.y, rect.w, rect.h]);
            }
        }
        showcase::card(c, l.groups[1].cards[0], "Start, Stop", theme);
        self.toggle.render(c, l.toggle, theme);
    }
}

impl Component for SpinnerPage {
    fn root(&self) -> Option<&SemId> {
        self.root.as_ref()
    }

    fn project(&self, b: &mut SnapshotBuilder, root: &SemId) {
        let group = |g: usize| child_id(Some(root), "group").map(|f| f.key(g.to_string()));
        for (g, (title, _)) in GROUPS.iter().enumerate() {
            if let Some(gid) = group(g) {
                b.node(&gid, Role::Group, *title).parent(root);
            }
        }
        for (c, ((chip, _), id)) in SIZES.iter().zip(&self.ids).enumerate() {
            let (Some(gid), Some(cid), Some(id)) = (group(0), card_id(Some(root), 0, c), id) else {
                continue;
            };
            b.node(&cid, Role::Group, *chip).parent(&gid);
            b.node(id, Role::ProgressBar, "Loading")
                .parent(&cid)
                .state("busy", true)
                .state("spinning", self.spinning);
        }
        let (Some(gid), Some(cid), Some(id)) =
            (group(1), card_id(Some(root), 1, 0), self.toggle.sem_id())
        else {
            return;
        };
        b.node(&cid, Role::Group, "Start, Stop").parent(&gid);
        self.toggle.project(b, id, Some(&cid));
        if self.focus {
            b.focus(id);
        }
    }

    /// The button's `activate` starts or stops the spinners, as its click.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        if self.toggle.sem_id() == Some(&req.target) && req.action.as_str() == "activate" {
            self.set_spinning(!self.spinning);
            return Ok(After::Nothing);
        }
        Err(format!(
            "the spinner page has no {} on {}",
            req.action, req.target
        ))
    }

    fn focus(&mut self, id: &SemId) -> bool {
        if !self.root.as_ref().is_some_and(|root| under(root, id)) {
            return false;
        }
        self.set_focus(self.toggle.sem_id() == Some(id));
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

    const FRAME: f32 = 1.0 / 60.0;

    /// Started, the page asks for every frame; with the system's "reduce
    /// motion" on (task-75 phase 2a) it asks for none, and asks again once
    /// the preference is off.
    #[test]
    fn the_page_asks_for_no_frame_with_reduced_motion() {
        let mut page = SpinnerPage::new();
        assert!(!page.tick(FRAME), "at rest until the person starts it");
        page.spinning = true;
        assert!(page.tick(FRAME), "with motion");
        engine::compose::animation::set_reduced_motion(true);
        let moved = page.tick(FRAME);
        engine::compose::animation::set_reduced_motion(false);
        assert!(!moved, "reduced: it stands still");
        assert!(page.tick(FRAME), "motion again");
    }

    /// The width sweep, 280 to 1600 every 8 px: every spinner and the
    /// button inside its card, the groups down the page.
    #[test]
    fn the_page_holds_over_the_whole_width_range() {
        let page = SpinnerPage::new();
        let inside = |a: Rect, b: Rect| {
            a.x >= b.x && a.y >= b.y && a.right() <= b.right() && a.bottom() <= b.bottom()
        };
        for theme in [Theme::hoff_core(), Theme::hoff()] {
            for w in (280..=1600).step_by(8) {
                let content = Rect::new(24.0, 0.0, w as f32, 800.0);
                let l = page.layout(content, &theme);
                for (s, card) in l.spinners.iter().zip(&l.groups[0].cards) {
                    assert!(inside(*s, *card), "{w}");
                }
                assert!(inside(l.toggle, l.groups[1].cards[0]), "{w}");
                assert!(l.groups[1].title_y >= l.groups[0].rule_y);
            }
        }
    }

    /// Each spinner is an indeterminate progress bar that says whether it
    /// spins, and a start turns them.
    #[test]
    fn a_spinner_is_busy_and_says_whether_it_spins() {
        let mut page = SpinnerPage::new();
        let root = page.root().cloned().expect("the root parses");
        let spinning = |page: &SpinnerPage| {
            let mut b = SnapshotBuilder::new();
            b.node(&root, Role::Region, "Spinner");
            page.project(&mut b, &root);
            let seen = b.finish().expect("the projection validates");
            let id = page.ids[1].clone().expect("an id");
            let n = seen.get(&id).cloned().expect("the spinner");
            assert_eq!(n.role, Role::ProgressBar);
            n.state.get("spinning").cloned()
        };
        assert_eq!(spinning(&page), Some(accord::Value::Bool(false)));
        page.set_spinning(true);
        assert_eq!(spinning(&page), Some(accord::Value::Bool(true)));
    }
}
