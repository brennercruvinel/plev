//! the name grammar `[a-z0-9_]+`, shared by `SemId` name segments, state
//! keys and action names.

/// true for a character a name may hold.
pub(crate) fn is_name_char(c: char) -> bool {
    c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'
}

/// why `text` is not a name, or `None` when it is one.
pub(crate) fn name_fault(text: &str) -> Option<String> {
    if text.is_empty() {
        return Some("empty name".to_owned());
    }
    text.char_indices()
        .find(|&(_, c)| !is_name_char(c))
        .map(|(at, c)| format!("invalid character {c:?} at byte {at}"))
}
