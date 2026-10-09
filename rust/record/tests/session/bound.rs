//! where each bound control shows, frame by frame (task-63 phase d): taken
//! from the compositor with the frame, kept only when it changed, in
//! stacking order, read back from memory and from the file, cut with the
//! ring and the export, and absent from a file written before it.

use accord::{Millis, SemId};
use engine::drawing::compositor::{Compositor, LayerId, SceneNode};
use record::{Bound, FileError, Limits, Recorder, Session};
use sha2::{Digest, Sha256};

type Fallible = Result<(), Box<dyn std::error::Error>>;

fn sid(text: &str) -> SemId {
    SemId::parse(text).unwrap_or_else(|e| panic!("{text}: {e}"))
}

fn rect(x: f32, y: f32, w: f32, h: f32) -> SceneNode {
    SceneNode::Rect {
        x,
        y,
        w,
        h,
        color: [0.2, 0.2, 0.25, 1.0],
    }
}

/// what one frame of the test app shows.
#[derive(Clone, Copy)]
struct Look {
    /// the toast's top edge; `None` when no toast shows.
    toast: Option<f32>,
    /// the second row is in the list.
    second: bool,
}

/// a list with rows under a clip that cuts the second row in half, a
/// panel on a hidden layer, and a toast on a layer above, bound first.
struct App {
    c: Compositor,
    over: LayerId,
    hidden: LayerId,
}

impl App {
    fn new() -> App {
        let mut c = Compositor::new();
        let over = c.create_layer(10);
        let hidden = c.create_layer(20);
        c.set_layer_visible(hidden, false);
        App { c, over, hidden }
    }

    fn draw(&mut self, look: Look) -> &Compositor {
        let c = &mut self.c;
        c.begin_frame();
        if let Some(y) = look.toast {
            c.push_to_layer(self.over, rect(150.0, y, 100.0, 40.0));
            c.bind_to_layer(self.over, &sid("notices[0]"), [150.0, y, 100.0, 40.0]);
        }
        c.push_to_layer(self.hidden, rect(0.0, 0.0, 400.0, 400.0));
        c.bind_to_layer(self.hidden, &sid("panel"), [0.0, 0.0, 400.0, 400.0]);
        c.push(rect(0.0, 0.0, 200.0, 400.0));
        c.bind(&sid("changes"), [0.0, 0.0, 200.0, 400.0]);
        c.push_clip(0.0, 0.0, 200.0, 30.0);
        c.push(rect(0.0, 0.0, 200.0, 20.0));
        c.bind(&sid("changes.file[a]"), [0.0, 0.0, 200.0, 20.0]);
        if look.second {
            c.push(rect(0.0, 20.0, 200.0, 20.0));
            c.bind(&sid("changes.file[b]"), [0.0, 20.0, 200.0, 20.0]);
        }
        // a row scrolled out of the clip binds, and shows nowhere.
        c.bind(&sid("changes.file[z]"), [0.0, 60.0, 200.0, 20.0]);
        c.pop_clip();
        &self.c
    }
}

fn ids(list: &[Bound]) -> Vec<&str> {
    list.iter().map(|b| b.id.as_str()).collect()
}

/// the frames: a toast sliding up, then the second row leaving.
const FRAMES: [(Millis, Look); 6] = [
    (
        100,
        Look {
            toast: Some(360.0),
            second: true,
        },
    ),
    // the same frame again: nothing new.
    (
        116,
        Look {
            toast: Some(360.0),
            second: true,
        },
    ),
    // the toast moves: only its rect.
    (
        133,
        Look {
            toast: Some(340.0),
            second: true,
        },
    ),
    // the row leaves: the whole list.
    (
        150,
        Look {
            toast: Some(340.0),
            second: false,
        },
    ),
    // the toast leaves too.
    (
        300,
        Look {
            toast: None,
            second: false,
        },
    ),
    (
        316,
        Look {
            toast: None,
            second: false,
        },
    ),
];

fn recorded(limits: Limits) -> Result<Recorder, Box<dyn std::error::Error>> {
    let rec = Recorder::new(limits);
    let mut app = App::new();
    for (t, look) in FRAMES {
        rec.frame(t, app.draw(look))?;
    }
    Ok(rec)
}

