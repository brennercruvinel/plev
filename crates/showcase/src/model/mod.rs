//! pure domain models behind the showcase tabs: state first, pixels later
//! (agents.md, backend before ui). everything here is gpu-free, window-free
//! and unit tested before any view consumes it.

pub mod dock;
pub mod todo;

#[cfg(test)]
mod tests_dock;
#[cfg(test)]
mod tests_todo;
