//! Integration tests for the media / multimodal pillars of `UrnaBackend`
//! against a real .urna built with the urna writer: a blob_refs (0x14) +
//! blob_span_overlay (0x16) + blob_data (0x17) trio and one multimodal
//! space (0x15 + 0x21). Covers what the mtg card corpus exercises in the
//! app: overlay spans in `load_chunks`, `search_space`, `validate`,
//! hash-verified `export_blob`, and the frame decode plumbing up to the
//! ffmpeg boundary.

#![cfg(not(target_arch = "wasm32"))]

use std::path::PathBuf;

use sha2::{Digest, Sha256};
use urna_format::manifest::CapabilitiesExt;
use urna_format::{
    BLOB_REF_NONE, BlobRefRecord, BlobSpanEntry, ChunkInput, Manifest, SECTION_ENCODING_RAW,
    SPACE_DTYPE_F32, SpaceEntry, UrnaFileBuilder, encode_blob_data, encode_blob_refs,
    encode_blob_span_overlay, encode_space_table,
};
use urnaui::model::backend::UrnaBackend;
use urnaui::model::types::SearchMode;

const TEXT_DIM: usize = 4;
const VIS_DIM: usize = 2;
const VIS_HASH: &str = "sha256:vis111";
/// The "media stream" the chunks index into: opaque bytes, one "frame"
/// per chunk (the overlay says which).
const BLOB: &[u8] = b"not really av1 but a blob with bytes";

/// Three chunks: 0 and 1 map to frames 0 and 1 of the inlined blob, 2 is
/// a plain text chunk (BLOB_REF_NONE) that keeps its 0x03 span.
fn media_corpus() -> PathBuf {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("media.urna");
    let manifest = Manifest {
        embedding_model: "urnaui-test".into(),
        embedding_dim: TEXT_DIM as u32,
        n_chunks: 3,
        chunker_version: "urnaui-test/1".into(),
        model_hash: format!("sha256:{}", "0".repeat(64)),
        capabilities_ext: Some(CapabilitiesExt {
            supports_multimodal: Some(true),
            blobs_present: Some(true),
            ..Default::default()
        }),
        ..Default::default()
    };
    let mut builder = UrnaFileBuilder::new(manifest).reproducible(true);
    for (i, text) in ["card alpha", "card beta", "plain gamma"]
        .iter()
        .enumerate()
    {
        let mut emb = vec![0.0f32; TEXT_DIM];
        emb[i % TEXT_DIM] = 1.0;
        builder = builder.add_chunk(ChunkInput {
            canonical_text: text.to_string(),
            source_uri: if i < 2 {
                "frames".into()
            } else {
                "doc.txt".into()
            },
            byte_start: (i * 10) as u64,
            byte_end: (i * 10 + 5) as u64,
            embedding: emb,
        });
    }
    let records = vec![BlobRefRecord {
        content_hash: Sha256::digest(BLOB).into(),
        original_uri: "media://cards-av1.mp4".into(),
        byte_len: BLOB.len() as u64,
        inlined: true,
    }];
    builder = builder
        .blob_refs(encode_blob_refs(&records).unwrap())
        .blob_data(encode_blob_data(&[Some(BLOB)]).unwrap())
        .blob_span_overlay(
            encode_blob_span_overlay(&[
                BlobSpanEntry {
                    blob_ref_index: 0,
                    byte_start: 0,
                    byte_end: 1,
                },
                BlobSpanEntry {
                    blob_ref_index: 0,
                    byte_start: 1,
                    byte_end: 2,
                },
                BlobSpanEntry {
                    blob_ref_index: BLOB_REF_NONE,
                    byte_start: 0,
                    byte_end: 0,
                },
            ])
            .unwrap(),
        );
    let entries = vec![SpaceEntry {
        space_index: 1,
        name: "vision".into(),
        dim: VIS_DIM as u32,
        dtype: SPACE_DTYPE_F32,
        model_hash: VIS_HASH.into(),
        n_vectors: 3,
    }];
    builder = builder.space_table(encode_space_table(&entries).unwrap());
    let mut band = Vec::new();
    for v in [[1.0f32, 0.0], [0.0, 1.0], [-1.0, 0.0]] {
        for x in v {
            band.extend_from_slice(&x.to_le_bytes());
        }
    }
    builder = builder.space_band(1, SECTION_ENCODING_RAW, band);
    builder.write_to_path(&path).unwrap();
    std::mem::forget(dir);
    path
}

