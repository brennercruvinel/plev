//! Shared data shapes.
//!
//! The HTTP version of this app answered `serde_json::Value` everywhere and
//! let the JavaScript read fields by name. Nothing checked that a field the
//! UI read was a field the query selected, and the two drifted more than
//! once. These are the same payloads, typed, so the compiler enforces the
//! contract instead of a runtime `undefined`.
//!
//! Per-domain shapes (decks, collection entries, games, ...) live beside
//! their operations in `ops/`; this module holds only what more than one
//! domain needs.

use serde::{Deserialize, Serialize};

/// A card's artwork, as two halves the UI needs separately.
///
/// The index stores remote Scryfall URLs; the cache mirrors Scryfall's own
/// directory layout under `data/images/`. The old HTTP layer collapsed both
/// into a `/img/<rel>` URL that a route resolved. There is no route any
/// more: the UI loads `cache_dir()/rel` off disk and falls back to asking
/// the worker thread to fetch `remote`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageRef {
    /// Path relative to the image cache, e.g. `normal/front/a/4/x.jpg`.
    /// `None` for a URL that is not on Scryfall's image host, which is
    /// never cached and can only be fetched live.
    pub rel: Option<String>,
    /// The original upstream URL.
    pub remote: String,
}

impl ImageRef {
    /// Builds a reference from an index URL, classifying it as cacheable or
    /// not. Returns `None` for an absent or empty URL.
    pub fn from_url(url: Option<&str>) -> Option<Self> {
        let url = url?.trim();
        if url.is_empty() {
            return None;
        }
        Some(Self {
            rel: crate::images::relative_path(url),
            remote: url.to_string(),
        })
    }

    /// Absolute path this image would occupy in the cache, if cacheable.
    pub fn cached_path(&self) -> Option<std::path::PathBuf> {
        self.rel.as_deref().map(crate::images::cached_file)
    }

    /// The same art at another Scryfall variant (`art_crop`, `normal`).
    /// Deck tiles use the crop, the card modal uses the full image.
    pub fn variant(&self, variant: &str) -> Option<std::path::PathBuf> {
        let rel = self.rel.as_deref()?;
        Some(crate::images::cached_file(&crate::images::with_variant(
            rel, variant,
        )))
    }
}

/// One card as the index knows it - the `CARD_COLS` projection, typed.
///
/// Every field is optional exactly where the column is nullable, because the
/// index is rebuilt from Scryfall bulk data and a partial row is normal (a
/// token has no mana cost, an un-priced printing has no `price_usd`).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Card {
    pub oracle_id: Option<String>,
    pub name: String,
    pub mana_cost: Option<String>,
    pub cmc: Option<f64>,
    pub type_line: Option<String>,
    pub oracle_text: Option<String>,
    pub power: Option<String>,
    pub toughness: Option<String>,
    pub loyalty: Option<String>,
    /// Scryfall's compact colour string, e.g. `WU`.
    pub colors: Option<String>,
    pub color_identity: Option<String>,
    pub rarity: Option<String>,
    pub set_code: Option<String>,
    pub keywords: Option<String>,
    pub commander_legal: Option<String>,
    pub price_usd: Option<String>,
    pub reserved: Option<i64>,
    pub edhrec_rank: Option<i64>,
    pub uri: Option<String>,
    pub layout: Option<String>,
    pub game_changer: Option<i64>,
    pub image: Option<ImageRef>,
    pub image_back: Option<ImageRef>,
}

impl Card {
    /// Reads a row selected with `ops::cards::CARD_COLS`.
    pub fn from_row(r: &rusqlite::Row) -> rusqlite::Result<Self> {
        let get_s = |k: &str| r.get::<_, Option<String>>(k).unwrap_or(None);
        let get_i = |k: &str| r.get::<_, Option<i64>>(k).unwrap_or(None);
        Ok(Self {
            oracle_id: get_s("oracle_id"),
            name: get_s("name").unwrap_or_default(),
            mana_cost: get_s("mana_cost"),
            cmc: r.get::<_, Option<f64>>("cmc").unwrap_or(None),
            type_line: get_s("type_line"),
            oracle_text: get_s("oracle_text"),
            power: get_s("power"),
            toughness: get_s("toughness"),
            loyalty: get_s("loyalty"),
            colors: get_s("colors"),
            color_identity: get_s("color_identity"),
            rarity: get_s("rarity"),
            set_code: get_s("set_code"),
            keywords: get_s("keywords"),
            commander_legal: get_s("commander_legal"),
            price_usd: get_s("price_usd"),
            reserved: get_i("reserved"),
            edhrec_rank: get_i("edhrec_rank"),
            uri: get_s("uri"),
            layout: get_s("layout"),
            game_changer: get_i("game_changer"),
            image: ImageRef::from_url(get_s("image_uri").as_deref()),
            image_back: ImageRef::from_url(get_s("image_uri_back").as_deref()),
        })
    }

    /// Colour identity as individual WUBRG letters, for the colour pips.
    pub fn identity(&self) -> Vec<char> {
        self.color_identity
            .as_deref()
            .unwrap_or("")
            .chars()
            .filter(|c| "WUBRG".contains(*c))
            .collect()
    }

    /// `true` when the card has a second face worth flipping to. Only the
    /// layouts that actually print two faces - a `split` card is one image.
    pub fn is_double_faced(&self) -> bool {
        self.image_back.is_some()
            && matches!(
                self.layout.as_deref(),
                Some("transform" | "modal_dfc" | "double_faced_token" | "reversible_card")
            )
    }
}

/// One row of the activity log shown on the dashboard.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Activity {
    pub kind: String,
    pub description: String,
    /// SQLite `datetime('now')` string, UTC.
    pub ts: String,
}
