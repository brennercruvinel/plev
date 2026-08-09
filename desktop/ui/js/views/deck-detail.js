import { api } from "../api.js?v=25";
import { manaCostHtml, manaGlyphSvg } from "../icons.js?v=25";
import { FILTER_COLORS, deckTagsHtml } from "../deck-bits.js?v=2";
import { mainEl } from "../router.js?v=2";
import { showCardModal } from "../ui/card-modal.js?v=2";
import { h } from "../util.js?v=2";
import { openDeleteDeckModal, openEditDeckModal, openImportDeckModal } from "../views/decks.js?v=2";

const CATEGORY_LABELS = {
  Comandante: "Comandante", Land: "Terrenos", Creature: "Criaturas",
  Instant: "Instantâneas", Sorcery: "Feitiços", Artifact: "Artefatos",
  Enchantment: "Encantamentos", Planeswalker: "Planeswalker", Outro: "Outro",
};
const CATEGORY_ORDER = ["Comandante", "Creature", "Instant", "Sorcery", "Artifact", "Enchantment", "Planeswalker", "Outro", "Land"];
// Short labels just for the type filter chips — the full CATEGORY_LABELS names (used for section
// headers) run long enough that six of them plus the color/CMC rows push the filter bar to wrap
// across several lines; these abbreviations keep the whole bar closer to a single line.
const FILTER_TYPE_LABELS = {
  Land: "Terreno", Creature: "Criatura", Instant: "Inst.", Sorcery: "Feit.",
  Artifact: "Art.", Enchantment: "Enc.", Planeswalker: "PW", Outro: "Outro",
};
// Filterable by the type chips — the commander(s) are always shown regardless of filters, so this
// list intentionally excludes "Comandante": losing sight of your own commander via a filter would be confusing.
const FILTERABLE_TYPES = CATEGORY_ORDER.filter((c) => c !== "Comandante");
const CMC_BUCKETS = ["0", "1", "2", "3", "4", "5", "6+"];

let deckViewMode = "stack"; // "list", "grid" (MTG Arena tiles), or "stack" (Moxfield-style overlap) — default view
const VIEW_MODES = ["list", "grid", "stack"];
const VIEW_MODE_LABELS = { list: "Lista", grid: "Visual", stack: "Empilhado" };

// Deck card filter/sort state — module-level so it survives view-mode switches and persists
// across decks in the same session (matches how deckViewMode already behaves).
// groupBy: "type" (default, existing category headers) or "tag" (EDHREC theme tags — see
// currentDeckTags / filteredDeckByTag below).
let deckFilters = { q: "", types: new Set(), colors: new Set(), cmcs: new Set(), sort: "name", groupBy: "type" };

// card_name -> [tag, ...], populated per-deck from GET /decks/:id/tags (EDHREC cardlist
// headers for the deck's commander) right before renderDeckCards can use it.
let currentDeckTags = {};

// Collapsed by default (see synergyPanelHtml) — not something you need to see every time you
// open a deck. Tracked at module level so it survives the re-render triggered by adding/removing
// a card instead of snapping shut again on every action.
let synergyPanelOpen = false;

function deckFiltersActive() {
  return !!(deckFilters.q || deckFilters.types.size || deckFilters.colors.size || deckFilters.cmcs.size);
}

function cardCmcBucket(cmc) {
  const n = Math.floor(cmc || 0);
  return n >= 6 ? "6+" : String(n);
}

// Client-side mirror of the backend's classify() — needed so per-card type filtering still
// works when cards are grouped by tag instead of by type (there's no "category" bucket to
// pre-filter by in that mode, so each card has to be checked individually).
function cardCategory(c) {
  const t = c.type_line || "";
  for (const cat of ["Land", "Creature", "Planeswalker", "Battle", "Artifact", "Enchantment", "Instant", "Sorcery"]) {
    if (t.includes(cat)) return cat;
  }
  return "Outro";
}

function cardMatchesFilters(c) {
  if (deckFilters.q && !c.card_name.toLowerCase().includes(deckFilters.q.toLowerCase())) return false;
  if (deckFilters.types.size && !deckFilters.types.has(cardCategory(c))) return false;
  if (deckFilters.colors.size) {
    const letters = c.colors ? c.colors.split("") : [];
    const isColorless = letters.length === 0;
    const hit = (isColorless && deckFilters.colors.has("C")) || letters.some((l) => deckFilters.colors.has(l));
    if (!hit) return false;
  }
  if (deckFilters.cmcs.size && !deckFilters.cmcs.has(cardCmcBucket(c.cmc))) return false;
  return true;
}

function sortDeckCards(cards) {
  const arr = [...cards];
  switch (deckFilters.sort) {
    case "cmc-asc":
      arr.sort((a, b) => (a.cmc ?? 0) - (b.cmc ?? 0) || a.card_name.localeCompare(b.card_name));
      break;
    case "cmc-desc":
      arr.sort((a, b) => (b.cmc ?? 0) - (a.cmc ?? 0) || a.card_name.localeCompare(b.card_name));
      break;
    case "price-desc":
      arr.sort((a, b) => (parseFloat(b.price_usd) || 0) - (parseFloat(a.price_usd) || 0));
      break;
    case "qty-desc":
      arr.sort((a, b) => b.quantity - a.quantity || a.card_name.localeCompare(b.card_name));
      break;
    default:
      arr.sort((a, b) => a.card_name.localeCompare(b.card_name));
  }
  return arr;
}

/** Applies deckFilters to deck.by_type, returning a same-shaped object. Commander is never filtered out. */
function filteredDeckByType(deck) {
  const out = {};
  for (const cat of CATEGORY_ORDER) {
    const cards = deck.by_type[cat];
    if (!cards) continue;
    if (cat === "Comandante") {
      out[cat] = cards;
      continue;
    }
    if (deckFilters.types.size && !deckFilters.types.has(cat)) continue;
    const filtered = sortDeckCards(cards.filter(cardMatchesFilters));
    if (filtered.length) out[cat] = filtered;
  }
  return out;
}

/** Groups deck cards (excluding the commander, same as filteredDeckByType) by their EDHREC
 * theme tags ("Subtipo") instead of by card type. A card with 2+ tags (e.g. both "Removal" and
 * "Utility Creature") shows up under each — that's expected for a theme browser, not a bug: the
 * same card really does serve both roles. Cards with no cached EDHREC tag fall back to their own
 * card type (the same category filteredDeckByType would've used) instead of a generic
 * catch-all group, so every card still lands somewhere meaningful. */
