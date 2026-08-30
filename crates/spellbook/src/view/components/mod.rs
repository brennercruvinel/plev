//! Shared UI components: the modals and fields more than one screen uses.
//!
//! Ports of the web UI's `js/ui/` helpers. Each component is a retained
//! struct owned by a screen: the screen feeds it events, reads its answers,
//! and calls `render` inside its own overlay render pass.
//!
//! Answers flow one way, uniformly across the components: a component NEVER
//! receives a `ScreenCtx`. Instead every `handle_event`/`handle_edit_key`
//! call takes an `out: &mut Vec<ComponentAnswer>` (or returns an action enum
//! the screen reads), and the screen alone decides what an answer becomes -
//! a `Command` through `ctx.send`, a toast, a navigation, or a local state
//! change. The only exception is live search suggestions
//! ([`set_picker::SetPicker`], [`add_card::AddCardModal`]): those need an
//! async `SearchSets`/`SearchCards` round trip driven by typing, so their
//! methods take `&mut ScreenCtx` purely to send commands. No component ever
//! pushes its own toast or navigates.

// The screens that own these components land in the next commit; until then
// nothing references them.
#![allow(dead_code)]

pub mod add_card;
pub mod card_modal;
pub mod confirm;
pub mod field;
pub mod modal;
pub mod search_field;
pub mod set_picker;
