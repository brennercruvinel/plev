//! what every arm ends with: the truth read from git, the verdict against
//! what the session says, the exit code, and the report written as json
//! and printed as a table.

use std::path::Path;
use std::time::Duration;

use rec::Stats;
use serde_json::{Value as Json, json};
use wire::Traffic;

use crate::repo::{Repo, is_the_task};

/// exit codes.
pub const DONE: i32 = 0;
pub const DIFFERS: i32 = 1;
pub const NO_COMMIT: i32 = 2;
pub const ERROR: i32 = 3;

/// the session's account against git's.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    /// the completed receipt's commit and files are HEAD's.
    Match,
    /// they differ, or one side has a commit the other does not.
    Mismatch,
    /// no commit was made and no receipt says one was.
    NoCommit,
    /// no session to compare (the git arm).
    None,
}

impl Verdict {
    pub fn name(self) -> &'static str {
        match self {
            Verdict::Match => "MATCH",
            Verdict::Mismatch => "MISMATCH",
            Verdict::NoCommit => "NO_COMMIT",
            Verdict::None => "none",
        }
    }
}

/// what git says at the end, next to what the session says.
pub struct Truth {
    pub head_before: String,
    pub head_after: String,
    pub head_files: Vec<String>,
    pub head_subject: String,
    pub log: String,
    pub status: String,
    /// the commit and files of the completed `commit.submit` receipt read
    /// back from the session file, when there is one.
    pub session: Option<(String, Vec<String>)>,
    pub verdict: Verdict,
}

impl Truth {
    /// reads git now and judges it against `session` (`None` for an arm
    /// with no session: the verdict is `None`).
    pub fn read(
        repo: &Repo,
        head_before: &str,
        session: Option<Option<(String, Vec<String>)>>,
    ) -> Result<Truth, String> {
        let head_after = repo.head()?;
        let head_files = repo.head_files()?;
        let committed = head_after != head_before;
        let (session, verdict) = match session {
            None => (None, Verdict::None),
            Some(None) if !committed => (None, Verdict::NoCommit),
            Some(None) => (None, Verdict::Mismatch),
            Some(Some((sha, files))) => {
                let same = committed && sha == head_after && files == head_files;
                let verdict = if same {
                    Verdict::Match
                } else {
                    Verdict::Mismatch
                };
                (Some((sha, files)), verdict)
            }
        };
        Ok(Truth {
            head_before: head_before.to_owned(),
            head_after,
            head_files,
            head_subject: repo.head_subject()?,
            log: repo.log_oneline()?,
            status: repo.status()?,
            session,
            verdict,
        })
    }

    pub fn committed(&self) -> bool {
        self.head_after != self.head_before
    }

    /// true when a commit exists and records exactly the task's files.
    pub fn files_are_the_task(&self) -> bool {
        self.committed() && is_the_task(&self.head_files)
    }

    /// 0 when the commit is the task's and, with a session, the session
    /// agrees; 1 when a commit exists and differs; 2 without a commit.
    pub fn exit_code(&self) -> i32 {
        if !self.committed() {
            return NO_COMMIT;
        }
        let agrees = matches!(self.verdict, Verdict::Match | Verdict::None);
        if agrees && self.files_are_the_task() {
            DONE
        } else {
            DIFFERS
        }
    }

    pub fn json(&self) -> Json {
        json!({
            "head_before": self.head_before,
            "head_after": self.head_after,
            "committed": self.committed(),
            "head_files": self.head_files,
            "head_subject": self.head_subject,
            "files_are_the_task": self.files_are_the_task(),
            "session_commit": self.session.as_ref().map(|(sha, _)| sha.clone()),
            "session_files": self.session.as_ref().map(|(_, files)| files.clone()),
            "verdict": self.verdict.name(),
            "log": self.log.lines().collect::<Vec<_>>(),
            "status": self.status.lines().collect::<Vec<_>>(),
        })
    }

