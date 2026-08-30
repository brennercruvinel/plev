//! Portable, dependency-free `.nest` reader over in-memory bytes — the
//! web backend's foundation, compiled and unit-tested on every target.
//!
//! Why a reimplementation: `nest-runtime` needs mmap + rayon and
//! `nest-format` pulls in `zstd` (a C build), none of which compile to
//! wasm32-unknown-unknown, and this phase must not patch the nest repo.
//! The formats are small and fixed by spec, so this module ports exactly
//! the slices the UI needs, with **parity tests against the real
//! writer/reader running natively** (see `tests`):
//!
//! - header (128 B) + section table (32 B entries) + footer (40 B)
//! - manifest JSON (raw)
//! - `chunk_ids` (raw and the intpack kind-0 repack)
//! - `chunks_canonical` / `chunks_original_spans` (raw)
//! - `graph_adjacency` CSR (intpack columns + delta-gapped neighbors)
//! - `embeddings` (raw float32; float16 decoded to f32)
//! - identity hashes (file_hash, content_hash) and per-section checksums
//!
//! Unsupported encodings (zstd, txt_streams, fsst, int8/int4 embeddings,
//! spans intpack) fail open with a clear error naming the section — those
//! files need the desktop app. Exact search is a straightforward exact
//! cosine over the decoded slab (the runtime's ground-truth path), so
//! scores are at parity by construction.

use std::path::PathBuf;

use web_time::Instant;

use super::bench::{BenchmarkView, gen_queries, latency_stats};
use super::types::{
    ChunkMeta, ChunksData, InspectView, ManifestView, OpenedDbView, SearchHitView,
    SearchResultsView, SectionInfo,
};
use engine::graph::GraphData;

// Section ids (mirror nest_format::layout; the view model never names
// nest crates).
const SECTION_CHUNK_IDS: u32 = 0x01;
const SECTION_CHUNKS_CANONICAL: u32 = 0x02;
const SECTION_CHUNKS_ORIGINAL_SPANS: u32 = 0x03;
const SECTION_EMBEDDINGS: u32 = 0x04;
const SECTION_PROVENANCE: u32 = 0x05;
const SECTION_SEARCH_CONTRACT: u32 = 0x06;
const SECTION_HNSW_INDEX: u32 = 0x07;
const SECTION_BM25_INDEX: u32 = 0x08;
const SECTION_GRAPH_ADJACENCY: u32 = 0x0C;
const SECTION_SPACE_TABLE: u32 = 0x15;

/// Section encodings (nest_format::layout).
const ENCODING_RAW: u32 = 0;
const ENCODING_FLOAT16: u32 = 2;
const ENCODING_INTPACK: u32 = 4;

/// Canonical sections, in content_hash order (fixed by spec).
const CANONICAL_SECTIONS: [(u32, &str); 6] = [
    (SECTION_CHUNK_IDS, "chunk_ids"),
    (SECTION_CHUNKS_CANONICAL, "chunks_canonical"),
    (SECTION_CHUNKS_ORIGINAL_SPANS, "chunks_original_spans"),
    (SECTION_EMBEDDINGS, "embeddings"),
    (SECTION_PROVENANCE, "provenance"),
    (SECTION_SEARCH_CONTRACT, "search_contract"),
];

const HEADER_SIZE: usize = 128;
const SECTION_ENTRY_SIZE: usize = 32;
const FOOTER_SIZE: usize = 40;
const PAYLOAD_PREFIX_SIZE: usize = 12; // u32 version + u64 count
const PAYLOAD_VERSION: u32 = 1;
/// intpack repack kinds (nest_format::sections).
const REPACK_KIND_CHUNK_IDS: u8 = 0;

pub type Result<T> = std::result::Result<T, String>;

// ---------------------------------------------------------------------------
// Little-endian cursor
// ---------------------------------------------------------------------------

