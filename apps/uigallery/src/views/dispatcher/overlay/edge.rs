//! The edge panels of the view's overlay layer (task-104 front B): the
//! sheet the Sheet page opens and the drawer the Drawer page opens. Like
//! the modal they block the page beneath while they hold the pointer, so
//! the view owns them (`ActiveOverlay`), they take an entry on its one
//! overlay stack, and what is asked of them runs through the gallery's
//! one dispatcher (`super::Overlays`).
//!
//! The sheet is a dialog: its close and its cancel cancel, its confirm
//! saves (a toast says so); the focus is its close, the first of its
//! buttons, until a reader or the keyboard moves it. The drawer is a
//! dialog of navigation: a link picked closes it (a toast names the
//! link), its close closes it; the focus is its current link.

use accord::{Request, SemId, SnapshotBuilder};
use engine::styling::theme::Intent;
use engine::widgets::overlay::OverlayKind;
use engine::widgets::overlay::drawer::Drawer;
use engine::widgets::overlay::sheet::{Edge, Sheet, SheetPart};
use engine::widgets::prelude::Rect;

use super::super::super::components::{drawer, sheet};
use super::super::super::{ActiveOverlay, ShowcaseView};
use super::{Asked, key_under};

/// The ids the view gives its edge panels.
const SHEET: &str = "sheet";
const DRAWER: &str = "drawer";

impl ShowcaseView {
    /// The window, the container the edge panels are held to.
    fn window(&self) -> Rect {
        Rect::new(0.0, 0.0, self.width, self.height)
    }

    /// The Sheet page's sheet opens, held to `edge` of the window.
    pub(in crate::views) fn open_sheet(&mut self, edge: Edge) {
        let mut widget = sheet::demo(edge);
        if let Some(id) = super::id(SHEET) {
            widget = widget.sem(id);
        }
        let p = widget.panel_rect(&self.theme, self.window());
        let id = self.push_edge(p);
        self.active_overlay = Some(ActiveOverlay::Sheet {
            id,
            widget: Box::new(widget),
        });
    }

    /// The Drawer page's drawer opens, held to the window's left.
    pub(in crate::views) fn open_drawer(&mut self) {
        let mut widget = drawer::demo();
        if let Some(id) = super::id(DRAWER) {
            widget = widget.sem(id);
        }
        let p = widget.panel_rect(&self.theme, self.window());
        let id = self.push_edge(p);
        self.active_overlay = Some(ActiveOverlay::Drawer {
            id,
            widget: Box::new(widget),
        });
    }

    /// An entry for an edge panel at `p`, sliding in with the neutral
    /// motion; the focus is the panel's own until a reader moves it.
    fn push_edge(&mut self, p: Rect) -> engine::widgets::overlay::OverlayId {
        self.overlay_focus = None;
        let motion = self.theme.intent_motion(Intent::Neutral);
        self.overlay_mgr
            .push_animated(OverlayKind::Anchored, p.x, p.y, p.w, p.h, &motion)
    }
}

/// The sheet into `b`, with the focus on the button `focus` names or on
/// its close.
pub(super) fn project_sheet(widget: &Sheet, focus: Option<&SemId>, b: &mut SnapshotBuilder) {
    let Some(id) = widget.sem_id() else {
        return;
    };
    widget.project(b, id, None);
    let parts = SheetPart::ALL.map(|p| id.key(p.key()));
    let at = focus.filter(|f| parts.contains(f));
    b.focus(at.unwrap_or(&parts[0]));
}

/// The drawer's stops: its close, then its links.
fn stops(widget: &Drawer, id: &SemId) -> Vec<SemId> {
    let links = (0..widget.link_count()).map(|i| id.key(i.to_string()));
    std::iter::once(id.key("close")).chain(links).collect()
}

/// The drawer into `b`, with the focus on the stop `focus` names or on
/// its current link.
pub(super) fn project_drawer(widget: &Drawer, focus: Option<&SemId>, b: &mut SnapshotBuilder) {
    let Some(id) = widget.sem_id() else {
        return;
    };
    widget.project(b, id, None);
    let stops = stops(widget, id);
    let current = widget.current.map(|i| id.key(i.to_string()));
    let at = focus
        .filter(|f| stops.contains(f))
        .cloned()
        .or(current)
        .unwrap_or_else(|| stops[0].clone());
    b.focus(&at);
}

/// What `req` asks of the sheet `overlay`: an answer.
pub(super) fn sheet_asked(
    widget: &Sheet,
    overlay: engine::widgets::overlay::OverlayId,
    req: &Request,
) -> Option<Asked> {
    let sheet = widget.sem_id()?;
    let button = key_under(&req.target, sheet);
    let confirm = match (req.target == *sheet, button, req.action.as_str()) {
        (true, _, "confirm") | (false, Some("confirm"), "activate") => true,
        (true, _, "cancel") | (false, Some("cancel" | "close"), "activate") => false,
        _ => return None,
    };
    Some(Asked::Sheet(overlay, confirm))
}

/// What `req` asks of the drawer `overlay`: to close, or a link.
pub(super) fn drawer_asked(
    widget: &Drawer,
    overlay: engine::widgets::overlay::OverlayId,
    req: &Request,
) -> Option<Asked> {
    let drawer = widget.sem_id()?;
    let key = key_under(&req.target, drawer);
    match (req.target == *drawer, key, req.action.as_str()) {
        (true, _, "close") | (false, Some("close"), "activate") => Some(Asked::Close(overlay)),
        (false, Some(k), "activate") => {
            let link = k.parse::<usize>().ok()?;
            (link < widget.link_count()).then_some(Asked::Link(overlay, link))
        }
        _ => None,
    }
}

/// The sheet's button or the drawer's stop `id` names, drawn focused.
pub(super) fn show_focus(active: &mut ActiveOverlay, id: &SemId) {
    match active {
        ActiveOverlay::Sheet { widget, .. } => {
            let key = widget.sem_id().and_then(|s| key_under(id, s));
            let part = SheetPart::ALL.into_iter().find(|p| Some(p.key()) == key);
            widget.set_focused(part);
        }
        ActiveOverlay::Drawer { widget, .. } => {
            let key = widget.sem_id().and_then(|d| key_under(id, d));
            let link = key.and_then(|k| k.parse::<usize>().ok());
            widget.set_focused(key == Some("close"), link);
        }
        _ => {}
    }
}
