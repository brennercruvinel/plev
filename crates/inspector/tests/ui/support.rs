//! what the tests share: the recorded sessions, a click, and every text a
//! frame of the view drew.

use std::path::PathBuf;

use engine::compositor::{Compositor, SceneNode};
use inspector::InspectorView;
use sem::SemId;

pub type Fallible = Result<(), Box<dyn std::error::Error>>;

/// a session the repository keeps: `a2ppa`, the commit scenario of
/// task-63 (a stale submit refused with `changes:staged: removed
/// src/session.rs`, then accepted with its sha), or `bug84`.
pub fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../ide/tests/fixtures/sessions")
        .join(format!("{name}.monster"))
}

pub fn open(
    name: &str,
    width: f32,
    height: f32,
) -> Result<InspectorView, Box<dyn std::error::Error>> {
    Ok(InspectorView::open(&fixture(name), width, height)?)
}

pub fn id(text: &str) -> SemId {
    match SemId::parse(text) {
        Ok(id) => id,
        Err(e) => panic!("{text}: {e}"),
    }
}

/// a press and a release at `(x, y)`, the pointer moved there first.
pub fn click(view: &mut InspectorView, x: f32, y: f32) -> bool {
    let moved = view.pointer_move(x, y);
    let down = view.pointer_down(x, y);
    let up = view.pointer_up(x, y);
    moved | down | up
}

/// a click on the middle of `id` where it shows in the frame on screen.
pub fn click_control(view: &mut InspectorView, id: &SemId) -> Result<(), String> {
    let r = view
        .control_rect(id)
        .ok_or_else(|| format!("{id} is not on the frame at +{}ms", view.at()))?;
    let (x, y) = r.center();
    click(view, x, y);
    Ok(())
}

/// the view drawn into a fresh compositor.
pub fn draw(view: &mut InspectorView) -> Compositor {
    let mut c = Compositor::new();
    view.render(&mut c);
    c
}

/// every node of every layer, in composite order.
pub fn nodes(c: &Compositor) -> Vec<SceneNode> {
    c.layers()
        .iter()
        .flat_map(|l| l.nodes().iter().cloned())
        .collect()
}

/// every text the compositor holds.
pub fn texts(c: &Compositor) -> Vec<String> {
    nodes(c)
        .into_iter()
        .filter_map(|n| match n {
            SceneNode::Text { key, .. } => Some(key.text),
            _ => None,
        })
        .collect()
}

/// the panel's rows, one string each.
pub fn rows(view: &InspectorView) -> Vec<String> {
    view.rows().iter().map(|r| r.text.clone()).collect()
}

/// the index of the first row at or after `from` that holds every one of
/// `parts`.
pub fn row_after(rows: &[String], from: usize, parts: &[&str]) -> Option<usize> {
    rows.iter()
        .enumerate()
        .skip(from)
        .find(|(_, r)| parts.iter().all(|p| r.contains(p)))
        .map(|(i, _)| i)
}
