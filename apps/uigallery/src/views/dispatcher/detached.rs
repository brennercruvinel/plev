//! The active page in a window of its own as well (task-76 item 2): the
//! view's side. The shell (`crate::app::detached`) owns the window; the
//! view lays the page out for it, draws it into that window's
//! compositor, takes its pointer events, paces it, and projects the
//! window beside the page.

use accord::{Role, SemErrors, Snapshot, Value};
use engine::control::input::scroll::ScrollState;
use engine::drawing::compositor::{Compositor, SceneNode};
use engine::styling::theme::Theme;
use engine::widgets::overlay::OverlayManager;
use engine::widgets::prelude::{EventResult, Rect, Wake, WidgetEvent};

use super::super::{Layers, Pane, ShowcaseView};

/// The active page in a second window as well: the same pane state and
/// the same dispatcher, laid out in that window's size and drawn into
/// that window's compositor, with layers of its own there (a layer id
/// names a layer of one compositor). The main window keeps the chrome
/// and the page; a change made in either window shows in the other on
/// its next frame, and the page's motion asks frames of both.
pub struct Detached {
    width: f32,
    height: f32,
    scale: f32,
    layers: Option<Layers>,
    /// The page scroll of this window; the main window keeps its own.
    scroll: ScrollState,
    /// The page's ask of the loop since the shell last read it
    /// (`page_wake` merges into it; the main window's merge gets it too).
    pub(super) wake: Wake,
}

impl Detached {
    /// The rect the page lays out in: the window inside a gutter, never
    /// narrower than a card.
    fn viewport(&self, theme: &Theme) -> Rect {
        let gutter = theme.spacing.xl;
        Rect::new(
            gutter,
            gutter,
            (self.width - gutter * 2.0).max(theme.size.card_min_w),
            (self.height - gutter * 2.0).max(theme.size.field_min_w),
        )
    }

    /// The viewport shifted up by this window's page scroll.
    fn content(&self, theme: &Theme) -> Rect {
        let mut rect = self.viewport(theme);
        rect.y -= self.scroll.offset();
        rect
    }
}

impl ShowcaseView {
    /// The active page shows in a window of its own too.
    pub fn is_detached(&self) -> bool {
        self.panes.detached.is_some()
    }

    /// The active page shows in a window of its own too, one of `width x
    /// height` logical pixels at `scale`. `false` for a placeholder page
    /// (nothing to show) or when it has one already.
    pub fn detach(&mut self, width: f32, height: f32, scale: f32) -> bool {
        if self.pane().is_none() || self.panes.detached.is_some() {
            return false;
        }
        self.panes.detached = Some(Detached {
            width,
            height,
            scale,
            layers: None,
            scroll: ScrollState::new(),
            wake: Wake::idle(),
        });
        true
    }

    /// The page's own window closed: the main window is again the only
    /// one showing it. `false` when it was already.
    pub fn attach(&mut self) -> bool {
        self.panes.detached.take().is_some()
    }

    /// The page's own window changed size or scale.
    pub fn resize_detached(&mut self, width: f32, height: f32, scale: f32) {
        if let Some(detached) = self.panes.detached.as_mut() {
            detached.width = width;
            detached.height = height;
            detached.scale = scale;
        }
    }

    /// The frame of the page's own window: the page alone, laid out in
    /// that window, drawn into `c` (that window's compositor) with layers
    /// made there on the first frame. Nothing when the page is in the
    /// main window.
    pub fn render_detached(&mut self, c: &mut Compositor) {
        let Some(detached) = self.panes.detached.as_ref() else {
            return;
        };
        let theme = self.theme.clone();
        let viewport = detached.viewport(&theme);
        let scale = detached.scale;
        let height = self.pane_content_height(viewport);
        let Some(detached) = self.panes.detached.as_mut() else {
            return;
        };
        detached.scroll.set_viewport(viewport.h);
        detached.scroll.set_content(height);
        let content = detached.content(&theme);

        c.begin_frame();
        let layers = *detached.layers.get_or_insert_with(|| Layers {
            list: c.create_layer(10),
            overlay: c.create_layer(OverlayManager::BASE_Z),
            toast: c.create_layer(OverlayManager::BASE_Z + 200),
        });
        // the virtualized list clips to its panel, as in the main window
        let clip = (self.pane() == Some(Pane::ScrollArea)).then(|| {
            let b = super::super::components::scroll_area::ScrollAreaPage::bounds(content, &theme);
            (
                (b.x * scale) as u32,
                (b.y * scale) as u32,
                (b.w * scale).ceil() as u32,
                (b.h * scale).ceil() as u32,
            )
        });
        c.set_layer_clip_rect(layers.list, clip);
        c.push(SceneNode::PushClip {
            x: viewport.x,
            y: viewport.y,
            w: viewport.w,
            h: viewport.h,
        });
        self.pane_render(c, layers, content, &theme);
        c.push(SceneNode::PopClip);
    }

