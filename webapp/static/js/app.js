import { api } from "./api.js?v=17";
import { renderScanner } from "./scanner.js?v=17";
import { activityIcon, manaCostHtml } from "./icons.js?v=17";

const mainEl = document.getElementById("main");
const navItems = document.querySelectorAll(".nav-item");

const routes = {
  dashboard: renderDashboard,
  decks: renderDecksList,
  deck: renderDeckDetail,
  collection: renderCollection,
  scanner: (params) => renderScanner(mainEl, params, { showCardModal }),
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
}

navItems.forEach((el) => {
  el.addEventListener("click", () => {
    location.hash = "#" + el.dataset.route;
  });
});
window.addEventListener("hashchange", navigate);

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
      <div><h1>Visão Geral</h1><p>Seu laboratório de coleção e decks — tudo lido do banco local.</p></div>
      <div style="display:flex;gap:10px">
        <button class="btn" id="add-card-btn">+ Adicionar Carta</button>
        <button class="btn secondary" id="dash-new-deck-btn">Novo Deck</button>
      </div>
    </div>
    <div class="stat-grid">
      <div class="stat-card"><div class="label">Decks montados</div><div class="value">${decks.length}</div><div class="sub">${totalCards} cartas ao todo</div></div>
      <div class="stat-card"><div class="label">Cartas na coleção</div><div class="value">${collectionTotal.total_units}</div><div class="sub">${collectionTotal.distinct_cards} nomes distintos, com repetidas</div></div>
      <div class="stat-card"><div class="label">Cartas livres</div><div class="value">${freeCollection.length}</div><div class="sub">fora de deck montado</div></div>
      <div class="stat-card"><div class="label">Partidas registradas</div><div class="value">${totalGames}</div><div class="sub">${totalWins} vitórias</div></div>
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
      <div class="sidebar-data-info warn">Base de cartas ainda não configurada.</div>
      <button class="btn small" id="update-data-btn" style="width:100%;margin-top:6px">Baixar base de dados agora</button>
      <div id="update-data-status"></div>`;
  }
  return h`
    <div class="sidebar-data-info">Dados: ${info.cards.toLocaleString("pt-BR")} cartas + EDHREC cacheado${info.built_at ? ` · atualizado em ${formatBuiltAt(info.built_at)}` : ""}. Só preço ao vivo precisa de rede.</div>
    <button class="btn small secondary" id="update-data-btn" style="width:100%;margin-top:6px">Atualizar base de dados</button>
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

