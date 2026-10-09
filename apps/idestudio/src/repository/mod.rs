//! The git side: the repository the workbench reads and acts on, its status,
//! the watcher that wakes the window, the adapters to the views and the
//! experiment's throwaway repository.

pub mod adapters;
pub mod eval_repo;
pub mod git;
pub mod status;
