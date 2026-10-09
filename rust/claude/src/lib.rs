//! the Claude as the model of a proposer: `propose::Model` over the
//! `claude -p` of Claude Code, run as a child process.
//!
//! ```text
//! propose::Llm -> ClaudeCli::complete -> claude -p (prompt on stdin)
//!              <- the reply (the json result on stdout) or a ModelError
//! ```
//!
//! the child gets the prompt on its stdin and answers one json object on
//! its stdout (`--output-format json`): the reply, or the error, with the
//! cost and the turns it took. it runs with no tools, no saved session,
//! no MCP server, no settings file, no skill, a short system prompt of its
//! own instead of Claude Code's, in a directory of its own, and without
//! the variables a Claude Code session sets for the processes it runs.
//! past `timeout` it is killed by its pid, and so is it when the call is
//! cancelled from another thread (`Cancel`). nothing is interactive: a
//! child that would ask for a login or a permission fails, and its text
//! becomes the `ModelError`.
//!
//! the network is the child's: this crate opens no socket and holds no
//! key; the child uses the login its Claude Code already has. what the
//! model answers is never trusted here: `propose::propose_verified` is
//! the way out.

mod reply;
mod run;

use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use offers::{Model, ModelError};

/// the Claude Code a `ClaudeCli` runs, when `PLEV_CLAUDE` names no other.
pub const PROGRAM: &str = "claude";
/// the variable that names the program to run instead of `claude`: a path
/// or a name on the PATH.
pub const PROGRAM_ENV: &str = "PLEV_CLAUDE";
/// the model a `ClaudeCli` asks. in the probe of phase 3b-i it answered
/// the todo's prompt with an IR the verifier accepted at once, in 24 to
/// 40 s.
pub const MODEL: &str = "sonnet";
/// the longest one call may take: about twice the slowest call measured
/// (138 s, the Silences review of task-69 phase 4b; 126 s in the checkpoint
/// b). 180 s, twice the 89 s of phase 3b-i, killed three calls of the same
/// review in a row after the prompt asked for each item's picture and
/// named numbers (task-70 phase 2), paid and thrown away. a proposal makes
/// three calls at most (`Llm::ATTEMPTS`).
pub const TIMEOUT: Duration = Duration::from_secs(300);
/// the most one call may spend, as the child counts it: about ten times
/// a call of the probe. the child checks it after its turn, so it caps a
/// session that would go on, and never cuts a call short.
pub const BUDGET_USD: f64 = 0.5;
/// the system prompt, in place of Claude Code's: the prompt of `propose`
/// says what to answer and how, and the child has no tool to talk about.
pub const SYSTEM: &str = "you are called by a program, not by a person. its message is the \
whole task: answer exactly what it asks for, in the form it asks for, with nothing before or \
after it.";
/// the variables a Claude Code session sets for the processes it runs.
/// an app run from such a session would hand them to the child, which is
/// not part of that session: the effort is one of them.
pub const SESSION_VARS: [&str; 11] = [
    "CLAUDECODE",
    "CLAUDE_CODE_ENTRYPOINT",
    "CLAUDE_CODE_SESSION_ID",
    "CLAUDE_CODE_CHILD_SESSION",
    "CLAUDE_CODE_SESSION_ATTENDED",
    "CLAUDE_CODE_MESSAGING_SOCKET",
    "CLAUDE_CODE_MESSAGING_TOKEN",
    "CLAUDE_CODE_EXECPATH",
    "CLAUDE_PID",
    "CLAUDE_EFFORT",
    "AI_AGENT",
];

/// what one call came to.
#[derive(Clone, Debug, PartialEq)]
pub struct Call {
    /// from the child's start to its answer, or to its kill.
    pub took: Duration,
    /// what the call cost, as the child counts it (list prices, also
    /// under a subscription); none when it did not say.
    pub cost_usd: Option<f64>,
    /// the turns the child took: one, since it has no tools.
    pub turns: Option<u64>,
    /// the reply's length in bytes, or why there was none.
    pub reply: Result<usize, ModelError>,
}

