//! The keyboard on the component pages (task-75 phase 2c), as the window
//! delivers it and nothing more: Tab and Shift-Tab (`handle_edit_key` with
//! the modifiers the shell forwards), the arrows (`handle_nav` and
//! `handle_edit_key`), Enter (`handle_enter`), Space (`handle_key(" ")`)
//! and Escape (`close_top_overlay`). No window, no key of the system.

use engine::accessibility::focus;
use engine::compositor::SceneNode;
use engine::theme::Theme;
use sem::Value;
use showcase::view::{EditKey, NavKey};
use winit::keyboard::ModifiersState;

use super::harness::{Gallery, gallery, sid};

/// Every component page (the 19 panes of `dispatch/component.rs`, under
/// the 30 titles the gallery gives them).
const PAGES: &[&str] = &[
    "Alert Dialog",
    "Breadcrumb",
    "Button",
    "Button Group",
    "Checkbox",
    "Chip",
    "Combobox",
    "Data Table",
    "Dialog",
    "Field",
    "Hover Card",
    "Icon Button",
    "Input",
    "Item",
    "Kbd",
    "Label",
    "Navigation Menu",
    "Popover",
    "Resizable",
    "Scroll Area",
    "Select",
    "Slider",
    "Spinner",
    "Switch",
    "Table",
    "Tabs",
    "Toast",
    "Toggle",
    "Tooltip",
    "Tree",
];

/// The pages Tab does not reach, and why: their projection has no node
/// that takes the focus (task-75, "fase 2c: a sonda do teclado", (c)).
/// Making the rows focusable changes their baselines; the decision is in
/// the handoff of phase 2c.
const UNREACHABLE: &[(&str, &str)] = &[
    ("Data Table", "rows select, none is focusable"),
    ("Item", "list items select, none is focusable"),
    ("Resizable", "the split has no splitter role"),
    ("Scroll Area", "list items select, none is focusable"),
    ("Table", "rows select, none is focusable"),
];

impl Gallery {
    fn focused(&self) -> Option<String> {
        self.view
            .semantics()
            .and_then(|s| s.focused().map(ToString::to_string))
    }

    fn order(&self) -> Vec<String> {
        let seen = self.view.semantics().unwrap_or_default();
        focus::order(&seen)
            .iter()
            .map(|id| id.to_string())
            .collect()
    }

    fn tab(&mut self) -> bool {
        let took = self.view.handle_edit_key(EditKey::Tab);
        self.frame();
        took
    }

    fn back_tab(&mut self) -> bool {
        self.view.set_modifiers(ModifiersState::SHIFT);
        let took = self.tab();
        self.view.set_modifiers(ModifiersState::empty());
        took
    }

    /// Tab until `id` has the focus.
    fn tab_to(&mut self, id: &str) {
        for _ in 0..self.order().len() {
            if self.focused().as_deref() == Some(id) {
                return;
            }
            self.tab();
        }
        assert_eq!(self.focused().as_deref(), Some(id), "Tab never reached it");
    }

    fn key(&mut self, key: EditKey) -> bool {
        let took = self.view.handle_edit_key(key);
        self.frame();
        took
    }

    fn nav(&mut self, key: NavKey) -> bool {
        let took = self.view.handle_nav(key);
        self.frame();
        took
    }

    fn enter(&mut self) -> bool {
        let took = self.view.handle_enter();
        self.frame();
        took
    }

    fn space(&mut self) -> bool {
        let took = self.view.handle_key(" ");
        self.frame();
        took
    }

    /// Every focus ring drawn in the frame: the accent stroke of
    /// `comps::recipe::focus_ring`, as `(x, y, w, h)`.
    fn rings(&self) -> Vec<[f32; 4]> {
        let theme = Theme::hoff_core();
        let accent = theme.colors.accent.0;
        let ring_w = theme.control.focus_ring_width;
        self.c
            .layers()
            .iter()
            .flat_map(|l| l.nodes())
            .filter_map(|n| match *n {
                SceneNode::RoundedRect {
                    x,
                    y,
                    w,
                    h,
                    color,
                    border_width,
                    border_color,
                    ..
                } if color[3] == 0.0 && border_width == ring_w && border_color == accent => {
                    Some([x, y, w, h])
                }
                _ => None,
            })
            .collect()
    }

