//! Decklist parsing — port of webapp/decklist_import.py.
//!
//! Handles the shapes Moxfield / Archidekt / TappedOut / plain text export: "1 Card Name",
//! "1x Name", bare "Name", trailing set codes and collector numbers ("1 Sol Ring (C21) 289"),
//! foil markers, and section headers that must be skipped rather than read as card names.
//!
//! This module only turns text into candidate (quantity, name) pairs — it never touches a
//! database. Matching happens against the local index, and nothing is written until the user
//! confirms the preview: an unmatched name is surfaced, never guessed into existence.

/// Section labels like "Commander", "Creatures (23)", "Sideboard:" — skipped only when the line
/// doesn't start with a quantity, so a real card that happens to start with one of these words
/// ("Landfall Ritual") still reads normally.
const SECTION_WORDS: &[&str] = &[
    "commander",
    "commanders",
    "deck",
    "mainboard",
    "maybeboard",
    "sideboard",
    "companion",
    "creature",
    "creatures",
    "land",
    "lands",
    "instant",
    "instants",
    "sorcery",
    "sorceries",
    "artifact",
    "artifacts",
    "enchantment",
    "enchantments",
    "planeswalker",
    "planeswalkers",
    "battle",
    "battles",
    "other",
    "spell",
    "spells",
];

fn is_section_header(line: &str) -> bool {
    let mut s = line.trim().trim_end_matches(':').trim();
    // drop a trailing "(23)" count
    if s.ends_with(')') {
        if let Some(open) = s.rfind('(') {
            if s[open + 1..s.len() - 1].chars().all(|c| c.is_ascii_digit()) {
                s = s[..open].trim();
            }
        }
    }
    SECTION_WORDS.iter().any(|w| w.eq_ignore_ascii_case(s))
}

/// Removes trailing set/collector/foil annotations exporters add. Loops until stable, since the
/// annotations stack in either order — "Zulaport Cutthroat (C21) 12" needs the number stripped
/// and then the "(C21)" that's only at the end once the number is gone.
fn strip_set_info(name: &str) -> String {
    let mut cur = name.trim().to_string();
    loop {
        let before = cur.clone();

        if cur.ends_with(')') {
            if let Some(i) = cur.rfind('(') {
                cur = cur[..i].trim().to_string();
            }
        }
        if cur.ends_with(']') {
            if let Some(i) = cur.rfind('[') {
                cur = cur[..i].trim().to_string();
            }
        }
        // "*F*" / "*f*" foil marker
        let lower = cur.to_lowercase();
        if lower.ends_with("*f*") {
            cur = cur[..cur.len() - 3].trim().to_string();
        }
        // trailing collector number, optionally "#12" or "12a", possibly preceded by a set code
        let toks: Vec<&str> = cur.split_whitespace().collect();
        if toks.len() > 1 {
            let last = toks[toks.len() - 1].trim_start_matches('#');
            let is_collector = !last.is_empty()
                && last.chars().next().is_some_and(|c| c.is_ascii_digit())
                && last.chars().all(|c| c.is_ascii_alphanumeric());
            if is_collector {
                cur = toks[..toks.len() - 1].join(" ");
                // a bare set code left behind by the number, e.g. "... ZNR"
                let toks2: Vec<&str> = cur.split_whitespace().collect();
                if toks2.len() > 1 {
                    let l = toks2[toks2.len() - 1];
                    let looks_like_set = (2..=6).contains(&l.len())
                        && l.chars()
                            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit());
                    if looks_like_set {
                        cur = toks2[..toks2.len() - 1].join(" ");
                    }
                }
            }
        }

        cur = cur.trim().to_string();
        if cur == before {
            return cur;
        }
    }
}

/// Returns [(quantity, name), ...] from pasted plain-text decklist content.
pub fn parse_text(text: &str) -> Vec<(i64, String)> {
    let mut out = Vec::new();
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("//") {
            continue;
        }
        if is_section_header(line) {
            continue;
        }
        // optional leading quantity: "1", "1x", "1 x"
        let mut qty = 1i64;
        let mut rest = line;
        let digits: String = line.chars().take_while(|c| c.is_ascii_digit()).collect();
        if !digits.is_empty() {
            let after = &line[digits.len()..];
            let after = after
                .strip_prefix('x')
                .or_else(|| after.strip_prefix('X'))
                .unwrap_or(after);
            if after.starts_with(char::is_whitespace) || digits.len() < line.len() {
                if let Ok(n) = digits.parse::<i64>() {
                    let trimmed = after.trim_start();
                    if !trimmed.is_empty() {
                        qty = n;
                        rest = trimmed;
                    }
                }
            }
        }
        let name = strip_set_info(rest);
        if !name.is_empty() {
            out.push((qty, name));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::parse_text;

    #[test]
    fn parses_the_shapes_exporters_produce() {
        let text = "\
// a comment
Commander (1)
1 Syr Konrad, the Grim
1x Sol Ring (C21) 289
31 Swamp
Blood Artist
Creatures (2)
2 Zulaport Cutthroat [ZNR] *F*
1 Anduril, Flame of the West ZNR 189
";
        let got = parse_text(text);
        assert_eq!(
            got,
            vec![
                (1, "Syr Konrad, the Grim".to_string()),
                (1, "Sol Ring".to_string()),
                (31, "Swamp".to_string()),
                (1, "Blood Artist".to_string()),
                (2, "Zulaport Cutthroat".to_string()),
                (1, "Anduril, Flame of the West".to_string()),
            ]
        );
    }

    #[test]
    fn keeps_cards_whose_name_starts_like_a_section_word() {
        // "Deck" alone is a header; "1 Deckhand" is a card.
        let got = parse_text("Deck\n1 Deckhand\n");
        assert_eq!(got, vec![(1, "Deckhand".to_string())]);
    }
}