#[test]
fn the_bindings_of_a_frame_are_kept_when_they_change_and_read_back_at_any_time() -> Fallible {
    let rec = recorded(Limits::default())?;
    let stats = rec.stats();
    assert_eq!(
        (stats.bindings_whole, stats.bindings_moved),
        (3, 1),
        "the first frame, the row leaving and the toast leaving are whole, the toast moving is a move: {stats:?}"
    );
    let memory = rec.snapshot_window();

    // stacking order: the list, its rows as bound, the toast on the layer
    // above last though it was bound first; the hidden layer's panel and
    // the row clipped away are not there, and the half-clipped row shows
    // its visible half.
    assert_eq!(memory.bindings_at(99), None, "before the first frame");
    let first = memory.bindings_at(100).ok_or("the first frame")?;
    assert_eq!(
        ids(&first),
        [
            "changes",
            "changes.file[a]",
            "changes.file[b]",
            "notices[0]"
        ]
    );
    assert_eq!(
        memory.binding_at(&sid("changes.file[b]"), 100),
        Some([0.0, 20.0, 200.0, 10.0])
    );
    assert_eq!(memory.binding_at(&sid("panel"), 100), None);
    assert_eq!(memory.binding_at(&sid("changes.file[z]"), 100), None);

    // between two frames the screen held the earlier one.
    assert_eq!(memory.bindings_at(125), Some(first.clone()));
    let toast = sid("notices[0]");
    assert_eq!(
        memory.binding_at(&toast, 133),
        Some([150.0, 340.0, 100.0, 40.0])
    );
    assert_eq!(
        memory.binding_at(&toast, 132),
        Some([150.0, 360.0, 100.0, 40.0])
    );
    let gone = memory.bindings_at(150).ok_or("a frame")?;
    assert_eq!(ids(&gone), ["changes", "changes.file[a]", "notices[0]"]);
    assert_eq!(
        memory.binding_at(&toast, 200),
        Some([150.0, 340.0, 100.0, 40.0])
    );
    assert_eq!(memory.binding_at(&toast, 300), None, "the toast left");
    assert_eq!(
        ids(&memory.bindings_at(10_000).ok_or("a frame")?),
        ["changes", "changes.file[a]"]
    );

    // the hit test of the frame on screen, as the compositor answered it.
    assert_eq!(memory.hit_at(100, 10.0, 10.0), Some(sid("changes.file[a]")));
    assert_eq!(memory.hit_at(100, 10.0, 25.0), Some(sid("changes.file[b]")));
    assert_eq!(
        memory.hit_at(100, 10.0, 35.0),
        Some(sid("changes")),
        "the clipped half of the row is the list's"
    );
    assert_eq!(memory.hit_at(100, 180.0, 380.0), Some(sid("notices[0]")));
    assert_eq!(memory.hit_at(100, 300.0, 10.0), None);

    // the file: version 1, and every instant reads as in memory.
    let tmp = tempfile::tempdir()?;
    let path = tmp.path().join("bound.monster");
    let written = rec.persist(&path)?;
    assert!(written.bindings_bytes > 0, "{written:?}");
    let bytes = std::fs::read(&path)?;
    assert_eq!(bytes[4..6], record::VERSION_BINDINGS.to_le_bytes());
    let file = Session::open(&path)?;
    for t in 90..400 {
        assert_eq!(file.bindings_at(t), memory.bindings_at(t), "at {t}");
    }
    Ok(())
}

#[test]
fn every_chunk_opens_with_the_whole_list_so_the_ring_and_the_export_keep_them() -> Fallible {
    let limits = Limits {
        max_age: 300,
        chunk_every: 100,
        ..Limits::default()
    };
    let rec = Recorder::new(limits);
    let mut app = App::new();
    let mut t = 0;
    while t <= 2_000 {
        // the toast drifts every 50 ms: a move inside a chunk, the whole
        // list at each chunk's first frame.
        let look = Look {
            toast: Some(360.0 - (t / 50) as f32),
            second: true,
        };
        rec.frame(t, app.draw(look))?;
        t += 25;
    }
    let stats = rec.stats();
    assert!(stats.evicted_chunks > 0, "the ring cut: {stats:?}");
    let window = rec.snapshot_window();
    let keys = window.frame_keys();
    let first = *keys.first().ok_or("a chunk")?;
    assert!(first > 1_000, "the window starts late: {first}");
    for key in &keys {
        let list = window.bindings_at(*key).ok_or("a frame")?;
        assert_eq!(list.len(), 4, "at the chunk start {key}");
    }
    let toast = sid("notices[0]");
    for at in (first..=2_000).step_by(25) {
        let y = 360.0 - (at / 50) as f32;
        assert_eq!(
            window.binding_at(&toast, at),
            Some([150.0, y, 100.0, 40.0]),
            "at {at}"
        );
    }

    let tmp = tempfile::tempdir()?;
    let path = tmp.path().join("part.monster");
    let (t0, t1) = (first + 130, first + 260);
    rec.export(t0, t1, &path)?;
    let part = Session::open(&path)?;
    for at in t0..=t1 {
        assert_eq!(part.bindings_at(at), window.bindings_at(at), "at {at}");
    }
    let (_, end) = part.frames_span().ok_or("frames")?;
    assert!(end <= t1);
    Ok(())
}

