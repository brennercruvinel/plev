//! What a text input admits: which characters, and how many at most. A
//! one-time code takes digits and as many as it has cells; a plain field
//! takes anything. Typing, a paste and an input method's commit go
//! through [`Accept::admit`]: what is not admitted is dropped, and what
//! does not fit is cut, never an error. A value set whole (an agent's
//! `set_value`) is admitted only whole ([`Accept::takes`]).

/// The characters an input takes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Chars {
    #[default]
    Any,
    /// `0`-`9`; the full-width digits an input method of East Asia types
    /// (`０`-`９`) are read as them.
    Digits,
    /// The ASCII letters and digits (a recovery code), the full-width
    /// digits read as digits.
    Alphanumeric,
}

/// What an input admits.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Accept {
    pub chars: Chars,
    /// The most characters the text holds.
    pub max: Option<usize>,
}

/// A full-width digit as its ASCII digit; any other character as it is.
fn narrow(c: char) -> char {
    match c {
        '０'..='９' => char::from(b'0' + (c as u32 - '０' as u32) as u8),
        c => c,
    }
}

impl Accept {
    /// Anything, without a limit.
    pub const ANY: Accept = Accept {
        chars: Chars::Any,
        max: None,
    };

    /// `n` digits, a one-time code.
    pub fn digits(n: usize) -> Self {
        Self {
            chars: Chars::Digits,
            max: Some(n),
        }
    }

    /// `c` as the input takes it, or `None` when it does not.
    pub fn admits(&self, c: char) -> Option<char> {
        let c = match self.chars {
            Chars::Any => return Some(c),
            _ => narrow(c),
        };
        let ok = match self.chars {
            Chars::Digits => c.is_ascii_digit(),
            _ => c.is_ascii_alphanumeric(),
        };
        ok.then_some(c)
    }

    /// What of `s` goes into a text of `len` characters with `replaced`
    /// of them (a selection) going: the characters admitted, as many as
    /// there is room for.
    pub fn admit(&self, s: &str, len: usize, replaced: usize) -> String {
        let room = self.max.map_or(usize::MAX, |max| {
            max.saturating_sub(len - replaced.min(len))
        });
        s.chars()
            .filter_map(|c| self.admits(c))
            .take(room)
            .collect()
    }

    /// Whether `text` is admitted whole, as it is.
    pub fn takes(&self, text: &str) -> bool {
        self.max.is_none_or(|max| text.chars().count() <= max)
            && text.chars().all(|c| self.admits(c) == Some(c))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_code_takes_digits_up_to_its_cells_and_drops_the_rest() {
        let a = Accept::digits(6);
        assert_eq!(a.admit("12a3", 0, 0), "123");
        assert_eq!(a.admit("98765", 4, 0), "98", "two cells left");
        assert_eq!(a.admit("7", 6, 0), "", "full");
        assert_eq!(a.admit("7", 6, 1), "7", "over a selected cell");
        assert_eq!(a.admit("１２３", 0, 0), "123", "full-width digits");
        assert_eq!(a.admit("٣", 0, 0), "", "another script's digit is not one");
        assert!(a.takes("012345"));
        assert!(!a.takes("0123456"), "too long");
        assert!(!a.takes("12a"), "a letter");
        assert!(!a.takes("１"), "set whole, as it is");
        assert!(a.takes(""));
    }

    #[test]
    fn anything_goes_into_a_plain_field() {
        assert_eq!(Accept::ANY.admit("a b\u{5d0}", 99, 0), "a b\u{5d0}");
        assert!(Accept::ANY.takes("anything at all"));
        let code = Accept {
            chars: Chars::Alphanumeric,
            max: Some(8),
        };
        assert_eq!(code.admit("ab-12 CD", 0, 0), "ab12CD");
    }
}