struct Cursor<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        if self.pos + n > self.data.len() {
            return Err(format!(
                "unexpected EOF: want {n} bytes at {}, have {}",
                self.pos,
                self.data.len()
            ));
        }
        let out = &self.data[self.pos..self.pos + n];
        self.pos += n;
        Ok(out)
    }

    fn u8(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }

    fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    fn u64(&mut self) -> Result<u64> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }

    fn lp_str(&mut self) -> Result<String> {
        let len = self.u32()? as usize;
        let bytes = self.take(len)?;
        std::str::from_utf8(bytes)
            .map(|s| s.to_string())
            .map_err(|e| format!("invalid utf-8: {e}"))
    }

    /// The shared 12-byte payload prefix; returns the count.
    fn prefix(&mut self) -> Result<u64> {
        if self.data.len() < PAYLOAD_PREFIX_SIZE {
            return Err("payload shorter than the 12-byte prefix".to_string());
        }
        let version = self.u32()?;
        if version != PAYLOAD_VERSION {
            return Err(format!("unsupported section payload version {version}"));
        }
        self.u64()
    }
}

/// Lowercase hex of `bytes` (no `hex` crate in this workspace).
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn sha256(data: &[u8]) -> [u8; 32] {
    use sha2::Digest;
    let mut out = [0u8; 32];
    out.copy_from_slice(&sha2::Sha256::digest(data)[..]);
    out
}

// ---------------------------------------------------------------------------
// intpack (frame-of-reference bitpacking) port of `unpack_u64s`
// ---------------------------------------------------------------------------

const INTPACK_BLOCK: usize = 128;

fn unpack_u64s(bytes: &[u8]) -> Result<Vec<u64>> {
    if bytes.len() < 8 {
        return Err("intpack: truncated header".to_string());
    }
    let count = u32::from_le_bytes(bytes[0..4].try_into().unwrap()) as usize;
    let n_blocks = u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize;
    if n_blocks != count.div_ceil(INTPACK_BLOCK) {
        return Err("intpack: block count inconsistent with count".to_string());
    }
    if 8 + n_blocks * 4 > bytes.len() {
        return Err("intpack: truncated directory".to_string());
    }
    let mut out = Vec::with_capacity(count.min(1 << 20));
    for b in 0..n_blocks {
        let dir_pos = 8 + b * 4;
        let off = u32::from_le_bytes(bytes[dir_pos..dir_pos + 4].try_into().unwrap()) as usize;
        if off + 9 > bytes.len() {
            return Err("intpack: truncated block header".to_string());
        }
        let min = u64::from_le_bytes(bytes[off..off + 8].try_into().unwrap());
        let width = bytes[off + 8];
        if width > 64 {
            return Err("intpack: block width out of range".to_string());
        }
        let block_len = (count - b * INTPACK_BLOCK).min(INTPACK_BLOCK);
        let body_start = off + 9;
        let body_len = (block_len * width as usize).div_ceil(8);
        if body_start + body_len > bytes.len() {
            return Err("intpack: truncated block body".to_string());
        }
        let body = &bytes[body_start..body_start + body_len];
        for i in 0..block_len {
            let v = if width == 0 {
                0
            } else {
                let bit = i * width as usize;
                let (first, last) = (bit / 8, (bit + width as usize - 1) / 8);
                let mut acc: u128 = 0;
                for (k, &byte) in body[first..=last].iter().enumerate() {
                    acc |= (byte as u128) << (k * 8);
                }
                ((acc >> (bit % 8)) as u64) & ((1u64 << width) - 1)
            };
            out.push(min.wrapping_add(v));
        }
    }
    Ok(out)
}

/// float16 → float32 (pure bit math; the `half` crate is not a workspace
/// dep). Exact for normals/subnormals; Inf/NaN map to their f32 forms.
fn f16_to_f32(bits: u16) -> f32 {
    let sign = ((bits >> 15) & 1) as u32;
    let exp = ((bits >> 10) & 0x1f) as u32;
    let frac = (bits & 0x3ff) as u32;
    let value = match exp {
        0 => (frac as f32) * 2f32.powi(-24),
        0x1f => {
            if frac == 0 {
                f32::INFINITY
            } else {
                f32::NAN
            }
        }
        _ => (1.0 + frac as f32 / 1024.0) * 2f32.powi(exp as i32 - 15),
    };
    if sign == 1 { -value } else { value }
}

