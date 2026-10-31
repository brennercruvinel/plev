//! Front B, overlays (task-104, #155): popover, hover card, dropdown menu,
//! alert dialog, menubar, sheet and drawer, the same request from the
//! person, the reader and an agent.

use accord::{Actor, Request, Value};
use engine::drawing::compositor::{Compositor, LayerId};
use engine::widgets::prelude::WidgetEvent;
use uigallery::views::{EditKey, ShowcaseView};

use super::harness::{Gallery, ask, by_agent, gallery, same, sid};

const PAGE: &str = "Popover";
/// The live card's trigger, its panel, and its "Zoom to 200%" preset.
const LIVE: &str = "popover_page.group[0].card[0]";
const PANEL: &str = "popover_page.group[0].card[0].panel";
const TO_200: &str = "popover_page.group[0].card[0].panel[3]";

fn expanded(g: &Gallery) -> Option<Value> {
    g.value(&format!("{LIVE}:expanded"))
}

fn label(g: &Gallery, id: &str) -> Option<String> {
    g.state()
        .into_iter()
        .find(|n| n.id == sid(id))
        .map(|n| n.label)
}

fn focused(g: &Gallery) -> Option<String> {
    g.view
        .semantics()
        .and_then(|s| s.focused().map(ToString::to_string))
}

/// The popover opens from the three: a click, a reader's press, an
/// agent's `activate`; its panel is projected, the trigger expanded.
#[test]
fn the_popover_opens_the_same_from_the_person_the_reader_and_an_agent() {
    let mut person = gallery(PAGE);
    assert_eq!(
        expanded(&person),
        Some(Value::Bool(false)),
        "it loads closed"
    );
    person.click(LIVE);
    let mut reader = gallery(PAGE);
    reader.press(LIVE);
    let agent = by_agent(PAGE, "popover-open", vec![ask(LIVE, "activate")]);
    same(PAGE, &person, &reader, &agent);
    assert_eq!(expanded(&agent), Some(Value::Bool(true)));
    assert_eq!(label(&agent, PANEL).as_deref(), Some("Zoom"));
}

/// A preset picked in the panel, from the three: the zoom the value
/// pill shows changes and the popover closes.
#[test]
fn a_preset_picked_from_the_three_sets_the_zoom_and_closes_the_panel() {
    let mut person = gallery(PAGE);
    person.click(LIVE);
    person.click(TO_200);
    let mut reader = gallery(PAGE);
    reader.press(LIVE);
    reader.press(TO_200);
    let asks = vec![ask(LIVE, "activate"), ask(TO_200, "activate")];
    let agent = by_agent(PAGE, "popover-pick", asks);
    same(PAGE, &person, &reader, &agent);
    assert_eq!(label(&agent, LIVE).as_deref(), Some("200%"));
    assert_eq!(expanded(&agent), Some(Value::Bool(false)));
    assert_eq!(label(&agent, PANEL), None, "the panel is gone");
}

/// Closed again from the three: Escape, a second press of the trigger,
/// the panel's `close`; the page is as it loaded.
#[test]
fn the_popover_closes_the_same_from_the_three() {
    let before = gallery(PAGE).state();
    let mut person = gallery(PAGE);
    person.click(LIVE);
    assert!(person.view.close_top_overlay(), "Escape closes it");
    person.frame();
    let mut reader = gallery(PAGE);
    reader.press(LIVE);
    reader.press(LIVE);
    let asks = vec![ask(LIVE, "activate"), ask(PANEL, "close")];
    let agent = by_agent(PAGE, "popover-close", asks);
    for (who, g) in [("person", &person), ("reader", &reader), ("agent", &agent)] {
        assert_eq!(g.state(), before, "{who}");
    }
}

/// The keyboard: the trigger focused, Space opens and the focus stays on
/// it (headlessui's popover); Tab walks into the panel; Escape closes it
/// and gives the focus back to the trigger; a second Escape lets it go.
#[test]
fn space_opens_tab_enters_and_escape_gives_the_focus_back() {
    let mut g = gallery(PAGE);
    assert!(
        g.view.assist_focus(&sid(LIVE)),
        "a reader moves the focus there"
    );
    g.frame();
    assert_eq!(focused(&g).as_deref(), Some(LIVE));
    assert!(g.view.handle_key(" "));
    g.frame();
    assert_eq!(expanded(&g), Some(Value::Bool(true)));
    assert_eq!(
        focused(&g).as_deref(),
        Some(LIVE),
        "opening keeps the focus"
    );
    assert!(g.view.handle_edit_key(EditKey::Tab));
    g.frame();
    let first = format!("{PANEL}[0]");
    assert_eq!(
        focused(&g).as_deref(),
        Some(first.as_str()),
        "Tab enters the panel"
    );
    assert!(g.view.close_top_overlay());
    g.frame();
    assert_eq!(expanded(&g), Some(Value::Bool(false)));
    assert_eq!(focused(&g).as_deref(), Some(LIVE), "back on the trigger");
    assert!(g.view.close_top_overlay());
    assert_eq!(focused(&g), None);
}