/// cancels a call from another thread: the child is killed by its pid,
/// as at the deadline, and the call ends as a `ModelError` that says so;
/// a call not started yet does not start. a clone is the same signal, so
/// a `ClaudeCli` sent to a worker is cancelled through the handle kept
/// behind. once raised it stays raised: a new call takes a new one.
#[derive(Clone, Debug, Default)]
pub struct Cancel(Arc<AtomicBool>);

impl Cancel {
    pub fn new() -> Cancel {
        Cancel::default()
    }

    pub fn cancel(&self) {
        self.0.store(true, Ordering::Relaxed);
    }

    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}

/// the Claude behind `propose::Model`: each `complete` runs `program -p`
/// once, with the flags of `args`.
#[derive(Clone, Debug)]
pub struct ClaudeCli {
    /// the program: `claude`, or what `PLEV_CLAUDE` names.
    pub program: PathBuf,
    /// the model, an alias (`sonnet`, `opus`) or a full name.
    pub model: String,
    /// the longest one call may take; past it the child is killed.
    pub timeout: Duration,
    /// the most one call may spend (`--max-budget-usd`).
    pub budget_usd: f64,
    /// where the child runs: no project of the app is read from it.
    pub dir: PathBuf,
    /// the signal that ends a call before its deadline, shared with every
    /// clone of this `ClaudeCli`.
    pub cancel: Cancel,
    calls: Vec<Call>,
}

impl Default for ClaudeCli {
    fn default() -> ClaudeCli {
        ClaudeCli::new()
    }
}

impl ClaudeCli {
    /// the defaults: `PLEV_CLAUDE` or `claude`, `MODEL`, `TIMEOUT`,
    /// `BUDGET_USD`, the system's temporary directory.
    pub fn new() -> ClaudeCli {
        let program = std::env::var_os(PROGRAM_ENV)
            .filter(|p| !p.is_empty())
            .map_or_else(|| PathBuf::from(PROGRAM), PathBuf::from);
        ClaudeCli {
            program,
            model: MODEL.to_owned(),
            timeout: TIMEOUT,
            budget_usd: BUDGET_USD,
            dir: std::env::temp_dir(),
            cancel: Cancel::new(),
            calls: Vec::new(),
        }
    }

    /// whether the program is there to run, without running it: a
    /// program named with a directory is a file at that path; a bare name
    /// is a file in one of the directories of the PATH. what an app asks
    /// before it offers the Claude at all.
    pub fn exists(&self) -> bool {
        let named_with_dir = self
            .program
            .parent()
            .is_some_and(|dir| !dir.as_os_str().is_empty());
        if named_with_dir {
            return self.program.is_file();
        }
        std::env::var_os("PATH").is_some_and(|path| {
            std::env::split_paths(&path).any(|dir| dir.join(&self.program).is_file())
        })
    }

