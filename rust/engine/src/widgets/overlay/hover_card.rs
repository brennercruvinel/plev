//! The hover card: a popover that opens while the pointer rests on its
//! trigger and stays while it moves into the panel, for content richer
//! than a tooltip's line (a person, a link's preview). The kit has no
//! hover card; it is the kit's popover ([`Popover`], its panel and its
//! placement) opened by the tooltip's gesture, with the delays of
//! `theme.overlay.hover_card`.
//!
//! The pointer is not the only way in: where there is no hover (touch, a
//! pen that does not hover), a press of the trigger toggles it, and the
//! keyboard's focus on the trigger opens it at once, as a screen reader's
//! does; Escape closes it. All of these are requests on the one
//! [`Popover`], so its projection and its stack entry are the popover's.

use crate::widgets::overlay::popover::Popover;

/// Where the hover is in its delays.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
enum Wait {
    #[default]
    Idle,
    /// The pointer rests on the trigger: it opens once `open_delay` is
    /// out.
    Opening(f32),
    /// The pointer left trigger and panel: it closes once `close_delay`
    /// is out, unless the pointer comes back.
    Closing(f32),
}

/// A popover opened by hover, with an open and a close delay.
#[derive(Clone, Debug, Default)]
pub struct HoverCard {
    pub popover: Popover,
    wait: Wait,
}

impl HoverCard {
    /// Below its trigger, centered on it.
    pub fn new() -> Self {
        Self {
            popover: Popover::new().align(crate::widgets::overlay::popover::Align::Center),
            wait: Wait::Idle,
        }
    }

    pub fn is_open(&self) -> bool {
        self.popover.is_open()
    }

    /// Where the pointer is: over the trigger, over the open panel. Starts
    /// or cancels the delays; `true` when that changed something.
    pub fn hover(&mut self, on_trigger: bool, on_panel: bool) -> bool {
        let inside = on_trigger || (self.is_open() && on_panel);
        let next = match (self.is_open(), inside, self.wait) {
            (false, true, Wait::Idle) => Wait::Opening(0.0),
            (false, false, Wait::Opening(_)) => Wait::Idle,
            (true, false, Wait::Idle) => Wait::Closing(0.0),
            (true, true, Wait::Closing(_)) => Wait::Idle,
            (_, _, wait) => wait,
        };
        let changed = next != self.wait;
        self.wait = next;
        changed
    }

    /// Advances the delay under way by `dt` seconds against `open_delay`
    /// and `close_delay`. `true` while it waits or the moment it opens or
    /// closes (request a frame).
    pub fn tick(&mut self, dt: f32, open_delay: f32, close_delay: f32) -> bool {
        match self.wait {
            Wait::Idle => false,
            Wait::Opening(t) if t + dt >= open_delay => {
                self.wait = Wait::Idle;
                self.popover.open();
                true
            }
            Wait::Closing(t) if t + dt >= close_delay => {
                self.wait = Wait::Idle;
                self.popover.close();
                true
            }
            Wait::Opening(t) => {
                self.wait = Wait::Opening(t + dt);
                true
            }
            Wait::Closing(t) => {
                self.wait = Wait::Closing(t + dt);
                true
            }
        }
    }

    /// Opens at once: the keyboard's focus, a press where nothing hovers,
    /// a reader's or an agent's request.
    pub fn open(&mut self) -> bool {
        self.wait = Wait::Idle;
        self.popover.open()
    }

    /// Closes at once: Escape, a press outside, a request.
    pub fn close(&mut self) -> bool {
        self.wait = Wait::Idle;
        self.popover.close()
    }

    /// A press of the trigger: open when closed, closed when open.
    pub fn toggle(&mut self) {
        if self.is_open() {
            self.close();
        } else {
            self.open();
        }
    }
}
