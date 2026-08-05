// Ícones SVG minimalistas (estilo line-icon), nunca emoji. currentColor herda a cor do texto.
const ICONS = {
  card_new: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="9"/><path d="M12 8v8M8 12h8"/></svg>`,
  card_added_deck: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M15 3h4a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-4"/><polyline points="10 17 5 12 10 7"/><path d="M5 12h12"/></svg>`,
  card_removed_deck: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M9 3H5a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h4"/><polyline points="14 7 19 12 14 17"/><path d="M19 12H7"/></svg>`,
  deck_built: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polygon points="12 2 2 7 12 12 22 7 12 2"/><polyline points="2 17 12 22 22 17"/><polyline points="2 12 12 17 22 12"/></svg>`,
  deck_disassembled: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"/><path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6"/><path d="M10 11v6M14 11v6"/></svg>`,
  default: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="9"/><path d="M12 7v5l3 3"/></svg>`,
};

export function activityIcon(type) {
  return ICONS[type] || ICONS.default;
}

// ---------------------------------------------------------- custo de mana ----

// Cores oficiais aproximadas dos símbolos de mana do Magic.
const MANA_COLORS = {
  W: "#fffbd5",
  U: "#aae0fa",
  B: "#cbc2bf",
  R: "#f9aa8f",
  G: "#9bd3ae",
  C: "#ccc",
};

function singleSymbolHtml(inner) {
  if (/^\d+$/.test(inner)) {
    return `<span class="mana-sym" style="background:#d8d3c9">${inner}</span>`;
  }
  if (inner === "X" || inner === "Y" || inner === "Z") {
    return `<span class="mana-sym" style="background:#d8d3c9">${inner}</span>`;
  }
  if (MANA_COLORS[inner]) {
    return `<span class="mana-sym" style="background:${MANA_COLORS[inner]}">${inner}</span>`;
  }
  // híbrido (ex: B/P, 2/B, W/U) — mostra os dois componentes bem pequenos
  if (inner.includes("/")) {
    const parts = inner.split("/");
    const bg = MANA_COLORS[parts[parts.length - 1]] || "#d8d3c9";
    return `<span class="mana-sym mana-sym-hybrid" style="background:${bg}">${parts.join("/")}</span>`;
  }
  return `<span class="mana-sym" style="background:#d8d3c9">${inner}</span>`;
}

/** Renderiza um custo de mana ("{3}{B}{B}") como símbolos redondos coloridos, estilo MTG real. */
export function manaCostHtml(cost) {
  if (!cost) return "";
  const symbols = cost.match(/\{[^}]+\}/g);
  if (!symbols) return "";
  return `<span class="mana-cost">${symbols.map((s) => singleSymbolHtml(s.slice(1, -1))).join("")}</span>`;
}
