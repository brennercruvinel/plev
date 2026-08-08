// Hash router and the sidebar chrome around it.
//
// This module deliberately knows nothing about the pages it shows. Views register themselves
// through `registerRoutes`, so adding a page means writing a view module and naming it once in
// app.js — the router never grows a new branch, and it can't end up importing half the app.

import { api } from "./api.js?v=24";

export const mainEl = document.getElementById("main");
const navItems = document.querySelectorAll(".nav-item");

// route name -> render function, filled in by app.js at startup.
const routes = Object.create(null);

export function registerRoutes(map) {
  Object.assign(routes, map);
}

function setActiveNav(route) {
  navItems.forEach((el) => el.classList.toggle("active", el.dataset.route === route));
}

export async function navigate() {
  const hash = location.hash.slice(1) || "dashboard";
  const [route, ...rest] = hash.split("/");
  setActiveNav(route);
  mainEl.innerHTML = `<div class="empty-state">Carregando…</div>`;
  try {
    await routes[route]?.(rest);
  } catch (err) {
    mainEl.innerHTML = `<div class="empty-state">Erro: ${err.message}</div>`;
    console.error(err);
  }
  // Sub-item lists (currently just the deck list under "Meus Decks") stay hidden unless
  // you're actually browsing that section — no point showing every deck name while you're
  // on the Scanner or Coleção pages. Same "visible" toggle pattern would apply to any
  // future sidebar sub-list.
  const onDecksSection = route === "decks" || route === "deck";
  document.getElementById("nav-decks-list")?.classList.toggle("visible", onDecksSection);
  if (onDecksSection) renderNavDecksList(route === "deck" ? rest[0] : null);
}

// Deck sub-list under "Meus Decks" in the sidebar — lets you jump straight to any deck
// without going back to the deck grid first. Refreshed on every navigation so it stays
// in sync after a deck is created, renamed, or deleted.
let navDecksCache = null;

async function renderNavDecksList(activeId) {
  const el = document.getElementById("nav-decks-list");
  if (!el) return;
  try {
    navDecksCache = await api.decks();
  } catch (err) {
    console.error(err);
    return;
  }
  el.innerHTML = navDecksCache
    .map(
      (d) =>
        `<button class="nav-subitem ${String(d.id) === String(activeId) ? "active" : ""}" data-deck-link="${d.id}" title="${d.name}">${d.name}</button>`
    )
    .join("");
  el.querySelectorAll("[data-deck-link]").forEach((btn) =>
    btn.addEventListener("click", () => {
      location.hash = `#deck/${btn.dataset.deckLink}`;
      btn.blur();
    })
  );
}

// Buttons keep browser focus after a click (Chrome/Edge default) — left unchecked, that keeps
// .sidebar's :focus-within true after navigating, which held the collapsed sidebar (and, on the
// decks route, the whole deck sub-list) stuck open until focus moved elsewhere. Blurring right
// after navigating lets it collapse again as soon as the mouse isn't over it, same as it would
// on hover alone.
navItems.forEach((el) => {
  el.addEventListener("click", () => {
    location.hash = "#" + el.dataset.route;
    el.blur();
  });
});
window.addEventListener("hashchange", navigate);

// Closes the deck-detail "Exportar" dropdown, the "Adicionar carta" inline search, and the
// "Filtro" menu on an outside click — registered once here (not inside renderDeckDetail, which
// re-runs on every card add/remove) so it doesn't pile up a new document-level listener on every
// re-render. Harmless no-op on any page without these elements. Outside-click (rather than blur)
// is used because a plain blur handler would fire, and collapse the dropdown, before a click on
// one of the rows/chips inside it gets a chance to register.
document.addEventListener("click", (e) => {
  const dropdown = document.getElementById("export-dropdown");
  if (dropdown && !dropdown.contains(e.target)) dropdown.classList.remove("open");
  const addCardWrap = document.getElementById("add-card-inline");
  if (addCardWrap && !addCardWrap.contains(e.target)) addCardWrap.classList.remove("expanded");
  const filterDropdown = document.getElementById("filter-dropdown");
  if (filterDropdown && !filterDropdown.contains(e.target)) filterDropdown.classList.remove("open");
});
