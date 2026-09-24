//! Apps Demo, Surface: the todo's capabilities as a surface materialized
//! from verified plev IR (task-69), beside the todo page written by hand.
//!
//! The surface has two forms, each an IR verified against the registry
//! the todo projects (`uir::Capabilities::from_snapshot`) and materialized
//! by the `surface` crate: compact, the fixture of task-68
//! (`crates/uir/fixtures/showcase-todo.ir.json`: the field, the add
//! button, the filter as chips, the counters as stats, one under the
//! other), and two columns (`crates/surface/tests/fixtures/showcase-todo-columns.ir.json`:
//! a grid, the filter and the counters in one column and the entry in the
//! other, one column where the width does not hold two). A
//! segmented control above the surface switches the form: the surface
//! transforms in place (`SurfaceHost::transform`), reconciled by `SemId`,
//! so what was typed in its field, the caret and the focus go on, and the
//! filter is the todo's either way; the nodes move to their new places,
//! the born ones fade in and the gone ones fade out.
//!
//! Both halves operate one state through one dispatcher, the todo page's:
//! adding a todo or changing the filter on the generated surface shows on
//! the fixed todo, and back. The page projects the todo and the surface
//! as one snapshot, so an agent or a screen reader observes the surface
//! and acts on it like on any page; an act on one of its nodes is routed
//! to the capability the IR bound (`SurfaceHost::route`).
//!
//! Wide, the two halves sit side by side; narrow, the todo is on top.

use comps::prelude::{EmptyState, EventResult, Rect, Tabs, Wake, WidgetEvent};
use engine::compositor::{Compositor, SceneNode, TextNodeKey};
use engine::text::{TextMeasurer, TextStyle};
use engine::theme::Theme;
use sem::{Receipt, Request, SemErrors, SemId, Semantics, Snapshot, SnapshotBuilder};
use surface::{Dispatch, SurfaceHost};
use uir::{Capabilities, GenerationPolicy, verify_json};

use super::EditKey;
use super::app::TodoPage;

/// One form of the generated surface: what the control calls it, the
/// caption naming its IR, the IR.
struct Form {
    label: &'static str,
    caption: &'static str,
    ir: &'static str,
}

/// The two forms of the todo's surface, the compact one first (the one
/// the page opens with).
const FORMS: [Form; 2] = [
    Form {
        label: "compact",
        caption: "the same capabilities, materialized from showcase-todo.ir.json",
        ir: include_str!("../../../../uir/fixtures/showcase-todo.ir.json"),
    },
    Form {
        label: "two columns",
        caption: "the same capabilities, materialized from showcase-todo-columns.ir.json",
        ir: include_str!("../../../../surface/tests/fixtures/showcase-todo-columns.ir.json"),
    },
];

const FIXED: &str = "the todo page, written by hand";

