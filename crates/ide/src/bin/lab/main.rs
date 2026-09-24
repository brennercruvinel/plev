//! `ide-lab`: the laboratory of the plev experiment (task-62, phase c).
//!
//! the ide's `Workbench` without a window, on a real repository built for
//! the task, served to an agent outside through the agent port, recorded
//! by the flight recorder on the workbench's clock, with a scripted person
//! who may move the index right after the agent observed or right before
//! its submit reaches the dispatcher, and who may answer, in the modal,
//! what an agent asks her; at the end the
//! session file is reopened and checked against the real git, and the
//! costs are measured. the laboratory prints what happened, including
//! what breaks the hypothesis.
//!
//! ```text
//! ide-lab --arm plev|scripted|git|screenshot
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
mod bench;
#[cfg(unix)]
mod finish;
#[cfg(unix)]
mod gitarm;
#[cfg(unix)]
mod host;
#[cfg(unix)]
mod lab;
#[cfg(unix)]
mod refused;
// the repository of the experiment lives in the lib (`ide::lab_repo`), so
// the session tests start from the same one; the laboratory names it
// `crate::repo` as before.
#[cfg(unix)]
use ide::lab_repo as repo;
#[cfg(unix)]
mod report;
#[cfg(unix)]
mod screenshot;
#[cfg(unix)]
mod scripted;
#[cfg(unix)]
mod story;

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
    };
    std::process::exit(code);
}

#[cfg(not(unix))]
fn main() {
    eprintln!("ide-lab: the agent port is a unix socket, and this platform has none");
    std::process::exit(3);
}