    /// Whether a ring is drawn on the control bound as `id`.
    fn ringed(&self, id: &str) -> bool {
        let Some([x, y, w, h]) = self.c.binding(&sid(id)).map(|b| b.rect) else {
            panic!("{id} is not bound");
        };
        self.rings()
            .iter()
            .any(|[rx, ry, rw, rh]| rx < &(x + w) && x < rx + rw && ry < &(y + h) && y < ry + rh)
    }

    fn state_of(&self, id: &str, key: &str) -> Option<Value> {
        let seen = self.view.semantics()?;
        seen.get(&sid(id))?.state.get(key).cloned()
    }
}

/// Tab walks every component page in the order of its projection, the
/// one a screen reader walks, and wraps; Shift-Tab walks it back; the
/// focus is never nowhere once Tab took it, and the control that has it
/// shows its ring.
#[test]
fn tab_walks_every_component_page_in_the_projection_s_order_with_the_ring_on() {
    let mut unreachable = Vec::new();
    for page in PAGES {
        let mut g = gallery(page);
        let order = g.order();
        if order.is_empty() {
            assert!(!g.tab(), "{page}: nothing to reach");
            assert_eq!(g.focused(), None, "{page}");
            unreachable.push(*page);
            continue;
        }
        let rings_before = g.rings().len();
        for step in 0..=order.len() {
            assert!(g.tab(), "{page}: Tab {step} is taken");
            let at = &order[step % order.len()];
            assert_eq!(g.focused().as_ref(), Some(at), "{page}: Tab {step}");
            assert!(g.ringed(at), "{page}: {at} shows its ring");
            assert_eq!(g.rings().len(), rings_before + 1, "{page}: one ring more");
        }
        for step in 1..=order.len() {
            assert!(g.back_tab(), "{page}: Shift-Tab {step}");
            let at = &order[(order.len() * 2 - step) % order.len()];
            assert_eq!(g.focused().as_ref(), Some(at), "{page}: Shift-Tab {step}");
        }
        assert!(g.view.close_top_overlay(), "{page}: Escape blurs");
        g.frame();
        assert_eq!(g.focused(), None, "{page}: after Escape");
        assert_eq!(g.rings().len(), rings_before, "{page}: the ring goes");
    }
    let known: Vec<&str> = UNREACHABLE.iter().map(|(p, _)| *p).collect();
    assert_eq!(unreachable, known);
}

/// The tab list: the arrows move along it and wrap, Home and End go to
/// its ends, Enter and Space select the focused tab.
#[test]
fn the_arrows_walk_the_tabs_and_enter_and_space_select() {
    let mut g = gallery("Tabs");
    g.tab_to("tabs_page.tabs[0]");
    let steps = [
        (EditKey::Right, "tabs_page.tabs[1]"),
        (EditKey::Right, "tabs_page.tabs[2]"),
        (EditKey::Right, "tabs_page.tabs[0]"),
        (EditKey::Left, "tabs_page.tabs[2]"),
        (EditKey::Home, "tabs_page.tabs[0]"),
        (EditKey::End, "tabs_page.tabs[2]"),
    ];
    for (key, at) in steps {
        assert!(g.key(key), "{key:?}");
        assert_eq!(g.focused().as_deref(), Some(at), "{key:?}");
        assert!(g.ringed(at));
    }
    let selected = |g: &Gallery| g.state_of("tabs_page.tabs", "selected");
    assert_eq!(
        selected(&g),
        Some(Value::Text("0".into())),
        "arrows only move"
    );
    assert!(g.enter());
    assert_eq!(selected(&g), Some(Value::Text("2".into())));
    g.key(EditKey::Left);
    assert!(g.space());
    assert_eq!(selected(&g), Some(Value::Text("1".into())));
    assert!(!g.nav(NavKey::Down), "a tab list is horizontal");
}

