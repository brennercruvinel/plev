//! a text's wrap rule and OpenType features in the session file (version
//! 2, task-104): a textarea's `Anywhere` text and a counter's tabular
//! figures replay as they were drawn; a window of default texts costs
//! nothing for them; the files of versions 0 and 1 read as before, their
//! texts at the defaults, and a version 2 byte is never read in part.

use std::path::PathBuf;

use engine::drawing::compositor::{SceneNode, TextNodeKey};
use engine::drawing::text::{OverflowWrap, TextFeatures, TextStyle};
use record::{FileError, Recorder, Session};
use sha2::{Digest, Sha256};

type Fallible = Result<(), Box<dyn std::error::Error>>;

const ENTRY: usize = 1 + 8 + 8 + 32;

/// a textarea whose word is wider than its line, a counter in tabular
/// figures, a label at the defaults; `styled` false draws the same
/// strings with the defaults only.
fn frame(n: u32, styled: bool) -> Vec<SceneNode> {
    let base = TextStyle::new(14.0).with_line_height(20.0);
    let (area, counter) = if styled {
        (
            base.clone().with_overflow_wrap(OverflowWrap::Anywhere),
            base.clone().with_features(TextFeatures::TABULAR),
        )
    } else {
        (base.clone(), base.clone())
    };
    let text = |key: TextNodeKey, y: f32| SceneNode::Text {
        key,
        x: 10.0,
        y,
        color: [0.9, 0.9, 0.9, 1.0],
    };
    vec![
        SceneNode::Rect {
            x: 0.0,
            y: 0.0,
            w: 200.0,
            h: 120.0,
            color: [0.1, 0.1, 0.12, 1.0],
        },
        text(
            TextNodeKey::from_style(
                "https://example.org/a-path-wider-than-its-line",
                &area,
                Some(120.0),
            ),
            10.0,
        ),
        text(
            TextNodeKey::from_style(&format!("{} items", 1_000 + n), &counter, None),
            60.0,
        ),
        text(TextNodeKey::from_style("label", &base, None), 90.0),
    ]
}

fn recorded(styled: bool) -> Result<Recorder, Box<dyn std::error::Error>> {
    let rec = Recorder::default();
    for n in 0..4 {
        rec.frame_scene(u64::from(n) * 16, &frame(n, styled))?;
    }
    Ok(rec)
}

fn keys(scene: &[SceneNode]) -> Vec<TextNodeKey> {
    scene
        .iter()
        .filter_map(|n| match n {
            SceneNode::Text { key, .. } => Some(key.clone()),
            _ => None,
        })
        .collect()
}

fn version(bytes: &[u8]) -> u16 {
    u16::from_le_bytes([bytes[4], bytes[5]])
}

/// each section of a file, `(tag, payload)`, in index order.
fn sections(bytes: &[u8]) -> Vec<(u8, &[u8])> {
    let mut at = 4 + 2 + 2 + 8;
    for _ in 0..2 {
        at += 1 + usize::from(bytes[at]);
    }
    let count = usize::from(u16::from_le_bytes([bytes[at], bytes[at + 1]]));
    at += 2;
    (0..count)
        .map(|n| {
            let e = &bytes[at + n * ENTRY..at + (n + 1) * ENTRY];
            let word = |r: std::ops::Range<usize>| {
                let mut w = [0u8; 8];
                w.copy_from_slice(&e[r]);
                u64::from_le_bytes(w) as usize
            };
            let (offset, len) = (word(1..9), word(9..17));
            (e[0], &bytes[offset..offset + len])
        })
        .collect()
}

/// a file with the header of `bytes` (its build and start), the version
/// `v` and the sections `parts`, its index made right.
fn assemble(bytes: &[u8], v: u16, parts: &[(u8, Vec<u8>)]) -> Vec<u8> {
    let mut at = 4 + 2 + 2 + 8;
    for _ in 0..2 {
        at += 1 + usize::from(bytes[at]);
    }
    let mut out = bytes[..at].to_vec();
    out[4..6].copy_from_slice(&v.to_le_bytes());
    out.extend_from_slice(&(parts.len() as u16).to_le_bytes());
    let mut offset = (out.len() + parts.len() * ENTRY) as u64;
    for (tag, p) in parts {
        out.push(*tag);
        out.extend_from_slice(&offset.to_le_bytes());
        out.extend_from_slice(&(p.len() as u64).to_le_bytes());
        out.extend_from_slice(&Sha256::digest(p));
        offset += p.len() as u64;
    }
    for (_, p) in parts {
        out.extend_from_slice(p);
    }
    out
}

