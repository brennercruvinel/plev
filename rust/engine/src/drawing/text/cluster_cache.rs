//! The per-line shaping cache behind [`TextMeasurer::clusters_styled`]
//! (super::measure::TextMeasurer::clusters_styled). An editor asks for the
//! clusters of every line with a caret or a selection on screen, every
//! frame; they only change when the line's text does.
//!
//! The key is the text and the style, never a line index: an edit
//! invalidates exactly the lines whose text it changed, and a newline that
//! shifts every line below it keeps them all. Memory is a byte budget
//! (the key and the clusters of each entry), the least recently used entry
//! out first.

use std::mem::size_of;

use lru::LruCache;

use super::backend::{OverflowWrap, TextStyle};
use super::measure::ClusterBox;

/// Room for about 1700 lines of 70 characters, a few dozen screens.
pub(super) const CLUSTER_CACHE_BUDGET: usize = 4 * 1024 * 1024;

/// Every input the clusters depend on: the text, the style fields the
/// shaper reads and the exact wrap width.
#[derive(Clone, PartialEq, Eq, Hash)]
pub(super) struct ClusterKey {
    text: String,
    font_size_bits: u32,
    line_height_bits: u32,
    font_weight: u16,
    letter_spacing_bits: u32,
    font_family: Option<String>,
    overflow_wrap: OverflowWrap,
    max_width_bits: Option<u32>,
}

impl ClusterKey {
    pub(super) fn new(text: &str, style: &TextStyle, max_width: Option<f32>) -> Self {
        Self {
            text: text.to_string(),
            font_size_bits: style.font_size.to_bits(),
            line_height_bits: style.line_height.to_bits(),
            font_weight: style.font_weight,
            letter_spacing_bits: style.letter_spacing.to_bits(),
            font_family: style.font_family.clone(),
            overflow_wrap: style.overflow_wrap,
            max_width_bits: max_width.map(f32::to_bits),
        }
    }
}

/// What an entry costs against the budget.
fn entry_bytes(key: &ClusterKey, clusters: &[ClusterBox]) -> usize {
    size_of::<ClusterKey>()
        + key.text.len()
        + key.font_family.as_ref().map_or(0, String::len)
        + size_of::<Vec<ClusterBox>>()
        + std::mem::size_of_val(clusters)
}

/// Shaped clusters by line content, bounded by bytes.
pub(super) struct ClusterCache {
    entries: LruCache<ClusterKey, Vec<ClusterBox>>,
    bytes: usize,
    budget: usize,
}

impl ClusterCache {
    pub(super) fn new(budget: usize) -> Self {
        Self {
            entries: LruCache::unbounded(),
            bytes: 0,
            budget,
        }
    }

    /// The clusters of `key`, marked as just used.
    pub(super) fn get(&mut self, key: &ClusterKey) -> Option<Vec<ClusterBox>> {
        self.entries.get(key).cloned()
    }

    /// Keep `clusters` for `key`, then drop the least recently used
    /// entries until the cache fits its budget. An entry larger than the
    /// whole budget is not kept.
    pub(super) fn put(&mut self, key: ClusterKey, clusters: Vec<ClusterBox>) {
        let cost = entry_bytes(&key, &clusters);
        if cost > self.budget {
            return;
        }
        if let Some((old_key, old)) = self.entries.push(key, clusters) {
            self.bytes -= entry_bytes(&old_key, &old);
        }
        self.bytes += cost;
        while self.bytes > self.budget {
            let Some((k, v)) = self.entries.pop_lru() else {
                break;
            };
            self.bytes -= entry_bytes(&k, &v);
        }
    }

    /// Drops every entry: the faces they were shaped with changed.
    pub(super) fn clear(&mut self) {
        self.entries.clear();
        self.bytes = 0;
    }

    #[cfg(test)]
    pub(super) fn len(&self) -> usize {
        self.entries.len()
    }

    #[cfg(test)]
    pub(super) fn bytes(&self) -> usize {
        self.bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn style() -> TextStyle {
        TextStyle::new(14.0)
    }

    fn clusters(n: usize) -> Vec<ClusterBox> {
        (0..n)
            .map(|i| ClusterBox {
                start: i,
                end: i + 1,
                x: i as f32,
                w: 1.0,
                line_top: 0.0,
                level: 0,
                paragraph_rtl: false,
            })
            .collect()
    }

    fn cost(text: &str, n: usize) -> usize {
        entry_bytes(&ClusterKey::new(text, &style(), None), &clusters(n))
    }

    #[test]
    fn a_hit_returns_what_was_put_and_the_key_is_the_whole_style() {
        let mut cache = ClusterCache::new(CLUSTER_CACHE_BUDGET);
        cache.put(ClusterKey::new("line 1", &style(), None), clusters(6));
        let got = cache.get(&ClusterKey::new("line 1", &style(), None));
        assert_eq!(got, Some(clusters(6)));
        assert!(
            cache
                .get(&ClusterKey::new("line 2", &style(), None))
                .is_none()
        );
        let bold = style().with_weight(700);
        assert!(cache.get(&ClusterKey::new("line 1", &bold, None)).is_none());
        assert!(
            cache
                .get(&ClusterKey::new("line 1", &style(), Some(90.0)))
                .is_none()
        );
    }

    #[test]
    fn the_budget_drops_the_least_recently_used_line() {
        let one = cost("line 0", 6);
        let mut cache = ClusterCache::new(3 * one);
        for i in 0..3 {
            cache.put(
                ClusterKey::new(&format!("line {i}"), &style(), None),
                clusters(6),
            );
        }
        assert_eq!((cache.len(), cache.bytes()), (3, 3 * one));
        // line 0 is used again, so line 1 is the one to go.
        assert!(
            cache
                .get(&ClusterKey::new("line 0", &style(), None))
                .is_some()
        );
        cache.put(ClusterKey::new("line 3", &style(), None), clusters(6));
        assert_eq!((cache.len(), cache.bytes()), (3, 3 * one));
        assert!(
            cache
                .get(&ClusterKey::new("line 1", &style(), None))
                .is_none()
        );
        assert!(
            cache
                .get(&ClusterKey::new("line 0", &style(), None))
                .is_some()
        );
    }

    #[test]
    fn a_line_larger_than_the_budget_is_not_kept_and_a_replace_is_counted_once() {
        let mut cache = ClusterCache::new(cost("long", 10));
        cache.put(ClusterKey::new("long", &style(), None), clusters(11));
        assert_eq!((cache.len(), cache.bytes()), (0, 0));
        cache.put(ClusterKey::new("long", &style(), None), clusters(4));
        cache.put(ClusterKey::new("long", &style(), None), clusters(10));
        assert_eq!((cache.len(), cache.bytes()), (1, cost("long", 10)));
    }
}
