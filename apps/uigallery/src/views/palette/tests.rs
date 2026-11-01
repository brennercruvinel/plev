//! The palette from the three actors (task-104 front H, #163): a person
//! (⌘K, typing, Enter), a screen reader and an agent open it, filter it
//! and go to a tab through it, and land on the same state.

use accord::{Actor, Request, SemId, Snapshot, Stage, Value};

use super::{Cmd, THEMES, commands};
use crate::views::foundations::theme;
use crate::views::{EditKey, PAGES, ShowcaseView};

const SEARCH: &str = "shell.sidebar.search";
const BOX: &str = "palette.palette";

fn sid(text: &str) -> SemId {
    SemId::parse(text).expect("a valid id")
}

fn agent() -> Actor {
    Actor::Agent {
        name: "claude".into(),
    }
}

fn view(page: &str) -> ShowcaseView {
    let mut view = ShowcaseView::new(1280.0, 800.0);
    view.jump_to_page(page);
    view
}

/// What `actor` reads: the reader its tree, an agent the gallery.
fn seen(view: &ShowcaseView, actor: &Actor) -> Snapshot {
    match actor {
        Actor::Assistive => view.reader_semantics(),
        _ => view.gallery_semantics().ok(),
    }
    .expect("a projection")
}

/// `actor`'s request on what it reads; whether it completed.
fn ask(
    view: &mut ShowcaseView,
    actor: Actor,
    id: &str,
    action: &str,
    param: Option<Value>,
) -> bool {
    let snapshot = seen(view, &actor);
    let req = Request::observe(&snapshot, actor, id, action, param)
        .unwrap_or_else(|e| panic!("{id} {action}: {e}"));
    let receipt = view.act(req).expect("a receipt");
    let done = matches!(receipt.stage(), Some(Stage::Completed(_)));
    if !done {
        eprintln!("{id} {action}: {:?}", receipt.stage());
    }
    done
}

/// The id of the command labelled `label` in the open palette, as
/// `actor` reads it.
fn command(view: &ShowcaseView, actor: &Actor, label: &str) -> String {
    let snapshot = seen(view, actor);
    let list = sid(BOX).child("list").expect("the list's id");
    snapshot
        .children(&list)
        .find(|n| n.label == label)
        .map(|n| n.id.to_string())
        .unwrap_or_else(|| panic!("no {label} in the list"))
}

#[test]
fn the_themes_are_the_theme_pages() {
    let names: Vec<&str> = THEMES.iter().map(|(n, _)| *n).collect();
    assert_eq!(names, theme::THEMES);
}

#[test]
fn it_lists_every_tab_grouped_as_the_sidebar_then_the_themes_and_the_pin() {
    let (command, cmds) = commands(false);
    assert_eq!(command.items().len(), cmds.len());
    assert_eq!(cmds.len(), PAGES.len() + THEMES.len() + 1);
    let mut pages: Vec<usize> = cmds
        .iter()
        .filter_map(|c| match c {
            Cmd::Page(i) => Some(*i),
            _ => None,
        })
        .collect();
    pages.sort_unstable();
    assert_eq!(
        pages,
        (0..PAGES.len()).collect::<Vec<_>>(),
        "every tab once"
    );
    for (item, cmd) in command.items().iter().zip(&cmds) {
        if let Cmd::Page(i) = cmd {
            assert_eq!(item.label, PAGES[*i].title);
        }
    }
    assert_eq!(cmds.last(), Some(&Cmd::Pin));
    let (pinned, _) = commands(true);
    assert_eq!(pinned.items().last().unwrap().label, "Unpin the sidebar");
}

