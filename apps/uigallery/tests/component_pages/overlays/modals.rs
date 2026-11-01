//! The Modals tab (task-104 front B, onda 3): a variation chosen, a
//! section of Settings and of Export picked, a switch toggled and a select
//! set, the same from the person, the reader and an agent; the keyboard's
//! Tab, arrows and Escape over the same controls.

use accord::Value;
use engine::control::accessibility::accesskit::{Action, ActionData};
use uigallery::views::EditKey;

use super::super::harness::{Gallery, ask, ask_with, by_agent, gallery, same, sid};

const PAGE: &str = "Modals";
const TILE: &str = "modals_page.group[0].card[0].tile[2]";
const TILES: &str = "modals_page.group[0].card[0].tile";
const NOTIFICATIONS: &str = "modals_page.group[5].card[0].section[3]";
const GENERAL: &str = "modals_page.group[5].card[0].section[0]";
const PROFILE: &str = "modals_page.group[5].card[0].section[1]";
const SECTIONS: &str = "modals_page.group[5].card[0].section";
const OBJECT: &str = "modals_page.group[5].card[0].notifications.object";
const VIDEO: &str = "modals_page.group[4].card[0].section[1]";
const RATE: &str = "modals_page.group[4].card[0].video.rate";
const RATE_30: &str = "modals_page.group[4].card[0].video.rate[1]";
const INVITE_CAN: &str = "modals_page.group[2].card[0].invite.choice";
const USERNAME: &str = "modals_page.group[5].card[0].general.username";

/// The person, and how far they scrolled the page.
struct Person {
    g: Gallery,
    scrolled: f32,
}

impl Person {
    fn new() -> Self {
        Self {
            g: gallery(PAGE),
            scrolled: 0.0,
        }
    }

    /// Scrolls so `id` shows, then clicks it.
    fn click(&mut self, id: &str) {
        let bound = self.g.c.binding(&sid(id)).map(|b| b.rect);
        let [_, y, _, h] = bound.unwrap_or_else(|| panic!("{id} is not bound"));
        if y < 0.0 || y + h > 700.0 {
            self.scrolled = (self.scrolled + y - 200.0).max(0.0);
            self.g.view.scroll_page_to(self.scrolled);
            self.g.frame();
            self.g.frame();
        }
        self.g.click(id);
    }
}

fn focused(g: &Gallery) -> Option<String> {
    g.view
        .semantics()
        .and_then(|s| s.focused().map(ToString::to_string))
}

/// A variation chosen from the three: a click on the tile, a reader's
/// press, an agent's `select`; the count reads it.
#[test]
fn a_variation_is_chosen_the_same_from_the_three() {
    let mut person = Person::new();
    person.click(TILE);
    let person = person.g;
    let mut reader = gallery(PAGE);
    reader.press(TILE);
    let agent = by_agent(PAGE, "modals-tile", vec![ask(TILE, "select")]);
    same(PAGE, &person, &reader, &agent);
    let selected = agent.value(&format!("{TILES}:selected"));
    assert_eq!(selected, Some(Value::Text("2".into())));
}

/// A section of Settings picked, then one of its switches toggled, from
/// the three.
#[test]
fn a_settings_section_and_its_switch_answer_the_three() {
    let mut person = Person::new();
    person.click(NOTIFICATIONS);
    person.click(OBJECT);
    let person = person.g;
    let mut reader = gallery(PAGE);
    reader.press(NOTIFICATIONS);
    reader.press(OBJECT);
    let asks = vec![ask(NOTIFICATIONS, "select"), ask(OBJECT, "toggle")];
    let agent = by_agent(PAGE, "modals-settings", asks);
    same(PAGE, &person, &reader, &agent);
    assert_eq!(
        agent.value(&format!("{OBJECT}:on")),
        Some(Value::Bool(true))
    );
    assert_eq!(
        agent.value(&format!("{SECTIONS}:selected")),
        Some(Value::Text("3".into()))
    );
}

