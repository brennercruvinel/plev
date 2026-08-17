import { api } from "../api.js?v=25";
import { mainEl } from "../router.js?v=3";
import { openAddCardModal } from "../ui/add-card.js?v=3";
import { showCardModal } from "../ui/card-modal.js?v=3";
import { h } from "../util.js?v=3";
import { attachClear } from "../ui/search-field.js?v=1";
import { cardImgHtml, wireCardFlips } from "../ui/card-face.js?v=1";
import {
  filterDropdownHtml, filterMenuContentHtml, filterToggleBtnHtml, matchesFilters, newFilterState,
  wireFilterMenu,
} from "../ui/card-filters.js?v=3";

let collectionFilter = "all";
// Same type/colour/CMC/rarity filters as the deck page, plus rarity — kept at module level so a
// filtered view survives opening a card and coming back.
const collFilters = newFilterState();

// Sort order, module-level like the filters so it survives opening a card and coming back.
// "added" uses the acquisition dates the API returns per card: a card with several copies has
// several dates, so newest sorts on the most recent and oldest on the first.
let collSort = "name-asc";
const COLL_SORTS = {
  "name-asc": "Nome (A-Z)",
  "name-desc": "Nome (Z-A)",
  "added-desc": "Adicionado por último",
  "added-asc": "Adicionado primeiro",
};

function sortCollection(items) {
  const byName = (a, b) => a.card_name.localeCompare(b.card_name, "pt-BR");
  // Cards predating the created_at column sort last under "newest" and first under "oldest",
  // which is where an unknown date honestly belongs in each direction.
  const ts = (v, fallback) => (v ? Date.parse(v.replace(" ", "T") + "Z") : fallback);
  const arr = [...items];
  switch (collSort) {
    case "name-desc":
      arr.sort((a, b) => byName(b, a));
      break;
    case "added-desc":
      arr.sort((a, b) => ts(b.last_added, 0) - ts(a.last_added, 0) || byName(a, b));
      break;
    case "added-asc":
      arr.sort((a, b) => ts(a.first_added, Infinity) - ts(b.first_added, Infinity) || byName(a, b));
      break;
    default:
      arr.sort(byName);
  }
  return arr;
}

