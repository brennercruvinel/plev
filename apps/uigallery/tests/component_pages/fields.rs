//! Front A, fields (task-104, #154): field, label, input group, input OTP,
//! textarea, combobox and command, the same request from the person, the
//! reader and an agent.

use accord::Value;
use engine::control::accessibility::accesskit::{Action, ActionData};

use super::harness::{Gallery, ask, ask_with, by_agent, gallery, same, sid};

const EMAIL: &str = "field_page.group[0].card[0]";
const PASSWORD: &str = "field_page.group[1].card[0]";
const EYE: &str = "field_page.group[1].card[0].reveal";

/// The live email field: a person's typing, a reader's value and an
/// agent's `set_value` put the same text.
#[test]
fn a_field_takes_the_same_text_from_the_three() {
    let mut person = gallery("Field");
    person.click(EMAIL);
    person.type_in("ada@hoff.dev");
    let mut reader = gallery("Field");
    reader.reader(
        EMAIL,
        Action::SetValue,
        Some(ActionData::Value("ada@hoff.dev".into())),
    );
    let agent = by_agent(
        "Field",
        "field",
        vec![ask_with(EMAIL, "set_value", Value::from("ada@hoff.dev"))],
    );
    same("Field", &person, &reader, &agent);
    assert_eq!(
        person.value("field_page.group[0].card[0]:text"),
        Some(Value::from("ada@hoff.dev"))
    );
}

/// The password: the same secret from the three, and none of them sees
/// more of it than its length.
#[test]
pub(super) fn a_password_takes_the_same_secret_from_the_three_and_shows_its_length_only() {
    let mut person = gallery("Field");
    person.click(PASSWORD);
    person.type_in("hunter2");
    let mut reader = gallery("Field");
    reader.reader(
        PASSWORD,
        Action::SetValue,
        Some(ActionData::Value("hunter2".into())),
    );
    let agent = by_agent(
        "Field",
        "password",
        vec![ask_with(PASSWORD, "set_value", Value::from("hunter2"))],
    );
    same("Field", &person, &reader, &agent);
    assert_eq!(
        person.value("field_page.group[1].card[0]:text"),
        Some(Value::Int(7))
    );
    for g in [&person, &reader, &agent] {
        assert!(!format!("{:?}", g.state()).contains("hunter2"));
    }
}

/// The eye: a click, a reader's press and an agent's `toggle` reveal the
/// same password.
#[test]
fn the_eye_reveals_the_password_the_same_from_the_three() {
    let mut person = gallery("Field");
    person.click(EYE);
    let mut reader = gallery("Field");
    reader.press(EYE);
    let agent = by_agent("Field", "eye", vec![ask(EYE, "toggle")]);
    same("Field", &person, &reader, &agent);
    assert_eq!(
        person.value("field_page.group[1].card[0].reveal:on"),
        Some(Value::Bool(true))
    );
}

const CUT: &str = "context_menu[101]";
const COPY: &str = "context_menu[102]";
const SELECT_ALL: &str = "context_menu[104]";

fn enabled(g: &Gallery, id: &str) -> Option<bool> {
    g.view.semantics()?.get(&sid(id)).map(|n| n.enabled)
}

fn right_click(g: &mut Gallery, id: &str) -> bool {
    let (x, y) = g.middle(id);
    let opened = g.view.handle_right_click(x, y);
    g.frame();
    opened
}

/// A password's menu offers neither Cut nor Copy, even over a selection;
/// the email's offers both.
#[test]
fn a_password_menu_offers_no_cut_and_no_copy() {
    for (id, offers) in [(EMAIL, true), (PASSWORD, false)] {
        let mut g = gallery("Field");
        g.click(id);
        g.type_in("hunter2");
        assert!(right_click(&mut g, id), "{id}: the menu opens");
        g.click(SELECT_ALL);
        assert!(right_click(&mut g, id), "{id}: again, over the selection");
        assert_eq!(enabled(&g, COPY), Some(offers), "{id}: Copy");
        assert_eq!(enabled(&g, CUT), Some(offers), "{id}: Cut");
    }
}

const LABELLED: &str = "label_page.group[1].card[0]";

