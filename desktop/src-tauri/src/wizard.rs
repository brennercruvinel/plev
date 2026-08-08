//! Deterministic Commander deckbuilder — port of webapp/deck_wizard.py, plus the ownership
//! modes the Python version lacked.
//!
//! No LLM: every card comes out of a query against the local Scryfall index, never generated.
//!
//! The ownership problem this adds: the original always picked from the whole legal card pool,
//! so it happily produced a 100-card list full of cards the user doesn't own, with nothing
//! marking which ones those were. Two modes fix that:
//!
//!   Suggest  — pick from the full pool (best deck), but record each card's ownership so the
//!              deck view can flag what still needs buying.
//!   Owned    — restrict the pool to cards already in the collection, so the list is buildable
//!              today. Cards currently allocated to another deck are still eligible (you own
//!              them) but get flagged, since using them means dismantling something else.
//!
//! Crucially, neither mode writes to the `collection` table: building a deck never implies
//! owning its cards. Only deck_cards rows are created.

use rusqlite::{params, Connection};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};

use crate::db::{fold_text, open_app_db, open_cards_db};
use crate::routes::cards::CARD_COLS;

// Midpoints of the guide's ranges; land count + these sum to 99, +1 commander = 100.
const T_LANDS: usize = 37;
const T_RAMP: usize = 11;
const T_DRAW: usize = 10;
const T_REMOVAL: usize = 6;
const T_WIPES: usize = 3;
const T_PROTECTION: usize = 4;
const T_NONLAND: usize = 99 - T_LANDS;

fn game_changer_budget(bracket: i64) -> usize {
    match bracket {
        1 | 2 => 0,
        3 => 3,
        _ => usize::MAX,
    }
}
fn allows_mld(bracket: i64) -> bool {
    bracket >= 4
}

fn basic_land_for(color: char) -> &'static str {
    match color {
        'W' => "Plains",
        'U' => "Island",
        'B' => "Swamp",
        'R' => "Mountain",
        'G' => "Forest",
        _ => "Wastes",
    }
}

/// Same keyword heuristics as the Python version — the local index has no community otags, so
/// category membership is approximated from oracle_text/type_line.
fn classify(type_line: &str, text_lower: &str) -> &'static str {
    if type_line.contains("Land") {
        return "land";
    }
    if text_lower.contains("add {")
        || text_lower.contains("add one mana")
        || text_lower.contains("add mana")
        || (text_lower.contains("search your library for a") && text_lower.contains("land"))
    {
        return "ramp";
    }
    if text_lower.contains("destroy all creatures")
        || text_lower.contains("exile all creatures")
        || text_lower.contains("damage to each creature")
    {
        return "board_wipes";
    }
    if text_lower.contains("destroy target")
        || text_lower.contains("exile target")
        || text_lower.contains("target player sacrifices")
    {
        return "removal_spot";
    }
    if text_lower.contains("hexproof")
        || text_lower.contains("indestructible")
        || text_lower.contains("protection from")
        || text_lower.contains("counter target spell")
    {
        return "protection";
    }
    if text_lower.contains("draw a card")
        || text_lower.contains("draw two cards")
        || text_lower.contains("draw three cards")
    {
        return "draw";
    }
    "other"
}

fn is_mass_land_denial(text_lower: &str) -> bool {
    text_lower.contains("destroy all lands")
        || (text_lower.contains("each player sacrifices") && text_lower.contains("land"))
}

#[derive(Clone)]
struct Candidate {
    name: String,
    oracle_id: String,
    mana_cost: String,
    category: &'static str,
    game_changer: bool,
    mld: bool,
    edhrec_rank: i64,
    /// "owned_free" | "owned_in_deck" | "missing"
    ownership: &'static str,
    owned_in_deck: Option<String>,
}

#[derive(Deserialize)]
pub struct AutoBuildIn {
    pub name: String,
    pub commander_name: String,
    #[serde(default = "three")]
    pub bracket: i64,
    #[serde(default)]
    pub philosophy: Option<String>,
    /// "suggest" (default, full pool) or "owned" (collection only).
    #[serde(default)]
    pub mode: Option<String>,
}
fn three() -> i64 { 3 }

pub struct BuildOutcome {
    pub deck_id: i64,
    pub meta: Value,
}

/// How many physical copies of one card the user owns, and where they are.
///
/// Every row in `collection` is real cardboard: a card sleeved in two decks is two rows, because
/// it is two cards. Collapsing that to a yes/no "do you own it" was what made a deck report its
/// own copies as borrowed from itself — the counts have to survive all the way to the UI.
#[derive(Default, Clone)]
pub struct Copies {
    pub free: i64,
    /// deck_id -> (deck name, copies allocated to it)
    pub by_deck: HashMap<i64, (String, i64)>,
}

