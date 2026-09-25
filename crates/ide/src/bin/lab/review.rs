//! the review arm: the demo 5 of task-80 (task-96), a person reviewing
//! the three newest commits of the laboratory's repository of the review,
//! with no window, first with the fixed panel (the stacks and the diff),
//! then with the surface her intent generates in the column beside it,
//! which she closes at the end. no port and no agent: the person of the
//! script clicks in the middle of each control as the frame on screen
//! draws it (found by its id among the frame's bindings, never at a blind
//! point, through `Workbench::click`, as her answer of task-93 does) and
//! types her intent into the line; every act is a request through the
//! workbench's one dispatcher.
//!
//! recorded by the flight recorder on the workbench's clock: the receipts,
//! the state and every frame with its bindings, her input as its kind and
//! the control under it (never what she typed; the intent reaches the
//! recording as the line's projected state, as the demo 4 found). saved
//! as `review.monster` in the out dir, read back, and measured against
//! the criterion written in task-80 before this arm existed: the acts and
//! the time to see the files and the diff of the three; what shows
//! without scrolling, by the bindings of the frame; every act of the
//! surface in the dispatcher with her receipt; and nothing of the surface
//! after it closes. with a GPU the frames at the three moments are also
//! written as PNG, to look at.
//!
//! since task-97 the review shows the diff in a `code` (the lines and the
//! kind of each), and its lines on screen are counted by the code block's
//! geometry; each side also says how many of the lines on screen are cut
//! at the width, the panel's by its own gutters, the code's by its
//! padding. the criterion is the one written before.

use std::path::{Path, PathBuf};

use comps::content::{CodeBlock, CodeLine};
use comps::nav::PanelHeader;
use engine::text::TextMeasurer;
use engine::theme::Theme;
use git::GitEvent;
use ide::lab_repo::REVIEWED;
use rec::{InputKind, Kind, Limits, Recorder, Session};
use sem::{Actor, Millis, SemId, Stage, Value};
use serde_json::{Value as Json, json};

use crate::args::Args;
use crate::bench::Bench;
use crate::host::Log;
use crate::repo::Repo;
use crate::report::{self, DONE, ERROR, row};

/// what the person writes in the intent line.
pub const INTENT: &str = "review these three commits";

/// the session's file in the out dir.
const SESSION: &str = "review.monster";

/// the surface the rules open for the intent (a fresh id of theirs).
const SURFACE: &str = "proposed";

pub fn run(args: &Args) -> i32 {
    match demo(args) {
        Ok(()) => DONE,
        Err(e) => {
            eprintln!("ide-lab: {e}");
            ERROR
        }
    }
}

/// one commit of the review, as the stacks project it.
struct Commit {
    id: SemId,
    subject: String,
    files: Vec<String>,
}

struct Demo {
    app: Bench,
    rec: Recorder,
    log: Log,
    out: PathBuf,
    /// the PNG frames written, with a GPU.
    shots: Vec<String>,
    /// the person's clicks and keystrokes so far.
    clicks: usize,
    keys: usize,
}

fn id(text: &str) -> Result<SemId, String> {
    SemId::parse(text).map_err(|e| format!("{text}: {e}"))
}

fn millis(t: u64) -> Json {
    json!(t)
}

/// how many of `texts` are wider than `room` in the mono step: the ones
/// a view cuts at its width.
fn cut(texts: &[&str], room: f32, theme: &Theme) -> usize {
    let style = theme.typography.mono();
    texts
        .iter()
        .filter(|t| TextMeasurer::measure_styled(t, &style, None).0 > room + 0.5)
        .count()
}

/// the lines of the fixed diff panel at `[x, _, w, _]` cut at its width,
/// of the first `on_screen`: the room its code column has once the two
/// gutters of line numbers and the mark's column are taken
/// (`crates/ide/src/views/diff_view.rs`, `render`).
fn panel_cut(app: &Bench, rect: [f32; 4], on_screen: usize, theme: &Theme) -> usize {
    let lines = &app.bench.workspace().diff.lines;
    let widest = lines
        .iter()
        .flat_map(|l| [l.line_no_old, l.line_no_new])
        .flatten()
        .max()
        .unwrap_or(0);
    let numbers = theme.typography.small_r();
    let (num_w, _) = TextMeasurer::measure_styled(&widest.to_string(), &numbers, None);
    let pad = theme.spacing.md;
    let gutter = num_w + theme.spacing.sm;
    let code_x = rect[0] + pad + gutter * 2.0 + theme.spacing.md;
    let room = (rect[0] + rect[2] - pad - code_x).max(0.0);
    let texts: Vec<&str> = lines
        .iter()
        .take(on_screen)
        .map(|l| l.content.as_str())
        .collect();
    cut(&texts, room, theme)
}

