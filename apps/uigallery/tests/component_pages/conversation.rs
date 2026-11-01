//! Front C, conversation and feedback (task-104, #156): message, bubble,
//! message scroller, alert, attachment, marker and questionnaire, the same
//! request from the person, the reader and an agent.

use engine::widgets::prelude::WidgetEvent;

use engine::control::accessibility::accesskit::Action;

use accord::Value;
use engine::control::accessibility::accesskit::ActionData;
use uigallery::views::{EditKey, NavKey};

use super::harness::{Gallery, ask, ask_with, by_agent, gallery, same, sid};

/// The Message page's like chip of the quick comment, and a thread row.
const CHIP: &str = "message_page.group[1].card[0].message[0].reaction";
const ROW: &str = "message_page.group[0].card[0].message[0]";

/// The Alert page's invite, unread, and its `Accept`.
const ALERT: &str = "alert_page.group[1].card[1].alert[0]";
const ACCEPT: &str = "alert_page.group[1].card[1].alert[0].action[1]";

/// The Attachment page's tray of pictures: its first tile, that tile's
/// remove button, and the add button.
const TILE: &str = "attachment_page.group[1].card[0][0]";
const REMOVE: &str = "attachment_page.group[1].card[0][0].remove";
const ADD: &str = "attachment_page.group[1].card[0].add";

/// The Marker page's unread single-comment pin.
const PIN: &str = "marker_page.group[1].card[1].marker[0]";

/// The Badge page's chip at rest, and its destructive count.
const BADGE: &str = "badge_page.group[0].card[0].badge[0]";
const COUNT: &str = "badge_page.group[1].card[2].badge[3]";