/// Export's video section, its frame rate set to 30: the person opens the
/// select and clicks the option, the reader presses both, the agent asks
/// `select` with the value.
#[test]
fn an_export_select_is_set_the_same_from_the_three() {
    let mut person = Person::new();
    person.click(VIDEO);
    person.click(RATE);
    person.click(RATE_30);
    let person = person.g;
    let mut reader = gallery(PAGE);
    reader.press(VIDEO);
    reader.press(RATE);
    reader.press(RATE_30);
    let asks = vec![
        ask(VIDEO, "select"),
        ask_with(RATE, "select", Value::Text("30".into())),
    ];
    let agent = by_agent(PAGE, "modals-export", asks);
    same(PAGE, &person, &reader, &agent);
    let rate = agent.value(&format!("{RATE}:selected"));
    assert_eq!(rate, Some(Value::Text("30".into())));
    assert_eq!(
        agent.value(&format!("{RATE}:expanded")),
        Some(Value::Bool(false))
    );
}

/// The keyboard over the side menu: a reader's focus on its first tab,
/// the right arrow moves along the tabs, Space picks; Tab walks on to the
/// next control; Escape lets the focus go.
#[test]
fn arrows_walk_the_side_menu_space_picks_and_escape_lets_go() {
    let mut g = gallery(PAGE);
    assert!(g.view.assist_focus(&sid(GENERAL)));
    g.frame();
    assert_eq!(focused(&g).as_deref(), Some(GENERAL));
    assert!(g.view.handle_edit_key(EditKey::Right));
    g.frame();
    assert_eq!(focused(&g).as_deref(), Some(PROFILE), "the arrow moves");
    assert!(g.view.handle_key(" "));
    g.frame();
    let selected = g.value(&format!("{SECTIONS}:selected"));
    assert_eq!(selected, Some(Value::Text("1".into())), "Space picks");
    assert!(g.view.handle_edit_key(EditKey::Tab));
    g.frame();
    let next = focused(&g).unwrap_or_default();
    assert_ne!(next, PROFILE, "Tab walks on");
    assert!(next.starts_with("modals_page."), "{next} is the page's");
    assert!(g.view.close_top_overlay(), "Escape lets the focus go");
    g.frame();
    assert_eq!(focused(&g), None);
}

/// Escape over an open select: the first closes the list and keeps the
/// focus, the second lets the focus go.
#[test]
fn escape_closes_an_open_select_first() {
    let mut g = gallery(PAGE);
    assert!(g.view.assist_focus(&sid(INVITE_CAN)));
    g.frame();
    assert!(g.view.handle_key(" "), "Space opens it");
    g.frame();
    let expanded = |g: &Gallery| g.value(&format!("{INVITE_CAN}:expanded"));
    assert_eq!(expanded(&g), Some(Value::Bool(true)));
    assert!(g.view.close_top_overlay());
    g.frame();
    assert_eq!(expanded(&g), Some(Value::Bool(false)), "the list closes");
    assert!(focused(&g).is_some(), "the focus stays");
    assert!(g.view.close_top_overlay());
    g.frame();
    assert_eq!(focused(&g), None);
}

/// The username of Settings edited in place: the person presses it,
/// types and presses Enter; a reader sets the value; an agent asks
/// `set_value`. The three save the same, the box gone.
#[test]
fn a_settings_value_is_typed_and_saved_the_same_from_the_three() {
    let mut person = Person::new();
    person.click(USERNAME);
    person.g.type_in("grace");
    person.g.view.handle_enter();
    person.g.frame();
    let person = person.g;
    let mut reader = gallery(PAGE);
    reader.reader(
        USERNAME,
        Action::SetValue,
        Some(ActionData::Value("grace".into())),
    );
    let ask = ask_with(USERNAME, "set_value", Value::from("grace"));
    let agent = by_agent(PAGE, "modals-username", vec![ask]);
    same(PAGE, &person, &reader, &agent);
    let text = person.value(&format!("{USERNAME}:text"));
    assert_eq!(text, Some(Value::from("grace")));
    let editing = person.value(&format!("{USERNAME}:editing"));
    assert_eq!(editing, Some(Value::Bool(false)));
}