impl Demo {
    /// the state as projected now and the frame as drawn now, both
    /// recorded at the workbench's clock.
    fn record(&mut self) -> Result<(), String> {
        let now = self.app.bench.now();
        let snap = self
            .app
            .bench
            .snapshot()
            .map_err(|e| format!("the projection does not validate: {e}"))?;
        self.rec.snapshot(now, &snap);
        self.app.frame();
        self.rec
            .frame(now, &self.app.compositor)
            .map_err(|e| format!("the frame is not recorded: {e}"))
    }

    /// the frame on screen as PNG, `<name>.png` in the out dir, when the
    /// machine has a GPU.
    fn shot(&mut self, name: &str) -> Result<(), String> {
        let clear = self.app.clear();
        let Some(gpu) = self.app.gpu.as_mut() else {
            return Ok(());
        };
        self.app.compositor.begin_frame();
        self.app.bench.render(&mut self.app.compositor);
        let frame = gpu
            .render(&mut self.app.compositor, clear)
            .map_err(|e| format!("the frame {name}: {e}"))?;
        let png = frame.encode_png().map_err(|e| format!("{name}: {e}"))?;
        let path = self.out.join(format!("{name}.png"));
        std::fs::write(&path, png).map_err(|e| format!("{}: {e}", path.display()))?;
        self.shots.push(path.display().to_string());
        Ok(())
    }

    /// the visible rect of `control` in the frame on screen, whole: `None`
    /// when it is not drawn, or only part of it shows (half a pixel is the
    /// rounding of the clip, not a part cut).
    fn whole(&self, control: &SemId) -> Option<[f32; 4]> {
        let b = self.app.compositor.binding(control)?;
        let shown = b.visible_rect()?;
        let cut = shown.iter().zip(b.rect).any(|(s, r)| (s - r).abs() > 0.5);
        (!cut).then_some(shown)
    }

    /// the person clicks in the middle of `control` as the frame on screen
    /// draws it; her input is recorded, with the control under the
    /// pointer, as the window records it; the state and the frame after.
    fn click(&mut self, control: &SemId) -> Result<Millis, String> {
        let [x, y, w, h] = self
            .app
            .compositor
            .binding(control)
            .and_then(|b| b.visible_rect())
            .ok_or_else(|| format!("{control} is not on screen"))?;
        let at = [x + w / 2.0, y + h / 2.0];
        let under = self.app.compositor.hit(at[0], at[1]).cloned();
        if under.as_ref() != Some(control) {
            return Err(format!("the middle of {control} is {under:?}"));
        }
        let t = self.app.bench.now();
        self.rec.input(t, InputKind::PointerDown, Some(at), under);
        self.app.bench.click(at[0], at[1]);
        self.clicks += 1;
        self.record()?;
        Ok(t)
    }

    /// the person types `text`, one key at a time, into the field that
    /// has the keyboard: each key recorded as text, never the character.
    fn type_text(&mut self, text: &str) -> Result<(), String> {
        for c in text.chars() {
            let t = self.app.bench.now();
            self.rec.input(t, InputKind::Text, None, None);
            if !self.app.bench.type_text(&c.to_string()) {
                return Err("no field has the keyboard".to_owned());
            }
            self.keys += 1;
        }
        self.record()
    }

    /// waits for the worker's diff of the commit asked last, then records.
    fn diff_arrived(&mut self) -> Result<Millis, String> {
        self.app
            .until(|e| matches!(e, GitEvent::DiffCommit { .. }))?;
        self.record()?;
        Ok(self.app.bench.now())
    }

    fn value(&self, at: &str) -> Result<Value, String> {
        let snap = self.app.bench.snapshot().map_err(|e| e.to_string())?;
        let at = at.parse().map_err(|e| format!("{at}: {e}"))?;
        snap.value(&at)
            .cloned()
            .ok_or_else(|| format!("nothing projected at {at}"))
    }