    /// A pointer event in the page's own window, in its logical pixels:
    /// the page takes it, then the page scroll of that window. `true`
    /// when a frame is needed. Nothing when the page is in the main
    /// window.
    pub fn detached_event(&mut self, event: &WidgetEvent) -> bool {
        let Some(detached) = self.panes.detached.as_ref() else {
            return false;
        };
        let content = detached.content(&self.theme);
        let mut result = self.pane_event(event, content);
        if let WidgetEvent::Scroll { delta, .. } = *event
            && !result.handled
            && let Some(detached) = self.panes.detached.as_mut()
        {
            let old = detached.scroll.offset();
            detached.scroll.scroll_by(delta);
            if detached.scroll.offset() != old {
                result = result.merge(EventResult::changed());
            }
        }
        result.changed
    }

    /// The page's ask of the loop since the last call, for its own
    /// window's pacing; idle when the page has no window of its own.
    pub fn detached_wake(&mut self) -> Wake {
        match self.panes.detached.as_mut() {
            Some(detached) => std::mem::take(&mut detached.wake),
            None => Wake::idle(),
        }
    }
}

/// The projection with the page's own window in it: a `window` node (the
/// window's title, the page it shows, the page roots shown in it) beside
/// the page's nodes, which keep their ids, tree, state and actions. Only
/// while the page has a window of its own; with one window the
/// projection is the page's alone, byte for byte. What the rebuild loses is each node's
/// `source`: the builder records its call site, and that is here. The
/// contract has no way to carry a finished snapshot into another; that
/// is the change `sem` would need to keep it.
pub(super) fn with_window(page: &Snapshot, title: &str) -> Result<Snapshot, SemErrors> {
    let mut b = accord::SnapshotBuilder::new();
    let roots: Vec<String> = page.roots().map(|n| n.id.to_string()).collect();
    b.node("window", Role::Window, format!("plev, {title}"))
        .state("page", Value::Text(title.to_string()))
        .state("shows", Value::List(roots));
    extend(&mut b, page);
    b.finish()
}

/// Every node of `page` declared again into `b`, with its parent,
/// description, state, constraints, flags, actions and the focus, so a
/// finished snapshot joins another (the window beside the page, the
/// shell beside the page). What the rebuild loses is each node's
/// `source`: the builder records its call site, and that is here.
pub(super) fn extend(b: &mut accord::SnapshotBuilder, page: &Snapshot) {
    for node in page.nodes().values() {
        let mut n = b.node(&node.id, node.role, node.label.clone());
        if let Some(parent) = &node.parent {
            n.parent(parent);
        }
        if let Some(text) = &node.description {
            n.description(text.clone());
        }
        for (key, value) in &node.state {
            n.state(key, value.clone());
        }
        n.constraints(node.constraints.clone())
            .enabled(node.enabled)
            .inert(node.inert)
            .focusable(node.focusable);
        for decl in &node.actions {
            let mut a = n.action(&decl.name);
            if let Some(label) = &decl.label {
                a.label(label.clone());
            }
            a.enabled(decl.enabled).policy(decl.policy);
            for at in &decl.reads {
                a.reads(at.clone());
            }
            if let Some(spec) = &decl.param {
                a.param(*spec);
            }
        }
    }
    if let Some(focused) = page.focused() {
        b.focus(focused);
    }
}