function filteredDeckByTag(deck, tagsMap) {
  const allCards = Object.entries(deck.by_type || {})
    .filter(([cat]) => cat !== "Comandante")
    .flatMap(([, cards]) => cards)
    .filter(cardMatchesFilters);
  const out = {};
  for (const c of allCards) {
    const tags = tagsMap[c.card_name]?.length ? tagsMap[c.card_name] : [CATEGORY_LABELS[cardCategory(c)] || cardCategory(c)];
    for (const t of tags) {
      (out[t] = out[t] || []).push(c);
    }
  }
  for (const t of Object.keys(out)) out[t] = sortDeckCards(out[t]);
  return out;
}

// Every non-commander card in the deck, same filtering (search/type/color/CMC) applied
// regardless of how the results end up grouped.
function nonCommanderFilteredCards(deck) {
  return Object.entries(deck.by_type || {})
    .filter(([cat]) => cat !== "Comandante")
    .flatMap(([, cards]) => cards)
    .filter(cardMatchesFilters);
}

/** Groups pre-filtered cards by a single-valued key function — shared by the color/CMC/rarity
 * group-by modes (each card lands in exactly one bucket, unlike the tag mode above where a
 * card can have several tags at once). */
function groupCardsByKey(cards, keyFn) {
  const out = {};
  for (const c of cards) (out[keyFn(c)] = out[keyFn(c)] || []).push(c);
  for (const k of Object.keys(out)) out[k] = sortDeckCards(out[k]);
  return out;
}

const COLOR_GROUP_LABELS = { W: "Branco", U: "Azul", B: "Preto", R: "Vermelho", G: "Verde", M: "Multicolor", C: "Incolor" };
function colorGroupKey(c) {
  const letters = (c.colors || "").split("").filter(Boolean);
  return letters.length === 0 ? "C" : letters.length === 1 ? letters[0] : "M";
}

const RARITY_ORDER = ["common", "uncommon", "rare", "mythic", "special", "bonus"];
const RARITY_LABELS = { common: "Comum", uncommon: "Incomum", rare: "Rara", mythic: "Mítica", special: "Especial", bonus: "Bônus" };
function rarityGroupKey(c) {
  return c.rarity || "outro";
}

const GROUP_MODES = ["type", "tag", "color", "cmc", "rarity"];
const GROUP_MODE_LABELS = { type: "Tipo", tag: "Subtipo", color: "Cor", cmc: "Custo", rarity: "Raridade" };

/** Single entry point for "how are the deck's cards currently organized" — returns the grouped
 * cards, the order/set of group keys to render, and how to turn a key into a display label.
 * Each group-by mode (see GROUP_MODES) plugs in here rather than renderDeckCards branching on
 * deckFilters.groupBy directly, so adding another mode later is a single addition to this switch. */
function computeDeckGroups(deck) {
  switch (deckFilters.groupBy) {
    case "tag": {
      const groups = filteredDeckByTag(deck, currentDeckTags);
      return { groups, groupKeys: Object.keys(groups).sort((a, b) => a.localeCompare(b)), groupLabel: (k) => k };
    }
    case "color": {
      const groups = groupCardsByKey(nonCommanderFilteredCards(deck), colorGroupKey);
      return { groups, groupKeys: CURVE_COLOR_ORDER.filter((c) => groups[c]?.length), groupLabel: (k) => COLOR_GROUP_LABELS[k] || k };
    }
    case "cmc": {
      const groups = groupCardsByKey(nonCommanderFilteredCards(deck), (c) => cardCmcBucket(c.cmc));
      return { groups, groupKeys: CMC_BUCKETS.filter((v) => groups[v]?.length), groupLabel: (k) => k };
    }
    case "rarity": {
      const groups = groupCardsByKey(nonCommanderFilteredCards(deck), rarityGroupKey);
      const order = [...RARITY_ORDER, ...Object.keys(groups).filter((k) => !RARITY_ORDER.includes(k))];
      return { groups, groupKeys: order.filter((k) => groups[k]?.length), groupLabel: (k) => RARITY_LABELS[k] || (k === "outro" ? "Outra" : k) };
    }
    default: {
      const groups = filteredDeckByType(deck);
      return { groups, groupKeys: CATEGORY_ORDER.filter((c) => c !== "Comandante" && groups[c]?.length), groupLabel: (k) => CATEGORY_LABELS[k] || k };
    }
  }
}

/** Just the "Filtro" trigger button — a separate function from the menu body below so refreshing
 * the filter state (deckFiltersActive() badge) can swap the button without touching the open/
 * closed state of the menu, which lives on the wrapping #filter-dropdown element instead.
 * Icon-only circular button, deliberately styled to match the sort control right next to it
 * (.sort-icon-wrap/.sort-icon-glyph) rather than a labeled .btn — a plain <button> here instead
 * of the invisible-<select>-overlay trick sort uses, since this opens a custom menu, not a
 * native dropdown. */
function filterToggleBtnHtml() {
  return h`
    <button type="button" class="filter-toggle-btn" id="filter-toggle-btn" title="Filtro" aria-label="Filtro" aria-haspopup="true" aria-expanded="false">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 3H2l8 9.46V19l4 2v-8.54Z"/></svg>
      ${deckFiltersActive() ? `<span class="filter-badge"></span>` : ""}
    </button>`;
}

/** The filter menu body (search/type/color/CMC) — previously its own always-visible bar
 * (.filters-bar) above the card list; now tucked behind the "Filtro" button in the view/group/
 * sort toolbar instead, so it only takes up screen space while you're actually adjusting it. */