pub struct SurfacePage {
    host: SurfaceHost,
    /// The surface that opened (the IR's id), when it did.
    id: Option<SemId>,
    /// Why it did not: the verifier's first error.
    refused: Option<String>,
    /// The form the surface shows, an index of `FORMS`.
    form: usize,
    /// The control that switches the form.
    forms: Tabs,
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
/// each under its caption (the rect a caption wraps in), and the control
/// of the surface's form between its caption and the surface.
struct Halves {
    todo: Rect,
    surface: Rect,
    captions: [Rect; 2],
    forms: Rect,
}

fn caption_style(theme: &Theme) -> TextStyle {
    theme.typography.caption_sm()
}

/// The band a caption takes at `width`: its measured lines and a gap.
fn caption_h(text: &str, width: f32, theme: &Theme) -> f32 {
    let (_, h) = TextMeasurer::measure_styled(text, &caption_style(theme), Some(width));
    h + theme.spacing.sm
}

/// The band the surface's caption takes at `width`: the taller of the
/// forms' captions, so that switching the form moves nothing but the
/// surface.
fn generated_h(width: f32, theme: &Theme) -> f32 {
    FORMS
        .iter()
        .map(|f| caption_h(f.caption, width, theme))
        .fold(0.0, f32::max)
}

/// The form control at the top of `band`: as wide as the narrowest card
/// at most, one tab strip tall.
fn forms_rect(band: Rect, theme: &Theme) -> Rect {
    Rect::new(
        band.x,
        band.y,
        band.w.min(theme.size.card_min_w),
        Tabs::height(theme),
    )
}

/// The band the form control takes above the surface.
fn forms_h(theme: &Theme) -> f32 {
    Tabs::height(theme) + theme.spacing.md
}

impl SurfacePage {
    /// Verifies the compact form against what `todo` projects now and
    /// opens it.
    pub fn new(todo: &TodoPage) -> Self {
        let mut page = Self {
            host: SurfaceHost::new(),
            id: None,
            refused: None,
            form: 0,
            forms: Tabs::new(FORMS.iter().map(|f| f.label)),
        };
        let Ok(app) = todo.snapshot() else {
            page.refused = Some("the todo page does not project".to_owned());
            return page;
        };
        let caps = Capabilities::from_snapshot(&app);
        match verify_json(FORMS[0].ir, &caps, &GenerationPolicy::default()) {
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
        let control = forms_h(theme);
        if content.w >= theme.size.card_min_w * 2.0 + gap {
            let w = (content.w - gap) / 2.0;
            let cap = caption_h(FIXED, w, theme).max(generated_h(w, theme));
            let top = content.y + cap;
            let h = (content.h - cap).max(0.0);
            let right = content.x + w + gap;
            let band = Rect::new(right, top, w, h);
            return Halves {
                todo: Rect::new(content.x, top, w, h),
                surface: Rect::new(right, top + control, w, (h - control).max(0.0)),
                captions: [
                    Rect::new(content.x, content.y, w, cap),
                    Rect::new(right, content.y, w, cap),
                ],
                forms: forms_rect(band, theme),
            };
        }
        // narrow: the todo at its smallest height, the surface under it
        let (x, w) = (content.x, content.w);
        let fixed = caption_h(FIXED, w, theme);
        let generated = generated_h(w, theme);
        let todo_h = todo.content_height(Rect::new(x, content.y, w, 0.0));
        let below = content.y + fixed + todo_h + gap;
        let band = Rect::new(x, below + generated, w, content.h);
        Halves {
            todo: Rect::new(x, content.y + fixed, w, todo_h),
            surface: Rect::new(x, band.y + control, w, content.h),
            captions: [
                Rect::new(x, content.y, w, fixed),
                Rect::new(x, below, w, generated),
            ],
            forms: forms_rect(band, theme),
        }
    }

    /// The surface takes the form `to` of `FORMS`, verified against what
    /// the todo projects now, in place: `SurfaceHost::transform`
    /// reconciles it by `SemId`. When the form does not verify (or does
    /// not transform), the surface keeps the one it has, and the control
    /// says so. True when the form changed.
    fn switch(&mut self, to: usize, todo: &TodoPage) -> bool {
        let changed = self.transform(to, todo);
        match &changed {
            Ok(()) => self.form = to,
            Err(e) => log::warn!("the surface keeps its form: {e}"),
        }
        self.forms.active = self.form;
        changed.is_ok()
    }

    fn transform(&mut self, to: usize, todo: &TodoPage) -> Result<(), String> {
        let form = FORMS.get(to).ok_or("no such form")?;
        let app = todo.snapshot().map_err(|e| e.to_string())?;
        let caps = Capabilities::from_snapshot(&app);
        let ir = verify_json(form.ir, &caps, &GenerationPolicy::default())
            .map_err(|errors| errors.first().map_or_else(String::new, ToString::to_string))?;
        self.host
            .transform(ir, &app)
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    fn surface_h(&self, width: f32, theme: &Theme) -> f32 {
        let id = self.id.as_ref();
        match id.and_then(|id| self.host.get(id)) {
            Some(s) => s.height(width, theme),
            None => theme.size.card_min_w / 2.0,
        }
    }

    /// Natural height: the taller half, or both halves stacked (the
    /// surface at the height of the form it goes to).
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
        if !inert && self.id.is_some() {
            let asked = self.forms.handle_event(event, h.forms, theme);
            if asked.clicked {
                self.switch(self.forms.active, todo);
            }
            r = r.merge(asked);
        }
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

    /// The caret of either field, and the surface's change of form while
    /// it runs.
    pub fn wake(&mut self, dt: f32, todo: &mut TodoPage) -> Wake {
        todo.wake(dt).merge(self.host.wake(dt))
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
        let generated = FORMS[self.form].caption;
        for (at, text) in h.captions.into_iter().zip([FIXED, generated]) {
            c.push(SceneNode::Text {
                key: TextNodeKey::from_style(text, &style, Some(at.w)),
                x: at.x,
                y: at.y,
                color: theme.glass.text_faint.0,
            });
        }
        todo.render(c, h.todo, theme);
        let id = self.id.clone();
        if id.is_some() {
            self.forms.render(c, h.forms, theme);
        }
        self.host.draw_closing(c, theme);
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
