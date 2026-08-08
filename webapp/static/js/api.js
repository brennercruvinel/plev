const BASE = "/api";

async function req(path, opts = {}) {
  const res = await fetch(BASE + path, {
    headers: { "Content-Type": "application/json" },
    ...opts,
  });
  if (!res.ok) {
    const text = await res.text();
    let body;
    try { body = JSON.parse(text); } catch { body = text; }
    // status + parsed body are attached to the thrown error so callers can react to specific
    // cases (e.g. 409 "card already in deck, needs confirmation") instead of just showing text.
    const err = new Error(`${res.status} ${path}: ${text}`);
    err.status = res.status;
    err.body = body;
    throw err;
  }
  return res.json();
}

export const api = {
  decks: () => req("/decks"),
  deck: (id) => req(`/decks/${id}`),
  createDeck: (payload) => req("/decks", { method: "POST", body: JSON.stringify(payload) }),
  updateDeck: (id, payload) => req(`/decks/${id}`, { method: "PUT", body: JSON.stringify(payload) }),
  deleteDeck: (id) => req(`/decks/${id}`, { method: "DELETE" }),
  startAutoBuildDeck: (payload) => req("/decks/auto-build", { method: "POST", body: JSON.stringify(payload) }),
  autoBuildStatus: () => req("/decks/auto-build/status"),
  deckSynergy: (id) => req(`/decks/${id}/synergy`),
  fetchDeckSynergy: (id) => req(`/decks/${id}/synergy/fetch`, { method: "POST" }),
  deckTags: (id) => req(`/decks/${id}/tags`),
  addDeckCard: (id, card_name, quantity = 1, oracle_id = null, confirm = false) =>
    req(`/decks/${id}/cards`, { method: "POST", body: JSON.stringify({ card_name, quantity, oracle_id, confirm }) }),
  removeDeckCard: (deckId, cardId) => req(`/decks/${deckId}/cards/${cardId}`, { method: "DELETE" }),

  // Decklist import — text/CSV go through req() as JSON; the file variant needs
  // multipart/form-data, so it bypasses req()'s forced "Content-Type: application/json".
  importPreviewText: (id, text) =>
    req(`/decks/${id}/import/preview`, { method: "POST", body: JSON.stringify({ text }) }),
  importPreviewFile: async (id, file) => {
    const fd = new FormData();
    fd.append("file", file);
    const res = await fetch(`${BASE}/decks/${id}/import/preview-file`, { method: "POST", body: fd });
    if (!res.ok) throw new Error(`${res.status}: ${await res.text()}`);
    return res.json();
  },
  importCommit: (id, cards, mode) =>
    req(`/decks/${id}/import/commit`, { method: "POST", body: JSON.stringify({ cards, mode }) }),

  searchCards: (q, limit = 24) => req(`/cards/search?q=${encodeURIComponent(q)}&limit=${limit}`),
  card: (name, oracleId = null) =>
    req(`/cards/${encodeURIComponent(name)}${oracleId ? `?oracle_id=${encodeURIComponent(oracleId)}` : ""}`),
  cardVariants: (name) => req(`/cards/${encodeURIComponent(name)}/variants`),
  scanRecognize: (text) => req("/scan/recognize", { method: "POST", body: JSON.stringify({ text }) }),

  collection: (status = "all", q = "") => req(`/collection?status=${status}&q=${encodeURIComponent(q)}`),
  collectionDuplicates: () => req("/collection/duplicates"),
  collectionTotal: () => req("/collection/total"),
  addCollection: (payload) => req("/collection", { method: "POST", body: JSON.stringify(payload) }),
  bulkResolveCollection: (text) => req("/collection/bulk-resolve", { method: "POST", body: JSON.stringify({ text }) }),
  allocate: (id, deck_id) => req(`/collection/${id}/allocate`, { method: "PATCH", body: JSON.stringify({ deck_id }) }),

  activity: (limit = 30) => req(`/activity?limit=${limit}`),

  dataInfo: () => req("/data/info"),
  startDataUpdate: () => req("/data/update", { method: "POST" }),
  dataUpdateStatus: () => req("/data/update/status"),

  games: (deckId) => req(`/games${deckId ? `?deck_id=${deckId}` : ""}`),
  addGame: (payload) => req("/games", { method: "POST", body: JSON.stringify(payload) }),
  gamesStats: (deckId) => req(`/games/stats${deckId ? `?deck_id=${deckId}` : ""}`),
};