/// What the page refuses: a closed panel is not there to close or pick
/// from, and a forced card shows its state but does not open.
#[test]
fn a_closed_panel_and_a_forced_card_refuse_what_they_cannot_do() {
    let g = gallery(PAGE);
    let seen = g.view.semantics().expect("the page projects");
    let actor = Actor::Test {
        name: "popover".to_owned(),
    };
    for (target, action) in [(PANEL, "close"), (TO_200, "activate")] {
        let req = Request::observe(&seen, actor.clone(), &sid(target), action, None);
        assert!(req.is_err(), "{target} {action} while closed");
    }
    let forced = sid("popover_page.group[1].card[0]");
    let node = seen.get(&forced).expect("the forced trigger");
    assert_eq!(node.state.get("expanded"), Some(&Value::Bool(true)));
    let req = Request::observe(&seen, actor, &forced, "close", None);
    assert!(req.is_err(), "a forced card has nothing to close");
}

/// The page in a window of its own: the live popover opens there, inside
/// that window's page, on a layer above it; a frame later it has faded in.
#[test]
fn the_popover_opens_inside_a_detached_window() {
    let (w, h) = (700.0, 600.0);
    let mut view = ShowcaseView::new(1280.0, 800.0);
    view.jump_to_page(PAGE);
    assert!(view.detach(w, h, 1.0));
    let mut c = Compositor::new();
    view.render_detached(&mut c);
    let [x, y, bw, bh] = c
        .binding(&sid(LIVE))
        .expect("bound in the page window")
        .rect;
    let (x, y) = (x + bw / 2.0, y + bh / 2.0);
    assert!(view.detached_event(&WidgetEvent::MouseDown { x, y }));
    view.detached_event(&WidgetEvent::MouseUp { x, y });
    for _ in 0..120 {
        if !view.tick(1.0 / 60.0) {
            break;
        }
    }
    view.render_detached(&mut c);
    for i in 0..5 {
        let preset = sid(&format!("{PANEL}[{i}]"));
        let bound = c.binding(&preset).expect("the panel in the page window");
        let [px, py, pw, ph] = bound.rect;
        assert!(
            px >= 0.0 && py >= 0.0 && px + pw <= w && py + ph <= h,
            "{preset}: {:?}",
            bound.rect
        );
        assert_ne!(bound.layer, LayerId::DEFAULT, "above the page");
    }
}

// Hover Card

const HOVER: &str = "Hover Card";
const HANDLE: &str = "hover_card_page.group[0].card[0]";
const CARD: &str = "hover_card_page.group[0].card[0].panel";

/// The view's frames for `seconds`, as the window loop runs them.
fn wait(g: &mut Gallery, seconds: f32) {
    for _ in 0..(seconds * 60.0) as usize {
        g.view.tick(1.0 / 60.0);
    }
    g.frame();
}

/// The card opens from the three: the person's pointer resting on the
/// handle past the delay, a reader's press, an agent's `activate`.
#[test]
fn the_hover_card_opens_the_same_from_the_person_the_reader_and_an_agent() {
    let mut person = gallery(HOVER);
    let (x, y) = person.middle(HANDLE);
    person.view.handle_event(&WidgetEvent::MouseMove { x, y });
    wait(&mut person, 0.2);
    assert_eq!(label(&person, CARD), None, "still waiting out the delay");
    wait(&mut person, 0.5);
    let mut reader = gallery(HOVER);
    reader.press(HANDLE);
    let agent = by_agent(HOVER, "hover-open", vec![ask(HANDLE, "activate")]);
    same(HOVER, &person, &reader, &agent);
    assert_eq!(label(&agent, CARD).as_deref(), Some("Brenner Cruvinel"));
}

/// Closed again from the three: the pointer leaving handle and card past
/// the close delay, Escape, the card's `close`.
#[test]
fn the_hover_card_closes_the_same_from_the_three() {
    let before = gallery(HOVER).state();
    let mut person = gallery(HOVER);
    let (x, y) = person.middle(HANDLE);
    person.view.handle_event(&WidgetEvent::MouseMove { x, y });
    wait(&mut person, 0.7);
    // Off to the side, still on the page: off the handle and the card.
    let away = WidgetEvent::MouseMove { x: x - 420.0, y };
    person.view.handle_event(&away);
    wait(&mut person, 0.7);
    let mut reader = gallery(HOVER);
    reader.press(HANDLE);
    assert!(reader.view.close_top_overlay(), "Escape closes it");
    reader.frame();
    let asks = vec![ask(HANDLE, "activate"), ask(CARD, "close")];
    let agent = by_agent(HOVER, "hover-close", asks);
    for (who, g) in [("person", &person), ("reader", &reader), ("agent", &agent)] {
        assert_eq!(g.state(), before, "{who}");
    }
}

