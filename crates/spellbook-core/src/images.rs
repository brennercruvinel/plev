//! Local card-art cache.
//!
//! The card index stores remote `https://cards.scryfall.io/...` URLs. Serving those directly
//! means the grids are blank without a connection, which defeats the point of an offline app.
//!
//! Rather than rewriting URLs conditionally (which would need a filesystem check per card on
//! every response), the API always rewrites them to a local `/img/<path>` route. That route
//! serves the cached file when it exists and redirects to Scryfall when it doesn't — so a
//! half-finished cache, or a card downloaded after the last update, still shows art online and
//! degrades to a broken image only when both the cache and the network are missing.
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

/// The URL the frontend should request. Falls through unchanged for anything not on Scryfall's
/// image host, so a hand-entered or future URL still renders instead of 404ing.
pub fn local_url(url: &str) -> String {
    match relative_path(url) {
        Some(rel) => format!("/img/{rel}"),
        None => url.to_string(),
    }
}

/// Rewrites a card's image fields in place — both faces, since a two-sided card's back is
/// cached and served exactly like its front.
pub fn rewrite_card(card: &mut serde_json::Value) {
    for field in ["image_uri", "image_uri_back"] {
        let Some(u) = card.get(field).and_then(|v| v.as_str()) else { continue };
        let local = local_url(u);
        if let Some(m) = card.as_object_mut() {
            m.insert(field.into(), serde_json::Value::from(local));
        }
    }
}

pub fn cached_file(rel: &str) -> PathBuf {
    cache_dir().join(rel)
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
        let Ok(entries) = std::fs::read_dir(dir) else { return 0 };
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
        assert_eq!(relative_path("https://cards.scryfall.io/../../etc/passwd"), None);
    }

    #[test]
    fn leaves_foreign_urls_alone() {
        let u = "https://example.com/a.jpg";
        assert_eq!(local_url(u), u);
    }

    #[test]
    fn swaps_the_variant_segment() {
        assert_eq!(with_variant("normal/front/a/4/x.jpg", "art_crop"), "art_crop/front/a/4/x.jpg");
    }
}
