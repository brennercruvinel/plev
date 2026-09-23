//! the session file: the thesis session persisted and read back equal to
//! the window in memory; an export that opens at a full snapshot; the
//! strict reader against truncated, corrupted, foreign and animation
//! files; what the format keeps of paths and images.

use std::path::PathBuf;

use engine::compositor::{SceneNode, TextNodeKey};
use engine::gpu::ImageHandle;
use engine::path::PathBuilder;
use rec::{Build, FileError, FileKind, InputKind, Kind, Limits, Recorder, Session, probe};
use sem::{Actor, Dispatcher, Millis, Request, Role, Semantics, SnapshotBuilder, Value};

use crate::apps::{Git, agent, id, look};

type Fallible = Result<(), Box<dyn std::error::Error>>;

fn dir() -> Result<tempfile::TempDir, std::io::Error> {
    tempfile::tempdir()
}

/// what the git client draws: a card, a row per staged file, the button.
fn draw(git: &Git, hover: bool) -> Vec<SceneNode> {
    let mut scene = vec![SceneNode::RoundedRect {
        x: 10.0,
        y: 10.0,
        w: 300.0,
        h: 220.0,
        color: [0.1, 0.1, 0.12, 1.0],
        corner_radius: 8.0,
        border_width: 1.0,
        border_color: [1.0, 1.0, 1.0, 0.1],
    }];
    for (i, file) in git.staged.iter().enumerate() {
        scene.push(SceneNode::Text {
            key: TextNodeKey::new(file, 14.0, 20.0, None),
            x: 20.0,
            y: 20.0 + 24.0 * i as f32,
            color: [0.9, 0.9, 0.9, 1.0],
        });
    }
    scene.push(SceneNode::Rect {
        x: 20.0,
        y: 180.0,
        w: 120.0,
        h: 32.0,
        color: if hover {
            [0.3, 0.5, 0.9, 1.0]
        } else {
            [0.2, 0.4, 0.8, 1.0]
        },
    });
    scene.push(SceneNode::Text {
        key: TextNodeKey::new(&git.message, 14.0, 20.0, Some(100.0)),
        x: 30.0,
        y: 186.0,
        color: [1.0; 4],
    });
    scene
}

/// the thesis session with the frames the app drew along it.
fn thesis() -> Result<Recorder, Box<dyn std::error::Error>> {
    let rec = Recorder::new(Limits {
        chunk_every: 250,
        ..Limits::default()
    });
    let mut dispatcher = Dispatcher::new();
    dispatcher.add_sink(Box::new(rec.sink()));
    let mut git = Git::new(&["A", "B", "C"], "fix login");
    rec.frame_scene(900, &draw(&git, false))?;

    let seen = look(&git);
    rec.snapshot(1_000, &seen);
    rec.observed(1_000, agent(), None, seen.revision());
    let commit = Request::observe(&seen, agent(), "commit.submit", "submit", None)?;
    rec.input(
        1_100,
        InputKind::PointerDown,
        Some([40.0, 44.0]),
        Some(id("changes")),
    );
    let unstage = Request::observe(
        &look(&git),
        Actor::Human,
        "changes",
        "unstage",
        Some("B".into()),
    )?;
    dispatcher.dispatch(&mut git, unstage, 1_100);
    rec.snapshot(1_100, &look(&git));
    rec.frame_scene(1_116, &draw(&git, false))?;
    dispatcher.dispatch(&mut git, commit, 1_300);
    rec.frame_scene(1_350, &draw(&git, true))?;
    let seen = look(&git);
    rec.snapshot(1_400, &seen);
    rec.observed(1_400, agent(), None, seen.revision());
    let again = Request::observe(&seen, agent(), "commit.submit", "submit", None)?;
    dispatcher.dispatch(&mut git, again, 1_500);
    rec.snapshot(1_500, &look(&git));
    rec.frame_scene(1_516, &draw(&git, false))?;
    Ok(rec)
}