/// The tree: up and down along the rows it shows, right opens a closed
/// branch or goes into an open one, left closes an open branch or goes
/// to the parent row, Home and End, Enter selects.
#[test]
fn the_arrows_walk_the_tree_open_and_close_its_branches() {
    let mut g = gallery("Tree");
    g.tab_to("tree_page.tree[1]");
    let expanded = |g: &Gallery, id: &str| g.state_of(id, "expanded");
    assert!(g.nav(NavKey::Down));
    assert_eq!(g.focused().as_deref(), Some("tree_page.tree[2]"));
    assert!(g.key(EditKey::Right), "into the open branch");
    assert_eq!(g.focused().as_deref(), Some("tree_page.tree[3]"));
    assert!(g.key(EditKey::Left), "up to the parent row");
    assert_eq!(g.focused().as_deref(), Some("tree_page.tree[2]"));
    assert!(g.key(EditKey::Left), "closes the open branch");
    assert_eq!(expanded(&g, "tree_page.tree[2]"), Some(Value::Bool(false)));
    assert_eq!(g.focused().as_deref(), Some("tree_page.tree[2]"));
    assert!(g.nav(NavKey::Down), "past the closed branch");
    assert_eq!(g.focused().as_deref(), Some("tree_page.tree[5]"));
    assert!(g.nav(NavKey::Up));
    assert!(g.key(EditKey::Right), "opens the closed branch");
    assert_eq!(expanded(&g, "tree_page.tree[2]"), Some(Value::Bool(true)));
    assert!(g.key(EditKey::End));
    assert_eq!(g.focused().as_deref(), Some("tree_page.tree[11]"));
    assert!(!g.nav(NavKey::Down), "a tree does not wrap");
    assert!(g.key(EditKey::Home));
    assert_eq!(g.focused().as_deref(), Some("tree_page.tree[1]"));
    assert!(g.ringed("tree_page.tree[1]"));
    g.tab_to("tree_page.tree[5]");
    assert!(g.enter());
    assert_eq!(
        g.state_of("tree_page.tree[5]", "selected"),
        Some(Value::Bool(true))
    );
}

/// The select: down opens it on the chosen option, the arrows, Home and
/// End walk the options, Enter picks one and the focus is the select's
/// again; Space opens it too, Escape closes it.
#[test]
fn the_select_opens_on_its_choice_and_the_arrows_walk_its_options() {
    let mut g = gallery("Select");
    g.tab_to("select_page.select");
    assert!(g.nav(NavKey::Down), "opens");
    assert_eq!(
        g.state_of("select_page.select", "expanded"),
        Some(Value::Bool(true))
    );
    assert_eq!(g.focused().as_deref(), Some("select_page.select[0]"));
    assert!(g.nav(NavKey::Down));
    assert_eq!(g.focused().as_deref(), Some("select_page.select[1]"));
    assert!(g.key(EditKey::End));
    assert_eq!(g.focused().as_deref(), Some("select_page.select[3]"));
    assert!(!g.nav(NavKey::Down), "the options do not wrap");
    assert!(g.nav(NavKey::Up));
    assert!(g.enter(), "picks the focused option");
    assert_eq!(
        g.state_of("select_page.select", "selected"),
        Some(Value::Text("Always light".into()))
    );
    assert_eq!(g.focused().as_deref(), Some("select_page.select"));
    assert!(g.space(), "opens");
    assert_eq!(g.focused().as_deref(), Some("select_page.select[2]"));
    assert!(g.view.close_top_overlay(), "Escape closes");
    assert_eq!(
        g.state_of("select_page.select", "expanded"),
        Some(Value::Bool(false))
    );
    assert_eq!(g.focused().as_deref(), Some("select_page.select"));
}

/// The sliders: right and up raise, left and down lower, Home and End go
/// to the ends; the stepped one by its step, the continuous one by a
/// hundredth of its range.
#[test]
fn the_arrows_step_the_sliders_home_and_end_reach_their_ends() {
    let mut g = gallery("Slider");
    let value = |g: &Gallery, id: &str| match g.state_of(id, "value") {
        Some(Value::Num(v)) => v,
        other => panic!("{id}: {other:?}"),
    };
    for id in ["slider_page.slider[0]", "slider_page.slider[1]"] {
        g.tab_to(id);
        let range = g
            .view
            .semantics()
            .and_then(|s| s.get(&sid(id)).and_then(|n| n.constraints.range));
        let (min, max) = range.expect("a range");
        let start = value(&g, id);
        assert!(g.key(EditKey::Right));
        let up = value(&g, id);
        assert!(up > start, "{id}: right raises");
        assert!(g.nav(NavKey::Down));
        assert!(
            (value(&g, id) - start).abs() < 1e-3,
            "{id}: down lowers back"
        );
        assert!(g.nav(NavKey::Up));
        assert!((value(&g, id) - up).abs() < 1e-3, "{id}: up raises");
        assert!(g.key(EditKey::Left));
        assert!(g.key(EditKey::End));
        assert!((value(&g, id) - max).abs() < 1e-3, "{id}: End");
        assert!(g.key(EditKey::Home));
        assert!((value(&g, id) - min).abs() < 1e-3, "{id}: Home");
    }
    let range = g.view.semantics().and_then(|s| {
        s.get(&sid("slider_page.slider[0]"))
            .map(|n| n.constraints.range)
    });
    let (min, max) = range.flatten().expect("a range");
    g.tab_to("slider_page.slider[0]");
    g.key(EditKey::Right);
    let nudge = value(&g, "slider_page.slider[0]") - min;
    assert!((nudge - (max - min) / 100.0).abs() < 1e-3, "{nudge}");
}