function filterMenuContentHtml() {
  const typeChips = FILTERABLE_TYPES.map(
    (t) => `<span class="chip type-chip ${deckFilters.types.has(t) ? "active" : ""}" data-deck-type="${t}" title="${CATEGORY_LABELS[t]}">${FILTER_TYPE_LABELS[t]}</span>`
  ).join("");
  const colorChips = Object.entries(FILTER_COLORS)
    .map(([c, bg]) => {
      const dimmed = deckFilters.colors.size && !deckFilters.colors.has(c);
      const active = deckFilters.colors.has(c);
      const glyph = manaGlyphSvg(c);
      return `<span class="chip color-chip ${active ? "active" : ""}" data-deck-color="${c}"
        style="background:${bg};color:#1a1a1a;opacity:${dimmed ? 0.35 : 1}">${glyph ? `<span class="mana-sym-inline">${glyph}</span>` : c}</span>`;
    })
    .join("");
  const cmcChips = CMC_BUCKETS.map(
    (v) => `<span class="chip ${deckFilters.cmcs.has(v) ? "active" : ""}" data-deck-cmc="${v}">${v}</span>`
  ).join("");

  return h`
    <div class="filter-menu" id="filter-menu">
      <div class="filter-group filter-group-block">
        <span class="filter-group-label">Buscar</span>
        <input type="text" id="deck-filter-q" placeholder="Nome da carta no deck…" value="${deckFilters.q}">
      </div>
      <div class="filter-group"><span class="filter-group-label">Tipo</span>${typeChips}</div>
      <div class="filter-group"><span class="filter-group-label">Cor</span>${colorChips}</div>
      <div class="filter-group"><span class="filter-group-label">CMC</span>${cmcChips}</div>
      ${deckFiltersActive() ? `<button class="btn small secondary" id="deck-filter-clear">Limpar filtros</button>` : ""}
    </div>`;
}

function filterMenuHtml() {
  return h`
    <div class="filter-dropdown" id="filter-dropdown">
      ${filterToggleBtnHtml()}
      ${filterMenuContentHtml()}
    </div>`;
}

/** Filter menu + view mode + group-by + sort controls, all in one toolbar. The search/type/color/
 * CMC filters (what's included) live behind the "Filtro" dropdown (see filterMenuHtml above) so
 * they don't compete for space with everything else here, which controls how the resulting cards
 * are displayed and organized — this toolbar lives right above the card groups it affects. */
function viewControlsHtml(tagsAvailable) {
  const viewChips = VIEW_MODES.map(
    (v) => `<span class="chip ${deckViewMode === v ? "active" : ""}" data-view="${v}">${VIEW_MODE_LABELS[v]}</span>`
  ).join("");
  const groupChips = GROUP_MODES.map((mode) => {
    const hint = mode === "tag" && !tagsAvailable
      ? ' title="Sem tags do EDHREC cacheadas para este comandante — cartas sem tag usam o tipo"'
      : "";
    return `<span class="chip ${deckFilters.groupBy === mode ? "active" : ""}" data-group-by="${mode}"${hint}>${GROUP_MODE_LABELS[mode]}</span>`;
  }).join("");
  return h`
    <div class="filters-bar cards-toolbar" id="cards-toolbar">
      <div class="filter-group">
        <span class="filter-group-label">Adicionar Card</span>
        <div class="search-collapse add-card-inline" id="add-card-inline">
          <button type="button" class="search-icon-btn" id="add-card-toggle" title="Adicionar carta ao deck" aria-label="Adicionar carta ao deck">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="9"/><path d="M12 8v8M8 12h8"/></svg>
          </button>
          <input type="text" id="add-card-input" class="search-collapse-input" placeholder="Adicionar carta (PT ou EN)…" autocomplete="off">
          <div id="add-card-results" class="add-card-results-dropdown"></div>
        </div>
      </div>
      <div class="filter-group">
        <span class="filter-group-label">Visualização</span>
        ${viewChips}
      </div>
      <div class="filter-group">
        <span class="filter-group-label">Agrupar</span>
        ${groupChips}
      </div>
      <div class="filter-group">
        <span class="filter-group-label">Filtro</span>
        ${filterMenuHtml()}
      </div>
      <div class="filter-group">
        <span class="filter-group-label">Ordenar</span>
        <div class="sort-icon-wrap" title="Ordenar">
          <span class="sort-icon-glyph"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m21 16-4 4-4-4"/><path d="M17 20V4"/><path d="m3 8 4-4 4 4"/><path d="M7 4v16"/></svg></span>
          <select id="deck-sort" class="sort-icon-select" aria-label="Ordenar">
            <option value="name" ${deckFilters.sort === "name" ? "selected" : ""}>Nome (A-Z)</option>
            <option value="cmc-asc" ${deckFilters.sort === "cmc-asc" ? "selected" : ""}>Custo de mana ↑</option>
            <option value="cmc-desc" ${deckFilters.sort === "cmc-desc" ? "selected" : ""}>Custo de mana ↓</option>
            <option value="price-desc" ${deckFilters.sort === "price-desc" ? "selected" : ""}>Preço ↓</option>
            <option value="qty-desc" ${deckFilters.sort === "qty-desc" ? "selected" : ""}>Quantidade ↓</option>
          </select>
        </div>
      </div>
    </div>`;
}

function wireCardsToolbar(deck) {
  document.querySelectorAll("[data-view]").forEach((chip) =>
    chip.addEventListener("click", () => {
      deckViewMode = chip.dataset.view;
      refreshCardsToolbar(deck);
    })
  );
  document.querySelectorAll("[data-group-by]").forEach((chip) =>
    chip.addEventListener("click", () => {
      deckFilters.groupBy = chip.dataset.groupBy;
      refreshCardsToolbar(deck);
    })
  );
  document.getElementById("deck-sort").addEventListener("change", (e) => {
    deckFilters.sort = e.target.value;
    renderDeckCards(deck);
  });
  wireAddCardInline(deck);
  wireDeckFilterBar(deck);
}

/** The "Adicionar carta" search — lives right in the view/group/sort toolbar (next to the card
 * groups it fills), collapsed to a plain icon until clicked. Results show as a dropdown under
 * the input instead of a permanent sidebar block, since it's only needed while actively
 * searching. Re-wired every time the toolbar re-renders (wireCardsToolbar → here), same as the
 * view/group chips and sort select right next to it. */
