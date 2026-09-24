//! `plev-inspect --export`: a stretch of a recorded session written as a
//! session of its own, the bug report (task-64 phase c). the file reopens
//! with the same content in the stretch, on both tracks: the events, the
//! state, the requests, the frames and where each control showed. the
//! session it was cut from stays the same bytes.

use std::path::Path;
use std::process::{Command, Output};

use engine::compositor::{Compositor, LayerId, SceneNode};
use rec::{Limits, Recorder, Session};
use sem::{Dispatcher, Millis, Request, SemId};

use crate::apps::{Git, agent, look};

type Fallible = Result<(), Box<dyn std::error::Error>>;

fn run(args: &[&str]) -> std::io::Result<Output> {
    Command::new(env!("CARGO_BIN_EXE_plev-inspect"))
        .args(args)
        .output()
}

fn text(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// the frame of the toy at `t`: a row per staged file, and the commit
/// button, bound, drifting a pixel every 50 ms so each frame differs.
fn draw(c: &mut Compositor, layer: LayerId, git: &Git, t: Millis) -> Result<(), sem::SemError> {
    c.begin_frame();
    for (n, _) in git.staged.iter().enumerate() {
        let y = 20.0 * n as f32;
        c.push_to_layer(
            layer,
            SceneNode::Rect {
                x: 0.0,
                y,
                w: 200.0,
                h: 18.0,
                color: [0.2, 0.2, 0.25, 1.0],
            },
        );
    }
    let x = (t / 50) as f32;
    let button = [x, 100.0, 80.0, 24.0];
    c.push_to_layer(
        layer,
        SceneNode::Rect {
            x,
            y: 100.0,
            w: 80.0,
            h: 24.0,
            color: [0.1, 0.4, 0.9, 1.0],
        },
    );
    c.bind_to_layer(layer, &SemId::parse("commit.submit")?, button);
    Ok(())
}

/// the thesis, 2 s long, with the state and a frame every 50 ms and both
/// tracks keyed often (the state re-keys when an event comes with its
/// cadence due), so a stretch in the middle opens at a keyframe of each
/// before it: the agent observes at +0.100s, a person unstages B at
/// +0.300s, the agent's submit is refused stale at +0.500s, it observes
/// again at +0.900s and its submit completes at +1.100s.
fn recorded(dir: &Path) -> Result<(std::path::PathBuf, Session), Box<dyn std::error::Error>> {
    let rec = Recorder::new(Limits {
        full_every: 400,
        chunk_every: 300,
        ..Limits::default()
    });
    let mut dispatcher = Dispatcher::new();
    dispatcher.add_sink(Box::new(rec.sink()));
    let mut git = Git::new(&["A", "B", "C"], "fix login");
    let mut c = Compositor::new();
    let layer = c.create_layer(0);
    let mut seen = look(&git);
    for t in (1_000..=3_000).step_by(50) {
        match t {
            1_100 | 1_900 => {
                seen = look(&git);
                rec.observed(t, agent(), None, seen.revision());
            }
            1_300 => {
                let now = look(&git);
                let req = Request::observe(
                    &now,
                    sem::Actor::Human,
                    "changes",
                    "unstage",
                    Some("B".into()),
                )?;
                dispatcher.dispatch(&mut git, req, t);
            }
            1_500 | 2_100 => {
                let req = Request::observe(&seen, agent(), "commit.submit", "submit", None)?;
                dispatcher.dispatch(&mut git, req, t);
            }
            _ => {}
        }
        rec.snapshot(t, &look(&git));
        draw(&mut c, layer, &git, t)?;
        rec.frame(t, &c)?;
    }
    let path = dir.join("session.monster");
    rec.persist(&path)?;
    let session = Session::open(&path)?;
    Ok((path, session))
}

#[test]
fn a_stretch_reopens_with_the_same_content_on_both_tracks() -> Fallible {
    let tmp = tempfile::tempdir()?;
    let (path, whole) = recorded(tmp.path())?;
    let file = path.to_str().ok_or("a utf-8 path")?;
    let bytes = std::fs::read(&path)?;
    let out = tmp.path().join("report.monster");
    let report = out.to_str().ok_or("a utf-8 path")?;

    let done = run(&[file, "--export", "650", "1150", report])?;
    assert_eq!(
        done.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&done.stderr)
    );
    let said = text(&done);
    assert!(
        said.starts_with(&format!(
            "exported +0.650s to +1.150s of {file} to {report}\n"
        )),
        "{said}"
    );
    for line in [
        "  opens at  +0.",
        "  events    ",
        "  frames    ",
        "  size      ",
    ] {
        assert!(said.contains(line), "{line:?} in {said}");
    }
    assert_eq!(
        std::fs::read(&path)?,
        bytes,
        "the session it reads is untouched"
    );

    let part = Session::open(&out)?;
    let start = whole.start();
    let (t0, t1) = (start + 650, start + 1_150);
    // it opens at the keyframes of both tracks that govern t0, and ends at
    // t1.
    let first = part.events().first().ok_or("an empty stretch")?;
    assert!(matches!(first.kind, rec::Kind::Snapshot { .. }), "keyed");
    assert!(first.t <= t0 && first.t > start, "{} of {t0}", first.t);
    assert_eq!(whole.keyframe_at(t0), Some(first));
    let (key, end) = part.frames_span().ok_or("frames")?;
    assert!(key <= t0 && key > start, "{key} of {t0}");
    assert!(end <= t1 && part.events().iter().all(|e| e.t <= t1));
    // the same content, from t0 to t1.
    assert_eq!(part.between(t0, t1), whole.between(t0, t1));
    for t in (t0..=t1).step_by(10) {
        assert_eq!(part.state_at(t)?, whole.state_at(t)?, "state at {t}");
        assert_eq!(part.scene_at(t)?, whole.scene_at(t)?, "frame at {t}");
        assert_eq!(part.frame_at(t), whole.frame_at(t), "frame time at {t}");
        assert_eq!(part.bindings_at(t), whole.bindings_at(t), "bindings at {t}");
    }
    // the requests the stretch holds whole, as the session holds them.
    let held = part.receipts(t1);
    assert!(held.iter().all(|r| r.whole), "{held:?}");
    let same: Vec<_> = whole
        .receipts(t1)
        .into_iter()
        .filter(|r| held.iter().any(|h| h.receipt.id == r.receipt.id))
        .collect();
    assert_eq!(held, same);
    let ends: Vec<&str> = held
        .iter()
        .filter_map(|r| r.receipt.stage().map(sem::Stage::name))
        .collect();
    assert_eq!(ends, ["rejected", "completed"], "both submits");

    // the report reads like any session.
    let summary = run(&[report])?;
    assert_eq!(summary.status.code(), Some(0));
    assert!(
        text(&summary).contains("rejected: stale"),
        "{}",
        text(&summary)
    );
    Ok(())
}

