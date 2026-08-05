import { api } from "./api.js?v=13";
import { manaCostHtml } from "./icons.js?v=13";

const ICON_LIST = `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" width="14" height="14" style="vertical-align:-2px"><line x1="8" y1="6" x2="21" y2="6"/><line x1="8" y1="12" x2="21" y2="12"/><line x1="8" y1="18" x2="21" y2="18"/><line x1="3" y1="6" x2="3.01" y2="6"/><line x1="3" y1="12" x2="3.01" y2="12"/><line x1="3" y1="18" x2="3.01" y2="18"/></svg>`;
const ICON_BOOK = `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" width="14" height="14" style="vertical-align:-2px"><path d="M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z"/><path d="M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z"/></svg>`;
const ICON_CAMERA = `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" width="14" height="14" style="vertical-align:-2px"><path d="M23 19a2 2 0 0 1-2 2H3a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h4l2-3h6l2 3h4a2 2 0 0 1 2 2z"/><circle cx="12" cy="13" r="4"/></svg>`;

let stream = null;
let mode = "scan"; // "scan" (builds a decklist) or "pokedex" (lookup)
let scanBatch = []; // cards confirmed during this scan session

export async function renderScanner(mainEl, params, { showCardModal }) {
  mainEl.innerHTML = `
    <div class="page-header">
      <div><h1>Scanner</h1><p>Aponte a câmera pra uma carta física. O reconhecimento roda por OCR + busca fuzzy na base local (38 mil cartas) — não é um classificador visual treinado, então confirme sempre o resultado antes de adicionar.</p></div>
    </div>
    <div class="mode-toggle">
      <span class="chip ${mode === "scan" ? "active" : ""}" data-mode="scan">${ICON_LIST} Montar decklist</span>
      <span class="chip ${mode === "pokedex" ? "active" : ""}" data-mode="pokedex">${ICON_BOOK} Pokédex (consulta)</span>
    </div>
    <div class="scanner-layout">
      <div>
        <div class="camera-box" id="camera-box">
          <div class="placeholder">Clique em "Ligar câmera" para começar<br><small style="color:var(--text-faint)">Precisa de permissão do navegador. Primeira vez carrega a lib de OCR (precisa de internet uma vez; depois fica em cache do navegador).</small></div>
        </div>
        <div style="display:flex;gap:10px;margin-top:12px">
          <button class="btn" id="cam-toggle">Ligar câmera</button>
          <button class="btn secondary" id="cam-capture" disabled>${ICON_CAMERA} Capturar e reconhecer</button>
        </div>
        <div id="ocr-status" style="margin-top:10px;font-size:12px;color:var(--text-faint)"></div>

        <div id="scan-batch-panel" style="margin-top:20px"></div>
      </div>
      <div>
        <div class="sidebar-panel">
          <h3>Candidatos</h3>
          <div id="candidates">
            <div class="empty-state" style="padding:20px 10px">Capture uma carta pra ver sugestões aqui.</div>
          </div>
        </div>
        <div class="sidebar-panel pokedex-panel" id="pokedex-detail" style="margin-top:16px;display:none"></div>
      </div>
    </div>
  `;

  renderScanBatch();

  mainEl.querySelectorAll("[data-mode]").forEach((chip) =>
    chip.addEventListener("click", () => {
      mode = chip.dataset.mode;
      renderScanner(mainEl, params, { showCardModal });
    })
  );

  const camBox = document.getElementById("camera-box");
  const toggleBtn = document.getElementById("cam-toggle");
  const captureBtn = document.getElementById("cam-capture");

  toggleBtn.addEventListener("click", async () => {
    if (stream) {
      stream.getTracks().forEach((t) => t.stop());
      stream = null;
      camBox.innerHTML = `<div class="placeholder">Câmera desligada.</div>`;
      toggleBtn.textContent = "Ligar câmera";
      captureBtn.disabled = true;
      return;
    }
    try {
      stream = await navigator.mediaDevices.getUserMedia({ video: { facingMode: "environment" } });
      camBox.innerHTML = `<video id="cam-video" autoplay playsinline></video><div class="scan-frame"></div>`;
      document.getElementById("cam-video").srcObject = stream;
      toggleBtn.textContent = "Desligar câmera";
      captureBtn.disabled = false;
    } catch (err) {
      camBox.innerHTML = `<div class="placeholder">Não consegui acessar a câmera: ${err.message}<br>Você pode digitar o nome manualmente na busca da Coleção enquanto isso.</div>`;
    }
  });

  captureBtn.addEventListener("click", () => captureAndRecognize());
}

async function captureAndRecognize() {
  const video = document.getElementById("cam-video");
  const status = document.getElementById("ocr-status");
  if (!video) return;

  const canvas = document.createElement("canvas");
  canvas.width = video.videoWidth;
  canvas.height = video.videoHeight;
  canvas.getContext("2d").drawImage(video, 0, 0);
  const dataUrl = canvas.toDataURL("image/png");

  status.textContent = "Lendo o texto da carta (OCR)…";
  try {
    if (typeof Tesseract === "undefined") {
      status.textContent = "Biblioteca de OCR não carregou (precisa de internet na primeira vez). Digite o nome manualmente abaixo.";
      showManualFallback();
      return;
    }
    const { data } = await Tesseract.recognize(dataUrl, "eng");
    const text = data.text.trim();
    status.textContent = text ? `Texto lido: "${text.slice(0, 60)}${text.length > 60 ? "…" : ""}"` : "Não consegui ler texto — tente reenquadrar a carta.";
    const { candidates } = await api.scanRecognize(text);
    renderCandidates(candidates);
  } catch (err) {
    status.textContent = `Erro no OCR: ${err.message}`;
    showManualFallback();
  }
}

