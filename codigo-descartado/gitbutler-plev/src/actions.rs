/// Typed actions emitted by gitbutler-plev components.
///
/// Components emit these via [`plev::dispatch::ActionQueue`]; parents drain
/// and pattern-match to update state.
use plev::dispatch::WidgetAction;

/// Actions emitted by the file list (UnassignedView) and context menu.
#[derive(Debug, Clone, PartialEq)]
pub enum FileAction {
    Stage(String),
    Discard(String),
    Ignore(String),
}

impl WidgetAction for FileAction {}

/// Actions emitted by the confirmation modal.
#[derive(Debug, Clone, PartialEq)]
pub enum ModalAction {
    Confirmed,
    Cancelled,
}

impl WidgetAction for ModalAction {}
