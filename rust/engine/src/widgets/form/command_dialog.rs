//! Command dialog: the palette ([`Command`]) on the modal's sheet
//! ([`crate::widgets::feedback::sheet`]) over its scrim, the
//! sheet as wide as a modal in that window and as tall as the palette
//! with the modal's padding around it, centered. Closed, it is the kit's
//! Search (`components/Search/index.tsx`): the box with its `⌘ K` cap,
//! which a press opens. ⌘K (Ctrl+K off macOS) opens and closes it
//! ([`CommandDialog::toggle`], the owner's key), Escape closes it, a
//! press off the sheet closes it, and a command run closes it, as
//! [`Command::ran`] for the owner to act on.
//!
//! Every value is the modal's (`theme.overlay.modal`) and the palette's
//! (`theme.form`).
//!
//! Projected ([`CommandDialog::project`]): closed, the trigger, a
//! `Role::Button` named "Command palette", described by its `⌘ K`, whose
//! `activate` opens it; open, a `Role::Dialog` named "Command palette" with
//! the palette in it, as the palette projects.

use accord::{Role, SemId, SnapshotBuilder};

use crate::drawing::compositor::{Compositor, LayerId, SceneNode};
use crate::styling::theme::Theme;
use crate::widgets::feedback::{Modal, sheet};
use crate::widgets::form::command::Command;
use crate::widgets::form::input_group::{Addon, InputGroup};
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

/// The dialog's name to a reader.
const NAME: &str = "Command palette";

/// A palette that opens over the window.
pub struct CommandDialog {
    pub command: Command,
    /// The closed dialog: the Search with its `⌘ K`.
    pub trigger: InputGroup,
    open: bool,
    sem: Option<SemId>,
}

impl CommandDialog {
    pub fn new(command: Command) -> Self {
        let trigger = InputGroup::new("Search...")
            .start(Addon::Icon("search"))
            .end(Addon::Hint("⌘ K".into()));
        Self {
            command,
            trigger,
            open: false,
            sem: None,
        }
    }

    /// Shown open, as a state.
    pub fn opened(mut self) -> Self {
        self.open();
        self
    }

    /// `id` is the dialog's; the palette is `<id>.palette`, the trigger
    /// `<id>.trigger`.
    pub fn sem(mut self, id: SemId) -> Self {
        if let Ok(palette) = id.child("palette") {
            self.command = self.command.sem(palette);
        }
        if let Ok(trigger) = id.child("trigger") {
            self.trigger = self.trigger.sem(trigger);
        }
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Opens with the caret in the palette's box.
    pub fn open(&mut self) {
        self.open = true;
        self.trigger.input.set_focused(false);
        self.command.search.input.focus();
    }

    /// Closes. `false` when it was not open.
    pub fn close(&mut self) -> bool {
        let was = self.open;
        self.open = false;
        self.command.search.input.set_focused(false);
        was
    }

    /// ⌘K: open when closed, close when open.
    pub fn toggle(&mut self) {
        if !self.close() {
            self.open();
        }
    }

    /// Escape: closes. `false` when it was not open.
    pub fn escape(&mut self) -> bool {
        self.close()
    }

    /// The sheet in `window`: a modal's width there, as tall as the
    /// palette needs, centered.
    pub fn sheet_rect(&self, theme: &Theme, window: Rect) -> Rect {
        let pad = theme.overlay.modal.pad;
        let w = Modal::width(theme, window.w).min(window.w);
        let h = self.command.height_for((w - pad * 2.0).max(0.0), theme) + pad * 2.0;
        let x = window.x + ((window.w - w) / 2.0).max(0.0);
        let y = window.y + ((window.h - h) / 2.0).max(0.0);
        Rect::new(x, y, w, h)
    }

    /// The palette inside `sheet`.
    pub fn palette_rect(&self, theme: &Theme, sheet: Rect) -> Rect {
        let pad = theme.overlay.modal.pad;
        Rect::new(
            sheet.x + pad,
            sheet.y + pad,
            (sheet.w - pad * 2.0).max(0.0),
            (sheet.h - pad * 2.0).max(0.0),
        )
    }

    /// The trigger's height, the Search's.
    pub fn trigger_height(&self, theme: &Theme) -> f32 {
        self.trigger.height(theme)
    }

    /// Closed: the trigger takes the pointer, a press on it opens. Open:
    /// a press off the sheet closes, the rest is the palette's, and a
    /// command run closes.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        window: Rect,
        trigger: Rect,
        theme: &Theme,
    ) -> EventResult {
        if !self.open {
            let r = self.trigger.handle_event(event, trigger, theme);
            if let WidgetEvent::MouseDown { x, y } = *event
                && trigger.contains(x, y)
            {
                self.open();
                return r.merge(EventResult::clicked());
            }
            return r;
        }
        let sheet = self.sheet_rect(theme, window);
        if let WidgetEvent::MouseDown { x, y } = *event
            && !sheet.contains(x, y)
        {
            self.close();
            return EventResult::changed();
        }
        let r = self
            .command
            .handle_event(event, self.palette_rect(theme, sheet), theme);
        if r.clicked && self.command.ran().is_some() {
            self.close();
        }
        r
    }

    /// Enter runs the highlighted command and closes.
    pub fn enter(&mut self) -> bool {
        let ran = self.open && self.command.enter();
        if ran {
            self.close();
        }
        ran
    }

    /// Closed, the trigger at `trigger`; open, the scrim over `window`
    /// and the sheet with the palette.
    pub fn render(&self, c: &mut Compositor, window: Rect, trigger: Rect, theme: &Theme) {
        if !self.open {
            self.trigger.render(c, trigger, theme);
            return;
        }
        c.push(SceneNode::Rect {
            x: window.x,
            y: window.y,
            w: window.w,
            h: window.h,
            color: theme.overlay.modal.scrim.0,
        });
        self.render_in(c, theme, self.sheet_rect(theme, window));
    }

    /// The sheet at `sheet` with the palette, without a scrim: the
    /// dialog on screen, or shown as a state in a page's card.
    pub fn render_in(&self, c: &mut Compositor, theme: &Theme, sheet_at: Rect) {
        if let Some(id) = &self.sem {
            c.bind(id, [sheet_at.x, sheet_at.y, sheet_at.w, sheet_at.h]);
        }
        sheet(c, LayerId::DEFAULT, theme, sheet_at, |_| {});
        self.command
            .render(c, self.palette_rect(theme, sheet_at), theme);
    }

    /// Declares the dialog (see the module docs). It only reads the
    /// widget: the owner runs `open` on the trigger and the palette's own.
    pub fn project(&self, b: &mut SnapshotBuilder, parent: Option<&SemId>) {
        let Some(id) = &self.sem else {
            return;
        };
        if !self.open {
            if let Some(t) = self.trigger.sem_id() {
                let mut node = b.node(t, Role::Button, NAME);
                if let Some(parent) = parent {
                    node.parent(parent);
                }
                node.description("⌘ K").action("activate");
            }
            return;
        }
        let mut node = b.node(id, Role::Dialog, NAME);
        if let Some(parent) = parent {
            node.parent(parent);
        }
        node.action("close");
        if let Some(p) = self.command.sem_id() {
            let name = self.command.search.input.input.placeholder.clone();
            self.command.project(b, p, name, Some(id));
        }
    }
}
