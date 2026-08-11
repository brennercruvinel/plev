// In-app confirmation dialog.
//
// `window.confirm` cannot be used here. wry's WKUIDelegate implements only the file-upload,
// media-permission and new-window callbacks — there is no runJavaScriptConfirmPanel — and
// WKWebView's documented behaviour with no delegate method is to return false immediately.
// So every `if (!confirm(...)) return;` was an unconditional early return: the button appeared
// dead. `alert()` is a silent no-op for the same reason, which is why failures never surfaced.
//
// Anything needing a yes/no answer, or an error the user must see, goes through this module or
// `toast` instead.

import { h } from "../util.js?v=3";

/**
 * Resolves true when confirmed, false otherwise. Escape and a backdrop click both cancel, and
 * Enter confirms, so the dialog keeps the keyboard behaviour people expect from the native one.
 */
export function confirmDialog({
  title,
  message,
  confirmLabel = "Confirmar",
  cancelLabel = "Cancelar",
  danger = false,
} = {}) {
  return new Promise((resolve) => {
    const backdrop = document.createElement("div");
    backdrop.className = "modal-backdrop";
    backdrop.innerHTML = h`
      <div class="modal" style="max-width:420px" role="dialog" aria-modal="true">
        <h3>${title}</h3>
        <p style="font-size:13px;color:var(--text-dim);line-height:1.5">${message}</p>
        <div style="display:flex;gap:10px;margin-top:18px;justify-content:flex-end">
          <button class="btn secondary" data-confirm-no>${cancelLabel}</button>
          <button class="btn ${danger ? "danger" : ""}" data-confirm-yes>${confirmLabel}</button>
        </div>
      </div>`;
    document.body.appendChild(backdrop);

    const done = (value) => {
      document.removeEventListener("keydown", onKey);
      backdrop.remove();
      resolve(value);
    };
    const onKey = (e) => {
      if (e.key === "Escape") done(false);
      if (e.key === "Enter") done(true);
    };

    backdrop.addEventListener("click", (e) => {
      if (e.target === backdrop) done(false);
    });
    backdrop.querySelector("[data-confirm-no]").addEventListener("click", () => done(false));
    backdrop.querySelector("[data-confirm-yes]").addEventListener("click", () => done(true));
    document.addEventListener("keydown", onKey);
    backdrop.querySelector("[data-confirm-yes]").focus();
  });
}
