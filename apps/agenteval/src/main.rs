//! `agenteval`: the laboratory of the plev experiment (task-62, phase c).
//!
//! the ide's `Workbench` without a window, on a real repository built for
//! the task, served to an agent outside through the agent port, recorded
//! by the flight recorder on the workbench's clock, with a scripted person
//! who may move the index right after the agent observed or right before
//! its submit reaches the dispatcher, and who may answer, in the modal,
//! what an agent asks her; at the end the
//! session file is reopened and checked against the real git, and the
//! costs are measured. the laboratory prints what happened, including
//! what breaks the hypothesis. the review arm is the demo 5 of task-80
//! (task-96): a person reviewing three commits with the fixed panel and
//! with the surface her intent generates, recorded and measured.
//!
//! ```text
//! agenteval --arm plev|scripted|git|screenshot|review
//!         [--human none|unstage-after-observe:<path>|unstage-before-submit:<path>]
//!         [--out <dir>] [--until commit|timeout] [--timeout <s>] [--grace <s>] [--app <name>]
//!         [--scripted observe|chain|ask] [--answer none|allow|decline] [--answer-after <ms>]
//! ```
//!
//! unix only: the port is a unix socket.

#[cfg(unix)]
mod answer;
#[cfg(unix)]
mod args;
#[cfg(unix)]
mod asked;
#[cfg(unix)]
mod finish;
#[cfg(unix)]
mod host;
#[cfg(unix)]
mod refused;
// the repository of the experiment lives in the lib (`ide::lab_repo`), so
// the session tests start from the same one; the laboratory names it
// `crate::repo` as before.
#[cfg(unix)]
use idestudio::repository::eval_repo as repo;
#[cfg(unix)]
mod report;
#[cfg(unix)]
mod story;
pub mod arms;
#[cfg(unix)]
mod subject;

#[cfg(unix)]
fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let args = match args::parse(&argv) {
        Ok(args) => args,
        Err(e) => {
            eprintln!("{e}");
            if !e.starts_with("usage") {
                eprintln!("\n{}", args::USAGE);
            }
            std::process::exit(report::ERROR);
        }
    };
    let code = match args.arm {
        args::Arm::Plev | args::Arm::Scripted => lab::run(&args),
        args::Arm::Git => gitarm::run(&args),
        args::Arm::Screenshot => screenshot::run(&args),
        args::Arm::Review => review::run(&args),
    };
    std::process::exit(code);
}

#[cfg(not(unix))]
fn main() {
    eprintln!("agenteval: the agent port is a unix socket, and this platform has none");
    std::process::exit(3);
}
