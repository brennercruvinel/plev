import { api } from "../api.js?v=25";
import { activityIcon } from "../icons.js?v=25";
import { deckCardHtml } from "../deck-bits.js?v=2";
import { mainEl } from "../router.js?v=2";
import { openAddCardModal } from "../ui/add-card.js?v=2";
import { formatTs, h } from "../util.js?v=2";
import { openNewDeckModal } from "../views/decks.js?v=2";

export async function renderDashboard() {
  const [decks, collectionTotal, freeCollection, activity] = await Promise.all([
    api.decks(),
    api.collectionTotal(),
    api.collection("free"),
    api.activity(12),
  ]);
  const totalCards = decks.reduce((s, d) => s + d.total_cards, 0);
  const totalGames = decks.reduce((s, d) => s + d.wins + d.losses, 0);
  const totalWins = decks.reduce((s, d) => s + d.wins, 0);

  mainEl.innerHTML = h`
    <div class="page-header">
      <div><h1>Home</h1><p>Seu laboratório de coleção e decks — tudo lido do banco local.</p></div>
      <div style="display:flex;gap:10px">
        <button class="btn" id="add-card-btn">+ Adicionar Carta</button>
        <button class="btn secondary" id="dash-new-deck-btn">Novo Deck</button>
      </div>
    </div>
    <div class="stat-grid">
      <div class="stat-card clickable" data-stat-nav="decks"><div class="label">Decks montados</div><div class="value">${decks.length}</div><div class="sub">${totalCards} cartas ao todo</div></div>
      <div class="stat-card clickable" data-stat-nav="collection"><div class="label">Cartas na coleção</div><div class="value">${collectionTotal.total_units}</div><div class="sub">${collectionTotal.distinct_cards} nomes distintos, com repetidas</div></div>
      <div class="stat-card clickable" data-stat-nav="collection"><div class="label">Cartas livres</div><div class="value">${collectionTotal.free_units}</div><div class="sub">${freeCollection.length} nomes · ${collectionTotal.allocated_units} em decks</div></div>
      <div class="stat-card clickable" data-stat-nav="games"><div class="label">Partidas registradas</div><div class="value">${totalGames}</div><div class="sub">${totalWins} vitórias</div></div>
    </div>

    <div class="page-header"><h1 style="font-size:17px">Seus decks</h1></div>
    <div class="deck-grid" id="dash-decks"></div>

    <div class="page-header" style="margin-top:28px"><h1 style="font-size:17px">Histórico de atividades</h1>
      <p>Cartas novas, movimentações entre decks e decks montados — registrado conforme você usa o app.</p></div>
    <div id="dash-activity"></div>
  `;

  document.getElementById("dash-decks").innerHTML = decks
    .map(deckCardHtml)
    .join("") + h`
      <div class="deck-add-tile" id="dash-new-deck-tile">
        <span class="plus">+</span>
        <span class="label">Novo Deck</span>
      </div>`;
  document.querySelectorAll("[data-deck-link]").forEach((el) =>
    el.addEventListener("click", () => (location.hash = `#deck/${el.dataset.deckLink}`))
  );
  document.getElementById("add-card-btn").addEventListener("click", () =>
    openAddCardModal({ onSaved: renderDashboard })
  );
  document.getElementById("dash-new-deck-btn").addEventListener("click", () => openNewDeckModal());
  document.getElementById("dash-new-deck-tile").addEventListener("click", () => openNewDeckModal());
  document.querySelectorAll("[data-stat-nav]").forEach((el) =>
    el.addEventListener("click", () => (location.hash = `#${el.dataset.statNav}`))
  );

  document.getElementById("dash-activity").innerHTML = activity
    .map(
      (a) => h`
      <div class="activity-row">
        <span class="activity-icon">${activityIcon(a.type)}</span>
        <span class="activity-desc">${a.description}</span>
        <span class="activity-ts">${formatTs(a.ts)}</span>
      </div>`
    )
    .join("") || `<div class="empty-state">Nenhuma atividade ainda — mexa num deck ou na coleção pra ver o histórico aqui.</div>`;
}

export function renderScannerSoon() {
  mainEl.innerHTML = h`
    <div class="page-header">
      <div><h1>Scanner</h1><p>Reconhecimento de cartas pela câmera.</p></div>
    </div>
    <div class="ownership-summary" style="border-left-color:var(--accent)">
      <div class="own-line">
        <span class="own-tag tag-other-deck">Em breve</span>
        <span>Esta é a última funcionalidade pendente do app nativo. Enquanto isso, use
          <b>+ Adicionar Carta</b> (com o modo <b>Adicionar por lista</b>) ou
          <b>Importar decklist</b> na tela do deck.</span>
      </div>
    </div>`;
}
