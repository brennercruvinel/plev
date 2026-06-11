//! Path asset payload codec tests: round-trip on the twips grid,
//! deterministic dedup across packs, chunk splitting under the u16
//! asset limit, and graceful rejection of malformed bytes.

use crate::asset_path::{pack_chunks, unpack};
use plev::compositor::QuadVertex;
use plev::path::TessellatedPath;

fn path(positions: &[[f32; 2]], indices: &[u32], color: [f32; 4]) -> TessellatedPath {
    TessellatedPath {
        vertices: positions
            .iter()
            .map(|&position| QuadVertex { position, color })
            .collect(),
        indices: indices.to_vec(),
        hash: 7,
    }
}

#[test]
fn round_trips_grid_aligned_geometry() {
    // 0.05 = one twip: grid-aligned input survives bit-exactly.
    let p = path(
        &[[0.0, 0.0], [10.05, 0.0], [0.0, 7.25]],
        &[0, 1, 2],
        [1.0, 0.0, 0.0, 1.0],
    );
    let chunks = pack_chunks(&p);
    assert_eq!(chunks.len(), 1);
    let out = unpack(&chunks[0]).expect("well-formed payload");
    let pos: Vec<[f32; 2]> = out.vertices.iter().map(|v| v.position).collect();
    assert_eq!(pos, vec![[0.0, 0.0], [10.05, 0.0], [0.0, 7.25]]);
    assert_eq!(out.indices, vec![0, 1, 2]);
    assert!(out.vertices.iter().all(|v| v.color == [1.0, 0.0, 0.0, 1.0]));
}

#[test]
fn identical_geometry_packs_to_identical_bytes() {
    let a = path(&[[1.0, 2.0], [3.0, 4.0], [5.0, 6.0]], &[0, 1, 2], [0.0; 4]);
    let mut b = a.clone();
    b.hash = 99; // the engine hash is not part of the payload
    assert_eq!(pack_chunks(&a), pack_chunks(&b));
}

#[test]
fn sub_twip_jitter_collapses_to_the_same_payload() {
    let a = path(&[[1.0, 2.0], [3.0, 4.0], [5.0, 6.0]], &[0, 1, 2], [0.0; 4]);
    let b = path(
        &[[1.001, 1.999], [3.0004, 4.0], [5.0, 6.0009]],
        &[0, 1, 2],
        [0.0; 4],
    );
    assert_eq!(pack_chunks(&a), pack_chunks(&b));
}

#[test]
fn empty_path_packs_to_nothing() {
    assert!(pack_chunks(&path(&[], &[], [0.0; 4])).is_empty());
    assert!(pack_chunks(&path(&[[0.0, 0.0]], &[], [0.0; 4])).is_empty());
}

#[test]
fn large_path_splits_and_every_chunk_fits_and_draws() {
    // a strip of ~12000 shared vertices: one payload would be ~96KB+,
    // past the u16 asset cap, so it must split at triangle boundaries.
    let n = 12_000usize;
    let positions: Vec<[f32; 2]> = (0..n).map(|i| [i as f32, (i % 2) as f32]).collect();
    let indices: Vec<u32> = (0..n - 2)
        .flat_map(|i| [i as u32, i as u32 + 1, i as u32 + 2])
        .collect();
    let p = path(&positions, &indices, [0.2, 0.4, 0.6, 1.0]);
    let chunks = pack_chunks(&p);
    assert!(
        chunks.len() > 1,
        "must split, got {} chunk(s)",
        chunks.len()
    );
    let mut tris = 0usize;
    for c in &chunks {
        assert!(
            c.len() <= usize::from(u16::MAX),
            "chunk of {} bytes",
            c.len()
        );
        let out = unpack(c).expect("every chunk well-formed");
        tris += out.indices.len() / 3;
    }
    assert_eq!(tris, indices.len() / 3, "no triangle lost in the split");
}

#[test]
fn split_is_deterministic_across_packs() {
    let n = 9_000usize;
    let positions: Vec<[f32; 2]> = (0..n).map(|i| [i as f32 * 0.5, i as f32]).collect();
    let indices: Vec<u32> = (0..n - 2)
        .flat_map(|i| [i as u32, i as u32 + 1, i as u32 + 2])
        .collect();
    let p = path(&positions, &indices, [0.0; 4]);
    assert_eq!(pack_chunks(&p), pack_chunks(&p));
}

#[test]
fn malformed_payloads_unpack_to_none() {
    let good = pack_chunks(&path(
        &[[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
        &[0, 1, 2],
        [0.0; 4],
    ))
    .remove(0);
    assert!(unpack(&good[..good.len() - 1]).is_none(), "truncated");
    assert!(unpack(&[]).is_none(), "empty");
    let mut bad_index = good.clone();
    let at = bad_index.len() - 2;
    bad_index[at] = 0xFF; // index past vertex_count
    bad_index[at + 1] = 0xFF;
    assert!(unpack(&bad_index).is_none(), "index out of range");
}

#[test]
fn unpack_hash_tracks_content() {
    let a = pack_chunks(&path(
        &[[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
        &[0, 1, 2],
        [0.0; 4],
    ))
    .remove(0);
    let b = pack_chunks(&path(
        &[[0.0, 0.0], [2.0, 0.0], [0.0, 1.0]],
        &[0, 1, 2],
        [0.0; 4],
    ))
    .remove(0);
    let (ha, hb) = (unpack(&a).unwrap().hash, unpack(&b).unwrap().hash);
    assert_eq!(ha, unpack(&a).unwrap().hash, "stable");
    assert_ne!(ha, hb, "content-sensitive");
}
