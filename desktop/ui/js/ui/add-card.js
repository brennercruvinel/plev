import { api } from "../api.js?v=25";
import { manaCostHtml } from "../icons.js?v=25";
import { h, toast } from "../util.js?v=3";
import { getSetCode, setInputHtml, wireSetPicker } from "./set-picker.js?v=1";

export async function openAddCardModal({ onSaved } = {}) {
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
        <span class="chip" data-ac-mode="wishlist">Wishlist</span>
      </div>

      <div id="ac-single-wrap">
        <div style="margin-bottom:12px">
          <label style="font-size:12px;color:var(--text-dim)">Nome da carta *</label>
          <input type="text" id="ac-name" placeholder="Nome (PT ou EN)…" autocomplete="off"
            style="width:100%;margin-top:5px;padding:9px 11px;border-radius:8px;border:1px solid var(--border-light);background:var(--bg-card);color:var(--text)">
          <div id="ac-suggestions"></div>
        </div>
        <div class="form-grid">
          <div><label>Edição</label>${setInputHtml("ac-set")}</div>
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

      <div id="ac-wish-wrap" style="display:none">
        <p style="font-size:12.5px;color:var(--text-dim);line-height:1.5;margin:0 0 12px">
          Cartas que você quer comprar. Não entram na coleção nem contam como cartas que você tem —
          quando comprar, é um clique para movê-las.
        </p>
        <div style="margin-bottom:12px">
          <label style="font-size:12px;color:var(--text-dim)">Nome da carta *</label>
          <input type="text" id="aw-name" placeholder="Nome (PT ou EN)…" autocomplete="off"
            style="width:100%;margin-top:5px;padding:9px 11px;border-radius:8px;border:1px solid var(--border-light);background:var(--bg-card);color:var(--text)">
          <div id="aw-suggestions"></div>
        </div>
        <div class="form-grid">
          <div><label>Edição</label>${setInputHtml("aw-set")}</div>
          <div><label>Artista</label><input type="text" id="aw-artist" placeholder="Nome do artista"></div>
          <div><label>Idioma</label><select id="aw-lang"><option value="en">Inglês</option><option value="pt">Português</option></select></div>
          <div><label>Quantidade</label><input type="number" id="aw-qty" min="1" value="1"></div>
        </div>
        <div style="margin-top:12px">
          <label style="font-size:12px;color:var(--text-dim)">Notas</label>
          <textarea id="aw-notes" rows="2" placeholder="Ex: foil, versão retro, comprar até R$ 40"
            style="width:100%;margin-top:5px;padding:9px 11px;border-radius:8px;border:1px solid var(--border-light);background:var(--bg-card);color:var(--text);font-family:inherit"></textarea>
        </div>
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
  const wraps = {
    single: backdrop.querySelector("#ac-single-wrap"),
    list: backdrop.querySelector("#ac-list-wrap"),
    wishlist: backdrop.querySelector("#ac-wish-wrap"),
  };
  const firstField = { single: "#ac-name", list: "#ac-list-text", wishlist: "#aw-name" };
  const errorEl = backdrop.querySelector("#ac-error");
  backdrop.querySelectorAll("[data-ac-mode]").forEach((chip) =>
    chip.addEventListener("click", () => {
      mode = chip.dataset.acMode;
      backdrop.querySelectorAll("[data-ac-mode]").forEach((c) => c.classList.toggle("active", c === chip));
      for (const [key, el] of Object.entries(wraps)) el.style.display = key === mode ? "block" : "none";
      errorEl.style.display = "none";
      backdrop.querySelector(firstField[mode])?.focus();
      backdrop.querySelector("#ac-save").textContent = mode === "wishlist" ? "Adicionar à wishlist" : "Salvar";
    })
  );

  wireSetPicker(backdrop, "ac-set");
  wireSetPicker(backdrop, "aw-set");

  // Both name fields get the same autocomplete — the wishlist needs it just as much, since the
  // whole point of recording a card you don't own is getting its name right.
  function wireNameSearch(inputSel, listSel) {
    const nameInput = backdrop.querySelector(inputSel);
    const suggestionsEl = backdrop.querySelector(listSel);
    if (!nameInput) return;
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
  }
  wireNameSearch("#ac-name", "#ac-suggestions");
  wireNameSearch("#aw-name", "#aw-suggestions");
  backdrop.querySelector("#ac-name").focus();

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
      const qty = Number(backdrop.querySelector("#ac-qty").value) || 1;
      await api.addCollection({
        card_name,
        set_code: getSetCode(backdrop, "ac-set"),
        artist: backdrop.querySelector("#ac-artist").value.trim() || null,
        lang: backdrop.querySelector("#ac-lang").value,
        quantity: qty,
        notes: backdrop.querySelector("#ac-notes").value.trim() || null,
        deck_id: deckVal ? Number(deckVal) : null,
      });
      backdrop.remove();
      // The modal closes over whatever list is underneath, so without this the write left no
      // trace on screen and looked like it hadn't happened.
      toast(qty > 1 ? `${qty} cópias de ${card_name} adicionadas.` : `${card_name} adicionada à coleção.`);
      onSaved?.();
      return;
    }

    if (mode === "wishlist") {
      const card_name = backdrop.querySelector("#aw-name").value.trim();
      if (!card_name) {
        errorEl.textContent = "Preencha o nome da carta.";
        errorEl.style.display = "block";
        backdrop.querySelector("#aw-name").focus();
        return;
      }
      saveBtn.disabled = true;
      const qty = Number(backdrop.querySelector("#aw-qty").value) || 1;
      try {
        await api.addWishlist({
          card_name,
          set_code: getSetCode(backdrop, "aw-set"),
          artist: backdrop.querySelector("#aw-artist").value.trim() || null,
          lang: backdrop.querySelector("#aw-lang").value,
          quantity: qty,
          notes: backdrop.querySelector("#aw-notes").value.trim() || null,
        });
        backdrop.remove();
        toast(qty > 1 ? `${qty}x ${card_name} na wishlist.` : `${card_name} entrou na wishlist.`);
        onSaved?.();
      } catch (err) {
        saveBtn.disabled = false;
        errorEl.textContent = err.message;
        errorEl.style.display = "block";
      }
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
      toast(`${found.length} carta${found.length > 1 ? "s" : ""} adicionada${found.length > 1 ? "s" : ""} à coleção.`);
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

// Panel inside the card modal: how many physical copies the user owns, where they are, and a
// way to add one more. Two ways in, because they answer different questions — "I bought another
// one" needs nothing but a count, while cataloguing a specific printing needs set/artist/lang.
