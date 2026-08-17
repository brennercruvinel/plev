// Clear button for search inputs.
//
// Attaches at runtime rather than being part of each field's markup: the three search boxes live
// in three different views, and wrapping them here means a call site only has to say which input
// it wants enhanced — no parallel markup to keep in sync, and none of them had to change shape.
//
// The button dispatches a real `input` event rather than calling the view's loader directly, so
// whatever each screen already wired up (debounced reload, filter recompute) runs unchanged and
// this module stays ignorant of what clearing means on any given page.

/**
 * Wraps `input` in a positioned container and adds a clear button that appears only when there
 * is something to clear. Idempotent — calling it twice on the same field does nothing.
 */
export function attachClear(input) {
  if (!input || input.dataset.clearAttached === "1") return;
  input.dataset.clearAttached = "1";

  const wrap = document.createElement("div");
  wrap.className = "search-field";
  input.parentNode.insertBefore(wrap, input);
  wrap.appendChild(input);

  const btn = document.createElement("button");
  btn.type = "button";
  btn.className = "search-clear";
  btn.title = "Limpar busca";
  btn.setAttribute("aria-label", "Limpar busca");
  btn.innerHTML =
    '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round"><path d="M18 6 6 18M6 6l12 12"/></svg>';
  wrap.appendChild(btn);

  const sync = () => wrap.classList.toggle("has-value", input.value !== "");

  const clear = () => {
    if (input.value === "") return;
    input.value = "";
    sync();
    // Let the screen's own handler react. `bubbles` so a delegated listener would also see it.
    input.dispatchEvent(new Event("input", { bubbles: true }));
    input.focus();
  };

  input.addEventListener("input", sync);
  // Escape clears too — the habit every search field on every platform has trained.
  input.addEventListener("keydown", (e) => {
    if (e.key === "Escape" && input.value !== "") {
      e.preventDefault();
      e.stopPropagation();
      clear();
    }
  });
  // mousedown, not click: the field may be blurring as the button is pressed, and some callers
  // close menus on blur — acting on the earlier event keeps the clear from being swallowed.
  btn.addEventListener("mousedown", (e) => {
    e.preventDefault();
    clear();
  });

  sync();
}