/// The Avatar page's pin-sized avatar, and the fourth of a stack of five
/// (under the count where the card is narrow).
const AVATAR: &str = "avatar_page.group[0].card[0].avatar[0]";
const STACKED: &str = "avatar_page.group[2].card[1].stack.avatar[3]";

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
    for (page, id) in [
        ("Message", ROW),
        ("Message", CHIP),
        ("Bubble", BUBBLE),
        ("Alert", ALERT),
        ("Alert", ACCEPT),
        ("Attachment", TILE),
        ("Marker", PIN),
        ("Badge", BADGE),
    ] {
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
        let row = id
            .strip_suffix(".reaction")
            .or_else(|| id.strip_suffix(".action[1]"))
            .unwrap_or(id);
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

/// Answering the invite reads it: the person's click on `Accept`, the
/// reader's press and the agent's `activate` end in the same state, the
/// answer chosen and the unread dot gone.
#[test]
fn an_alert_s_answer_reads_it_the_same_from_the_three() {
    let before = gallery("Alert").state();
    let mut person = seen("Alert", ACCEPT);
    person.click(ACCEPT);
    let mut reader = gallery("Alert");
    reader.press(ACCEPT);
    let agent = by_agent("Alert", "answer", vec![ask(ACCEPT, "activate")]);
    assert_ne!(person.state(), before, "the answer changed the page");
    assert_eq!(reader.state(), person.state(), "the reader");
    assert_eq!(agent.state(), person.state(), "the agent");
    let s = person.view.semantics().expect("the page projects");
    let alert = s.get(&sid(ALERT)).expect("the alert is projected");
    assert_eq!(alert.state.get("unread"), Some(&accord::Value::Bool(false)));
}

/// A click on the alert reads it, as the reader's and the agent's
/// `activate` do.
#[test]
fn reading_an_alert_is_the_same_from_the_three() {
    const COMMENT: &str = "alert_page.group[1].card[0].alert[0]";
    let before = gallery("Alert").state();
    let mut person = seen("Alert", COMMENT);
    person.click(COMMENT);
    let mut reader = gallery("Alert");
    reader.press(COMMENT);
    let agent = by_agent("Alert", "read", vec![ask(COMMENT, "activate")]);
    assert_ne!(person.state(), before, "the dot went");
    assert_eq!(reader.state(), person.state(), "the reader");
    assert_eq!(agent.state(), person.state(), "the agent");
}

/// A remove button removes its tile, the same from the three: the
/// person's click (the button takes it hidden, as the kit's), the
/// reader's press, the agent's `activate`.
#[test]
fn removing_an_attachment_is_the_same_from_the_three() {
    let before = gallery("Attachment").state();
    let mut person = seen("Attachment", REMOVE);
    person.click(REMOVE);
    let mut reader = gallery("Attachment");
    reader.press(REMOVE);
    let agent = by_agent("Attachment", "remove", vec![ask(REMOVE, "activate")]);
    assert_ne!(person.state(), before, "the tile went");
    assert_eq!(reader.state(), person.state(), "the reader");
    assert_eq!(agent.state(), person.state(), "the agent");
    let s = person.view.semantics().expect("the page projects");
    assert!(s.get(&sid(TILE)).is_none(), "its node went with it");
}

/// The add button adds a picture, the same from the three.
#[test]
fn adding_an_attachment_is_the_same_from_the_three() {
    let before = gallery("Attachment").state();
    let mut person = seen("Attachment", ADD);
    person.click(ADD);
    let mut reader = gallery("Attachment");
    reader.press(ADD);
    let agent = by_agent("Attachment", "add", vec![ask(ADD, "activate")]);
    assert_ne!(person.state(), before, "a picture came");
    assert_eq!(reader.state(), person.state(), "the reader");
    assert_eq!(agent.state(), person.state(), "the agent");
}

/// A click on an unread pin opens its thread and reads it, as the reader's
/// press and the agent's `activate` do.
#[test]
fn opening_a_pin_reads_it_the_same_from_the_three() {
    let before = gallery("Marker").state();
    let mut person = seen("Marker", PIN);
    person.click(PIN);
    let mut reader = gallery("Marker");
    reader.press(PIN);
    let agent = by_agent("Marker", "open", vec![ask(PIN, "activate")]);
    assert_ne!(person.state(), before, "the pin was read");
    assert_eq!(reader.state(), person.state(), "the reader");
    assert_eq!(agent.state(), person.state(), "the agent");
}

/// A badge is a status a reader lands on: its label, its intent, and no
/// action for the reader nor for an agent.
#[test]
fn a_badge_is_read_with_its_intent_and_offers_nothing() {
    let g = gallery("Badge");
    let s = g.view.semantics().expect("the page projects");
    for id in [BADGE, COUNT] {
        let n = s
            .get(&sid(id))
            .unwrap_or_else(|| panic!("{id} is projected"));
        assert!(n.actions.is_empty(), "{id}: {:?}", n.actions);
        assert!(n.focusable, "{id}");
    }
    let count = s.get(&sid(COUNT)).unwrap();
    assert_eq!(count.label, "4");
    assert!(format!("{:?}", count.state).contains("destructive"));
}

/// An avatar is a person a reader lands on: an image named by them, with
/// their presence, offering nothing to the reader nor to an agent; a
/// person's click leaves the page as it was.
#[test]
fn an_avatar_is_read_by_its_person_and_offers_nothing() {
    let before = gallery("Avatar").state();
    let mut person = seen("Avatar", AVATAR);
    person.click(AVATAR);
    assert_eq!(person.state(), before, "the person");
    let reader = gallery("Avatar");
    let s = reader.view.semantics().expect("the page projects");
    for id in [AVATAR, STACKED] {
        let n = s
            .get(&sid(id))
            .unwrap_or_else(|| panic!("{id} is projected"));
        assert!(n.actions.is_empty(), "{id}: {:?}", n.actions);
        assert!(n.focusable, "{id}");
        assert!(!reader.offers(id, Action::Click), "{id}");
    }
    assert_eq!(s.get(&sid(AVATAR)).unwrap().label, "Luna Starfield");
    assert!(format!("{:?}", s.get(&sid(STACKED)).unwrap().state).contains("online"));
}

// wave 3: the Prompt Input page's live box, its line, its model menu, its
// voice toggle and its add-files menu.
const PROMPT_TEXT: &str = "prompt_input_page.group[0].card[0].prompt.text";
const PROMPT_MODEL: &str = "prompt_input_page.group[0].card[0].prompt.model";
const PROMPT_KLING: &str = "prompt_input_page.group[0].card[0].prompt.model[2]";
const PROMPT_VOICE: &str = "prompt_input_page.group[0].card[0].prompt.voice";
const PROMPT_ADD: &str = "prompt_input_page.group[0].card[0].prompt.add";
const PROMPT_FILE: &str = "prompt_input_page.group[0].card[0].prompt.files[1]";

/// The prompt's line: a person's typing, a reader's value and an agent's
/// `set_value` put the same text.
#[test]
fn the_prompt_takes_the_same_text_from_the_three() {
    let page = "Prompt Input";
    let mut person = gallery(page);
    person.click(PROMPT_TEXT);
    person.type_in("a low-poly fox");
    let mut reader = gallery(page);
    reader.reader(
        PROMPT_TEXT,
        Action::SetValue,
        Some(ActionData::Value("a low-poly fox".into())),
    );
    let agent = by_agent(
        page,
        "prompt",
        vec![ask_with(
            PROMPT_TEXT,
            "set_value",
            Value::from("a low-poly fox"),
        )],
    );
    same(page, &person, &reader, &agent);
}

/// The model menu: the person's two clicks, the reader's two presses and
/// the agent's `open` and `select` choose the same model and close it.
#[test]
fn choosing_a_model_is_the_same_from_the_three() {
    let page = "Prompt Input";
    let mut person = gallery(page);
    person.click(PROMPT_MODEL);
    person.click(PROMPT_KLING);
    let mut reader = gallery(page);
    reader.press(PROMPT_MODEL);
    reader.press(PROMPT_KLING);
    let agent = by_agent(
        page,
        "model",
        vec![ask(PROMPT_MODEL, "open"), ask(PROMPT_KLING, "select")],
    );
    same(page, &person, &reader, &agent);
    assert_eq!(
        person.value(&format!("{PROMPT_MODEL}:selected")),
        Some(Value::from("Kling AI"))
    );
}

/// Voice input, from the three: the line left out, the same state.
#[test]
fn voice_input_is_the_same_from_the_three() {
    let page = "Prompt Input";
    let mut person = gallery(page);
    person.click(PROMPT_VOICE);
    let mut reader = gallery(page);
    reader.press(PROMPT_VOICE);
    let agent = by_agent(page, "voice", vec![ask(PROMPT_VOICE, "toggle")]);
    same(page, &person, &reader, &agent);
}

/// The keyboard on the model menu: Tab reaches the trigger, Down opens it
/// on the chosen model, Down walks, Enter picks and the focus is the
/// trigger's again; on the add-files menu, Escape closes it and gives the
/// focus back to its button, and a second Escape lets it go.
#[test]
fn the_keyboard_walks_the_prompt_s_menus_and_escape_closes_them() {
    let mut g = gallery("Prompt Input");
    let focused = |g: &Gallery| {
        g.view
            .semantics()
            .and_then(|s| s.focused().map(ToString::to_string))
    };
    for _ in 0..40 {
        if focused(&g).as_deref() == Some(PROMPT_MODEL) {
            break;
        }
        g.view.handle_edit_key(EditKey::Tab);
        g.frame();
    }
    assert_eq!(focused(&g).as_deref(), Some(PROMPT_MODEL), "Tab reaches it");
    assert!(g.view.handle_nav(NavKey::Down), "opens");
    g.frame();
    assert_eq!(focused(&g).as_deref(), Some(&*format!("{PROMPT_MODEL}[0]")));
    assert!(g.view.handle_nav(NavKey::Down));
    g.frame();
    assert_eq!(focused(&g).as_deref(), Some(&*format!("{PROMPT_MODEL}[1]")));
    assert!(g.view.handle_enter(), "picks");
    g.frame();
    assert_eq!(
        g.value(&format!("{PROMPT_MODEL}:selected")),
        Some(Value::from("Sora 4"))
    );
    assert_eq!(focused(&g).as_deref(), Some(PROMPT_MODEL));

    g.press(PROMPT_ADD);
    assert!(g.view.assist_focus(&sid(PROMPT_FILE)));
    assert!(g.view.close_top_overlay(), "Escape closes the menu");
    g.frame();
    let s = g.view.semantics().expect("the page projects");
    assert!(s.get(&sid(PROMPT_FILE)).is_none(), "its rows went");
    assert_eq!(
        focused(&g).as_deref(),
        Some(PROMPT_ADD),
        "back on its button"
    );
    assert!(g.view.close_top_overlay(), "then the focus goes");
    assert_eq!(focused(&g), None);
}

// wave 3: the Comments page's single-comment thread, its conversation and
// the new comment that is being typed.
const THREAD: &str = "comments_page.group[1].card[1].thread";
const THREAD_REPLY: &str = "comments_page.group[1].card[1].thread.reply";
const THREAD_SEND: &str = "comments_page.group[1].card[1].thread.send";
const THREAD_CLOSE: &str = "comments_page.group[1].card[1].thread.close";
const TALK: &str = "comments_page.group[1].card[2].thread.messages";
const TYPING_ADD: &str = "comments_page.group[2].card[1].new.tools[0]";

/// A reply typed and sent: the person's typing and clicks, the reader's
/// value and press, the agent's `set_value` and `activate` end the thread
/// with the same message.
#[test]
fn a_reply_is_sent_the_same_from_the_three() {
    let page = "Comments";
    let mut person = seen(page, THREAD_REPLY);
    person.click(THREAD_REPLY);
    person.type_in("On it");
    person.click(THREAD_SEND);
    let mut reader = gallery(page);
    reader.reader(
        THREAD_REPLY,
        Action::SetValue,
        Some(ActionData::Value("On it".into())),
    );
    reader.press(THREAD_SEND);
    let agent = by_agent(
        page,
        "reply",
        vec![
            ask_with(THREAD_REPLY, "set_value", Value::from("On it")),
            ask(THREAD_SEND, "activate"),
        ],
    );
    same(page, &person, &reader, &agent);
    assert_eq!(
        person.value(&format!("{THREAD}.messages:count")),
        Some(Value::Int(2))
    );
}

/// The close disc closes the thread to its pin, from the three.
#[test]
fn closing_a_thread_is_the_same_from_the_three() {
    let page = "Comments";
    let mut person = seen(page, THREAD_CLOSE);
    person.click(THREAD_CLOSE);
    let mut reader = gallery(page);
    reader.press(THREAD_CLOSE);
    let agent = by_agent(page, "close", vec![ask(THREAD_CLOSE, "activate")]);
    same(page, &person, &reader, &agent);
    assert_eq!(
        person.value(&format!("{THREAD}:open")),
        Some(Value::Bool(false))
    );
}

/// The add tool of a new comment being typed attaches a picture, from the
/// three.
#[test]
fn attaching_to_a_new_comment_is_the_same_from_the_three() {
    let page = "Comments";
    let mut person = seen(page, TYPING_ADD);
    person.click(TYPING_ADD);
    let mut reader = gallery(page);
    reader.press(TYPING_ADD);
    let agent = by_agent(page, "attach", vec![ask(TYPING_ADD, "activate")]);
    same(page, &person, &reader, &agent);
}

/// The keyboard in a conversation: a reader's focus on its first message,
/// Down to the next, no wrap past the last; Escape lets the focus go.
#[test]
fn the_arrows_walk_a_conversation_and_escape_lets_go() {
    let mut g = gallery("Comments");
    let focused = |g: &Gallery| {
        g.view
            .semantics()
            .and_then(|s| s.focused().map(ToString::to_string))
    };
    let first = format!("{TALK}[0]");
    let second = format!("{TALK}[1]");
    assert!(g.view.assist_focus(&sid(&first)));
    g.frame();
    assert!(g.view.handle_nav(NavKey::Down));
    g.frame();
    assert_eq!(focused(&g).as_deref(), Some(second.as_str()));
    assert!(!g.view.handle_nav(NavKey::Down), "no wrap");
    assert!(g.view.close_top_overlay(), "Escape");
    assert_eq!(focused(&g), None);
}

// wave 3: the Notifications page's comment variation and its dropdown.
const VARIATION: &str = "notifications_page.group[0].card[0].alert";
const DROP: &str = "notifications_page.group[1].card[0].dropdown";
const UNREAD: &str = "notifications_page.group[1].card[0].dropdown.filter[1]";
const DROP_ACCEPT: &str = "notifications_page.group[1].card[0].dropdown.list[2].action[1]";

/// The Unread filter, from the three: the same rows shown.
#[test]
fn the_unread_filter_is_the_same_from_the_three() {
    let page = "Notifications";
    let mut person = seen(page, UNREAD);
    person.click(UNREAD);
    let mut reader = gallery(page);
    reader.press(UNREAD);
    let agent = by_agent(page, "unread", vec![ask(UNREAD, "select")]);
    same(page, &person, &reader, &agent);
    assert_eq!(
        person.value(&format!("{DROP}.list:count")),
        Some(Value::Int(5))
    );
}

/// Answering the invite in the dropdown reads it, from the three.
#[test]
fn answering_in_the_dropdown_is_the_same_from_the_three() {
    let page = "Notifications";
    let mut person = seen(page, DROP_ACCEPT);
    person.click(DROP_ACCEPT);
    let mut reader = gallery(page);
    reader.press(DROP_ACCEPT);
    let agent = by_agent(page, "accept", vec![ask(DROP_ACCEPT, "activate")]);
    same(page, &person, &reader, &agent);
}

/// A variation read by a click, a press and `activate`.
#[test]
fn reading_a_variation_is_the_same_from_the_three() {
    let page = "Notifications";
    let mut person = gallery(page);
    person.click(VARIATION);
    let mut reader = gallery(page);
    reader.press(VARIATION);
    let agent = by_agent(page, "read", vec![ask(VARIATION, "activate")]);
    same(page, &person, &reader, &agent);
}

/// The keyboard in the dropdown: the arrows walk its rows, End goes to
/// the last (scrolled in), the filters are a tab list the arrows cross;
/// Escape lets the focus go.
#[test]
fn the_keyboard_walks_the_dropdown() {
    let mut g = gallery("Notifications");
    let focused = |g: &Gallery| {
        g.view
            .semantics()
            .and_then(|s| s.focused().map(ToString::to_string))
    };
    assert!(g.view.assist_focus(&sid(&format!("{DROP}.list[0]"))));
    g.frame();
    assert!(g.view.handle_nav(NavKey::Down));
    g.frame();
    assert_eq!(focused(&g), Some(format!("{DROP}.list[1]")));
    assert!(g.view.handle_edit_key(EditKey::End));
    g.frame();
    assert_eq!(focused(&g), Some(format!("{DROP}.list[7]")));
    assert!(g.view.assist_focus(&sid(&format!("{DROP}.filter[0]"))));
    g.frame();
    assert!(g.view.handle_edit_key(EditKey::Right));
    g.frame();
    assert_eq!(focused(&g).as_deref(), Some(UNREAD));
    assert!(g.view.close_top_overlay(), "Escape");
    assert_eq!(focused(&g), None);
}
