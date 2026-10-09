//! Buttons: a press from the three, and the focus a reader leaves.

use engine::widgets::prelude::WidgetEvent;

use super::harness::{Gallery, ask, by_agent, gallery, same, sid};

/// its modal, the Spinner page's starts the spinners, from the three.
#[test]
fn a_button_s_press_is_the_same_from_the_person_the_reader_and_an_agent() {
    for (page, id) in [
        ("Toast", "toast_page.button[success]"),
        ("Dialog", "dialog_page.delete"),
        ("Alert Dialog", "dialog_page.delete"),
        ("Spinner", "spinner_page.toggle"),
    ] {
        let mut person = gallery(page);
        person.click(id);
        let mut reader = gallery(page);
        reader.press(id);
        let agent = by_agent(page, "button", vec![ask(id, "activate")]);
        same(page, &person, &reader, &agent);
    }
    let mut g = gallery("Toast");
    g.press("toast_page.button[success]");
    let said = |g: &Gallery, words: &str| g.state().iter().any(|n| n.label == words);
    assert!(said(&g, "Saved! Everything went fine."), "the toast");
    let mut g = gallery("Dialog");
    g.press("dialog_page.delete");
    assert!(said(&g, "Delete repository?"), "the destructive modal");
    let mut g = gallery("Spinner");
    g.press("spinner_page.toggle");
    assert!(g.state().iter().any(|n| n.label == "Stop spinners"));
}

/// the page is as it was, as after a click.
#[test]
fn a_press_that_does_nothing_leaves_the_page_as_a_click_does() {
    for (page, id) in [
        ("Button", "button_page.group[0].card[0]"),
        ("Icon Button", "icon_button_page.button[copy]"),
        ("Tooltip", "tooltip_page.button"),
    ] {
        let before = gallery(page).state();
        let mut person = gallery(page);
        person.click(id);
        let mut reader = gallery(page);
        reader.press(id);
        let agent = by_agent(page, "nothing", vec![ask(id, "activate")]);
        assert_eq!(person.state(), before, "{page}");
        assert_eq!(reader.state(), before, "{page}");
        assert_eq!(agent.state(), before, "{page}");
    }
}

/// until Escape, or a click of the person, lets it go.
#[test]
fn the_focus_a_reader_leaves_on_a_component_page_goes_with_escape_and_a_click() {
    for (page, id) in [
        ("Navigation Menu", "navigation_menu_page.link[1]"),
        ("Breadcrumb", "breadcrumb_page.trail[0]"),
        ("Tree", "tree_page.tree[5]"),
        ("Chip", "chip_page.chip[5]"),
        ("Toast", "toast_page.button[info]"),
        ("Button", "button_page.group[1].card[0]"),
    ] {
        let focused = |g: &Gallery| {
            g.view
                .semantics()
                .and_then(|s| s.focused().map(ToString::to_string))
        };
        let mut g = gallery(page);
        assert!(g.view.assist_focus(&sid(id)), "{page}: {id}");
        assert_eq!(focused(&g).as_deref(), Some(id), "{page}: projected");
        assert!(g.view.close_top_overlay(), "{page}: Escape lets it go");
        assert_eq!(focused(&g), None, "{page}: after Escape");

        assert!(g.view.assist_focus(&sid(id)));
        g.frame();
        // the page's far corner, where no widget is
        let (x, y) = (1200.0, 780.0);
        assert!(g.c.hit(x, y).is_none(), "{page}: nothing there");
        g.view.handle_event(&WidgetEvent::MouseDown { x, y });
        g.view.handle_event(&WidgetEvent::MouseUp { x, y });
        assert_eq!(focused(&g), None, "{page}: a click elsewhere lets it go");
    }
}
