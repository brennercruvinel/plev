import { api } from "./api.js?v=25";
import { navigate } from "./router.js?v=2";
import { formatBuiltAt, h, pollJob } from "./util.js?v=2";

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

export async function renderSidebarDataPanel() {
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