function wireAddCardInline(deck) {
  const wrap = document.getElementById("add-card-inline");
  const toggle = document.getElementById("add-card-toggle");
  const input = document.getElementById("add-card-input");
  const resultsEl = document.getElementById("add-card-results");

  toggle.addEventListener("click", () => {
    wrap.classList.add("expanded");
    input.focus();
  });
  input.addEventListener("keydown", (e) => {
    if (e.key !== "Escape") return;
    input.value = "";
    resultsEl.innerHTML = "";
    wrap.classList.remove("expanded");
    input.blur();
  });

  let debounce;
  input.addEventListener("input", () => {
    clearTimeout(debounce);
    const q = input.value.trim();
    if (q.length < 2) { resultsEl.innerHTML = ""; return; }
    debounce = setTimeout(async () => {
      const results = await api.searchCards(q, 8);
      // Two different cards can share the exact same printed name (e.g. "Phyrexian Hydra" the
      // 5-mana creature vs. the token it makes) — each shows up as its own row here already
      // (search returns every distinct oracle_id), so the type line is shown as a subtitle and
      // the oracle_id is carried through the click so the right one actually gets added.
      resultsEl.innerHTML = results
        .map(
          (c) => h`
          <div class="card-row" data-add="${c.name}" data-add-oracle="${c.oracle_id || ""}" style="cursor:pointer">
            <span class="name">${c.name}${c.type_line ? `<span class="card-row-sub">${c.type_line}</span>` : ""}</span>
            <span class="cost">${manaCostHtml(c.mana_cost)}</span>
          </div>`
        )
        .join("");
      resultsEl.querySelectorAll("[data-add]").forEach((el) =>
        el.addEventListener("click", async () => {
          try {
            const added = await addCardToDeckWithConfirm(deck.id, el.dataset.add, el.dataset.addOracle || null);
            if (!added) return;
            input.value = "";
            resultsEl.innerHTML = "";
            renderDeckDetail([String(deck.id)]);
          } catch (err) {
            alert(`Falhou ao adicionar: ${err.message}`);
          }
        })
      );
    }, 250);
  });
}

/** Re-renders the toolbar (to reflect the new active view/group-by chip) plus the card list below it. */
function refreshCardsToolbar(deck) {
  const tagsAvailable = Object.keys(currentDeckTags).length > 0;
  document.getElementById("cards-toolbar").outerHTML = viewControlsHtml(tagsAvailable);
  wireCardsToolbar(deck);
  renderDeckCards(deck);
}

function wireDeckFilterBar(deck) {
  document.getElementById("filter-toggle-btn").addEventListener("click", (e) => {
    e.stopPropagation();
    const dropdown = document.getElementById("filter-dropdown");
    const willOpen = !dropdown.classList.contains("open");
    dropdown.classList.toggle("open", willOpen);
    e.currentTarget.setAttribute("aria-expanded", String(willOpen));
  });

  const qInput = document.getElementById("deck-filter-q");
  let debounce;
  qInput.addEventListener("input", () => {
    clearTimeout(debounce);
    debounce = setTimeout(() => {
      deckFilters.q = qInput.value.trim();
      renderDeckCards(deck);
    }, 200);
  });

  document.querySelectorAll("[data-deck-type]").forEach((chip) =>
    chip.addEventListener("click", () => {
      const t = chip.dataset.deckType;
      deckFilters.types.has(t) ? deckFilters.types.delete(t) : deckFilters.types.add(t);
      refreshDeckFilterBar(deck);
    })
  );
  document.querySelectorAll("[data-deck-color]").forEach((chip) =>
    chip.addEventListener("click", () => {
      const c = chip.dataset.deckColor;
      deckFilters.colors.has(c) ? deckFilters.colors.delete(c) : deckFilters.colors.add(c);
      refreshDeckFilterBar(deck);
    })
  );
  document.querySelectorAll("[data-deck-cmc]").forEach((chip) =>
    chip.addEventListener("click", () => {
      const v = chip.dataset.deckCmc;
      deckFilters.cmcs.has(v) ? deckFilters.cmcs.delete(v) : deckFilters.cmcs.add(v);
      refreshDeckFilterBar(deck);
    })
  );
  const clearBtn = document.getElementById("deck-filter-clear");
  if (clearBtn) {
    clearBtn.addEventListener("click", () => {
      deckFilters = { q: "", types: new Set(), colors: new Set(), cmcs: new Set(), sort: deckFilters.sort, groupBy: deckFilters.groupBy };
      refreshDeckFilterBar(deck);
    });
  }
}

/** Re-renders the toggle button (for the active-filter badge) and the menu body (for the new
 * chip states) — but NOT the #filter-dropdown wrapper itself, so its "open" class (and thus
 * whether the menu is actually showing) survives toggling a chip inside it. */
function refreshDeckFilterBar(deck) {
  document.getElementById("filter-toggle-btn").outerHTML = filterToggleBtnHtml();
  document.getElementById("filter-menu").outerHTML = filterMenuContentHtml();
  wireDeckFilterBar(deck);
  renderDeckCards(deck);
}

function buildOwnershipMap(collectionAll) {
  const map = {};
  for (const c of collectionAll) map[c.card_name.toLowerCase()] = c;
  return map;
}

function ownershipTag(cardName, ownershipMap) {
  const entry = ownershipMap[cardName.toLowerCase()];
  if (!entry) return { label: "Missing", cls: "tag-missing" };
  const isFree = entry.decks.some((d) => d.deck_name === "Livre");
  if (isFree) return { label: "Available", cls: "tag-available" };
  const otherDeck = entry.decks.find((d) => d.deck_name !== "Livre");
  return { label: otherDeck ? otherDeck.deck_name : "Available", cls: "tag-other-deck" };
}

// Order colors are stacked in each curve bar (bottom to top) and shown in the legend.
// "M" = multicolor cards (2+ colors), grouped together rather than split, since MTG deck tools
// conventionally show multicolor as its own gold segment instead of dividing it between colors.
const CURVE_COLOR_ORDER = ["W", "U", "B", "R", "G", "M", "C"];
// Its own palette rather than FILTER_COLORS (the mana-pip pastels, tuned for a black glyph drawn
// on top). Every value here clears WCAG 1.4.11's 3:1 against the panel background (#18172e) —
// measured, not eyeballed; the worst case is 6.07:1.
//
// "B" is the one that isn't the literal mana colour: charcoal on a dark panel measured 1.74:1,
// which is why a mono-black deck's curve read as an undifferentiated grey block. It borrows the
// app's own violet instead, so black decks look like this app rather than like a rendering bug.
// U and G are pulled toward --accent and --accent-bright for the same reason.
const CURVE_COLORS = { W: "#efe6bb", U: "#6aa9f0", B: "#9b8fd6", R: "#ef8f74", G: "#5fcf98", M: "#d9b45c", C: "#b9b5d4" };

