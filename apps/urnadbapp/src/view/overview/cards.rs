//! The overview's data: one card per part of the snapshot (identity,
//! manifest, spaces, media) with its rows and row buttons, and the
//! capability chips. Built once per opened db; the screen lays them out.

use crate::backend::workers::messages::OpenedDbView;

use crate::view::fmt_bytes;

/// One card's rows: (key, value, row button).
pub(super) struct Card {
    pub(super) title: &'static str,
    pub(super) rows: Vec<(String, String, Option<RowButton>)>,
}

/// A retained per-row button: copy a hash, or export a blob.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub(super) enum RowButton {
    CopyFile,
    CopyContent,
    CopyModel,
    CopySpace(usize),
    CopyBlob(usize),
    ExportBlob(usize),
}

/// A capability chip: present = Constructive, absent = Neutral outline
/// (engine `Chip`; static — capabilities are not clickable).
pub(super) fn chip_for(label: &str, present: bool) -> engine::widgets::prelude::Chip {
    engine::widgets::prelude::Chip::new(label).intent(if present {
        engine::styling::theme::Intent::Constructive
    } else {
        engine::styling::theme::Intent::Neutral
    })
}

pub(super) fn chips(db: &OpenedDbView) -> Vec<(&'static str, bool)> {
    let caps = &db.inspect.manifest.capabilities;
    let ext = db.inspect.manifest.capabilities_ext.as_ref();
    vec![
        ("exact", caps.supports_exact),
        ("ann", db.has_ann),
        ("bm25", db.has_bm25),
        ("citations", caps.supports_citations),
        ("reproducible", caps.supports_reproducible_build),
        ("graph", db.has_graph),
        ("blobs", ext.and_then(|e| e.blobs_present).unwrap_or(false)),
        ("inlined media", db.has_blob_data),
        ("spaces", db.has_spaces),
    ]
}

/// Card contents derived from the snapshot. Value column strings are
/// truncated at render time against the real column width. Cards with
/// nothing to show (no spaces, no blobs) are omitted.
pub(super) fn cards(db: &OpenedDbView) -> Vec<Card> {
    let m = &db.inspect.manifest;
    type Row = (String, String, Option<RowButton>);
    let row = |k: &str, v: String, b: Option<RowButton>| -> Row { (k.to_string(), v, b) };

    let mut manifest_rows = vec![
        row("embedding_model", m.embedding_model.clone(), None),
        row("embedding_dim", m.embedding_dim.to_string(), None),
        row("dtype", m.dtype.clone(), None),
        row("metric", m.metric.clone(), None),
        row("index_type", m.index_type.clone(), None),
        row("n_chunks", m.n_chunks.to_string(), None),
        row("chunker_version", m.chunker_version.clone(), None),
        row(
            "model_hash",
            m.model_hash.clone(),
            Some(RowButton::CopyModel),
        ),
    ];
    if let Some(title) = &m.title {
        manifest_rows.push(row("title", title.clone(), None));
    }
    if let Some(version) = &m.version {
        manifest_rows.push(row("version", version.clone(), None));
    }
    if let Some(created) = &m.created {
        manifest_rows.push(row("created", created.clone(), None));
    }
    if let Some(producer) = &m.producer {
        manifest_rows.push(row("producer", producer.clone(), None));
    }
    if let Some(full_dim) = m.full_dim {
        manifest_rows.push(row("full_dim", full_dim.to_string(), None));
    }

    let mut out = vec![
        Card {
            title: "IDENTITY",
            rows: vec![
                row("path", db.path.display().to_string(), None),
                row("magic", db.inspect.magic.clone(), None),
                row("file size", fmt_bytes(db.inspect.file_size), None),
                row(
                    "file_hash",
                    db.inspect.file_hash.clone(),
                    Some(RowButton::CopyFile),
                ),
                row(
                    "content_hash",
                    db.inspect.content_hash.clone(),
                    Some(RowButton::CopyContent),
                ),
                row("simd backend", db.inspect.simd_backend.clone(), None),
            ],
        },
        Card {
            title: "MANIFEST",
            rows: manifest_rows,
        },
    ];

    // One row per multimodal space: what `search-space` can score.
    if !db.inspect.spaces.is_empty() {
        let rows = db
            .inspect
            .spaces
            .iter()
            .enumerate()
            .map(|(i, s)| {
                row(
                    &s.name,
                    format!(
                        "{} vectors × {} {} · band {} · {}",
                        s.n_vectors,
                        s.dim,
                        s.dtype,
                        fmt_bytes(s.band_bytes),
                        s.model_hash
                    ),
                    Some(RowButton::CopySpace(i)),
                )
            })
            .collect();
        out.push(Card {
            title: "SPACES",
            rows,
        });
    }

    // One row per media blob; inlined ones export straight off the file.
    if !db.inspect.blobs.is_empty() {
        let rows = db
            .inspect
            .blobs
            .iter()
            .enumerate()
            .map(|(i, b)| {
                let button = if b.inlined && db.has_blob_data {
                    RowButton::ExportBlob(i)
                } else {
                    RowButton::CopyBlob(i)
                };
                row(
                    b.original_uri.trim_start_matches("media://"),
                    format!(
                        "{} · {} · {}",
                        fmt_bytes(b.byte_len),
                        if b.inlined { "inlined" } else { "sidecar" },
                        b.content_hash
                    ),
                    Some(button),
                )
            })
            .collect();
        out.push(Card {
            title: "MEDIA",
            rows,
        });
    }
    out
}
