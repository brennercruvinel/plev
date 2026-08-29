//! Integration tests for `NestBackend` against real .nest files: a
//! synthetic corpus built with `nest_format::NestFileBuilder` (always
//! available) and the checked-in corpus fixture (skipped silently when the
//! git-lfs payload is absent).

#![cfg(not(target_arch = "wasm32"))]

use std::path::PathBuf;

use nest_format::{ChunkInput, Edge, Manifest, NestFileBuilder, encode_graph_adjacency};
use nestui::model::backend::NestBackend;
use nestui::model::types::SearchMode;

/// Build a small 3-chunk float32 corpus in a tempdir and return its path
/// (the tempdir guard is leaked so the path outlives the test body — the
/// OS reaps it).
fn synthetic_corpus() -> PathBuf {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("synthetic.nest");
    let manifest = Manifest {
        embedding_model: "nestui-test".into(),
        embedding_dim: 4,
        n_chunks: 3,
        chunker_version: "nestui-test/1".into(),
        model_hash: format!("sha256:{}", "0".repeat(64)),
        ..Default::default()
    };
    let chunks = ["alpha", "beta", "gamma"]
        .iter()
        .enumerate()
        .map(|(i, text)| {
            let mut embedding = vec![0.0f32; 4];
            embedding[i % 4] = 1.0;
            ChunkInput {
                canonical_text: text.to_string(),
                source_uri: "corpus.txt".into(),
                byte_start: (i * 8) as u64,
                byte_end: (i * 8 + text.len()) as u64,
                embedding,
            }
        });
    NestFileBuilder::new(manifest)
        .add_chunks(chunks)
        .write_to_path(&path)
        .unwrap();
    std::mem::forget(dir);
    path
}

#[test]
fn opens_searches_and_decodes_a_synthetic_corpus() {
    let backend = NestBackend::open(synthetic_corpus()).unwrap();

    let view = backend.opened_view().unwrap();
    assert_eq!(view.inspect.embedding_dim, 4);
    assert_eq!(view.inspect.n_chunks, 3);
    assert_eq!(view.chunk_ids.len(), 3);
    assert!(!view.has_ann);
    assert!(!view.has_graph);
    assert_eq!(view.graph_nodes, None);

    // Exact search: the query is the first basis vector, so chunk 0
    // ("alpha", embedding [1,0,0,0]) must score highest.
    let mut backend = backend;
    let result = backend
        .search(&SearchMode::Exact, &[1.0, 0.0, 0.0, 0.0], 2)
        .unwrap();
    assert_eq!(result.k_returned, 2);
    assert_eq!(result.index_type, "exact");
    assert_eq!(result.recall, 1.0);
    assert_eq!(result.hits[0].score, 1.0);

    let texts = backend.canonical_texts().unwrap();
    assert_eq!(texts, &["alpha", "beta", "gamma"]);
    // Second call hits the cache and returns the same data.
    let cached = backend.canonical_texts().unwrap();
    assert_eq!(cached, &["alpha", "beta", "gamma"]);
}

/// A corpus with a graph_adjacency section: the backend must load the CSR
/// GUI-side (via NestView + CsrIndex, mirroring the runtime's open gate).
#[test]
fn loads_the_csr_graph_when_the_capability_is_set() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("graphed.nest");
    let manifest = Manifest {
        embedding_model: "nestui-test".into(),
        embedding_dim: 4,
        n_chunks: 3,
        chunker_version: "nestui-test/1".into(),
        model_hash: format!("sha256:{}", "0".repeat(64)),
        ..Default::default()
    };
    let chunks = ["alpha", "beta", "gamma"]
        .iter()
        .enumerate()
        .map(|(i, text)| {
            let mut embedding = vec![0.0f32; 4];
            embedding[i % 4] = 1.0;
            ChunkInput {
                canonical_text: text.to_string(),
                source_uri: "corpus.txt".into(),
                byte_start: (i * 8) as u64,
                byte_end: (i * 8 + text.len()) as u64,
                embedding,
            }
        });
    // 0 -> 1 (next-chunk), 1 -> 2 (next-chunk), 0 -> 2 (semantic).
    let edges = [
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
    ];
    let payload = encode_graph_adjacency(3, &edges).unwrap();
    NestFileBuilder::new(manifest)
        .add_chunks(chunks)
        .graph_adjacency(payload)
        .write_to_path(&path)
        .unwrap();

    let backend = NestBackend::open(&path).unwrap();
    let view = backend.opened_view().unwrap();
    assert!(view.has_graph);
    assert_eq!(view.graph_nodes, Some(3));

    let graph = backend.graph().unwrap();
    assert_eq!(graph.n_nodes(), 3);
    // Canonical edge order is (src, edge_type, dst): node 0's edges are
    // [NEXT_CHUNK->1, SEMANTIC->2].
    assert_eq!(graph.neighbors(0), &[1, 2]);
    assert_eq!(graph.neighbors(1), &[2]);
    assert_eq!(graph.neighbors(2), &[] as &[u32]);
    assert_eq!(graph.edge_type(0, 1), Some(nest_format::EDGE_TYPE_SEMANTIC));

    // Graph search reranks to real cosines like every other path.
    let result = backend
        .search(
            &SearchMode::Graph { hops: 1, ef: 2 },
            &[1.0, 0.0, 0.0, 0.0],
            2,
        )
        .unwrap();
    assert_eq!(result.index_type, "graph");
    assert_eq!(result.hits[0].score, 1.0);

    std::mem::forget(dir);
}

