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
//! past `timeout` it is killed by its pid. nothing is interactive: a
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
use std::time::{Duration, Instant};

use propose::{Model, ModelError};

/// the Claude Code a `ClaudeCli` runs, when `PLEV_CLAUDE` names no other.
pub const PROGRAM: &str = "claude";
/// the variable that names the program to run instead of `claude`: a path
/// or a name on the PATH.
pub const PROGRAM_ENV: &str = "PLEV_CLAUDE";
/// the model a `ClaudeCli` asks. in the probe of phase 3b-i it answered
/// the todo's prompt with an IR the verifier accepted at once, in 24 to
/// 40 s.
pub const MODEL: &str = "sonnet";
/// the longest one call may take: three times the slowest call of the
/// probe (40 s). a proposal makes three calls at most (`Llm::ATTEMPTS`).
pub const TIMEOUT: Duration = Duration::from_secs(120);
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
            calls: Vec::new(),
        }
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

    /// what the calls so far cost, as the child counts it.
    pub fn spent_usd(&self) -> f64 {
        self.calls.iter().filter_map(|c| c.cost_usd).sum()
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
        match run::run(self.command(), &self.program, prompt, self.timeout) {
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
        assert!(cli.calls().is_empty());
        assert_eq!(cli.spent_usd(), 0.0);
    }
}
