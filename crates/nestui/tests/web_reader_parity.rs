//! Parity tests for the portable web reader (`model::nestread`): files
//! are built with the real `nest_format` writer and cross-checked against
//! `nest_runtime` (the mmap ground truth), so the GUI-side port is proven
//! byte/value-compatible, not just plausible.

#![cfg(not(target_arch = "wasm32"))]

use nest_format::{ChunkInput, Edge, Manifest, NestFileBuilder, encode_graph_adjacency};
use nestui::model::backend::NestBackend;
use nestui::model::nestread::NestBytes;
use nestui::model::types::SearchMode;

fn demo_manifest() -> Manifest {
    Manifest {
        embedding_model: "parity-test".into(),
        embedding_dim: 4,
        n_chunks: 3,
        chunker_version: "parity/1".into(),
        model_hash: format!("sha256:{}", "0".repeat(64)),
        ..Default::default()
    }
}

fn demo_chunks() -> Vec<ChunkInput> {
    [
        ("alpha text", [1.0, 0.2, 0.0, 0.0]),
        ("beta text", [0.0, 1.0, 0.3, 0.0]),
        ("gamma text", [0.1, 0.0, 1.0, 0.4]),
    ]
    .into_iter()
    .enumerate()
    .map(|(i, (text, emb))| ChunkInput {
        canonical_text: text.to_string(),
        source_uri: "corpus.txt".into(),
        byte_start: (i * 16) as u64,
        byte_end: (i * 16 + text.len()) as u64,
        embedding: emb.to_vec(),
    })
    .collect()
}

fn demo_edges() -> Vec<Edge> {
    vec![
        Edge {
            src: 0,
            dst: 1,
            edge_type: nest_format::EDGE_TYPE_NEXT_CHUNK,
        },
        Edge {
            src: 1,
            dst: 2,
            edge_type: nest_format::EDGE_TYPE_NEXT_CHUNK,
        },
        Edge {
            src: 0,
            dst: 2,
            edge_type: nest_format::EDGE_TYPE_SEMANTIC,
        },
    ]
}

/// Open the same bytes through both readers.
fn both(bytes: &[u8]) -> (NestBytes, NestBackend) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("parity.nest");
    std::fs::write(&path, bytes).unwrap();
    let native = NestBackend::open(&path).unwrap();
    std::mem::forget(dir);
    (
        NestBytes::open("parity.nest".into(), bytes.to_vec()).unwrap(),
        native,
    )
}

#[test]
fn identity_chunks_and_inspect_match_the_runtime() {
    let bytes = NestFileBuilder::new(demo_manifest())
        .add_chunks(demo_chunks())
        .graph_adjacency(encode_graph_adjacency(3, &demo_edges()).unwrap())
        .build_bytes()
        .unwrap();
    let (web, native) = both(&bytes);

    let inspect = web.inspect_view();
    let native_inspect = native.inspect().unwrap();

    // Identity hashes: the load-bearing parity claim (citations derive
    // from content_hash).
    assert_eq!(inspect.file_hash, native_inspect.file_hash);
    assert_eq!(inspect.content_hash, native_inspect.content_hash);

    // Header/manifest fields.
    assert_eq!(inspect.embedding_dim, native_inspect.embedding_dim);
    assert_eq!(inspect.n_chunks, native_inspect.n_chunks);
    assert_eq!(inspect.file_size, native_inspect.file_size);
    assert_eq!(
        inspect.manifest.embedding_model,
        native_inspect.manifest.embedding_model
    );
    assert_eq!(
        inspect.manifest.model_hash,
        native_inspect.manifest.model_hash
    );
    assert!(inspect.manifest.capabilities.supports_exact);

    // Section table: same ids/names/sizes.
    let web_sections: Vec<(u32, &str, u64)> = inspect
        .sections
        .iter()
        .map(|s| (s.section_id, s.name.as_str(), s.size))
        .collect();
    let native_sections: Vec<(u32, &str, u64)> = native_inspect
        .sections
        .iter()
        .map(|s| (s.section_id, s.name.as_str(), s.size))
        .collect();
    assert_eq!(web_sections, native_sections);

    // Chunks payload.
    let opened = web.opened_view();
    assert_eq!(opened.chunk_ids, native.chunk_ids());
    let chunks = web.chunks_data().unwrap();
    assert_eq!(chunks.texts, ["alpha text", "beta text", "gamma text"]);
    assert_eq!(chunks.metas.len(), 3);
    assert_eq!(chunks.metas[1].source_uri, "corpus.txt");
    assert_eq!(chunks.metas[1].offset_start, 16);
}

