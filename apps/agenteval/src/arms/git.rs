//! arm B, the conventional alternative: the same repository and the same
//! task for an agent that uses `git` on the command line. no workbench,
//! no port, no recording, so the report says "receipts: none; session:
//! none".
//!
//! the equivalent intervention is a `pre-commit` hook the laboratory
//! installs: once, at the instant of the commit, it takes the path out of
//! the index, and git commits what is left without telling the author.
//! a shim of `git` on the agent's PATH writes one json line per
//! invocation (the time, the arguments, the bytes of stdout, the exit
//! code), which is the volume and the round trips of this arm.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::{Duration, Instant};

use serde_json::{Value as Json, json};

use crate::args::{Args, Until};
use crate::repo::{Repo, TASK, TASK_FILES, TASK_MESSAGE};
use crate::report::{self, ERROR, Truth, row};

/// how often the laboratory looks at `.git` for a new HEAD.
const LOOK: Duration = Duration::from_millis(200);

pub fn run(args: &Args) -> i32 {
    match arm(args) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("agenteval: {e}");
            ERROR
        }
    }
}

fn arm(args: &Args) -> Result<i32, String> {
    fs::create_dir_all(&args.out).map_err(|e| format!("{}: {e}", args.out.display()))?;
    let repo = Repo::build()?;
    let head_before = repo.head()?;
    let log = args.out.join("git-log.jsonl");
    let bin = args.out.join("bin");
    let shim = write_shim(&bin, &log)?;
    let hook = match args.human.path() {
        Some(path) => Some(install_hook(&repo, &args.out, path)?),
        None => None,
    };

    // the agent gets the path of this repository: it works on git itself.
    println!("agenteval: arm git");
    println!("task: {TASK}");
    println!("repository: {}", repo.path().display());
    println!("export PATH={}:$PATH", bin.display());
    println!("git shim: {}", shim.display());
    println!("git log of the shim: {}", log.display());
    match &hook {
        Some(hook) => println!(
            "pre-commit hook: {} (once: git reset -q HEAD -- {})",
            hook.display(),
            args.human.path().unwrap_or_default()
        ),
        None => println!("pre-commit hook: none"),
    }
    println!(
        "until: {}, timeout {} s",
        match args.until {
            Until::Commit => "commit",
            Until::Timeout => "timeout",
        },
        args.timeout.as_secs_f64()
    );
    println!("waiting for the agent");

    let started = Instant::now();
    let deadline = started + args.timeout;
    let mut ended_by = "timeout";
    while Instant::now() < deadline {
        sleep(LOOK);
        if args.until == Until::Commit && repo.head().is_ok_and(|head| head != head_before) {
            ended_by = "commit";
            // the hook runs inside the commit; give the agent a moment to
            // react to what git told it.
            sleep(args.grace);
            break;
        }
    }

    let truth = Truth::read(&repo, &head_before, None)?;
    let calls = read_log(&log);
    let summary = summarize(&calls);
    let duration_ms = started.elapsed().as_secs_f64() * 1e3;
    let report = json!({
        "arm": "git",
        "human": args.human.describe(),
        "human_path": args.human.path(),
        "repository": repo.path().display().to_string(),
        "task": TASK,
        "task_files": TASK_FILES,
        "task_message": TASK_MESSAGE,
        "run": { "duration_ms": duration_ms, "ended_by": ended_by },
        "hook": hook.as_ref().map(|h| h.display().to_string()),
        "hook_fired": hook_fired(&args.out),
        "shim": {
            "path": shim.display().to_string(),
            "log": log.display().to_string(),
            "calls": summary.calls,
            "bytes_out": summary.bytes,
            "commands": summary.commands,
            "failed": summary.failed,
            "invocations": calls,
        },
        "receipts": "none",
        "session": "none",
        "truth": truth.json(),
        "notes": [
            "no workbench, no agent port and no recording in this arm: nothing explains the \
             commit afterwards except git itself",
            "the intervention of this arm fires inside git commit, so the agent is told nothing; \
             in the plev arm it fires right after the observation and the submit is refused",
        ],
        "exit": truth.exit_code(),
    });
    report::write(&args.out, &report)?;

    println!();
    report::print_table(
        "run",
        &[
            row("ended by", ended_by),
            row("duration", &format!("{duration_ms:.0} ms")),
            row("hook fired", &hook_fired(&args.out).to_string()),
        ],
    );
    report::print_table(
        "git shim",
        &[
            row("invocations", &summary.calls.to_string()),
            row("bytes of stdout", &summary.bytes.to_string()),
            row("failed invocations", &summary.failed.to_string()),
            row("commands", &summary.commands.join(", ")),
            row("log", &log.display().to_string()),
        ],
    );
    report::print_table(
        "receipts and session",
        &[row("receipts", "none"), row("session", "none")],
    );
    let mut rows = truth.rows();
    rows.push(row("exit", &truth.exit_code().to_string()));
    report::print_table("truth", &rows);
    println!("git log --oneline");
    for line in truth.log.lines() {
        println!("  {line}");
    }
    println!("git status --porcelain");
    for line in truth.status.lines() {
        println!("  {line}");
    }
    if hook.is_some() && truth.committed() && !truth.files_are_the_task() {
        println!(
            "the commit differs from the task: the hook took {} out of the index at the instant \
             of the commit and git said nothing",
            args.human.path().unwrap_or_default()
        );
    }
    println!("report: {}", args.out.join("report.json").display());
    Ok(truth.exit_code())
}