// ---------------------------------------------------------------------------
// NestBytes
// ---------------------------------------------------------------------------

/// An in-memory parsed `.nest` file.
pub struct NestBytes {
    name: String,
    bytes: Vec<u8>,
    version_major: u16,
    manifest: ManifestView,
    sections: Vec<SectionInfo>,
    n_chunks: usize,
    n_embeddings: usize,
    dim: usize,
    chunk_ids: Vec<String>,
    spans: Vec<ChunkMeta>,
    /// Decoded embeddings (f32), row-major `n * dim`.
    embeddings: Vec<f32>,
    graph: Option<GraphData>,
    file_hash: String,
    content_hash: String,
    has_spaces: bool,
}

impl NestBytes {
    /// Parse and validate the file; decodes chunk ids, spans, embeddings
    /// and the graph up front (in-memory anyway — there is no lazy mmap
    /// win to take). Canonical texts stay on demand (`chunks_data`).
    pub fn open(name: String, bytes: Vec<u8>) -> Result<Self> {
        if bytes.len() < HEADER_SIZE + FOOTER_SIZE {
            return Err("file too small for a .nest header + footer".to_string());
        }
        let mut h = Cursor::new(&bytes[..HEADER_SIZE]);
        if h.take(4)? != b"NEST" {
            return Err("bad magic: not a .nest file".to_string());
        }
        let version_major = h.u16()?;
        let _version_minor = h.u16()?;
        let _flags = h.u32()?;
        let dim = h.u32()? as usize;
        let n_chunks = h.u64()? as usize;
        let n_embeddings = h.u64()? as usize;
        let file_size = h.u64()? as usize;
        let table_offset = h.u64()? as usize;
        let table_count = h.u64()? as usize;
        let manifest_offset = h.u64()? as usize;
        let manifest_size = h.u64()? as usize;
        if version_major != 1 {
            return Err(format!("unsupported format version {version_major}"));
        }
        if file_size != bytes.len() {
            return Err(format!(
                "file_size mismatch: header says {file_size}, got {}",
                bytes.len()
            ));
        }

        // Section table + per-section checksum validation (first 8 bytes
        // of the payload's SHA-256, hex).
        if table_offset + table_count * SECTION_ENTRY_SIZE > bytes.len() {
            return Err("section table out of bounds".to_string());
        }
        let mut sections = Vec::with_capacity(table_count);
        for i in 0..table_count {
            let mut c =
                Cursor::new(&bytes[table_offset + i * SECTION_ENTRY_SIZE..][..SECTION_ENTRY_SIZE]);
            let section_id = c.u32()?;
            let encoding = c.u32()?;
            let offset = c.u64()? as usize;
            let size = c.u64()? as usize;
            let checksum = hex(c.take(8)?);
            if offset + size > bytes.len() {
                return Err(format!("section 0x{section_id:02X} out of bounds"));
            }
            let actual = hex(&sha256(&bytes[offset..offset + size])[..8]);
            if actual != checksum {
                return Err(format!("section 0x{section_id:02X} checksum mismatch"));
            }
            sections.push(SectionInfo {
                section_id,
                name: section_name(section_id).to_string(),
                encoding,
                offset: offset as u64,
                size: size as u64,
                checksum,
            });
        }

        // Manifest (raw JSON).
        if manifest_offset + manifest_size > bytes.len() {
            return Err("manifest out of bounds".to_string());
        }
        let manifest: ManifestView =
            serde_json::from_slice(&bytes[manifest_offset..manifest_offset + manifest_size])
                .map_err(|e| format!("manifest JSON parse error: {e}"))?;

        let mut file = Self {
            name,
            bytes,
            version_major,
            manifest,
            sections,
            n_chunks,
            n_embeddings,
            dim,
            chunk_ids: Vec::new(),
            spans: Vec::new(),
            embeddings: Vec::new(),
            graph: None,
            file_hash: String::new(),
            content_hash: String::new(),
            has_spaces: false,
        };
        // file_hash covers the file as written, INCLUDING the footer
        // (nest_format's `file_hash_hex`).
        file.file_hash = format!("sha256:{}", hex(&sha256(&file.bytes)));
        file.chunk_ids = file.decode_chunk_ids()?;
        file.spans = file.decode_spans()?;
        file.embeddings = file.decode_embeddings()?;
        file.content_hash = file.compute_content_hash()?;
        file.graph = file.decode_graph()?;
        file.has_spaces = file
            .sections
            .iter()
            .any(|s| s.section_id == SECTION_SPACE_TABLE);
        Ok(file)
    }

