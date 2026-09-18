//! Native .urna backend: wraps `MmapUrnaFile` and exposes a typed,
//! UI-ready view of the database (inspect document, chunk ids, canonical
//! texts, spans, optional CSR graph, media blobs, multimodal spaces) plus
//! the search entry points.
//!
//! View-model types live in `model::types` (shared with the web backend);
//! this module is mmap + std::fs based and compiles native-only.
//!
//! The file is mapped once more here (`memmap2`) besides the runtime's own
//! private map: every section the UI decodes itself (spans, overlay, graph
//! adjacency, blob offset table) is read through a `UrnaView` over that
//! map, so a multi-GB corpus never gets copied into RAM.

use std::path::{Path, PathBuf};

use memmap2::Mmap;
use sha2::{Digest, Sha256};
use urna_format::layout::{
    SECTION_BLOB_DATA, SECTION_BLOB_SPAN_OVERLAY, SECTION_CHUNKS_ORIGINAL_SPANS,
    SECTION_GRAPH_ADJACENCY,
};
use urna_format::reader::UrnaView;
use urna_format::{BLOB_REF_NONE, BlobSpanEntry, decode_blob_data_table, decode_blob_span_overlay};
use urna_runtime::graph::CsrIndex;
use urna_runtime::{MmapUrnaFile, RuntimeError, SearchResult};

use super::types::{
    BlobSpan, ChunkMeta, ChunksData, InspectView, OpenedDbView, SearchHitView, SearchMode,
    SearchResultsView,
};

