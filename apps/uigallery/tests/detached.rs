//! The active page in a window of its own as well (task-76 item 2),
//! without a window: two headless hosts (`engine::runtime::window::WindowHost`,
//! the window's frame path into a texture), one `ShowcaseView`. The same
//! page, over the same state and the same dispatcher, lays out and binds
//! in both compositors; a click in the page's own window changes the
//! state, and the main window's next frame shows it; the projection
//! carries the window while it is open and is the page's alone, byte for
//! byte, before and after. Desktop only, like the shell's page window;
//! skips without a GPU adapter, like the headless tests.
#![cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]

use accord::{Role, SemId, Value};
use engine::drawing::gpu::RenderConfig;
use engine::drawing::headless::Frame;
use engine::runtime::window::WindowHost;
use engine::widgets::prelude::WidgetEvent;
use uigallery::views::ShowcaseView;

const MAIN: (u32, u32) = (1200, 800);
const PAGE: (u32, u32) = (900, 640);

fn host((w, h): (u32, u32)) -> Option<WindowHost> {
    let host = WindowHost::headless(w, h, 1.0, RenderConfig::default());
    if host.is_none() {
        eprintln!("no GPU adapter; skipping");
    }
    host
}

fn sid(text: &str) -> SemId {
    SemId::parse(text).expect("a valid id")
}

/// The main window's frame: the whole view.
fn frame_main(view: &mut ShowcaseView, host: &mut WindowHost) -> Frame {
    host.begin_frame();
    view.render(host.compositor_mut());
    let clear = view.theme.colors.bg.to_linear_array();
    assert!(host.present(clear, "main").is_some(), "the main frame drew");
    host.read().expect("the main frame reads")
}

/// The page window's frame: the page alone.
fn frame_page(view: &mut ShowcaseView, host: &mut WindowHost) -> Frame {
    host.begin_frame();
    view.render_detached(host.compositor_mut());
    let clear = view.theme.colors.bg.to_linear_array();
    assert!(host.present(clear, "page").is_some(), "the page frame drew");
    host.read().expect("the page frame reads")
}

fn completed(view: &ShowcaseView, item: &SemId) -> Value {
    view.semantics()
        .expect("the todo projects")
        .get(item)
        .expect("the item is projected")
        .state["completed"]
        .clone()
}

/// The pixels inside `rect` of `frame`, for a before/after comparison.
fn pixels(frame: &Frame, [x, y, w, h]: [f32; 4]) -> Vec<[u8; 4]> {
    let mut out = Vec::new();
    for py in (y as u32)..((y + h) as u32) {
        for px in (x as u32)..((x + w) as u32) {
            out.push(frame.pixel(px, py).expect("inside the frame"));
        }
    }
    out
}