    // -- raw section access -------------------------------------------------

    fn section(&self, id: u32) -> Option<&SectionInfo> {
        self.sections.iter().find(|s| s.section_id == id)
    }

    /// Physical payload bytes of a section (bounds already validated).
    fn payload(&self, id: u32) -> Result<&[u8]> {
        let s = self
            .section(id)
            .ok_or_else(|| format!("section {} not found", section_name(id)))?;
        Ok(&self.bytes[s.offset as usize..(s.offset + s.size) as usize])
    }

    /// Decoded payload: raw bytes, or the intpack repack reconstructed to
    /// its raw form (chunk_ids only). Everything else errors with the
    /// encoding id.
    fn decoded(&self, id: u32) -> Result<Vec<u8>> {
        let s = self
            .section(id)
            .ok_or_else(|| format!("section 0x{id:02X} not found"))?;
        let phys = self.payload(id)?;
        match s.encoding {
            ENCODING_RAW => Ok(phys.to_vec()),
            // Quantized dtypes (float16/int8/int4) are physical-is-logical:
            // the runtime dispatches on the dtype, and content_hash hashes
            // the on-disk bytes by spec.
            ENCODING_FLOAT16 | 3 | 7 => Ok(phys.to_vec()),
            ENCODING_INTPACK if id == SECTION_CHUNK_IDS => {
                let mut c = Cursor::new(phys);
                let kind = c.u8()?;
                if kind != REPACK_KIND_CHUNK_IDS {
                    return Err(format!("chunk_ids intpack: unsupported repack kind {kind}"));
                }
                let count = c.u32()? as usize;
                let body = c.take(count * 32)?;
                let mut out = Vec::new();
                out.extend_from_slice(&PAYLOAD_VERSION.to_le_bytes());
                out.extend_from_slice(&(count as u64).to_le_bytes());
                for digest in body.chunks_exact(32) {
                    let id_str = format!("sha256:{}", hex(digest));
                    out.extend_from_slice(&(id_str.len() as u32).to_le_bytes());
                    out.extend_from_slice(id_str.as_bytes());
                }
                Ok(out)
            }
            other => Err(format!(
                "section {} uses encoding {other}, which the web reader does not support — \
                 open this file in the desktop app",
                s.name
            )),
        }
    }

    // -- decoders -------------------------------------------------------------

    fn decode_chunk_ids(&self) -> Result<Vec<String>> {
        let payload = self.decoded(SECTION_CHUNK_IDS)?;
        let mut c = Cursor::new(&payload);
        let count = c.prefix()? as usize;
        if count != self.n_chunks {
            return Err(format!(
                "chunk_ids count mismatch: expected {}, got {count}",
                self.n_chunks
            ));
        }
        (0..count).map(|_| c.lp_str()).collect()
    }

    fn decode_spans(&self) -> Result<Vec<ChunkMeta>> {
        let payload = self.decoded(SECTION_CHUNKS_ORIGINAL_SPANS)?;
        let mut c = Cursor::new(&payload);
        let count = c.prefix()? as usize;
        let mut out = Vec::with_capacity(count.min(1 << 20));
        for _ in 0..count {
            out.push(ChunkMeta {
                source_uri: c.lp_str()?,
                offset_start: c.u64()?,
                offset_end: c.u64()?,
            });
        }
        Ok(out)
    }