function deckCardHtml(d) {
  const cls = d.total_cards === 100 ? "ok" : "bad";
  return h`
    <div class="deck-card" data-deck-link="${d.id}">
      <div class="deck-card-thumb">
        ${d.commander_image ? `<img src="${d.commander_image}" alt="${d.commander_name}">` : `<div class="no-image">${d.commander_name}</div>`}
      </div>
      <div class="deck-card-body">
        <h3>${d.name}</h3>
        <div class="philosophy">${d.philosophy || ""}</div>
        <div class="meta-row">
          <span class="count-pill ${cls}">${d.total_cards}/100</span>
          <span class="wl">${d.wins}<b class="win">V</b> · ${d.losses}<b class="loss">D</b></span>
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
      </div>

      <div>
        <label style="font-size:12px;color:var(--text-dim)" id="nd-philosophy-label">Estratégia / filosofia (opcional)</label>
        <textarea id="nd-philosophy" rows="3" style="width:100%;margin-top:5px;padding:9px 11px;border-radius:8px;border:1px solid var(--border-light);background:var(--bg-card);color:var(--text);font-family:inherit"></textarea>
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
    saveBtn.disabled = true;
    commanderInput.disabled = true;

    if (!autoToggle.checked) {
      const { id } = await api.createDeck({ name, commander_name, commander_name_2, philosophy });
      backdrop.remove();
      location.hash = `#deck/${id}`;
      return;
    }

    const bracket = Number(backdrop.querySelector("#nd-bracket").value);
    const statusEl = backdrop.querySelector("#nd-build-status");
    try {
      await api.startAutoBuildDeck({ name, commander_name, bracket, philosophy });
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
      <div>
        <label style="font-size:12px;color:var(--text-dim)">Estratégia / filosofia</label>
        <textarea id="ed-philosophy" rows="3" style="width:100%;margin-top:5px;padding:9px 11px;border-radius:8px;border:1px solid var(--border-light);background:var(--bg-card);color:var(--text);font-family:inherit">${deck.philosophy || ""}</textarea>
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
    const saveBtn = backdrop.querySelector("#ed-save");
    saveBtn.disabled = true;
    try {
      await api.updateDeck(deck.id, { name, commander_name, commander_name_2, philosophy });
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
        Cole uma lista (Moxfield, Archidekt, ou texto simples — "1 Nome da Carta" por linha) ou envie um arquivo .txt/.csv/.xlsx.
        Nada é adicionado ao deck até você confirmar a pré-visualização abaixo.
      </p>
      <textarea id="im-text" rows="6" placeholder="1 Sol Ring&#10;1 Arcane Signet&#10;1 Command Tower&#10;..."
        style="width:100%;margin-top:6px;padding:9px 11px;border-radius:8px;border:1px solid var(--border-light);background:var(--bg-card);color:var(--text);font-family:monospace;font-size:12.5px"></textarea>
      <div style="display:flex;gap:10px;margin-top:10px;align-items:center;flex-wrap:wrap">
        <button class="btn secondary small" id="im-preview-text">Pré-visualizar texto colado</button>
        <span style="font-size:12px;color:var(--text-faint)">ou envie um arquivo:</span>
        <input type="file" id="im-file" accept=".txt,.csv,.xlsx" style="font-size:12px;color:var(--text-dim)">
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
  }

  backdrop.querySelector("#im-preview-text").addEventListener("click", async () => {
    const text = backdrop.querySelector("#im-text").value;
    const errEl = backdrop.querySelector("#im-error");
    errEl.style.display = "none";
    if (!text.trim()) return;
    try {
      renderPreview(await api.importPreviewText(deckId, text));
    } catch (err) {
      errEl.textContent = err.message;
      errEl.style.display = "block";
    }
  });

  backdrop.querySelector("#im-file").addEventListener("change", async (e) => {
    const file = e.target.files[0];
    const errEl = backdrop.querySelector("#im-error");
    errEl.style.display = "none";
    if (!file) return;
    try {
      renderPreview(await api.importPreviewFile(deckId, file));
    } catch (err) {
      errEl.textContent = err.message;
      errEl.style.display = "block";
    }
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
const CATEGORY_ORDER = ["Comandante", "Land", "Creature", "Instant", "Sorcery", "Artifact", "Enchantment", "Planeswalker", "Outro"];
// Filterable by the type chips — the commander(s) are always shown regardless of filters, so this
// list intentionally excludes "Comandante": losing sight of your own commander via a filter would be confusing.
const FILTERABLE_TYPES = CATEGORY_ORDER.filter((c) => c !== "Comandante");
const CMC_BUCKETS = ["0", "1", "2", "3", "4", "5", "6+"];
const FILTER_COLORS = { W: "#fffbd5", U: "#aae0fa", B: "#cbc2bf", R: "#f9aa8f", G: "#9bd3ae", C: "#8b8398" };

let deckViewMode = "list"; // "list", "grid" (MTG Arena tiles), or "stack" (Moxfield-style overlap)

// Deck card filter/sort state — module-level so it survives view-mode switches and persists
// across decks in the same session (matches how deckViewMode already behaves).
let deckFilters = { q: "", types: new Set(), colors: new Set(), cmcs: new Set(), sort: "name" };

function deckFiltersActive() {
  return !!(deckFilters.q || deckFilters.types.size || deckFilters.colors.size || deckFilters.cmcs.size);
}

function cardCmcBucket(cmc) {
  const n = Math.floor(cmc || 0);
  return n >= 6 ? "6+" : String(n);
}

function cardMatchesFilters(c) {
  if (deckFilters.q && !c.card_name.toLowerCase().includes(deckFilters.q.toLowerCase())) return false;
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

function deckFilterBarHtml() {
  const typeChips = FILTERABLE_TYPES.map(
    (t) => `<span class="chip ${deckFilters.types.has(t) ? "active" : ""}" data-deck-type="${t}">${CATEGORY_LABELS[t]}</span>`
  ).join("");
  const colorChips = Object.entries(FILTER_COLORS)
    .map(([c, bg]) => {
      const dimmed = deckFilters.colors.size && !deckFilters.colors.has(c);
      const active = deckFilters.colors.has(c);
      return `<span class="chip color-chip ${active ? "active" : ""}" data-deck-color="${c}"
        style="background:${bg};color:#1a1a1a;opacity:${dimmed ? 0.35 : 1}">${c}</span>`;
    })
    .join("");
  const cmcChips = CMC_BUCKETS.map(
    (v) => `<span class="chip ${deckFilters.cmcs.has(v) ? "active" : ""}" data-deck-cmc="${v}">${v}</span>`
  ).join("");

  return h`
    <div class="filters-bar" id="deck-filters">
      <input type="text" id="deck-filter-q" placeholder="Buscar carta no deck…" value="${deckFilters.q}">
      <div class="filter-group"><span class="filter-group-label">Tipo</span>${typeChips}</div>
      <div class="filter-group"><span class="filter-group-label">Cor</span>${colorChips}</div>
      <div class="filter-group"><span class="filter-group-label">CMC</span>${cmcChips}</div>
      <select id="deck-sort">
        <option value="name" ${deckFilters.sort === "name" ? "selected" : ""}>Nome (A-Z)</option>
        <option value="cmc-asc" ${deckFilters.sort === "cmc-asc" ? "selected" : ""}>Custo de mana ↑</option>
        <option value="cmc-desc" ${deckFilters.sort === "cmc-desc" ? "selected" : ""}>Custo de mana ↓</option>
        <option value="price-desc" ${deckFilters.sort === "price-desc" ? "selected" : ""}>Preço ↓</option>
        <option value="qty-desc" ${deckFilters.sort === "qty-desc" ? "selected" : ""}>Quantidade ↓</option>
      </select>
      ${deckFiltersActive() ? `<button class="btn small secondary" id="deck-filter-clear">Limpar filtros</button>` : ""}
    </div>`;
}

function wireDeckFilterBar(deck) {
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
  document.getElementById("deck-sort").addEventListener("change", (e) => {
    deckFilters.sort = e.target.value;
    renderDeckCards(deck);
  });
  const clearBtn = document.getElementById("deck-filter-clear");
  if (clearBtn) {
    clearBtn.addEventListener("click", () => {
      deckFilters = { q: "", types: new Set(), colors: new Set(), cmcs: new Set(), sort: deckFilters.sort };
      refreshDeckFilterBar(deck);
    });
  }
}

/** Re-renders just the filter bar (to reflect new active/dimmed chip states) plus the card list below it. */
function refreshDeckFilterBar(deck) {
  document.getElementById("deck-filters").outerHTML = deckFilterBarHtml();
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

async function renderDeckDetail([idStr]) {
  const id = Number(idStr);
  const [deck, synergy, collectionAll] = await Promise.all([
    api.deck(id),
    api.deckSynergy(id).catch(() => ({ cached: false })),
    api.collection("all"),
  ]);
  const ownershipMap = buildOwnershipMap(collectionAll);

  const maxCmc = Math.max(1, ...Object.keys(deck.mana_curve).map(Number));
  const maxCount = Math.max(1, ...Object.values(deck.mana_curve));
  const curveBars = Array.from({ length: Math.min(maxCmc, 7) + 1 }, (_, i) => i)
    .map((cmc) => {
      const label = cmc === 7 ? "7+" : String(cmc);
      const count = cmc === 7
        ? Object.entries(deck.mana_curve).filter(([k]) => Number(k) >= 7).reduce((s, [, v]) => s + v, 0)
        : (deck.mana_curve[cmc] || 0);
      return { label, count };
    });

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
      <div><h1>${deck.name}</h1><p>${deck.philosophy || ""}</p></div>
      <div style="display:flex;align-items:center;gap:10px">
        <span class="count-pill ${deck.is_valid_100 ? "ok" : "bad"}" style="font-size:15px">${deck.total_cards}/100</span>
        <button class="btn secondary small" id="import-deck-btn">Importar decklist</button>
        <button class="btn secondary small" id="edit-deck-btn">Editar deck</button>
        <button class="btn secondary small" id="delete-deck-btn">Excluir deck</button>
      </div>
    </div>
    <div class="mode-toggle">
      <span class="chip ${deckViewMode === "list" ? "active" : ""}" data-view="list">Lista</span>
      <span class="chip ${deckViewMode === "grid" ? "active" : ""}" data-view="grid">Visual</span>
      <span class="chip ${deckViewMode === "stack" ? "active" : ""}" data-view="stack">Empilhado</span>
    </div>
    ${deckFilterBarHtml()}
    <div class="builder-layout">
      <div id="deck-cards"></div>
      <div>
        <div class="sidebar-panel" style="margin-bottom:16px">
          <h3>Curva de mana</h3>
          <div class="curve-bars">
            ${curveBars.map((b) => `<div class="curve-bar" style="height:${Math.max(4, (b.count / maxCount) * 90)}px" title="${b.count} cartas"></div>`).join("")}
          </div>
          <div class="curve-labels">${curveBars.map((b) => `<span>${b.label}</span>`).join("")}</div>
        </div>
        <div class="sidebar-panel">
          <h3>Adicionar carta</h3>
          ${overageWarning}
          <input type="text" id="add-card-input" placeholder="Nome (PT ou EN)…" style="width:100%;padding:9px 11px;border-radius:8px;border:1px solid var(--border-light);background:var(--bg-card);color:var(--text);font-size:13px;margin-bottom:8px;">
          <div id="add-card-results"></div>
        </div>
        ${synergyPanelHtml(synergy, ownershipMap)}
      </div>
    </div>
  `;

  document.querySelector('[data-nav="decks"]').addEventListener("click", (e) => {
    e.preventDefault();
    location.hash = "#decks";
  });

  document.getElementById("delete-deck-btn").addEventListener("click", async () => {
    if (!confirm(`Excluir o deck "${deck.name}"? As cartas voltam pra coleção livre — isso não pode ser desfeito.`)) return;
    await api.deleteDeck(id);
    location.hash = "#decks";
  });

  document.getElementById("edit-deck-btn").addEventListener("click", () =>
    openEditDeckModal(deck, () => renderDeckDetail([idStr]))
  );
  document.getElementById("import-deck-btn").addEventListener("click", () =>
    openImportDeckModal(id, () => renderDeckDetail([idStr]))
  );

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

  document.querySelectorAll("[data-view]").forEach((chip) =>
    chip.addEventListener("click", () => {
      deckViewMode = chip.dataset.view;
      renderDeckDetail([idStr]);
    })
  );

  // Synergy suggestions can be added straight to the deck — no need to retype the name in "Adicionar carta".
  document.querySelectorAll("[data-add-synergy]").forEach((btn) =>
    btn.addEventListener("click", async (e) => {
      e.stopPropagation();
      btn.disabled = true;
      btn.textContent = "Adicionando…";
      try {
        await api.addDeckCard(id, btn.dataset.addSynergy, 1);
        renderDeckDetail([idStr]);
      } catch (err) {
        console.error(err);
        btn.disabled = false;
        btn.textContent = "Falhou — tentar de novo";
      }
    })
  );

  wireDeckFilterBar(deck);
  renderDeckCards(deck);

  const input = document.getElementById("add-card-input");
  let debounce;
  input.addEventListener("input", () => {
    clearTimeout(debounce);
    const q = input.value.trim();
    if (q.length < 2) { document.getElementById("add-card-results").innerHTML = ""; return; }
    debounce = setTimeout(async () => {
      const results = await api.searchCards(q, 8);
      document.getElementById("add-card-results").innerHTML = results
        .map((c) => h`<div class="card-row" data-add="${c.name}" style="cursor:pointer"><span class="name">${c.name}</span><span class="cost">${manaCostHtml(c.mana_cost)}</span></div>`)
        .join("");
      document.querySelectorAll("[data-add]").forEach((el) =>
        el.addEventListener("click", async () => {
          await api.addDeckCard(id, el.dataset.add, 1);
          input.value = "";
          document.getElementById("add-card-results").innerHTML = "";
          renderDeckDetail([idStr]);
        })
      );
    }, 250);
  });
}

function renderDeckCards(deck) {
  const cardsWrap = document.getElementById("deck-cards");
  const byType = filteredDeckByType(deck);
  const categories = CATEGORY_ORDER.filter((c) => byType[c]?.length);

  if (!categories.length) {
    cardsWrap.innerHTML = `<div class="empty-state">Nenhuma carta corresponde aos filtros atuais.</div>`;
    return;
  }

  if (deckViewMode === "grid") {
    cardsWrap.innerHTML = categories
      .map((cat) => {
        const cards = byType[cat];
        const tiles = cards
          .map((c) => h`
            <div class="mtg-card" data-card-view="${c.card_name}">
              ${c.image_uri ? `<img src="${c.image_uri}" loading="lazy" alt="${c.card_name}">` : `<div class="no-image">${c.card_name}</div>`}
              <span class="qty-badge">${c.quantity}x</span>
              ${cat !== "Comandante" ? `<button class="btn small secondary tile-remove" data-remove="${c.id}">✕</button>` : ""}
            </div>`)
          .join("");
        return `<div class="category-block"><h4>${CATEGORY_LABELS[cat] || cat} <span class="n">(${cards.length})</span></h4><div class="card-grid">${tiles}</div></div>`;
      })
      .join("");
  } else if (deckViewMode === "stack") {
    // Columns side by side (not one category block under another) so the whole deck
    // fits in a single scroll instead of a long vertical chain of separate stacks.
    cardsWrap.innerHTML = `<div class="stack-columns">${categories
      .map((cat) => {
        const cards = byType[cat];
        const items = cards
          .map((c) => h`
            <div class="stack-item" data-card-view="${c.card_name}">
              ${c.image_uri ? `<img src="${c.image_uri}" loading="lazy" alt="${c.card_name}">` : `<div class="no-image">${c.card_name}</div>`}
              <div class="stack-name-bar">${c.quantity > 1 ? `${c.quantity}x ` : ""}${c.card_name}</div>
              ${cat !== "Comandante" ? `<button class="btn small secondary tile-remove" data-remove="${c.id}">✕</button>` : ""}
            </div>`)
          .join("");
        return `<div class="category-block"><h4>${CATEGORY_LABELS[cat] || cat} <span class="n">(${cards.length})</span></h4><div class="card-stack">${items}</div></div>`;
      })
      .join("")}</div>`;
  } else {
    cardsWrap.innerHTML = categories
      .map((cat) => {
        const cards = byType[cat];
        const rows = cards
          .map((c) => {
            const sharedTag = c.shared_with?.length
              ? `<span class="shared-tag">também em ${c.shared_with.map((s) => s.deck).join(", ")}</span>`
              : "";
            return h`
              <div class="card-row">
                <span class="qty">${c.quantity}x</span>
                <span class="name" data-card-view="${c.card_name}" style="cursor:pointer">${c.card_name}</span>
                ${sharedTag}
                <span class="cost">${manaCostHtml(c.mana_cost)}</span>
                ${cat !== "Comandante" ? `<button class="btn small secondary" data-remove="${c.id}">✕</button>` : ""}
              </div>`;
          })
          .join("");
        return `<div class="category-block"><h4>${CATEGORY_LABELS[cat] || cat} <span class="n">(${cards.length})</span></h4>${rows}</div>`;
      })
      .join("");
  }

  cardsWrap.querySelectorAll("[data-remove]").forEach((btn) =>
    btn.addEventListener("click", async (e) => {
      e.stopPropagation();
      await api.removeDeckCard(deck.id, Number(btn.dataset.remove));
      renderDeckDetail([String(deck.id)]);
    })
  );
  cardsWrap.querySelectorAll("[data-card-view]").forEach((el) =>
    el.addEventListener("click", () => showCardModal(el.dataset.cardView))
  );
}

function synergyPanelHtml(synergy, ownershipMap) {
  if (!synergy.cached) {
    return `
      <div class="sidebar-panel" style="margin-top:16px">
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
            <button class="btn small secondary" data-add-synergy="${r.name}" style="margin-left:auto">+ Adicionar</button>
          </div>
        </div>`;
    })
    .join("");
  const similar = (synergy.similar_commanders || []).slice(0, 5).map((s) => `<span class="chip">${s}</span>`).join(" ");
  return h`
    <div class="sidebar-panel" style="margin-top:16px">
      <h3>Sugestões de sinergia — ainda não no deck</h3>
      ${recs || '<div class="empty-state" style="padding:10px">Nada fora do deck nas categorias de alta sinergia.</div>'}
      ${similar ? `<h3 style="margin-top:16px">Comandantes parecidos</h3><div style="display:flex;flex-wrap:wrap;gap:6px">${similar}</div>` : ""}
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
        // a card can have several entries (one per deck/copy) — already grouped and summed by the API
        const realDecks = [...new Set(c.decks.filter((d) => d.deck_name !== "Livre").map((d) => d.deck_name))];
        const isAllocated = realDecks.length > 0;
        return h`
          <div class="mtg-card ${isAllocated ? "allocated" : ""}" data-card-view="${c.card_name}">
            ${c.image_uri ? `<img src="${c.image_uri}" loading="lazy" alt="${c.card_name}">` : `<div class="no-image">${c.card_name}</div>`}
            <span class="qty-badge">${c.total_quantity}x</span>
            ${isAllocated ? `<div class="deck-badge">${realDecks.join(" + ")}</div>` : ""}
          </div>`;
      })
      .join("") || `<div class="empty-state">Nada encontrado.</div>`;
    document.querySelectorAll("[data-card-view]").forEach((el) =>
      el.addEventListener("click", () => showCardModal(el.dataset.cardView))
    );
  }

  document.querySelectorAll("[data-filter]").forEach((chip) =>
    chip.addEventListener("click", () => {
      document.querySelectorAll("[data-filter]").forEach((c) => c.classList.remove("active"));
      chip.classList.add("active");
      collectionFilter = chip.dataset.filter;
      load();
    })
  );
  document.getElementById("coll-search").addEventListener("input", () => load());
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
  const backdrop = document.createElement("div");
  backdrop.className = "modal-backdrop";
  backdrop.innerHTML = h`
    <div class="modal">
      <h3>Adicionar carta</h3>
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
          <option value="">Nenhum (fica livre na coleção)</option>
          ${decks.map((d) => `<option value="${d.id}">${d.name}</option>`).join("")}
        </select>
      </div>
      <div style="margin-top:12px">
        <label style="font-size:12px;color:var(--text-dim)">Notas</label>
        <textarea id="ac-notes" rows="2" style="width:100%;margin-top:5px;padding:9px 11px;border-radius:8px;border:1px solid var(--border-light);background:var(--bg-card);color:var(--text);font-family:inherit"></textarea>
      </div>
      <div id="ac-error" style="color:var(--bad);font-size:12px;margin-top:10px;display:none">Preencha o nome da carta.</div>
      <div style="display:flex;gap:10px;margin-top:18px;justify-content:flex-end">
        <button class="btn secondary" id="ac-cancel">Cancelar</button>
        <button class="btn" id="ac-save">Salvar</button>
      </div>
    </div>`;
  document.body.appendChild(backdrop);
  backdrop.addEventListener("click", (e) => { if (e.target === backdrop) backdrop.remove(); });

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
    const card_name = nameInput.value.trim();
    if (!card_name) {
      backdrop.querySelector("#ac-error").style.display = "block";
      nameInput.focus();
      return;
    }
    const saveBtn = backdrop.querySelector("#ac-save");
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
  });
}

// ------------------------------------------------------------ card modal ----

export async function showCardModal(name) {
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
        <div style="flex:1;min-width:200px">
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
      <div class="qa-box">
        <label style="font-size:12px;color:var(--text-dim)">Perguntar sobre esta carta (busca no texto/oracle — não é IA generativa)</label>
        <input type="text" id="qa-input" placeholder="Ex: sinergia, cemitério, sacrifício…" style="margin-top:6px">
        <div id="qa-answer"></div>
      </div>
      <div style="margin-top:16px;text-align:right"><button class="btn secondary" id="modal-close">Fechar</button></div>
    `;
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