impl Copies {
    /// Copies sitting in decks other than `deck_id`, as (deck name, copies).
    pub fn elsewhere(&self, deck_id: i64) -> Vec<(String, i64)> {
        let mut v: Vec<(String, i64)> = self
            .by_deck
            .iter()
            .filter(|(id, _)| **id != deck_id)
            .map(|(_, (n, q))| (n.clone(), *q))
            .collect();
        v.sort();
        v
    }
}

/// Folded card name -> copies owned. Folded so a name typed without accents still finds its card.
fn collection_index(con: &Connection) -> HashMap<String, Copies> {
    let mut map: HashMap<String, Copies> = HashMap::new();
    let Ok(mut stmt) = con.prepare(
        "SELECT collection.card_name, collection.allocated_deck_id, decks.name, collection.quantity
         FROM collection LEFT JOIN decks ON decks.id = collection.allocated_deck_id",
    ) else {
        return map;
    };
    let Ok(rows) = stmt.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, Option<i64>>(1)?,
            r.get::<_, Option<String>>(2)?,
            r.get::<_, i64>(3)?,
        ))
    }) else {
        return map;
    };
    for (name, deck_id, deck_name, quantity) in rows.flatten() {
        let e = map.entry(fold_text(&name)).or_default();
        match deck_id {
            None => e.free += quantity,
            Some(id) => {
                let slot = e.by_deck.entry(id).or_insert_with(|| (deck_name.unwrap_or_default(), 0));
                slot.1 += quantity;
            }
        }
    }
    map
}