    /// Canonical texts, decoded on demand (they can be large).
    fn decode_texts(&self) -> Result<Vec<String>> {
        let payload = self.decoded(SECTION_CHUNKS_CANONICAL)?;
        let mut c = Cursor::new(&payload);
        let count = c.prefix()? as usize;
        let mut out = Vec::with_capacity(count.min(1 << 20));
        for _ in 0..count {
            out.push(c.lp_str()?);
        }
        Ok(out)
    }

    fn decode_embeddings(&self) -> Result<Vec<f32>> {
        let s = self
            .section(SECTION_EMBEDDINGS)
            .ok_or("embeddings section missing")?;
        let (n, dim) = (self.n_embeddings, self.dim);
        match (self.manifest.dtype.as_str(), s.encoding) {
            ("float32", ENCODING_RAW) => {
                let phys = self.payload(SECTION_EMBEDDINGS)?;
                if phys.len() < n * dim * 4 {
                    return Err("embeddings section smaller than n*dim*4".to_string());
                }
                Ok(phys[..n * dim * 4]
                    .chunks_exact(4)
                    .map(|b| f32::from_le_bytes(b.try_into().unwrap()))
                    .collect())
            }
            ("float16", ENCODING_FLOAT16) => {
                let phys = self.payload(SECTION_EMBEDDINGS)?;
                if phys.len() < n * dim * 2 {
                    return Err("embeddings section smaller than n*dim*2".to_string());
                }
                Ok(phys[..n * dim * 2]
                    .chunks_exact(2)
                    .map(|b| f16_to_f32(u16::from_le_bytes(b.try_into().unwrap())))
                    .collect())
            }
            (dtype, _) => Err(format!(
                "embeddings dtype {dtype} is not supported by the web reader — \
                 open this file in the desktop app"
            )),
        }
    }

    /// CSR graph port (intpack offsets + delta-gapped neighbors + edge
    /// types). Gated like the runtime: manifest capability + section.
    fn decode_graph(&self) -> Result<Option<GraphData>> {
        let capable = self
            .manifest
            .capabilities_ext
            .as_ref()
            .and_then(|e| e.graph_present)
            .unwrap_or(false);
        if !capable || self.section(SECTION_GRAPH_ADJACENCY).is_none() {
            return Ok(None);
        }
        if self.section(SECTION_GRAPH_ADJACENCY).unwrap().encoding != ENCODING_RAW {
            return Err("graph_adjacency: web reader supports raw encoding only".to_string());
        }
        let payload = self.payload(SECTION_GRAPH_ADJACENCY)?;
        let mut c = Cursor::new(payload);
        let version = c.u32()?;
        if version != 1 {
            return Err(format!(
                "graph_adjacency: unsupported payload version {version}"
            ));
        }
        let n_nodes = c.u64()? as usize;
        if n_nodes != self.n_embeddings {
            return Err(format!(
                "graph node count {n_nodes} != n_embeddings {}",
                self.n_embeddings
            ));
        }
        let read_intpack = |c: &mut Cursor| -> Result<Vec<u64>> {
            let len = c.u32()? as usize;
            unpack_u64s(c.take(len)?)
        };
        let offsets = read_intpack(&mut c)?;
        let dst_gaps = read_intpack(&mut c)?;
        if offsets.len() != n_nodes + 1 {
            return Err("graph_adjacency: offsets length mismatch".to_string());
        }
        let total_edges = dst_gaps.len();
        // Edge-type column: iso scalar or an intpack run.
        let edge_types: Vec<u8> = match c.u8()? {
            0 => vec![c.u8()?; total_edges],
            1 => read_intpack(&mut c)?
                .iter()
                .map(|&v| u8::try_from(v).map_err(|_| "graph_adjacency: edge-type > 255".into()))
                .collect::<Result<Vec<u8>>>()?,
            other => {
                return Err(format!("graph_adjacency: unknown edge-type kind {other}"));
            }
        };

        // Prefix-sum the delta gaps within each (src, edge_type) run.
        let mut neighbors = Vec::with_capacity(total_edges);
        for node in 0..n_nodes {
            let (start, end) = (offsets[node] as usize, offsets[node + 1] as usize);
            if end > total_edges || start > end {
                return Err("graph_adjacency: offsets out of range".to_string());
            }
            let mut prev_dst: Option<u64> = None;
            let mut prev_type: Option<u8> = None;
            for (i, &et) in edge_types[start..end].iter().enumerate() {
                let idx = start + i;
                if prev_type != Some(et) {
                    prev_dst = None;
                    prev_type = Some(et);
                }
                let dst = match prev_dst {
                    Some(p) => p + dst_gaps[idx],
                    None => dst_gaps[idx],
                };
                if dst >= n_nodes as u64 {
                    return Err("graph_adjacency: neighbor id out of range".to_string());
                }
                neighbors.push(dst as u32);
                prev_dst = Some(dst);
            }
        }
        Ok(Some(GraphData {
            n_nodes,
            offsets: offsets.iter().map(|&o| o as u32).collect(),
            neighbors,
            kinds: edge_types,
        }))
    }