#[test]
fn the_thesis_session_persists_and_reads_back_the_same() -> Fallible {
    let rec = thesis()?;
    let memory = rec.snapshot_window();
    let tmp = dir()?;
    let path = tmp.path().join("thesis.monster");
    let written = rec.persist(&path)?;
    let bytes = std::fs::read(&path)?;
    assert_eq!(written.bytes, bytes.len());
    assert_eq!(probe(&bytes), FileKind::Session);
    assert_eq!(written.dropped, 0);
    assert_eq!(written.events, memory.events().len());
    assert!(
        written.chunks >= 2,
        "the cadence cut the frames: {written:?}"
    );

    let file = Session::open(&path)?;
    assert_eq!(file, memory, "every event, every chunk, every asset");
    assert_eq!(file.build(), &Build::current());
    assert_eq!(file.start(), 900);
    for t in (850..=1_600).step_by(25) {
        assert_eq!(file.state_at(t)?, memory.state_at(t)?, "state at {t}");
        assert_eq!(file.receipts(t), memory.receipts(t), "receipts at {t}");
        assert_eq!(
            file.receipts_for(&id("commit.submit"), t),
            memory.receipts_for(&id("commit.submit"), t)
        );
        assert_eq!(file.scene_at(t)?, memory.scene_at(t)?, "scene at {t}");
    }
    let sources = |s: &Session| -> Vec<Option<sem::Source>> {
        let state = s.state_at(1_600).ok().flatten();
        state
            .map(|st| st.nodes().iter().map(|n| n.source.clone()).collect())
            .unwrap_or_default()
    };
    assert_eq!(
        sources(&file),
        sources(&memory),
        "the projected sources travel"
    );
    assert!(file.scene_at(1_600)?.is_some_and(|s| !s.is_empty()));
    Ok(())
}

/// a toy list whose items change every step, projected and drawn.
struct Ticker {
    n: i64,
}

impl Semantics for Ticker {
    fn project(&self, b: &mut SnapshotBuilder) {
        b.node("ticker", Role::Group, "ticker").state("n", self.n);
        b.node("ticker.label", Role::Label, "label")
            .parent("ticker")
            .state("text", format!("tick {}", self.n / 10));
    }
}

fn ticker_scene(n: i64) -> Vec<SceneNode> {
    vec![
        SceneNode::Rect {
            x: (n % 50) as f32,
            y: 0.0,
            w: 10.0,
            h: 10.0,
            color: [1.0, 0.5, 0.0, 1.0],
        },
        SceneNode::Text {
            key: TextNodeKey::new(&format!("tick {}", n / 10), 12.0, 16.0, None),
            x: 0.0,
            y: 20.0,
            color: [1.0; 4],
        },
    ]
}

#[test]
fn an_export_of_a_middle_window_opens_at_a_full_snapshot() -> Fallible {
    let rec = Recorder::new(Limits {
        full_every: 500,
        chunk_every: 400,
        ..Limits::default()
    });
    let mut ticker = Ticker { n: 0 };
    for t in (0..10_000u64).step_by(50) {
        ticker.n += 1;
        rec.snapshot(t, &look(&ticker));
        rec.frame_scene(t, &ticker_scene(ticker.n))?;
    }
    let memory = rec.snapshot_window();
    let (t0, t1) = (4_020, 6_010);
    let tmp = dir()?;
    let path = tmp.path().join("excerpt.monster");
    let whole = memory.to_bytes()?.1;
    let part = rec.export(t0, t1, &path)?;
    assert!(
        part.chunks < whole.chunks && part.assets < whole.assets,
        "{part:?} of {whole:?}"
    );

    let file = Session::open(&path)?;
    let first = file.events().first().ok_or("an empty export")?;
    assert!(
        matches!(first.kind, Kind::Snapshot { .. }),
        "it opens keyed"
    );
    assert!(first.t <= t0 && t0 - first.t < 500);
    let (key, _) = file.frames_span().ok_or("no frames")?;
    assert_eq!(file.frame_keys().first(), Some(&key));
    assert!(key <= t0 && t0 - key < 400);
    assert!(file.events().iter().all(|e| e.t <= t1));
    assert!(file.frame_times().iter().all(|&t| t <= t1));
    for t in (t0..=t1).step_by(10) {
        assert_eq!(file.state_at(t)?, memory.state_at(t)?, "state at {t}");
        assert_eq!(file.scene_at(t)?, memory.scene_at(t)?, "scene at {t}");
    }
    Ok(())
}