pub fn build(p: &AutoBuildIn) -> Result<BuildOutcome, String> {
    let owned_only = p.mode.as_deref() == Some("owned");
    let cdb = open_cards_db()
        .ok_or("Base de cartas ainda não construída — atualize a base de dados primeiro.")?;
    let con = open_app_db().map_err(|e| e.to_string())?;

    // Resolve + validate the commander.
    let commander: (String, String, String) = cdb
        .query_row(
            &format!("SELECT {CARD_COLS} FROM cards WHERE name = ?1 COLLATE NOCASE"),
            [&p.commander_name],
            |r| {
                Ok((
                    r.get::<_, String>("name")?,
                    r.get::<_, Option<String>>("color_identity")?.unwrap_or_default(),
                    r.get::<_, Option<String>>("type_line")?.unwrap_or_default(),
                ))
            },
        )
        .map_err(|_| format!("Comandante não encontrado na base local: {}", p.commander_name))?;
    let (commander_name, commander_ci, ctype) = commander;
    if !ctype.contains("Legendary") || !(ctype.contains("Creature") || ctype.contains("Planeswalker"))
    {
        return Err(format!("'{commander_name}' não é uma carta lendária elegível para comandante."));
    }

    let owned = collection_index(&con);

    // Candidate pool: commander-legal, within colour identity, not the commander itself.
    let mut pool: Vec<Candidate> = Vec::new();
    {
        let mut stmt = cdb
            .prepare(&format!("SELECT {CARD_COLS} FROM cards WHERE commander_legal = 'legal'"))
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>("name")?,
                    r.get::<_, Option<String>>("oracle_id")?.unwrap_or_default(),
                    r.get::<_, Option<String>>("mana_cost")?.unwrap_or_default(),
                    r.get::<_, Option<String>>("color_identity")?.unwrap_or_default(),
                    r.get::<_, Option<String>>("type_line")?.unwrap_or_default(),
                    r.get::<_, Option<String>>("oracle_text")?.unwrap_or_default(),
                    r.get::<_, Option<i64>>("game_changer")?.unwrap_or(0),
                    r.get::<_, Option<i64>>("edhrec_rank")?.unwrap_or(i64::MAX),
                ))
            })
            .map_err(|e| e.to_string())?;

        for row in rows.flatten() {
            let (name, oracle_id, mana_cost, ci, type_line, text, gc, rank) = row;
            let _ = &ci; // used for the color-identity filter above, not stored on the candidate
            if name.eq_ignore_ascii_case(&commander_name) {
                continue;
            }
            if !ci.chars().all(|c| commander_ci.contains(c)) {
                continue;
            }
            let text_lower = text.to_lowercase();
            let category = classify(&type_line, &text_lower);
            if category == "land" {
                continue; // basics handled separately (v1 scope: no curated nonbasic lands)
            }

            // A deck being built doesn't exist yet, so every copy the user owns is fair game:
            // free copies first, then ones that would have to come out of another deck.
            let copies = owned.get(&fold_text(&name));
            let (ownership, owned_in_deck) = match copies {
                Some(c) if c.free > 0 => ("owned_free", None),
                Some(c) => match c.elsewhere(-1).first() {
                    Some((deck, _)) => ("owned_in_deck", Some(deck.clone())),
                    None => ("missing", None),
                },
                None => ("missing", None),
            };
            // In "owned" mode the pool is the collection — anything not owned is out entirely.
            if owned_only && ownership == "missing" {
                continue;
            }

            pool.push(Candidate {
                name,
                oracle_id,
                mana_cost,
                category,
                game_changer: gc == 1,
                mld: is_mass_land_denial(&text_lower),
                edhrec_rank: rank,
                ownership,
                owned_in_deck,
            });
        }
    }

    // Prefer cards you already have free, then ones you own but are in another deck, then by
    // EDHREC rank. In "suggest" mode this still surfaces strong unowned staples, but only after
    // the equivalent card you already own.
    let rank_of = |c: &Candidate| match c.ownership {
        "owned_free" => 0,
        "owned_in_deck" => 1,
        _ => 2,
    };
    pool.sort_by(|a, b| rank_of(a).cmp(&rank_of(b)).then(a.edhrec_rank.cmp(&b.edhrec_rank)));

    let mut gc_left = game_changer_budget(p.bracket);
    let mld_ok = allows_mld(p.bracket);
    let mut chosen: Vec<Candidate> = Vec::new();
    let mut taken: HashSet<String> = HashSet::new();

    let mut eligible = |c: &Candidate, gc_left: &usize| -> bool {
        if taken.contains(&fold_text(&c.name)) {
            return false;
        }
        if c.mld && !mld_ok {
            return false;
        }
        if c.game_changer && *gc_left == 0 {
            return false;
        }
        true
    };

    let take = |cat: &str, n: usize, chosen: &mut Vec<Candidate>, gc_left: &mut usize, taken: &mut HashSet<String>| {
        let mut got = 0;
        for c in pool.iter().filter(|c| c.category == cat) {
            if got >= n {
                break;
            }
            if taken.contains(&fold_text(&c.name)) || (c.mld && !mld_ok) {
                continue;
            }
            if c.game_changer && *gc_left == 0 {
                continue;
            }
            if c.game_changer {
                *gc_left -= 1;
            }
            taken.insert(fold_text(&c.name));
            chosen.push(c.clone());
            got += 1;
        }
    };
    let _ = &mut eligible; // skeleton fill uses the inlined checks in `take`

    take("ramp", T_RAMP, &mut chosen, &mut gc_left, &mut taken);
    take("draw", T_DRAW, &mut chosen, &mut gc_left, &mut taken);
    take("removal_spot", T_REMOVAL, &mut chosen, &mut gc_left, &mut taken);
    take("board_wipes", T_WIPES, &mut chosen, &mut gc_left, &mut taken);
    take("protection", T_PROTECTION, &mut chosen, &mut gc_left, &mut taken);

    // Fill the rest from anything left, best-ranked first.
    for c in pool.iter() {
        if chosen.len() >= T_NONLAND {
            break;
        }
        if taken.contains(&fold_text(&c.name)) || (c.mld && !mld_ok) {
            continue;
        }
        if c.game_changer && gc_left == 0 {
            continue;
        }
        if c.game_changer {
            gc_left -= 1;
        }
        taken.insert(fold_text(&c.name));
        chosen.push(c.clone());
    }

    // Manabase: basics split by the colored-pip weight of what was chosen.
    let lands = build_manabase(&commander_ci, &chosen, T_LANDS);

    // Persist. Only deck_cards — never collection: building a list doesn't mean owning the cards.
    con.execute(
        "INSERT INTO decks (name, commander_name, philosophy) VALUES (?1, ?2, ?3)",
        params![
            p.name,
            commander_name,
            p.philosophy.clone().unwrap_or_else(|| {
                let m = if owned_only { "apenas cartas da coleção" } else { "sugerindo cartas fora da coleção" };
                format!("Montado automaticamente (bracket {}, {m}) a partir da base local do Scryfall.", p.bracket)
            })
        ],
    )
    .map_err(|e| e.to_string())?;
    let deck_id = con.last_insert_rowid();

    con.execute(
        "INSERT INTO deck_cards (deck_id, card_name, quantity, is_commander) VALUES (?1, ?2, 1, 1)",
        params![deck_id, commander_name],
    )
    .map_err(|e| e.to_string())?;
    for c in &chosen {
        let _ = con.execute(
            "INSERT INTO deck_cards (deck_id, card_name, quantity, is_commander, oracle_id)
             VALUES (?1, ?2, 1, 0, ?3)",
            params![deck_id, c.name, c.oracle_id],
        );
    }
    for (land, qty) in &lands {
        if *qty > 0 {
            let _ = con.execute(
                "INSERT INTO deck_cards (deck_id, card_name, quantity, is_commander)
                 VALUES (?1, ?2, ?3, 0)",
                params![deck_id, land, qty],
            );
        }
    }

    let missing: Vec<&Candidate> = chosen.iter().filter(|c| c.ownership == "missing").collect();
    let borrowed: Vec<&Candidate> =
        chosen.iter().filter(|c| c.ownership == "owned_in_deck").collect();
    let mode_label = if owned_only { "owned" } else { "suggest" };

    let _ = con.execute(
        "INSERT INTO activity (type, description) VALUES ('deck_built', ?1)",
        params![format!(
            "Deck {} montado automaticamente (bracket {}, modo {}): {} carta(s) que você não tem, {} vinda(s) de outro deck",
            p.name, p.bracket, mode_label, missing.len(), borrowed.len()
        )],
    );

    let meta = json!({
        "commander": commander_name,
        "color_identity": commander_ci,
        "bracket": p.bracket,
        "mode": mode_label,
        "nonland_count": chosen.len(),
        "land_count": lands.iter().map(|(_, q)| q).sum::<i64>(),
        "missing_count": missing.len(),
        "borrowed_count": borrowed.len(),
        "missing": missing.iter().map(|c| json!(c.name)).collect::<Vec<_>>(),
        "borrowed": borrowed.iter()
            .map(|c| json!({ "name": c.name, "deck": c.owned_in_deck }))
            .collect::<Vec<_>>(),
    });
    Ok(BuildOutcome { deck_id, meta })
}