    /// content_hash over the six canonical sections' decoded bytes
    /// (domain-separated by name, spec order).
    fn compute_content_hash(&self) -> Result<String> {
        use sha2::Digest;
        let mut h = sha2::Sha256::new();
        for (id, name) in CANONICAL_SECTIONS {
            let bytes = self.decoded(id)?;
            h.update((name.len() as u32).to_le_bytes());
            h.update(name.as_bytes());
            h.update((bytes.len() as u64).to_le_bytes());
            h.update(&bytes);
        }
        Ok(format!("sha256:{}", hex(&h.finalize())))
    }

    // -- view models ----------------------------------------------------------

    /// The inspect document as a view model (same shape the native
    /// backend parses out of `inspect_json()`).
    pub fn inspect_view(&self) -> InspectView {
        InspectView {
            magic: "NEST".to_string(),
            version_major: self.version_major as u32,
            version_minor: 0,
            format_version: self.manifest.format_version,
            schema_version: self.manifest.schema_version,
            embedding_dim: self.dim as u32,
            n_chunks: self.n_chunks as u64,
            n_embeddings: self.n_embeddings as u64,
            file_size: self.bytes.len() as u64,
            manifest: self.manifest.clone(),
            sections: self.sections.clone(),
            blobs: serde_json::Value::Null,
            file_hash: self.file_hash.clone(),
            content_hash: self.content_hash.clone(),
            simd_backend: "wasm (scalar)".to_string(),
        }
    }

    /// Snapshot delivered on `Open`.
    pub fn opened_view(&self) -> OpenedDbView {
        OpenedDbView {
            path: PathBuf::from(&self.name),
            inspect: self.inspect_view(),
            chunk_ids: self.chunk_ids.clone(),
            has_ann: self.section(SECTION_HNSW_INDEX).is_some(),
            has_bm25: self.section(SECTION_BM25_INDEX).is_some(),
            has_graph: self.graph.is_some(),
            has_spaces: self.has_spaces,
            // Space names need the space_table codec; web v1 lists none.
            space_names: Vec::new(),
            graph_nodes: self.graph.as_ref().map(|g| g.n_nodes),
        }
    }

    /// Canonical texts + spans for the Chunks screen.
    pub fn chunks_data(&self) -> Result<ChunksData> {
        Ok(ChunksData {
            texts: self.decode_texts()?,
            metas: self.spans.clone(),
        })
    }

    /// The CSR graph (decoded at open).
    pub fn graph_data(&self) -> Option<GraphData> {
        self.graph.clone()
    }