/// the start of the section index: the fixed header, then the build.
fn index_at() -> usize {
    let b = Build::current();
    4 + 2 + 2 + 8 + 1 + b.version.len() + 1 + b.git.len() + 2
}

fn thesis_bytes() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    Ok(thesis()?.snapshot_window().to_bytes()?.0)
}

#[test]
fn a_truncated_file_is_an_error_at_every_length() -> Fallible {
    let bytes = thesis_bytes()?;
    for n in 0..bytes.len() {
        assert!(Session::from_bytes(&bytes[..n]).is_err(), "cut at {n}");
    }
    let mut longer = bytes.clone();
    longer.push(0);
    assert!(matches!(
        Session::from_bytes(&longer),
        Err(FileError::SectionLayout(_))
    ));
    assert!(Session::from_bytes(&bytes).is_ok());
    Ok(())
}

#[test]
fn a_corrupted_section_fails_its_sha256() -> Fallible {
    let bytes = thesis_bytes()?;
    let payloads = index_at() + 4 * 49;
    for at in (payloads..bytes.len()).step_by(97) {
        let mut bad = bytes.clone();
        bad[at] ^= 0x5a;
        assert!(
            matches!(Session::from_bytes(&bad), Err(FileError::Checksum(_))),
            "byte {at}"
        );
    }
    Ok(())
}

#[test]
fn a_foreign_header_is_refused_by_name() -> Fallible {
    let bytes = thesis_bytes()?;
    let with = |at: usize, value: &[u8]| {
        let mut out = bytes.clone();
        out[at..at + value.len()].copy_from_slice(value);
        out
    };
    let unknown = with(index_at(), b"X");
    assert!(matches!(
        Session::from_bytes(&unknown),
        Err(FileError::UnknownSection(b'X'))
    ));
    let twice = with(index_at() + 49, b"S");
    assert!(matches!(
        Session::from_bytes(&twice),
        Err(FileError::DuplicateSection('S'))
    ));
    assert!(matches!(
        Session::from_bytes(&with(0, b"RIFF")),
        Err(FileError::BadMagic)
    ));
    assert!(matches!(
        Session::from_bytes(&with(4, &1u16.to_le_bytes())),
        Err(FileError::UnsupportedVersion(1))
    ));
    assert!(matches!(
        Session::from_bytes(&with(6, &1u16.to_le_bytes())),
        Err(FileError::UnsupportedFlags(1))
    ));
    assert!(matches!(
        Session::from_bytes(&with(8, &7u64.to_le_bytes())),
        Err(FileError::StartMismatch { header: 7, .. })
    ));
    assert!(matches!(
        monster::decode(&bytes),
        Err(monster::ReadError::BadMagic)
    ));
    Ok(())
}

#[test]
fn an_animation_handed_to_the_session_reader_is_named() -> Fallible {
    let golden: PathBuf = [
        env!("CARGO_MANIFEST_DIR"),
        "..",
        "monster",
        "fixtures",
        "golden_v0_minimal.monster",
    ]
    .iter()
    .collect();
    let bytes = std::fs::read(golden)?;
    assert_eq!(probe(&bytes), FileKind::Animation);
    assert!(monster::decode(&bytes).is_ok());
    let refused = Session::from_bytes(&bytes);
    assert!(matches!(refused, Err(FileError::Animation)), "{refused:?}");
    let message = refused.err().map(|e| e.to_string()).unwrap_or_default();
    assert!(message.contains("monster_player"), "{message}");
    assert_eq!(probe(b"MO"), FileKind::Unknown);
    assert!(matches!(
        Session::open("/nonexistent/session.monster"),
        Err(FileError::Io(_))
    ));
    Ok(())
}

