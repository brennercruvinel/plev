//! the item of a repeat inside an id: the key segment `[{item}]`, and the
//! pattern a template id makes, which every instance id fits.

use crate::ir::ITEM;

/// the byte ranges of the key segments of a canonical id, brackets
/// included. inside a key, `\` escapes the next character.
fn keys(id: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut open = None;
    let mut escaped = false;
    for (at, c) in id.char_indices() {
        match open {
            None if c == '[' => open = Some(at),
            None => {}
            Some(_) if escaped => escaped = false,
            Some(_) if c == '\\' => escaped = true,
            Some(start) if c == ']' => {
                out.push((start, at + 1));
                open = None;
            }
            Some(_) => {}
        }
    }
    out
}

/// true for the key segment `[{item}]`.
fn is_slot(id: &str, (start, end): (usize, usize)) -> bool {
    id.get(start + 1..end - 1) == Some(ITEM)
}

/// how many key segments of `id` are the item.
pub(crate) fn slots(id: &str) -> usize {
    keys(id).into_iter().filter(|&k| is_slot(id, k)).count()
}

/// the ids a template id stands for: whatever key replaces its one
/// `[{item}]`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Pattern {
    prefix: String,
    suffix: String,
}

impl Pattern {
    /// the pattern of `id`, which must carry the item exactly once.
    pub(crate) fn of(id: &str) -> Option<Pattern> {
        let mut found = keys(id).into_iter().filter(|&k| is_slot(id, k));
        let (start, end) = found.next()?;
        if found.next().is_some() {
            return None;
        }
        Some(Pattern {
            prefix: id.get(..start)?.to_owned(),
            suffix: id.get(end..)?.to_owned(),
        })
    }

    /// true when `id` is an instance of this template for some item: the
    /// same text with one key segment in the item's place.
    pub(crate) fn matches(&self, id: &str) -> bool {
        let Some(middle) = id
            .strip_prefix(self.prefix.as_str())
            .and_then(|rest| rest.strip_suffix(self.suffix.as_str()))
        else {
            return false;
        };
        keys(middle) == [(0, middle.len())]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_template_id_carries_the_item_once() {
        assert_eq!(slots("changes.file[{item}]"), 1);
        assert_eq!(slots("review.file[{item}].stage"), 1);
        assert_eq!(slots("a[{item}].b[{item}]"), 2);
        assert_eq!(slots("changes.file[src/lib.rs]"), 0);
        assert_eq!(slots("a[x[{item}]"), 0, "a `[` inside a key opens nothing");
        assert_eq!(slots("a[\\]{item}]"), 0);
        assert!(Pattern::of("a.b").is_none());
        assert!(Pattern::of("a[{item}].b[{item}]").is_none());
    }

    #[test]
    fn an_instance_fits_its_template_and_nothing_else_does() {
        let file = Pattern::of("changes.file[{item}]").unwrap();
        assert!(file.matches("changes.file[src/lib.rs]"));
        assert!(file.matches("changes.file[a\\]b]"));
        assert!(file.matches("changes.file[]"));
        assert!(!file.matches("changes.file"));
        assert!(!file.matches("changes.filex[a]"));
        assert!(!file.matches("changes.file[a].stage"));
        assert!(!file.matches("changes.file[a][b]"));
        assert!(!file.matches("changes.file[a]b"));
        let stage = Pattern::of("review.file[{item}].stage").unwrap();
        assert!(stage.matches("review.file[notes.md].stage"));
        assert!(!stage.matches("review.file[notes.md].unstage"));
        assert!(!stage.matches("review.file[a].b[c].stage"));
    }
}