/// the file of `session` as a reader of version 0 wrote it: section B and
/// its index entry dropped, the offsets moved up, the version 0.
fn without_bindings(bytes: &[u8]) -> Vec<u8> {
    const ENTRY: usize = 49;
    let build = record::Build::current();
    let count_at = 4 + 2 + 2 + 8 + 1 + build.version.len() + 1 + build.git.len();
    let entries = count_at + 2;
    let count = u16::from_le_bytes([bytes[count_at], bytes[count_at + 1]]);
    assert_eq!(count, 5);
    let b_entry = entries + 4 * ENTRY;
    assert_eq!(bytes[b_entry], b'B');
    let b_offset = u64::from_le_bytes(bytes[b_entry + 1..b_entry + 9].try_into().unwrap_or([0; 8]));
    let mut out = bytes[..b_entry].to_vec();
    out[4..6].copy_from_slice(&0u16.to_le_bytes());
    out[count_at..count_at + 2].copy_from_slice(&4u16.to_le_bytes());
    for n in 0..4 {
        let at = entries + n * ENTRY + 1;
        let offset = u64::from_le_bytes(out[at..at + 8].try_into().unwrap_or([0; 8]));
        out[at..at + 8].copy_from_slice(&(offset - ENTRY as u64).to_le_bytes());
    }
    out.extend_from_slice(&bytes[b_entry + ENTRY..b_offset as usize]);
    out
}

#[test]
fn a_file_from_before_the_bindings_opens_with_none() -> Fallible {
    let rec = recorded(Limits::default())?;
    let (bytes, _) = rec.snapshot_window().to_bytes()?;
    let old = without_bindings(&bytes);
    let file = Session::from_bytes(&old)?;
    let memory = rec.snapshot_window();
    assert_eq!(file.frame_times(), memory.frame_times());
    for t in [100, 133, 150, 316] {
        assert_eq!(file.scene_at(t)?, memory.scene_at(t)?);
        assert_eq!(file.bindings_at(t), Some(Vec::new()), "at {t}");
        assert_eq!(file.hit_at(t, 10.0, 10.0), None);
    }
    assert_eq!(file.bindings_at(99), None);

    // a window with no bindings is written as version 0, the same bytes a
    // reader before them reads, and says so in what it wrote.
    let bare = Recorder::default();
    bare.frame_scene(10, &[rect(0.0, 0.0, 4.0, 4.0)])?;
    let (bare, written) = bare.snapshot_window().to_bytes()?;
    assert_eq!(bare[4..6], record::VERSION.to_le_bytes());
    assert_eq!(written.bindings_bytes, 0);
    let back = Session::from_bytes(&bare)?;
    assert_eq!(back.bindings_at(10), Some(Vec::new()));

    // and a version 0 file does not carry section B.
    let mut forged = bytes.clone();
    forged[4..6].copy_from_slice(&record::VERSION.to_le_bytes());
    assert!(matches!(
        Session::from_bytes(&forged),
        Err(FileError::UnknownSection(b'B'))
    ));
    Ok(())
}

/// `bytes` (a version 1 file) with section B replaced by `payload`, its
/// sha256 and length made right, so only the payload is wrong.
fn with_section_b(bytes: &[u8], payload: &[u8]) -> Vec<u8> {
    const ENTRY: usize = 49;
    let build = record::Build::current();
    let entries = 4 + 2 + 2 + 8 + 1 + build.version.len() + 1 + build.git.len() + 2;
    let b_entry = entries + 4 * ENTRY;
    let b_offset = u64::from_le_bytes(bytes[b_entry + 1..b_entry + 9].try_into().unwrap_or([0; 8]));
    let mut out = bytes[..b_offset as usize].to_vec();
    out[b_entry + 9..b_entry + 17].copy_from_slice(&(payload.len() as u64).to_le_bytes());
    let digest: [u8; 32] = Sha256::digest(payload).into();
    out[b_entry + 17..b_entry + 49].copy_from_slice(&digest);
    out.extend_from_slice(payload);
    out
}

fn u32s(values: &[u32]) -> Vec<u8> {
    values.iter().flat_map(|v| v.to_le_bytes()).collect()
}

