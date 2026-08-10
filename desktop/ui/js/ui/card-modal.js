import { api } from "../api.js?v=25";
import { manaCostHtml } from "../icons.js?v=25";
import { h, highlightMatch, priceLabel, toast } from "../util.js?v=3";
import { cardImgHtml, wireCardFlips } from "./card-face.js?v=1";

// One row per stored copy, each with its own delete. A row can represent several identical
// copies (quantity > 1), in which case deleting takes one off rather than discarding the stack —
// the label says how many are behind it so that isn't a surprise.
function unitRowHtml(e) {
  const where = e.deck_name
    ? `<span class="copy-dot dot-deck"></span>${e.deck_name}`
    : `<span class="copy-dot dot-free"></span>Livre`;
  const details = [
    e.set_code ? String(e.set_code).toUpperCase() : null,
    e.lang && e.lang !== "en" ? String(e.lang).toUpperCase() : null,
    e.notes || null,
  ].filter(Boolean).join(" · ");
  return `
    <div class="copy-unit" data-entry="${e.id}">
      <span class="copy-unit-where">${where}${e.quantity > 1 ? ` <b>${e.quantity}x</b>` : ""}</span>
      <span class="copy-unit-detail">${details}</span>
      <button class="icon-btn danger" data-del-entry="${e.id}" title="Remover uma unidade" aria-label="Remover uma unidade">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" width="14" height="14"><path d="M3 6h18"/><path d="M8 6V4a1 1 0 0 1 1-1h6a1 1 0 0 1 1 1v2"/><path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6"/><path d="M10 11v6M14 11v6"/></svg>
      </button>
    </div>`;
}

// Deleting the last copy of a card removes it from the collection entirely, which is a different
// thing from thinning a playset — so that one asks first, and says exactly what will happen.
function confirmLastCopy(cardName) {
  return new Promise((resolve) => {
    const backdrop = document.createElement("div");
    backdrop.className = "modal-backdrop";
    backdrop.innerHTML = h`
      <div class="modal" style="max-width:420px">
        <h3>Remover da coleção?</h3>
        <p style="font-size:13px;color:var(--text-dim);line-height:1.5">
          Esta é a <b>última cópia</b> de <b>${cardName}</b>. Removê-la tira a carta da sua coleção
          — ela deixa de aparecer nas buscas e nas sugestões de "cartas que você tem".
        </p>
        <div style="display:flex;gap:10px;margin-top:18px;justify-content:flex-end">
          <button class="btn secondary" id="lc-cancel">Cancelar</button>
          <button class="btn danger" id="lc-ok">Remover</button>
        </div>
      </div>`;
    document.body.appendChild(backdrop);
    const done = (v) => { backdrop.remove(); resolve(v); };
    backdrop.addEventListener("click", (e) => { if (e.target === backdrop) done(false); });
    backdrop.querySelector("#lc-cancel").addEventListener("click", () => done(false));
    backdrop.querySelector("#lc-ok").addEventListener("click", () => done(true));
  });
}

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

      ${copies.entries?.length ? `<div class="copy-units">${copies.entries.map(unitRowHtml).join("")}</div>` : ""}

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
      const qty = payload.quantity || 1;
      await api.addCollection({ card_name: cardName, ...payload });
      toast(qty > 1 ? `${qty} cópias de ${cardName} adicionadas.` : `${cardName} adicionada à coleção.`);
      await renderCopiesBox(backdrop, cardName, onCollectionChange);
      onCollectionChange?.();
    } catch (err) {
      msg.textContent = err.message;
      msg.style.color = "var(--bad)";
      btn.disabled = false;
      btn.textContent = label;
    }
  }

  box.querySelectorAll("[data-del-entry]").forEach((btn) =>
    btn.addEventListener("click", async () => {
      const entryId = Number(btn.dataset.delEntry);
      const entry = copies.entries.find((e) => e.id === entryId);
      // Last copy overall = this row holds the only remaining unit of the card.
      const isLast = copies.total === 1 && entry?.quantity === 1;
      if (isLast && !(await confirmLastCopy(cardName))) return;
      btn.disabled = true;
      try {
        const res = await api.deleteCollectionEntry(entryId);
        toast(
          res.remaining === 0
            ? `${cardName} removida da coleção.`
            : `1 cópia removida — restam ${res.remaining}.`
        );
        await renderCopiesBox(backdrop, cardName, onCollectionChange);
        onCollectionChange?.();
      } catch (err) {
        toast(err.message, "bad");
        btn.disabled = false;
      }
    })
  );

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
        ${cardImgHtml(c, { cls: "modal-card-img" })}
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
    wireCardFlips(backdrop.querySelector(".modal"));
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
