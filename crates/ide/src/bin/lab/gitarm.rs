//! arm B: not written yet.

use crate::args::Args;
use crate::report::ERROR;

pub fn run(_args: &Args) -> i32 {
    eprintln!("ide-lab: the git arm is not built yet");
    ERROR
}