    fn lines(&self, at: &str) -> Result<Vec<String>, String> {
        match self.value(at)? {
            Value::List(lines) => Ok(lines),
            other => Err(format!("{at} is {other:?}")),
        }
    }

    /// the three commits of the review, as the stacks project them.
    fn commits(&self) -> Result<Vec<Commit>, String> {
        let app = self.app.bench.app_snapshot().map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        for node in app.children(&id("stacks")?).take(REVIEWED.len()) {
            let files = match node.state.iter().find(|(k, _)| k.as_str() == "files") {
                Some((_, Value::List(files))) => files.clone(),
                _ => return Err(format!("{} carries no files", node.id)),
            };
            out.push(Commit {
                id: node.id.clone(),
                subject: node.label.clone(),
                files,
            });
        }
        Ok(out)
    }

    /// the three commits' cards whole in the frame on screen.
    fn cards(&self, commits: &[Commit], prefix: &str) -> Result<usize, String> {
        let mut n = 0;
        for c in commits {
            n += usize::from(self.whole(&id(&format!("{prefix}{}", c.id))?).is_some());
        }
        Ok(n)
    }

    /// the control: the person clicks each commit's card in the stacks
    /// panel and reads its diff in the diff panel. what shows of each:
    /// the files the diff names on screen (a hunk header names its file
    /// only when the commit changes more than one) and how many of its
    /// lines fit the panel, by the panel's own geometry.
    fn fixed(&mut self, commits: &[Commit]) -> Result<Json, String> {
        let theme = self.app.bench.workspace().theme().clone();
        let header = PanelHeader::new("").height(&theme);
        let line = theme.typography.mono().line_height;
        let started = self.app.bench.now();
        let mut each = Vec::new();
        let mut last = started;
        let mut at_once = 0;
        for c in commits {
            let clicked = self.click(&c.id)?;
            last = self.diff_arrived()?;
            let lines = self.lines("diff:lines")?;
            let panel = self
                .whole(&id("diff")?)
                .ok_or("the diff panel is not whole on screen")?;
            let fit = ((panel[3] - header) / line).floor().max(0.0) as usize;
            let on_screen = lines.len().min(fit);
            let cut_on_screen = panel_cut(&self.app, panel, on_screen, &theme);
            let named: Vec<&String> = c
                .files
                .iter()
                .filter(|f| {
                    lines
                        .iter()
                        .take(on_screen)
                        .any(|l| l.starts_with(&format!("{f}: ")))
                })
                .collect();
            // one diff shows at a time: the commits whose files all show
            // now are this one, when its diff names them all, or none
            at_once = at_once.max(usize::from(named.len() == c.files.len()));
            each.push(json!({
                "commit": c.id.to_string(),
                "subject": c.subject,
                "clicked_ms": clicked,
                "diff_ms": last,
                "files": c.files,
                "files_named_on_screen": named,
                "diff_lines": lines.len(),
                "diff_lines_on_screen": on_screen,
                "diff_lines_cut_at_the_width": cut_on_screen,
                "cards_whole": self.cards(commits, "")?,
            }));
        }
        self.shot("fixed")?;
        Ok(json!({
            "acts": { "clicks": commits.len(), "keys": 0 },
            "ms_to_all": last - started,
            "files_at_once": at_once,
            "commits": each,
        }))
    }

