//! Data update — port of webapp/data_update.py plus mtgdb.py's index builder.
//!
//! Downloads Scryfall's bulk exports, rebuilds the local card index, and refreshes the EDHREC
//! cache for the commanders already in the user's decks. Runs on a worker thread; progress is
//! polled by the frontend through the same {state, task, percent} shape the Python job used.
//!
//! The index is built into a temp file and swapped in with a rename at the end, so a failed or
//! interrupted update can never leave a half-written database where the app expects a working
//! one — readers keep using the old file until the moment it's replaced.

use flate2::read::GzDecoder;
use rusqlite::{params, Connection};
use serde_json::Value;
use std::io::{BufRead, BufReader, Read, Write};
use std::sync::Mutex;

use crate::db::fold_text;
use crate::paths;

// Shares of the 0-100 bar. Downloading dominates (~400MB), indexing is CPU-bound over the same
// data, and the EDHREC refresh is a handful of small requests.
const W_DOWNLOAD: f64 = 70.0;
const W_BUILD: f64 = 20.0;
const W_SYNERGY: f64 = 10.0;

const USER_AGENT: &str = "SpellbookMTG/1.0 (https://github.com/ofelipenavarro/spellbook-mtg)";

pub struct Status {
    pub state: &'static str, // idle | running | done | error
    pub task: Option<String>,
    pub percent: f64,
    pub error: Option<String>,
    pub result: Option<Value>,
}

pub static STATUS: Mutex<Status> = Mutex::new(Status {
    state: "idle",
    task: None,
    percent: 0.0,
    error: None,
    result: None,
});

fn set_progress(task: &str, percent: f64) {
    let mut s = STATUS.lock().unwrap();
    s.task = Some(task.to_string());
    s.percent = percent.clamp(0.0, 100.0);
}


/// Kicks off the update on a worker thread. Returns false if one is already in flight.
pub fn start() -> bool {
    {
        let mut s = STATUS.lock().unwrap();
        if s.state == "running" {
            return false;
        }
        s.state = "running";
        s.task = None;
        s.percent = 0.0;
        s.error = None;
        s.result = None;
    }
    std::thread::spawn(|| match run() {
        Ok(result) => {
            let mut s = STATUS.lock().unwrap();
            s.state = "done";
            s.task = Some("Concluído.".into());
            s.percent = 100.0;
            s.result = Some(result);
        }
        Err(e) => {
            let mut s = STATUS.lock().unwrap();
            s.state = "error";
            s.error = Some(e);
        }
    });
    true
}

/// One entry of Scryfall's /bulk-data listing.
struct BulkFile {
    uri: String,
    size: u64,
}

fn bulk_listing() -> Result<(BulkFile, BulkFile), String> {
    let body = ureq::get("https://api.scryfall.com/bulk-data")
        .set("User-Agent", USER_AGENT)
        .set("Accept", "application/json")
        .call()
        .map_err(|e| format!("Não consegui falar com o Scryfall: {e}"))?
        .into_string()
        .map_err(|e| e.to_string())?;
    let v: Value = serde_json::from_str(&body).map_err(|e| e.to_string())?;
    let items = v.get("data").and_then(|d| d.as_array()).ok_or("resposta inesperada do Scryfall")?;

    let pick = |kind: &str| -> Option<BulkFile> {
        let it = items.iter().find(|x| x.get("type").and_then(|t| t.as_str()) == Some(kind))?;
        Some(BulkFile {
            uri: it
                .get("download_uri")
                .or_else(|| it.get("jsonl_download_uri"))?
                .as_str()?
                .to_string(),
            size: it.get("compressed_size").and_then(|s| s.as_u64()).unwrap_or(0),
        })
    };
    let oracle = pick("oracle_cards").ok_or("bulk oracle_cards não encontrado")?;
    let all = pick("all_cards").ok_or("bulk all_cards não encontrado")?;
    Ok((oracle, all))
}

/// Streams one bulk file to disk, reporting bytes read so the bar moves during the long download.
fn download(file: &BulkFile, dest: &std::path::Path, on_bytes: &mut dyn FnMut(u64)) -> Result<(), String> {
    let resp = ureq::get(&file.uri)
        .set("User-Agent", USER_AGENT)
        .call()
        .map_err(|e| format!("Falha no download: {e}"))?;
    let tmp = dest.with_extension("part");
    if let Some(dir) = tmp.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let mut out = std::fs::File::create(&tmp).map_err(|e| e.to_string())?;
    let mut reader = resp.into_reader();
    let mut buf = vec![0u8; 1 << 20];
    loop {
        let n = reader.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        out.write_all(&buf[..n]).map_err(|e| e.to_string())?;
        on_bytes(n as u64);
    }
    out.flush().map_err(|e| e.to_string())?;
    drop(out);
    std::fs::rename(&tmp, dest).map_err(|e| e.to_string())?;
    Ok(())
}