export async function renderCollection() {
  mainEl.innerHTML = h`
    <div class="page-header">
      <div><h1>Coleção</h1><p>Toda carta possuída — vermelho e opaco quando alocada a um deck (com o nome do deck/comandante); sem nada por cima quando livre.</p></div>
      <button class="btn small" id="add-card-btn">+ Adicionar Carta</button>
    </div>
    <div class="filters-bar">
      <input type="text" id="coll-search" placeholder="Buscar carta (PT ou EN)…">
      <span class="chip active" data-filter="all">Todas</span>
      <span class="chip" data-filter="allocated">Em decks</span>
      <span class="chip" data-filter="free">Livres</span>
      ${filterDropdownHtml(collFilters, { rarity: true, search: false })}
      <div class="sort-icon-wrap" title="Ordenar">
        <span class="sort-icon-glyph"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m21 16-4 4-4-4"/><path d="M17 20V4"/><path d="m3 8 4-4 4 4"/><path d="M7 4v16"/></svg></span>
        <select id="coll-sort" class="sort-icon-select" aria-label="Ordenar">
          ${Object.entries(COLL_SORTS)
            .map(([v, label]) => `<option value="${v}" ${collSort === v ? "selected" : ""}>${label}</option>`)
            .join("")}
        </select>
      </div>
      <div class="size-slider-row">
        <label for="card-size">Tamanho</label>
        <input type="range" id="card-size" min="100" max="280" step="10" value="160">
      </div>
    </div>
    <div class="card-grid" id="coll-grid"></div>
  `;

  const grid = document.getElementById("coll-grid");
  document.getElementById("card-size").addEventListener("input", (e) => {
    grid.style.setProperty("--card-min", `${e.target.value}px`);
  });

  async function load() {
    const q = document.getElementById("coll-search").value.trim();
    const all = await api.collection(collectionFilter, q);
    // Server-side search stays (it resolves translated names); the chip filters run here, over
    // data the list already carries, so toggling a chip doesn't cost a round trip.
    const items = sortCollection(all.filter((c) => matchesFilters(c, collFilters)));
    grid.innerHTML = items
      .map((c) => {
        // Each entry is a physical copy (or a stack of them): a card sleeved in two decks shows
        // up as two entries, because it is two cards. Sum them per deck so the tile can say how
        // many copies exist and where they are, instead of just naming the decks.
        const inDecks = c.decks.filter((d) => d.deck_name !== "Livre");
        const free = c.decks.reduce((n, d) => (d.deck_name === "Livre" ? n + d.quantity : n), 0);
        const perDeck = new Map();
        for (const d of inDecks) perDeck.set(d.deck_name, (perDeck.get(d.deck_name) || 0) + d.quantity);
        const isAllocated = perDeck.size > 0;
        const deckLabel = [...perDeck]
          .map(([name, qty]) => (qty > 1 ? `${name} (${qty}x)` : name))
          .join(" + ");
        const title = [
          `${c.total_quantity} cópia(s) de ${c.card_name}`,
          free ? `${free} livre(s)` : null,
          ...[...perDeck].map(([name, qty]) => `${qty} em ${name}`),
        ].filter(Boolean).join(" · ");
        return h`
          <div class="mtg-card ${isAllocated ? "allocated" : ""}" data-card-view="${c.card_name}" title="${title}">
            ${cardImgHtml(c, { attrs: 'loading="lazy" decoding="async"' })}
            <span class="qty-badge">${c.total_quantity}x</span>
            ${free && isAllocated ? `<span class="free-badge">${free} livre${free > 1 ? "s" : ""}</span>` : ""}
            ${isAllocated ? `<div class="deck-badge">${deckLabel}</div>` : ""}
          </div>`;
      })
      .join("") ||
      `<div class="empty-state">${all.length ? "Nenhuma carta corresponde aos filtros." : "Nada encontrado."}</div>`;
  }

  // One delegated listener on the grid instead of one per tile — the collection renders hundreds
  // of cards, and load() re-runs on every filter/search change, so per-tile binding meant
  // re-attaching hundreds of listeners each time.
  wireCardFlips(grid);
  grid.addEventListener("click", (e) => {
    const tile = e.target.closest("[data-card-view]");
    // load() so adding a unit from the modal updates the tile's count behind it.
    if (tile) showCardModal(tile.dataset.cardView, load);
  });

  document.querySelectorAll("[data-filter]").forEach((chip) =>
    chip.addEventListener("click", () => {
      document.querySelectorAll("[data-filter]").forEach((c) => c.classList.remove("active"));
      chip.classList.add("active");
      collectionFilter = chip.dataset.filter;
      load();
    })
  );
  // Debounced: without this every keystroke fired a full /api/collection request and re-rendered
  // the whole grid. Same 250ms used by the card-name autocompletes elsewhere in this file.
  document.getElementById("coll-sort").addEventListener("change", (e) => {
    collSort = e.target.value;
    load();
  });

  attachClear(document.getElementById("coll-search"));

  let searchDebounce;
  document.getElementById("coll-search").addEventListener("input", () => {
    clearTimeout(searchDebounce);
    searchDebounce = setTimeout(load, 250);
  });
  document.getElementById("add-card-btn").addEventListener("click", () =>
    openAddCardModal({ onSaved: load })
  );

  // Mirrors refreshDeckFilterBar: redraw the badge and the chip states without touching the
  // #filter-dropdown wrapper, so the menu doesn't slam shut every time a chip is toggled.
  function refreshFilterMenu() {
    document.getElementById("filter-toggle-btn").outerHTML = filterToggleBtnHtml(collFilters);
    document.getElementById("filter-menu").outerHTML = filterMenuContentHtml(collFilters, {
      rarity: true,
      search: false, // the collection's own search box above is server-side and multi-language
    });
    wireFilters();
    load();
  }
  function wireFilters() {
    wireFilterMenu(collFilters, (structural) => (structural ? refreshFilterMenu() : load()));
  }
  wireFilters();

  load();
}

// --------------------------------------------------------------- games ----