    /// the surface: the person clicks the intent field, types the intent,
    /// clicks propose; then each commit's "show diff" on the surface, and
    /// reads the diff beside the three. what shows: the files table of
    /// each commit, and how many lines of the diff the code shows, by the
    /// code block's geometry (its rows area over one row's height), and
    /// how many of those its width cuts.
    fn surface(&mut self, commits: &[Commit]) -> Result<Json, String> {
        let theme = self.app.bench.workspace().theme().clone();
        let (clicks, keys) = (self.clicks, self.keys);
        let started = self.click(&id("ide_surface.intent")?)?;
        self.type_text(INTENT)?;
        self.click(&id("ide_surface.propose")?)?;
        let opened = self.app.bench.now();
        // what the proposal took the dispatcher, by its receipt: the rules
        // run inside `execute`, between its first and its last stage
        let proposed = self
            .log
            .receipts()
            .into_iter()
            .rev()
            .find(|r| r.request.action.as_str() == "propose")
            .ok_or("no proposal was asked")?;
        let first = proposed.stages.first().map_or(0, |(_, t)| *t);
        let last_stage = proposed.stages.last().map_or(0, |(_, t)| *t);
        if self
            .app
            .bench
            .surface_open()
            .map(ToString::to_string)
            .as_deref()
            != Some(SURFACE)
        {
            return Err(format!(
                "the surface did not open: {:?}",
                self.value("ide_surface.status:message")
            ));
        }
        self.shot("surface")?;
        let mut files_whole = 0;
        let mut shown_files = Vec::new();
        for c in commits {
            let table = id(&format!("{SURFACE}.{}.files", c.id))?;
            let rows = self.lines(&format!("{table}:rows"))?;
            if self.whole(&table).is_some() {
                files_whole += 1;
                shown_files.push(json!({ "commit": c.id.to_string(), "files": rows }));
            }
        }
        let cards = self.cards(commits, &format!("{SURFACE}."))?;
        let mut each = Vec::new();
        let mut last = opened;
        for c in commits {
            let button = id(&format!("{SURFACE}.{}.show_diff", c.id))?;
            let clicked = self.click(&button)?;
            last = self.diff_arrived()?;
            let code = id(&format!("{SURFACE}.diff.lines"))?;
            let lines = self.lines(&format!("{code}:lines"))?;
            let kinds = self.lines(&format!("{code}:kinds"))?;
            let [x, y, w, h] = self
                .whole(&code)
                .ok_or("the diff's code is not whole on screen")?;
            let block = CodeBlock::lines(vec![CodeLine::default()]).caption("lines");
            let area = block.rows_rect(comps::Rect::new(x, y, w, h), &theme);
            // half a pixel is the layout's rounding of the block, not a
            // row cut (as `whole` takes it)
            let fit = ((area.h + 0.5) / CodeBlock::row_h(&theme)).floor().max(0.0) as usize;
            let on_screen = lines.len().min(fit);
            let shown: Vec<&str> = lines.iter().take(on_screen).map(String::as_str).collect();
            let room = w - theme.spacing.md * 2.0;
            each.push(json!({
                "commit": c.id.to_string(),
                "clicked_ms": clicked,
                "diff_ms": last,
                "diff_lines": lines.len(),
                "diff_lines_on_screen": on_screen,
                "diff_lines_cut_at_the_width": cut(&shown, room, &theme),
                "kinds_beside_the_lines": kinds.len() == lines.len(),
                "target": match self.value("diff:target")? {
                    Value::Text(t) => t,
                    other => format!("{other:?}"),
                },
            }));
        }
        self.shot("surface-diff")?;
        Ok(json!({
            "intent": INTENT,
            "acts": { "clicks": self.clicks - clicks, "keys": self.keys - keys },
            "ms_to_all": last - started,
            "propose_ms": last_stage - first,
            "opened_ms": last_stage,
            "cards_whole": cards,
            "files_at_once": files_whole,
            "files_on_screen": shown_files,
            "commits": each,
        }))
    }

    /// the person closes the surface; what is left of it in the state, in
    /// the frame, and in the host.
    fn close(&mut self) -> Result<Json, String> {
        let closed = self.click(&id("ide_surface.close")?)?;
        let snap = self.app.bench.snapshot().map_err(|e| e.to_string())?;
        let nodes = snap.nodes().keys().filter(|k| under(k.as_str())).count();
        let bound = self
            .app
            .compositor
            .bindings()
            .filter(|(k, _)| under(k.as_str()))
            .count();
        let host = self.app.bench.surfaces().map_or(0, |h| h.len());
        // what fades out after the close is drawn, never bound: a few
        // frames later nothing of it is drawn either
        for _ in 0..3 {
            self.app.bench.wake(0.25);
            self.record()?;
        }
        self.shot("closed")?;
        Ok(json!({
            "closed_ms": closed,
            "nodes_after": nodes,
            "bindings_after": bound,
            "surfaces_in_host": host,
        }))
    }

    /// every request the dispatcher took, with who asked, what, and how
    /// it ended.
    fn requests(&self) -> Vec<Json> {
        self.log
            .receipts()
            .iter()
            .map(|r| {
                let ended = match r.stage() {
                    Some(Stage::Completed(_)) => "completed".to_owned(),
                    Some(other) => other.name().to_owned(),
                    None => "open".to_owned(),
                };
                json!({
                    "id": r.id.to_string(),
                    "actor": r.request.actor.to_string(),
                    "target": r.request.target.to_string(),
                    "action": r.request.action.to_string(),
                    "ended": ended,
                })
            })
            .collect()
    }
}

