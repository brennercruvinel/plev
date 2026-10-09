//! The arms of the experiment: the same task through plev's agent port
//! (`plev`), a script (`scripted`), the conventional git CLI (`git`), a
//! person reviewing (`review`) and a screenshot reader (`screenshot`).

#[cfg(unix)]
pub(super) mod git;
#[cfg(unix)]
pub(super) mod plev;
#[cfg(unix)]
pub(super) mod review;
#[cfg(unix)]
pub(super) mod screenshot;
#[cfg(unix)]
pub(super) mod scripted;