/// a section B for one chunk: a table of ids, then `steps` as given.
fn section(table: &[&str], steps: &[u8], chunks: u32) -> Vec<u8> {
    let mut out = u32s(&[table.len() as u32]);
    for id in table {
        out.extend_from_slice(&(id.len() as u16).to_le_bytes());
        out.extend_from_slice(id.as_bytes());
    }
    out.extend_from_slice(&u32s(&[chunks]));
    out.extend_from_slice(steps);
    out
}

fn rect_bits(r: [f32; 4]) -> Vec<u8> {
    u32s(&r.map(f32::to_bits))
}

#[test]
fn section_b_is_read_strictly() -> Fallible {
    let rec = Recorder::default();
    let mut app = App::new();
    rec.frame(0, app.draw(FRAMES[0].1))?;
    let (bytes, written) = rec.snapshot_window().to_bytes()?;
    assert_eq!(written.chunks, 1);
    assert!(Session::from_bytes(&bytes).is_ok());

    let whole = |id: u32, r: [f32; 4]| [u32s(&[0]), vec![0], u32s(&[1, id]), rect_bits(r)].concat();
    let good = [u32s(&[1]), whole(0, [0.0, 0.0, 5.0, 5.0])].concat();
    let ok = with_section_b(&bytes, &section(&["a"], &good, 1));
    let file = Session::from_bytes(&ok)?;
    assert_eq!(file.binding_at(&sid("a"), 0), Some([0.0, 0.0, 5.0, 5.0]));

    let cases: Vec<(&str, Vec<u8>)> = vec![
        ("an id that does not parse", section(&["A B"], &good, 1)),
        ("an id twice in the table", section(&["a", "a"], &good, 1)),
        ("a chunk count off the index", section(&["a"], &good, 2)),
        (
            "an id index past the table",
            section(
                &["a"],
                &[u32s(&[1]), whole(1, [0.0, 0.0, 5.0, 5.0])].concat(),
                1,
            ),
        ),
        (
            "a rect with no area",
            section(
                &["a"],
                &[u32s(&[1]), whole(0, [0.0, 0.0, 0.0, 5.0])].concat(),
                1,
            ),
        ),
        (
            "a rect that is not finite",
            section(
                &["a"],
                &[u32s(&[1]), whole(0, [f32::NAN, 0.0, 5.0, 5.0])].concat(),
                1,
            ),
        ),
        (
            "a move before a whole list",
            section(
                &["a"],
                &[
                    u32s(&[1, 0]),
                    vec![1],
                    u32s(&[1, 0]),
                    rect_bits([0.0, 0.0, 5.0, 5.0]),
                ]
                .concat(),
                1,
            ),
        ),
        (
            "a move past the list",
            section(
                &["a"],
                &[
                    u32s(&[2]),
                    whole(0, [0.0, 0.0, 5.0, 5.0]),
                    u32s(&[5]),
                    vec![1],
                    u32s(&[1, 1]),
                    rect_bits([0.0, 0.0, 5.0, 5.0]),
                ]
                .concat(),
                1,
            ),
        ),
        (
            "step times that do not increase",
            section(
                &["a"],
                &[
                    u32s(&[2]),
                    whole(0, [0.0, 0.0, 5.0, 5.0]),
                    whole(0, [1.0, 0.0, 5.0, 5.0]),
                ]
                .concat(),
                1,
            ),
        ),
        (
            "an unknown step kind",
            section(&["a"], &[u32s(&[1, 0]), vec![7], u32s(&[0])].concat(), 1),
        ),
        (
            "bytes past the last chunk",
            [section(&["a"], &good, 1), vec![0]].concat(),
        ),
    ];
    for (what, payload) in cases {
        let bad = with_section_b(&bytes, &payload);
        let read = Session::from_bytes(&bad);
        assert!(
            matches!(
                read,
                Err(FileError::BoundId { .. }
                    | FileError::Bindings { .. }
                    | FileError::Truncated(_)
                    | FileError::Trailing(_))
            ),
            "{what}: {read:?}"
        );
    }

    // cut anywhere, and damaged anywhere, the file is an error, never a
    // panic.
    for n in 0..bytes.len() {
        assert!(Session::from_bytes(&bytes[..n]).is_err(), "cut at {n}");
    }
    let mut seed: u32 = 0x2545_f491;
    for _ in 0..500 {
        seed ^= seed << 13;
        seed ^= seed >> 17;
        seed ^= seed << 5;
        let mut bad = bytes.clone();
        let at = seed as usize % bad.len();
        bad[at] ^= (seed >> 24) as u8 | 1;
        let _ = Session::from_bytes(&bad);
    }
    Ok(())
}
