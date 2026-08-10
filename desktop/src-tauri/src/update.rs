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
use crate::images;
use crate::paths;

// Shares of the 0-100 bar. Downloading dominates (~400MB), indexing is CPU-bound over the same
// data, and the EDHREC refresh is a handful of small requests.
const W_DOWNLOAD: f64 = 12.0;
const W_BUILD: f64 = 5.0;
const W_SYNERGY: f64 = 3.0;
// Art dominates the wall-clock time by an order of magnitude — ~77k requests against ~2 files —
// so it gets most of the bar. Without this the progress sat at 100% for two hours.
const W_IMAGES: f64 = 80.0;

// Scryfall asks for 50-100ms between requests. That is a limit on the *aggregate* rate, so it is
// enforced globally across the workers below rather than per thread.
const IMAGE_DELAY: std::time::Duration = std::time::Duration::from_millis(100);

// Downloads are latency-bound, not bandwidth-bound: each image is ~90KB but a round trip costs
// ~180ms, so one sequential worker idles most of the time and finishes 77k files in five hours.
// A handful of workers sharing one rate gate keeps the aggregate at Scryfall's stated ceiling
// while actually using it.
const IMAGE_WORKERS: usize = 4;

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

/// Set by the cancel endpoint and checked between image fetches. A two-hour job the user can't
/// stop is a job they will kill the app to escape, and killing it mid-write is how caches rot.
pub static CANCEL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn cancel() {
    CANCEL.store(true, std::sync::atomic::Ordering::Relaxed);
}

fn cancelled() -> bool {
    CANCEL.load(std::sync::atomic::Ordering::Relaxed)
}

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
        CANCEL.store(false, std::sync::atomic::Ordering::Relaxed);
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
    game_changer INTEGER, name_folded TEXT,
    -- Scryfall's layout, and the second physical face when there is one. A "//" in the name is
    -- not enough to tell them apart: a split or adventure card prints both halves on one piece of
    -- cardboard, while a transform or modal DFC has a genuine back that has to be shown somehow.
    layout TEXT, image_uri_back TEXT
);
CREATE TABLE names_localized (
    printed_name TEXT, oracle_id TEXT, set_code TEXT, printed_name_folded TEXT,
    lang TEXT NOT NULL DEFAULT 'pt', lang_rank INTEGER NOT NULL DEFAULT 1
);
CREATE INDEX idx_name ON cards(name COLLATE NOCASE);
CREATE INDEX idx_type ON cards(type_line);
CREATE INDEX idx_loc ON names_localized(printed_name COLLATE NOCASE);
CREATE INDEX idx_loc_oracle ON names_localized(oracle_id);
CREATE INDEX idx_name_folded ON cards(name_folded);
CREATE INDEX idx_loc_folded ON names_localized(printed_name_folded);
CREATE INDEX idx_loc_lang ON names_localized(lang);
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