// Dropdown Menu

const DROPDOWN: &str = "Dropdown Menu";
const MENU_TRIGGER: &str = "dropdown_menu_page.group[0].card[0]";
const MENU: &str = "dropdown_menu_page.group[0].card[0].panel";
const SETTINGS: &str = "dropdown_menu_page.group[0].card[0].panel[4]";

/// The menu opens from the three, and an item picked from the three
/// closes it and says what was picked.
#[test]
fn the_dropdown_menu_opens_and_picks_the_same_from_the_three() {
    let mut person = gallery(DROPDOWN);
    person.click(MENU_TRIGGER);
    let mut reader = gallery(DROPDOWN);
    reader.press(MENU_TRIGGER);
    let agent = by_agent(DROPDOWN, "menu-open", vec![ask(MENU_TRIGGER, "activate")]);
    same(DROPDOWN, &person, &reader, &agent);
    assert_eq!(label(&agent, MENU).as_deref(), Some("profile menu"));

    person.click(SETTINGS);
    reader.press(SETTINGS);
    let asks = vec![ask(MENU_TRIGGER, "activate"), ask(SETTINGS, "activate")];
    let agent = by_agent(DROPDOWN, "menu-pick", asks);
    same(DROPDOWN, &person, &reader, &agent);
    let picked = "dropdown_menu_page.picked";
    assert_eq!(label(&agent, picked).as_deref(), Some("Settings"));
    assert_eq!(label(&agent, MENU), None, "the menu closed");
}

// Alert Dialog

const ALERT: &str = "Alert Dialog";
const DELETE: &str = "alert_dialog_page.group[0].card[0]";

/// The dialog opens from the three and blocks the page; its cancel,
/// from the three, leaves the page as it was.
#[test]
fn the_alert_dialog_opens_and_cancels_the_same_from_the_three() {
    let mut person = gallery(ALERT);
    person.click(DELETE);
    let mut reader = gallery(ALERT);
    reader.press(DELETE);
    let agent = by_agent(ALERT, "alert-open", vec![ask(DELETE, "activate")]);
    same(ALERT, &person, &reader, &agent);
    assert_eq!(
        label(&agent, "dialog").as_deref(),
        Some("Delete this file?")
    );

    let before = gallery(ALERT).state();
    person.click("dialog[cancel]");
    reader.press("dialog[cancel]");
    let asks = vec![ask(DELETE, "activate"), ask("dialog", "cancel")];
    let agent = by_agent(ALERT, "alert-cancel", asks);
    for g in [&mut person, &mut reader] {
        for _ in 0..120 {
            if !g.view.tick(1.0 / 60.0) {
                break;
            }
        }
        g.frame();
    }
    for (who, g) in [("person", &person), ("reader", &reader), ("agent", &agent)] {
        assert_eq!(label(g, "dialog"), None, "{who}: answered");
        assert_eq!(g.state(), before, "{who}");
    }
}

// Menubar

const MENUBAR: &str = "Menubar";
const EDIT: &str = "menubar_page.bar[1]";
const COPY: &str = "menubar_page.bar[1].panel[3]";

/// A menu of the bar opens from the three, and an item picked from the
/// three closes it and says what was picked.
#[test]
fn the_menubar_opens_and_picks_the_same_from_the_three() {
    let mut person = gallery(MENUBAR);
    person.click(EDIT);
    let mut reader = gallery(MENUBAR);
    reader.press(EDIT);
    let agent = by_agent(MENUBAR, "bar-open", vec![ask(EDIT, "activate")]);
    same(MENUBAR, &person, &reader, &agent);

    person.click(COPY);
    reader.press(COPY);
    let agent = by_agent(
        MENUBAR,
        "bar-pick",
        vec![ask(EDIT, "activate"), ask(COPY, "activate")],
    );
    same(MENUBAR, &person, &reader, &agent);
    assert_eq!(
        label(&agent, "menubar_page.picked").as_deref(),
        Some("Copy")
    );
}

// Sheet and Drawer

/// The edge panel slid in (or out): the overlay stack settled, drawn.
fn settle(g: &mut Gallery) {
    for _ in 0..240 {
        if !g.view.tick(1.0 / 60.0) {
            break;
        }
    }
    g.frame();
}

const SHEET: &str = "Sheet";
const FROM_BOTTOM: &str = "sheet_page.group[0].card[2]";

