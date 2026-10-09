//! pure domain models behind the showcase tabs: state first, pixels later
//! (agents.md, backend before ui). everything here is gpu-free, window-free
//! and unit tested before any view consumes it.

pub mod dock;
pub mod silences;
#[cfg(test)]
mod tests;
pub mod todo;
