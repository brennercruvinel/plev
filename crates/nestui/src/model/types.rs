//! Target-independent view models: everything the screens render and the
//! command/event vocabulary the worker speaks. Both backends (the native
//! mmap `backend`/`worker` pair and the in-memory web `nestread`/`web`
//! pair) produce these exact shapes, so the explorer never names
//! nest-runtime/nest-format types and compiles unchanged on wasm.

use std::path::PathBuf;

use serde::Deserialize;

// ---------------------------------------------------------------------------
// Manifest / inspect document
// ---------------------------------------------------------------------------

/// The v1 capability bools (the manifest's `capabilities` object).
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct CapabilitiesView {
    pub supports_exact: bool,
    pub supports_ann: bool,
    pub supports_bm25: bool,
    pub supports_citations: bool,
    pub supports_reproducible_build: bool,
}

/// The additive capability flags (`capabilities_ext`), each absent when
/// unset.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct CapabilitiesExtView {
    pub supports_multimodal: Option<bool>,
    pub graph_present: Option<bool>,
    pub blobs_present: Option<bool>,
}

/// The manifest fields the UI renders. Deserializes from the manifest
/// object inside `inspect_json()` (native) and from the raw manifest
/// section (web); unknown keys are ignored.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct ManifestView {
    pub format_version: u32,
    pub schema_version: u32,
    pub embedding_model: String,
    pub embedding_dim: u32,
    pub n_chunks: u64,
    pub dtype: String,
    pub metric: String,
    pub score_type: String,
    pub normalize: String,
    pub index_type: String,
    pub rerank_policy: String,
    pub model_hash: String,
    pub chunker_version: String,
    pub capabilities: CapabilitiesView,
    pub capabilities_ext: Option<CapabilitiesExtView>,
    pub title: Option<String>,
    pub version: Option<String>,
    pub created: Option<String>,
    pub description: Option<String>,
}

/// One row of the .nest section table, from the inspect document.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct SectionInfo {
    pub section_id: u32,
    pub name: String,
    pub encoding: u32,
    pub offset: u64,
    pub size: u64,
    pub checksum: String,
}

/// Typed inspect document: header fields, manifest, section table, hashes
/// and the SIMD backend line.
#[derive(Clone, Debug, Deserialize)]
pub struct InspectView {
    pub magic: String,
    pub version_major: u32,
    pub version_minor: u32,
    pub format_version: u32,
    pub schema_version: u32,
    pub embedding_dim: u32,
    pub n_chunks: u64,
    pub n_embeddings: u64,
    pub file_size: u64,
    pub manifest: ManifestView,
    pub sections: Vec<SectionInfo>,
    /// blob_refs table when the file has the blob capability, else null.
    #[serde(default)]
    pub blobs: serde_json::Value,
    pub file_hash: String,
    pub content_hash: String,
    pub simd_backend: String,
}

// ---------------------------------------------------------------------------
// Opened database snapshot
// ---------------------------------------------------------------------------

/// Snapshot of an opened database, sent to the UI on `Open`. Everything a
/// screen needs to render chrome (capabilities, sizes, hashes) without
/// touching the backend again; heavy payloads (canonical texts) load on
/// demand via `LoadChunks`.
#[derive(Clone, Debug)]
pub struct OpenedDbView {
    /// File path on desktop; the picked file's name on the web.
    pub path: PathBuf,
    pub inspect: InspectView,
    pub chunk_ids: Vec<String>,
    pub has_ann: bool,
    pub has_bm25: bool,
    pub has_graph: bool,
    pub has_spaces: bool,
    pub space_names: Vec<String>,
    /// Graph node count when the CSR section is present (the full
    /// adjacency loads on demand via `LoadGraph`).
    pub graph_nodes: Option<usize>,
}

// ---------------------------------------------------------------------------
// Chunks
// ---------------------------------------------------------------------------

/// Per-chunk source location (the original-spans section), parallel to
/// `ChunksData::texts` and `OpenedDbView::chunk_ids`.
#[derive(Clone, Debug, PartialEq)]
pub struct ChunkMeta {
    pub source_uri: String,
    pub offset_start: u64,
    pub offset_end: u64,
}

/// What `LoadChunks` delivers: canonical texts plus source spans.
#[derive(Clone, Debug)]
pub struct ChunksData {
    pub texts: Vec<String>,
    pub metas: Vec<ChunkMeta>,
}

// ---------------------------------------------------------------------------
// Search
// ---------------------------------------------------------------------------

