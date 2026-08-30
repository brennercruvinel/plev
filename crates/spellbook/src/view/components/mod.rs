//! Shared UI components: the modals and fields more than one screen uses.
//!
//! Ports of the web UI's `js/ui/` helpers. Each component is a retained
//! struct owned by a screen: the screen feeds it events, reads its answers
//! through an explicit `take_*` or a `ScreenAction`, and calls `render`
//! inside its own render pass.

pub mod add_card;
pub mod card_modal;
pub mod confirm;
pub mod field;
pub mod modal;
pub mod search_field;
pub mod set_picker;