/** Buckets every non-commander, non-land card in the deck by CMC, then by color (W/U/B/R/G,
 * "M" for multicolor, "C" for colorless) — mirrors the backend's mana_curve totals but split
 * out per color so the sidebar chart can render a stacked column per mana value. */
function manaCurveByColorData(deck) {
  const buckets = {};
  Object.entries(deck.by_type || {}).forEach(([cat, cards]) => {
    if (cat === "Comandante") return;
    cards.forEach((c) => {
      if ((c.type_line || "").includes("Land")) return;
      const cmc = Math.trunc(c.cmc || 0);
      const letters = (c.colors || "").split("").filter(Boolean);
      const key = letters.length === 0 ? "C" : letters.length === 1 ? letters[0] : "M";
      buckets[cmc] = buckets[cmc] || {};
      buckets[cmc][key] = (buckets[cmc][key] || 0) + c.quantity;
    });
  });
  return buckets;
}

function curveBarSegmentsHtml(byColor, maxCount) {
  return CURVE_COLOR_ORDER.filter((c) => byColor[c])
    .map((c) => {
      const heightPx = Math.max(2, (byColor[c] / maxCount) * 90);
      return `<div class="curve-bar-seg" style="height:${heightPx}px;background:${CURVE_COLORS[c]}" title="${byColor[c]} ${c === "M" ? "multicolor" : c === "C" ? "incolor" : c}"></div>`;
    })
    .join("");
}

/** Adds a card to the deck. Commander is singleton, so the backend rejects a second copy of
 * anything but a basic land (or another explicitly-unlimited card) with 409 + needs_confirmation
 * — this catches that, asks the user to confirm, and resubmits with confirm:true if they agree.
 * Returns false (no throw) if the user declined, so callers can just bail out quietly. */
async function addCardToDeckWithConfirm(deckId, cardName, oracleId = null) {
  try {
    await api.addDeckCard(deckId, cardName, 1, oracleId);
    return true;
  } catch (err) {
    if (err.status === 409 && err.body?.detail?.needs_confirmation) {
      const qty = err.body.detail.existing_quantity;
      if (!confirm(`"${cardName}" já está no deck (${qty}x). Adicionar mais uma cópia mesmo assim?`)) {
        return false;
      }
      await api.addDeckCard(deckId, cardName, 1, oracleId, true);
      return true;
    }
    throw err;
  }
}

