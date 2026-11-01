//! Pseudo-localization (ADR 0086, variable content): a test mode in which
//! every text is measured and drawn as a translation about 1.4 times as
//! long would be, still legible. Each word of Latin letters takes its
//! accented letters and grows by four tenths of its length, so a layout
//! that holds English only shows where a longer language would break it:
//! a label that does not wrap, a slot of fixed width, a row that does not
//! grow.
//!
//! The mode is the thread's, as the text system is. One text passes
//! through it where it is measured (`TextMeasurer::measure_styled`) and
//! where it is drawn (`TextNodeKey`), so the two agree; a text the mode
//! already made is left as it is, so measuring a drawn text again does
//! not grow it twice. Byte offsets into the original (a caret, a
//! selection) do not follow: the mode is for frames, not for typing.

use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};

/// How much longer a word grows.
pub const GROWTH: f32 = 0.4;

thread_local! {
    static ON: Cell<bool> = const { Cell::new(false) };
    /// Each text the mode made, by its source, and the set of what it
    /// made (to leave those alone).
    static MADE: RefCell<(HashMap<String, String>, HashSet<String>)> =
        RefCell::new((HashMap::new(), HashSet::new()));
}

/// Turns the mode on or off for this thread.
pub fn set_pseudo_locale(on: bool) {
    ON.with(|c| c.set(on));
}

pub fn pseudo_locale() -> bool {
    ON.with(Cell::get)
}

/// `text` as the mode shows it: itself when the mode is off, when it has
/// no Latin word, or when the mode made it.
pub fn localized(text: &str) -> Cow<'_, str> {
    if !pseudo_locale() || !text.bytes().any(|b| b.is_ascii_alphabetic()) {
        return Cow::Borrowed(text);
    }
    MADE.with(|m| {
        let mut m = m.borrow_mut();
        if m.1.contains(text) {
            return Cow::Borrowed(text);
        }
        if let Some(out) = m.0.get(text) {
            return Cow::Owned(out.clone());
        }
        let out = expand(text);
        m.1.insert(out.clone());
        m.0.insert(text.to_string(), out.clone());
        Cow::Owned(out)
    })
}

/// Each run of ASCII letters accented and grown by [`GROWTH`] of its
/// length (its own letters again, from the start); everything else as it
/// is.
pub fn expand(text: &str) -> String {
    let mut out = String::with_capacity(text.len() * 2);
    let mut word: Vec<char> = Vec::new();
    let flush = |word: &mut Vec<char>, out: &mut String| {
        if word.is_empty() {
            return;
        }
        let extra = (word.len() as f32 * GROWTH).round() as usize;
        for &c in word.iter() {
            out.push(accent(c));
        }
        for i in 0..extra {
            out.push(accent(word[i % word.len()].to_ascii_lowercase()));
        }
        word.clear();
    };
    for c in text.chars() {
        if c.is_ascii_alphabetic() {
            word.push(c);
        } else {
            flush(&mut word, &mut out);
            out.push(c);
        }
    }
    flush(&mut word, &mut out);
    out
}

/// A Latin-1 accented form of an ASCII letter, which every embedded face
/// draws; the letter itself where Latin-1 has none.
fn accent(c: char) -> char {
    match c {
        'a' => 'à',
        'e' => 'é',
        'i' => 'î',
        'o' => 'ö',
        'u' => 'ü',
        'y' => 'ý',
        'n' => 'ñ',
        'c' => 'ç',
        'A' => 'Å',
        'E' => 'É',
        'I' => 'Î',
        'O' => 'Ö',
        'U' => 'Ü',
        'Y' => 'Ý',
        'N' => 'Ñ',
        'C' => 'Ç',
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn words_grow_by_four_tenths_and_stay_words() {
        assert_eq!(expand("Save"), "Sàvésà");
        assert_eq!(expand("Open file"), "Öpéñöp fîléfî");
        assert_eq!(expand("3 of 12"), "3 öfö 12");
        assert_eq!(expand("日本"), "日本");
    }

    #[test]
    fn the_mode_is_off_by_default_and_never_grows_its_own_text() {
        assert_eq!(localized("Save"), "Save");
        set_pseudo_locale(true);
        let once = localized("Save").into_owned();
        assert_eq!(once, expand("Save"));
        assert_eq!(localized(&once), once, "made by the mode: left alone");
        assert_eq!(localized("12:30"), "12:30");
        set_pseudo_locale(false);
    }
}
