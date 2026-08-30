//! Local card-art cache.
//!
//! The card index stores remote `https://cards.scryfall.io/...` URLs. Serving those directly
//! means the grids are blank without a connection, which defeats the point of an offline app.
//!
//! A card carries both halves of the reference (see `types::ImageRef`): the path it would
//! occupy in this cache, and the upstream URL. The UI reads the file when it is there and shows
//! its placeholder when it is not, so a half-finished cache degrades one tile at a time rather
//! than failing a screen.
//!
//! Files mirror Scryfall's own layout (`normal/front/a/4/<id>.jpg`), which is already sharded
//! two levels deep — 38k files spread over 256 directories per variant instead of one flat
//! directory that every tool on the machine would then struggle to list.

use std::path::{Path, PathBuf};

use crate::paths;

pub const HOST: &str = "https://cards.scryfall.io/";

/// The variants the app actually renders: `normal` in the card modal, `art_crop` on deck tiles
/// and in the grids. Downloading others would multiply the cache for images nothing displays.
pub const VARIANTS: [&str; 2] = ["normal", "art_crop"];

pub fn cache_dir() -> PathBuf {
    paths::data_dir().join("images")
}

/// `https://cards.scryfall.io/normal/front/a/4/x.jpg?123` -> `normal/front/a/4/x.jpg`
///
/// The query string is Scryfall's own cache-buster. It is dropped: a card's art is immutable
/// per printing, and keeping it would mean the same bytes cached under many names.
pub fn relative_path(url: &str) -> Option<String> {
    let rest = url.strip_prefix(HOST)?;
    let rest = rest.split('?').next()?;
    // Refuse anything that could climb out of the cache directory.
    if rest.is_empty() || rest.contains("..") || rest.starts_with('/') {
        return None;
    }
    Some(rest.to_string())
}

pub fn cached_file(rel: &str) -> PathBuf {
    cache_dir().join(rel)
}

/// Reads one cached image and decodes it to RGBA, downscaled so its longest
/// side is at most `max_edge` device pixels.
///
/// Called from the worker thread, never from the UI thread: this is a disk
/// read plus a JPEG decode, and a grid wants a hundred of them at once.
///
/// The downscale is not an optimisation, it is a correctness requirement. The
/// engine packs every image into one atlas that stops growing at 8192px, and
/// Scryfall art_crop is 626x457 — roughly 220 of those fill the atlas, after
/// which nothing else can be drawn. A tile rendered 300px wide has no use for
/// the other 326 columns.
///
/// Returns `None` for a file that is absent or will not decode. Both are
/// ordinary: the art cache fills over a long download, and the caller shows a
/// placeholder either way.
pub fn load_scaled(rel: &str, max_edge: u32) -> Option<(u32, u32, Vec<u8>)> {
    let bytes = std::fs::read(cached_file(rel)).ok()?;
    let decoded = image::load_from_memory(&bytes)
        .inspect_err(|e| log::warn!("card art {rel} would not decode: {e}"))
        .ok()?;
    let (w, h) = (decoded.width(), decoded.height());
    let scaled = if w.max(h) > max_edge {
        // Triangle filter: cheap enough to run a hundred times without a
        // visible pause, and clean enough that card art does not alias.
        decoded.resize(max_edge, max_edge, image::imageops::FilterType::Triangle)
    } else {
        decoded
    };
    let rgba = scaled.to_rgba8();
    Some((rgba.width(), rgba.height(), rgba.into_raw()))
}

/// Swaps the variant segment of a Scryfall image path: `normal/front/..` -> `art_crop/front/..`.
pub fn with_variant(rel: &str, variant: &str) -> String {
    match rel.split_once('/') {
        Some((_, tail)) => format!("{variant}/{tail}"),
        None => rel.to_string(),
    }
}

/// Bytes on disk, for reporting how much the cache is using.
pub fn cache_size_bytes() -> u64 {
    fn walk(dir: &Path) -> u64 {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return 0;
        };
        entries
            .flatten()
            .map(|e| match e.file_type() {
                Ok(t) if t.is_dir() => walk(&e.path()),
                Ok(_) => e.metadata().map(|m| m.len()).unwrap_or(0),
                Err(_) => 0,
            })
            .sum()
    }
    walk(&cache_dir())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_host_and_cache_buster() {
        assert_eq!(
            relative_path("https://cards.scryfall.io/normal/front/a/4/x.jpg?1783907750").as_deref(),
            Some("normal/front/a/4/x.jpg")
        );
    }

    #[test]
    fn refuses_paths_that_escape_the_cache() {
        assert_eq!(
            relative_path("https://cards.scryfall.io/../../etc/passwd"),
            None
        );
    }

    #[test]
    fn refuses_foreign_urls() {
        assert_eq!(relative_path("https://example.com/a.jpg"), None);
    }

    #[test]
    fn swaps_the_variant_segment() {
        assert_eq!(
            with_variant("normal/front/a/4/x.jpg", "art_crop"),
            "art_crop/front/a/4/x.jpg"
        );
    }
}
