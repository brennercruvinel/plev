//! Message: the things a conversation shows (task-104 front C,
//! conversation and feedback, #156): one module per widget, and the
//! front's token slot (`Theme::message`).

pub mod attachment;
pub mod bubble;
pub mod comment;
pub mod compose;
pub mod marker;
// the file #156 names: the Message widget itself, in its own category
#[allow(clippy::module_inception)]
pub mod message;
pub mod new_comment;
pub mod prompt;
pub mod questionnaire;
pub mod quick_comment;
pub mod scroller;
pub mod select_ai;
pub mod tokens;
pub mod tool;