/// Benchmark runs end-to-end against the synthetic corpus (no ANN
/// section → no recall numbers), and the progress callback fires.
#[test]
fn benchmark_runs_against_a_synthetic_corpus() {
    let backend = NestBackend::open(synthetic_corpus()).unwrap();
    let ticks = std::cell::Cell::new(0);
    let view = backend
        .benchmark(8, 2, 8, &|_| ticks.set(ticks.get() + 1))
        .unwrap();
    assert_eq!(view.n_queries, 8);
    assert_eq!(view.dim, 4);
    assert_eq!(view.dtype, "float32");
    assert!(view.ann.is_none());
    assert!(view.recall_at_k.is_none());
    assert!(view.exact.p50 >= view.exact.min);
    assert!(view.exact.p99 <= view.exact.max);
    assert!(ticks.get() > 0);
}

/// The shared corpus fixture lives in the sibling nest workspace. It is a
/// git-lfs file: when the payload was never pulled (pointer file or absent
/// path), skip silently — the synthetic test above covers the same paths.
#[test]
fn opens_the_shared_fixture_when_present() {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../nest/dat/corpus_next.v1.nest");
    let Ok(backend) = NestBackend::open(&path) else {
        eprintln!("skipping: {path:?} is absent or not a valid .nest (git-lfs pointer?)");
        return;
    };

    let view = backend.opened_view().unwrap();
    assert!(view.inspect.n_chunks > 0);
    assert_eq!(view.chunk_ids.len() as u64, view.inspect.n_chunks);
    assert!(!view.inspect.file_hash.is_empty());
    assert!(!view.inspect.content_hash.is_empty());
    assert!(!view.inspect.sections.is_empty());

    let dim = view.inspect.embedding_dim as usize;
    let query = vec![0.0f32; dim];
    let mut query = query;
    query[0] = 1.0;
    let result = backend.search(&SearchMode::Exact, &query, 5).unwrap();
    assert!(result.k_returned > 0);
}

/// Dev tool: `cargo test -p nestui -- --ignored make_demo_fixture` writes a
/// small demo corpus to `target/tmp/nestui_demo.nest` so the real app can
/// be run against it (`cargo run -p nestui -- target/tmp/nestui_demo.nest`).
/// Ignored by default; the output path is gitignored build territory.
#[test]
#[ignore = "dev tool: writes target/tmp/nestui_demo.nest"]
fn make_demo_fixture() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/tmp/nestui_demo.nest");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let manifest = Manifest {
        embedding_model: "nestui-demo".into(),
        embedding_dim: 8,
        n_chunks: 6,
        chunker_version: "nestui-demo/1".into(),
        model_hash: format!("sha256:{}", "ab".repeat(32)),
        title: Some("nestui demo corpus".into()),
        ..Default::default()
    };
    let texts = [
        "the quick brown fox jumps over the lazy dog",
        "pack my box with five dozen liquor jugs",
        "how vexingly quick daft zebras jump",
        "sphinx of black quartz, judge my vow",
        "the five boxing wizards jump quickly",
        "jackdaws love my big sphinx of quartz",
    ];
    let chunks = texts.iter().enumerate().map(|(i, text)| {
        let mut embedding = vec![0.0f32; 8];
        embedding[i % 8] = 1.0;
        embedding[(i + 3) % 8] = 0.5;
        ChunkInput {
            canonical_text: text.to_string(),
            source_uri: "pangrams.txt".into(),
            byte_start: (i * 48) as u64,
            byte_end: (i * 48 + text.len()) as u64,
            embedding,
        }
    });
    let edges: Vec<Edge> = (0..5u32)
        .map(|i| Edge {
            src: i,
            dst: i + 1,
            edge_type: nest_format::EDGE_TYPE_NEXT_CHUNK,
        })
        .collect();
    let graph = encode_graph_adjacency(6, &edges).unwrap();
    NestFileBuilder::new(manifest)
        .add_chunks(chunks)
        .graph_adjacency(graph)
        .write_to_path(&path)
        .unwrap();
    eprintln!("wrote {}", path.display());

    // A larger sibling fixture (300 chunks, next-chunk chain + semantic
    // chords) so the Graph screen has something worth laying out.
    let big_path = path.with_file_name("nestui_demo_big.nest");
    let n = 300usize;
    let manifest = Manifest {
        embedding_model: "nestui-demo".into(),
        embedding_dim: 8,
        n_chunks: n as u64,
        chunker_version: "nestui-demo/1".into(),
        model_hash: format!("sha256:{}", "ab".repeat(32)),
        title: Some("nestui demo corpus (big)".into()),
        ..Default::default()
    };
    let chunks = (0..n).map(|i| {
        let mut embedding = vec![0.0f32; 8];
        embedding[i % 8] = 1.0;
        embedding[(i + 3) % 8] = 0.5;
        ChunkInput {
            canonical_text: format!("chunk {i}: the quick brown fox jumps over the lazy dog"),
            source_uri: "pangrams.txt".into(),
            byte_start: (i * 64) as u64,
            byte_end: (i * 64 + 40) as u64,
            embedding,
        }
    });
    let mut edges: Vec<Edge> = (0..(n - 1) as u32)
        .map(|i| Edge {
            src: i,
            dst: i + 1,
            edge_type: nest_format::EDGE_TYPE_NEXT_CHUNK,
        })
        .collect();
    // Deterministic semantic chords: every 7th node links +29 ahead.
    for i in (0..n).step_by(7) {
        if i + 29 < n {
            edges.push(Edge {
                src: i as u32,
                dst: (i + 29) as u32,
                edge_type: nest_format::EDGE_TYPE_SEMANTIC,
            });
        }
    }
    let graph = encode_graph_adjacency(n, &edges).unwrap();
    NestFileBuilder::new(manifest)
        .add_chunks(chunks)
        .graph_adjacency(graph)
        .write_to_path(&big_path)
        .unwrap();
    eprintln!("wrote {}", big_path.display());
}