#[test]
fn graph_decode_matches_the_runtime_csr() {
    let bytes = NestFileBuilder::new(demo_manifest())
        .add_chunks(demo_chunks())
        .graph_adjacency(encode_graph_adjacency(3, &demo_edges()).unwrap())
        .build_bytes()
        .unwrap();
    let (web, native) = both(&bytes);

    let web_graph = web.graph_data().expect("graph decoded");
    let native_graph = native.graph_data().expect("graph decoded");
    assert_eq!(web_graph, native_graph);
    assert_eq!(web_graph.neighbors(0), &[1, 2]);
    assert_eq!(web_graph.kind(0, 1), Some(1)); // SEMANTIC
}

#[test]
fn exact_search_matches_the_runtime() {
    let bytes = NestFileBuilder::new(demo_manifest())
        .add_chunks(demo_chunks())
        .build_bytes()
        .unwrap();
    let (web, native) = both(&bytes);

    let query = [1.0, 0.1, 0.0, 0.0];
    let web_result = web.search_exact(&query, 2).unwrap();
    let native_result = native.search(&SearchMode::Exact, &query, 2).unwrap();

    assert_eq!(web_result.k_returned, native_result.k_returned);
    for (wh, nh) in web_result.hits.iter().zip(native_result.hits.iter()) {
        assert_eq!(wh.chunk_id, nh.chunk_id);
        assert!(
            (wh.score - nh.score).abs() < 1e-6,
            "{} vs {}",
            wh.score,
            nh.score
        );
        assert_eq!(wh.source_uri, nh.source_uri);
        assert_eq!(wh.offset_start, nh.offset_start);
        assert_eq!(wh.citation_id, nh.citation_id);
    }
    assert_eq!(web_result.route, "exact");
    assert_eq!(web_result.recall, 1.0);
    assert_eq!(web_result.rerank_disclosure, "real cosine");
}

#[test]
fn float16_embeddings_decode_at_parity() {
    let bytes = NestFileBuilder::new(demo_manifest())
        .add_chunks(demo_chunks())
        .embedding_dtype(nest_format::EmbeddingDType::Float16)
        .build_bytes()
        .unwrap();
    let (web, native) = both(&bytes);

    let query = [1.0, 0.1, 0.0, 0.0];
    let web_result = web.search_exact(&query, 2).unwrap();
    let native_result = native.search(&SearchMode::Exact, &query, 2).unwrap();
    assert_eq!(web_result.hits.len(), native_result.hits.len());
    for (wh, nh) in web_result.hits.iter().zip(native_result.hits.iter()) {
        assert_eq!(wh.chunk_id, nh.chunk_id);
        assert!(
            (wh.score - nh.score).abs() < 1e-3,
            "{} vs {}",
            wh.score,
            nh.score
        );
    }
    assert_eq!(
        web_result.rerank_disclosure,
        "real cosine at stored precision"
    );
}

#[test]
fn hostile_and_unsupported_inputs_error_cleanly() {
    // Garbage.
    assert!(NestBytes::open("x".into(), vec![0u8; 16]).is_err());
    // A zstd-compressed text section is a clean "desktop only" error, not
    // a panic.
    let bytes = NestFileBuilder::new(demo_manifest())
        .add_chunks(demo_chunks())
        .text_encoding(nest_format::SectionEncoding::Zstd)
        .build_bytes()
        .unwrap();
    let result = NestBytes::open("z.nest".into(), bytes);
    let err = result.err().expect("zstd text sections must fail open");
    assert!(err.contains("desktop app"), "{err}");
}

#[test]
fn benchmark_runs_on_the_web_reader() {
    let bytes = NestFileBuilder::new(demo_manifest())
        .add_chunks(demo_chunks())
        .build_bytes()
        .unwrap();
    let web = NestBytes::open("b.nest".into(), bytes).unwrap();
    let view = web.benchmark(4, 2, &|_| {}).unwrap();
    assert_eq!(view.n_queries, 4);
    assert!(view.ann.is_none());
    assert!(view.exact.p50 >= view.exact.min);
}
