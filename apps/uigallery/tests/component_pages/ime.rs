//! The input method on the gallery (task-74 phase 4), as the shell
//! forwards it (`ShowcaseView::handle_ime`, winit-free) and nothing more:
//! the Input page's focused field composes inline and takes the commit as
//! typing, the Editor page composes in the editor, and with no field
//! focused nothing takes it and the shell keeps the IME off
//! (`ime_area` is `None`). The preedit is only drawn: the projection
//! never holds it. No window, no input source of the system.
//!
//! With `IME_FIELD_OUT=<dir>` and a GPU, the Input page test also writes
//! `ime-field.png` there: the page with a field composing `にほんご`.

use engine::compositor::{Compositor, SceneNode};
use engine::gpu::RenderConfig;
use engine::headless::{HeadlessError, Renderer, Viewport};
use sem::Value;
use showcase::view::{EditKey, ImeEvent};

use super::harness::{Gallery, gallery, sid};

const NIHONGO: &str = "にほんご";
const FIELD: &str = "input_page.field[0]";

fn texts(c: &Compositor) -> Vec<String> {
    c.layers()
        .iter()
        .flat_map(|layer| layer.nodes())
        .filter_map(|n| match n {
            SceneNode::Text { key, .. } => Some(key.text.clone()),
            _ => None,
        })
        .collect()
}

/// Whether any node of the page's projection, its label or its state,
/// holds `text`.
fn projected(g: &Gallery, text: &str) -> bool {
    let snapshot = g.view.semantics().unwrap_or_default();
    snapshot.nodes().values().any(|n| {
        n.label.contains(text)
            || n.state
                .values()
                .any(|v| matches!(v, Value::Text(t) if t.contains(text)))
    })
}

#[test]
fn the_focused_field_composes_inline_and_takes_the_commit_as_typing() {
    let mut g = gallery("Input");
    assert_eq!(g.view.ime_area(), None, "no field focused: the IME is off");
    g.click(FIELD);
    let [x, y, w, h] = g.c.binding(&sid(FIELD)).unwrap().rect;
    let at_rest = g.view.ime_area().expect("a field focused: the IME is on");
    assert!(
        at_rest.x > x && at_rest.x < x + w && at_rest.y > y && at_rest.y + at_rest.h < y + h,
        "the candidate window at the caret, inside the field: {at_rest:?}"
    );

    assert!(g.view.handle_ime(ImeEvent::Preedit(NIHONGO, Some((6, 6)))));
    g.frame();
    assert!(texts(&g.c).iter().any(|t| t == NIHONGO), "drawn inline");
    assert_eq!(g.value(&format!("{FIELD}:text")), Some(Value::from("")));
    assert!(!projected(&g, NIHONGO), "the preedit is only drawn");
    let composing = g.view.ime_area().unwrap();
    assert!(
        composing.x > at_rest.x,
        "the window follows the preedit's caret"
    );

    // winit clears the preedit, then commits.
    assert!(g.view.handle_ime(ImeEvent::Preedit("", None)));
    assert!(g.view.handle_ime(ImeEvent::Commit("日本語")));
    g.frame();
    assert_eq!(
        g.value(&format!("{FIELD}:text")),
        Some(Value::from("日本語"))
    );
    assert!(g.state().iter().any(|n| n.label == "live value: 日本語"));
    assert!(!texts(&g.c).iter().any(|t| t.contains(NIHONGO)));

    // Typing goes on after it, as keys.
    g.type_in("!");
    assert_eq!(
        g.value(&format!("{FIELD}:text")),
        Some(Value::from("日本語!"))
    );

    assert!(g.view.handle_ime(ImeEvent::Preedit(NIHONGO, None)));
    g.frame();
    if let Some(dir) = std::env::var_os("IME_FIELD_OUT") {
        write_png(&mut g, dir.as_ref());
    }

    // Escape blurs: the composition goes with the focus, the IME off.
    assert!(g.view.close_top_overlay());
    g.frame();
    assert_eq!(g.view.ime_area(), None);
    assert!(!texts(&g.c).iter().any(|t| t.contains(NIHONGO)));
}

#[test]
fn with_no_field_focused_nothing_takes_the_input_method() {
    let mut g = gallery("Input");
    let before = g.state();
    let theme = g.view.theme_name.clone();
    assert!(!g.view.handle_ime(ImeEvent::Preedit(NIHONGO, None)));
    assert!(!g.view.handle_ime(ImeEvent::Commit("t")));
    assert!(!g.view.handle_ime(ImeEvent::Disabled));
    g.frame();
    assert_eq!(g.state(), before, "nothing changed");
    assert_eq!(g.view.theme_name, theme, "a commit is never a shortcut");
    assert_eq!(g.view.ime_area(), None);

    // A page without a field to compose in keeps it off too.
    let mut b = gallery("Button");
    assert_eq!(b.view.ime_area(), None);
    assert!(!b.view.handle_ime(ImeEvent::Commit("x")));
}

/// A composition left on the Input page by a switch to another page is
/// dropped when the IME goes off there, and never shows again.
#[test]
fn a_composition_left_behind_by_a_page_switch_is_dropped() {
    let mut g = gallery("Input");
    g.view.handle_edit_key(EditKey::Tab);
    assert!(g.view.ime_area().is_some(), "Tab reached a field");
    assert!(g.view.handle_ime(ImeEvent::Preedit(NIHONGO, None)));
    g.view.jump_to_page("Button");
    assert_eq!(g.view.ime_area(), None, "the shell turns the IME off");
    assert!(g.view.handle_ime(ImeEvent::Disabled), "and winit says so");
    g.view.jump_to_page("Input");
    g.frame();
    assert!(!texts(&g.c).iter().any(|t| t.contains(NIHONGO)));
    assert!(g.view.ime_area().is_some(), "the field kept the focus");
}

#[test]
fn the_editor_page_composes_in_the_editor() {
    let mut g = gallery("Editor");
    let at_rest = g.view.ime_area().expect("the editor takes the IME");
    assert!(g.view.handle_ime(ImeEvent::Preedit(NIHONGO, None)));
    g.frame();
    assert!(
        texts(&g.c).iter().any(|t| t.contains(NIHONGO)),
        "drawn inline"
    );
    assert!(g.view.ime_area().unwrap().x > at_rest.x);
    assert!(g.view.handle_ime(ImeEvent::Preedit("", None)));
    assert!(g.view.handle_ime(ImeEvent::Commit("日本語")));
    g.frame();
    let shown = texts(&g.c);
    assert!(shown.iter().any(|t| t.starts_with("日本語")), "{shown:?}");
    assert!(!shown.iter().any(|t| t.contains(NIHONGO)));
}

/// The page through the window's frame path at 2x, when a GPU answers.
fn write_png(g: &mut Gallery, dir: &std::path::Path) {
    let (w, h) = (1280.0, 800.0);
    g.view.resize(w, h, 2.0);
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
        .write_png(dir.join("ime-field.png"))
        .expect("the png writes");
}