/// The person: ⌘K opens it over the window, typing filters, Enter goes
/// to the tab and closes it; the reader and the agent open it from the
/// Search, set the box and select the command: all three on Badge.
#[test]
fn the_three_actors_open_it_filter_it_and_go_to_a_tab() {
    let badge = PAGES.iter().position(|p| p.title == "Badge").unwrap();

    let mut person = view("Button");
    assert!(person.handle_chord("k", true), "⌘K opens it");
    assert!(person.palette_open());
    for c in ["b", "a", "d", "g"] {
        assert!(person.handle_key(c), "typing is the palette's");
    }
    assert_eq!(
        person.page,
        PAGES.iter().position(|p| p.title == "Button").unwrap()
    );
    assert!(person.handle_edit_key(EditKey::Backspace));
    assert!(person.handle_key("g"));
    let typed = person
        .palette_mut()
        .map(|p| p.command.search.input.text().to_owned());
    assert_eq!(typed.as_deref(), Some("badg"));
    assert!(person.handle_enter());
    assert!(!person.palette_open(), "running closes it");
    assert_eq!(person.page, badge);

    for actor in [Actor::Assistive, agent()] {
        let mut v = view("Button");
        assert!(
            ask(&mut v, actor.clone(), SEARCH, "activate", None),
            "{actor}"
        );
        assert!(v.palette_open(), "{actor}");
        // under it the sidebar is inert, the box has the focus
        let snapshot = seen(&v, &actor);
        assert_eq!(snapshot.focused(), Some(&sid(BOX)), "{actor}");
        let text = Some(Value::Text("badg".into()));
        assert!(
            ask(&mut v, actor.clone(), BOX, "set_value", text),
            "{actor}"
        );
        let id = command(&v, &actor, "Badge");
        assert!(ask(&mut v, actor.clone(), &id, "select", None), "{actor}");
        assert!(!v.palette_open(), "{actor}");
        assert_eq!(v.page, badge, "{actor}");
    }
}

/// Opened with the focus in the sidebar (the Search, or ⌘K on the logo),
/// it gives the focus back there when it closes, by Escape or by a
/// `close`; opened from the page, the page keeps its own.
#[test]
fn closing_gives_the_focus_back_to_whoever_opened_it() {
    let mut v = view("Button");
    assert!(v.focus_sidebar(&sid(SEARCH)));
    assert!(v.handle_chord("k", true));
    assert!(v.palette_open());
    assert_eq!(v.sidebar_focus(), None, "the palette has the keys");
    assert!(v.close_top_overlay(), "Escape closes it");
    assert_eq!(v.sidebar_focus(), Some(&sid(SEARCH)));

    let logo = v.nav.sidebar.logo_id().cloned().expect("the logo's id");
    assert!(v.focus_sidebar(&logo));
    assert!(v.handle_chord("k", true));
    assert!(ask(&mut v, agent(), "palette", "close", None));
    assert_eq!(v.sidebar_focus(), Some(&logo));

    let mut page = view("Button");
    assert!(page.handle_chord("k", true));
    assert!(page.close_top_overlay());
    assert_eq!(page.sidebar_focus(), None);
}

/// ⌘K again, Escape and an agent's `close` close it; the pin and a theme
/// run through the shell as the same request a person's click makes.
#[test]
fn it_closes_and_runs_the_pin_and_a_theme() {
    let mut v = view("Button");
    assert!(v.handle_chord("k", true));
    assert!(v.handle_chord("k", true), "⌘K again closes it");
    assert!(!v.palette_open());
    assert!(v.handle_chord("K", true));
    assert!(v.close_top_overlay(), "Escape closes it");
    assert!(!v.palette_open());
    assert!(ask(&mut v, agent(), SEARCH, "activate", None));
    assert!(ask(&mut v, agent(), "palette", "close", None));
    assert!(!v.palette_open());

    let pinned = |v: &ShowcaseView| v.shell.dock.as_ref().unwrap().is_pinned();
    assert!(!pinned(&v));
    assert!(ask(&mut v, agent(), SEARCH, "activate", None));
    let id = command(&v, &agent(), "Pin the sidebar");
    assert!(ask(&mut v, agent(), &id, "select", None));
    assert!(pinned(&v));

    assert!(v.handle_chord("k", true));
    for c in "dracula".chars() {
        v.handle_key(&c.to_string());
    }
    assert!(v.handle_enter());
    assert_eq!(v.theme_name, "dracula");
}
