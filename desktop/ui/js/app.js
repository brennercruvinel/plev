// Entry point.
//
// The only place that knows the full set of pages. Everything else is a leaf: views render into
// `mainEl` and import what they need, the router resolves hashes against whatever is registered
// here, and no module imports its own caller.
//
// To add a page: write `views/<name>.js` exporting a render function, add the route below, and
// add a matching `data-route` button in index.html. Nothing else has to change.

import { registerRoutes, navigate } from "./router.js?v=1";
import { renderSidebarDataPanel } from "./sidebar.js?v=1";
import { renderDashboard, renderScannerSoon } from "./views/home.js?v=1";
import { renderDecksList } from "./views/decks.js?v=1";
import { renderDeckDetail } from "./views/deck-detail.js?v=1";
import { renderCollection } from "./views/collection.js?v=1";
import { renderGames } from "./views/games.js?v=1";

registerRoutes({
  dashboard: renderDashboard,
  decks: renderDecksList,
  // Each render function receives the path segments after the route name, so #deck/12 passes
  // ["12"] — renderDeckDetail destructures it. Don't "simplify" this to (id) => ...: a string
  // destructures per character, and deck 12 would silently load deck 1.
  deck: renderDeckDetail,
  collection: renderCollection,
  scanner: renderScannerSoon,
  games: renderGames,
});

navigate();
renderSidebarDataPanel();
