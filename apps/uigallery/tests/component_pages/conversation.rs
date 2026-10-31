//! Front C, conversation and feedback (task-104, #156): message, bubble,
//! message scroller, alert, attachment, marker and questionnaire, the same
//! request from the person, the reader and an agent.

use engine::widgets::prelude::WidgetEvent;

use super::harness::{Gallery, ask, by_agent, gallery, sid};

/// The Message page's like chip of the quick comment, and a thread row.
const CHIP: &str = "message_page.group[1].card[0].message[0].reaction";
const ROW: &str = "message_page.group[0].card[0].message[0]";

/// The gallery on `page`, scrolled so a person sees `id`.
fn seen(page: &str, id: &str) -> Gallery {
    let mut g = gallery(page);
    let bound = g.c.binding(&sid(id)).map(|b| b.rect);
    let [_, y, _, h] = bound.unwrap_or_else(|| panic!("{id} is not bound"));
    if y + h > 700.0 {
        g.view.scroll_page_to(y - 200.0);
        g.frame();
        g.frame();
    }
    g
}

/// The like chip presses and nothing else happens, as in the kit: the
/// person's click, the reader's press and the agent's `activate` leave
/// the page as it was.
#[test]
fn a_like_chip_s_press_leaves_the_message_page_as_it_was_from_the_three() {
    let before = gallery("Message").state();
    let mut person = seen("Message", CHIP);
    person.click(CHIP);
    let mut reader = gallery("Message");
    reader.press(CHIP);
    let agent = by_agent("Message", "like", vec![ask(CHIP, "activate")]);
    assert_eq!(person.state(), before, "the person");
    assert_eq!(reader.state(), before, "the reader");
    assert_eq!(agent.state(), before, "the agent");
}

/// A message is a place a reader lands on, not a control: it offers no
/// action to the reader nor to an agent, only its like chip does.
#[test]
fn a_message_row_is_read_and_its_chip_is_pressed() {
    let g = gallery("Message");
    let s = g.view.semantics().expect("the page projects");
    let row = s.get(&sid(ROW)).expect("the row is projected");
    assert!(row.actions.is_empty(), "{:?}", row.actions);
    assert!(row.focusable);
    let chip = s.get(&sid(CHIP)).expect("the chip is projected");
    assert!(chip.actions.iter().any(|a| a.name.as_str() == "activate"));
}

/// The focus a reader leaves on a message, its chip or a bubble goes with
/// Escape and with a click of the person where no widget is.
#[test]
fn the_focus_a_reader_leaves_on_a_message_goes_with_escape_and_a_click() {
    const BUBBLE: &str = "bubble_page.group[0].card[0].bubble[0]";
    for (page, id) in [("Message", ROW), ("Message", CHIP), ("Bubble", BUBBLE)] {
        let focused = |g: &Gallery| {
            g.view
                .semantics()
                .and_then(|s| s.focused().map(ToString::to_string))
        };
        let mut g = seen(page, id);
        assert!(g.view.assist_focus(&sid(id)), "{id}");
        assert_eq!(focused(&g).as_deref(), Some(id), "{id}: projected");
        assert!(g.view.close_top_overlay(), "{id}: Escape lets it go");
        assert_eq!(focused(&g), None, "{id}: after Escape");

        assert!(g.view.assist_focus(&sid(id)));
        g.frame();
        // beside the row, in the card's inset, level with what is
        // focused: on screen, where no widget is (a bubble's row starts
        // at the card's inset too)
        let row = id.strip_suffix(".reaction").unwrap_or(id);
        let [rx, _, _, _] = g.c.binding(&sid(row)).unwrap().rect;
        let (_, y) = g.middle(id);
        let x = rx - 4.0;
        assert!(g.c.hit(x, y).is_none(), "{id}: nothing at {x},{y}");
        g.view.handle_event(&WidgetEvent::MouseDown { x, y });
        g.view.handle_event(&WidgetEvent::MouseUp { x, y });
        assert_eq!(focused(&g), None, "{id}: a click elsewhere lets it go");
    }
}

/// The scroller's like chip, in the long thread, presses and nothing else
/// happens, from the three.
#[test]
fn a_like_chip_in_the_scroller_presses_as_it_was_from_the_three() {
    const IN_THREAD: &str = "message_scroller_page.group[0].card[1][2].reaction";
    let page = "Message Scroller";
    let before = gallery(page).state();
    let mut person = seen(page, IN_THREAD);
    person.click(IN_THREAD);
    let mut reader = gallery(page);
    reader.press(IN_THREAD);
    let agent = by_agent(page, "thread", vec![ask(IN_THREAD, "activate")]);
    assert_eq!(person.state(), before, "the person");
    assert_eq!(reader.state(), before, "the reader");
    assert_eq!(agent.state(), before, "the agent");
}

/// A reader landing on a message scrolled away brings it into view: the
/// thread scrolls, the page does not, and the person sees it where it is
/// bound.
#[test]
fn a_reader_s_focus_scrolls_a_hidden_message_into_view() {
    const HIDDEN: &str = "message_scroller_page.group[0].card[1][10]";
    let mut g = gallery("Message Scroller");
    assert!(
        g.c.binding(&sid(HIDDEN)).is_none(),
        "scrolled away at first"
    );
    assert!(g.view.assist_focus(&sid(HIDDEN)));
    g.frame();
    g.frame();
    let _ = g.middle(HIDDEN);
    assert_eq!(
        g.view.page_offset(),
        0.0,
        "the thread scrolled, not the page"
    );
}