    /// Exact flat cosine search — the recall=1.0 ground truth, at parity
    /// with the runtime's exact path by construction.
    pub fn search_exact(&self, query: &[f32], k: i32) -> Result<SearchResultsView> {
        let t0 = Instant::now();
        if k <= 0 {
            return Err(format!("invalid k: {k}"));
        }
        if query.len() != self.dim {
            return Err(format!(
                "dimension mismatch: expected {}, got {}",
                self.dim,
                query.len()
            ));
        }
        if query.iter().any(|v| v.is_nan() || v.is_infinite()) {
            return Err("NaN or Inf in query".to_string());
        }
        let norm = query.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm == 0.0 {
            return Err("zero-norm query".to_string());
        }
        let qnorm: Vec<f32> = query.iter().map(|x| x / norm).collect();

        let (n, dim) = (self.n_embeddings, self.dim);
        let mut scores: Vec<(usize, f32)> = Vec::with_capacity(n);
        for i in 0..n {
            let row = &self.embeddings[i * dim..(i + 1) * dim];
            scores.push((i, row.iter().zip(&qnorm).map(|(a, b)| a * b).sum()));
        }
        scores.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.cmp(&b.0))
        });
        let k_usize = k as usize;
        let hits = scores[..k_usize.min(n)]
            .iter()
            .map(|&(i, score)| {
                let span = &self.spans[i];
                SearchHitView {
                    chunk_id: self.chunk_ids[i].clone(),
                    score,
                    source_uri: span.source_uri.clone(),
                    offset_start: span.offset_start,
                    offset_end: span.offset_end,
                    citation_id: format!("nest://{}/{}", self.content_hash, self.chunk_ids[i]),
                    reranked: false,
                }
            })
            .collect::<Vec<_>>();
        Ok(SearchResultsView {
            k_returned: hits.len(),
            hits,
            query_time_ms: t0.elapsed().as_secs_f64() * 1000.0,
            index_type: "exact".to_string(),
            recall: 1.0,
            truncated: k_usize < n,
            k_requested: k,
            route: "exact".to_string(),
            exact_candidates: n,
            ann_candidates: 0,
            bm25_candidates: 0,
            graph_candidates: 0,
            fusion_mode: "none".to_string(),
            rerank_disclosure: if self.manifest.dtype == "float32" {
                "real cosine"
            } else {
                "real cosine at stored precision"
            }
            .to_string(),
            recall_estimate: 1.0,
        })
    }

    /// Exact-only latency benchmark (same deterministic queries as the
    /// desktop; ANN needs the runtime, so no recall comparison on web).
    /// Runs inline — see `model::web` for the threading note.
    pub fn benchmark(
        &self,
        n_queries: usize,
        k: i32,
        progress: &dyn Fn(usize),
    ) -> Result<BenchmarkView> {
        let queries = gen_queries(self.dim, n_queries);
        let mut times = Vec::with_capacity(n_queries);
        for (i, q) in queries.iter().enumerate() {
            let t0 = Instant::now();
            self.search_exact(q, k)?;
            times.push(t0.elapsed().as_secs_f64() * 1000.0);
            if i % 8 == 0 {
                progress(i);
            }
        }
        progress(n_queries);
        Ok(BenchmarkView {
            n_queries,
            k,
            dim: self.dim,
            dtype: self.manifest.dtype.clone(),
            simd_backend: "wasm (scalar)".to_string(),
            exact: latency_stats(&times),
            ann: None,
            recall_at_k: None,
        })
    }
}

/// Section names for the inspect table (the id still renders as `0xNN`
/// for sections the web reader does not name).
fn section_name(id: u32) -> &'static str {
    match id {
        SECTION_CHUNK_IDS => "chunk_ids",
        SECTION_CHUNKS_CANONICAL => "chunks_canonical",
        SECTION_CHUNKS_ORIGINAL_SPANS => "chunks_original_spans",
        SECTION_EMBEDDINGS => "embeddings",
        SECTION_PROVENANCE => "provenance",
        SECTION_SEARCH_CONTRACT => "search_contract",
        SECTION_HNSW_INDEX => "hnsw_index",
        SECTION_BM25_INDEX => "bm25_index",
        0x09 => "embeddings_fp",
        0x0A => "dictionary",
        0x0B => "dedup_map",
        SECTION_GRAPH_ADJACENCY => "graph_adjacency",
        SECTION_SPACE_TABLE => "space_table",
        _ => "unknown",
    }
}
