// Two-faced cards: showing the back.
//
// A "//" in the name is not the test. Split, flip, adventure and aftermath cards carry two names
// on a single printed face — offering to flip one promises a side that doesn't exist. Only
// transform / modal DFC / reversible layouts have a real back, and the index records that as
// `image_uri_back`, so the presence of that field is the whole condition.
//
// Every surface renders the image through `cardImgHtml` and calls `wireCardFlips` on its
// container once. The flip button lives inside the image wrapper and swaps `src` in place — no
// re-render, so scroll position, hover state and open menus all survive.

export function isTwoFaced(card) {
  return !!(card && card.image_uri_back);
}

/**
 * The `<img>` (or placeholder) for a card, with a flip button when there is a real second face.
 *
 * `opts.cls`   extra classes for the <img>
 * `opts.attrs` extra attributes for the <img> (loading, decoding, …)
 * `opts.crop`  render art crops instead of the full card — flips still work, since the crop is
 *              derived from the same URL by swapping the variant segment.
 */
export function cardImgHtml(card, opts = {}) {
  const name = card.card_name || card.name || "";
  const front = opts.crop ? toCrop(card.image_uri) : card.image_uri;
  if (!front) return `<div class="no-image">${name}</div>`;

  const attrs = opts.attrs || "";
  const cls = opts.cls ? ` class="${opts.cls}"` : "";
  const img = `<img src="${front}" alt="${name}"${cls} ${attrs}>`;
  if (!isTwoFaced(card)) return img;

  const back = opts.crop ? toCrop(card.image_uri_back) : card.image_uri_back;
  return `<span class="card-face-wrap">${img}
    <button type="button" class="card-flip-btn" title="Virar carta" aria-label="Virar carta"
      data-flip-front="${front}" data-flip-back="${back}" aria-pressed="false">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 12a9 9 0 0 1 15-6.7L21 8"/><path d="M21 3v5h-5"/><path d="M21 12a9 9 0 0 1-15 6.7L3 16"/><path d="M3 21v-5h5"/></svg>
    </button></span>`;
}

function toCrop(url) {
  return url ? url.replace("/normal/", "/art_crop/") : url;
}

/**
 * Delegated, so it survives the list being re-rendered underneath and costs one listener per
 * screen rather than one per card. Safe to call more than once on the same node.
 */
export function wireCardFlips(container) {
  if (!container || container.dataset.flipsWired === "1") return;
  container.dataset.flipsWired = "1";
  container.addEventListener("click", (e) => {
    const btn = e.target.closest(".card-flip-btn");
    if (!btn || !container.contains(btn)) return;
    // The card underneath usually opens a modal on click; flipping is not that.
    e.preventDefault();
    e.stopPropagation();
    const img = btn.parentElement.querySelector("img");
    if (!img) return;
    const showingBack = btn.getAttribute("aria-pressed") === "true";
    img.src = showingBack ? btn.dataset.flipFront : btn.dataset.flipBack;
    btn.setAttribute("aria-pressed", String(!showingBack));
  });
}
