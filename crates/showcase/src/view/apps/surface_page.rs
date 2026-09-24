//! Apps Demo, Surface: the todo's capabilities as a surface materialized
//! from verified plev IR (task-69), beside the todo page written by hand.
//!
//! The IR is the fixture of task-68, `crates/uir/fixtures/showcase-todo.ir.json`
//! (the field, the add button, the filter as chips, the counters as
//! stats), verified against the registry the todo projects
//! (`uir::Capabilities::from_snapshot`) and materialized by the `surface`
//! crate. Both halves operate one state through one dispatcher, the todo
//! page's: adding a todo or changing the filter on the generated surface
//! shows on the fixed todo, and back. The page projects the todo and the
//! surface as one snapshot, so an agent or a screen reader observes the
//! surface and acts on it like on any page; an act on one of its nodes is
//! routed to the capability the IR bound (`SurfaceHost::route`).
//!
//! Wide, the two halves sit side by side; narrow, the todo is on top.

use comps::prelude::{EmptyState, EventResult, Rect, Wake, WidgetEvent};
use engine::compositor::{Compositor, SceneNode, TextNodeKey};
use engine::text::{TextMeasurer, TextStyle};
use engine::theme::Theme;
use sem::{Receipt, Request, SemErrors, SemId, Semantics, Snapshot, SnapshotBuilder};
use surface::{Dispatch, SurfaceHost};
use uir::{Capabilities, GenerationPolicy, verify_json};

use super::EditKey;
use super::app::TodoPage;

/// The generated surface of the todo: the IR fixture of task-68.
const IR: &str = include_str!("../../../../uir/fixtures/showcase-todo.ir.json");

const FIXED: &str = "the todo page, written by hand";
const GENERATED: &str = "the same capabilities, materialized from showcase-todo.ir.json";

pub struct SurfacePage {
    host: SurfaceHost,
    /// The surface that opened (the IR's id), when it did.
    id: Option<SemId>,
    /// Why it did not: the verifier's first error.
    refused: Option<String>,
}

/// The todo page as the surface reaches it: its one dispatcher and its
/// projection.
struct Todo<'a>(&'a mut TodoPage);

impl Dispatch for Todo<'_> {
    fn dispatch(&mut self, req: Request) -> Option<Receipt> {
        let id = self.0.dispatch(req);
        self.0.receipt(id).cloned()
    }

    fn project(&self) -> Option<Snapshot> {
        self.0.snapshot().ok()
    }
}

/// Where each half goes in `content`: the fixed todo and the surface,
/// each under its caption (the rect a caption wraps in).
struct Halves {
    todo: Rect,
    surface: Rect,
    captions: [Rect; 2],
}

fn caption_style(theme: &Theme) -> TextStyle {
    theme.typography.caption_sm()
}

/// The band a caption takes at `width`: its measured lines and a gap.
fn caption_h(text: &str, width: f32, theme: &Theme) -> f32 {
    let (_, h) = TextMeasurer::measure_styled(text, &caption_style(theme), Some(width));
    h + theme.spacing.sm
}

impl SurfacePage {
    /// Verifies the IR against what `todo` projects now and opens it.
    pub fn new(todo: &TodoPage) -> Self {
        let mut page = Self {
            host: SurfaceHost::new(),
            id: None,
            refused: None,
        };
        let Ok(app) = todo.snapshot() else {
            page.refused = Some("the todo page does not project".to_owned());
            return page;
        };
        let caps = Capabilities::from_snapshot(&app);
        match verify_json(IR, &caps, &GenerationPolicy::default()) {
            Ok(ir) => match page.host.open(ir, &app) {
                Ok(surface) => page.id = Some(surface.id().clone()),
                Err(e) => page.refused = Some(e.to_string()),
            },
            Err(errors) => {
                let first = errors.first().map(ToString::to_string);
                page.refused = Some(first.unwrap_or_default());
            }
        }
        page
    }

    fn halves(&self, content: Rect, theme: &Theme, todo: &TodoPage) -> Halves {
        let gap = theme.spacing.xl;
        if content.w >= theme.size.card_min_w * 2.0 + gap {
            let w = (content.w - gap) / 2.0;
            let cap = caption_h(FIXED, w, theme).max(caption_h(GENERATED, w, theme));
            let top = content.y + cap;
            let h = (content.h - cap).max(0.0);
            let right = content.x + w + gap;
            return Halves {
                todo: Rect::new(content.x, top, w, h),
                surface: Rect::new(right, top, w, h),
                captions: [
                    Rect::new(content.x, content.y, w, cap),
                    Rect::new(right, content.y, w, cap),
                ],
            };
        }
        // narrow: the todo at its smallest height, the surface under it
        let (x, w) = (content.x, content.w);
        let fixed = caption_h(FIXED, w, theme);
        let generated = caption_h(GENERATED, w, theme);
        let todo_h = todo.content_height(Rect::new(x, content.y, w, 0.0));
        let below = content.y + fixed + todo_h + gap;
        Halves {
            todo: Rect::new(x, content.y + fixed, w, todo_h),
            surface: Rect::new(x, below + generated, w, content.h),
            captions: [
                Rect::new(x, content.y, w, fixed),
                Rect::new(x, below, w, generated),
            ],
        }
    }

    fn surface_h(&self, width: f32, theme: &Theme) -> f32 {
        let id = self.id.as_ref();
        match id.and_then(|id| self.host.get(id)) {
            Some(s) => s.height(width, theme),
            None => theme.size.card_min_w / 2.0,
        }
    }

