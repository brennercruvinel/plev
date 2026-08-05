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
immediately; progress (a single task label + an overall 0-100 percent
spanning every phase) is exposed via get_status() for the frontend to poll.
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

# how much of the overall 0-100 bar each phase accounts for
DOWNLOAD_WEIGHT = 70
BUILD_WEIGHT = 20
SYNERGY_WEIGHT = 10

_status = {
    "state": "idle",  # idle | running | done | error
    "task": None,      # current task label, no file sizes — this is what the UI shows
    "percent": 0,
    "started_at": None,
    "finished_at": None,
    "error": None,
    "result": None,
}
_lock = threading.Lock()


def get_status():
    with _lock:
        return dict(_status)


def is_running():
    with _lock:
        return _status["state"] == "running"


def _progress(task, percent):
    with _lock:
        _status["task"] = task
        _status["percent"] = max(0, min(100, round(percent, 1)))


def _begin():
    with _lock:
        _status["state"] = "running"
        _status["task"] = None
        _status["percent"] = 0
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
        if state == "done":
            _status["percent"] = 100


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


def _download_bulk_file(match, on_bytes):
    """Streams one Scryfall bulk data file to disk, reporting bytes read via on_bytes(n)."""
    uri = match.get("download_uri") or match.get("jsonl_download_uri")
    filename = uri.split("/")[-1].split("?")[0]
    dest = os.path.join(DATA_DIR, filename)

    req = urllib.request.Request(uri, headers={"User-Agent": scryfall.USER_AGENT})
    tmp_dest = dest + ".part"
    with urllib.request.urlopen(req, timeout=600) as resp, open(tmp_dest, "wb") as out:
        while True:
            block = resp.read(1 << 20)
            if not block:
                break
            out.write(block)
            on_bytes(len(block))
    os.replace(tmp_dest, dest)
    return dest


def _cleanup_old_bulk_files(keep):
    """Removes previous downloads of the same kind so mtgdb.py's "newest" glob never picks a stale one."""
    for prefix in ("oracle-cards-", "all-cards-"):
        for f in glob.glob(os.path.join(DATA_DIR, f"{prefix}*.jsonl.gz")):
            if f not in keep:
                os.remove(f)


def _run(refresh_synergy):
    try:
        os.makedirs(DATA_DIR, exist_ok=True)

        task = "Baixando base de cartas do Scryfall…"
        _progress(task, 0)
        listing = scryfall._get("/bulk-data")
        items = listing.get("data", [])
        oracle_match = next((x for x in items if x["type"] == "oracle_cards"), None)
        all_match = next((x for x in items if x["type"] == "all_cards"), None)
        if not oracle_match or not all_match:
            raise RuntimeError("Scryfall não retornou os arquivos de bulk data esperados.")

        total_bytes = (oracle_match.get("compressed_size") or 0) + (all_match.get("compressed_size") or 0)
        downloaded = {"n": 0}

        def on_bytes(n):
            downloaded["n"] += n
            frac = (downloaded["n"] / total_bytes) if total_bytes else 0
            _progress(task, DOWNLOAD_WEIGHT * min(1.0, frac))

        oracle_dest = _download_bulk_file(oracle_match, on_bytes)
        all_dest = _download_bulk_file(all_match, on_bytes)
        _cleanup_old_bulk_files(keep={oracle_dest, all_dest})

        task = "Reconstruindo o índice local…"
        _progress(task, DOWNLOAD_WEIGHT)
        mtgdb.cmd_build(None)

        cdb = get_cards_db()
        n_cards = cdb.execute("SELECT COUNT(*) FROM cards").fetchone()[0]
        n_pt = cdb.execute("SELECT COUNT(DISTINCT printed_name) FROM names_pt").fetchone()[0]
        cdb.close()
        _progress(task, DOWNLOAD_WEIGHT + BUILD_WEIGHT)

        synergy_updated = []
        if refresh_synergy:
            con = get_app_db()
            commanders = sorted({
                r["commander_name"]
                for r in con.execute("SELECT DISTINCT commander_name FROM decks").fetchall()
            })
            con.close()
            n = len(commanders)
            for i, name in enumerate(commanders):
                _progress(f"Atualizando sinergia: {name}…", DOWNLOAD_WEIGHT + BUILD_WEIGHT + SYNERGY_WEIGHT * i / max(n, 1))
                ok, _err = fetch_one_commander(name, with_combos=True)
                if ok:
                    synergy_updated.append(name)

        _progress("Concluído.", 100)
        _finish("done", result={
            "cards": n_cards, "pt_names": n_pt, "synergy_updated": synergy_updated,
        })
    except BaseException as e:
        # BaseException, not Exception: mtgdb.cmd_build() can call sys.exit() on a
        # missing file, which raises SystemExit — must still flip state out of
        # "running" or a stuck job would block every future update attempt.
        _progress(f"Erro: {e}", get_status()["percent"])
        _finish("error", error=str(e))


def start(refresh_synergy=True):
    """Starts the update in a background thread. Returns False if one is already running."""
    if is_running():
        return False
    _begin()
    threading.Thread(target=_run, args=(refresh_synergy,), daemon=True).start()
    return True