/// Enter and Space press what has the focus, as its click does: a
/// checkbox, a switch, a filter chip toggle; a link moves the active
/// mark; a crumb is echoed; a button of the Toast page raises its toast.
#[test]
fn enter_and_space_press_what_has_the_focus_as_its_click_does() {
    for (page, id, key) in [
        ("Checkbox", "checkbox_page.box[1]", "checked"),
        ("Switch", "switch_page.switch[1]", "on"),
        ("Chip", "chip_page.chip[5]", "selected"),
    ] {
        let mut clicked = gallery(page);
        clicked.click(id);
        let mut g = gallery(page);
        let before = g.state_of(id, key);
        g.tab_to(id);
        assert!(g.space(), "{page}: Space");
        assert_ne!(g.state_of(id, key), before, "{page}: Space toggles");
        assert_eq!(g.state(), clicked.state(), "{page}: as the click does");
        assert!(g.enter(), "{page}: Enter");
        assert_eq!(g.state_of(id, key), before, "{page}: Enter toggles back");
    }
    let mut g = gallery("Navigation Menu");
    g.tab_to("navigation_menu_page.link[2]");
    assert!(g.enter());
    assert_eq!(
        g.state_of("navigation_menu_page.link[2]", "active"),
        Some(Value::Bool(true))
    );
    let mut g = gallery("Breadcrumb");
    g.tab_to("breadcrumb_page.trail[1]");
    assert!(g.enter());
    let said = |g: &Gallery, words: &str| g.state().iter().any(|n| n.label.contains(words));
    assert!(said(&g, "clicked crumb 1"), "the crumb is echoed");
    let mut g = gallery("Toast");
    g.tab_to("toast_page.button[success]");
    assert!(g.space());
    assert!(said(&g, "Saved! Everything went fine."), "the toast");
    assert_eq!(
        g.focused().as_deref(),
        Some("toast_page.button[success]"),
        "a toast never takes the focus"
    );
}

/// The modal the keyboard opens takes the focus on its cancel (the ring
/// shows); Tab and Shift-Tab stay inside it; Enter answers it and the
/// focus is back on the button that opened it; Escape closes it the same
/// way.
#[test]
fn the_modal_traps_the_focus_and_gives_it_back_when_it_closes() {
    let opener = "dialog_page.delete";
    let mut g = gallery("Dialog");
    g.tab_to(opener);
    assert!(g.enter(), "opens the modal");
    let said = |g: &Gallery, words: &str| g.state().iter().any(|n| n.label == words);
    assert!(said(&g, "Delete repository?"));
    assert_eq!(g.focused().as_deref(), Some("dialog[cancel]"));
    assert!(g.ringed("dialog[cancel]"), "the ring on the cancel");
    for at in ["dialog[confirm]", "dialog[cancel]", "dialog[confirm]"] {
        assert!(g.tab());
        assert_eq!(g.focused().as_deref(), Some(at), "Tab stays inside");
        assert!(g.ringed(at));
    }
    assert!(g.back_tab());
    assert_eq!(g.focused().as_deref(), Some("dialog[cancel]"), "and back");
    assert!(g.space(), "Space presses the cancel");
    assert!(!said(&g, "Delete repository?"), "the cancel answered it");
    assert_eq!(g.focused().as_deref(), Some(opener), "the focus is back");
    assert!(g.ringed(opener));

    assert!(g.enter(), "opens it again");
    assert_eq!(g.focused().as_deref(), Some("dialog[cancel]"));
    assert!(g.view.close_top_overlay(), "Escape closes the modal first");
    g.frame();
    assert!(!said(&g, "Delete repository?"));
    assert_eq!(g.focused().as_deref(), Some(opener), "back on the opener");
}
