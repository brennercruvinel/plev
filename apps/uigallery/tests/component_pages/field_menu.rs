//! The text field's menu (task-74, the person's request), as the shell
//! forwards a right-click (`ShowcaseView::handle_right_click`) and the
//! keys, with no window: the field takes the keys, the Context Menu
//! page's menu opens with Cut, Copy, Paste and Select all, Cut and Copy
//! disabled with nothing selected and Paste always enabled (whether the
//! clipboard holds anything is never told); a click, the arrows and Enter, or Escape
//! answer it; an agent's pick is refused, the clipboard being the
//! person's. Every gallery holds a `LocalClipboard`.
//!
//! With `FIELD_MENU_OUT=<dir>` and a GPU, the first test also writes
//! `field-menu.png` there: the menu open over the Input page's field.

use accord::{Actor, Request, Stage, Value};
use engine::drawing::gpu::RenderConfig;
use engine::drawing::headless::{HeadlessError, Renderer, Viewport};
use engine::runtime::platform::clipboard::{ClipboardProvider, LocalClipboard};
use uigallery::views::{EditKey, NavKey};

use super::harness::{Gallery, gallery, sid};

const NAME: &str = "input_page.field[0]";
const MENU: &str = "context_menu";
const CUT: &str = "context_menu[101]";
const COPY: &str = "context_menu[102]";
const PASTE: &str = "context_menu[103]";
const SELECT_ALL: &str = "context_menu[104]";

/// The Input page with `typed` in the name field, a clipboard of its own
/// holding `held`, and nothing focused.
fn page(typed: &str, held: Option<&str>) -> (Gallery, SharedBoard) {
    let mut g = gallery("Input");
    let board = SharedBoard::default();
    if let Some(text) = held {
        board.0.borrow_mut().set_text(text);
    }
    g.view.set_clipboard(Box::new(board.clone()));
    g.click(NAME);
    g.type_in(typed);
    assert!(g.view.close_top_overlay(), "Escape blurs");
    g.frame();
    (g, board)
}

/// A `LocalClipboard` the test reads after the view wrote it.
#[derive(Clone, Default)]
struct SharedBoard(std::rc::Rc<std::cell::RefCell<LocalClipboard>>);

impl ClipboardProvider for SharedBoard {
    fn get_text(&mut self) -> Option<String> {
        self.0.borrow_mut().get_text()
    }

    fn set_text(&mut self, text: &str) {
        self.0.borrow_mut().set_text(text);
    }
}

impl SharedBoard {
    fn text(&self) -> Option<String> {
        self.0.borrow_mut().get_text()
    }
}

impl Gallery {
    fn right_click(&mut self, id: &str) -> bool {
        let (x, y) = self.middle(id);
        let opened = self.view.handle_right_click(x, y);
        self.frame();
        opened
    }

    fn enabled(&self, id: &str) -> Option<bool> {
        let snapshot = self.view.semantics()?;
        snapshot.get(&sid(id)).map(|n| n.enabled)
    }

    fn focus(&self) -> Option<String> {
        self.view
            .semantics()
            .and_then(|s| s.focused().map(ToString::to_string))
    }

    fn name(&self) -> Option<Value> {
        self.value(&format!("{NAME}:text"))
    }
}

#[test]
fn a_right_click_opens_the_fields_menu_and_a_click_copies() {
    let (mut g, board) = page("Ada Lovelace", None);
    assert!(g.right_click(NAME), "the field's menu opens");
    assert_eq!(g.focus().as_deref(), Some(PASTE), "the first enabled item");
    assert_eq!(g.enabled(CUT), Some(false), "nothing selected");
    assert_eq!(g.enabled(COPY), Some(false));
    assert_eq!(
        g.enabled(PASTE),
        Some(true),
        "an empty clipboard is not told"
    );
    assert_eq!(g.enabled(SELECT_ALL), Some(true));

    g.click(SELECT_ALL);
    assert_eq!(g.enabled(MENU), None, "picked, it leaves");
    assert!(g.right_click(NAME), "again, over the selection");
    assert_eq!(g.enabled(COPY), Some(true));
    assert_eq!(g.enabled(CUT), Some(true));
    if let Some(dir) = std::env::var_os("FIELD_MENU_OUT") {
        write_png(&mut g, dir.as_ref());
    }
    g.click(COPY);
    assert_eq!(board.text().as_deref(), Some("Ada Lovelace"));
    assert_eq!(g.name(), Some(Value::from("Ada Lovelace")));
    assert!(!g.state().iter().any(|n| n.label.contains("Copy")));
}