/// The card-index schema, kept byte-compatible with mtgdb.py so either can read the other's build.
const INDEX_SCHEMA: &str = r#"
CREATE TABLE cards (
    oracle_id TEXT PRIMARY KEY,
    name TEXT, mana_cost TEXT, cmc REAL, type_line TEXT,
    oracle_text TEXT, power TEXT, toughness TEXT, loyalty TEXT,
    colors TEXT, color_identity TEXT, rarity TEXT, set_code TEXT,
    keywords TEXT, commander_legal TEXT, price_usd TEXT,
    reserved INTEGER, edhrec_rank INTEGER, uri TEXT, image_uri TEXT,
    game_changer INTEGER, name_folded TEXT
);
CREATE TABLE names_pt (
    printed_name TEXT, oracle_id TEXT, set_code TEXT, printed_name_folded TEXT
);
CREATE INDEX idx_name ON cards(name COLLATE NOCASE);
CREATE INDEX idx_type ON cards(type_line);
CREATE INDEX idx_pt ON names_pt(printed_name COLLATE NOCASE);
CREATE INDEX idx_pt_oracle ON names_pt(oracle_id);
CREATE INDEX idx_name_folded ON cards(name_folded);
CREATE INDEX idx_pt_folded ON names_pt(printed_name_folded);
"#;

fn s(v: &Value, k: &str) -> Option<String> {
    v.get(k).and_then(|x| x.as_str()).map(str::to_string)
}

/// Falls back to joining the card's faces for double-faced cards, matching mtgdb.py's face().
fn face(v: &Value, k: &str) -> Option<String> {
    if let Some(direct) = s(v, k) {
        return Some(direct);
    }
    let faces = v.get("card_faces")?.as_array()?;
    let parts: Vec<String> = faces.iter().filter_map(|f| s(f, k)).collect();
    (!parts.is_empty()).then(|| parts.join(" // "))
}

fn image_uri(v: &Value) -> Option<String> {
    let from = |o: &Value| {
        s(o, "normal").or_else(|| s(o, "large")).or_else(|| s(o, "small"))
    };
    if let Some(iu) = v.get("image_uris") {
        if let Some(u) = from(iu) {
            return Some(u);
        }
    }
    let faces = v.get("card_faces")?.as_array()?;
    from(faces.first()?.get("image_uris")?)
}

