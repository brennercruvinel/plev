//! the rerun's report: every request of the excerpt, its verdict and
//! terminal stage as recorded and as rerun, and every mark as read; when
//! two stages say the same, and how a stage is said in words.

use std::fmt;

use accord::{Change, Rejection, RequestId, Stage};

/// how a recorded stage and a rerun one compared.
#[derive(Clone, Debug, PartialEq)]
pub struct Compared {
    pub recorded: String,
    pub rerun: String,
    pub same: bool,
}

/// one request of the excerpt, recorded and rerun.
#[derive(Clone, Debug, PartialEq)]
pub struct RequestLine {
    pub origin: u16,
    pub id: RequestId,
    /// the id the rerun's dispatcher gave it.
    pub rerun: Option<RequestId>,
    /// `<action> on <target> by <actor>`.
    pub what: String,
    pub verdict: Compared,
    pub terminal: Compared,
    /// an act the rerun could not make (a confirmation the rerun's request
    /// did not wait for).
    pub refused_act: Option<String>,
}

impl RequestLine {
    pub fn same(&self) -> bool {
        self.verdict.same && self.terminal.same && self.refused_act.is_none()
    }
}

/// one mark, read in the rerun.
#[derive(Clone, Debug, PartialEq)]
pub struct MarkLine {
    pub line: usize,
    pub text: String,
    pub found: String,
    pub same: bool,
}

/// the rerun of an excerpt: every request and every mark, compared.
#[derive(Clone, Debug, PartialEq)]
pub struct Report {
    /// the build that recorded the excerpt.
    pub recorded_by: String,
    pub requests: Vec<RequestLine>,
    pub marks: Vec<MarkLine>,
}

impl Report {
    /// true when any request or mark differs from what the recording and
    /// the marks say.
    pub fn diverged(&self) -> bool {
        self.requests.iter().any(|r| !r.same()) || self.marks.iter().any(|m| !m.same)
    }
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bad_requests = self.requests.iter().filter(|r| !r.same()).count();
        let bad_marks = self.marks.iter().filter(|m| !m.same).count();
        writeln!(
            f,
            "rerun of an excerpt recorded by {}: {} requests, {} marks",
            self.recorded_by,
            self.requests.len(),
            self.marks.len()
        )?;
        for r in &self.requests {
            let rerun = r
                .rerun
                .map_or("never dispatched".to_owned(), |id| format!("rerun as {id}"));
            let head = format!(
                "request {} (origin {}, {rerun}): {}",
                r.id, r.origin, r.what
            );
            if r.same() {
                writeln!(f, "  same      {head}: {}", r.terminal.recorded)?;
                continue;
            }
            writeln!(f, "  DIVERGED  {head}")?;
            for (name, c) in [("verdict", &r.verdict), ("terminal", &r.terminal)] {
                let mark = if c.same { "" } else { "  <- differs" };
                writeln!(f, "            {name:<9} recorded  {}", c.recorded)?;
                writeln!(f, "                      rerun     {}{mark}", c.rerun)?;
            }
            if let Some(why) = &r.refused_act {
                writeln!(f, "            act       {why}")?;
            }
        }
        for m in &self.marks {
            if m.same {
                writeln!(f, "  held      line {}: {}", m.line, m.text)?;
            } else {
                writeln!(f, "  DIVERGED  line {}: {}", m.line, m.text)?;
                writeln!(f, "            found     {}", m.found)?;
            }
        }
        if self.diverged() {
            write!(
                f,
                "the rerun diverged: {bad_requests} request(s), {bad_marks} mark(s)"
            )
        } else {
            write!(f, "nothing diverged")
        }
    }
}

/// the rejection's kind, as serde names it.
fn kind(why: &Rejection) -> &'static str {
    match why {
        Rejection::NotFound { .. } => "not_found",
        Rejection::UnknownAction { .. } => "unknown_action",
        Rejection::NotAuthorized { .. } => "not_authorized",
        Rejection::Stale { .. } => "stale",
        Rejection::Inert { .. } => "inert",
        Rejection::Disabled { .. } => "disabled",
        Rejection::InvalidParam { .. } => "invalid_param",
        Rejection::InvalidProjection { .. } => "invalid_projection",
        Rejection::Declined { .. } => "declined",
    }
}

/// the reads a stale refusal names, in the order of the action's reads.
pub fn stale_reads(changes: &[Change]) -> Vec<String> {
    changes
        .iter()
        .map(|c| match c {
            Change::StateChanged { at, .. } => at.to_string(),
            other => format!("{other:?}"),
        })
        .collect()
}

/// a stage in words: `accepted`, `rejected: stale, changes:staged (removed
/// src/session.rs)`, `rejected: disabled`.
pub fn describe(stage: Option<&Stage>) -> String {
    let Some(stage) = stage else {
        return "none".to_owned();
    };
    let Stage::Rejected(why) = stage else {
        return stage.name().to_owned();
    };
    let Rejection::Stale { changes, .. } = why else {
        return format!("rejected: {}", kind(why));
    };
    let reads: Vec<String> = changes
        .iter()
        .map(|c| match c {
            Change::StateChanged {
                at, added, removed, ..
            } => {
                let mut items = Vec::new();
                if !added.is_empty() {
                    items.push(format!("added {}", added.join(", ")));
                }
                if !removed.is_empty() {
                    items.push(format!("removed {}", removed.join(", ")));
                }
                if items.is_empty() {
                    at.to_string()
                } else {
                    format!("{at} ({})", items.join("; "))
                }
            }
            other => format!("{other:?}"),
        })
        .collect();
    format!("rejected: stale, {}", reads.join(", "))
}

/// true when two stages say the same: the same stage and, for a refusal,
/// the same kind, and for a stale one the same reads.
pub fn alike(a: Option<&Stage>, b: Option<&Stage>) -> bool {
    match (a, b) {
        (Some(Stage::Rejected(x)), Some(Stage::Rejected(y))) => match (x, y) {
            (Rejection::Stale { changes: p, .. }, Rejection::Stale { changes: q, .. }) => {
                stale_reads(p) == stale_reads(q)
            }
            _ => kind(x) == kind(y),
        },
        (Some(x), Some(y)) => x.name() == y.name(),
        (None, None) => true,
        _ => false,
    }
}

/// the stage right after `requested`: the verdict.
pub fn verdict<'a>(mut stages: impl Iterator<Item = &'a Stage>) -> Option<&'a Stage> {
    stages.nth(1)
}