    /// every argument the program gets: print mode, the prompt read from
    /// stdin, one json result, and nothing the child could reach beyond
    /// the model.
    pub fn args(&self) -> Vec<String> {
        let mut args: Vec<String> = [
            "-p",
            "--output-format",
            "json",
            // no tool: the child answers text and reads no file.
            "--tools",
            "",
            "--no-session-persistence",
            // no settings file (user, project, local): no hooks, no
            // permissions, no memory of the user's.
            "--setting-sources",
            "",
            // no MCP server: none is given.
            "--strict-mcp-config",
            // no skill listed in the prompt.
            "--disable-slash-commands",
            // anything that would ask for a permission is denied.
            "--permission-prompts",
            "none",
            "--system-prompt",
            SYSTEM,
            "--model",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect();
        args.push(self.model.clone());
        args.push("--max-budget-usd".to_owned());
        args.push(self.budget_usd.to_string());
        args
    }

    /// every call so far, in order.
    pub fn calls(&self) -> &[Call] {
        &self.calls
    }

    /// what the calls so far cost, as the child counts it: zero before the
    /// first, and only what the calls said.
    pub fn spent_usd(&self) -> f64 {
        self.calls
            .iter()
            .filter_map(|c| c.cost_usd)
            .fold(0.0, |all, c| all + c)
    }

    fn command(&self) -> Command {
        let mut command = Command::new(&self.program);
        command.args(self.args()).current_dir(&self.dir);
        for var in SESSION_VARS {
            command.env_remove(var);
        }
        command
    }

    fn ask(&self, prompt: &str) -> reply::Answer {
        let program = self.program.display().to_string();
        match run::run(
            self.command(),
            &self.program,
            prompt,
            self.timeout,
            &self.cancel,
        ) {
            Ok(exit) => reply::read(&program, exit.status, &exit.stdout, &exit.stderr),
            Err(e) => reply::Answer {
                reply: Err(e),
                cost_usd: None,
                turns: None,
            },
        }
    }
}

impl Model for ClaudeCli {
    /// runs the child once with `prompt` on its stdin: its reply, or why
    /// there is none (it could not start, it took longer than `timeout`
    /// and was killed, it failed, or it answered no result).
    fn complete(&mut self, prompt: &str) -> Result<String, ModelError> {
        let started = Instant::now();
        let answer = self.ask(prompt);
        self.calls.push(Call {
            took: started.elapsed(),
            cost_usd: answer.cost_usd,
            turns: answer.turns,
            reply: answer.reply.as_ref().map(String::len).map_err(Clone::clone),
        });
        answer.reply
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// the flags the probe chose, each once, the empty lists as empty
    /// arguments, and the model and budget the fields say.
    #[test]
    fn the_args_are_the_probes() {
        let mut cli = ClaudeCli::new();
        cli.model = "opus".to_owned();
        cli.budget_usd = 0.25;
        let args = cli.args();
        let after = |flag: &str| {
            let at = args.iter().position(|a| a == flag).expect(flag);
            args[at + 1].as_str()
        };
        assert_eq!(args[0], "-p");
        assert_eq!(after("--output-format"), "json");
        assert_eq!(after("--tools"), "");
        assert_eq!(after("--setting-sources"), "");
        assert_eq!(after("--permission-prompts"), "none");
        assert_eq!(after("--system-prompt"), SYSTEM);
        assert_eq!(after("--model"), "opus");
        assert_eq!(after("--max-budget-usd"), "0.25");
        for flag in [
            "--no-session-persistence",
            "--strict-mcp-config",
            "--disable-slash-commands",
        ] {
            assert_eq!(args.iter().filter(|a| *a == flag).count(), 1, "{flag}");
        }
        assert!(
            !args
                .iter()
                .any(|a| a.contains("--mcp-config") || a.contains("--allowed"))
        );
    }

    #[test]
    fn the_defaults() {
        let cli = ClaudeCli::default();
        assert_eq!(cli.model, MODEL);
        assert_eq!(cli.timeout, TIMEOUT);
        assert_eq!(cli.budget_usd, BUDGET_USD);
        assert_eq!(cli.dir, std::env::temp_dir());
        assert!(!cli.cancel.is_cancelled());
        assert!(cli.calls().is_empty());
        assert!(cli.spent_usd().is_sign_positive() && cli.spent_usd() == 0.0);
    }

    /// a clone shares the signal; a new one does not.
    #[test]
    fn a_cancel_is_shared_by_its_clones() {
        let cli = ClaudeCli::new();
        let mut worker = cli.clone();
        cli.cancel.cancel();
        assert!(worker.cancel.is_cancelled());
        worker.cancel = Cancel::new();
        assert!(!worker.cancel.is_cancelled());
        assert!(cli.cancel.is_cancelled());
    }

    /// the program is looked for where it says, never run: a path with a
    /// directory at that path, a bare name on the PATH.
    #[test]
    #[cfg(unix)]
    fn exists_looks_for_the_program_without_running_it() {
        let mut cli = ClaudeCli::new();
        cli.program = PathBuf::from("sh");
        assert!(cli.exists(), "sh is on the PATH");
        cli.program = PathBuf::from("no-such-claude-anywhere");
        assert!(!cli.exists());
        cli.program = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fakes/claude");
        assert!(cli.exists(), "a file at its path");
        cli.program = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/no-such-claude");
        assert!(!cli.exists());
        cli.program = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");
        assert!(!cli.exists(), "a directory is not a program");
    }
}