/// The second face's image, for the layouts that have one. Split, flip, adventure and aftermath
/// cards all carry two names but a single printed face, so they get None — offering to "flip"
/// them would promise a side that doesn't exist.
fn image_uri_back(v: &Value) -> Option<String> {
    const TWO_SIDED: [&str; 5] =
        ["transform", "modal_dfc", "double_faced_token", "reversible_card", "art_series"];
    let layout = s(v, "layout")?;
    if !TWO_SIDED.contains(&layout.as_str()) {
        return None;
    }
    let faces = v.get("card_faces")?.as_array()?;
    let back = faces.get(1)?.get("image_uris")?;
    s(back, "normal").or_else(|| s(back, "large")).or_else(|| s(back, "small"))
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

/// Languages the index carries, in the order results should be preferred when the same typed
/// name matches in more than one. English first because it is the name the rest of the app keys
/// on; the rest follow the priority the user asked for.
///
/// Returning None for anything else keeps the index from doubling in size for languages nobody
/// here searches in — Scryfall ships around twenty.
fn lang_rank(lang: &str) -> Option<i64> {
    const ORDER: [&str; 8] = ["en", "pt", "es", "fr", "it", "ja", "ko", "zhs"];
    ORDER.iter().position(|l| *l == lang).map(|i| i as i64)
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
                                                      ?14,?15,?16,?17,?18,?19,?20,?21,?22,?23,?24)",
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
                s(&c, "layout"),
                image_uri_back(&c),
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
            .prepare("INSERT INTO names_localized VALUES (?1,?2,?3,?4,?5,?6)")
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
            let Some(lang) = c.get("lang").and_then(|l| l.as_str()) else { continue };
            let Some(rank) = lang_rank(lang) else { continue };
            let pn = s(&c, "printed_name").or_else(|| {
                let faces = c.get("card_faces")?.as_array()?;
                let parts: Vec<String> =
                    faces.iter().filter_map(|f| s(f, "printed_name")).collect();
                (!parts.is_empty()).then(|| parts.join(" // "))
            });
            let (Some(pn), Some(oid)) = (pn, s(&c, "oracle_id")) else { continue };
            // One row per (name, card, language): the same printed name recurs across every set
            // a card was printed in, and storing each would multiply the table for no gain.
            if !seen.insert((pn.to_lowercase(), oid.clone(), lang.to_string())) {
                continue;
            }
            stmt.execute(params![pn, oid, s(&c, "set"), fold_text(&pn), lang, rank])
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

/// Downloads every card's art into the local cache, in both variants the app renders.
///
/// Resumable by construction: a file that already exists is skipped, so an update interrupted at
/// 60% picks up where it left off rather than re-fetching two gigabytes. Individual failures are
/// counted and skipped — one dead URL among 38k must not fail the whole update, and the /img
/// route falls back to the network for anything missing.
fn cache_images(on_progress: &mut dyn FnMut(usize, usize, u64)) -> Result<(u64, u64), String> {
    let cdb = crate::db::open_cards_db().ok_or("Índice de cartas indisponível")?;
    let urls: Vec<String> = (|| -> rusqlite::Result<Vec<String>> {
        // Both faces: a transform card whose back was never cached shows a broken image the
        // moment the user flips it offline, which is exactly the case the cache exists for.
        let mut stmt = cdb.prepare(
            "SELECT image_uri FROM cards WHERE image_uri IS NOT NULL
             UNION SELECT image_uri_back FROM cards WHERE image_uri_back IS NOT NULL",
        )?;
        let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
        rows.collect()
    })()
    .map_err(|e| e.to_string())?;

    // Every card, in every variant the UI can ask for.
    let mut targets: Vec<String> = Vec::with_capacity(urls.len() * images::VARIANTS.len());
    for u in &urls {
        if let Some(rel) = images::relative_path(u) {
            for v in images::VARIANTS {
                targets.push(images::with_variant(&rel, v));
            }
        }
    }
    targets.sort();
    targets.dedup();

    let total = targets.len();
    let next = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let fetched = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
    let failed = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
    let bytes = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
    // The rate gate: whoever is about to send a request waits until this instant, then pushes it
    // forward. One shared clock is what keeps four workers from becoming four times the traffic.
    let gate = std::sync::Arc::new(Mutex::new(std::time::Instant::now()));
    let targets = std::sync::Arc::new(targets);

    std::thread::scope(|scope| {
        for _ in 0..IMAGE_WORKERS {
            let (next, fetched, failed, bytes, gate, targets) = (
                next.clone(), fetched.clone(), failed.clone(),
                bytes.clone(), gate.clone(), targets.clone(),
            );
            scope.spawn(move || {
                let agent = ureq::AgentBuilder::new()
                    .timeout_connect(std::time::Duration::from_secs(10))
                    .timeout_read(std::time::Duration::from_secs(30))
                    .build();
                loop {
                    if cancelled() {
                        return;
                    }
                    let i = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    if i >= targets.len() {
                        return;
                    }
                    let rel = &targets[i];
                    let dest = images::cached_file(rel);
                    if dest.is_file() {
                        continue;
                    }
                    {
                        let mut slot = gate.lock().unwrap();
                        let now = std::time::Instant::now();
                        if *slot > now {
                            std::thread::sleep(*slot - now);
                        }
                        *slot = std::time::Instant::now() + IMAGE_DELAY;
                    }
                    match fetch_image(&agent, &format!("{}{}", images::HOST, rel), &dest) {
                        Ok(n) => {
                            fetched.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            bytes.fetch_add(n, std::sync::atomic::Ordering::Relaxed);
                        }
                        Err(_) => {
                            failed.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        }
                    }
                }
            });
        }
        // Progress is reported from here rather than the workers, so the bar advances smoothly
        // instead of four times at once.
        while next.load(std::sync::atomic::Ordering::Relaxed) < total && !cancelled() {
            on_progress(
                next.load(std::sync::atomic::Ordering::Relaxed).min(total),
                total,
                bytes.load(std::sync::atomic::Ordering::Relaxed),
            );
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    });

    let (fetched, failed, bytes) = (
        fetched.load(std::sync::atomic::Ordering::Relaxed),
        failed.load(std::sync::atomic::Ordering::Relaxed),
        bytes.load(std::sync::atomic::Ordering::Relaxed),
    );
    on_progress(total, total, bytes);
    Ok((fetched, failed))
}

fn fetch_image(agent: &ureq::Agent, url: &str, dest: &std::path::Path) -> Result<u64, String> {
    let resp = agent
        .get(url)
        .set("User-Agent", USER_AGENT)
        .call()
        .map_err(|e| e.to_string())?;
    if let Some(dir) = dest.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let mut buf = Vec::new();
    resp.into_reader().read_to_end(&mut buf).map_err(|e| e.to_string())?;
    // Write-then-rename: a crash mid-write must not leave a truncated JPEG that the cache would
    // then happily serve forever, since "the file exists" is the whole skip condition.
    let tmp = dest.with_extension("part");
    std::fs::write(&tmp, &buf).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, dest).map_err(|e| e.to_string())?;
    Ok(buf.len() as u64)
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

    // Art last: the index and synergy are what the app needs to be usable, and this phase runs
    // for hours. Anything already cached is skipped, so re-running is cheap.
    let base = W_DOWNLOAD + W_BUILD + W_SYNERGY;
    let mut on_img = |done: usize, total: usize, bytes: u64| {
        let gb = bytes as f64 / 1_073_741_824.0;
        set_progress(
            &format!("Baixando imagens das cartas ({done}/{total} · {gb:.2} GB)"),
            base + W_IMAGES * (done as f64 / total.max(1) as f64),
        );
    };
    let (fetched, failed) = cache_images(&mut on_img)?;

    Ok(serde_json::json!({
        "cards": n_cards,
        "pt_names": n_pt,
        "synergy_updated": refreshed,
        "images_fetched": fetched,
        "images_failed": failed,
        "images_bytes": images::cache_size_bytes(),
    }))
}