#[test]
fn random_damage_never_panics_the_reader() -> Fallible {
    let bytes = thesis_bytes()?;
    let mut seed: u64 = 0x9e37_79b9_7f4a_7c15;
    let mut next = || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed
    };
    for _ in 0..2_000 {
        let mut bad = bytes.clone();
        for _ in 0..(next() % 4 + 1) {
            let at = (next() % bad.len() as u64) as usize;
            bad[at] = next() as u8;
        }
        let cut = (next() % (bad.len() as u64 + 1)) as usize;
        let _ = Session::from_bytes(&bad[..cut.max(bad.len() / 2)]);
    }
    Ok(())
}

#[test]
fn a_path_comes_back_on_the_twips_grid_and_an_image_as_its_size() -> Fallible {
    let path = PathBuilder::new()
        .move_to(0.33, 0.0)
        .line_to(10.0, 0.01)
        .line_to(5.0, 8.0)
        .close()
        .fill([0.2, 0.4, 0.6, 1.0]);
    let image = ImageHandle {
        id: 9,
        atlas_x: 1,
        atlas_y: 1,
        width: 16,
        height: 8,
    };
    let scene = vec![
        SceneNode::Path { data: path.clone() },
        SceneNode::Image {
            x: 0.0,
            y: 0.0,
            w: 16.0,
            h: 8.0,
            image,
            corner_radius: 0.0,
        },
    ];
    let rec = Recorder::default();
    rec.frame_scene(0, &scene)?;
    let memory = rec.snapshot_window();
    let (bytes, written) = memory.to_bytes()?;
    assert_eq!(written.images_without_pixels, 1);
    let file = Session::from_bytes(&bytes)?;
    let shown = file.scene_at(0)?.ok_or("no frame")?;
    let [SceneNode::Path { data }] = shown.as_slice() else {
        return Err(format!("the path alone, the image drawn as nothing: {shown:?}").into());
    };
    // the same triangles, corner by corner, within half a twip (monster
    // packs vertices in first-use order, so compare through the indices).
    let corners = |p: &engine::path::TessellatedPath| -> Vec<[f32; 2]> {
        p.indices
            .iter()
            .filter_map(|&i| p.vertices.get(i as usize).map(|v| v.position))
            .collect()
    };
    let (back, sent) = (corners(data), corners(&path));
    assert_eq!(back.len(), sent.len());
    assert_eq!(back.len(), path.indices.len());
    for (a, b) in back.iter().zip(&sent) {
        assert!((a[0] - b[0]).abs() <= 0.025, "{a:?} {b:?}");
        assert!((a[1] - b[1]).abs() <= 0.025, "{a:?} {b:?}");
    }
    // written again, the image keeps its size and still has no pixels.
    assert_eq!(file.to_bytes()?.1.images_without_pixels, 1);
    Ok(())
}

/// a projected number that is not finite has no json form.
struct Gauge {
    v: f64,
}

impl Semantics for Gauge {
    fn project(&self, b: &mut SnapshotBuilder) {
        b.node("gauge", Role::Group, "gauge")
            .state("v", Value::Num(self.v));
    }
}

#[test]
fn an_event_with_no_json_form_is_left_out_with_the_changes_after_it() -> Fallible {
    let rec = Recorder::default();
    let steps: [(Millis, f64); 5] = [(0, 1.0), (10, f64::NAN), (20, 2.0), (30, 3.0), (40, 4.0)];
    for (t, v) in steps {
        rec.snapshot(t, &look(&Gauge { v }));
    }
    rec.snapshot_full(50, &look(&Gauge { v: 5.0 }));
    let (bytes, written) = rec.snapshot_window().to_bytes()?;
    assert_eq!(written.dropped, 4, "the NaN change and the three after it");
    let file = Session::from_bytes(&bytes)?;
    let v = |t| -> Result<Option<Value>, Box<dyn std::error::Error>> {
        let at = sem::StateRef::parse("gauge:v")?;
        Ok(file.state_at(t)?.and_then(|s| s.value(&at).cloned()))
    };
    assert_eq!(v(0)?, Some(Value::Num(1.0)));
    assert_eq!(
        v(30)?,
        Some(Value::Num(1.0)),
        "the gap reads the last state before it"
    );
    assert_eq!(
        v(50)?,
        Some(Value::Num(5.0)),
        "the next snapshot keys it again"
    );
    Ok(())
}
