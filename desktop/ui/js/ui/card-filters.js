// Card filtering shared by the deck page and the collection.
//
// Both screens filter the same thing — a list of cards enriched from the Scryfall index — so the
// predicates, the chip markup and the dropdown wiring live here once. Each screen owns its own
// state object and decides what to do when it changes; this module never re-renders anything.
//
// State is mutated in place rather than replaced, including on "clear", so a caller holding a
// reference (and every already-bound handler) keeps seeing the live object.

import { manaGlyphSvg } from "../icons.js?v=25";
import { FILTER_COLORS } from "../deck-bits.js?v=3";
import { h } from "../util.js?v=3";

export const CATEGORY_LABELS = {
  Comandante: "Comandante", Land: "Terrenos", Creature: "Criaturas",
  Instant: "Instantâneas", Sorcery: "Feitiços", Artifact: "Artefatos",
  Enchantment: "Encantamentos", Planeswalker: "Planeswalker", Outro: "Outro",
};
export const CATEGORY_ORDER = ["Comandante", "Creature", "Instant", "Sorcery", "Artifact", "Enchantment", "Planeswalker", "Outro", "Land"];

// Short labels just for the type chips — the full CATEGORY_LABELS names run long enough that six
// of them plus the colour and CMC rows push the menu to several lines.
export const FILTER_TYPE_LABELS = {
  Land: "Terreno", Creature: "Criatura", Instant: "Inst.", Sorcery: "Feit.",
  Artifact: "Art.", Enchantment: "Enc.", Planeswalker: "PW", Outro: "Outro",
};
// The commander is always shown regardless of filters — losing sight of your own commander
// behind a filter is confusing — so it isn't offered as a chip.
export const FILTERABLE_TYPES = CATEGORY_ORDER.filter((c) => c !== "Comandante");
export const CMC_BUCKETS = ["0", "1", "2", "3", "4", "5", "6+"];
export const RARITIES = ["common", "uncommon", "rare", "mythic"];
const RARITY_LABELS = { common: "Comum", uncommon: "Incomum", rare: "Rara", mythic: "Mítica" };

export function newFilterState(extra = {}) {
  return { q: "", types: new Set(), colors: new Set(), cmcs: new Set(), rarities: new Set(), ...extra };
}

export function filtersActive(f) {
  return !!(f.q || f.types.size || f.colors.size || f.cmcs.size || f.rarities?.size);
}

export function cardCmcBucket(cmc) {
  const n = Math.floor(cmc || 0);
  return n >= 6 ? "6+" : String(n);
}

export function cardCategory(c) {
  const t = c.type_line || "";
  for (const cat of ["Land", "Creature", "Planeswalker", "Battle", "Artifact", "Enchantment", "Instant", "Sorcery"]) {
    if (t.includes(cat)) return cat;
  }
  return "Outro";
}

export function matchesFilters(c, f) {
  const name = c.card_name || c.name || "";
  if (f.q && !name.toLowerCase().includes(f.q.toLowerCase())) return false;
  if (f.types.size && !f.types.has(cardCategory(c))) return false;
  if (f.colors.size) {
    const letters = c.colors ? c.colors.split("") : [];
    const isColorless = letters.length === 0;
    const hit = (isColorless && f.colors.has("C")) || letters.some((l) => f.colors.has(l));
    if (!hit) return false;
  }
  if (f.cmcs.size && !f.cmcs.has(cardCmcBucket(c.cmc))) return false;
  if (f.rarities?.size && !f.rarities.has((c.rarity || "").toLowerCase())) return false;
  return true;
}

export function filterToggleBtnHtml(f) {
  return h`
    <button type="button" class="filter-toggle-btn" id="filter-toggle-btn" title="Filtro" aria-label="Filtro" aria-haspopup="true" aria-expanded="false">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 3H2l8 9.46V19l4 2v-8.54Z"/></svg>
      ${filtersActive(f) ? `<span class="filter-badge"></span>` : ""}
    </button>`;
}