/// Which search path a `SearchByVector` command takes. Parameters mirror
/// the runtime's search methods; every path reranks to a real cosine.
/// The web backend supports `Exact` only (the other modes fall back to it,
/// mirroring the runtime's own no-section fallback).
#[derive(Clone, Debug, PartialEq)]
pub enum SearchMode {
    /// Exact flat scan — the recall=1.0 ground truth.
    Exact,
    /// HNSW shortlist + exact rerank (falls back to exact without an ANN
    /// section).
    Ann { ef_search: usize },
    /// Exact seed + bounded BFS over the chunk graph + exact rerank.
    Graph { hops: usize, ef: usize },
    /// BM25 ∪ vector shortlist, RRF fusion, exact rerank.
    Hybrid {
        query_text: String,
        candidates_per_path: usize,
    },
}

/// UI-facing search hit: `SearchHit` flattened to owned plain data.
#[derive(Clone, Debug, PartialEq)]
pub struct SearchHitView {
    pub chunk_id: String,
    pub score: f32,
    pub source_uri: String,
    pub offset_start: u64,
    pub offset_end: u64,
    pub citation_id: String,
    pub reranked: bool,
}

/// UI-facing search result: hits plus the explain panel's provenance
/// (route, candidate counts, fusion, rerank-source honesty, recall).
#[derive(Clone, Debug)]
pub struct SearchResultsView {
    pub hits: Vec<SearchHitView>,
    pub query_time_ms: f64,
    pub index_type: String,
    /// `NaN` on candidate-generating paths ("not computed").
    pub recall: f32,
    pub truncated: bool,
    pub k_requested: i32,
    pub k_returned: usize,
    pub route: String,
    pub exact_candidates: usize,
    pub ann_candidates: usize,
    pub bm25_candidates: usize,
    pub graph_candidates: usize,
    pub fusion_mode: String,
    /// The honesty marker: "real cosine" | "real cosine at stored precision".
    pub rerank_disclosure: String,
    pub recall_estimate: f32,
}

// ---------------------------------------------------------------------------
// Worker command/event vocabulary (native thread + web inline worker)
// ---------------------------------------------------------------------------

/// Requests the worker understands.
#[derive(Clone, Debug)]
pub enum NestCommand {
    /// Open (or replace) the database at `path` (desktop).
    Open(PathBuf),
    /// Open from in-memory bytes (web file picker; `name` is the picked
    /// file's name, used where a path would be shown).
    OpenBytes {
        name: String,
        bytes: Vec<u8>,
    },
    /// Vector search against the open database.
    SearchByVector {
        query: Vec<f32>,
        mode: SearchMode,
        k: i32,
    },
    /// Text search: embed `query` offline via the potion bridge (desktop
    /// only; the web worker reports it unsupported).
    SearchByText {
        query: String,
        mode: SearchMode,
        k: i32,
    },
    /// Decode and cache the canonical chunk texts (expensive on first call).
    LoadChunks,
    /// Copy the CSR graph into a view-owned `GraphData` and run the
    /// deterministic force layout (O(n²) — off the UI thread on desktop,
    /// inline on the web).
    LoadGraph,
    /// Latency benchmark: `n_queries` deterministic random queries at
    /// `k`, plus ANN + recall when the file has an HNSW section (web runs
    /// the exact leg only).
    Benchmark {
        n_queries: usize,
        k: i32,
    },
    /// Probe the python embedder (desktop; web reports unavailable).
    CheckEmbedder,
    Shutdown,
}

/// Results delivered to the UI, one per command. Payloads are `String`
/// errors (not typed) because the view only ever displays them. `Opened`
/// boxes its snapshot: it dwarfs the other variants (an inspect document
/// plus every chunk id) and is delivered exactly once per open.
#[derive(Debug)]
pub enum NestEvent {
    Opened(Result<Box<OpenedDbView>, String>),
    /// Hits plus explain provenance, already flattened to view models.
    SearchResults(Result<SearchResultsView, String>),
    ChunksLoaded(Result<ChunksData, String>),
    /// The laid-out graph (positions included — computed by the worker).
    GraphLoaded(Result<crate::model::graph::GraphScene, String>),
    /// Benchmark progress heartbeat (`done` of `total` queries).
    BenchmarkProgress {
        done: usize,
        total: usize,
    },
    BenchmarkDone(Result<crate::model::bench::BenchmarkView, String>),
    /// Embedder probe: `Ok` carries a short status line, `Err` the reason
    /// text search is unavailable.
    EmbedderStatus(Result<String, String>),
}
