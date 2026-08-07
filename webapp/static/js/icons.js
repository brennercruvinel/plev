// Minimalist SVG line-icons, never emoji. currentColor inherits the text color.
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

// ------------------------------------------------------------- mana cost ----

// Approximate official colors of Magic's mana symbols.
const MANA_COLORS = {
  W: "#fffbd5",
  U: "#aae0fa",
  B: "#cbc2bf",
  R: "#f9aa8f",
  G: "#9bd3ae",
  C: "#ccc",
};

// Flat black-silhouette glyphs (sun / droplet / skull / flame / tree), drawn to read
// clearly at 16px on their pastel color-pip backgrounds — mirrors the classic MTG pip set.
const GLYPH_FILL = "#1a1a1a";
function sunburstPoints(cx, cy, outerR, innerR, spikes) {
  const pts = [];
  for (let i = 0; i < spikes * 2; i++) {
    const r = i % 2 === 0 ? outerR : innerR;
    const angle = (Math.PI / spikes) * i - Math.PI / 2;
    pts.push(`${(cx + r * Math.cos(angle)).toFixed(2)},${(cy + r * Math.sin(angle)).toFixed(2)}`);
  }
  return pts.join(" ");
}
function circleSubpath(cx, cy, r) {
  return `M${cx - r} ${cy} A${r} ${r} 0 1 0 ${cx + r} ${cy} A${r} ${r} 0 1 0 ${cx - r} ${cy} Z`;
}
const MANA_GLYPHS = {
  // White — spiky sun.
  W: `<svg viewBox="0 0 24 24"><polygon points="${sunburstPoints(12, 12, 11, 6.2, 9)}" fill="${GLYPH_FILL}"/><circle cx="12" cy="12" r="4.1" fill="${GLYPH_FILL}"/></svg>`,
  // Blue — water droplet.
  U: `<svg viewBox="0 0 24 24"><path d="M12 2.3C12 2.3 5 11.2 5 15.7a7 7 0 0 0 14 0C19 11.2 12 2.3 12 2.3z" fill="${GLYPH_FILL}"/></svg>`,
  // Black — skull, eye sockets cut out via evenodd.
  B: `<svg viewBox="0 0 24 24"><path fill-rule="evenodd" fill="${GLYPH_FILL}" d="M12 3.4c-4.3 0-7.7 3.3-7.7 7.4 0 2.9 1.7 5.4 4.3 6.6.1 1 .2 1.9.2 1.9h6.4s.1-.9.2-1.9c2.6-1.2 4.3-3.7 4.3-6.6 0-4.1-3.4-7.4-7.7-7.4z ${circleSubpath(8.9, 10.6, 1.7)} ${circleSubpath(15.1, 10.6, 1.7)}"/></svg>`,
  // Red — flame.
  R: `<svg viewBox="0 0 24 24"><path fill-rule="evenodd" fill="${GLYPH_FILL}" d="M12.963 2.286a.75.75 0 00-1.071-.136 9.742 9.742 0 00-3.539 6.176 7.547 7.547 0 01-1.705-1.715.75.75 0 00-1.152-.082A9 9 0 1015.68 4.534a7.46 7.46 0 01-2.717-2.248zM15.75 14.25a3.75 3.75 0 11-7.313-1.172c.628.465 1.35.81 2.133 1a5.99 5.99 0 011.925-3.545 3.75 3.75 0 013.255 3.717z"/></svg>`,
  // Green — tree / canopy on a trunk.
  G: `<svg viewBox="0 0 24 24"><circle cx="9" cy="10.3" r="3.9" fill="${GLYPH_FILL}"/><circle cx="15" cy="10.3" r="3.9" fill="${GLYPH_FILL}"/><circle cx="12" cy="7.4" r="4.2" fill="${GLYPH_FILL}"/><rect x="10.6" y="13" width="2.8" height="6.3" rx="1.2" fill="${GLYPH_FILL}"/></svg>`,
};

/** Returns the flat icon glyph (sun/droplet/skull/flame/tree) for W/U/B/R/G, or null if the
 * letter has no icon (e.g. "C" colorless) — callers can fall back to the plain letter. */
export function manaGlyphSvg(letter) {
  return MANA_GLYPHS[letter] || null;
}

function singleSymbolHtml(inner) {
  if (/^\d+$/.test(inner)) {
    return `<span class="mana-sym" style="background:#d8d3c9">${inner}</span>`;
  }
  if (inner === "X" || inner === "Y" || inner === "Z") {
    return `<span class="mana-sym" style="background:#d8d3c9">${inner}</span>`;
  }
  if (MANA_GLYPHS[inner]) {
    return `<span class="mana-sym" style="background:${MANA_COLORS[inner]}" title="${inner}">${MANA_GLYPHS[inner]}</span>`;
  }
  if (MANA_COLORS[inner]) {
    return `<span class="mana-sym" style="background:${MANA_COLORS[inner]}">${inner}</span>`;
  }
  // hybrid (e.g. B/P, 2/B, W/U) — shows both components very small
  if (inner.includes("/")) {
    const parts = inner.split("/");
    const bg = MANA_COLORS[parts[parts.length - 1]] || "#d8d3c9";
    return `<span class="mana-sym mana-sym-hybrid" style="background:${bg}">${parts.join("/")}</span>`;
  }
  return `<span class="mana-sym" style="background:#d8d3c9">${inner}</span>`;
}

/** Renders a mana cost ("{3}{B}{B}") as colored round symbols, matching real MTG style. */
export function manaCostHtml(cost) {
  if (!cost) return "";
  const symbols = cost.match(/\{[^}]+\}/g);
  if (!symbols) return "";
  return `<span class="mana-cost">${symbols.map((s) => singleSymbolHtml(s.slice(1, -1))).join("")}</span>`;
}