export async function renderDeckDetail([idStr]) {
  const id = Number(idStr);
  const [deck, synergy, collectionAll, tagsResp] = await Promise.all([
    api.deck(id),
    api.deckSynergy(id).catch(() => ({ cached: false })),
    api.collection("all"),
    api.deckTags(id).catch(() => ({ cached: false, tags: {} })),
  ]);
  const ownershipMap = buildOwnershipMap(collectionAll);
  currentDeckTags = tagsResp.tags || {};
  const tagsAvailable = Object.keys(currentDeckTags).length > 0;

  const maxCmc = Math.max(1, ...Object.keys(deck.mana_curve).map(Number));
  const maxCount = Math.max(1, ...Object.values(deck.mana_curve));
  const curveByColor = manaCurveByColorData(deck);
  const curveBars = Array.from({ length: Math.min(maxCmc, 7) + 1 }, (_, i) => i)
    .map((cmc) => {
      const label = cmc === 7 ? "7+" : String(cmc);
      const count = cmc === 7
        ? Object.entries(deck.mana_curve).filter(([k]) => Number(k) >= 7).reduce((s, [, v]) => s + v, 0)
        : (deck.mana_curve[cmc] || 0);
      const byColor = cmc === 7
        ? Object.entries(curveByColor).filter(([k]) => Number(k) >= 7).reduce((acc, [, colors]) => {
            Object.entries(colors).forEach(([c, n]) => { acc[c] = (acc[c] || 0) + n; });
            return acc;
          }, {})
        : curveByColor[cmc] || {};
      return { label, count, byColor };
    })
    // Most decks have no 0-cost spells (Ornithopter, Mishra's Bauble, etc. are the exception),
    // so an always-empty leading column just wastes space — drop it unless it's actually used.
    .filter((b) => b.label !== "0" || b.count > 0);

  // A banner that only says "everything is fine" is a banner the eye learns to skip, which is
  // exactly what you don't want the day it turns into a real warning. The 100/100 pill above
  // already states a complete deck, so the banner is reserved for the two states that need an
  // action: over the limit, and short of it.
  const overage = deck.total_cards - 100;
  let overageWarning = "";
  if (overage > 0) {
    overageWarning = `<div class="overage-warning bad">Deck com ${overage} carta${overage > 1 ? "s" : ""} além do limite — remova ${overage} antes de continuar.</div>`;
  } else if (overage < 0) {
    const missing = -overage;
    overageWarning = `<div class="overage-warning">Faltam ${missing} carta${missing > 1 ? "s" : ""} para fechar os 100. Use <b>Adicionar card</b>, <b>Importar decklist</b>, ou veja as sugestões de sinergia acima.</div>`;
  }

  mainEl.innerHTML = h`
    <div class="breadcrumb"><a href="#decks" data-nav="decks">Meus Decks</a><span class="sep">/</span>${deck.name}</div>
    <div class="page-header">
      <div>
        <div style="display:flex;align-items:center;gap:10px;flex-wrap:wrap">
          <h1>${deck.name}</h1>
          ${deckTagsHtml(deck.tags)}
        </div>
        <p>${deck.philosophy || ""}</p>
      </div>
      <div style="display:flex;align-items:center;gap:10px;flex-wrap:wrap">
        <span class="count-pill ${deck.is_valid_100 ? "ok" : "bad"}" style="font-size:15px">${deck.total_cards}/100</span>
        <div class="export-dropdown" id="export-dropdown">
          <button class="btn secondary small" id="export-toggle-btn" aria-haspopup="true" aria-expanded="false">Exportar ▾</button>
          <div class="export-menu" id="export-menu">
            <a href="/api/decks/${id}/export?format=moxfield" download>Formato Moxfield (.txt)</a>
            <a href="/api/decks/${id}/export?format=text" download>Texto simples (.txt)</a>
          </div>
        </div>
        <button class="btn secondary small" id="import-deck-btn">Importar decklist</button>
        <button class="btn icon-btn secondary small" id="edit-deck-btn" title="Editar deck" aria-label="Editar deck">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z"/><path d="m15 5 4 4"/></svg>
        </button>
        <button class="btn icon-btn danger small" id="delete-deck-btn" title="Excluir deck" aria-label="Excluir deck">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 6h18"/><path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/><path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6"/><path d="M10 11v6"/><path d="M14 11v6"/></svg>
        </button>
      </div>
    </div>
    ${overageWarning}
    ${ownershipSummaryHtml(deck)}
    <div class="deck-stats-bar">
      ${commanderPanelHtml(deck)}
      <div class="stats-bar-col">
        ${synergyPanelHtml(synergy, ownershipMap)}
        ${similarCommandersPanelHtml(synergy)}
      </div>
      <div class="stats-bar-panel curve-panel">
        <h3>Curva de mana</h3>
        <div class="curve-bars">
          ${curveBars.map((b) => `<div class="curve-bar" title="${b.count} cartas">${curveBarSegmentsHtml(b.byColor, maxCount)}</div>`).join("")}
        </div>
        <div class="curve-labels">${curveBars.map((b) => `<span>${b.label}</span>`).join("")}</div>
        <div class="curve-legend">
          ${CURVE_COLOR_ORDER.filter((c) => curveBars.some((b) => b.byColor[c])).map((c) => `<span class="curve-legend-item"><span class="dot" style="background:${CURVE_COLORS[c]}"></span>${c === "M" ? "Multi" : c === "C" ? "Incolor" : c}</span>`).join("")}
        </div>
      </div>
    </div>
    ${viewControlsHtml(tagsAvailable)}
    <div id="deck-cards"></div>
  `;

  document.querySelector('[data-nav="decks"]').addEventListener("click", (e) => {
    e.preventDefault();
    location.hash = "#decks";
  });

  document.getElementById("delete-deck-btn").addEventListener("click", () =>
    openDeleteDeckModal(deck, id)
  );

  document.getElementById("own-toggle-missing")?.addEventListener("click", (e) => {
    const list = document.getElementById("own-missing-list");
    const open = list.style.display !== "none";
    list.style.display = open ? "none" : "block";
    e.target.textContent = open ? "ver lista" : "ocultar";
  });

  document.getElementById("edit-deck-btn").addEventListener("click", () =>
    openEditDeckModal(deck, () => renderDeckDetail([idStr]))
  );
  document.getElementById("import-deck-btn").addEventListener("click", () =>
    openImportDeckModal(id, () => renderDeckDetail([idStr]))
  );

  // Export dropdown — two plain <a download> links (Moxfield-format / plain-text) straight to
  // the export endpoint, so the browser just downloads the file with no extra JS plumbing.
  // Only the open/close toggle needs wiring here; the outside-click-closes behavior is a single
  // delegated listener registered once at module scope (see bottom of file), not re-added on
  // every render.
  document.getElementById("export-toggle-btn").addEventListener("click", (e) => {
    e.stopPropagation();
    const dropdown = document.getElementById("export-dropdown");
    const willOpen = !dropdown.classList.contains("open");
    dropdown.classList.toggle("open", willOpen);
    e.currentTarget.setAttribute("aria-expanded", String(willOpen));
  });
  document.getElementById("export-menu").addEventListener("click", () => {
    document.getElementById("export-dropdown")?.classList.remove("open");
  });

  const fetchSynergyBtn = document.getElementById("fetch-synergy-btn");
  if (fetchSynergyBtn) {
    fetchSynergyBtn.addEventListener("click", async () => {
      const statusEl = document.getElementById("fetch-synergy-status");
      fetchSynergyBtn.disabled = true;
      fetchSynergyBtn.textContent = "Buscando…";
      try {
        await api.fetchDeckSynergy(id);
        renderDeckDetail([idStr]);
      } catch (err) {
        fetchSynergyBtn.disabled = false;
        fetchSynergyBtn.textContent = "Buscar sinergia agora";
        statusEl.textContent = `Falhou: ${err.message}`;
      }
    });
  }

  // Synergy suggestions can be added straight to the deck — no need to retype the name in "Adicionar carta".
  document.querySelectorAll("[data-add-synergy]").forEach((btn) =>
    btn.addEventListener("click", async (e) => {
      e.stopPropagation();
      btn.disabled = true;
      btn.textContent = "…";
      try {
        const added = await addCardToDeckWithConfirm(id, btn.dataset.addSynergy);
        if (added) { renderDeckDetail([idStr]); return; }
        btn.disabled = false;
        btn.textContent = "+";
      } catch (err) {
        console.error(err);
        btn.disabled = false;
        btn.textContent = "✕";
      }
    })
  );

  // Synergy suggestion names weren't wired to the card preview modal (that wiring is scoped
  // to the deck card list below) — the sidebar panel needs its own click handler.
  document.querySelectorAll(".synergy-item [data-card-view]").forEach((el) =>
    el.addEventListener("click", () => showCardModal(el.dataset.cardView))
  );
  // Same deal for the commander panel, now that it lives outside the main card list.
  document.querySelectorAll(".commander-card[data-card-view]").forEach((el) =>
    el.addEventListener("click", () => showCardModal(el.dataset.cardView))
  );
  // Remembers whether the synergy panel was expanded, so re-rendering the page (e.g. after
  // adding a card) doesn't snap it shut again if the user had it open.
  document.getElementById("synergy-details")?.addEventListener("toggle", (e) => {
    synergyPanelOpen = e.target.open;
  });

  wireCardsToolbar(deck);
  renderDeckCards(deck);
}

/** Sum of quantities across a group's cards — the count that belongs next to a group header.
 * NOT cards.length: a group of "31x Swamp" is one distinct deck_cards row but 31 actual cards,
 * and the header should say 31, not 1. */
function groupQuantity(cards) {
  return cards.reduce((s, c) => s + c.quantity, 0);
}

