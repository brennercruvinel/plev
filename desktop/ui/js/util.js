export function h(strings, ...values) {
  return strings.reduce((acc, s, i) => acc + s + (values[i] ?? ""), "");
}

export function priceLabel(p) {
  return p ? `$${Number(p).toFixed(2)}` : "—";
}

// ------------------------------------------------------------- dashboard ----

export function formatTs(ts) {
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

export function pollJob(statusEl, prefix, fetchStatusFn, { onDone, onError, onSettle } = {}) {
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

export function formatBuiltAt(unixSeconds) {
  if (!unixSeconds) return null;
  return new Date(unixSeconds * 1000).toLocaleString("pt-BR", {
    day: "2-digit", month: "2-digit", year: "numeric", hour: "2-digit", minute: "2-digit",
  });
}

export function highlightMatch(text, q) {
  const idx = text.toLowerCase().indexOf(q);
  if (idx === -1) return text.slice(0, 80);
  return text.slice(Math.max(0, idx - 30), idx + q.length + 30);
}