/// Errors produced by [`UrnaBackend`] operations.
#[derive(Debug, thiserror::Error)]
pub enum BackendError {
    /// The urna runtime rejected the file or the query.
    #[error(transparent)]
    Runtime(#[from] RuntimeError),
    /// Mapping or writing a file failed at the OS level.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// The urna format reader rejected the raw file bytes.
    #[error(transparent)]
    Format(#[from] urna_format::UrnaError),
    /// `inspect_json()` output did not match the expected document shape.
    #[error("inspect document parse error: {0}")]
    Inspect(#[from] serde_json::Error),
    /// A media operation the file cannot serve (no inlined bytes, a chunk
    /// outside every blob, a corrupt blob…).
    #[error("{0}")]
    Media(String),
}

pub type Result<T> = std::result::Result<T, BackendError>;

impl From<SearchResult> for SearchResultsView {
    fn from(r: SearchResult) -> Self {
        Self {
            hits: r
                .hits
                .into_iter()
                .map(|h| SearchHitView {
                    chunk_id: h.chunk_id,
                    score: h.score,
                    source_uri: h.source_uri,
                    offset_start: h.offset_start,
                    offset_end: h.offset_end,
                    citation_id: h.citation_id,
                    reranked: h.reranked,
                })
                .collect(),
            query_time_ms: r.query_time_ms,
            index_type: r.index_type.to_string(),
            recall: r.recall,
            truncated: r.truncated,
            k_requested: r.k_requested,
            k_returned: r.k_returned,
            route: r.explain.route.to_string(),
            exact_candidates: r.explain.exact_candidates,
            ann_candidates: r.explain.ann_candidates,
            bm25_candidates: r.explain.bm25_candidates,
            graph_candidates: r.explain.graph_candidates,
            fusion_mode: r.explain.fusion_mode.to_string(),
            rerank_disclosure: r.explain.rerank_source.disclosure().to_string(),
            recall_estimate: r.explain.recall_estimate,
        }
    }
}

/// Where one inlined blob's bytes sit in the file: absolute offset + len.
/// Resolved once at open from the 0x17 offset table so a frame decode can
/// hand ffmpeg a byte range instead of a copy of the blob.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlobRange {
    pub abs_start: u64,
    pub len: u64,
}

// ---------------------------------------------------------------------------
// UrnaBackend
// ---------------------------------------------------------------------------

/// Owns the opened .urna file and every derived index. Not `Sync` — the
/// worker thread is its only owner (see `worker.rs`).
pub struct UrnaBackend {
    file: MmapUrnaFile,
    path: PathBuf,
    /// Read-only map for the sections the UI decodes itself.
    map: Mmap,
    /// CSR adjacency, loaded eagerly when the file declares `graph_present`.
    graph: Option<CsrIndex>,
    /// The 0x16 overlay (one entry per chunk) when the file has one.
    overlay: Option<Vec<BlobSpanEntry>>,
    /// Absolute byte ranges of the inlined blobs (parallel to blob_refs;
    /// `None` for an out-of-line record), when 0x17 is present.
    blob_ranges: Option<Vec<Option<BlobRange>>>,
    /// Canonical chunk texts, decoded on first use (the decode re-parses
    /// the whole section table, so it is cached here).
    canonical_texts: Option<Vec<String>>,
}

impl UrnaBackend {
    /// Open a .urna file read-only and derive the optional graph, overlay
    /// and blob tables.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let file = MmapUrnaFile::open(&path)?;
        // SAFETY: the map is read-only and the file is treated as immutable
        // for the life of the backend (the runtime holds its own map of the
        // same bytes under the same assumption).
        let map = unsafe { Mmap::map(&std::fs::File::open(&path)?)? };
        let view = UrnaView::from_bytes(&map)?;
        // Mirror `MmapUrnaFile::open`'s graph gate (manifest capability +
        // section presence) — the runtime keeps its CSR private, so the UI
        // parses its own copy for graph screens.
        let graph = if file.has_graph() {
            let payload = view.decoded_section(SECTION_GRAPH_ADJACENCY)?;
            Some(CsrIndex::from_bytes(&payload, file.n_embeddings())?)
        } else {
            None
        };
        let overlay = if file.has_blobs() && view.entry(SECTION_BLOB_SPAN_OVERLAY).is_ok() {
            Some(decode_blob_span_overlay(
                &view.decoded_section(SECTION_BLOB_SPAN_OVERLAY)?,
            )?)
        } else {
            None
        };
        let blob_ranges = match (file.blob_refs(), view.entry(SECTION_BLOB_DATA)) {
            (Some(refs), Ok(entry)) => {
                let table = decode_blob_data_table(view.get_section_data(SECTION_BLOB_DATA)?)?;
                let data_start = entry.offset + table.data_start as u64;
                Some(
                    refs.iter()
                        .zip(&table.entries)
                        .map(|(r, &(off, len))| {
                            r.inlined.then_some(BlobRange {
                                abs_start: data_start + off,
                                len,
                            })
                        })
                        .collect(),
                )
            }
            _ => None,
        };
        Ok(Self {
            file,
            path,
            map,
            graph,
            overlay,
            blob_ranges,
            canonical_texts: None,
        })
    }

    /// Parse `inspect_json()` into the typed view model.
    pub fn inspect(&self) -> Result<InspectView> {
        Ok(serde_json::from_str(&self.file.inspect_json()?)?)
    }

    /// Build the snapshot shipped to the UI when a database opens.
    pub fn opened_view(&self) -> Result<OpenedDbView> {
        Ok(OpenedDbView {
            path: self.path.clone(),
            inspect: self.inspect()?,
            chunk_ids: self.file.chunk_ids().to_vec(),
            has_ann: self.file.has_ann(),
            has_bm25: self.file.has_bm25(),
            has_graph: self.file.has_graph(),
            has_spaces: self.file.has_spaces(),
            space_names: self
                .file
                .space_names()
                .iter()
                .map(|s| s.to_string())
                .collect(),
            has_blob_data: self.file.has_blob_data(),
            graph_nodes: self.graph.as_ref().map(CsrIndex::n_nodes),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Chunk ids in file order (decoded at open time, so cheap).
    pub fn chunk_ids(&self) -> &[String] {
        self.file.chunk_ids()
    }

    /// Canonical (TIER-1 citation) text of every chunk, decoded lazily and
    /// cached; parallel to `chunk_ids()`.
    pub fn canonical_texts(&mut self) -> Result<&[String]> {
        if self.canonical_texts.is_none() {
            self.canonical_texts = Some(self.file.canonical_texts()?);
        }
        Ok(self.canonical_texts.as_deref().unwrap_or(&[]))
    }

    /// Canonical texts + per-chunk source spans for the Chunks screen.
    ///
    /// `canonical_texts` decodes the whole chunks_canonical section in one
    /// pass and the backend caches it — there is no paged decode API in
    /// urna-runtime. The worker ships the decoded `Vec<String>` to the UI
    /// once; the Chunks screen windows the *rendering* with `VirtualList`.
    /// Spans come from the overlay when the chunk lives in a media blob
    /// (what search hits and `cite` report), else from 0x03.
    pub fn load_chunks(&mut self) -> Result<ChunksData> {
        let texts = self.canonical_texts()?.to_vec();
        let view = UrnaView::from_bytes(&self.map)?;
        let n = self.file.n_embeddings();
        let spans = urna_format::sections::decode_chunks_original_spans(
            &view.decoded_section(SECTION_CHUNKS_ORIGINAL_SPANS)?,
            n,
        )?;
        let refs = self.file.blob_refs().unwrap_or(&[]);
        let metas = spans
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let blob = self
                    .overlay
                    .as_ref()
                    .and_then(|o| o.get(i))
                    .filter(|e| e.blob_ref_index != BLOB_REF_NONE)
                    .map(|e| BlobSpan {
                        blob_index: e.blob_ref_index,
                        start: e.byte_start,
                        end: e.byte_end,
                    });
                match blob.and_then(|b| refs.get(b.blob_index as usize).map(|r| (b, r))) {
                    Some((b, r)) => ChunkMeta {
                        source_uri: r.original_uri.clone(),
                        offset_start: b.start,
                        offset_end: b.end,
                        blob: Some(b),
                    },
                    None => ChunkMeta {
                        source_uri: s.source_uri.clone(),
                        offset_start: s.byte_start,
                        offset_end: s.byte_end,
                        blob: None,
                    },
                }
            })
            .collect();
        Ok(ChunksData { texts, metas })
    }

    /// The chunk-to-chunk CSR graph, when present.
    pub fn graph(&self) -> Option<&CsrIndex> {
        self.graph.as_ref()
    }

    /// Corpus identity for the embedder gate: `(embedding_model,
    /// embedding_dim, model_hash)` from the manifest.
    pub fn embed_identity(&self) -> Result<(String, usize, String)> {
        let inspect = self.inspect()?;
        Ok((
            inspect.manifest.embedding_model,
            inspect.manifest.embedding_dim as usize,
            inspect.manifest.model_hash,
        ))
    }

    /// One space's identity for the per-space embedder gate: `(dim,
    /// model_hash)`, from the space_table.
    pub fn space_identity(&self, name: &str) -> Result<(usize, String)> {
        let inspect = self.inspect()?;
        inspect
            .spaces
            .iter()
            .find(|s| s.name == name)
            .map(|s| (s.dim as usize, s.model_hash.clone()))
            .ok_or_else(|| RuntimeError::SpaceNotFound(name.to_string()).into())
    }

    /// The CSR graph copied into the engine's
    /// [`GraphData`](engine::graph::GraphData) (the runtime's `CsrIndex`
    /// never leaves this module).
    pub fn graph_data(&self) -> Option<engine::graph::GraphData> {
        let g = self.graph.as_ref()?;
        let n = g.n_nodes();
        let mut data = engine::graph::GraphData {
            n_nodes: n,
            offsets: Vec::with_capacity(n + 1),
            neighbors: Vec::new(),
            kinds: Vec::new(),
        };
        data.offsets.push(0);
        for node in 0..n {
            for (i, &dst) in g.neighbors(node).iter().enumerate() {
                data.neighbors.push(dst);
                data.kinds.push(g.edge_type(node, i).unwrap_or(0));
            }
            data.offsets.push(data.neighbors.len() as u32);
        }
        Some(data)
    }

    /// Run a search down the requested path. Every mode returns the real
    /// cosine score (candidate generators rerank exactly); `Space` scores
    /// the named band and never touches the text slab.
    pub fn search(&self, mode: &SearchMode, query: &[f32], k: i32) -> Result<SearchResult> {
        let result = match mode {
            SearchMode::Exact => self.file.search(query, k)?,
            SearchMode::Ann { ef_search } => self.file.search_ann(query, k, *ef_search)?,
            SearchMode::Graph { hops, ef } => self.file.search_graph(query, k, *hops, *ef)?,
            SearchMode::Hybrid {
                query_text,
                candidates_per_path,
            } => self
                .file
                .search_hybrid(query, query_text, k, *candidates_per_path)?,
            SearchMode::Space { name } => self.file.search_space(name, query, k, None)?,
        };
        Ok(result)
    }

    /// Re-run every reader-side check (`urna validate`): checksums, hashes,
    /// embedding values, search contract. Returns the wall time in ms.
    pub fn validate(&self) -> Result<f64> {
        let t0 = std::time::Instant::now();
        self.file.revalidate()?;
        Ok(t0.elapsed().as_secs_f64() * 1000.0)
    }

    // -- media ---------------------------------------------------------------

    /// Absolute byte range of blob `index` when its bytes are inlined.
    pub fn blob_range(&self, index: usize) -> Option<BlobRange> {
        self.blob_ranges.as_ref()?.get(index).copied().flatten()
    }

    /// The overlay span of chunk `ordinal`, when it maps into a blob.
    pub fn blob_span(&self, ordinal: usize) -> Option<BlobSpan> {
        self.overlay
            .as_ref()?
            .get(ordinal)
            .filter(|e| e.blob_ref_index != BLOB_REF_NONE)
            .map(|e| BlobSpan {
                blob_index: e.blob_ref_index,
                start: e.byte_start,
                end: e.byte_end,
            })
    }

    /// Write blob `index`'s inlined bytes to `dest`, proven against its
    /// blob_refs content_hash BEFORE writing (the `urna media --export`
    /// contract: a corrupt section never fans out to disk).
    pub fn export_blob(&self, index: usize, dest: &Path) -> Result<()> {
        let record = self
            .file
            .blob_refs()
            .and_then(|r| r.get(index))
            .ok_or_else(|| BackendError::Media(format!("no blob at index {index}")))?;
        let bytes = self.file.blob_bytes(index)?;
        if Sha256::digest(bytes)[..] != record.content_hash {
            return Err(BackendError::Media(format!(
                "blob {index} ({}) failed its content_hash check; refusing to export",
                record.original_uri
            )));
        }
        std::fs::write(dest, bytes)?;
        Ok(())
    }

    /// Latency benchmark, port of `urna benchmark`'s methodology: `n`
    /// deterministic random normalized queries timed through exact search,
    /// plus ANN latency and recall@k when an HNSW section exists. `ef` is
    /// the ANN search width; `progress` fires every few queries so the UI
    /// can show progress instead of a bare spinner.
    pub fn benchmark(
        &self,
        n_queries: usize,
        k: i32,
        ef: usize,
        progress: &dyn Fn(usize),
    ) -> Result<crate::model::bench::BenchmarkView> {
        use crate::model::bench::{BenchmarkView, gen_queries, latency_stats, recall_at_k};

        let dim = self.file.embedding_dim();
        let queries = gen_queries(dim, n_queries);

        let mut exact_times = Vec::with_capacity(n_queries);
        for (i, q) in queries.iter().enumerate() {
            let t0 = std::time::Instant::now();
            self.file.search(q, k)?;
            exact_times.push(t0.elapsed().as_secs_f64() * 1000.0);
            if i % 8 == 0 {
                progress(i);
            }
        }

        let (ann, recall) = if self.file.has_ann() {
            let mut ann_times = Vec::with_capacity(n_queries);
            let mut recall_total = 0.0f64;
            for q in &queries {
                let t0 = std::time::Instant::now();
                let approx = self.file.search_ann(q, k, ef)?;
                ann_times.push(t0.elapsed().as_secs_f64() * 1000.0);
                // recall@k against the exact ground truth, same query.
                let exact = self.file.search(q, k)?;
                let exact_ids: Vec<&str> = exact.hits.iter().map(|h| h.chunk_id.as_str()).collect();
                let approx_ids: Vec<&str> =
                    approx.hits.iter().map(|h| h.chunk_id.as_str()).collect();
                recall_total += recall_at_k(&exact_ids, &approx_ids, k as usize) as f64;
            }
            (
                Some(latency_stats(&ann_times)),
                Some((recall_total / n_queries as f64) as f32),
            )
        } else {
            (None, None)
        };
        progress(n_queries);

        Ok(BenchmarkView {
            n_queries,
            k,
            dim,
            dtype: self.file.dtype().name().to_string(),
            simd_backend: self.file.simd_backend().name().to_string(),
            exact: latency_stats(&exact_times),
            ann,
            recall_at_k: recall,
        })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal but shape-faithful inspect document: every required field of
    /// `InspectView` plus a v1 manifest, a blob and a space.
    fn synthetic_inspect_json() -> String {
        serde_json::json!({
            "magic": "URNA",
            "version_major": 1,
            "version_minor": 0,
            "format_version": 1,
            "schema_version": 1,
            "embedding_dim": 4,
            "n_chunks": 3,
            "n_embeddings": 3,
            "file_size": 4096,
            "manifest": {
                "format_version": 1,
                "schema_version": 1,
                "embedding_model": "demo-model",
                "embedding_dim": 4,
                "n_chunks": 3,
                "dtype": "float32",
                "metric": "cosine",
                "score_type": "cosine",
                "normalize": "l2",
                "index_type": "flat",
                "rerank_policy": "exact",
                "model_hash": format!("sha256:{}", "0".repeat(64)),
                "chunker_version": "demo-chunker/1",
                "capabilities": {
                    "supports_exact": true,
                    "supports_ann": false,
                    "supports_bm25": false,
                    "supports_citations": true,
                    "supports_reproducible_build": true
                },
                "capabilities_ext": { "graph_present": true }
            },
            "sections": [
                {
                    "section_id": 1,
                    "name": "chunk_ids",
                    "encoding": 0,
                    "offset": 128,
                    "size": 256,
                    "checksum": "ab12"
                },
                {
                    "section_id": 4,
                    "name": "embeddings",
                    "encoding": 0,
                    "offset": 448,
                    "size": 48,
                    "checksum": "cd34"
                }
            ],
            "blobs": [
                {
                    "content_hash": format!("sha256:{}", "3".repeat(64)),
                    "original_uri": "media://cards-av1.mp4",
                    "byte_len": 1234,
                    "inlined": true
                }
            ],
            "spaces": [
                {
                    "name": "clip-vit-b32",
                    "space_index": 1,
                    "dim": 512,
                    "dtype": "int8",
                    "model_hash": format!("sha256:{}", "4".repeat(64)),
                    "n_vectors": 3,
                    "band_bytes": 1536
                }
            ],
            "file_hash": format!("sha256:{}", "1".repeat(64)),
            "content_hash": format!("sha256:{}", "2".repeat(64)),
            "simd_backend": "neon"
        })
        .to_string()
    }

    #[test]
    fn inspect_view_parses_synthetic_document() {
        let view: InspectView = serde_json::from_str(&synthetic_inspect_json()).unwrap();
        assert_eq!(view.magic, "URNA");
        assert_eq!(view.embedding_dim, 4);
        assert_eq!(view.n_chunks, 3);
        assert_eq!(view.sections.len(), 2);
        assert_eq!(view.sections[0].name, "chunk_ids");
        assert_eq!(view.manifest.embedding_model, "demo-model");
        assert!(view.manifest.capabilities.supports_exact);
        assert!(!view.manifest.capabilities.supports_ann);
        assert_eq!(
            view.manifest
                .capabilities_ext
                .as_ref()
                .and_then(|e| e.graph_present),
            Some(true)
        );
        assert_eq!(view.simd_backend, "neon");
        assert_eq!(view.blobs.len(), 1);
        assert_eq!(view.blobs[0].original_uri, "media://cards-av1.mp4");
        assert!(view.blobs[0].inlined);
        assert_eq!(view.spaces.len(), 1);
        assert_eq!(view.spaces[0].name, "clip-vit-b32");
        assert_eq!(view.spaces[0].dim, 512);
    }

    #[test]
    fn inspect_view_accepts_null_blobs_and_spaces() {
        let doc = synthetic_inspect_json()
            .replace(r#""blobs":[{"#, r#""blobs":null,"unused":[{"#)
            .replace(r#""spaces":[{"#, r#""spaces":null,"unused2":[{"#);
        let view: InspectView = serde_json::from_str(&doc).unwrap();
        assert!(view.blobs.is_empty());
        assert!(view.spaces.is_empty());
    }

    #[test]
    fn inspect_view_rejects_missing_fields() {
        let doc = serde_json::json!({ "magic": "URNA" }).to_string();
        assert!(serde_json::from_str::<InspectView>(&doc).is_err());
    }

    #[test]
    fn opening_a_missing_file_errors() {
        let result = UrnaBackend::open("/definitely/not/a/real/file.urna");
        assert!(matches!(
            result.err(),
            Some(BackendError::Runtime(RuntimeError::Io(_)))
        ));
    }
}
