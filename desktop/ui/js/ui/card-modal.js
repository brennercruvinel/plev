import { api } from "../api.js?v=24";
import { manaCostHtml } from "../icons.js?v=24";
import { h, highlightMatch, priceLabel } from "../util.js?v=1";

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
