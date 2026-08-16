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
  // mode: "free" (default) leaves the deck's cards in the collection, unallocated;
  // "remove" also deletes those collection rows, for a deck whose cards were never owned.
  deleteDeck: (id, mode = "free") => req(`/decks/${id}?mode=${mode}`, { method: "DELETE" }),
  startAutoBuildDeck: (payload) => req("/decks/auto-build", { method: "POST", body: JSON.stringify(payload) }),
  autoBuildStatus: () => req("/decks/auto-build/status"),
  deckSynergy: (id) => req(`/decks/${id}/synergy`),
  fetchDeckSynergy: (id) => req(`/decks/${id}/synergy/fetch`, { method: "POST" }),
  deckTags: (id) => req(`/decks/${id}/tags`),
  addDeckCard: (id, card_name, quantity = 1, oracle_id = null, confirm = false) =>
    req(`/decks/${id}/cards`, { method: "POST", body: JSON.stringify({ card_name, quantity, oracle_id, confirm }) }),
  removeDeckCard: (deckId, cardId) => req(`/decks/${deckId}/cards/${cardId}`, { method: "DELETE" }),

  // Decklist import. There is deliberately no file-upload variant: uploaded .txt/.csv files are
  // read in the browser and dropped into the textarea, so every import — pasted or loaded from
  // disk — goes through this one endpoint and shows the user exactly what will be imported.
  importPreviewText: (id, text) =>
    req(`/decks/${id}/import/preview`, { method: "POST", body: JSON.stringify({ text }) }),
  importCommit: (id, cards, mode) =>
    req(`/decks/${id}/import/commit`, { method: "POST", body: JSON.stringify({ cards, mode }) }),

  // Set autocomplete — names are English (Scryfall doesn't localise them), so the code matches too.
  // `card` scopes the list to sets that actually printed it — without it the field happily
  // accepts a set the card was never in, which then resolves to no artwork.
  sets: (q, card = "", limit = 12) =>
    req(`/sets?q=${encodeURIComponent(q)}&limit=${limit}${card ? `&card=${encodeURIComponent(card)}` : ""}`),

  // Wishlist: cards wanted but not owned. Same grouped shape as the collection.
  wishlist: (q = "") => req(`/wishlist?q=${encodeURIComponent(q)}`),
  wishlistTotal: () => req("/wishlist/total"),
  addWishlist: (payload) => req("/wishlist", { method: "POST", body: JSON.stringify(payload) }),
  deleteWishlistEntry: (id) => req(`/wishlist/${id}`, { method: "DELETE" }),
  acquireWishlistEntry: (id) => req(`/wishlist/${id}/acquire`, { method: "POST" }),

  searchCards: (q, limit = 24) => req(`/cards/search?q=${encodeURIComponent(q)}&limit=${limit}`),
  card: (name, oracleId = null) =>
    req(`/cards/${encodeURIComponent(name)}${oracleId ? `?oracle_id=${encodeURIComponent(oracleId)}` : ""}`),
  cardVariants: (name) => req(`/cards/${encodeURIComponent(name)}/variants`),
  scanRecognize: (text) => req("/scan/recognize", { method: "POST", body: JSON.stringify({ text }) }),

  collection: (status = "all", q = "") => req(`/collection?status=${status}&q=${encodeURIComponent(q)}`),
  collectionDuplicates: () => req("/collection/duplicates"),
  collectionTotal: () => req("/collection/total"),
  // Removes one physical copy from an entry; the row goes away only when its last unit does.
  deleteCollectionEntry: (id) => req(`/collection/${id}`, { method: "DELETE" }),
  // Edits a stored copy in place. Cannot change the card or its deck — see the Rust handler.
  editCollectionEntry: (id, payload) =>
    req(`/collection/${id}`, { method: "PATCH", body: JSON.stringify(payload) }),
  // Physical copies of one exact card: how many are free, and which decks hold the rest.
  // Every distinct artwork of a card — 8k of 38k cards have more than one.
  cardPrintings: (name) => req(`/cards/${encodeURIComponent(name)}/printings`),
  cardCopies: (name) => req(`/collection/copies?name=${encodeURIComponent(name)}`),
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
