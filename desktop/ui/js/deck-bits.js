import { manaGlyphSvg, resultIcon } from "./icons.js?v=25";
import { h } from "./util.js?v=2";

function colorIdentityPipsHtml(identity) {
  const letters = identity ? identity.split("") : [];
  if (!letters.length) letters.push("C"); // truly colorless commander
  return letters
    .map((l) => {
      const glyph = manaGlyphSvg(l);
      return `<span class="color-pip" style="background:${FILTER_COLORS[l] || "#8b8398"}" title="${l}">${
        glyph ? `<span class="mana-sym-inline">${glyph}</span>` : l
      }</span>`;
    })
    .join("");
}

/** Deck.tags is stored as a plain comma-separated string (see db.py) — split/trim/drop-empties
 * here rather than parsing JSON, so a hand-typed "Competitivo,, Budget ," doesn't need any
 * special-casing beyond what .filter(Boolean) already does. */
function parseDeckTags(tagsStr) {
  return (tagsStr || "").split(",").map((t) => t.trim()).filter(Boolean);
}

/** Small non-interactive pills for a deck's custom tags (see openEditDeckModal) — shared between
 * the deck grid thumbnails and the deck detail header so both stay visually consistent. Rendered
 * inline right next to the deck name at both call sites (see the wrapping flex row there). */
export function deckTagsHtml(tagsStr) {
  const tags = parseDeckTags(tagsStr);
  if (!tags.length) return "";
  return `<div class="deck-tags">${tags.map((t) => `<span class="deck-tag">${t}</span>`).join("")}</div>`;
}

export function deckCardHtml(d) {
  const cls = d.total_cards === 100 ? "ok" : "bad";
  const hasPartner = !!d.commander_name_2;
  // Custom tags render as an overlay on the art itself (top-left corner) rather than as text
  // next to the deck name — reads faster as a label on the "cover" of the deck, and doesn't
  // compete with the name for space in the already-tight card body below.
  const thumb = hasPartner
    ? `<div class="deck-card-thumb split">
        ${deckTagsHtml(d.tags)}
        ${d.commander_image ? `<img src="${d.commander_image}" alt="${d.commander_name}">` : `<div class="no-image">${d.commander_name}</div>`}
        ${d.commander_image_2 ? `<img src="${d.commander_image_2}" alt="${d.commander_name_2}">` : `<div class="no-image">${d.commander_name_2}</div>`}
      </div>`
    : `<div class="deck-card-thumb">
        ${deckTagsHtml(d.tags)}
        ${d.commander_image ? `<img src="${d.commander_image}" alt="${d.commander_name}">` : `<div class="no-image">${d.commander_name}</div>`}
      </div>`;
  const commanderLine = hasPartner ? `${d.commander_name} + ${d.commander_name_2}` : d.commander_name;

  return h`
    <div class="deck-card" data-deck-link="${d.id}">
      ${thumb}
      <div class="deck-card-body">
        <h3>${d.name}</h3>
        <div class="deck-card-commander">${commanderLine}</div>
        <div class="philosophy">${d.philosophy || ""}</div>
        <div class="meta-row">
          <span class="count-pill ${cls}">${d.total_cards}/100</span>
          <span class="deck-card-colors">${colorIdentityPipsHtml(d.color_identity)}</span>
          <span class="wl">
            <span class="wl-item win">${resultIcon("win")}${d.wins}</span>
            <span class="wl-item loss">${resultIcon("loss")}${d.losses}</span>
          </span>
        </div>
      </div>
    </div>`;
}

// --------------------------------------------------------------- decks ----

export const BRACKET_LABELS = {
  1: "1 — Exhibition (ultra-casual)",
  2: "2 — Core (nível de precon)",
  3: "3 — Upgraded (até 3 Game Changers)",
  4: "4 — Optimized (sem restrições)",
  5: "5 — cEDH (poder máximo)",
};

export const FILTER_COLORS = { W: "#fffbd5", U: "#aae0fa", B: "#cbc2bf", R: "#f9aa8f", G: "#9bd3ae", C: "#8b8398" };