/// the sections of `bytes` but section B, as they are.
fn but_b(bytes: &[u8]) -> Vec<(u8, Vec<u8>)> {
    sections(bytes)
        .into_iter()
        .filter(|(tag, _)| *tag != b'B')
        .map(|(tag, p)| (tag, p.to_vec()))
        .collect()
}

#[test]
fn a_textarea_and_a_tabular_counter_replay_as_they_were_drawn() -> Fallible {
    let rec = recorded(true)?;
    let memory = rec.snapshot_window();
    let (bytes, written) = memory.to_bytes()?;
    assert_eq!(version(&bytes), record::VERSION_TEXT_STYLE);
    assert_eq!(written.bindings_bytes, 0);
    assert!(
        sections(&bytes).iter().all(|(tag, _)| *tag != b'B'),
        "no bindings, no section B"
    );
    let file = Session::from_bytes(&bytes)?;
    assert_eq!(file, memory, "every asset, its style included");
    for (n, t) in [0u64, 16, 32, 48].into_iter().enumerate() {
        let live = frame(n as u32, true);
        let shown = file.scene_at(t)?.ok_or("a frame")?;
        assert_eq!(shown, memory.scene_at(t)?.ok_or("a frame")?, "at {t}");
        // the keys the replay draws are the keys the app drew, bit for
        // bit: its shaping and its line breaks are the live ones.
        assert_eq!(keys(&shown), keys(&live), "at {t}");
    }
    let shown = keys(&file.scene_at(0)?.ok_or("a frame")?);
    assert_eq!(shown[0].overflow_wrap, OverflowWrap::Anywhere);
    assert_eq!(shown[1].features, TextFeatures::TABULAR);
    assert_eq!(
        (shown[2].overflow_wrap, shown[2].features),
        (OverflowWrap::Normal, TextFeatures::NONE)
    );
    // written again, the same bytes.
    assert_eq!(file.to_bytes()?.0, bytes);
    Ok(())
}

#[test]
fn a_default_text_costs_nothing_and_a_styled_one_a_byte() -> Fallible {
    let plain = recorded(false)?.snapshot_window().to_bytes()?;
    let styled = recorded(true)?.snapshot_window().to_bytes()?;
    // the same strings and places: only the style bytes differ, one per
    // styled text asset (the textarea's and the four counters').
    assert_eq!(styled.1.assets, plain.1.assets);
    assert_eq!(styled.1.assets_bytes, plain.1.assets_bytes + 5);
    // a window of default texts and no bindings is version 0's file but
    // for the version: labelled 0, it reads the same.
    let mut as_v0 = plain.0.clone();
    as_v0[4..6].copy_from_slice(&record::VERSION.to_le_bytes());
    assert_eq!(Session::from_bytes(&as_v0)?, Session::from_bytes(&plain.0)?);
    Ok(())
}

#[test]
fn a_style_byte_is_never_read_in_part() -> Fallible {
    let (bytes, _) = recorded(true)?.snapshot_window().to_bytes()?;
    // a reader of version 1 does not know the byte: a version 2 file
    // labelled 1 (with the section B version 1 requires; the assets are
    // read before it) has a text payload longer than it reads, refused.
    let mut parts = but_b(&bytes);
    parts.push((b'B', Vec::new()));
    let as_v1 = assemble(&bytes, record::VERSION_BINDINGS, &parts);
    let refused = Session::from_bytes(&as_v1);
    assert!(
        matches!(refused, Err(FileError::Trailing("asset payload"))),
        "{refused:?}"
    );
    // and a style bit no writer sets, or a 0 the writer leaves out, is
    // refused in a version 2 file.
    for bad in [0x80u8, 0] {
        let parts: Vec<(u8, Vec<u8>)> = but_b(&bytes)
            .into_iter()
            .map(|(tag, p)| (tag, if tag == b'A' { flip_style(&p, bad) } else { p }))
            .collect();
        let forged = assemble(&bytes, record::VERSION_TEXT_STYLE, &parts);
        assert!(
            matches!(
                Session::from_bytes(&forged),
                Err(FileError::Asset { reason, .. }) if reason.contains("style")
            ),
            "style {bad:#04x}: {:?}",
            Session::from_bytes(&forged).err()
        );
    }
    // a reader of today refuses a version it does not know by its number.
    let mut later = bytes.clone();
    later[4..6].copy_from_slice(&3u16.to_le_bytes());
    let refused = Session::from_bytes(&later).err().map(|e| e.to_string());
    assert_eq!(
        refused.as_deref(),
        Some("unsupported session version 3; this reader reads 0, 1 and 2")
    );
    Ok(())
}

