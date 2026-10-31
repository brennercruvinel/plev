//! Feedback: the things that tell the user what is happening.

mod context_menu;
mod modal;
mod progress;
mod scrollbar;
mod spinner;
mod toast;
mod tooltip;

// task-104 front C, conversation and feedback (#156): the Alert.
pub mod alert;

pub use context_menu::{ContextMenu, MenuEntry};
pub use modal::{Modal, ModalAction, sheet};
pub use progress::ProgressBar;
pub use scrollbar::Scrollbar;
pub use spinner::{Spinner, SpinnerSize};
pub use toast::{Toast, ToastManager};
pub use tooltip::Tooltip;