/// the real git, resolved once when the shim is written, so the shim
/// never calls itself.
fn real_git() -> Result<PathBuf, String> {
    let out = std::process::Command::new("/bin/sh")
        .args(["-c", "command -v git"])
        .output()
        .map_err(|e| format!("looking for git: {e}"))?;
    if !out.status.success() {
        return Err("no git on PATH".to_owned());
    }
    let path = PathBuf::from(String::from_utf8_lossy(&out.stdout).trim());
    if path.is_absolute() {
        Ok(path)
    } else {
        Err(format!(
            "git resolved to {}, not an absolute path",
            path.display()
        ))
    }
}

/// writes `<out>/bin/git`, a shim that records every invocation and calls
/// the real git. it is what the agent of this arm runs; both streams go
/// through a file so the bytes are counted without changing them.
fn write_shim(bin: &Path, log: &Path) -> Result<PathBuf, String> {
    fs::create_dir_all(bin).map_err(|e| format!("{}: {e}", bin.display()))?;
    let git = real_git()?;
    let path = bin.join("git");
    let script = format!(
        r#"#!/bin/sh
# the git of arm B: one json line per invocation, then the real git,
# byte for byte. written by agenteval (apps/agenteval/src/arms/git.rs).
real='{git}'
log='{log}'
out=$(mktemp) || exit 1
err=$(mktemp) || exit 1
start=$(date +%s)
"$real" "$@" >"$out" 2>"$err"
code=$?
bytes=$(wc -c <"$out" | tr -d ' ')
errbytes=$(wc -c <"$err" | tr -d ' ')
args=''
for a in "$@"; do
    escaped=$(printf '%s' "$a" | sed 's/\\/\\\\/g; s/"/\\"/g')
    args="$args\"$escaped\","
done
printf '{{"t":%s,"args":[%s],"bytes":%s,"err_bytes":%s,"exit":%s}}\n' \
    "$start" "${{args%,}}" "$bytes" "$errbytes" "$code" >>"$log"
cat "$out"
cat "$err" >&2
rm -f "$out" "$err"
exit $code
"#,
        git = git.display(),
        log = log.display()
    );
    fs::write(&path, script).map_err(|e| format!("{}: {e}", path.display()))?;
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755))
        .map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(path)
}

/// installs the one-shot `pre-commit` hook through `core.hooksPath`: it
/// takes `path` out of the index once, at the instant of the commit.
fn install_hook(repo: &Repo, out: &Path, path: &str) -> Result<PathBuf, String> {
    let dir = out.join("hooks");
    fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let marker = out.join("hook-fired");
    let hook = dir.join("pre-commit");
    let git = real_git()?;
    let script = format!(
        r#"#!/bin/sh
# the intervention of arm B: once, at the instant of the commit, the path
# leaves the index and git commits what is left, telling nobody.
# written by agenteval (apps/agenteval/src/arms/git.rs).
marker={marker}
[ -e "$marker" ] && exit 0
{git} reset -q HEAD -- {path}
date +%s > "$marker"
exit 0
"#,
        marker = marker.display(),
        git = git.display(),
        path = path
    );
    fs::write(&hook, script).map_err(|e| format!("{}: {e}", hook.display()))?;
    fs::set_permissions(&hook, fs::Permissions::from_mode(0o755))
        .map_err(|e| format!("{}: {e}", hook.display()))?;
    repo.git(&["config", "core.hooksPath", &dir.display().to_string()])?;
    Ok(hook)
}

fn hook_fired(out: &Path) -> bool {
    out.join("hook-fired").exists()
}

struct Summary {
    calls: usize,
    bytes: u64,
    failed: usize,
    commands: Vec<String>,
}

/// the invocations the shim recorded, as json values.
fn read_log(log: &Path) -> Vec<Json> {
    let Ok(text) = fs::read_to_string(log) else {
        return Vec::new();
    };
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect()
}

/// git's subcommand: the first argument that is not a global option,
/// skipping the value of the ones that take one (`-C <path>`, `-c
/// <name=value>`, `--git-dir <path>`).
fn verb_of(args: &[Json]) -> &str {
    let mut skip = false;
    for arg in args {
        let Some(arg) = arg.as_str() else { continue };
        if skip {
            skip = false;
            continue;
        }
        if matches!(
            arg,
            "-C" | "-c" | "--git-dir" | "--work-tree" | "--namespace"
        ) {
            skip = true;
            continue;
        }
        if !arg.starts_with('-') {
            return arg;
        }
    }
    "git"
}

fn summarize(calls: &[Json]) -> Summary {
    let mut commands: Vec<String> = Vec::new();
    let mut bytes = 0;
    let mut failed = 0;
    for call in calls {
        bytes += call.get("bytes").and_then(Json::as_u64).unwrap_or(0);
        if call.get("exit").and_then(Json::as_i64).unwrap_or(0) != 0 {
            failed += 1;
        }
        let verb = call
            .get("args")
            .and_then(Json::as_array)
            .map_or("git", |args| verb_of(args))
            .to_owned();
        if !commands.contains(&verb) {
            commands.push(verb);
        }
    }
    Summary {
        calls: calls.len(),
        bytes,
        failed,
        commands,
    }
}
