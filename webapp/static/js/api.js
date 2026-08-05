const BASE = "/api";

async function req(path, opts = {}) {
  const res = await fetch(BASE + path, {
    headers: { "Content-Type": "application/json" },
    ...opts,
  });
  if (!res.ok) {
    const body = await res.text();
    throw new Error(`${res.status} ${path}: ${body}`);
  }
  return res.json();
}

export const api = {
  decks: () => req("/decks"),
  deck: (id) => req(`/decks/${id}`),
  createDeck: (payload) => req("/decks", { method: "POST", body: JSON.stringify(payload) }),
  deleteDeck: (id) => req(`/decks/${id}`, { method: "DELETE" }),
  deckSynergy: (id) => req(`/decks/${id}/synergy`),
  addDeckCard: (id, card_name, quantity = 1) =>
    req(`/decks/${id}/cards`, { method: "POST", body: JSON.stringify({ card_name, quantity }) }),
  removeDeckCard: (deckId, cardId) => req(`/decks/${deckId}/cards/${cardId}`, { method: "DELETE" }),

  searchCards: (q, limit = 24) => req(`/cards/search?q=${encodeURIComponent(q)}&limit=${limit}`),
  card: (name) => req(`/cards/${encodeURIComponent(name)}`),
  scanRecognize: (text) => req("/scan/recognize", { method: "POST", body: JSON.stringify({ text }) }),

  collection: (status = "all", q = "") => req(`/collection?status=${status}&q=${encodeURIComponent(q)}`),
  collectionDuplicates: () => req("/collection/duplicates"),
  collectionTotal: () => req("/collection/total"),
  addCollection: (payload) => req("/collection", { method: "POST", body: JSON.stringify(payload) }),
  allocate: (id, deck_id) => req(`/collection/${id}/allocate`, { method: "PATCH", body: JSON.stringify({ deck_id }) }),

  activity: (limit = 30) => req(`/activity?limit=${limit}`),

  games: (deckId) => req(`/games${deckId ? `?deck_id=${deckId}` : ""}`),
  addGame: (payload) => req("/games", { method: "POST", body: JSON.stringify(payload) }),
  gamesStats: (deckId) => req(`/games/stats${deckId ? `?deck_id=${deckId}` : ""}`),
};