/// the asset table `p` with the style byte of its first styled text set
/// to `bits`.
fn flip_style(p: &[u8], bits: u8) -> Vec<u8> {
    let mut out = p.to_vec();
    let count = u32::from_le_bytes([p[0], p[1], p[2], p[3]]);
    let mut at = 4;
    for _ in 0..count {
        let kind = p[at];
        let len = u32::from_le_bytes([p[at + 1], p[at + 2], p[at + 3], p[at + 4]]) as usize;
        let payload = &p[at + 5..at + 5 + len];
        // a text's payload: its string, then 4 + 4, a width flag (and 4),
        // 2 + 4, a family flag (and its bytes); a byte past that is style.
        if kind == 0 {
            let text = u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]);
            let mut q = 4 + text as usize + 8;
            q += 1 + if payload[q] == 1 { 4 } else { 0 };
            q += 6;
            q += 1 + if payload[q] == 1 {
                2 + usize::from(u16::from_le_bytes([payload[q + 1], payload[q + 2]]))
            } else {
                0
            };
            if q < len {
                out[at + 5 + q] = bits;
                return out;
            }
        }
        at += 5 + len;
    }
    out
}

/// the session fixtures of the gallery and the ide, written in version 1
/// before version 2 existed.
fn fixtures() -> Vec<PathBuf> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    [
        "apps/uigallery/tests/fixtures/sessions/silences-demo.monster",
        "apps/idestudio/tests/fixtures/sessions/bug84.monster",
    ]
    .iter()
    .map(|p| root.join(p))
    .collect()
}

#[test]
fn the_sessions_of_version_1_and_0_read_as_before() -> Fallible {
    for path in fixtures() {
        let name = path.display().to_string();
        let old = std::fs::read(&path)?;
        assert_eq!(version(&old), record::VERSION_BINDINGS, "{name}");
        let file = Session::from_bytes(&old)?;
        // every text at the defaults, the only form version 1 has.
        for t in file.frame_keys() {
            for key in keys(&file.scene_at(t)?.unwrap_or_default()) {
                assert_eq!(
                    (key.overflow_wrap, key.features),
                    (OverflowWrap::Normal, TextFeatures::NONE),
                    "{name} at {t}"
                );
            }
        }
        // written today, as version 2: the frames, the assets and the
        // bindings are the bytes they were (a default text adds none),
        // and the file reads back the same session.
        let (new, _) = file.to_bytes()?;
        assert_eq!(version(&new), record::VERSION_TEXT_STYLE, "{name}");
        for (tag, payload) in sections(&new) {
            if tag == b'S' {
                continue;
            }
            let before = sections(&old).into_iter().find(|(t, _)| *t == tag);
            assert_eq!(
                before.map(|(_, p)| p),
                Some(payload),
                "{name} section {tag}"
            );
        }
        assert_eq!(Session::from_bytes(&new)?, file, "{name}");
        // and as version 0, section B left out: the same frames, no
        // bindings.
        let v0 = assemble(&old, record::VERSION, &but_b(&old));
        let bare = Session::from_bytes(&v0)?;
        assert_eq!(bare.frame_times(), file.frame_times(), "{name}");
        assert_eq!(bare.events(), file.events(), "{name}");
        for t in file.frame_keys() {
            assert_eq!(bare.scene_at(t)?, file.scene_at(t)?, "{name} at {t}");
            assert_eq!(bare.bindings_at(t), Some(Vec::new()), "{name} at {t}");
        }
    }
    Ok(())
}