#[test]
fn an_export_that_cannot_be_made_says_why() -> Fallible {
    let tmp = tempfile::tempdir()?;
    let (path, _) = recorded(tmp.path())?;
    let file = path.to_str().ok_or("a utf-8 path")?;
    let bytes = std::fs::read(&path)?;
    let out = tmp.path().join("report.monster");
    let report = out.to_str().ok_or("a utf-8 path")?;

    let backwards = run(&[file, "--export", "1400", "700", report])?;
    assert_eq!(backwards.status.code(), Some(2));
    let with_why = run(&[file, "--export", "0", "700", report, "--why", "2"])?;
    assert_eq!(with_why.status.code(), Some(2));
    let over = run(&[file, "--export", "0", "700", file])?;
    assert_eq!(over.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&over.stderr).contains("would write over"));
    assert_eq!(std::fs::read(&path)?, bytes, "never over the file it reads");
    assert!(!out.exists(), "nothing written");

    let nowhere = tmp.path().join("missing").join("report.monster");
    let failed = run(&[
        file,
        "--export",
        "0",
        "700",
        nowhere.to_str().ok_or("utf-8")?,
    ])?;
    assert_eq!(failed.status.code(), Some(4));
    let unread = tmp.path().join("none.monster");
    let failed = run(&[
        unread.to_str().ok_or("utf-8")?,
        "--export",
        "0",
        "700",
        report,
    ])?;
    assert_eq!(failed.status.code(), Some(3));
    Ok(())
}