/** `opts.rarity` adds the rarity row; `opts.placeholder` labels the search box for the screen. */
export function filterMenuContentHtml(f, opts = {}) {
  const typeChips = FILTERABLE_TYPES.map(
    (t) => `<span class="chip type-chip ${f.types.has(t) ? "active" : ""}" data-deck-type="${t}" title="${CATEGORY_LABELS[t]}">${FILTER_TYPE_LABELS[t]}</span>`
  ).join("");
  const colorChips = Object.entries(FILTER_COLORS)
    .map(([c, bg]) => {
      const dimmed = f.colors.size && !f.colors.has(c);
      const glyph = manaGlyphSvg(c);
      return `<span class="chip color-chip ${f.colors.has(c) ? "active" : ""}" data-deck-color="${c}"
        style="background:${bg};color:#1a1a1a;opacity:${dimmed ? 0.35 : 1}">${glyph ? `<span class="mana-sym-inline">${glyph}</span>` : c}</span>`;
    })
    .join("");
  const cmcChips = CMC_BUCKETS.map(
    (v) => `<span class="chip ${f.cmcs.has(v) ? "active" : ""}" data-deck-cmc="${v}">${v}</span>`
  ).join("");
  const rarityChips = RARITIES.map(
    (r) => `<span class="chip ${f.rarities?.has(r) ? "active" : ""}" data-deck-rarity="${r}">${RARITY_LABELS[r]}</span>`
  ).join("");

  return h`
    <div class="filter-menu" id="filter-menu">
      ${opts.search === false ? "" : `
      <div class="filter-group filter-group-block">
        <span class="filter-group-label">Buscar</span>
        <input type="text" id="deck-filter-q" placeholder="${opts.placeholder || "Nome da carta…"}" value="${f.q}">
      </div>`}
      <div class="filter-group"><span class="filter-group-label">Tipo</span>${typeChips}</div>
      <div class="filter-group"><span class="filter-group-label">Cor</span>${colorChips}</div>
      <div class="filter-group"><span class="filter-group-label">CMC</span>${cmcChips}</div>
      ${opts.rarity ? `<div class="filter-group"><span class="filter-group-label">Raridade</span>${rarityChips}</div>` : ""}
      ${filtersActive(f) ? `<button class="btn small secondary" id="deck-filter-clear">Limpar filtros</button>` : ""}
    </div>`;
}

export function filterDropdownHtml(f, opts = {}) {
  return h`
    <div class="filter-dropdown" id="filter-dropdown">
      ${filterToggleBtnHtml(f)}
      ${filterMenuContentHtml(f, opts)}
    </div>`;
}

/**
 * Binds the menu currently in the DOM. `onChange(structural)` is called after every change:
 * `structural` is true when the chips themselves need redrawing (so the caller can re-render the
 * menu body), false for the debounced search box, which must keep its focus and caret.
 */
export function wireFilterMenu(f, onChange) {
  document.getElementById("filter-toggle-btn")?.addEventListener("click", (e) => {
    e.stopPropagation();
    const dropdown = document.getElementById("filter-dropdown");
    const willOpen = !dropdown.classList.contains("open");
    dropdown.classList.toggle("open", willOpen);
    e.currentTarget.setAttribute("aria-expanded", String(willOpen));
  });

  const qInput = document.getElementById("deck-filter-q");
  let debounce;
  qInput?.addEventListener("input", () => {
    clearTimeout(debounce);
    debounce = setTimeout(() => {
      f.q = qInput.value.trim();
      onChange(false);
    }, 200);
  });

  const toggle = (set, value) => {
    set.has(value) ? set.delete(value) : set.add(value);
    onChange(true);
  };
  document.querySelectorAll("[data-deck-type]").forEach((chip) =>
    chip.addEventListener("click", () => toggle(f.types, chip.dataset.deckType))
  );
  document.querySelectorAll("[data-deck-color]").forEach((chip) =>
    chip.addEventListener("click", () => toggle(f.colors, chip.dataset.deckColor))
  );
  document.querySelectorAll("[data-deck-cmc]").forEach((chip) =>
    chip.addEventListener("click", () => toggle(f.cmcs, chip.dataset.deckCmc))
  );
  document.querySelectorAll("[data-deck-rarity]").forEach((chip) =>
    chip.addEventListener("click", () => toggle(f.rarities, chip.dataset.deckRarity))
  );

  document.getElementById("deck-filter-clear")?.addEventListener("click", () => {
    // Cleared in place: callers (and the deck page's sort/group-by fields living on the same
    // object) hold a reference to this state, and replacing it would strand them on the old one.
    f.q = "";
    f.types.clear();
    f.colors.clear();
    f.cmcs.clear();
    f.rarities?.clear();
    onChange(true);
  });
}