fn build_manabase(ci: &str, chosen: &[Candidate], land_count: usize) -> Vec<(String, i64)> {
    if ci.is_empty() {
        return vec![("Wastes".to_string(), land_count as i64)];
    }
    let colors: Vec<char> = ci.chars().collect();
    let mut pips: HashMap<char, usize> = colors.iter().map(|c| (*c, 0)).collect();
    for card in chosen {
        for c in &colors {
            pips.entry(*c).and_modify(|n| *n += card.mana_cost.matches(&format!("{{{c}}}")).count());
        }
    }
    let total: usize = pips.values().sum();
    let mut out: Vec<(String, i64)> = Vec::new();
    let mut assigned = 0i64;
    for c in &colors {
        let share = if total == 0 {
            (land_count / colors.len()) as i64
        } else {
            ((land_count * pips[c]) as f64 / total as f64).round() as i64
        };
        out.push((basic_land_for(*c).to_string(), share));
        assigned += share;
    }
    if let Some(first) = out.first_mut() {
        first.1 += land_count as i64 - assigned; // absorb the rounding remainder
    }
    out
}

/// Ownership status for every card in a deck, so the deck view can flag what isn't owned.
/// Computed on read (not stored) so it stays correct as the collection changes.
pub fn deck_ownership(con: &Connection, deck_id: i64) -> HashMap<String, Value> {
    let owned = collection_index(con);
    let mut out = HashMap::new();
    let Ok(mut stmt) = con.prepare(
        "SELECT card_name, quantity FROM deck_cards WHERE deck_id = ?1 AND is_commander = 0",
    ) else {
        return out;
    };
    let Ok(rows) = stmt.query_map([deck_id], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))
    else {
        return out;
    };
    for (name, needed) in rows.flatten() {
        let copies = owned.get(&fold_text(&name)).cloned().unwrap_or_default();
        let here = copies.by_deck.get(&deck_id).map(|(_, q)| *q).unwrap_or(0);
        let elsewhere = copies.elsewhere(deck_id);

        // The deck's own copies come first: a card this deck already holds is simply here, no
        // matter how many other decks also run one. Only the shortfall is worth flagging, and
        // only then does it matter whether a spare is free or sleeved in another deck.
        let short = needed - here;
        let v = if short <= 0 {
            json!({ "status": "owned_here", "copies": here })
        } else if copies.free > 0 {
            json!({ "status": "owned_free", "copies": copies.free, "short": short })
        } else if let Some((deck, _)) = elsewhere.first() {
            json!({
                "status": "owned_in_deck",
                "deck": deck,
                "decks": elsewhere.iter().map(|(n, q)| json!({ "deck": n, "copies": q })).collect::<Vec<_>>(),
                "short": short,
            })
        } else {
            json!({ "status": "missing", "short": short })
        };
        out.insert(name, v);
    }
    out
}