fn build_index(oracle: &std::path::Path, all: &std::path::Path) -> Result<(i64, i64), String> {
    let final_path = paths::cards_db();
    let tmp = final_path.with_extension("sqlite.tmp");
    let _ = std::fs::remove_file(&tmp);
    let con = Connection::open(&tmp).map_err(|e| e.to_string())?;
    con.execute_batch(INDEX_SCHEMA).map_err(|e| e.to_string())?;
    con.execute_batch("PRAGMA journal_mode=OFF; PRAGMA synchronous=OFF;").ok();

    // ---- cards, from oracle_cards ----
    let mut n_cards = 0i64;
    {
        let tx = con.unchecked_transaction().map_err(|e| e.to_string())?;
        let mut stmt = tx
            .prepare(
                "INSERT OR REPLACE INTO cards VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,
                                                      ?14,?15,?16,?17,?18,?19,?20,?21,?22)",
            )
            .map_err(|e| e.to_string())?;
        let f = std::fs::File::open(oracle).map_err(|e| e.to_string())?;
        for line in BufReader::new(GzDecoder::new(f)).lines() {
            let line = line.map_err(|e| e.to_string())?;
            let line = line.trim().trim_end_matches(',');
            if line.is_empty() || line == "[" || line == "]" {
                continue;
            }
            let Ok(c) = serde_json::from_str::<Value>(line) else { continue };
            if c.get("object").and_then(|o| o.as_str()) != Some("card") {
                continue;
            }
            let name = s(&c, "name").unwrap_or_default();
            stmt.execute(params![
                s(&c, "oracle_id"),
                name,
                face(&c, "mana_cost"),
                c.get("cmc").and_then(|x| x.as_f64()),
                s(&c, "type_line"),
                face(&c, "oracle_text"),
                face(&c, "power"),
                face(&c, "toughness"),
                s(&c, "loyalty"),
                c.get("colors").and_then(|x| x.as_array())
                    .map(|a| a.iter().filter_map(|v| v.as_str()).collect::<String>())
                    .unwrap_or_default(),
                c.get("color_identity").and_then(|x| x.as_array())
                    .map(|a| a.iter().filter_map(|v| v.as_str()).collect::<String>())
                    .unwrap_or_default(),
                s(&c, "rarity"),
                s(&c, "set"),
                c.get("keywords").and_then(|x| x.as_array())
                    .map(|a| a.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(","))
                    .unwrap_or_default(),
                c.get("legalities").and_then(|l| l.get("commander")).and_then(|x| x.as_str()),
                c.get("prices").and_then(|p| p.get("usd")).and_then(|x| x.as_str()),
                c.get("reserved").and_then(|x| x.as_bool()).unwrap_or(false) as i64,
                c.get("edhrec_rank").and_then(|x| x.as_i64()),
                s(&c, "scryfall_uri"),
                image_uri(&c),
                c.get("game_changer").and_then(|x| x.as_bool()).unwrap_or(false) as i64,
                fold_text(&name),
            ])
            .map_err(|e| e.to_string())?;
            n_cards += 1;
        }
        drop(stmt);
        tx.commit().map_err(|e| e.to_string())?;
    }

    // ---- Portuguese printed names, from all_cards ----
    let mut n_pt = 0i64;
    if all.exists() {
        let tx = con.unchecked_transaction().map_err(|e| e.to_string())?;
        let mut stmt = tx
            .prepare("INSERT INTO names_pt VALUES (?1,?2,?3,?4)")
            .map_err(|e| e.to_string())?;
        let mut seen = std::collections::HashSet::new();
        let f = std::fs::File::open(all).map_err(|e| e.to_string())?;
        for line in BufReader::new(GzDecoder::new(f)).lines() {
            let line = line.map_err(|e| e.to_string())?;
            let line = line.trim().trim_end_matches(',');
            if line.is_empty() || line == "[" || line == "]" {
                continue;
            }
            let Ok(c) = serde_json::from_str::<Value>(line) else { continue };
            if c.get("lang").and_then(|l| l.as_str()) != Some("pt") {
                continue;
            }
            let pn = s(&c, "printed_name").or_else(|| {
                let faces = c.get("card_faces")?.as_array()?;
                let parts: Vec<String> =
                    faces.iter().filter_map(|f| s(f, "printed_name")).collect();
                (!parts.is_empty()).then(|| parts.join(" // "))
            });
            let (Some(pn), Some(oid)) = (pn, s(&c, "oracle_id")) else { continue };
            if !seen.insert((pn.to_lowercase(), oid.clone())) {
                continue;
            }
            stmt.execute(params![pn, oid, s(&c, "set"), fold_text(&pn)])
                .map_err(|e| e.to_string())?;
            n_pt += 1;
        }
        drop(stmt);
        tx.commit().map_err(|e| e.to_string())?;
    }

    con.execute_batch("VACUUM").ok();
    drop(con);
    // Atomic swap: readers keep the old inode until this instant, so an interrupted build never
    // leaves a truncated index in place.
    std::fs::rename(&tmp, &final_path).map_err(|e| e.to_string())?;
    Ok((n_cards, n_pt))
}

fn run() -> Result<Value, String> {
    let data_dir = paths::data_dir();
    std::fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;

    set_progress("Consultando o Scryfall…", 0.0);
    let (oracle, all) = bulk_listing()?;
    let total = (oracle.size + all.size).max(1);

    let oracle_path = data_dir.join("oracle-cards.jsonl.gz");
    let all_path = data_dir.join("all-cards.jsonl.gz");

    let mut done: u64 = 0;
    let mut tick = |n: u64| {
        done += n;
        let pct = W_DOWNLOAD * (done as f64 / total as f64).min(1.0);
        set_progress("Baixando base de cartas do Scryfall…", pct);
    };
    download(&oracle, &oracle_path, &mut tick)?;
    download(&all, &all_path, &mut tick)?;

    set_progress("Reconstruindo o índice local…", W_DOWNLOAD);
    let (n_cards, n_pt) = build_index(&oracle_path, &all_path)?;
    set_progress("Reconstruindo o índice local…", W_DOWNLOAD + W_BUILD);

    // Refresh synergy for the commanders actually in use — never a crawl of EDHREC.
    let mut refreshed = Vec::new();
    if let Ok(con) = crate::db::open_app_db() {
        let commanders: Vec<String> = (|| -> rusqlite::Result<Vec<String>> {
            let mut stmt = con.prepare(
                "SELECT DISTINCT commander_name FROM decks
                 UNION SELECT DISTINCT commander_name_2 FROM decks WHERE commander_name_2 IS NOT NULL",
            )?;
            let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
            rows.collect()
        })()
        .unwrap_or_default();

        let n = commanders.len().max(1);
        for (i, name) in commanders.iter().enumerate() {
            set_progress(
                &format!("Atualizando sinergia: {name}…"),
                W_DOWNLOAD + W_BUILD + W_SYNERGY * (i as f64 / n as f64),
            );
            // A commander with no EDHREC page must not fail the whole update.
            if crate::edhrec::fetch(name, true).is_ok() {
                refreshed.push(name.clone());
            }
        }
    }

    Ok(serde_json::json!({
        "cards": n_cards,
        "pt_names": n_pt,
        "synergy_updated": refreshed,
    }))
}