function showManualFallback() {
  document.getElementById("candidates").innerHTML = `
    <input type="text" id="manual-search" placeholder="Digite o nome da carta…" style="width:100%;padding:8px 10px;border-radius:8px;border:1px solid var(--border-light);background:var(--bg-card);color:var(--text);margin-bottom:10px">
    <div id="manual-results"></div>
  `;
  const input = document.getElementById("manual-search");
  input.addEventListener("input", async () => {
    if (input.value.trim().length < 2) return;
    const results = await api.searchCards(input.value.trim(), 8);
    renderCandidates(results);
  });
}

function renderCandidates(candidates) {
  const wrap = document.getElementById("candidates");
  if (!candidates?.length) {
    wrap.innerHTML = `<div class="empty-state" style="padding:20px 10px">Nenhum candidato. Tente reenquadrar ou buscar manualmente.</div>`;
    showManualFallback();
    return;
  }
  wrap.innerHTML = candidates
    .map(
      (c) => `
      <div class="candidate-card" data-name="${c.name}">
        ${c.image_uri ? `<img src="${c.image_uri}">` : `<div style="width:48px;height:67px;background:var(--bg-card);border-radius:4px"></div>`}
        <div class="info">
          <div class="name">${c.name}${c.pt_name ? ` <span style="color:var(--text-faint);font-weight:400">(${c.pt_name})</span>` : ""}</div>
          <div class="type">${c.type_line || ""} ${manaCostHtml(c.mana_cost)}</div>
        </div>
      </div>`
    )
    .join("");

  wrap.querySelectorAll(".candidate-card").forEach((el) =>
    el.addEventListener("click", () => {
      if (mode === "scan") {
        addToScanBatch(el.dataset.name);
      } else {
        showPokedexDetail(el.dataset.name);
      }
    })
  );
}

function addToScanBatch(name) {
  scanBatch.push(name);
  renderScanBatch();
}

function renderScanBatch() {
  const panel = document.getElementById("scan-batch-panel");
  if (!panel) return;
  if (mode !== "scan") { panel.innerHTML = ""; return; }
  panel.innerHTML = `
    <div class="sidebar-panel">
      <h3>Decklist sendo montada (${scanBatch.length} cartas)</h3>
      ${scanBatch.length
        ? scanBatch.map((n, i) => `<div class="card-row"><span class="name">${n}</span><button class="btn small secondary" data-remove-batch="${i}">✕</button></div>`).join("")
        : `<div class="empty-state" style="padding:14px">Escaneie cartas físicas uma a uma — cada confirmação entra aqui.</div>`}
      ${scanBatch.length ? `<button class="btn" id="export-batch" style="margin-top:12px;width:100%">Copiar lista (.txt)</button>` : ""}
    </div>
  `;
  panel.querySelectorAll("[data-remove-batch]").forEach((btn) =>
    btn.addEventListener("click", () => {
      scanBatch.splice(Number(btn.dataset.removeBatch), 1);
      renderScanBatch();
    })
  );
  const exportBtn = document.getElementById("export-batch");
  if (exportBtn) {
    exportBtn.addEventListener("click", () => {
      const text = scanBatch.map((n) => `1 ${n}`).join("\n");
      navigator.clipboard?.writeText(text);
      exportBtn.textContent = "Copiado!";
      setTimeout(() => (exportBtn.textContent = "Copiar lista (.txt)"), 1500);
    });
  }
}

async function showPokedexDetail(name) {
  const panel = document.getElementById("pokedex-detail");
  panel.style.display = "block";
  panel.innerHTML = `<div class="empty-state">Carregando…</div>`;
  try {
    const c = await api.card(name);
    panel.innerHTML = `
      ${c.image_uri ? `<img src="${c.image_uri}">` : ""}
      <h3 style="margin:0 0 4px">${c.name}</h3>
      <div style="margin-bottom:8px">${manaCostHtml(c.mana_cost)} · ${c.type_line || ""}</div>
      <div class="oracle">${(c.oracle_text || "").replace(/\n/g, "<br>")}</div>
      <div style="margin-top:12px;font-size:12px;color:var(--text-dim)">Preço: ${c.price_usd ? `$${c.price_usd}` : "—"} · EDHREC #${c.edhrec_rank ?? "?"}</div>
      <div class="qa-box">
        <input type="text" id="pokedex-qa" placeholder="Pergunte algo sobre a carta…">
        <div id="pokedex-qa-answer"></div>
      </div>
    `;
    document.getElementById("pokedex-qa").addEventListener("keydown", (e) => {
      if (e.key !== "Enter") return;
      const q = e.target.value.toLowerCase();
      const text = (c.oracle_text || "").toLowerCase();
      document.getElementById("pokedex-qa-answer").innerHTML = `<div class="qa-answer">${
        text.includes(q)
          ? `O texto da carta menciona "${q}", sim.`
          : `Não encontrei "${q}" no texto da carta. Isso é busca por palavra-chave, não IA — pra dúvidas de regra mais profundas, ver a Comprehensive Rules no vault.`
      }</div>`;
    });
  } catch {
    panel.innerHTML = `<div class="empty-state">Carta não encontrada.</div>`;
  }
}
