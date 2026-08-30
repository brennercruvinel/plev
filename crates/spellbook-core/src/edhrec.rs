//! EDHREC synergy cache — port of edhrec.py.
//!
//! EDHREC publishes no bulk export, so unlike Scryfall this is fetched one commander at a time,
//! on demand, and kept on disk forever after. Reads never touch the network: if a commander
//! isn't cached the UI says so and offers to fetch, rather than silently stalling on a request.
//! Fetching is deliberately one page at a time — never a crawl of the site.

use serde_json::Value;
use std::path::PathBuf;

use crate::paths;

const BASE: &str = "https://json.edhrec.com/pages";
const USER_AGENT: &str =
    "SpellbookMTG/1.0 (https://github.com/ofelipenavarro/spellbook-mtg; on-demand personal lookup)";

/// Commander name -> EDHREC's URL slug. Same rules as edhrec.py's slugify(): strip accents,
/// lowercase, collapse anything non-alphanumeric into single hyphens.
pub fn slugify(name: &str) -> String {
    let folded = crate::db::fold_text(name); // already accent-stripped + lowercased
    let mut out = String::with_capacity(folded.len());
    let mut prev_dash = false;
    for c in folded.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c);
            prev_dash = false;
        } else if !prev_dash {
            out.push('-');
            prev_dash = true;
        }
    }
    out.trim_matches('-').to_string()
}

fn cache_path(kind: &str, slug: &str) -> PathBuf {
    paths::data_dir().join("edhrec").join(kind).join(format!("{slug}.json"))
}


fn read_cache(kind: &str, commander: &str) -> Option<Value> {
    let raw = std::fs::read_to_string(cache_path(kind, &slugify(commander))).ok()?;
    serde_json::from_str(&raw).ok()
}

/// Fetches and caches one commander. `with_combos` also pulls the combo page.
pub fn fetch(commander: &str, with_combos: bool) -> Result<(), String> {
    let slug = slugify(commander);
    for (kind, wanted) in [("commanders", true), ("combos", with_combos)] {
        if !wanted {
            continue;
        }
        let url = format!("{BASE}/{kind}/{slug}.json");
        let resp = ureq::get(&url)
            .set("User-Agent", USER_AGENT)
            .set("Accept", "application/json")
            .call();
        let body = match resp {
            Ok(r) => r.into_string().map_err(|e| e.to_string())?,
            Err(ureq::Error::Status(404, _)) if kind == "combos" => continue, // no combos is fine
            Err(ureq::Error::Status(404, _)) => {
                return Err(format!(
                    "O EDHREC não tem página para \"{commander}\" (slug: {slug})."
                ))
            }
            Err(e) => return Err(format!("Não consegui falar com o EDHREC: {e}")),
        };
        let path = cache_path(kind, &slug);
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        std::fs::write(&path, body).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Card recommendations for a commander, minus anything already in the deck — the shape the
/// synergy panel expects. Returns None when nothing is cached yet.
pub fn recommendations(commander: &str, already_in_deck: &[String]) -> Option<(Vec<Value>, Vec<Value>)> {
    let data = read_cache("commanders", commander)?;
    let have: Vec<String> = already_in_deck.iter().map(|n| n.to_lowercase()).collect();

    let mut recs = Vec::new();
    if let Some(lists) = data
        .get("container")
        .and_then(|c| c.get("json_dict"))
        .and_then(|j| j.get("cardlists"))
        .and_then(|c| c.as_array())
    {
        for list in lists {
            let header = list.get("header").and_then(|h| h.as_str()).unwrap_or("");
            // Same two lists the Python version surfaced: the commander-specific picks and the
            // generally-strong ones. Other lists (lands, mana rocks…) are noise here.
            if !(header.contains("High Synergy") || header.contains("Top Cards")) {
                continue;
            }
            for v in list.get("cardviews").and_then(|c| c.as_array()).into_iter().flatten() {
                let Some(name) = v.get("name").and_then(|n| n.as_str()) else { continue };
                if have.contains(&name.to_lowercase()) {
                    continue;
                }
                recs.push(serde_json::json!({
                    "name": name,
                    "synergy": v.get("synergy").cloned().unwrap_or(Value::Null),
                    "num_decks": v.get("num_decks").cloned().unwrap_or(Value::Null),
                    "already_owned": false,
                }));
            }
        }
    }
    recs.truncate(15);

    let similar = data
        .get("similar")
        .and_then(|s| s.as_array())
        .cloned()
        .unwrap_or_default();
    Some((recs, similar))
}

#[cfg(test)]
mod tests {
    use super::slugify;

    #[test]
    fn slugs_match_the_python_version() {
        assert_eq!(slugify("Syr Konrad, the Grim"), "syr-konrad-the-grim");
        assert_eq!(slugify("Krenko, Mob Boss"), "krenko-mob-boss");
        assert_eq!(slugify("Atraxa, Praetors' Voice"), "atraxa-praetors-voice");
        // accents are stripped, not hyphenated
        assert_eq!(slugify("Adéwalé, Breaker of Chains"), "adewale-breaker-of-chains");
    }
}
