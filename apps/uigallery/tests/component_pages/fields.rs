//! Front A, fields (task-104, #154): field, label, input group, input OTP,
//! textarea, combobox and command, the same request from the person, the
//! reader and an agent.

use accord::Value;
use engine::control::accessibility::accesskit::{Action, ActionData};

use super::harness::{ask, ask_with, by_agent, gallery, same};

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
fn a_password_takes_the_same_secret_from_the_three_and_shows_its_length_only() {
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
