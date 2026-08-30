//! Games played and the activity log.
//!
//! Both are append-mostly history: the app writes here as a side effect of everything else, and
//! the home screen reads it back. Nothing else depends on these tables, which is why they can
//! stay this simple.

use rusqlite::params;
use serde::{Deserialize, Serialize};

use crate::db::open_app_db;
use crate::error::{Error, Result};
use crate::types::Activity;

/// One recorded game, with the deck it was played with resolved by the join.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Game {
    pub id: i64,
    pub deck_id: i64,
    pub played_at: String,
    /// `vitoria`, `derrota` or `empate` - a CHECK constraint holds the column to those three.
    pub result: String,
    pub opponents: Option<String>,
    pub turns: Option<i64>,
    pub notes: Option<String>,
    pub deck_name: String,
    /// The standout cards recorded with the game, filled in by a second query.
    pub highlights: Vec<String>,
}

/// A card name and how many games it was a highlight of.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HighlightCount {
    pub card_name: String,
    pub n: i64,
}

/// The Partidas header numbers.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GamesStats {
    pub total_games: i64,
    pub wins: i64,
    pub losses: i64,
    pub draws: i64,
    /// Percentage with one decimal, `None` when no game has been recorded -
    /// the screen shows an em dash there rather than a misleading 0%.
    pub win_rate: Option<f64>,
    pub top_highlight_cards: Vec<HighlightCount>,
}

/// Every game, newest first.
///
/// Answers an empty list rather than an error when app.db is missing, like the other history
/// reads: the page is reachable before anything has been recorded.
pub fn list_games() -> Vec<Game> {
    let Ok(con) = open_app_db() else {
        return Vec::new();
    };
    let mut out = (|| -> rusqlite::Result<Vec<Game>> {
        let mut stmt = con.prepare(
            "SELECT games.id, games.deck_id, games.played_at, games.result, games.opponents,
                    games.turns, games.notes, decks.name
             FROM games JOIN decks ON decks.id = games.deck_id ORDER BY games.played_at DESC",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(Game {
                id: r.get(0)?,
                deck_id: r.get(1)?,
                played_at: r.get(2)?,
                result: r.get(3)?,
                opponents: r.get(4)?,
                turns: r.get(5)?,
                notes: r.get(6)?,
                deck_name: r.get(7)?,
                highlights: Vec::new(),
            })
        })?;
        rows.collect()
    })()
    .unwrap_or_default();

    // The standout cards recorded per game - the Partidas page lists them under each row.
    for game in out.iter_mut() {
        game.highlights = (|| -> rusqlite::Result<Vec<String>> {
            let mut stmt =
                con.prepare("SELECT card_name FROM game_highlights WHERE game_id = ?1")?;
            let rows = stmt.query_map([game.id], |r| r.get::<_, String>(0))?;
            rows.collect()
        })()
        .unwrap_or_default();
    }
    out
}

/// Win/loss/draw counts and the cards that carried the most games.
///
/// Answers zeroes rather than an error when app.db is missing, for the same reason
/// [`list_games`] answers an empty list.
pub fn games_stats() -> GamesStats {
    let Ok(con) = open_app_db() else {
        return GamesStats::default();
    };
    let count = |result: &str| -> i64 {
        con.query_row(
            "SELECT COUNT(*) FROM games WHERE result = ?1",
            [result],
            |r| r.get(0),
        )
        .unwrap_or(0)
    };
    let (wins, losses, draws) = (count("vitoria"), count("derrota"), count("empate"));
    let total = wins + losses + draws;
    let win_rate = (total > 0).then(|| ((wins as f64 / total as f64) * 1000.0).round() / 10.0);
    let top: Vec<HighlightCount> = (|| -> rusqlite::Result<Vec<HighlightCount>> {
        let mut stmt = con.prepare(
            "SELECT card_name, COUNT(*) n FROM game_highlights
             GROUP BY card_name ORDER BY n DESC LIMIT 10",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(HighlightCount {
                card_name: r.get(0)?,
                n: r.get(1)?,
            })
        })?;
        rows.collect()
    })()
    .unwrap_or_default();
    GamesStats {
        total_games: total,
        wins,
        losses,
        draws,
        win_rate,
        top_highlight_cards: top,
    }
}

/// The activity feed, newest first. `limit` was 30 when the UI did not say otherwise.
pub fn list_activity(limit: i64) -> Vec<Activity> {
    let Ok(con) = open_app_db() else {
        return Vec::new();
    };
    (|| -> rusqlite::Result<Vec<Activity>> {
        let mut stmt = con.prepare(
            "SELECT id, ts, type, description FROM activity ORDER BY ts DESC, id DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map([limit], |r| {
            Ok(Activity {
                ts: r.get(1)?,
                kind: r.get(2)?,
                description: r.get(3)?,
            })
        })?;
        rows.collect()
    })()
    .unwrap_or_default()
}

/// A game as the Registrar partida form describes it.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameIn {
    pub deck_id: i64,
    pub played_at: String,
    pub result: String,
    #[serde(default)]
    pub opponents: Option<String>,
    #[serde(default)]
    pub turns: Option<i64>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub highlights: Vec<String>,
}

/// Records a game and its highlights. Returns the new game's id.
pub fn add_game(p: GameIn) -> Result<i64> {
    let Ok(con) = open_app_db() else {
        return Err(Error::db_unavailable());
    };
    if con
        .execute(
            "INSERT INTO games (deck_id, played_at, result, opponents, turns, notes)
             VALUES (?1,?2,?3,?4,?5,?6)",
            params![
                p.deck_id,
                p.played_at,
                p.result,
                p.opponents,
                p.turns,
                p.notes
            ],
        )
        .is_err()
    {
        // The CHECK constraint on result rejects anything outside vitoria/derrota/empate.
        return Err(Error::BadRequest(
            "Não foi possível registrar a partida".into(),
        ));
    }
    let game_id = con.last_insert_rowid();
    for card_name in &p.highlights {
        let _ = con.execute(
            "INSERT INTO game_highlights (game_id, card_name) VALUES (?1, ?2)",
            params![game_id, card_name],
        );
    }
    Ok(game_id)
}
