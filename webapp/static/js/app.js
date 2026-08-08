import { api } from "./api.js?v=24";
import { activityIcon, manaCostHtml, manaGlyphSvg, resultIcon } from "./icons.js?v=23";

const mainEl = document.getElementById("main");
const navItems = document.querySelectorAll(".nav-item");

const routes = {
  dashboard: renderDashboard,
  decks: renderDecksList,
  deck: renderDeckDetail,
  collection: renderCollection,
  // Scanner is the last feature slated for the native port; until then the page states that
  // plainly instead of exposing a camera flow whose backend isn't there.
  scanner: () => renderScannerSoon(),
  games: renderGames,
};

function setActiveNav(route) {
  navItems.forEach((el) => el.classList.toggle("active", el.dataset.route === route));
}

async function navigate() {
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

// ----------------------------------------------------------------- utils ----

function h(strings, ...values) {
  return strings.reduce((acc, s, i) => acc + s + (values[i] ?? ""), "");
}

function priceLabel(p) {
  return p ? `$${Number(p).toFixed(2)}` : "—";
}

// ------------------------------------------------------------- dashboard ----

async function renderDashboard() {
  const [decks, collectionTotal, freeCollection, activity] = await Promise.all([
    api.decks(),
    api.collectionTotal(),
    api.collection("free"),
    api.activity(12),
  ]);
  const totalCards = decks.reduce((s, d) => s + d.total_cards, 0);
  const totalGames = decks.reduce((s, d) => s + d.wins + d.losses, 0);
  const totalWins = decks.reduce((s, d) => s + d.wins, 0);

  mainEl.innerHTML = h`
    <div class="page-header">
      <div><h1>Home</h1><p>Seu laboratório de coleção e decks — tudo lido do banco local.</p></div>
      <div style="display:flex;gap:10px">
        <button class="btn" id="add-card-btn">+ Adicionar Carta</button>
        <button class="btn secondary" id="dash-new-deck-btn">Novo Deck</button>
      </div>
    </div>
    <div class="stat-grid">
      <div class="stat-card clickable" data-stat-nav="decks"><div class="label">Decks montados</div><div class="value">${decks.length}</div><div class="sub">${totalCards} cartas ao todo</div></div>
      <div class="stat-card clickable" data-stat-nav="collection"><div class="label">Cartas na coleção</div><div class="value">${collectionTotal.total_units}</div><div class="sub">${collectionTotal.distinct_cards} nomes distintos, com repetidas</div></div>
      <div class="stat-card clickable" data-stat-nav="collection"><div class="label">Cartas livres</div><div class="value">${collectionTotal.free_units}</div><div class="sub">${freeCollection.length} nomes · ${collectionTotal.allocated_units} em decks</div></div>
      <div class="stat-card clickable" data-stat-nav="games"><div class="label">Partidas registradas</div><div class="value">${totalGames}</div><div class="sub">${totalWins} vitórias</div></div>
    </div>

    <div class="page-header"><h1 style="font-size:17px">Seus decks</h1></div>
    <div class="deck-grid" id="dash-decks"></div>

    <div class="page-header" style="margin-top:28px"><h1 style="font-size:17px">Histórico de atividades</h1>
      <p>Cartas novas, movimentações entre decks e decks montados — registrado conforme você usa o app.</p></div>
    <div id="dash-activity"></div>
  `;

  document.getElementById("dash-decks").innerHTML = decks
    .map(deckCardHtml)
    .join("") + h`
      <div class="deck-add-tile" id="dash-new-deck-tile">
        <span class="plus">+</span>
        <span class="label">Novo Deck</span>
      </div>`;
  document.querySelectorAll("[data-deck-link]").forEach((el) =>
    el.addEventListener("click", () => (location.hash = `#deck/${el.dataset.deckLink}`))
  );
  document.getElementById("add-card-btn").addEventListener("click", () =>
    openAddCardModal({ onSaved: renderDashboard })
  );
  document.getElementById("dash-new-deck-btn").addEventListener("click", () => openNewDeckModal());
  document.getElementById("dash-new-deck-tile").addEventListener("click", () => openNewDeckModal());
  document.querySelectorAll("[data-stat-nav]").forEach((el) =>
    el.addEventListener("click", () => (location.hash = `#${el.dataset.statNav}`))
  );

  document.getElementById("dash-activity").innerHTML = activity
    .map(
      (a) => h`
      <div class="activity-row">
        <span class="activity-icon">${activityIcon(a.type)}</span>
        <span class="activity-desc">${a.description}</span>
        <span class="activity-ts">${formatTs(a.ts)}</span>
      </div>`
    )
    .join("") || `<div class="empty-state">Nenhuma atividade ainda — mexa num deck ou na coleção pra ver o histórico aqui.</div>`;
}

function renderScannerSoon() {
  mainEl.innerHTML = h`
    <div class="page-header">
      <div><h1>Scanner</h1><p>Reconhecimento de cartas pela câmera.</p></div>
    </div>
    <div class="ownership-summary" style="border-left-color:var(--accent)">
      <div class="own-line">
        <span class="own-tag tag-other-deck">Em breve</span>
        <span>Esta é a última funcionalidade pendente do app nativo. Enquanto isso, use
          <b>+ Adicionar Carta</b> (com o modo <b>Adicionar por lista</b>) ou
          <b>Importar decklist</b> na tela do deck.</span>
      </div>
    </div>`;
}

function formatTs(ts) {
  // ts comes as "YYYY-MM-DD HH:MM:SS" (UTC, from SQLite datetime('now'))
  const d = new Date(ts.replace(" ", "T") + "Z");
  return d.toLocaleString("pt-BR", { day: "2-digit", month: "2-digit", hour: "2-digit", minute: "2-digit" });
}

// -------------------------------------------------------- background jobs ----
// Shared progress-bar UI for any backend job that exposes {state, task, percent, error}
// (data_update.py and deck_wizard.py both follow this exact shape).

function progressBarHtml(prefix) {
  return `
    <div class="update-progress-label">
      <span id="${prefix}-task-text" class="update-task-text"></span>
      <span id="${prefix}-percent-text" class="update-percent-text"></span>
    </div>
    <div class="update-progress-bar"><div class="update-progress-fill" id="${prefix}-progress-fill" style="width:0%"></div></div>
    <div id="${prefix}-error-text" class="update-error" style="display:none"></div>`;
}

function pollJob(statusEl, prefix, fetchStatusFn, { onDone, onError, onSettle } = {}) {
  statusEl.innerHTML = progressBarHtml(prefix);
  const taskText = statusEl.querySelector(`#${prefix}-task-text`);
  const percentText = statusEl.querySelector(`#${prefix}-percent-text`);
  const fill = statusEl.querySelector(`#${prefix}-progress-fill`);
  const errorText = statusEl.querySelector(`#${prefix}-error-text`);

  const tick = async () => {
    if (!document.body.contains(statusEl)) return; // user navigated away — stop polling from here
    const status = await fetchStatusFn();
    const pct = Math.round(status.percent || 0);
    taskText.textContent = status.task || "";
    percentText.textContent = `${pct}%`;
    fill.style.width = `${pct}%`;
    if (status.state === "running") {
      setTimeout(tick, 1000);
      return;
    }
    onSettle?.(status);
    if (status.state === "error") {
      errorText.textContent = `Falhou: ${status.error}`;
      errorText.style.display = "block";
      onError?.(status);
    } else if (status.state === "done") {
      onDone?.(status);
    }
  };
  tick();
}

// -------------------------------------------------- sidebar data-base panel ----
// Lives in the sidebar footer (persists across every page, not just the Dashboard).

function formatBuiltAt(unixSeconds) {
  if (!unixSeconds) return null;
  return new Date(unixSeconds * 1000).toLocaleString("pt-BR", {
    day: "2-digit", month: "2-digit", year: "numeric", hour: "2-digit", minute: "2-digit",
  });
}

function sidebarDataPanelHtml(info) {
  if (!info.exists) {
    return h`
      <button class="btn small" id="update-data-btn" style="width:100%;margin-top:16px">Baixar base de dados agora</button>
      <div class="sidebar-data-info warn" style="margin-top:8px">Base de cartas ainda não configurada.</div>
      <div id="update-data-status"></div>`;
  }
  return h`
    <button class="btn small secondary" id="update-data-btn" style="width:100%;margin-top:16px">Atualizar base de dados</button>
    <div class="sidebar-data-info" style="margin-top:8px">Dados: ${info.cards.toLocaleString("pt-BR")} cartas + EDHREC cacheado${info.built_at ? ` · atualizado em ${formatBuiltAt(info.built_at)}` : ""}. Só preço ao vivo precisa de rede.</div>
    <div id="update-data-status"></div>`;
}

async function renderSidebarDataPanel() {
  const panelEl = document.getElementById("sidebar-data-panel");
  if (!panelEl) return;
  const info = await api.dataInfo();
  panelEl.innerHTML = sidebarDataPanelHtml(info);

  const btn = document.getElementById("update-data-btn");
  const statusEl = document.getElementById("update-data-status");
  btn.dataset.idleLabel = btn.textContent;

  const startPolling = () => {
    pollJob(statusEl, "update-data", api.dataUpdateStatus, {
      onSettle: () => { btn.disabled = false; btn.textContent = btn.dataset.idleLabel; },
      onDone: () => { renderSidebarDataPanel(); navigate(); }, // refresh sidebar info + whatever page is showing
    });
  };

  btn.addEventListener("click", async () => {
    btn.disabled = true;
    btn.textContent = "Atualizando…";
    try {
      await api.startDataUpdate();
    } catch (err) {
      // 409 = already running (e.g. started from another tab) — just start polling
    }
    startPolling();
  });

  // resume polling if an update was already in progress (e.g. page was reloaded mid-update)
  const status = await api.dataUpdateStatus();
  if (status.state === "running") {
    btn.disabled = true;
    btn.textContent = "Atualizando…";
    startPolling();
  }
}

/** Small colored circles for a deck's color identity (commander's colors, combined for
 * partners) — reuses the same palette/glyphs as the deck filter bar's color chips, just
 * non-interactive and sized down to fit a deck card. */
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
function deckTagsHtml(tagsStr) {
  const tags = parseDeckTags(tagsStr);
  if (!tags.length) return "";
  return `<div class="deck-tags">${tags.map((t) => `<span class="deck-tag">${t}</span>`).join("")}</div>`;
}

function deckCardHtml(d) {
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

async function renderDecksList() {
  const decks = await api.decks();
  mainEl.innerHTML = h`
    <div class="page-header">
      <div><h1>Meus Decks</h1><p>Clique num deck para editar, ver curva de mana e sugestões de sinergia.</p></div>
      <button class="btn" id="new-deck-btn">+ Novo Deck</button>
    </div>
    <div class="deck-grid" id="decks-grid"></div>
  `;
  document.getElementById("decks-grid").innerHTML = decks.map(deckCardHtml).join("");
  document.querySelectorAll("[data-deck-link]").forEach((el) =>
    el.addEventListener("click", () => (location.hash = `#deck/${el.dataset.deckLink}`))
  );
  document.getElementById("new-deck-btn").addEventListener("click", () => openNewDeckModal());
}

const BRACKET_LABELS = {
  1: "1 — Exhibition (ultra-casual)",
  2: "2 — Core (nível de precon)",
  3: "3 — Upgraded (até 3 Game Changers)",
  4: "4 — Optimized (sem restrições)",
  5: "5 — cEDH (poder máximo)",
};

async function openNewDeckModal() {
  const backdrop = document.createElement("div");
  backdrop.className = "modal-backdrop";
  backdrop.innerHTML = h`
    <div class="modal">
      <h3>Novo deck</h3>
      <div style="margin-bottom:12px">
        <label style="font-size:12px;color:var(--text-dim)">Comandante *</label>
        <input type="text" id="nd-commander" placeholder="Nome (PT ou EN)…" autocomplete="off"
          style="width:100%;margin-top:5px;padding:9px 11px;border-radius:8px;border:1px solid var(--border-light);background:var(--bg-card);color:var(--text)">
        <div id="nd-suggestions"></div>
      </div>
      <div style="margin-bottom:12px" id="nd-partner-wrap">
        <label style="font-size:12px;color:var(--text-dim)">Comandante parceiro (opcional)</label>
        <input type="text" id="nd-commander-2" placeholder="Ex: Partner, Background…" autocomplete="off"
          style="width:100%;margin-top:5px;padding:9px 11px;border-radius:8px;border:1px solid var(--border-light);background:var(--bg-card);color:var(--text)">
        <div id="nd-suggestions-2"></div>
        <p id="nd-partner-note" style="display:none;margin:6px 0 0;font-size:11.5px;color:var(--text-faint)">Montagem automática ainda monta com um único comandante (v1) — o parceiro não entra na montagem, mas você pode adicioná-lo depois editando o deck.</p>
      </div>
      <div style="margin-bottom:12px">
        <label style="font-size:12px;color:var(--text-dim)">Nome do deck *</label>
        <input type="text" id="nd-name" placeholder="Ex: Syr Konrad Aristocratas"
          style="width:100%;margin-top:5px;padding:9px 11px;border-radius:8px;border:1px solid var(--border-light);background:var(--bg-card);color:var(--text)">
      </div>

      <label class="toggle-row" style="margin-bottom:12px">
        <input type="checkbox" id="nd-auto">
        <span>Montar automaticamente</span>
      </label>
      <div id="nd-auto-options" style="display:none;margin-bottom:12px">
        <p style="margin:0 0 10px;font-size:12px;color:var(--text-dim);line-height:1.5">
          Preenche as 99 cartas a partir da base local do Scryfall, sinergia do EDHREC e do guia de proporções
          (terrenos, ramp, compra, remoção, proteção) — sem IA generativa, toda carta é verificada na base local antes de entrar no deck.
        </p>
        <label style="font-size:12px;color:var(--text-dim)">Bracket alvo</label>
        <select id="nd-bracket" style="width:100%;margin-top:5px;padding:9px 11px;border-radius:8px;border:1px solid var(--border-light);background:var(--bg-card);color:var(--text)">
          ${Object.entries(BRACKET_LABELS).map(([v, label]) => `<option value="${v}" ${v === "3" ? "selected" : ""}>${label}</option>`).join("")}
        </select>

        <label style="font-size:12px;color:var(--text-dim);display:block;margin-top:12px">Quais cartas usar</label>
        <label class="toggle-row" style="align-items:flex-start;margin-top:6px">
          <input type="radio" name="nd-mode" value="suggest" checked style="margin-top:3px">
          <span><b>Sugerir as melhores</b><br>
            <span style="color:var(--text-dim);font-size:12px">Monta o melhor deck possível, mesmo com cartas que você não tem — as que faltam ficam marcadas para você comprar.</span>
          </span>
        </label>
        <label class="toggle-row" style="align-items:flex-start;margin-top:8px">
          <input type="radio" name="nd-mode" value="owned" style="margin-top:3px">
          <span><b>Só o que tenho na coleção</b><br>
            <span style="color:var(--text-dim);font-size:12px">Deck montável hoje. Cartas que estão em outro deck entram marcadas, avisando que você teria que desmontá-lo.</span>
          </span>
        </label>
      </div>

      <div style="margin-bottom:12px">
        <label style="font-size:12px;color:var(--text-dim)" id="nd-philosophy-label">Estratégia / filosofia (opcional)</label>
        <textarea id="nd-philosophy" rows="3" style="width:100%;margin-top:5px;padding:9px 11px;border-radius:8px;border:1px solid var(--border-light);background:var(--bg-card);color:var(--text);font-family:inherit"></textarea>
      </div>
      <div>
        <label style="font-size:12px;color:var(--text-dim)">Tags (opcional, separadas por vírgula)</label>
        <input type="text" id="nd-tags" placeholder="Ex: Competitivo, Orçamento baixo, cEDH"
          style="width:100%;margin-top:5px;padding:9px 11px;border-radius:8px;border:1px solid var(--border-light);background:var(--bg-card);color:var(--text)">
      </div>
      <div id="nd-error" style="color:var(--bad);font-size:12px;margin-top:10px;display:none">Preencha comandante e nome do deck.</div>
      <div id="nd-build-status"></div>
      <div style="display:flex;gap:10px;margin-top:18px;justify-content:flex-end">
        <button class="btn secondary" id="nd-cancel">Cancelar</button>
        <button class="btn" id="nd-save">Criar deck</button>
      </div>
    </div>`;
  document.body.appendChild(backdrop);
  backdrop.addEventListener("click", (e) => { if (e.target === backdrop) backdrop.remove(); });

  const commanderInput = backdrop.querySelector("#nd-commander");
  const suggestionsEl = backdrop.querySelector("#nd-suggestions");
  const commander2Input = backdrop.querySelector("#nd-commander-2");
  const suggestions2El = backdrop.querySelector("#nd-suggestions-2");
  const partnerNote = backdrop.querySelector("#nd-partner-note");
  const autoToggle = backdrop.querySelector("#nd-auto");
  const autoOptions = backdrop.querySelector("#nd-auto-options");
  const philosophyLabel = backdrop.querySelector("#nd-philosophy-label");
  const saveBtn = backdrop.querySelector("#nd-save");
  commanderInput.focus();

  autoToggle.addEventListener("change", () => {
    autoOptions.style.display = autoToggle.checked ? "block" : "none";
    partnerNote.style.display = autoToggle.checked ? "block" : "none";
    philosophyLabel.textContent = autoToggle.checked
      ? "Estratégia / filosofia (opcional — substitui o texto gerado automaticamente)"
      : "Estratégia / filosofia (opcional)";
    saveBtn.textContent = autoToggle.checked ? "Montar deck" : "Criar deck";
  });

  function wireCommanderAutocomplete(input, suggestEl, { fillName } = {}) {
    let debounce;
    input.addEventListener("input", () => {
      clearTimeout(debounce);
      const q = input.value.trim();
      if (q.length < 2) { suggestEl.innerHTML = ""; return; }
      debounce = setTimeout(async () => {
        const results = await api.searchCards(q, 6);
        suggestEl.innerHTML = results
          .map((c) => h`<div class="card-row" data-pick="${c.name}" style="cursor:pointer"><span class="name">${c.name}</span><span class="cost">${manaCostHtml(c.mana_cost)}</span></div>`)
          .join("");
        suggestEl.querySelectorAll("[data-pick]").forEach((el) =>
          el.addEventListener("click", () => {
            input.value = el.dataset.pick;
            suggestEl.innerHTML = "";
            if (fillName && !backdrop.querySelector("#nd-name").value.trim()) {
              backdrop.querySelector("#nd-name").value = el.dataset.pick;
            }
          })
        );
      }, 250);
    });
  }
  wireCommanderAutocomplete(commanderInput, suggestionsEl, { fillName: true });
  wireCommanderAutocomplete(commander2Input, suggestions2El);

  backdrop.querySelector("#nd-cancel").addEventListener("click", () => backdrop.remove());
  backdrop.querySelector("#nd-save").addEventListener("click", async () => {
    const commander_name = commanderInput.value.trim();
    const commander_name_2 = commander2Input.value.trim() || null;
    const name = backdrop.querySelector("#nd-name").value.trim();
    if (!commander_name || !name) {
      backdrop.querySelector("#nd-error").style.display = "block";
      return;
    }
    const philosophy = backdrop.querySelector("#nd-philosophy").value.trim() || null;
    const tags = backdrop.querySelector("#nd-tags").value.trim() || null;
    saveBtn.disabled = true;
    commanderInput.disabled = true;

    if (!autoToggle.checked) {
      const { id } = await api.createDeck({ name, commander_name, commander_name_2, philosophy, tags });
      backdrop.remove();
      location.hash = `#deck/${id}`;
      return;
    }

    const bracket = Number(backdrop.querySelector("#nd-bracket").value);
    const statusEl = backdrop.querySelector("#nd-build-status");
    try {
      const mode = backdrop.querySelector('input[name="nd-mode"]:checked')?.value || "suggest";
      await api.startAutoBuildDeck({ name, commander_name, bracket, philosophy, mode });
    } catch (err) {
      saveBtn.disabled = false;
      commanderInput.disabled = false;
      statusEl.innerHTML = `<div class="update-error">${err.message}</div>`;
      return;
    }
    pollJob(statusEl, "nd-build", api.autoBuildStatus, {
      onDone: (status) => {
        backdrop.remove();
        location.hash = `#deck/${status.result.deck_id}`;
      },
      onError: () => {
        saveBtn.disabled = false;
        commanderInput.disabled = false;
      },
    });
  });
}

/// Deleting a deck has two very different meanings depending on whether you physically own its
/// cards: a real deck you're taking apart should leave its cards in the collection, while a
/// planned/auto-built list you never bought should take them out with it. Asking is the only way
/// to know which — guessing either way silently corrupts the collection count.
async function openDeleteDeckModal(deck, id) {
  const inCollection = (await api.collection("allocated"))
    .filter((c) => c.decks.some((d) => d.deck_id === id))
    .reduce((sum, c) => sum + c.decks.filter((d) => d.deck_id === id).reduce((s, d) => s + d.quantity, 0), 0);

  const backdrop = document.createElement("div");
  backdrop.className = "modal-backdrop";
  backdrop.innerHTML = h`
    <div class="modal">
      <h3>Excluir "${deck.name}"</h3>
      <p style="color:var(--text-dim);font-size:13px;line-height:1.5;margin:0 0 14px">
        ${inCollection
          ? `Este deck tem <b>${inCollection}</b> carta(s) registradas na sua coleção. O que fazer com elas?`
          : "Nenhuma carta deste deck está registrada na sua coleção, então nada será removido de lá."}
      </p>
      ${inCollection ? `
      <label class="toggle-row" style="align-items:flex-start;margin-bottom:12px">
        <input type="radio" name="del-mode" value="free" checked style="margin-top:3px">
        <span><b>Devolver para cartas livres</b><br>
          <span style="color:var(--text-dim);font-size:12px">Você tem essas cartas fisicamente — o deck foi desmontado, mas elas continuam na coleção.</span>
        </span>
      </label>
      <label class="toggle-row" style="align-items:flex-start">
        <input type="radio" name="del-mode" value="remove" style="margin-top:3px">
        <span><b>Remover também da coleção</b><br>
          <span style="color:var(--text-dim);font-size:12px">Você não tem essas cartas — era uma lista planejada. Some da coleção junto com o deck.</span>
        </span>
      </label>` : ""}
      <div style="display:flex;gap:10px;margin-top:18px;justify-content:flex-end">
        <button class="btn secondary" id="dd-cancel">Cancelar</button>
        <button class="btn danger" id="dd-confirm">Excluir deck</button>
      </div>
    </div>`;
  document.body.appendChild(backdrop);
  backdrop.addEventListener("click", (e) => { if (e.target === backdrop) backdrop.remove(); });
  backdrop.querySelector("#dd-cancel").addEventListener("click", () => backdrop.remove());
  backdrop.querySelector("#dd-confirm").addEventListener("click", async (e) => {
    e.target.disabled = true;
    const mode = backdrop.querySelector('input[name="del-mode"]:checked')?.value || "free";
    await api.deleteDeck(id, mode);
    backdrop.remove();
    location.hash = "#decks";
  });
}

async function openEditDeckModal(deck, onSaved) {
  const backdrop = document.createElement("div");
  backdrop.className = "modal-backdrop";
  backdrop.innerHTML = h`
    <div class="modal">
      <h3>Editar deck</h3>
      <div style="margin-bottom:12px">
        <label style="font-size:12px;color:var(--text-dim)">Comandante *</label>
        <input type="text" id="ed-commander" value="${deck.commander_name}" autocomplete="off"
          style="width:100%;margin-top:5px;padding:9px 11px;border-radius:8px;border:1px solid var(--border-light);background:var(--bg-card);color:var(--text)">
        <div id="ed-suggestions"></div>
      </div>
      <div style="margin-bottom:12px">
        <label style="font-size:12px;color:var(--text-dim)">Comandante parceiro (opcional)</label>
        <input type="text" id="ed-commander-2" value="${deck.commander_name_2 || ""}" placeholder="Deixe em branco para remover o parceiro" autocomplete="off"
          style="width:100%;margin-top:5px;padding:9px 11px;border-radius:8px;border:1px solid var(--border-light);background:var(--bg-card);color:var(--text)">
        <div id="ed-suggestions-2"></div>
      </div>
      <div style="margin-bottom:12px">
        <label style="font-size:12px;color:var(--text-dim)">Nome do deck *</label>
        <input type="text" id="ed-name" value="${deck.name}"
          style="width:100%;margin-top:5px;padding:9px 11px;border-radius:8px;border:1px solid var(--border-light);background:var(--bg-card);color:var(--text)">
      </div>
      <div style="margin-bottom:12px">
        <label style="font-size:12px;color:var(--text-dim)">Estratégia / filosofia</label>
        <textarea id="ed-philosophy" rows="3" style="width:100%;margin-top:5px;padding:9px 11px;border-radius:8px;border:1px solid var(--border-light);background:var(--bg-card);color:var(--text);font-family:inherit">${deck.philosophy || ""}</textarea>
      </div>
      <div>
        <label style="font-size:12px;color:var(--text-dim)">Tags (separadas por vírgula)</label>
        <input type="text" id="ed-tags" value="${deck.tags || ""}" placeholder="Ex: Competitivo, Orçamento baixo, cEDH"
          style="width:100%;margin-top:5px;padding:9px 11px;border-radius:8px;border:1px solid var(--border-light);background:var(--bg-card);color:var(--text)">
      </div>
      <p style="margin:10px 0 0;font-size:11.5px;color:var(--text-faint)">Trocar um comandante aqui remove a carta antiga do deck (a menos que ela também esteja na lista como carta comum) e adiciona a nova como comandante, 1 cópia.</p>
      <div id="ed-error" style="color:var(--bad);font-size:12px;margin-top:10px;display:none">Preencha comandante e nome do deck.</div>
      <div style="display:flex;gap:10px;margin-top:18px;justify-content:flex-end">
        <button class="btn secondary" id="ed-cancel">Cancelar</button>
        <button class="btn" id="ed-save">Salvar</button>
      </div>
    </div>`;
  document.body.appendChild(backdrop);
  backdrop.addEventListener("click", (e) => { if (e.target === backdrop) backdrop.remove(); });

  function wireAutocomplete(input, suggestEl) {
    let debounce;
    input.addEventListener("input", () => {
      clearTimeout(debounce);
      const q = input.value.trim();
      if (q.length < 2) { suggestEl.innerHTML = ""; return; }
      debounce = setTimeout(async () => {
        const results = await api.searchCards(q, 6);
        suggestEl.innerHTML = results
          .map((c) => h`<div class="card-row" data-pick="${c.name}" style="cursor:pointer"><span class="name">${c.name}</span><span class="cost">${manaCostHtml(c.mana_cost)}</span></div>`)
          .join("");
        suggestEl.querySelectorAll("[data-pick]").forEach((el) =>
          el.addEventListener("click", () => {
            input.value = el.dataset.pick;
            suggestEl.innerHTML = "";
          })
        );
      }, 250);
    });
  }
  const commanderInput = backdrop.querySelector("#ed-commander");
  const commander2Input = backdrop.querySelector("#ed-commander-2");
  wireAutocomplete(commanderInput, backdrop.querySelector("#ed-suggestions"));
  wireAutocomplete(commander2Input, backdrop.querySelector("#ed-suggestions-2"));

  backdrop.querySelector("#ed-cancel").addEventListener("click", () => backdrop.remove());
  backdrop.querySelector("#ed-save").addEventListener("click", async () => {
    const commander_name = commanderInput.value.trim();
    const commander_name_2 = commander2Input.value.trim() || null;
    const name = backdrop.querySelector("#ed-name").value.trim();
    const errEl = backdrop.querySelector("#ed-error");
    if (!commander_name || !name) {
      errEl.textContent = "Preencha comandante e nome do deck.";
      errEl.style.display = "block";
      return;
    }
    const philosophy = backdrop.querySelector("#ed-philosophy").value.trim() || null;
    const tags = backdrop.querySelector("#ed-tags").value.trim() || null;
    const saveBtn = backdrop.querySelector("#ed-save");
    saveBtn.disabled = true;
    try {
      await api.updateDeck(deck.id, { name, commander_name, commander_name_2, philosophy, tags });
      backdrop.remove();
      onSaved?.();
    } catch (err) {
      saveBtn.disabled = false;
      errEl.textContent = `Falhou: ${err.message}`;
      errEl.style.display = "block";
    }
  });
}

async function openImportDeckModal(deckId, onImported) {
  const backdrop = document.createElement("div");
  backdrop.className = "modal-backdrop";
  backdrop.innerHTML = h`
    <div class="modal" style="max-width:560px">
      <h3>Importar decklist</h3>
      <p style="font-size:12.5px;color:var(--text-dim);line-height:1.5;margin-top:-6px">
        Cole uma lista (Moxfield, Archidekt, ou texto simples — "1 Nome da Carta" por linha) ou carregue um arquivo .txt/.csv.
        Nada é adicionado ao deck até você confirmar a pré-visualização abaixo.
      </p>
      <textarea id="im-text" rows="6" placeholder="1 Sol Ring&#10;1 Arcane Signet&#10;1 Command Tower&#10;..."
        style="width:100%;margin-top:6px;padding:9px 11px;border-radius:8px;border:1px solid var(--border-light);background:var(--bg-card);color:var(--text);font-family:monospace;font-size:12.5px"></textarea>
      <div style="display:flex;gap:10px;margin-top:10px;align-items:center;flex-wrap:wrap">
        <!-- The native file input is styled by the OS and can't be themed, so it's hidden and
             driven by a real app button. Picking a file fills the textarea above, which keeps a
             single source of truth: whatever you can see is exactly what gets imported. -->
        <input type="file" id="im-file" accept=".txt,.csv" style="display:none">
        <button class="btn secondary small" id="im-file-btn">Carregar arquivo (.txt/.csv)</button>
        <span id="im-file-name" style="font-size:12px;color:var(--text-faint)"></span>
      </div>
      <div id="im-results" style="margin-top:16px"></div>
      <div id="im-mode-wrap" style="display:none;margin-top:14px">
        <label style="font-size:12px;color:var(--text-dim);display:block;margin-bottom:6px">O que fazer com as cartas (comuns) já no deck?</label>
        <label style="display:flex;align-items:center;gap:6px;font-size:13px;margin-bottom:4px;cursor:pointer"><input type="radio" name="im-mode" value="merge" checked> Mesclar (soma às cartas existentes)</label>
        <label style="display:flex;align-items:center;gap:6px;font-size:13px;cursor:pointer"><input type="radio" name="im-mode" value="replace"> Substituir (remove as cartas não-comandante atuais antes de importar)</label>
      </div>
      <div id="im-error" style="color:var(--bad);font-size:12px;margin-top:10px;display:none"></div>
      <div style="display:flex;gap:10px;margin-top:18px;justify-content:flex-end">
        <button class="btn secondary" id="im-cancel">Cancelar</button>
        <button class="btn" id="im-preview-text">Pré-visualizar</button>
        <button class="btn" id="im-confirm" style="display:none">Confirmar importação</button>
      </div>
    </div>`;
  document.body.appendChild(backdrop);
  backdrop.addEventListener("click", (e) => { if (e.target === backdrop) backdrop.remove(); });

  let previewData = null;

  function renderPreview(data) {
    previewData = data;
    const resultsEl = backdrop.querySelector("#im-results");
    const matchedRows = data.matched
      .map(
        (c) => h`
        <div class="card-row">
          <span class="qty">${c.quantity}x</span>
          <span class="name">${c.name}</span>
          ${c.match_type && !c.match_type.startsWith("exata") ? `<span class="shared-tag">aproximado de "${c.requested_name}"</span>` : ""}
          <span class="cost">${manaCostHtml(c.mana_cost)}</span>
        </div>`
      )
      .join("");
    const notFoundRows = data.not_found
      .map(
        (c) => h`
        <div class="card-row">
          <span class="qty">${c.quantity}x</span>
          <span class="name" style="color:var(--bad)">${c.requested_name}</span>
          <span class="shared-tag" style="background:rgba(217,85,107,.15);color:var(--bad)">não encontrada</span>
        </div>`
      )
      .join("");

    resultsEl.innerHTML = h`
      <div class="sidebar-panel">
        <h3>${data.matched.length} reconhecida${data.matched.length === 1 ? "" : "s"} de ${data.total_lines} linha${data.total_lines === 1 ? "" : "s"}</h3>
        ${matchedRows || '<div class="empty-state" style="padding:10px">Nenhuma carta reconhecida.</div>'}
        ${notFoundRows ? `<h3 style="margin-top:14px;color:var(--bad)">Não encontradas (${data.not_found.length}) — confira o nome e tente de novo</h3>${notFoundRows}` : ""}
      </div>`;
    backdrop.querySelector("#im-mode-wrap").style.display = data.matched.length ? "block" : "none";
    backdrop.querySelector("#im-confirm").style.display = data.matched.length ? "inline-block" : "none";
    // Once there's something to confirm, "Pré-visualizar" would be a second primary button
    // competing with it; editing the text brings it back via resetPreview().
    backdrop.querySelector("#im-preview-text").style.display = data.matched.length ? "none" : "inline-block";
  }

  const textEl = backdrop.querySelector("#im-text");
  const errEl = backdrop.querySelector("#im-error");
  const previewBtn = backdrop.querySelector("#im-preview-text");
  const fileEl = backdrop.querySelector("#im-file");

  function showError(msg) {
    errEl.textContent = msg;
    errEl.style.display = "block";
  }

  /** Drops a stale preview whenever the text changes, so "Confirmar" can never import
      something other than what's on screen. */
  function resetPreview() {
    previewData = null;
    backdrop.querySelector("#im-results").innerHTML = "";
    backdrop.querySelector("#im-mode-wrap").style.display = "none";
    backdrop.querySelector("#im-confirm").style.display = "none";
    previewBtn.style.display = "inline-block";
    errEl.style.display = "none";
  }

  async function runPreview() {
    const text = textEl.value;
    errEl.style.display = "none";
    if (!text.trim()) {
      showError("Cole ou digite uma decklist antes de pré-visualizar.");
      textEl.focus();
      return;
    }
    previewBtn.disabled = true;
    previewBtn.textContent = "Lendo…";
    try {
      renderPreview(await api.importPreviewText(deckId, text));
    } catch (err) {
      showError(err.message);
    } finally {
      previewBtn.disabled = false;
      previewBtn.textContent = "Pré-visualizar";
    }
  }

  previewBtn.addEventListener("click", runPreview);
  textEl.addEventListener("input", resetPreview);

  backdrop.querySelector("#im-file-btn").addEventListener("click", () => fileEl.click());

  // The file is read here in the browser and dropped into the textarea rather than uploaded,
  // so the user sees and can edit exactly what will be imported — and both paths share one
  // preview endpoint.
  fileEl.addEventListener("change", () => {
    const file = fileEl.files[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = () => {
      resetPreview();
      textEl.value = String(reader.result || "");
      backdrop.querySelector("#im-file-name").textContent = file.name;
      runPreview();
    };
    reader.onerror = () => showError(`Não foi possível ler "${file.name}".`);
    reader.readAsText(file);
    fileEl.value = ""; // let the same file be picked again after an edit
  });

  backdrop.querySelector("#im-cancel").addEventListener("click", () => backdrop.remove());
  backdrop.querySelector("#im-confirm").addEventListener("click", async () => {
    if (!previewData?.matched?.length) return;
    const mode = backdrop.querySelector('input[name="im-mode"]:checked').value;
    const confirmBtn = backdrop.querySelector("#im-confirm");
    const errEl = backdrop.querySelector("#im-error");
    confirmBtn.disabled = true;
    confirmBtn.textContent = "Importando…";
    try {
      const cards = previewData.matched.map((c) => ({ card_name: c.name, quantity: c.quantity }));
      await api.importCommit(deckId, cards, mode);
      backdrop.remove();
      onImported?.();
    } catch (err) {
      confirmBtn.disabled = false;
      confirmBtn.textContent = "Confirmar importação";
      errEl.textContent = err.message;
      errEl.style.display = "block";
    }
  });
}

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
const FILTER_COLORS = { W: "#fffbd5", U: "#aae0fa", B: "#cbc2bf", R: "#f9aa8f", G: "#9bd3ae", C: "#8b8398" };

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
// Deliberately its own palette rather than reusing FILTER_COLORS (the mana-pip colors): those
// pastels are tuned for a black glyph drawn on top, but as flat, unlabeled chart segments they
// read poorly — black in particular ("B") all but disappears against the dark page background.
// Here B gets a proper dark charcoal and C a lighter neutral, so every segment stays legible.
const CURVE_COLORS = { W: "#f0e6b0", U: "#6ec3f5", B: "#4a4655", R: "#ef8c68", G: "#7fc98a", M: "#d9b45c", C: "#a8a4c0" };

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

async function renderDeckDetail([idStr]) {
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

  const overage = deck.total_cards - 100;
  let overageWarning = "";
  if (overage === 0) {
    overageWarning = `<div class="overage-warning">Deck completo (100/100). Adicionar outra carta vai deixar 101 — remova 1 antes ou depois.</div>`;
  } else if (overage > 0) {
    overageWarning = `<div class="overage-warning bad">Deck com ${overage} carta${overage > 1 ? "s" : ""} além do limite — remova ${overage} antes de continuar.</div>`;
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

/// Banner above the card list summarising what this deck would cost you to actually build:
/// how many cards you don't own (the shopping list) and how many you'd have to pull out of
/// another deck. Silent when the deck is fully owned and free — nothing to warn about.
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

/// Per-card ownership, as computed by the backend (deck.ownership). Returns null for cards whose
/// status the backend didn't report (e.g. the commander, or an older backend that predates the
/// field) so nothing is flagged on a guess.
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

/// Small corner dot for the image-based views, where a text pill wouldn't fit over the art.
function ownDotHtml(cardName, ownership) {
  const t = deckOwnTag(cardName, ownership);
  return t ? `<span class="own-dot ${t.cls}" title="${t.title}"></span>` : "";
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
            // Widely-reused staples (basic lands, ramp, etc.) can be "shared" with a dozen other
            // decks — without a cap, that whole comma-joined list rendered on one unbreakable
            // line and blew the row (and the whole page) out past the viewport. Truncate visibly,
            // keep the full list in the title tooltip.
            const sharedList = (c.shared_with || []).map((s) => s.deck).join(", ");
            const sharedTag = c.shared_with?.length
              ? `<span class="shared-tag" title="também em ${sharedList}">também em ${sharedList}</span>`
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

let collectionFilter = "all";

async function renderCollection() {
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
    const items = await api.collection(collectionFilter, q);
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
            ${c.image_uri ? `<img src="${c.image_uri}" loading="lazy" decoding="async" alt="${c.card_name}">` : `<div class="no-image">${c.card_name}</div>`}
            <span class="qty-badge">${c.total_quantity}x</span>
            ${free && isAllocated ? `<span class="free-badge">${free} livre${free > 1 ? "s" : ""}</span>` : ""}
            ${isAllocated ? `<div class="deck-badge">${deckLabel}</div>` : ""}
          </div>`;
      })
      .join("") || `<div class="empty-state">Nada encontrado.</div>`;
  }

  // One delegated listener on the grid instead of one per tile — the collection renders hundreds
  // of cards, and load() re-runs on every filter/search change, so per-tile binding meant
  // re-attaching hundreds of listeners each time.
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
  let searchDebounce;
  document.getElementById("coll-search").addEventListener("input", () => {
    clearTimeout(searchDebounce);
    searchDebounce = setTimeout(load, 250);
  });
  document.getElementById("add-card-btn").addEventListener("click", () =>
    openAddCardModal({ onSaved: load })
  );
  load();
}

// --------------------------------------------------------------- games ----

async function renderGames() {
  const [decks, games, stats] = await Promise.all([api.decks(), api.games(), api.gamesStats()]);

  mainEl.innerHTML = h`
    <div class="page-header">
      <div><h1>Partidas</h1><p>Registre vitórias, derrotas e o que mais apareceu — pra entender o motor de cada deck.</p></div>
      <button class="btn" id="new-game-btn" ${decks.length ? "" : "disabled title=\"Crie um deck primeiro\""}>+ Registrar partida</button>
    </div>
    <div class="stat-grid">
      <div class="stat-card"><div class="label">Taxa de vitória</div><div class="value">${stats.win_rate ?? "—"}${stats.win_rate != null ? "%" : ""}</div><div class="sub">${stats.total_games} partidas</div></div>
      <div class="stat-card"><div class="label">Vitórias</div><div class="value" style="color:var(--good)">${stats.wins}</div></div>
      <div class="stat-card"><div class="label">Derrotas</div><div class="value" style="color:var(--bad)">${stats.losses}</div></div>
    </div>
    <div class="page-header" style="margin-top:8px"><h1 style="font-size:16px">Cartas que mais se destacaram</h1></div>
    <div style="margin-bottom:24px">
      ${(stats.top_highlight_cards || []).map((c) => `<span class="chip" style="margin:0 6px 6px 0;display:inline-block">${c.card_name} · ${c.n}x</span>`).join("") || '<div class="empty-state">Nenhum destaque registrado ainda.</div>'}
    </div>
    <div class="page-header"><h1 style="font-size:16px">Histórico</h1></div>
    <div id="games-list"></div>
  `;

  document.getElementById("games-list").innerHTML = games
    .map((g) => h`
      <div class="game-row">
        <span class="result-pill ${g.result}">${g.result}</span>
        <b>${g.deck_name}</b>
        <span class="opp">vs ${g.opponents || "?"}</span>
        <span class="opp">${g.played_at}</span>
        ${g.notes ? `<div class="notes">${g.notes}</div>` : ""}
      </div>`)
    .join("") || `<div class="empty-state">${decks.length ? "Nenhuma partida registrada." : "Crie um deck primeiro (Meus Decks) para poder registrar partidas."}</div>`;

  if (decks.length) {
    document.getElementById("new-game-btn").addEventListener("click", () => openGameModal(decks));
  }
}

function openGameModal(decks) {
  const backdrop = document.createElement("div");
  backdrop.className = "modal-backdrop";
  backdrop.innerHTML = h`
    <div class="modal">
      <h3>Registrar partida</h3>
      <div class="form-grid">
        <div><label>Deck</label><select id="g-deck">${decks.map((d) => `<option value="${d.id}">${d.name}</option>`).join("")}</select></div>
        <div><label>Resultado</label><select id="g-result"><option value="vitoria">Vitória</option><option value="derrota">Derrota</option><option value="empate">Empate</option></select></div>
        <div><label>Data</label><input type="date" id="g-date" value="${new Date().toISOString().slice(0, 10)}"></div>
        <div><label>Turnos</label><input type="number" id="g-turns" min="1"></div>
      </div>
      <div style="margin-top:12px"><label style="font-size:12px;color:var(--text-dim)">Oponentes (comandantes)</label><input type="text" id="g-opponents" placeholder="Ex: Atraxa, Korvold" style="width:100%;margin-top:5px;padding:9px 11px;border-radius:8px;border:1px solid var(--border-light);background:var(--bg-card);color:var(--text)"></div>
      <div style="margin-top:12px"><label style="font-size:12px;color:var(--text-dim)">Cartas que se destacaram (separadas por vírgula)</label><input type="text" id="g-highlights" placeholder="Ex: Mindcrank, Blood Artist" style="width:100%;margin-top:5px;padding:9px 11px;border-radius:8px;border:1px solid var(--border-light);background:var(--bg-card);color:var(--text)"></div>
      <div style="margin-top:12px"><label style="font-size:12px;color:var(--text-dim)">Notas</label><textarea id="g-notes" rows="3" style="width:100%;margin-top:5px;padding:9px 11px;border-radius:8px;border:1px solid var(--border-light);background:var(--bg-card);color:var(--text);font-family:inherit"></textarea></div>
      <div style="display:flex;gap:10px;margin-top:18px;justify-content:flex-end">
        <button class="btn secondary" id="g-cancel">Cancelar</button>
        <button class="btn" id="g-save">Salvar</button>
      </div>
    </div>`;
  document.body.appendChild(backdrop);
  backdrop.querySelector("#g-cancel").addEventListener("click", () => backdrop.remove());
  backdrop.querySelector("#g-save").addEventListener("click", async () => {
    const highlights = backdrop.querySelector("#g-highlights").value.split(",").map((s) => s.trim()).filter(Boolean);
    await api.addGame({
      deck_id: Number(backdrop.querySelector("#g-deck").value),
      result: backdrop.querySelector("#g-result").value,
      played_at: backdrop.querySelector("#g-date").value,
      turns: Number(backdrop.querySelector("#g-turns").value) || null,
      opponents: backdrop.querySelector("#g-opponents").value,
      notes: backdrop.querySelector("#g-notes").value,
      highlights,
    });
    backdrop.remove();
    renderGames();
  });
}

// ------------------------------------------------------------ add card ----

async function openAddCardModal({ onSaved } = {}) {
  const decks = await api.decks();
  const deckOptions = h`
    <option value="">Nenhum (fica livre na coleção)</option>
    ${decks.map((d) => `<option value="${d.id}">${d.name}</option>`).join("")}`;

  const backdrop = document.createElement("div");
  backdrop.className = "modal-backdrop";
  backdrop.innerHTML = h`
    <div class="modal">
      <h3>Adicionar carta</h3>
      <div class="mode-toggle" style="margin-bottom:16px">
        <span class="chip active" data-ac-mode="single">Uma carta</span>
        <span class="chip" data-ac-mode="list">Adicionar por lista</span>
      </div>

      <div id="ac-single-wrap">
        <div style="margin-bottom:12px">
          <label style="font-size:12px;color:var(--text-dim)">Nome da carta *</label>
          <input type="text" id="ac-name" placeholder="Nome (PT ou EN)…" autocomplete="off"
            style="width:100%;margin-top:5px;padding:9px 11px;border-radius:8px;border:1px solid var(--border-light);background:var(--bg-card);color:var(--text)">
          <div id="ac-suggestions"></div>
        </div>
        <div class="form-grid">
          <div><label>Edição (set)</label><input type="text" id="ac-set" placeholder="Ex: znr"></div>
          <div><label>Artista</label><input type="text" id="ac-artist" placeholder="Nome do artista"></div>
          <div><label>Idioma</label><select id="ac-lang"><option value="en">Inglês</option><option value="pt">Português</option></select></div>
          <div><label>Quantidade</label><input type="number" id="ac-qty" min="1" value="1"></div>
        </div>
        <div style="margin-top:12px">
          <label style="font-size:12px;color:var(--text-dim)">Alocar a um deck (opcional)</label>
          <select id="ac-deck" style="width:100%;margin-top:5px;padding:9px 11px;border-radius:8px;border:1px solid var(--border-light);background:var(--bg-card);color:var(--text)">
            ${deckOptions}
          </select>
        </div>
        <div style="margin-top:12px">
          <label style="font-size:12px;color:var(--text-dim)">Notas</label>
          <textarea id="ac-notes" rows="2" style="width:100%;margin-top:5px;padding:9px 11px;border-radius:8px;border:1px solid var(--border-light);background:var(--bg-card);color:var(--text);font-family:inherit"></textarea>
        </div>
      </div>

      <div id="ac-list-wrap" style="display:none">
        <label style="font-size:12px;color:var(--text-dim)">Nomes das cartas — um por linha *</label>
        <textarea id="ac-list-text" rows="8" placeholder="Sol Ring&#10;Cultivate&#10;Demonic Tutor…"
          style="width:100%;margin-top:5px;padding:9px 11px;border-radius:8px;border:1px solid var(--border-light);background:var(--bg-card);color:var(--text);font-family:inherit"></textarea>
        <p style="font-size:12px;color:var(--text-dim);margin:6px 0 0">Cada linha vira 1 unidade, sem edição/artista específicos — pra registrar rápido. Ajuste os detalhes depois na Coleção se precisar.</p>
        <div style="margin-top:12px">
          <label style="font-size:12px;color:var(--text-dim)">Alocar todas a um deck (opcional)</label>
          <select id="ac-list-deck" style="width:100%;margin-top:5px;padding:9px 11px;border-radius:8px;border:1px solid var(--border-light);background:var(--bg-card);color:var(--text)">
            ${deckOptions}
          </select>
        </div>
        <div id="ac-list-status" style="margin-top:12px;font-size:13px"></div>
      </div>

      <div id="ac-error" style="color:var(--bad);font-size:12px;margin-top:10px;display:none">Preencha o nome da carta.</div>
      <div style="display:flex;gap:10px;margin-top:18px;justify-content:flex-end">
        <button class="btn secondary" id="ac-cancel">Cancelar</button>
        <button class="btn" id="ac-save">Salvar</button>
      </div>
    </div>`;
  document.body.appendChild(backdrop);
  backdrop.addEventListener("click", (e) => { if (e.target === backdrop) backdrop.remove(); });

  let mode = "single";
  const singleWrap = backdrop.querySelector("#ac-single-wrap");
  const listWrap = backdrop.querySelector("#ac-list-wrap");
  const errorEl = backdrop.querySelector("#ac-error");
  backdrop.querySelectorAll("[data-ac-mode]").forEach((chip) =>
    chip.addEventListener("click", () => {
      mode = chip.dataset.acMode;
      backdrop.querySelectorAll("[data-ac-mode]").forEach((c) => c.classList.toggle("active", c === chip));
      singleWrap.style.display = mode === "single" ? "block" : "none";
      listWrap.style.display = mode === "list" ? "block" : "none";
      errorEl.style.display = "none";
      if (mode === "single") backdrop.querySelector("#ac-name").focus();
      else backdrop.querySelector("#ac-list-text").focus();
    })
  );

  const nameInput = backdrop.querySelector("#ac-name");
  const suggestionsEl = backdrop.querySelector("#ac-suggestions");
  nameInput.focus();
  let debounce;
  nameInput.addEventListener("input", () => {
    clearTimeout(debounce);
    const q = nameInput.value.trim();
    if (q.length < 2) { suggestionsEl.innerHTML = ""; return; }
    debounce = setTimeout(async () => {
      const results = await api.searchCards(q, 6);
      suggestionsEl.innerHTML = results
        .map((c) => h`<div class="card-row" data-pick="${c.name}" style="cursor:pointer"><span class="name">${c.name}</span><span class="cost">${manaCostHtml(c.mana_cost)}</span></div>`)
        .join("");
      suggestionsEl.querySelectorAll("[data-pick]").forEach((el) =>
        el.addEventListener("click", () => {
          nameInput.value = el.dataset.pick;
          suggestionsEl.innerHTML = "";
        })
      );
    }, 250);
  });

  backdrop.querySelector("#ac-cancel").addEventListener("click", () => backdrop.remove());
  backdrop.querySelector("#ac-save").addEventListener("click", async () => {
    const saveBtn = backdrop.querySelector("#ac-save");

    if (mode === "single") {
      const card_name = nameInput.value.trim();
      if (!card_name) {
        errorEl.textContent = "Preencha o nome da carta.";
        errorEl.style.display = "block";
        nameInput.focus();
        return;
      }
      saveBtn.disabled = true;
      const deckVal = backdrop.querySelector("#ac-deck").value;
      await api.addCollection({
        card_name,
        set_code: backdrop.querySelector("#ac-set").value.trim() || null,
        artist: backdrop.querySelector("#ac-artist").value.trim() || null,
        lang: backdrop.querySelector("#ac-lang").value,
        quantity: Number(backdrop.querySelector("#ac-qty").value) || 1,
        notes: backdrop.querySelector("#ac-notes").value.trim() || null,
        deck_id: deckVal ? Number(deckVal) : null,
      });
      backdrop.remove();
      onSaved?.();
      return;
    }

    // list mode
    const listText = backdrop.querySelector("#ac-list-text");
    const statusEl = backdrop.querySelector("#ac-list-status");
    const text = listText.value.trim();
    if (!text) {
      errorEl.textContent = "Digite ao menos um nome de carta.";
      errorEl.style.display = "block";
      listText.focus();
      return;
    }
    errorEl.style.display = "none";
    saveBtn.disabled = true;
    statusEl.textContent = "Verificando…";
    const deckVal = backdrop.querySelector("#ac-list-deck").value;
    const deck_id = deckVal ? Number(deckVal) : null;

    const resolved = await api.bulkResolveCollection(text);
    const found = resolved.filter((r) => r.card_name);
    const missing = resolved.filter((r) => !r.card_name);

    if (found.length) {
      await Promise.all(found.map((r) =>
        api.addCollection({ card_name: r.card_name, lang: r.lang, oracle_id: r.oracle_id, quantity: 1, deck_id })
      ));
    }

    saveBtn.disabled = false;
    if (!missing.length) {
      backdrop.remove();
      onSaved?.();
      return;
    }
    // partial success — keep the modal open with only the failed lines, ready to fix and resend
    listText.value = missing.map((r) => r.input).join("\n");
    statusEl.innerHTML = `${found.length ? `<span style="color:var(--good)">${found.length} adicionada${found.length > 1 ? "s" : ""}.</span> ` : ""}<span style="color:var(--bad)">${missing.length} não encontrada${missing.length > 1 ? "s" : ""} — corrija e clique em Salvar de novo:</span>`;
    if (found.length) onSaved?.();
  });
}

// ------------------------------------------------------------ card modal ----

/// Panel inside the card modal: how many physical copies the user owns, where they are, and a
/// way to add one more. Two ways in, because they answer different questions — "I bought another
/// one" needs nothing but a count, while cataloguing a specific printing needs set/artist/lang.
async function renderCopiesBox(backdrop, cardName, onCollectionChange) {
  const box = backdrop.querySelector("#copies-box");
  if (!box) return;
  box.innerHTML = `<div class="sidebar-panel"><div class="empty-state" style="padding:10px">Carregando cópias…</div></div>`;

  let copies;
  try {
    copies = await api.cardCopies(cardName);
  } catch {
    box.innerHTML = "";
    return;
  }

  const where = [
    copies.free ? `<span class="copy-dot dot-free"></span>${copies.free} livre${copies.free > 1 ? "s" : ""}` : null,
    ...copies.decks.map((d) => `<span class="copy-dot dot-deck"></span>${d.copies} em <b>${d.deck_name}</b>`),
  ].filter(Boolean);

  box.innerHTML = h`
    <div class="sidebar-panel">
      <h3>Suas cópias
        <span class="own-tag ${copies.total ? "tag-in-deck" : "tag-missing"}" style="margin-left:6px">${copies.total}x</span>
      </h3>
      <div class="own-list" style="display:block;margin-top:2px">
        ${copies.total ? where.join(" &nbsp;·&nbsp; ") : "Você ainda não tem nenhuma cópia desta carta."}
      </div>

      <div style="display:flex;gap:8px;margin-top:12px;align-items:center;flex-wrap:wrap">
        <button class="btn small" id="cp-add-one">+ 1 unidade</button>
        <button class="btn secondary small" id="cp-add-detailed">Adicionar com detalhes…</button>
        <span id="cp-msg" style="font-size:12px;color:var(--text-faint)"></span>
      </div>

      <div id="cp-detail" style="display:none;margin-top:12px">
        <div class="form-grid">
          <div><label>Edição (set)</label><input type="text" id="cp-set" placeholder="Ex: znr" maxlength="10"></div>
          <div><label>Artista</label><input type="text" id="cp-artist" placeholder="Nome do artista"></div>
          <div><label>Idioma</label><select id="cp-lang"><option value="en">Inglês</option><option value="pt">Português</option></select></div>
          <div><label>Quantidade</label><input type="number" id="cp-qty" min="1" value="1"></div>
        </div>
        <div style="margin-top:12px">
          <label style="font-size:12px;color:var(--text-dim)">Notas</label>
          <input type="text" id="cp-notes" placeholder="Ex: foil, comprada na loja X"
            style="width:100%;margin-top:5px;padding:9px 11px;border-radius:8px;border:1px solid var(--border-light);background:var(--bg-card);color:var(--text)">
        </div>
        <div style="display:flex;gap:8px;margin-top:12px;justify-content:flex-end">
          <button class="btn secondary small" id="cp-cancel">Cancelar</button>
          <button class="btn small" id="cp-save">Adicionar à coleção</button>
        </div>
      </div>
    </div>`;

  const msg = box.querySelector("#cp-msg");
  const detail = box.querySelector("#cp-detail");

  // New copies land unallocated: buying a card doesn't put it in a deck, and the deck views
  // decide what's available by reading the free count.
  async function addCopy(payload, btn) {
    const label = btn.textContent;
    btn.disabled = true;
    btn.textContent = "Adicionando…";
    try {
      await api.addCollection({ card_name: cardName, ...payload });
      await renderCopiesBox(backdrop, cardName, onCollectionChange);
      onCollectionChange?.();
    } catch (err) {
      msg.textContent = err.message;
      msg.style.color = "var(--bad)";
      btn.disabled = false;
      btn.textContent = label;
    }
  }

  box.querySelector("#cp-add-one").addEventListener("click", (e) =>
    addCopy({ quantity: 1, notes: "Unidade avulsa" }, e.currentTarget)
  );
  box.querySelector("#cp-add-detailed").addEventListener("click", () => {
    detail.style.display = detail.style.display === "none" ? "block" : "none";
  });
  box.querySelector("#cp-cancel").addEventListener("click", () => { detail.style.display = "none"; });
  box.querySelector("#cp-save").addEventListener("click", (e) =>
    addCopy(
      {
        set_code: box.querySelector("#cp-set").value.trim().toUpperCase() || null,
        artist: box.querySelector("#cp-artist").value.trim() || null,
        lang: box.querySelector("#cp-lang").value,
        quantity: Math.max(1, parseInt(box.querySelector("#cp-qty").value, 10) || 1),
        notes: box.querySelector("#cp-notes").value.trim() || null,
      },
      e.currentTarget
    )
  );
}

export async function showCardModal(name, onCollectionChange) {
  const backdrop = document.createElement("div");
  backdrop.className = "modal-backdrop";
  backdrop.innerHTML = `<div class="modal"><div class="empty-state">Carregando…</div></div>`;
  document.body.appendChild(backdrop);
  backdrop.addEventListener("click", (e) => { if (e.target === backdrop) backdrop.remove(); });

  try {
    const c = await api.card(name);
    const ptNames = (c.pt_names || []).map((p) => p.printed_name).filter((v, i, a) => a.indexOf(v) === i);
    backdrop.querySelector(".modal").innerHTML = h`
      <div style="display:flex;gap:16px;flex-wrap:wrap;align-items:flex-start">
        ${c.image_uri ? `<img src="${c.image_uri}" style="width:200px;height:auto;border-radius:12px;flex-shrink:0" alt="${c.name}">` : ""}
        <!-- 200px art + this minimum has to stay under the modal's inner width, or the flex wraps
             and the art jumps above the text the moment a scrollbar appears. The art keeps its
             own width and aspect ratio either way — it is never squeezed to fit. -->
        <div style="flex:1;min-width:170px">
          <h3 style="margin-bottom:2px">${c.name}</h3>
          <div style="margin-bottom:6px">${manaCostHtml(c.mana_cost)}</div>
          <div style="font-size:12px;color:var(--text-dim);margin-bottom:10px">${c.type_line || ""}</div>
          <div class="oracle">${(c.oracle_text || "").replace(/\n/g, "<br>")}</div>
          ${ptNames.length ? `<div style="margin-top:10px;font-size:12px;color:var(--text-faint)">PT oficial: ${ptNames.join(" / ")}</div>` : ""}
          <div style="margin-top:10px;display:flex;gap:14px;font-size:12px;color:var(--text-dim)">
            <span>Preço: ${priceLabel(c.price_usd)}</span>
            <span>Raridade: ${c.rarity || "?"}</span>
            ${c.edhrec_rank ? `<span>EDHREC #${c.edhrec_rank.toLocaleString("pt-BR")}</span>` : ""}
          </div>
        </div>
      </div>
      <div id="copies-box" style="margin-top:16px"></div>
      <div class="qa-box">
        <label style="font-size:12px;color:var(--text-dim)">Perguntar sobre esta carta (busca no texto/oracle — não é IA generativa)</label>
        <input type="text" id="qa-input" placeholder="Ex: sinergia, cemitério, sacrifício…" style="margin-top:6px">
        <div id="qa-answer"></div>
      </div>
      <div style="margin-top:16px;text-align:right"><button class="btn secondary" id="modal-close">Fechar</button></div>
    `;
    renderCopiesBox(backdrop, c.name, onCollectionChange);
    backdrop.querySelector("#modal-close").addEventListener("click", () => backdrop.remove());
    backdrop.querySelector("#qa-input").addEventListener("keydown", (e) => {
      if (e.key !== "Enter") return;
      const q = e.target.value.toLowerCase();
      const text = (c.oracle_text || "").toLowerCase();
      const hit = text.includes(q);
      document.getElementById("qa-answer").innerHTML = `<div class="qa-answer">${
        hit
          ? `Sim — o texto da carta menciona isso: "…${highlightMatch(c.oracle_text, q)}…"`
          : `O texto da carta não menciona "${q}" diretamente. Tipo: ${c.type_line}. Para regras específicas, ver a Comprehensive Rules no vault.`
      }</div>`;
    });
  } catch (err) {
    backdrop.querySelector(".modal").innerHTML = `<div class="empty-state">Carta não encontrada.</div><div style="text-align:right"><button class="btn secondary" id="modal-close">Fechar</button></div>`;
    backdrop.querySelector("#modal-close").addEventListener("click", () => backdrop.remove());
  }
}

function highlightMatch(text, q) {
  const idx = text.toLowerCase().indexOf(q);
  if (idx === -1) return text.slice(0, 80);
  return text.slice(Math.max(0, idx - 30), idx + q.length + 30);
}

navigate();
renderSidebarDataPanel();
