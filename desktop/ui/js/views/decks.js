import { api } from "../api.js?v=25";
import { manaCostHtml } from "../icons.js?v=25";
import { BRACKET_LABELS, deckCardHtml } from "../deck-bits.js?v=2";
import { mainEl } from "../router.js?v=2";
import { h, pollJob } from "../util.js?v=2";

export async function renderDecksList() {
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

export async function openNewDeckModal() {
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

// Deleting a deck has two very different meanings depending on whether you physically own its
// cards: a real deck you're taking apart should leave its cards in the collection, while a
// planned/auto-built list you never bought should take them out with it. Asking is the only way
// to know which — guessing either way silently corrupts the collection count.
export async function openDeleteDeckModal(deck, id) {
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

export async function openEditDeckModal(deck, onSaved) {
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

export async function openImportDeckModal(deckId, onImported) {
  // Non-commander cards currently in the deck — the number "Trocar o deck inteiro" would remove.
  let currentDeckSize = NaN;
  api
    .deck(deckId)
    .then((d) => {
      currentDeckSize = Object.entries(d.by_type || {})
        .filter(([cat]) => cat !== "Comandante")
        .reduce((n, [, cards]) => n + cards.reduce((m, c) => m + (c.quantity || 0), 0), 0);
    })
    .catch(() => {});
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
      <!-- The old label said "as cartas (comuns) já no deck", which reads as though the choice
           only touched the overlap. "Substituir" wipes the whole deck, so the copy now says so
           and names the number of cards at stake. -->
      <div id="im-mode-wrap" style="display:none;margin-top:14px">
        <label style="font-size:12px;color:var(--text-dim);display:block;margin-bottom:6px">Este deck já tem cartas. O que fazer?</label>
        <label style="display:flex;align-items:flex-start;gap:6px;font-size:13px;margin-bottom:6px;cursor:pointer"><input type="radio" name="im-mode" value="merge" checked style="margin-top:3px">
          <span>Somar à lista atual<br><span style="font-size:12px;color:var(--text-faint)">Cartas repetidas viram 2x, 3x… As novas entram normalmente.</span></span></label>
        <label style="display:flex;align-items:flex-start;gap:6px;font-size:13px;cursor:pointer"><input type="radio" name="im-mode" value="replace" style="margin-top:3px">
          <span>Trocar o deck inteiro por esta lista<br><span style="font-size:12px;color:var(--bad)" id="im-replace-warn">Remove as cartas atuais do deck. O comandante fica.</span></span></label>
      </div>
      <div id="im-error" style="color:var(--bad);font-size:12px;margin-top:10px;display:none"></div>
      <div style="display:flex;gap:10px;margin-top:18px;justify-content:flex-end">
        <button class="btn secondary" id="im-cancel">Cancelar</button>
        <button class="btn" id="im-preview-text" disabled>Importar</button>
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
    // Naming the real number is what makes the consequence concrete: "remove as 99 cartas
    // atuais" lands where "remove as cartas atuais" slides past.
    const warn = backdrop.querySelector("#im-replace-warn");
    if (warn && Number.isFinite(currentDeckSize)) {
      warn.textContent = currentDeckSize
        ? `Remove as ${currentDeckSize} cartas atuais do deck. O comandante fica.`
        : "O deck não tem outras cartas — as duas opções fazem o mesmo.";
    }
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
    syncImportEnabled();
    errEl.style.display = "none";
  }

  async function runPreview() {
    const text = textEl.value;
    errEl.style.display = "none";
    if (!text.trim()) return; // the button is disabled in this state; belt and braces
    previewBtn.disabled = true;
    previewBtn.textContent = "Lendo…";
    try {
      renderPreview(await api.importPreviewText(deckId, text));
    } catch (err) {
      showError(err.message);
    } finally {
      previewBtn.disabled = false;
      previewBtn.textContent = "Importar";
      syncImportEnabled();
    }
  }

  // The button is the only way in, so it must never be clickable with nothing to import — an
  // empty click previously just surfaced an error the user could have been spared.
  function syncImportEnabled() {
    previewBtn.disabled = !textEl.value.trim();
  }

  previewBtn.addEventListener("click", runPreview);
  textEl.addEventListener("input", () => {
    resetPreview();
    syncImportEnabled();
  });

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
      syncImportEnabled();
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
