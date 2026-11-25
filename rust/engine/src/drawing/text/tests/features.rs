//! A run's OpenType features (tabular figures) reach the shaper's
//! attributes, the measure cache and the shaping key, so a run measured
//! with them is drawn with them and never shares a cache entry with the
//! same run without them.

use cosmic_text::fontdb::Database;
use cosmic_text::{Feature, FeatureTag};

use super::super::backend::{TextFeatures, TextStyle};
use super::super::cluster_cache::ClusterKey;
use super::super::fonts::register_embedded_fonts;
use super::super::measure_context::{MeasureKey, attrs_for};
use crate::drawing::compositor::TextNodeKey;

fn styles() -> (TextStyle, TextStyle) {
    let plain = TextStyle::new(12.0).with_family("Inter");
    let tabular = plain.clone().with_features(TextFeatures::TABULAR);
    (plain, tabular)
}

#[test]
fn tabular_figures_reach_the_shaper_s_attributes() {
    let mut db = Database::new();
    register_embedded_fonts(&mut db);
    let (plain, tabular) = styles();
    let tnum = Feature {
        tag: FeatureTag::new(b"tnum"),
        value: 1,
    };
    assert_eq!(attrs_for(&tabular, &db).font_features.features, [tnum]);
    assert!(attrs_for(&plain, &db).font_features.features.is_empty());
}

#[test]
fn the_features_key_the_caches_and_the_scene() {
    let (plain, tabular) = styles();
    // the keys have no Debug: compared, not printed
    assert!(MeasureKey::new("42", &plain, None) != MeasureKey::new("42", &tabular, None));
    assert!(ClusterKey::new("42", &plain, None) != ClusterKey::new("42", &tabular, None));
    let key = TextNodeKey::from_style("42", &tabular, None);
    assert_eq!(key.features, TextFeatures::TABULAR);
    assert_ne!(key, TextNodeKey::from_style("42", &plain, None));
}