/// true for an id of the surface.
fn under(id: &str) -> bool {
    id == SURFACE || id.starts_with(&format!("{SURFACE}."))
}

/// what the session file holds of the surface: the first and the last
/// recorded state that has it, and whether the last frame binds any of
/// it.
fn read_back(path: &Path) -> Result<Json, String> {
    let session = Session::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let (t0, t1) = session.span().ok_or("the session is empty")?;
    let mut with = Vec::new();
    let states = session
        .events()
        .iter()
        .filter(|e| matches!(e.kind, Kind::Snapshot { .. } | Kind::Changes(_)))
        .map(|e| e.t);
    for t in states {
        let state = session.state_at(t).map_err(|e| e.to_string())?;
        if state.is_some_and(|s| s.nodes().iter().any(|n| under(n.id.as_str()))) {
            with.push(t);
        }
    }
    let last = session.frame_times().last().copied();
    let bound = last
        .and_then(|t| session.bindings_at(t))
        .map_or(0, |b| b.iter().filter(|b| under(b.id.as_str())).count());
    let humans = session
        .receipts(t1)
        .iter()
        .filter(|r| r.receipt.request.actor == Actor::Human)
        .count();
    Ok(json!({
        "span_ms": [millis(t0), millis(t1)],
        "surface_in_state_ms": [with.first(), with.last()],
        "frames": session.frame_times().len(),
        "events": session.events().len(),
        "last_frame_binds_of_it": bound,
        "requests_by_the_person": humans,
    }))
}

fn demo(args: &Args) -> Result<(), String> {
    std::fs::create_dir_all(&args.out).map_err(|e| format!("{}: {e}", args.out.display()))?;
    let mut app = Bench::open_on(Repo::build_review()?, true)?;
    // the rules alone: the laboratory never asks the Claude, and the line
    // is the same on every machine, with or without the program
    app.bench.host_surfaces_with(None);
    let limits = Limits {
        max_age: u64::MAX / 2,
        max_bytes: 1 << 30,
        ..Limits::default()
    };
    let rec = Recorder::new(limits);
    let log = Log::default();
    app.bench.add_sink(Box::new(rec.sink()));
    app.bench.add_sink(Box::new(log.clone()));
    let mut d = Demo {
        app,
        rec,
        log,
        out: args.out.clone(),
        shots: Vec::new(),
        clicks: 0,
        keys: 0,
    };
    d.record()?;
    println!("ide-lab: arm review, intent \"{INTENT}\"");
    let commits = d.commits()?;
    let fixed = d.fixed(&commits)?;
    let surface = d.surface(&commits)?;
    let closed = d.close()?;
    let requests = d.requests();
    let path = d.out.join(SESSION);
    let written = d
        .rec
        .persist(&path)
        .map_err(|e| format!("{}: {e}", path.display()))?;
    let session = read_back(&path)?;
    let report = json!({
        "arm": "review",
        "viewport": { "width": crate::bench::WIDTH, "height": crate::bench::HEIGHT },
        "fixed": fixed,
        "surface": surface,
        "closed": closed,
        "requests": requests,
        "session": {
            "path": path.display().to_string(),
            "bytes": written.bytes,
            "read_back": session,
        },
        "frames_png": d.shots,
        "exit": DONE,
    });
    report::write(&args.out, &report)?;
    let rows = vec![
        row(
            "fixed: clicks",
            &report["fixed"]["acts"]["clicks"].to_string(),
        ),
        row(
            "surface: clicks, keys",
            &format!(
                "{}, {}",
                report["surface"]["acts"]["clicks"], report["surface"]["acts"]["keys"]
            ),
        ),
        row(
            "files of the three at once",
            &format!(
                "fixed {}, surface {}",
                report["fixed"]["files_at_once"], report["surface"]["files_at_once"]
            ),
        ),
        row(
            "after the close",
            &format!(
                "{} nodes, {} bindings, {} surfaces",
                report["closed"]["nodes_after"],
                report["closed"]["bindings_after"],
                report["closed"]["surfaces_in_host"]
            ),
        ),
        row(
            "session",
            &format!("{} ({} B)", path.display(), written.bytes),
        ),
    ];
    println!();
    report::print_table("the review against the fixed panel", &rows);
    println!("report: {}", args.out.join("report.json").display());
    Ok(())
}
