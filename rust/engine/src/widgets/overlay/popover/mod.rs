//! The popover of the hoff core kit: a panel of arbitrary content that a
//! trigger opens by a click, a tap or a key, placed by its anchor inside
//! the container that receives it ([`place`]), and closed by the trigger
//! again, by a press outside, or by Escape, which gives the focus back to
//! the trigger.
//!
//! The popover owns no content and no trigger: the app draws what the
//! panel holds inside [`Popover::placement`]'s rect, and any pressable
//! widget is a trigger (the kit's [`PopoverTrigger`], a toolbar icon
//! button, a button). What it owns is whether it is open and its entry in
//! the view's one overlay stack ([`OverlayManager`]): the app asks
//! ([`Popover::open`], [`Popover::close`], [`Popover::toggle`]) and
//! [`Popover::sync`] brings the stack in line, so the request of a
//! person, a screen reader or an agent is one change of state and the
//! stack follows it, and an entry the stack lost (popped from elsewhere)
//! reads as closed.
//!
//! Projected ([`Popover::project`]): the trigger, declared by its own
//! widget, gains its `expanded` state (its own `activate` is the press
//! that opens and closes it, as a screen reader's click is); while open, the panel is a non-modal `Role::Dialog`, `<trigger>.panel`,
//! beside the trigger, with a `close` of its own, under which the app
//! declares the content.
//!
//! No entry or exit animation yet: the panel shows and leaves on the
//! frame it is asked (the kit's 200 ms fade waits for a layer of its own
//! in the view, task-104's full wave), which is also what reduced motion
//! asks.

mod place;
mod render;

pub use place::{Align, Placement, Side, place};
pub use render::{PopoverTrigger, render_panel};

use accord::{NodeBuilder, Role, SemId, SnapshotBuilder};

use crate::styling::theme::Theme;
use crate::widgets::overlay::{OverlayId, OverlayKind, OverlayManager};
use crate::widgets::widget::{Rect, WidgetEvent};

/// An anchored panel's open state and its entry in the overlay stack.
#[derive(Clone, Debug, Default)]
pub struct Popover {
    /// The side asked; the placement may flip it.
    pub side: Side,
    pub align: Align,
    /// Asked open.
    open: bool,
    /// Its entry while on the stack.
    entry: Option<OverlayId>,
}

impl Popover {
    /// Below its anchor, its end aligned with the anchor's end (the
    /// kit's `bottom end`).
    pub fn new() -> Self {
        Self::default()
    }

    pub fn side(mut self, side: Side) -> Self {
        self.side = side;
        self
    }

    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Asks it open. `true` when that changed something.
    pub fn open(&mut self) -> bool {
        !std::mem::replace(&mut self.open, true)
    }

    /// Asks it closed. `true` when that changed something.
    pub fn close(&mut self) -> bool {
        std::mem::replace(&mut self.open, false)
    }

    /// Open when closed, closed when open: a press of its trigger.
    pub fn toggle(&mut self) {
        self.open = !self.open;
    }

    /// A press outside both the trigger and the open panel closes it.
    /// `true` when it did.
    pub fn dismiss(&mut self, event: &WidgetEvent, trigger: Rect, panel: Rect) -> bool {
        let WidgetEvent::MouseDown { x, y } = *event else {
            return false;
        };
        self.open && !trigger.contains(x, y) && !panel.contains(x, y) && self.close()
    }

    /// Brings the stack in line with what was asked: an entry for an open
    /// popover at `panel`, none for a closed one. An entry the stack no
    /// longer holds (another hand popped it, Escape on the stack) closes
    /// the popover. `true` when the stack or the popover changed.
    pub fn sync(&mut self, mgr: &mut OverlayManager, panel: Rect) -> bool {
        let held = self
            .entry
            .filter(|id| mgr.stack.iter().any(|o| o.id == *id && !o.is_closing()));
        match (self.open, self.entry, held) {
            // Lost from the stack: it is closed now.
            (true, Some(_), None) => {
                self.entry = None;
                self.open = false;
                true
            }
            (true, None, _) => {
                let id = mgr.push(OverlayKind::Anchored, panel.x, panel.y, panel.w, panel.h);
                self.entry = Some(id);
                true
            }
            (false, Some(id), held) => {
                if held.is_some() {
                    mgr.pop_id(id);
                }
                self.entry = None;
                true
            }
            _ => false,
        }
    }

    /// Where the panel of `size` goes by `anchor` inside `bounds`, on the
    /// side and alignment asked.
    pub fn placement(
        &self,
        anchor: Rect,
        size: (f32, f32),
        bounds: Rect,
        theme: &Theme,
    ) -> Placement {
        place(
            anchor,
            size,
            bounds,
            self.side,
            self.align,
            theme.overlay.popover.clearance,
        )
    }

    /// The panel's id beside `trigger`: `<trigger>.panel`.
    pub fn panel_id(trigger: &SemId) -> Option<SemId> {
        trigger.child("panel").ok()
    }

    /// Adds the popover to a projection where its trigger, `trigger`, is
    /// already declared: the trigger's `expanded`; while open, the panel `<trigger>.panel`, a non-modal
    /// dialog named `title`, under `parent`, with a `close`. Returns the
    /// panel's node for the app to add to, `None` while closed (or when
    /// the trigger was not declared).
    #[track_caller]
    pub fn project<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        trigger: &SemId,
        title: &str,
        parent: Option<&SemId>,
    ) -> Option<NodeBuilder<'b>> {
        let expanded = format!("{trigger}:expanded");
        b.reopen(trigger)?.state("expanded", self.open);
        if !self.open {
            return None;
        }
        let id = Self::panel_id(trigger)?;
        let mut panel = b.node(&id, Role::Dialog, title);
        if let Some(parent) = parent {
            panel.parent(parent);
        }
        panel.action("close").reads(expanded.as_str());
        Some(panel)
    }
}
