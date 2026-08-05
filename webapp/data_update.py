"""
On-demand refresh of the local data the app relies on, triggered by the
"Atualizar base de dados" button on the Dashboard.

Two sources, both already used elsewhere in this project (see
scryfall.py / mtgdb.py / edhrec.py and the vault notes they're built from):

- Scryfall bulk data (oracle_cards + all_cards) — the card database, official
  Portuguese names and card images. Re-downloading and reindexing picks up
  new sets/blocks as soon as Scryfall publishes them.
- EDHREC's per-commander synergy pages — re-fetched only for the commanders
  already in the user's own decks (never a bulk scan of the site; EDHREC has
  no bulk export and asks that it only be queried on demand).

Runs in a background thread so the request that starts it returns
immediately; progress is exposed via get_status() for the frontend to poll.
"""
import glob
import os
import sys
import threading
import time
import urllib.request

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
DATA_DIR = os.path.join(ROOT, "data")

sys.path.insert(0, ROOT)
import scryfall  # noqa: E402
import mtgdb  # noqa: E402
import edhrec  # noqa: E402

from db import get_app_db, get_cards_db  # noqa: E402

_status = {
    "state": "idle",  # idle | running | done | error
    "step": None,
    "log": [],
    "started_at": None,
    "finished_at": None,
    "error": None,
    "result": None,
}
_lock = threading.Lock()


def get_status():
    with _lock:
        return dict(_status, log=list(_status["log"]))


def is_running():
    with _lock:
        return _status["state"] == "running"


def _log(msg):
    with _lock:
        _status["log"].append(msg)
        _status["step"] = msg


def _begin():
    with _lock:
        _status["state"] = "running"
        _status["log"] = []
        _status["error"] = None
        _status["result"] = None
        _status["started_at"] = time.time()
        _status["finished_at"] = None


def _finish(state, error=None, result=None):
    with _lock:
        _status["state"] = state
        _status["error"] = error
        _status["result"] = result
        _status["finished_at"] = time.time()


def fetch_one_commander(name, with_combos=True):
    """Fetches and caches EDHREC synergy for a single commander. Returns (ok, error)."""
    import json
    slug = edhrec.slugify(name)
    data, err = edhrec._fetch_json(f"{edhrec.BASE}/commanders/{slug}.json")
    if err:
        return False, err
    os.makedirs(os.path.join(edhrec.CACHE, "commanders"), exist_ok=True)
    with open(edhrec._cache_path("commanders", slug), "w", encoding="utf-8") as fh:
        json.dump(data, fh, ensure_ascii=False)
    if with_combos:
        os.makedirs(os.path.join(edhrec.CACHE, "combos"), exist_ok=True)
        cdata, cerr = edhrec._fetch_json(f"{edhrec.BASE}/combos/{slug}.json")
        if not cerr:
            with open(edhrec._cache_path("combos", slug), "w", encoding="utf-8") as fh:
                json.dump(cdata, fh, ensure_ascii=False)
    return True, None


def _download_bulk(bulk_type):
    """Downloads a Scryfall bulk data file and drops any older file of the same kind."""
    data = scryfall._get("/bulk-data")
    items = data.get("data", [])
    match = next((x for x in items if x["type"] == bulk_type), None)
    if not match:
        raise RuntimeError(f"tipo de bulk data não encontrado no Scryfall: {bulk_type}")

    uri = match.get("download_uri") or match.get("jsonl_download_uri")
    mb = match.get("compressed_size", 0) / 1024 / 1024
    filename = uri.split("/")[-1].split("?")[0]
    dest = os.path.join(DATA_DIR, filename)

    _log(f"Baixando {bulk_type} do Scryfall (~{mb:.0f} MB)…")
    req = urllib.request.Request(uri, headers={"User-Agent": scryfall.USER_AGENT})
    tmp_dest = dest + ".part"
    with urllib.request.urlopen(req, timeout=600) as resp, open(tmp_dest, "wb") as out:
        while True:
            block = resp.read(1 << 20)
            if not block:
                break
            out.write(block)
    os.replace(tmp_dest, dest)

    prefix = "oracle-cards-" if bulk_type == "oracle_cards" else "all-cards-"
    for f in glob.glob(os.path.join(DATA_DIR, f"{prefix}*.jsonl.gz")):
        if f != dest:
            os.remove(f)
    return dest


def _run(refresh_synergy):
    try:
        os.makedirs(DATA_DIR, exist_ok=True)

        _download_bulk("oracle_cards")
        _download_bulk("all_cards")

        _log("Reconstruindo o índice local (SQLite)…")
        mtgdb.cmd_build(None)

        cdb = get_cards_db()
        n_cards = cdb.execute("SELECT COUNT(*) FROM cards").fetchone()[0]
        n_pt = cdb.execute("SELECT COUNT(DISTINCT printed_name) FROM names_pt").fetchone()[0]
        cdb.close()
        _log(f"Índice pronto: {n_cards:,} cartas, {n_pt:,} nomes em português.")

        synergy_updated = []
        if refresh_synergy:
            con = get_app_db()
            commanders = sorted({
                r["commander_name"]
                for r in con.execute("SELECT DISTINCT commander_name FROM decks").fetchall()
            })
            con.close()
            for name in commanders:
                _log(f"Atualizando sinergia (EDHREC): {name}…")
                ok, err = fetch_one_commander(name, with_combos=True)
                if not ok:
                    _log(f"  não foi possível atualizar '{name}': {err}")
                    continue
                synergy_updated.append(name)

        _log("Concluído.")
        _finish("done", result={
            "cards": n_cards, "pt_names": n_pt, "synergy_updated": synergy_updated,
        })
    except BaseException as e:
        # BaseException, not Exception: mtgdb.cmd_build() can call sys.exit() on a
        # missing file, which raises SystemExit — must still flip state out of
        # "running" or a stuck job would block every future update attempt.
        _log(f"Erro: {e}")
        _finish("error", error=str(e))


def start(refresh_synergy=True):
    """Starts the update in a background thread. Returns False if one is already running."""
    if is_running():
        return False
    _begin()
    threading.Thread(target=_run, args=(refresh_synergy,), daemon=True).start()
    return True