#[test]
fn a_click_in_the_page_window_shows_in_the_main_window_on_its_next_frame() {
    let (Some(mut main), Some(mut page)) = (host(MAIN), host(PAGE)) else {
        return;
    };
    let mut view = ShowcaseView::new(MAIN.0 as f32, MAIN.1 as f32);
    view.jump_to_page("Todo");
    view.resize(MAIN.0 as f32, MAIN.1 as f32, 1.0);
    let item = sid("todo.item[1]");
    let window = sid("window");

    // one window: the page alone in the projection
    let alone = view.semantics().expect("the todo projects");
    assert!(
        alone.get(&window).is_none(),
        "no window node with one window"
    );
    let before_main = frame_main(&mut view, &mut main);
    let in_main = main
        .compositor()
        .binding(&item)
        .expect("the item is bound in the main window")
        .rect;

    // the page in a window of its own as well
    assert!(!view.is_detached());
    assert!(view.detach(PAGE.0 as f32, PAGE.1 as f32, 1.0));
    assert!(view.is_detached());
    assert!(!view.detach(PAGE.0 as f32, PAGE.1 as f32, 1.0), "once");
    let before_page = frame_page(&mut view, &mut page);
    let in_page = page
        .compositor()
        .binding(&item)
        .expect("the item is bound in the page window")
        .rect;
    assert_ne!(in_main, in_page, "each window lays the page out for itself");
    assert!(
        main.compositor().binding(&item).is_some(),
        "the main window keeps the page"
    );

    // the projection carries the window: which page, which roots
    let both = view.semantics().expect("the todo projects");
    let node = both.get(&window).expect("the window node");
    assert_eq!(node.role, Role::Window);
    assert_eq!(node.label, "plev, Todo");
    assert_eq!(node.state["page"], Value::Text("Todo".into()));
    assert_eq!(node.state["shows"], Value::List(vec!["todo".into()]));
    assert_eq!(
        both.len(),
        alone.len() + 1,
        "the page's nodes, plus the window"
    );
    for (id, was) in alone.nodes() {
        let is = both.get(id).expect("every node of the page is still there");
        assert_eq!(is, was, "{id}: the same node, source aside");
    }
    assert_ne!(both.revision(), alone.revision());

    // a click in the page window, on the item's checkbox: the same state
    // and the same dispatcher change
    let was = completed(&view, &item);
    let (x, y) = (in_page[0] + 19.0, in_page[1] + in_page[3] / 2.0);
    assert!(view.detached_event(&WidgetEvent::MouseDown { x, y }));
    view.detached_event(&WidgetEvent::MouseUp { x, y });
    assert_ne!(completed(&view, &item), was, "the click toggled the item");

    // the main window's next frame shows it
    let after_main = frame_main(&mut view, &mut main);
    assert!(
        pixels(&before_main, in_main) != pixels(&after_main, in_main),
        "the item's row changed in the main window"
    );
    let after_page = frame_page(&mut view, &mut page);
    assert!(
        pixels(&before_page, in_page) != pixels(&after_page, in_page),
        "and in the page window"
    );

    // a click in the main window shows in the page window's next frame
    let (x, y) = (in_main[0] + 19.0, in_main[1] + in_main[3] / 2.0);
    assert!(view.handle_event(&WidgetEvent::MouseDown { x, y }));
    view.handle_event(&WidgetEvent::MouseUp { x, y });
    assert_eq!(
        completed(&view, &item),
        was,
        "toggled back from the main window"
    );
    let again_page = frame_page(&mut view, &mut page);
    assert!(pixels(&again_page, in_page) == pixels(&before_page, in_page));

    // the page window closes: the projection is the page's alone again,
    // as it was
    assert!(view.attach());
    assert!(!view.is_detached());
    assert!(!view.attach(), "once");
    let alone_again = view.semantics().expect("the todo projects");
    assert!(alone_again.get(&window).is_none());
    assert_eq!(alone_again.revision(), alone.revision());
}

#[test]
fn a_page_window_scrolls_on_its_own_and_a_placeholder_has_none() {
    let Some(mut page) = host((400, 300)) else {
        return;
    };
    let mut view = ShowcaseView::new(MAIN.0 as f32, MAIN.1 as f32);
    view.jump_to_page("Card");
    view.resize(MAIN.0 as f32, MAIN.1 as f32, 1.0);
    assert!(view.detach(400.0, 300.0, 1.0));
    frame_page(&mut view, &mut page);
    // the card page overflows a small window: the wheel scrolls it there,
    // and the main window's scroll stays where it was
    let offset = view.page_offset();
    assert!(view.detached_event(&WidgetEvent::Scroll {
        x: 100.0,
        y: 100.0,
        delta: 120.0,
    }));
    assert_eq!(view.page_offset(), offset, "the main window did not scroll");
    let scrolled = frame_page(&mut view, &mut page);
    frame_page(&mut view, &mut page);
    assert!(page.frames() >= 3);
    drop(scrolled);
    assert!(view.attach());

    // a placeholder page has nothing to show in a window of its own
    let mut view = ShowcaseView::new(MAIN.0 as f32, MAIN.1 as f32);
    let placeholder = uigallery::views::PAGES
        .iter()
        .position(|p| p.pane().is_none())
        .map(|i| uigallery::views::PAGES[i].title);
    if let Some(title) = placeholder {
        view.jump_to_page(title);
        assert!(!view.detach(400.0, 300.0, 1.0));
        assert!(!view.is_detached());
    }
}
