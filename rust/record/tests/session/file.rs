//! the session file: the thesis session persisted and read back equal to
//! the window in memory; an export that opens at a full snapshot; the
//! strict reader against truncated, corrupted, foreign and animation
//! files; what the format keeps of paths and images.

use std::path::PathBuf;

use accord::{Actor, Dispatcher, Millis, Request, Role, Semantics, SnapshotBuilder, Value};
use engine::drawing::compositor::{SceneNode, TextNodeKey};
use engine::drawing::gpu::ImageHandle;
use engine::drawing::path::PathBuilder;
use record::{
    Build, Event, FileError, FileKind, InputKind, Kind, Limits, Recorder, Session, probe,
};

use crate::toys::{Git, agent, id, look};

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
    let sources = |s: &Session| -> Vec<Option<accord::Source>> {
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

/// the S section of a file written before task-57b spells every default
/// out and writes `source: null`; the reader takes such a line to the same
/// state, with the same revision, and writes it back leaner.
#[test]
fn an_event_line_of_the_form_before_the_lean_one_still_reads() -> Fallible {
    struct Rows;

    impl Semantics for Rows {
        fn project(&self, b: &mut SnapshotBuilder) {
            b.node("changes", Role::List, "changes").state(
                "staged",
                Value::Set(["A".to_owned(), "B".to_owned()].into()),
            );
            b.node("commit.submit", Role::Button, "commit")
                .action("submit")
                .reads("changes:staged");
        }
    }

    const CONSTRAINTS: &str = r#"{"editable":false,"required":false,"range":null,"step":null,"max_len":null,"options":null,"secret":false}"#;
    const POLICY: &str = r#"{"actors":{"human":true,"agent":true,"assistive":true,"test":true},"confirm":"never","destructive":false}"#;
    let old = format!(
        concat!(
            r#"{{"t":1000,"kind":{{"snapshot":{{"snapshot":{{"nodes":["#,
            r#"{{"id":"changes","role":"list","label":"changes","description":null,"parent":null,"#,
            r#""state":{{"staged":{{"kind":"set","value":["A","B"]}}}},"actions":[],"#,
            r#""constraints":{c},"enabled":true,"inert":false,"focusable":false,"source":null}},"#,
            r#"{{"id":"commit.submit","role":"button","label":"commit","description":null,"parent":null,"#,
            r#""state":{{}},"actions":[{{"name":"submit","label":null,"enabled":true,"#,
            r#""reads":["changes:staged"],"param":null,"policy":{p}}}],"#,
            r#""constraints":{c},"enabled":true,"inert":false,"focusable":true,"#,
            r#""source":{{"file":"rust/record/tests/session/file.rs","line":7,"column":5}}}}"#,
            r#"],"focused":null}},"changes":[]}}}}}}"#
        ),
        c = CONSTRAINTS,
        p = POLICY
    );
    let event: Event = serde_json::from_str(&old)?;
    assert_eq!(event.t, 1_000);
    let Kind::Snapshot { snapshot, changes } = &event.kind else {
        return Err(format!("not a snapshot: {event:?}").into());
    };
    assert!(changes.is_empty());
    let now = look(&Rows);
    assert_eq!(**snapshot, now);
    assert_eq!(snapshot.revision(), now.revision());
    let sources: Vec<Option<(String, u32, u32)>> = snapshot
        .nodes()
        .values()
        .map(|n| {
            n.source
                .as_ref()
                .map(|s| (s.file.to_string(), s.line, s.column))
        })
        .collect();
    assert_eq!(
        sources,
        [
            None,
            Some(("rust/record/tests/session/file.rs".to_owned(), 7, 5))
        ],
        "the recorded source travels as recorded"
    );

    // written today the same event is leaner and reads back equal.
    let lean = serde_json::to_string(&event)?;
    assert!(
        lean.len() < old.len() / 2,
        "{} of {}",
        lean.len(),
        old.len()
    );
    for key in ["description", "inert", "constraints", "policy", "param"] {
        assert!(!lean.contains(key), "{key} in {lean}");
    }
    assert!(lean.contains(r#""source":{"file""#), "{lean}");
    assert_eq!(serde_json::from_str::<Event>(&lean)?, event);
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
        Session::from_bytes(&with(4, &2u16.to_le_bytes())),
        Err(FileError::UnsupportedVersion(2))
    ));
    // version 1 is version 0 with section B, so a version 0 layout that
    // says 1 misses it.
    assert!(matches!(
        Session::from_bytes(&with(4, &1u16.to_le_bytes())),
        Err(FileError::MissingSection('B'))
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
        motion::decode(&bytes),
        Err(motion::ReadError::BadMagic)
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
    assert!(motion::decode(&bytes).is_ok());
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
    let corners = |p: &engine::drawing::path::TessellatedPath| -> Vec<[f32; 2]> {
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
        let at = accord::StateRef::parse("gauge:v")?;
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

/// a list whose items come and go.
struct Items(Vec<&'static str>);

impl Semantics for Items {
    fn project(&self, b: &mut SnapshotBuilder) {
        b.node("todo", Role::List, "todo");
        for item in &self.0 {
            b.node(format!("todo.item[{item}]").as_str(), Role::ListItem, *item)
                .parent("todo")
                .state("done", false);
        }
    }
}

/// the same file with its S section replaced by `events`, the section
/// index rewritten (offsets, lengths, sha256) as the writer lays it out.
fn with_events(bytes: &[u8], events: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    use sha2::{Digest, Sha256};
    const ENTRY: usize = 1 + 8 + 8 + 32;
    let at = index_at();
    let mut payloads: Vec<(u8, &[u8])> = Vec::new();
    for n in 0..4 {
        let entry = &bytes[at + n * ENTRY..at + (n + 1) * ENTRY];
        let offset = usize::try_from(u64::from_le_bytes(entry[1..9].try_into()?))?;
        let len = usize::try_from(u64::from_le_bytes(entry[9..17].try_into()?))?;
        let payload = if entry[0] == b'S' {
            events
        } else {
            &bytes[offset..offset + len]
        };
        payloads.push((entry[0], payload));
    }
    let mut out = bytes[..at].to_vec();
    let mut offset = (at + 4 * ENTRY) as u64;
    for (tag, payload) in &payloads {
        out.push(*tag);
        out.extend_from_slice(&offset.to_le_bytes());
        out.extend_from_slice(&(payload.len() as u64).to_le_bytes());
        out.extend_from_slice(&Sha256::digest(payload));
        offset += payload.len() as u64;
    }
    for (_, payload) in payloads {
        out.extend_from_slice(payload);
    }
    Ok(out)
}

/// the S section of `session` as a file before task-83 held it: every
/// `node_added` a snapshot event keeps is its id alone, or `id` when
/// given.
fn events_before_83(session: &Session, id: Option<&str>) -> Result<Vec<u8>, serde_json::Error> {
    let mut out = Vec::new();
    for event in session.events() {
        let mut json = serde_json::to_value(event)?;
        if let Some(changes) = json
            .pointer_mut("/kind/snapshot/changes")
            .and_then(serde_json::Value::as_array_mut)
        {
            for change in changes {
                if let Some(node) = change.get("node_added") {
                    let named = id.map_or_else(|| node["id"].clone(), serde_json::Value::from);
                    change["node_added"] = named;
                }
            }
        }
        out.extend_from_slice(serde_json::to_string(&json)?.as_bytes());
        out.push(b'\n');
    }
    Ok(out)
}

/// a session written before task-83 kept `{"node_added": "<id>"}` in the
/// changes of a snapshot event. it still opens: the node is the one the
/// event's own snapshot holds, so it reads back equal to what the
/// recorder holds today. an id that snapshot does not hold is a typed
/// error on that line, and no node is made up.
#[test]
fn a_node_added_by_id_alone_reads_as_the_node_its_snapshot_holds() -> Fallible {
    let rec = Recorder::default();
    rec.snapshot(0, &look(&Items(vec!["milk"])));
    rec.snapshot(10, &look(&Items(vec!["milk", "mail"])));
    let window = rec.snapshot_window();
    let Some(Kind::Snapshot { snapshot, changes }) = window.events().get(1).map(|e| &e.kind) else {
        return Err(format!("a whole snapshot at 10: {:?}", window.events()).into());
    };
    let mail = snapshot.get(&id("todo.item[mail]")).ok_or("no mail")?;
    assert_eq!(
        changes,
        &[accord::Change::NodeAdded(Box::new(mail.clone()))]
    );
    let (bytes, _) = window.to_bytes()?;

    let old = events_before_83(&window, None)?;
    let text = String::from_utf8(old.clone())?;
    assert!(
        text.contains(r#""changes":[{"node_added":"todo.item[mail]"}]"#),
        "{text}"
    );
    let file = Session::from_bytes(&with_events(&bytes, &old)?)?;
    assert_eq!(file.events(), window.events(), "the node, as recorded");
    let state = file.state_at(10)?.ok_or("a state")?;
    assert!(state.get(&id("todo.item[mail]")).is_some());
    // written again, only the form of today comes out.
    let (again, _) = file.to_bytes()?;
    assert_eq!(Session::from_bytes(&again)?.events(), window.events());
    assert!(!String::from_utf8_lossy(&again).contains(r#""node_added":"todo"#));

    let ghost = events_before_83(&window, Some("todo.item[ghost]"))?;
    match Session::from_bytes(&with_events(&bytes, &ghost)?) {
        Err(FileError::Event { line, message }) => {
            assert_eq!(line, 2, "the snapshot event at 10");
            assert!(message.contains("todo.item[ghost]"), "{message}");
        }
        other => return Err(format!("a typed error on line 2: {other:?}").into()),
    }
    Ok(())
}

/// opens the session files named in `REC_OPEN` (paths joined by `:`) and
/// prints what each holds: its events by kind, the `node_added` its
/// snapshot events keep, its span, and the receipts of `commit.submit` at
/// its end. a check on files outside the repository (the sessions of
/// checkpoint a, task-83), so run by hand:
/// `REC_OPEN=a.monster:b.monster cargo test -p rec --test session open_the_named_sessions -- --ignored --nocapture`.
#[test]
#[ignore = "reads the files named in REC_OPEN"]
fn open_the_named_sessions() -> Fallible {
    let paths = std::env::var("REC_OPEN").unwrap_or_default();
    let mut refused = Vec::new();
    for path in paths.split(':').filter(|p| !p.is_empty()) {
        let session = match Session::open(path) {
            Ok(session) => session,
            Err(e) => {
                println!("{path}: does not open: {e}");
                refused.push(path.to_owned());
                continue;
            }
        };
        let mut kinds: Vec<(&str, usize)> = Vec::new();
        let mut added = Vec::new();
        for event in session.events() {
            let name = event.kind.name();
            match kinds.iter_mut().find(|(k, _)| *k == name) {
                Some((_, n)) => *n += 1,
                None => kinds.push((name, 1)),
            }
            if let Kind::Snapshot { changes, .. } = &event.kind {
                for change in changes {
                    if let accord::Change::NodeAdded(node) = change {
                        added.push(format!("{} ({:?} {:?})", node.id, node.role, node.label));
                    }
                }
            }
        }
        let end = session.span().map_or(0, |(_, end)| end);
        println!(
            "{path}: opens, {} events {kinds:?}, span {:?}",
            session.events().len(),
            session.span()
        );
        println!("  node_added in snapshot events: {added:?}");
        for r in session.receipts_for(&id("commit.submit"), end) {
            let stages: Vec<&str> = r.receipt.stages.iter().map(|(s, _)| s.name()).collect();
            println!(
                "  commit.submit request {} by {}: {} ({:?})",
                r.receipt.id,
                r.receipt.request.actor,
                stages.join(" -> "),
                r.receipt.stage()
            );
        }
    }
    assert!(refused.is_empty(), "did not open: {refused:?}");
    Ok(())
}