#[test]
fn snapshot_lists_blobs_and_spaces() {
    let backend = UrnaBackend::open(media_corpus()).unwrap();
    let view = backend.opened_view().unwrap();
    assert!(view.has_spaces);
    assert!(view.has_blob_data);
    assert_eq!(view.space_names, ["vision"]);
    assert_eq!(view.inspect.blobs.len(), 1);
    assert_eq!(view.inspect.blobs[0].original_uri, "media://cards-av1.mp4");
    assert!(view.inspect.blobs[0].inlined);
    assert_eq!(view.inspect.spaces.len(), 1);
    assert_eq!(view.inspect.spaces[0].dim, VIS_DIM as u32);
    assert_eq!(view.inspect.spaces[0].model_hash, VIS_HASH);
}

#[test]
fn load_chunks_prefers_the_overlay_span() {
    let mut backend = UrnaBackend::open(media_corpus()).unwrap();
    let data = backend.load_chunks().unwrap();
    assert_eq!(data.texts, ["card alpha", "card beta", "plain gamma"]);
    // Media chunks: the blob uri + frame span, and the blob handle.
    let m0 = &data.metas[0];
    assert_eq!(m0.source_uri, "media://cards-av1.mp4");
    assert_eq!((m0.offset_start, m0.offset_end), (0, 1));
    assert_eq!(m0.blob.unwrap().frame(), Some(0));
    assert_eq!(data.metas[1].blob.unwrap().frame(), Some(1));
    // The text chunk keeps its 0x03 span.
    let m2 = &data.metas[2];
    assert_eq!(m2.source_uri, "doc.txt");
    assert_eq!((m2.offset_start, m2.offset_end), (20, 25));
    assert!(m2.blob.is_none());
    // And the search hit reports the same coordinates the chunk does.
    let hit = &backend
        .search(&SearchMode::Exact, &[1.0, 0.0, 0.0, 0.0], 1)
        .unwrap()
        .hits[0];
    assert_eq!(hit.source_uri, m0.source_uri);
    assert_eq!((hit.offset_start, hit.offset_end), (0, 1));
}

#[test]
fn space_search_scores_the_band_and_gates_the_dim() {
    let backend = UrnaBackend::open(media_corpus()).unwrap();
    assert_eq!(
        backend.space_identity("vision").unwrap(),
        (VIS_DIM, VIS_HASH.to_string())
    );
    assert!(backend.space_identity("nope").is_err());
    let mode = SearchMode::Space {
        name: "vision".into(),
    };
    let r = backend.search(&mode, &[0.0, 1.0], 2).unwrap();
    // The runtime reports the per-space scan as the exact route (it is a
    // flat exact scan, recall 1.0, over the band).
    assert_eq!(r.explain.route, "exact");
    assert_eq!(r.recall, 1.0);
    assert_eq!(r.hits[0].chunk_id, backend.chunk_ids()[1]);
    assert!((r.hits[0].score - 1.0).abs() < 1e-6);
    // A text-dim query never scores the band.
    assert!(backend.search(&mode, &[1.0, 0.0, 0.0, 0.0], 2).is_err());
}

#[test]
fn validate_passes_on_a_fresh_file() {
    let backend = UrnaBackend::open(media_corpus()).unwrap();
    assert!(backend.validate().unwrap() >= 0.0);
}

#[test]
fn export_blob_writes_the_verified_bytes() {
    let backend = UrnaBackend::open(media_corpus()).unwrap();
    let dest = std::env::temp_dir().join(format!("urnaui_export_{}.bin", std::process::id()));
    backend.export_blob(0, &dest).unwrap();
    assert_eq!(std::fs::read(&dest).unwrap(), BLOB);
    let _ = std::fs::remove_file(&dest);
    assert!(backend.export_blob(7, &dest).is_err());
}

#[test]
fn frame_plumbing_resolves_the_blob_range() {
    let backend = UrnaBackend::open(media_corpus()).unwrap();
    let span = backend.blob_span(1).unwrap();
    assert_eq!(span.frame(), Some(1));
    assert!(
        backend.blob_span(2).is_none(),
        "text chunk has no blob span"
    );
    let range = backend.blob_range(0).unwrap();
    assert_eq!(range.len, BLOB.len() as u64);
    // The range points at the blob bytes inside the file itself.
    let bytes = std::fs::read(backend.path()).unwrap();
    assert_eq!(
        &bytes[range.abs_start as usize..(range.abs_start + range.len) as usize],
        BLOB
    );
    // ffmpeg on a non-media blob: a typed failure, never a panic.
    if urnaui::model::frames::ffmpeg_available() {
        let err =
            urnaui::model::frames::decode_frame(backend.path().to_str().unwrap(), range, 1, 64)
                .unwrap_err();
        assert!(!err.to_string().is_empty());
    }
}
