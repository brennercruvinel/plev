//! Toggles and values: the same flip, value, text and option from the
//! three.

use accord::Value;
use engine::control::accessibility::accesskit::{Action, ActionData};

use super::harness::{ask, ask_with, by_agent, gallery, same};

/// A checkbox, a switch and a filter chip flip from the three.
#[test]
fn a_toggle_flips_the_same_from_the_three() {
    for (page, id) in [
        ("Checkbox", "checkbox_page.box[1]"),
        ("Switch", "switch_page.switch[1]"),
        ("Chip", "chip_page.chip[5]"),
    ] {
        let mut person = gallery(page);
        person.click(id);
        let mut reader = gallery(page);
        reader.press(id);
        let agent = by_agent(page, "toggle", vec![ask(id, "toggle")]);
        same(page, &person, &reader, &agent);
    }
}

/// agent's `increment` move it the same step.
#[test]
fn a_slider_moves_the_same_from_the_three() {
    let id = "slider_page.slider[1]";
    let mut person = gallery("Slider");
    person.click(id);
    let Some(Value::Num(v)) = person.value("slider_page.slider[1]:value") else {
        panic!("the slider has a value");
    };
    let mut reader = gallery("Slider");
    reader.reader(id, Action::SetValue, Some(ActionData::NumericValue(v)));
    let agent = by_agent(
        "Slider",
        "slider",
        vec![ask_with(id, "set_value", Value::Num(v))],
    );
    same("Slider", &person, &reader, &agent);

    let mut reader = gallery("Slider");
    reader.reader(id, Action::Increment, None);
    let agent = by_agent("Slider", "step", vec![ask(id, "increment")]);
    assert_eq!(agent.state(), reader.state());
    assert_eq!(
        reader.value("slider_page.slider[1]:value"),
        Some(Value::Num(5.0))
    );
}

/// `set_value` put the same text, and the preview line says it.
#[test]
fn a_field_takes_the_same_text_from_the_three() {
    let id = "input_page.field[0]";
    let mut person = gallery("Input");
    person.click(id);
    person.type_in("Ada");
    let mut reader = gallery("Input");
    reader.reader(id, Action::SetValue, Some(ActionData::Value("Ada".into())));
    let agent = by_agent(
        "Input",
        "field",
        vec![ask_with(id, "set_value", Value::from("Ada"))],
    );
    same("Input", &person, &reader, &agent);
    assert!(person.state().iter().any(|n| n.label == "live value: Ada"));
}

/// option. The same option, the list shut.
#[test]
fn a_select_picks_the_same_option_from_the_three() {
    let (select, option) = ("select_page.select", "select_page.select[2]");
    let mut person = gallery("Select");
    person.click(select);
    person.click(option);
    let mut reader = gallery("Select");
    reader.press(select);
    reader.press(option);
    let agent = by_agent(
        "Select",
        "select",
        vec![ask_with(select, "select", Value::from("Always light"))],
    );
    same("Select", &person, &reader, &agent);
    assert_eq!(
        person.value("select_page.select:selected"),
        Some(Value::from("Always light"))
    );
    assert_eq!(
        person.value("select_page.select:expanded"),
        Some(Value::Bool(false))
    );
}