/// The Label page's field, named by its label: the same text from the
/// three.
#[test]
fn a_labelled_field_takes_the_same_text_from_the_three() {
    let mut person = gallery("Label");
    person.click(LABELLED);
    person.type_in("ada@hoff.dev");
    let mut reader = gallery("Label");
    reader.reader(
        LABELLED,
        Action::SetValue,
        Some(ActionData::Value("ada@hoff.dev".into())),
    );
    let agent = by_agent(
        "Label",
        "label",
        vec![ask_with(LABELLED, "set_value", Value::from("ada@hoff.dev"))],
    );
    same("Label", &person, &reader, &agent);
}

const CODE: &str = "input_otp_page.group[0].card[0]";

/// The one-time code: a person's typing (a letter in it dropped), a
/// reader's value and an agent's `set_value` put the same digits, and
/// none of them sees more of the code than its length.
#[test]
pub(super) fn a_code_takes_the_same_digits_from_the_three_and_shows_its_length_only() {
    let mut person = gallery("Input OTP");
    person.click(CODE);
    person.type_in("4829x13");
    let mut reader = gallery("Input OTP");
    reader.reader(
        CODE,
        Action::SetValue,
        Some(ActionData::Value("482913".into())),
    );
    let agent = by_agent(
        "Input OTP",
        "code",
        vec![ask_with(CODE, "set_value", Value::from("482913"))],
    );
    same("Input OTP", &person, &reader, &agent);
    assert_eq!(person.value(&format!("{CODE}:text")), Some(Value::Int(6)));
    for g in [&person, &reader, &agent] {
        assert!(!format!("{:?}", g.state()).contains("482913"));
    }
}

const AREA: &str = "textarea_page.group[0].card[0]";

/// The text area: a person's typing with Enter between the lines, a
/// reader's value and an agent's `set_value` put the same two lines.
#[test]
fn a_text_area_takes_the_same_lines_from_the_three() {
    let mut person = gallery("Textarea");
    person.click(AREA);
    person.type_in("first");
    person.view.handle_enter();
    person.type_in("second");
    let mut reader = gallery("Textarea");
    reader.reader(
        AREA,
        Action::SetValue,
        Some(ActionData::Value("first\nsecond".into())),
    );
    let agent = by_agent(
        "Textarea",
        "area",
        vec![ask_with(AREA, "set_value", Value::from("first\nsecond"))],
    );
    same("Textarea", &person, &reader, &agent);
    assert_eq!(
        person.value(&format!("{AREA}:text")),
        Some(Value::from("first\nsecond"))
    );
}

const COMBO: &str = "combobox_page.group[0].card[0]";

/// The combobox: a person's filter and Enter, a reader's press on the
/// box then on the option, and an agent's `select` choose the same
/// option, the list shut and the box showing it.
#[test]
fn a_combobox_picks_the_same_option_from_the_three() {
    let mut person = gallery("Combobox");
    person.click(COMBO);
    person.type_in("rem");
    person.view.handle_enter();
    person.frame();
    let mut reader = gallery("Combobox");
    reader.press(COMBO);
    reader.press("combobox_page.group[0].card[0][3]");
    let agent = by_agent(
        "Combobox",
        "combobox",
        vec![ask_with(COMBO, "select", Value::from("Remix"))],
    );
    same("Combobox", &person, &reader, &agent);
    assert_eq!(
        person.value(&format!("{COMBO}:selected")),
        Some(Value::from("Remix"))
    );
    assert_eq!(
        person.value(&format!("{COMBO}:query")),
        Some(Value::from("Remix"))
    );
}

const INVITE: &str = "input_group_page.group[1].card[0].choice";

/// The Invite's select: a person's presses, a reader's presses and an
/// agent's `select` give the same access, the list shut.
#[test]
fn the_invite_s_select_picks_the_same_access_from_the_three() {
    let option = "input_group_page.group[1].card[0].choice[1]";
    let mut person = gallery("Input Group");
    person.click(INVITE);
    person.click(option);
    let mut reader = gallery("Input Group");
    reader.press(INVITE);
    reader.press(option);
    let agent = by_agent(
        "Input Group",
        "invite",
        vec![ask_with(INVITE, "select", Value::from("can edit"))],
    );
    same("Input Group", &person, &reader, &agent);
    assert_eq!(
        person.value(&format!("{INVITE}:selected")),
        Some(Value::from("can edit"))
    );
}
