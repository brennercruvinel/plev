import { api } from "../api.js?v=25";
import { mainEl } from "../router.js?v=3";
import { openAddCardModal } from "../ui/add-card.js?v=3";
import { showCardModal } from "../ui/card-modal.js?v=3";
import { cardImgHtml, wireCardFlips } from "../ui/card-face.js?v=1";
import { confirmDialog } from "../ui/confirm.js?v=1";
import {
  filterDropdownHtml, filterMenuContentHtml, filterToggleBtnHtml, matchesFilters, newFilterState,
  wireFilterMenu,
} from "../ui/card-filters.js?v=3";
import { h, priceLabel, toast } from "../util.js?v=3";
import { attachClear } from "../ui/search-field.js?v=1";

// Same controls as the collection — search, chip filters, size slider, card modal — because it is
// the same job on a different list. The API returns the same grouped shape for exactly that
// reason, so this view differs only in what the tiles say and what the actions do.
const wishFilters = newFilterState();

export async function renderWishlist() {
  mainEl.innerHTML = h`
    <div class="page-header">
      <div><h1>Wishlist</h1><p>Cartas que você quer comprar. Não contam como cartas que você tem — quando comprar, mova para a coleção com um clique.</p></div>
      <button class="btn small" id="add-card-btn">+ Adicionar Carta</button>
    </div>
    <div class="filters-bar">
      <input type="text" id="wl-search" placeholder="Buscar carta (PT ou EN)…">
      <span id="wl-summary" class="wl-summary"></span>
      ${filterDropdownHtml(wishFilters, { rarity: true, search: false })}
      <div class="size-slider-row">
        <label for="card-size">Tamanho</label>
        <input type="range" id="card-size" min="100" max="280" step="10" value="160">
      </div>
    </div>
    <div class="card-grid" id="wl-grid"></div>
  `;

  const grid = document.getElementById("wl-grid");
  document.getElementById("card-size").addEventListener("input", (e) => {
    grid.style.setProperty("--card-min", `${e.target.value}px`);
  });

  async function load() {
    const q = document.getElementById("wl-search").value.trim();
    const [all, total] = await Promise.all([api.wishlist(q), api.wishlistTotal()]);
    const items = all.filter((c) => matchesFilters(c, wishFilters));

    document.getElementById("wl-summary").innerHTML = total.total_units
      ? h`<b>${total.total_units}</b> carta(s) · ${total.distinct_cards} nome(s) · ~${priceLabel(String(total.price_usd))}`
      : "";

    grid.innerHTML =
      items
        .map((c) => {
          const details = c.entries
            .map((e) => [e.set_code ? e.set_code.toUpperCase() : null, e.notes].filter(Boolean).join(" · "))
            .filter(Boolean)
            .join(" | ");
          return h`
            <div class="mtg-card wish-card" data-card-view="${c.card_name}" title="${details || c.card_name}">
              ${cardImgHtml(c, { attrs: 'loading="lazy" decoding="async"' })}
              <span class="qty-badge">${c.total_quantity}x</span>
              ${c.price_usd ? `<span class="wish-price">${priceLabel(c.price_usd)}</span>` : ""}
              <div class="wish-actions">
                <button class="btn small" data-acquire="${c.entries[0].id}" title="Já comprei — mover para a coleção">Comprei</button>
                <button class="icon-btn danger" data-drop="${c.entries[0].id}" data-drop-name="${c.card_name}" title="Tirar da wishlist" aria-label="Tirar da wishlist">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" width="14" height="14"><path d="M3 6h18"/><path d="M8 6V4a1 1 0 0 1 1-1h6a1 1 0 0 1 1 1v2"/><path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6"/><path d="M10 11v6M14 11v6"/></svg>
                </button>
              </div>
            </div>`;
        })
        .join("") ||
      `<div class="empty-state">${
        all.length
          ? "Nenhuma carta corresponde aos filtros."
          : "Sua wishlist está vazia. Use <b>+ Adicionar Carta</b> e escolha a aba <b>Wishlist</b>."
      }</div>`;
  }

  wireCardFlips(grid);
  grid.addEventListener("click", async (e) => {
    const acquire = e.target.closest("[data-acquire]");
    if (acquire) {
      e.stopPropagation();
      await api.acquireWishlistEntry(Number(acquire.dataset.acquire));
      toast("Movida para a coleção.");
      load();
      return;
    }
    const drop = e.target.closest("[data-drop]");
    if (drop) {
      e.stopPropagation();
      const name = drop.dataset.dropName;
      const ok = await confirmDialog({
        title: "Tirar da wishlist?",
        message: `${name} sai da sua lista de compras.`,
        confirmLabel: "Remover",
        danger: true,
      });
      if (!ok) return;
      await api.deleteWishlistEntry(Number(drop.dataset.drop));
      toast(`${name} saiu da wishlist.`);
      load();
      return;
    }
    const tile = e.target.closest("[data-card-view]");
    if (tile) showCardModal(tile.dataset.cardView, load);
  });

  document.getElementById("add-card-btn").addEventListener("click", () =>
    openAddCardModal({ onSaved: load })
  );

  function refreshFilterMenu() {
    document.getElementById("filter-toggle-btn").outerHTML = filterToggleBtnHtml(wishFilters);
    document.getElementById("filter-menu").outerHTML = filterMenuContentHtml(wishFilters, {
      rarity: true,
      search: false,
    });
    wireFilters();
    load();
  }
  function wireFilters() {
    wireFilterMenu(wishFilters, (structural) => (structural ? refreshFilterMenu() : load()));
  }
  wireFilters();

  attachClear(document.getElementById("wl-search"));

  let searchDebounce;
  document.getElementById("wl-search").addEventListener("input", () => {
    clearTimeout(searchDebounce);
    searchDebounce = setTimeout(load, 250);
  });

  load();
}
