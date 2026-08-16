// Set autocomplete.
//
// The field used to be a raw three-letter code, which meant knowing that Dominaria United is
// "dmu" before you could record it. It now searches the set list built from the same Scryfall
// data as the card index, matching the name accent-insensitively and the code as a prefix.
//
// Set names are English even on Portuguese cards — Scryfall does not localise them — so the code
// is matched too: for recent sets that is what people actually remember, and it is the same
// string in every language.

import { api } from "../api.js?v=25";

/** Markup for the input plus its (empty) suggestion list. `value` is the stored set code. */
export function setInputHtml(id, value = "") {
  return `
    <div class="set-picker" data-set-picker>
      <input type="text" id="${id}" class="set-picker-input" placeholder="Nome ou código da edição…"
        autocomplete="off" value="${value || ""}">
      <input type="hidden" id="${id}-code" value="${value || ""}">
      <div class="set-picker-list" id="${id}-list"></div>
    </div>`;
}

/**
 * Binds one picker. Reads the chosen code from the hidden field via `getSetCode(id)`.
 *
 * Free text is kept rather than forced to a match: a promo or a set the index doesn't know is
 * still worth recording, and silently clearing what someone typed is worse than storing it.
 */
/**
 * `card` scopes the list to one card's printings. Pass a selector when the name lives in a field
 * the user is still editing, or a plain string when the card is already known — the card modal
 * knows it, and has no name field to read.
 */
export function wireSetPicker(root, id, card = null) {
  const input = root.querySelector(`#${id}`);
  const hidden = root.querySelector(`#${id}-code`);
  const list = root.querySelector(`#${id}-list`);
  if (!input || !list) return;

  let debounce;
  let items = [];
  let active = -1;
  const cardName = () => {
    if (!card) return "";
    if (typeof card === "string" && card.startsWith("#")) {
      return root.querySelector(card)?.value.trim() || "";
    }
    return typeof card === "string" ? card : "";
  };

  const close = () => {
    list.innerHTML = "";
    active = -1;
  };

  const choose = (s) => {
    input.value = `${s.name} (${s.code.toUpperCase()})`;
    hidden.value = s.code;
    close();
  };

  const render = () => {
    // The card modal scrolls (max-height + overflow-y), so a dropdown opening near its bottom
    // would be clipped by the modal rather than overlaying it. Bringing it into view keeps the
    // suggestions reachable wherever the field happens to sit.
    requestAnimationFrame(() => {
      if (list.innerHTML) list.scrollIntoView({ block: "nearest", behavior: "smooth" });
    });
    list.innerHTML = items
      .map((s, i) => {
        const year = s.released_at ? String(s.released_at).slice(0, 4) : "";
        return `<button type="button" class="set-picker-item ${i === active ? "active" : ""}" data-set-idx="${i}">
          <span class="set-picker-code">${s.code.toUpperCase()}</span>
          <span class="set-picker-name">${s.name || ""}</span>
          <span class="set-picker-year">${year}</span>
        </button>`;
      })
      .join("");
  };

  input.addEventListener("input", () => {
    // Typing invalidates the previous pick; the raw text stands in until something is chosen, so
    // a set the index doesn't carry still gets stored.
    hidden.value = input.value.trim();
    clearTimeout(debounce);
    const q = input.value.trim();
    if (!q) return close();
    debounce = setTimeout(async () => {
      try {
        items = await api.sets(q, cardName());
        active = -1;
        render();
      } catch {
        close();
      }
    }, 180);
  });

  // With a card already chosen, focusing the empty field lists its printings straight away —
  // there is no useful "type to narrow" step when the answer is a handful of sets.
  input.addEventListener("focus", async () => {
    // Also when the field already holds a value: editing an existing copy pre-fills it, and the
    // whole reason to open this field is to change what is there.
    if (!cardName()) return;
    try {
      items = await api.sets("", cardName());
      active = -1;
      render();
    } catch {
      close();
    }
  });

  input.addEventListener("keydown", (e) => {
    if (!items.length) return;
    if (e.key === "ArrowDown" || e.key === "ArrowUp") {
      e.preventDefault();
      active = (active + (e.key === "ArrowDown" ? 1 : -1) + items.length) % items.length;
      render();
    } else if (e.key === "Enter" && active >= 0) {
      e.preventDefault();
      choose(items[active]);
    } else if (e.key === "Escape") {
      close();
    }
  });

  list.addEventListener("mousedown", (e) => {
    // mousedown, not click: blur would close the list before the click landed.
    const btn = e.target.closest("[data-set-idx]");
    if (btn) {
      e.preventDefault();
      choose(items[Number(btn.dataset.setIdx)]);
    }
  });

  input.addEventListener("blur", () => setTimeout(close, 120));
}

export function getSetCode(root, id) {
  return root.querySelector(`#${id}-code`)?.value.trim() || null;
}