// Banner above the card list summarising what this deck would cost you to actually build:
// how many cards you don't own (the shopping list) and how many you'd have to pull out of
// another deck. Silent when the deck is fully owned and free — nothing to warn about.
function ownershipSummaryHtml(deck) {
  const o = deck.ownership;
  if (!o) return "";
  const missing = Object.entries(o).filter(([, v]) => v.status === "missing").map(([n]) => n);
  const borrowed = Object.entries(o).filter(([, v]) => v.status === "owned_in_deck");
  if (!missing.length && !borrowed.length) return "";

  const byDeck = {};
  for (const [name, v] of borrowed) (byDeck[v.deck] ||= []).push(name);

  return h`
    <div class="ownership-summary">
      ${missing.length ? `
        <div class="own-line">
          <span class="own-tag tag-missing">${missing.length}</span>
          <span>carta(s) que você <b>não tem</b> na coleção
            <button class="linklike" id="own-toggle-missing">ver lista</button>
          </span>
        </div>
        <div id="own-missing-list" class="own-list" style="display:none">${missing.sort().join(" · ")}</div>` : ""}
      ${borrowed.length ? `
        <div class="own-line">
          <span class="own-tag tag-other-deck">${borrowed.length}</span>
          <span>carta(s) sem cópia própria neste deck — a sua está em: <b>${Object.keys(byDeck).join(", ")}</b>, usar aqui significa desmontar</span>
        </div>` : ""}
    </div>`;
}

// Per-card ownership, as computed by the backend (deck.ownership). Returns null for cards whose
// status the backend didn't report (e.g. the commander, or an older backend that predates the
// field) so nothing is flagged on a guess.
function deckOwnTag(cardName, ownership) {
  const o = ownership?.[cardName];
  if (!o) return null;
  if (o.status === "missing") {
    return { label: "Não tenho", cls: "tag-missing", title: "Esta carta não está na sua coleção — precisa comprar." };
  }
  if (o.status === "owned_in_deck") {
    const where = (o.decks || [{ deck: o.deck, copies: 1 }])
      .map((d) => `${d.copies}x em "${d.deck}"`)
      .join(", ");
    return {
      label: o.deck || "Em outro deck",
      cls: "tag-other-deck",
      title: `Este deck não tem cópia própria desta carta. Suas cópias: ${where} — usá-la aqui significa desmontar aquele deck.`,
    };
  }
  // owned_here (this deck has its own copy) and owned_free both mean nothing to resolve.
  return null;
}

function ownTagHtml(cardName, ownership) {
  const t = deckOwnTag(cardName, ownership);
  return t ? `<span class="own-tag ${t.cls}" title="${t.title}">${t.label}</span>` : "";
}

// Small corner dot for the image-based views, where a text pill wouldn't fit over the art.
function ownDotHtml(cardName, ownership) {
  const t = deckOwnTag(cardName, ownership);
  return t ? `<span class="own-dot ${t.cls}" title="${t.title}"></span>` : "";
}

// Cursor-following preview for the stacked view. Lives at the document level (one node, reused)
// rather than one per card: the stack renders a hundred of them, and a preview per card meant a
// hundred hidden images the browser would still decode.
//
// Positioned to the right of the cursor, flipping to the left only when it would run past the
// window edge — so it never covers what you are about to click, which was the whole complaint
// about the previous hover-to-lift behaviour.
function cardPeek() {
  let el = document.getElementById("card-peek");
  if (!el) {
    el = document.createElement("div");
    el.id = "card-peek";
    document.body.appendChild(el);
  }
  return el;
}

function attachStackPeek(container) {
  const GAP = 18;
  const el = cardPeek();
  let current = null;

  const place = (e) => {
    const w = el.offsetWidth || 240;
    const h = el.offsetHeight || 336;
    const flip = e.clientX + GAP + w > window.innerWidth;
    el.style.left = `${flip ? e.clientX - GAP - w : e.clientX + GAP}px`;
    // Keep the whole preview on screen vertically as well, near the bottom of a long stack.
    el.style.top = `${Math.max(8, Math.min(e.clientY - h / 2, window.innerHeight - h - 8))}px`;
  };

  container.addEventListener("mouseover", (e) => {
    const item = e.target.closest(".stack-item");
    if (!item || item === current) return;
    const img = item.querySelector("img");
    if (!img) return;
    current = item;
    el.innerHTML = `<img src="${img.src}" alt="">`;
    el.classList.add("visible");
    place(e);
  });
  container.addEventListener("mousemove", (e) => {
    if (current) place(e);
  });
  container.addEventListener("mouseout", (e) => {
    if (!e.relatedTarget || !e.relatedTarget.closest?.(".stack-item")) {
      current = null;
      el.classList.remove("visible");
    }
  });
}

