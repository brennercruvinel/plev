//! The Surface page in both forms of its surface, as the gallery draws it
//! without a window (the launch state `--render Surface` stages, then the
//! other form switched to and settled): each form's scene and projection,
//! and with a GPU the pixels through the window's frame path.
//!
//! With `SURFACE_PAGE_OUT=<dir>` it also writes each form's frame as a
//! png and its projection as the json `plev observe` serves
//! (`surface-compact.png`, `surface-compact.json`, `surface-two-columns.*`),
//! to look at.

use std::path::PathBuf;

use engine::drawing::compositor::Compositor;
use engine::drawing::gpu::RenderConfig;
use engine::drawing::headless::{HeadlessError, Renderer, Viewport};
use accord::SemId;

use super::super::FORMS;
use crate::shell::semantics_json;
use crate::views::ShowcaseView;

const DESKTOP: Viewport = Viewport {
    width: 1200.0,
    height: 800.0,
    scale: 1.0,
};

/// One turn of a 60 Hz display.
const TURN: f32 = 1.0 / 60.0;

fn sid(text: &str) -> SemId {
    SemId::parse(text).unwrap()
}

/// The gallery on the Surface page with its surface in the form `form`,
/// every transition settled, and the compositor of its last frame.
fn staged(form: usize) -> (ShowcaseView, Compositor) {
    let mut view = ShowcaseView::new(DESKTOP.width, DESKTOP.height);
    view.jump_to_page("Surface");
    assert_eq!(view.page_title(), "Surface");
    view.resize(DESKTOP.width, DESKTOP.height, DESKTOP.scale);
    let mut c = Compositor::new();
    view.render(&mut c);
    if form != 0 {
        let todo = &view.panes.todo;
        assert!(view.panes.surface.switch(form, todo), "the form switches");
    }
    let mut steps = 0;
    while view.tick(TURN) {
        view.render(&mut c);
        steps += 1;
        assert!(steps < 600, "the page never settles");
    }
    view.render(&mut c);
    (view, c)
}

fn out(name: &str) -> Option<PathBuf> {
    let dir = PathBuf::from(std::env::var_os("SURFACE_PAGE_OUT")?);
    Some(dir.join(format!("surface-{}", name.replace(' ', "-"))))
}

#[test]
fn the_page_draws_headless_in_both_forms() {
    let mut renderer = match Renderer::new(DESKTOP, RenderConfig::default()) {
        Ok(r) => Some(r),
        Err(HeadlessError::NoAdapter) => None,
        Err(e) => panic!("{e}"),
    };
    let field = sid("todo_panel.input");
    let mut rects = Vec::new();
    for (form, f) in FORMS.iter().enumerate() {
        let (view, mut c) = staged(form);
        let snap = view.semantics().expect("the page projects");
        // the todo and the surface in the form it has, one snapshot
        assert!(snap.get(&sid("todo.input")).is_some());
        assert!(snap.get(&field).is_some(), "{}", f.label);
        let columns = snap.get(&sid("todo_panel.side")).is_some();
        assert_eq!(columns, form == 1, "{}", f.label);
        assert_eq!(snap.get(&sid("todo_panel.counts")).is_some(), form == 0);
        let bound = c.binding(&field).expect("the field is drawn").rect;
        rects.push(bound);

        let Some(r) = renderer.as_mut() else {
            eprintln!("no GPU adapter: the scene and the projection only");
            continue;
        };
        let clear = view.theme.colors.bg.to_linear_array();
        let frame = r.render(&mut c, clear).expect("the frame draws");
        if let Some(path) = out(f.label) {
            frame
                .write_png(path.with_extension("png"))
                .expect("the png writes");
            let json = semantics_json(&snap, false).expect("the json");
            std::fs::write(path.with_extension("json"), json).expect("the json writes");
        }
    }
    assert_ne!(rects[0], rects[1], "the field sits elsewhere in each form");
}

/// The page after a proposal, as the gallery draws it without a window:
/// the intent written in the page's field and proposed, every transition
/// settled; the proposed form in the projection and the scene, and with
/// a GPU its pixels (`surface-proposed.png` and `.json` under
/// `SURFACE_PAGE_OUT`).
#[test]
fn the_page_draws_headless_after_a_proposal() {
    let (mut view, mut c) = staged(0);
    let intent = "only what is active and the add field";
    view.panes.surface.proposing.intent.set_text(intent);
    let todo = &mut view.panes.todo;
    assert!(view.panes.surface.propose(todo).is_some());
    view.render(&mut c);
    let mut steps = 0;
    while view.tick(TURN) {
        view.render(&mut c);
        steps += 1;
        assert!(steps < 600, "the page never settles");
    }
    view.render(&mut c);
    let snap = view.semantics().expect("the page projects");
    for (id, kept) in [
        ("todo_panel.filter.active", true),
        ("todo_panel.active", true),
        ("todo_panel.filter.all", false),
        ("todo_panel.done", false),
        ("surface_page.intent", true),
    ] {
        assert_eq!(snap.get(&sid(id)).is_some(), kept, "{id}");
        assert_eq!(c.binding(&sid(id)).is_some(), kept, "{id} drawn");
    }
    let Ok(mut r) = Renderer::new(DESKTOP, RenderConfig::default()) else {
        eprintln!("no GPU adapter: the scene and the projection only");
        return;
    };
    let clear = view.theme.colors.bg.to_linear_array();
    let frame = r.render(&mut c, clear).expect("the frame draws");
    if let Some(path) = out("proposed") {
        frame
            .write_png(path.with_extension("png"))
            .expect("the png writes");
        let json = semantics_json(&snap, false).expect("the json");
        std::fs::write(path.with_extension("json"), json).expect("the json writes");
    }
}
