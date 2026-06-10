/// Compute Levenshtein edit distance between two strings.
pub(crate) fn levenshtein(a: &str, b: &str) -> usize {
    let a_len = a.len();
    let b_len = b.len();
    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    let mut prev_row: Vec<usize> = (0..=b_len).collect();
    let mut curr_row = vec![0; b_len + 1];

    for (i, a_char) in a.chars().enumerate() {
        curr_row[0] = i + 1;
        for (j, b_char) in b.chars().enumerate() {
            let cost = if a_char == b_char { 0 } else { 1 };
            curr_row[j + 1] = (prev_row[j + 1] + 1)        // deletion
                .min(curr_row[j] + 1)                       // insertion
                .min(prev_row[j] + cost); // substitution
        }
        std::mem::swap(&mut prev_row, &mut curr_row);
    }

    prev_row[b_len]
}

/// Find the closest match from `candidates` for the given `input`.
///
/// Returns `Some(candidate)` if the best match has edit distance <= `max_dist`.
/// For short inputs (len <= 3), requires distance <= 1.
/// For longer inputs, requires distance <= 2.
pub fn suggest_similar<'a>(input: &str, candidates: &[&'a str]) -> Option<&'a str> {
    let max_dist = if input.len() <= 3 { 1 } else { 2 };

    let mut best: Option<(&str, usize)> = None;
    for &candidate in candidates {
        let dist = levenshtein(input, candidate);
        if dist <= max_dist && (best.is_none() || dist < best.unwrap().1) {
            best = Some((candidate, dist));
        }
    }
    best.map(|(s, _)| s)
}