    /// the lines of the table about the truth.
    pub fn rows(&self) -> Vec<(String, String)> {
        let mut rows = vec![
            row("head before", &self.head_before),
            row("head after", &self.head_after),
            row("head files", &self.head_files.join(", ")),
            row("head subject", &self.head_subject),
            row("files are the task", &self.files_are_the_task().to_string()),
        ];
        match &self.session {
            Some((sha, files)) => {
                rows.push(row("session commit", sha));
                rows.push(row("session files", &files.join(", ")));
            }
            None => rows.push(row("session commit", "none")),
        }
        rows.push(row("session x git", self.verdict.name()));
        rows
    }
}

pub fn row(label: &str, value: &str) -> (String, String) {
    (label.to_owned(), value.to_owned())
}

/// prints rows as `label ....... value`, one per line.
pub fn print_table(title: &str, rows: &[(String, String)]) {
    println!("{title}");
    let width = rows.iter().map(|(l, _)| l.len()).max().unwrap_or(0);
    for (label, value) in rows {
        println!("  {label:<width$}  {value}");
    }
}

/// writes `report` as `<out>/report.json`.
pub fn write(out: &Path, report: &Json) -> Result<(), String> {
    std::fs::create_dir_all(out).map_err(|e| format!("{}: {e}", out.display()))?;
    let path = out.join("report.json");
    let text = serde_json::to_string_pretty(report).map_err(|e| format!("report: {e}"))?;
    std::fs::write(&path, text).map_err(|e| format!("{}: {e}", path.display()))
}

pub fn traffic_json(t: &Traffic) -> Json {
    json!({
        "conns": t.conns,
        "msgs_in": t.msgs_in,
        "msgs_out": t.msgs_out,
        "bytes_in": t.bytes_in,
        "bytes_out": t.bytes_out,
        "observes_snapshot": t.observes_snapshot,
        "observes_delta": t.observes_delta,
        "acts": t.acts,
        "stages_out": t.stages_out,
        "rejected_stale": t.rejected_stale,
        "errors": t.errors,
    })
}

pub fn traffic_rows(t: &Traffic) -> Vec<(String, String)> {
    vec![
        row("connections", &t.conns.to_string()),
        row(
            "messages in / out",
            &format!("{} / {}", t.msgs_in, t.msgs_out),
        ),
        row(
            "bytes in / out",
            &format!("{} / {}", t.bytes_in, t.bytes_out),
        ),
        row(
            "observations snapshot / delta",
            &format!("{} / {}", t.observes_snapshot, t.observes_delta),
        ),
        row("acts", &t.acts.to_string()),
        row("stages out", &t.stages_out.to_string()),
        row("rejected stale", &t.rejected_stale.to_string()),
        row("errors", &t.errors.to_string()),
    ]
}

pub fn stats_json(s: &Stats) -> Json {
    json!({
        "events": s.events,
        "snapshots_full": s.snapshots_full,
        "snapshots_changed": s.snapshots_changed,
        "snapshots_unchanged": s.snapshots_unchanged,
        "rekeys": s.rekeys,
        "clamped": s.clamped,
        "evicted_segments": s.evicted_segments,
        "evicted_unkeyed": s.evicted_unkeyed,
        "over_budget": s.over_budget,
        "frames": s.frames,
        "frames_unchanged": s.frames_unchanged,
        "frame_errors": s.frame_errors,
        "skipped": s.skipped,
        "layers_approximated": s.layers_approximated,
        "chunks_closed": s.chunks_closed,
        "evicted_chunks": s.evicted_chunks,
        "segments": s.segments,
        "chunks": s.chunks,
        "semantic_bytes": s.semantic_bytes,
        "visual_bytes": s.visual_bytes,
        "asset_bytes": s.asset_bytes,
        "bytes": s.bytes,
    })
}

/// microseconds per call, `0` without calls.
pub fn per_call_us(total: Duration, calls: u64) -> f64 {
    if calls == 0 {
        0.0
    } else {
        total.as_secs_f64() * 1e6 / calls as f64
    }
}

pub fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1e3
}

/// `x.y ms` or `x.y us`, whichever reads.
pub fn human(d: Duration) -> String {
    let us = d.as_secs_f64() * 1e6;
    if us >= 1000.0 {
        format!("{:.2} ms", us / 1000.0)
    } else {
        format!("{us:.1} us")
    }
}