/// The sheet opens from the three, held to the edge its button names,
/// with the focus on its close; its save, from the three, closes it and
/// says so in a toast.
#[test]
fn the_sheet_opens_and_saves_the_same_from_the_three() {
    let mut person = gallery(SHEET);
    person.click(FROM_BOTTOM);
    settle(&mut person);
    let mut reader = gallery(SHEET);
    reader.press(FROM_BOTTOM);
    settle(&mut reader);
    let mut agent = by_agent(SHEET, "sheet-open", vec![ask(FROM_BOTTOM, "activate")]);
    settle(&mut agent);
    same(SHEET, &person, &reader, &agent);
    assert_eq!(label(&agent, "sheet").as_deref(), Some("Profile"));
    assert_eq!(
        agent.value("sheet:edge"),
        Some(Value::Text("bottom".into()))
    );
    assert_eq!(focused(&agent).as_deref(), Some("sheet[close]"));

    person.click("sheet[confirm]");
    reader.press("sheet[confirm]");
    let asks = vec![ask(FROM_BOTTOM, "activate"), ask("sheet", "confirm")];
    let mut agent = by_agent(SHEET, "sheet-save", asks);
    for g in [&mut person, &mut reader, &mut agent] {
        settle(g);
        assert_eq!(label(g, "sheet"), None, "answered");
    }
    same(SHEET, &person, &reader, &agent);
}

/// The keyboard: Tab never leaves the open sheet (the trap), Escape
/// closes it and the focus is the button that opened it again.
#[test]
fn tab_stays_in_the_sheet_and_escape_gives_the_focus_back() {
    let mut g = gallery(SHEET);
    assert!(g.view.assist_focus(&sid(FROM_BOTTOM)));
    assert!(g.view.handle_key(" "));
    settle(&mut g);
    let mut seen = Vec::new();
    for _ in 0..4 {
        assert!(g.view.handle_edit_key(EditKey::Tab));
        g.frame();
        seen.push(focused(&g).unwrap_or_default());
    }
    assert_eq!(
        seen,
        [
            "sheet[cancel]",
            "sheet[confirm]",
            "sheet[close]",
            "sheet[cancel]"
        ]
    );
    assert!(g.view.close_top_overlay());
    settle(&mut g);
    assert_eq!(label(&g, "sheet"), None);
    assert_eq!(focused(&g).as_deref(), Some(FROM_BOTTOM));
}

const DRAWER: &str = "Drawer";
const OPEN_NAV: &str = "drawer_page.group[0].card[0]";

/// The drawer opens from the three with the focus on its current link;
/// a link picked from the three closes it and names the link in a toast.
#[test]
fn the_drawer_opens_and_picks_the_same_from_the_three() {
    let mut person = gallery(DRAWER);
    person.click(OPEN_NAV);
    settle(&mut person);
    let mut reader = gallery(DRAWER);
    reader.press(OPEN_NAV);
    settle(&mut reader);
    let mut agent = by_agent(DRAWER, "drawer-open", vec![ask(OPEN_NAV, "activate")]);
    settle(&mut agent);
    same(DRAWER, &person, &reader, &agent);
    assert_eq!(
        focused(&agent).as_deref(),
        Some("drawer[1]"),
        "the current link"
    );
    assert_eq!(agent.value("drawer[1]:current"), Some(Value::Bool(true)));

    person.click("drawer[3]");
    reader.press("drawer[3]");
    let asks = vec![ask(OPEN_NAV, "activate"), ask("drawer[3]", "activate")];
    let mut agent = by_agent(DRAWER, "drawer-pick", asks);
    for g in [&mut person, &mut reader, &mut agent] {
        settle(g);
        assert_eq!(label(g, "drawer"), None, "closed by the pick");
    }
    same(DRAWER, &person, &reader, &agent);
}

/// A press on the scrim closes the drawer, as the close does for a
/// reader and an agent: the page is as it loaded.
#[test]
fn the_drawer_closes_on_the_scrim_and_by_its_close() {
    let before = gallery(DRAWER).state();
    let mut person = gallery(DRAWER);
    person.click(OPEN_NAV);
    settle(&mut person);
    let (w, h) = (person.view.width, person.view.height);
    person.view.handle_event(&WidgetEvent::MouseDown {
        x: w - 4.0,
        y: h / 2.0,
    });
    person.view.handle_event(&WidgetEvent::MouseUp {
        x: w - 4.0,
        y: h / 2.0,
    });
    let mut reader = gallery(DRAWER);
    reader.press(OPEN_NAV);
    settle(&mut reader);
    reader.press("drawer[close]");
    let asks = vec![ask(OPEN_NAV, "activate"), ask("drawer", "close")];
    let mut agent = by_agent(DRAWER, "drawer-close", asks);
    for (who, g) in [
        ("person", &mut person),
        ("reader", &mut reader),
        ("agent", &mut agent),
    ] {
        settle(g);
        assert_eq!(g.state(), before, "{who}");
    }
}
