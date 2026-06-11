//! malformed-input and wire-robustness tests for the anm decoder:
//! truncation, bit flips, checksums, magic/version, unknown sections and
//! delta ops the v0 timeline ir cannot represent.

use crate::container::{self, DeltaOp, SEC_DELTA, SEC_DESC, SEC_KEYFRAME, Section};
use crate::easing::EasingTable;
use crate::ir::{Node, NodeKind, Props};
use crate::read::{ReadError, decode};
use crate::tests_golden::golden_doc;
use crate::tests_write::parse;
use crate::write::encode;

const GOLDEN: &[u8] = include_bytes!("../fixtures/golden_v0_minimal.anm");

#[test]
fn every_truncation_prefix_errs_without_panic() {
    for len in 0..GOLDEN.len() {
        assert!(decode(&GOLDEN[..len]).is_err(), "prefix of {len} bytes");
    }
}

#[test]
fn flipping_any_payload_byte_errs_without_panic() {
    let parsed = parse(GOLDEN);
    let first = parsed
        .sections
        .iter()
        .map(|(_, off, ..)| *off)
        .min()
        .unwrap() as usize;
    for i in first..GOLDEN.len() {
        let mut bytes = GOLDEN.to_vec();
        bytes[i] ^= 0xFF;
        assert!(decode(&bytes).is_err(), "payload byte {i} flipped");
    }
}

#[test]
fn corrupted_checksum_is_a_typed_error() {
    // flip one payload byte: its stored sha256 no longer matches.
    let parsed = parse(GOLDEN);
    let (_, off, ..) = parsed.sections[0];
    let mut bytes = GOLDEN.to_vec();
    bytes[off as usize] ^= 1;
    assert!(matches!(
        decode(&bytes),
        Err(ReadError::ChecksumMismatch { tag: 'K', entry: 0 })
    ));
    // flip one digest byte in the index instead: same failure.
    let digest_at = GOLDEN.len()
        - parsed
            .sections
            .iter()
            .map(|(.., l, _)| *l as usize)
            .sum::<usize>()
        - 32;
    let mut bytes = GOLDEN.to_vec();
    bytes[digest_at] ^= 1;
    assert!(matches!(
        decode(&bytes),
        Err(ReadError::ChecksumMismatch { .. })
    ));
}

#[test]
fn bad_magic_version_and_flags_are_rejected() {
    let mut m = GOLDEN.to_vec();
    m[0] = b'X';
    assert_eq!(decode(&m).unwrap_err(), ReadError::BadMagic);
    let mut v = GOLDEN.to_vec();
    v[4] = 9;
    assert_eq!(decode(&v).unwrap_err(), ReadError::UnsupportedVersion(9));
    let mut f = GOLDEN.to_vec();
    f[6] = 1;
    assert_eq!(decode(&f).unwrap_err(), ReadError::UnsupportedFlags(1));
}

#[test]
fn unknown_section_tag_is_rejected() {
    // golden header: 14 head + 12 asset table + 6 easing table + 4
    // desc_offset + 2 sec_count puts the first index tag at byte 38.
    let mut bytes = GOLDEN.to_vec();
    assert_eq!(bytes[38], SEC_KEYFRAME, "index layout moved; fix offset");
    bytes[38] = b'Z';
    assert_eq!(
        decode(&bytes).unwrap_err(),
        ReadError::UnknownSectionTag(b'Z')
    );
}

fn kf_payload(t: f32, nodes: &[Node]) -> Vec<u8> {
    let mut buf = Vec::new();
    container::put_f32(&mut buf, t);
    container::put_u16(&mut buf, nodes.len() as u16);
    for node in nodes {
        container::put_node(&mut buf, node);
    }
    buf
}

fn one_rect_kf() -> Section {
    Section {
        tag: SEC_KEYFRAME,
        payload: kf_payload(0.0, &[crate::tests_write::rect(1, 0, 10.0)]),
    }
}

fn file_with_delta(payload: Vec<u8>) -> Vec<u8> {
    let sections = [
        one_rect_kf(),
        Section {
            tag: SEC_DELTA,
            payload,
        },
    ];
    container::assemble(1.0, 60, &[], &EasingTable::default(), &sections)
}

#[test]
fn place_replace_remove_are_unrepresentable_in_the_timeline_ir() {
    let table = EasingTable::default();
    let node = crate::tests_write::rect(2, 1, 0.0);
    let ops = [
        (
            0u8,
            DeltaOp::Place {
                at_s: 0.0,
                node: node.clone(),
            },
        ),
        (2u8, DeltaOp::Replace { at_s: 0.0, node }),
        (
            3u8,
            DeltaOp::Remove {
                at_s: 0.0,
                node_id: 1,
            },
        ),
    ];
    for (code, op) in ops {
        let mut payload = Vec::new();
        container::put_u16(&mut payload, 1);
        container::put_op(&mut payload, &op, &table);
        let bytes = file_with_delta(payload);
        assert_eq!(
            decode(&bytes).unwrap_err(),
            ReadError::UnrepresentableOp(code)
        );
    }
    let bytes = file_with_delta(vec![1, 0, 9]); // op_count 1, op code 9
    assert_eq!(decode(&bytes).unwrap_err(), ReadError::UnknownOpCode(9));
}