    /// Natural height: the taller half, or both halves stacked.
    pub fn content_height(&self, content: Rect, theme: &Theme, todo: &TodoPage) -> f32 {
        let h = self.halves(content, theme, todo);
        let surface = h.surface.y - content.y + self.surface_h(h.surface.w, theme);
        let fixed = h.todo.y - content.y + todo.content_height(h.todo);
        surface.max(fixed)
    }

    /// The view says whether an overlay holds the pointer: both halves
    /// project inert and take no input.
    fn set_inert(&mut self, inert: bool, todo: &mut TodoPage) {
        todo.set_inert(inert);
        for s in self.host.iter_mut() {
            s.set_inert(inert);
        }
    }

    /// Both halves read the todo's state again when it moved.
    fn sync(&mut self, todo: &TodoPage) {
        if let Ok(now) = todo.snapshot() {
            self.host.sync(&now);
        }
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
        todo: &mut TodoPage,
        inert: bool,
    ) -> EventResult {
        self.set_inert(inert, todo);
        let h = self.halves(content, theme, todo);
        let mut r = todo.handle_event(event, h.todo, theme);
        for s in self.host.iter_mut() {
            r = r.merge(s.handle_event(event, theme, &mut Todo(todo)));
        }
        r
    }

    /// A printable key: the surface's field when it has the keyboard, the
    /// todo's otherwise.
    pub fn handle_text(&mut self, key: &str, todo: &mut TodoPage) -> bool {
        for s in self.host.iter_mut() {
            if s.handle_text(key, &mut Todo(todo)) {
                return true;
            }
        }
        todo.handle_text(key)
    }

    pub fn handle_edit_key(&mut self, key: EditKey, todo: &mut TodoPage) -> bool {
        if key != EditKey::Tab {
            for s in self.host.iter_mut() {
                if s.handle_edit_key(key.into(), &mut Todo(todo)) {
                    return true;
                }
            }
        }
        todo.handle_edit_key(key)
    }

    pub fn handle_enter(&mut self, todo: &mut TodoPage, inert: bool) -> bool {
        self.set_inert(inert, todo);
        for s in self.host.iter_mut() {
            if s.handle_enter(&mut Todo(todo)) {
                return true;
            }
        }
        todo.handle_enter()
    }

    pub fn handle_escape(&mut self, todo: &mut TodoPage) -> bool {
        let surface = self.host.iter_mut().any(|s| s.handle_escape());
        surface || todo.handle_escape()
    }

    /// The caret of either field.
    pub fn wake(&mut self, dt: f32, todo: &mut TodoPage) -> Wake {
        let surface = self
            .host
            .iter_mut()
            .fold(Wake::idle(), |w, s| w.merge(s.wake(dt)));
        todo.wake(dt).merge(surface)
    }

    pub fn render(
        &mut self,
        c: &mut Compositor,
        content: Rect,
        theme: &Theme,
        todo: &mut TodoPage,
        inert: bool,
    ) {
        self.set_inert(inert, todo);
        self.sync(todo);
        let h = self.halves(content, theme, todo);
        let style = caption_style(theme);
        for (at, text) in h.captions.into_iter().zip([FIXED, GENERATED]) {
            c.push(SceneNode::Text {
                key: TextNodeKey::from_style(text, &style, Some(at.w)),
                x: at.x,
                y: at.y,
                color: theme.glass.text_faint.0,
            });
        }
        todo.render(c, h.todo, theme);
        let id = self.id.clone();
        match id.and_then(|id| self.host.get_mut(&id)) {
            Some(s) => s.render(c, h.surface, theme),
            None => {
                let message = self.refused.clone().unwrap_or_default();
                let rect = Rect::new(
                    h.surface.x,
                    h.surface.y,
                    h.surface.w,
                    theme.size.card_min_w / 2.0,
                );
                EmptyState::new("the surface did not open", message).render(c, rect, theme);
            }
        }
    }

    /// The page as the contract sees it: the todo and the surface, one
    /// snapshot.
    pub(crate) fn snapshot(&self, todo: &TodoPage) -> Result<Snapshot, SemErrors> {
        let mut b = SnapshotBuilder::new();
        todo.project(&mut b);
        self.host.project(&mut b);
        b.finish()
    }

    /// A request from an actor outside the page's own input (an agent, a
    /// screen reader): aimed at a node of the surface it is the
    /// capability the IR bound; either way it goes through the todo's one
    /// dispatcher, and the surface then shows what came out.
    pub(crate) fn act(
        &mut self,
        req: Request,
        todo: &mut TodoPage,
        inert: bool,
    ) -> Option<Receipt> {
        self.set_inert(inert, todo);
        let id = todo.dispatch(self.host.route(req));
        self.sync(todo);
        todo.receipt(id).cloned()
    }

    /// Keyboard focus from outside: a node of the surface takes it (its
    /// field, or away from the fields), or the todo does.
    pub(crate) fn focus(&mut self, id: &SemId, todo: &mut TodoPage, inert: bool) -> bool {
        self.set_inert(inert, todo);
        if self.host.iter_mut().any(|s| s.focus(id)) {
            todo.handle_escape();
            return true;
        }
        let taken = todo.focus(id);
        if taken {
            for s in self.host.iter_mut() {
                s.handle_escape();
            }
        }
        taken
    }
}

#[cfg(test)]
mod tests;