function renderDeckCards(deck) {
  const cardsWrap = document.getElementById("deck-cards");
  const ownership = deck.ownership || null;
  // Comandante has its own highlighted panel in the top stats bar (see commanderPanelHtml) so
  // every group-by mode excludes it from the regular card list here — see computeDeckGroups.
  const { groups, groupKeys, groupLabel } = computeDeckGroups(deck);

  if (!groupKeys.length) {
    cardsWrap.innerHTML = `<div class="empty-state">Nenhuma carta corresponde aos filtros atuais.</div>`;
    return;
  }

  if (deckViewMode === "grid") {
    cardsWrap.innerHTML = groupKeys
      .map((key) => {
        const cards = groups[key];
        const tiles = cards
          .map((c) => h`
            <div class="mtg-card" data-card-view="${c.card_name}">
              ${c.image_uri ? `<img src="${c.image_uri}" loading="lazy" alt="${c.card_name}">` : `<div class="no-image">${c.card_name}</div>`}
              <span class="qty-badge">${c.quantity}x</span>
              ${ownDotHtml(c.card_name, ownership)}
              <button class="btn small secondary tile-remove" data-remove="${c.id}" data-remove-name="${c.card_name}">✕</button>
            </div>`)
          .join("");
        return `<div class="category-block"><h4>${groupLabel(key)} <span class="n">(${groupQuantity(cards)})</span></h4><div class="card-grid">${tiles}</div></div>`;
      })
      .join("");
  } else if (deckViewMode === "stack") {
    // Columns side by side (not one category block under another) so the whole deck
    // fits in a single scroll instead of a long vertical chain of separate stacks.
    cardsWrap.innerHTML = `<div class="stack-columns">${groupKeys
      .map((key) => {
        const cards = groups[key];
        const items = cards
          .map((c) => h`
            <div class="stack-item" data-card-view="${c.card_name}">
              ${c.image_uri ? `<img src="${c.image_uri}" loading="lazy" alt="${c.card_name}">` : `<div class="no-image">${c.card_name}</div>`}
              ${ownDotHtml(c.card_name, ownership)}
              <button class="btn small secondary tile-remove" data-remove="${c.id}" data-remove-name="${c.card_name}">✕</button>
            </div>`)
          .join("");
        return `<div class="category-block"><h4>${groupLabel(key)} <span class="n">(${groupQuantity(cards)})</span></h4><div class="card-stack">${items}</div></div>`;
      })
      .join("")}</div>`;
  } else {
    cardsWrap.innerHTML = groupKeys
      .map((key) => {
        const cards = groups[key];
        const rows = cards
          .map((c) => {
            // "também em X" used to appear whenever another deck listed the same card name — which
            // says nothing useful, because owning two copies and lending one out look identical
            // from a name match. What matters is whether THIS deck has its own copy. When it
            // does, the card is simply here and the tag is noise, so it is dropped; when it
            // doesn't, ownTagHtml already says where the single copy actually is. The remaining
            // job of this tag is the genuinely informative middle case: you own a copy here AND
            // other decks run their own, which is worth knowing before you pull one apart.
            const own = ownership?.[c.card_name];
            const sharedList = (c.shared_with || []).map((s) => s.deck).join(", ");
            const sharedTag =
              c.shared_with?.length && own?.status === "owned_here"
                ? `<span class="shared-tag" title="Você tem cópias próprias aqui e em: ${sharedList}. Nada precisa ser desmontado.">cópia própria · também em ${sharedList}</span>`
                : "";
            return h`
              <div class="card-row">
                <span class="qty">${c.quantity}x</span>
                <span class="name" data-card-view="${c.card_name}" style="cursor:pointer">${c.card_name}</span>
                ${ownTagHtml(c.card_name, ownership)}
                ${sharedTag}
                <span class="cost">${manaCostHtml(c.mana_cost)}</span>
                <button class="btn small secondary" data-remove="${c.id}" data-remove-name="${c.card_name}">✕</button>
              </div>`;
          })
          .join("");
        return `<div class="category-block"><h4>${groupLabel(key)} <span class="n">(${groupQuantity(cards)})</span></h4>${rows}</div>`;
      })
      .join("");
  }

  if (deckViewMode === "stack") attachStackPeek(cardsWrap);

  cardsWrap.querySelectorAll("[data-remove]").forEach((btn) =>
    btn.addEventListener("click", async (e) => {
      e.stopPropagation();
      const cardName = btn.dataset.removeName || "esta carta";
      if (!confirm(`Remover ${cardName} do deck?`)) return;
      await api.removeDeckCard(deck.id, Number(btn.dataset.remove));
      renderDeckDetail([String(deck.id)]);
    })
  );
  cardsWrap.querySelectorAll("[data-card-view]").forEach((el) =>
    el.addEventListener("click", () => showCardModal(el.dataset.cardView))
  );
}

/** Pulled out of the main card list into its own highlighted panel in the top stats bar, next to
 * the mana curve, so the commander (or commander pair, for partners) stands out from the rest of
 * the deck while browsing instead of blending into the first category block. */
function commanderPanelHtml(deck) {
  const commanders = deck.by_type["Comandante"] || [];
  if (!commanders.length) return "";
  // Deck tags show next to the deck name in the page header on this page (see renderDeckDetail)
  // — only the "Meus Decks" thumbnails overlay them on the art itself.
  const cards = commanders
    .map((c) => h`
      <div class="commander-card" data-card-view="${c.card_name}">
        ${c.image_uri ? `<img src="${c.image_uri}" alt="${c.card_name}">` : `<div class="no-image">${c.card_name}</div>`}
        <div class="commander-info">
          <div class="commander-name">${c.card_name}</div>
          <div class="commander-type">${c.type_line || ""}</div>
          <div class="commander-cost">${manaCostHtml(c.mana_cost)}</div>
        </div>
      </div>`)
    .join("");
  return h`
    <div class="stats-bar-panel commander-panel">
      <div class="commander-list">${cards}</div>
    </div>`;
}

function synergyPanelHtml(synergy, ownershipMap) {
  if (!synergy.cached) {
    return `
      <div class="stats-bar-panel">
        <h3>Sinergia (EDHREC)</h3>
        <div class="empty-state" style="padding:20px 10px 10px">Sem cache do EDHREC para este comandante ainda.</div>
        <button class="btn" id="fetch-synergy-btn" style="width:100%">Buscar sinergia agora</button>
        <div id="fetch-synergy-status" style="margin-top:8px;font-size:12px;color:var(--text-dim)"></div>
      </div>`;
  }
  const recs = (synergy.recommendations || []).slice(0, 8)
    .map((r) => {
      const tag = ownershipTag(r.name, ownershipMap);
      return h`
        <div class="synergy-item">
          <div class="name" data-card-view="${r.name}" style="cursor:pointer">${r.name}</div>
          <div class="meta">
            sinergia ${r.synergy >= 0 ? "+" : ""}${r.synergy?.toFixed(2)} · ${r.num_decks?.toLocaleString("pt-BR")} decks
            <span class="own-tag ${tag.cls}">${tag.label}</span>
            <button class="btn small secondary synergy-add-btn" data-add-synergy="${r.name}" title="Adicionar ao deck" style="margin-left:auto">+</button>
          </div>
        </div>`;
    })
    .join("");
  // Collapsed by default (see synergyPanelOpen) — useful when you want it, but not something
  // you need staring at you every time you open a deck, so <details>/<summary> keeps it one
  // click away instead of always taking up space in the top stats bar.
  return h`
    <details class="stats-bar-panel synergy-panel" id="synergy-details" ${synergyPanelOpen ? "open" : ""}>
      <summary>Cards relacionados</summary>
      <div class="synergy-body">
        ${recs || '<div class="empty-state" style="padding:10px">Nada fora do deck nas categorias de alta sinergia.</div>'}
      </div>
    </details>`;
}

/** "Comandantes parecidos" — split out of the synergy panel above and rendered as its own panel
 * right underneath it (see the .stats-bar-col wrapper in renderDeckDetail), since it's a
 * different kind of suggestion (whole other commanders, not cards to add to this deck). */
function similarCommandersPanelHtml(synergy) {
  if (!synergy.cached) return "";
  const similar = (synergy.similar_commanders || []).slice(0, 5).map((s) => `<span class="chip">${s}</span>`).join(" ");
  if (!similar) return "";
  return h`
    <div class="stats-bar-panel">
      <h3>Comandantes parecidos</h3>
      <div style="display:flex;flex-wrap:wrap;gap:6px">${similar}</div>
    </div>`;
}

// ------------------------------------------------------------ collection ----