/// The menu's keyboard is the context menu's: the focus on its first
/// enabled item, the arrows down it, Enter picks, Escape closes and the
/// field keeps the keys.
#[test]
fn the_menus_keys_cut_and_paste() {
    let (mut g, board) = page("hello world", Some("big\nwide"));
    g.click(NAME);
    assert!(g.view.handle_edit_key(EditKey::End));
    for _ in 0..5 {
        assert!(g.view.handle_edit_key(EditKey::SelectLeft));
    }
    assert!(g.right_click(NAME));
    assert_eq!(g.focus().as_deref(), Some(CUT));
    assert!(g.view.handle_enter(), "Enter picks Cut");
    g.frame();
    assert_eq!(g.name(), Some(Value::from("hello ")));
    assert_eq!(board.text().as_deref(), Some("world"));

    assert!(g.right_click(NAME));
    assert_eq!(g.enabled(PASTE), Some(true));
    assert!(g.view.handle_nav(NavKey::Down));
    assert!(g.view.handle_nav(NavKey::Down));
    assert_eq!(g.focus().as_deref(), Some(PASTE));
    assert!(g.view.handle_enter());
    g.frame();
    assert_eq!(g.name(), Some(Value::from("hello world")));

    assert!(g.right_click(NAME));
    assert!(g.view.close_top_overlay(), "Escape closes it");
    g.frame();
    assert_eq!(g.enabled(MENU), None);
    assert_eq!(g.focus().as_deref(), Some(NAME), "the field keeps the keys");
}

/// The clipboard is the person's: an agent that reads the open menu and
/// picks Paste or Copy is refused, and neither the field nor the
/// clipboard moves; nor does the clipboard ever show in the projection.
#[test]
fn an_agents_pick_is_refused() {
    let (mut g, board) = page("abc", Some("secret"));
    g.click(NAME);
    assert!(g.view.handle_edit_key(EditKey::End));
    assert!(g.view.handle_edit_key(EditKey::SelectLeft));
    assert!(g.right_click(NAME));
    assert_eq!(g.enabled(PASTE), Some(true), "as with an empty one");
    let seen = g.view.semantics().expect("the projection");
    assert!(!seen.nodes().values().any(|n| {
        n.label.contains("secret")
            || n.state
                .values()
                .any(|v| matches!(v, Value::Text(t) if t.contains("secret")))
    }));
    for item in [PASTE, COPY] {
        let agent = Actor::Agent {
            name: "tester".to_owned(),
        };
        let req = Request::observe(&seen, agent, &sid(item), "activate", None)
            .unwrap_or_else(|e| panic!("{item}: {e}"));
        let receipt = g.view.act(req).expect("a receipt");
        assert!(
            matches!(receipt.stage(), Some(Stage::Failed { .. })),
            "{item}: {:?}",
            receipt.stage()
        );
    }
    assert_eq!(g.name(), Some(Value::from("abc")));
    assert_eq!(board.text().as_deref(), Some("secret"));
    assert!(g.enabled(MENU).is_some(), "still the person's to answer");
}

/// A right-click off the fields opens nothing, and a field's menu is the
/// Input page's alone for now.
#[test]
fn a_right_click_off_a_field_opens_nothing() {
    let (mut g, _) = page("abc", Some("clip"));
    let content = g.c.binding(&sid(NAME)).unwrap().rect;
    let below = content[1] + content[3] * 5.0;
    assert!(!g.view.handle_right_click(content[0] + 4.0, below));
    let mut b = gallery("Button");
    assert!(!b.view.handle_right_click(400.0, 300.0));
}

/// The page with the menu open, through the window's frame path at 2x,
/// when a GPU answers.
fn write_png(g: &mut Gallery, dir: &std::path::Path) {
    let (w, h) = (1280.0, 800.0);
    g.view.resize(w, h, 2.0);
    for _ in 0..120 {
        if !g.view.tick(1.0 / 60.0) {
            break;
        }
    }
    g.frame();
    let mut renderer = match Renderer::new(Viewport::new(w, h, 2.0), RenderConfig::default()) {
        Ok(r) => r,
        Err(HeadlessError::NoAdapter) => {
            eprintln!("no GPU adapter: no png");
            return;
        }
        Err(e) => panic!("{e}"),
    };
    let frame = renderer
        .render(&mut g.c, g.view.theme.colors.bg.to_linear_array())
        .expect("the frame draws");
    std::fs::create_dir_all(dir).expect("the out dir");
    frame
        .write_png(dir.join("field-menu.png"))
        .expect("the png writes");
}