/// One modify op down to the easing byte, hand-built so reserved easing
/// ids and out-of-range curve indices can be planted.
fn modify_payload(easing_byte: u8, curve_idx: Option<u16>) -> Vec<u8> {
    let mut p = Vec::new();
    container::put_u16(&mut p, 1); // op_count
    p.push(1); // OP_MODIFY
    container::put_f32(&mut p, 0.0); // at_s
    container::put_u16(&mut p, 1); // node id
    container::put_u16(&mut p, 0b1); // presence: X
    container::put_u16(&mut p, 1); // seg_count
    p.push(easing_byte);
    if let Some(idx) = curve_idx {
        container::put_u16(&mut p, idx);
    }
    container::put_f32(&mut p, 0.5); // dur_s
    container::put_i32(&mut p, 100); // target twips
    p
}

#[test]
fn reserved_easing_and_bad_curve_index_are_rejected() {
    let bytes = file_with_delta(modify_payload(0x21, None));
    assert_eq!(decode(&bytes).unwrap_err(), ReadError::UnknownEasing(0x21));
    let bytes = file_with_delta(modify_payload(0xFF, Some(0)));
    assert_eq!(
        decode(&bytes).unwrap_err(),
        ReadError::CurveIndexOutOfRange { index: 0, len: 0 }
    );
}

fn file_with_desc(payload: Vec<u8>) -> Vec<u8> {
    let sections = [
        one_rect_kf(),
        Section {
            tag: SEC_DESC,
            payload,
        },
    ];
    container::assemble(1.0, 60, &[], &EasingTable::default(), &sections)
}

fn desc_payload(entries: &[(u16, &[u8])]) -> Vec<u8> {
    let mut p = Vec::new();
    container::put_u16(&mut p, entries.len() as u16);
    for (kf, text) in entries {
        container::put_u16(&mut p, *kf);
        container::put_u16(&mut p, text.len() as u16);
        p.extend_from_slice(text);
    }
    p
}

#[test]
fn description_track_errors_are_typed() {
    let bad_utf8 = file_with_desc(desc_payload(&[(0, &[0xFF, 0xFE])]));
    assert!(matches!(decode(&bad_utf8), Err(ReadError::BadUtf8(_))));
    let out_of_range = file_with_desc(desc_payload(&[(7, b"x")]));
    assert_eq!(
        decode(&out_of_range).unwrap_err(),
        ReadError::DescOutOfRange {
            keyframe: 7,
            keyframes: 1
        }
    );
    let out_of_order = file_with_desc(desc_payload(&[(0, b"a"), (0, b"b")]));
    assert_eq!(
        decode(&out_of_order).unwrap_err(),
        ReadError::DescOutOfOrder(0)
    );
}

#[test]
fn structural_section_errors_are_typed() {
    let table = EasingTable::default();
    // delta before any keyframe.
    let orphan = container::assemble(
        1.0,
        60,
        &[],
        &table,
        &[Section {
            tag: SEC_DELTA,
            payload: vec![0, 0],
        }],
    );
    assert_eq!(decode(&orphan).unwrap_err(), ReadError::DeltaBeforeKeyframe);
    // trailing bytes after the last field of a K payload.
    let mut padded = kf_payload(0.0, &[]);
    padded.push(0);
    let bytes = container::assemble(
        1.0,
        60,
        &[],
        &table,
        &[Section {
            tag: SEC_KEYFRAME,
            payload: padded,
        }],
    );
    assert_eq!(decode(&bytes).unwrap_err(), ReadError::TrailingPayload('K'));
    // presence bit with no v0 prop, unknown node kind.
    let mut weird = Vec::new();
    container::put_f32(&mut weird, 0.0);
    container::put_u16(&mut weird, 1);
    container::put_u16(&mut weird, 1); // node id
    container::put_u16(&mut weird, 0); // depth
    weird.push(0); // rect
    container::put_u16(&mut weird, 1 << 12);
    let bytes = container::assemble(
        1.0,
        60,
        &[],
        &table,
        &[Section {
            tag: SEC_KEYFRAME,
            payload: weird,
        }],
    );
    assert_eq!(
        decode(&bytes).unwrap_err(),
        ReadError::UnknownPresenceBits(1 << 12)
    );
    let mut alien = Vec::new();
    container::put_f32(&mut alien, 0.0);
    container::put_u16(&mut alien, 1);
    container::put_u16(&mut alien, 1);
    container::put_u16(&mut alien, 0);
    alien.push(9); // no such kind
    let bytes = container::assemble(
        1.0,
        60,
        &[],
        &table,
        &[Section {
            tag: SEC_KEYFRAME,
            payload: alien,
        }],
    );
    assert_eq!(decode(&bytes).unwrap_err(), ReadError::UnknownNodeKind(9));
}

#[test]
fn desc_offset_mismatch_and_invalid_ir_are_rejected() {
    // a file with no T section must carry desc_offset 0; golden header
    // puts desc_offset right after the easing table.
    let bytes = container::assemble(1.0, 60, &[], &EasingTable::default(), &[one_rect_kf()]);
    let mut lying = bytes.clone();
    assert_eq!(&lying[18..22], &[0, 0, 0, 0], "desc_offset field moved");
    lying[18] = 1;
    assert!(matches!(
        decode(&lying),
        Err(ReadError::DescOffsetMismatch { .. })
    ));
    // structurally well-formed container, invalid IR: no opening keyframe.
    let late = Section {
        tag: SEC_KEYFRAME,
        payload: kf_payload(0.5, &[]),
    };
    let bytes = container::assemble(1.0, 60, &[], &EasingTable::default(), &[late]);
    assert!(matches!(decode(&bytes), Err(ReadError::Ir(_))));
}
