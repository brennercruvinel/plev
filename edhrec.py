#!/usr/bin/env python3
"""
Local cache of EDHREC card synergy data, for OFFLINE deckbuilding lookups.

Key difference from scryfall.py/mtgdb.py: EDHREC doesn't publish an official
dump of its whole database. Each commander/theme is its own page. So the flow
here is two steps:

  1. `fetch` — needs internet, fetches and SAVES the page locally
  2. `recs` / `combos` / `similar` — always offline, only reads what's saved

Fetch the commanders you play (or are considering) while you have internet at
home; after that, lookups work anywhere, no network needed.

Uses the JSON endpoints EDHREC's own site consumes (json.edhrec.com) — not an
officially documented/supported API, so it can change without notice. Because
of that: only fetch on demand (one commander at a time, when you're actually
about to use it), never crawl the whole site.

Usage (CLI output is in Portuguese, matching the owner's daily workflow):
    ./edhrec.py fetch "Syr Konrad, the Grim"     # fetches and caches (needs net)
    ./edhrec.py fetch "Syr Konrad, the Grim" --combos   # also fetches combos
    ./edhrec.py recs "Syr Konrad, the Grim"      # reads from cache (offline)
    ./edhrec.py recs "Syr Konrad, the Grim" --list "High Synergy Cards"
    ./edhrec.py combos "Syr Konrad, the Grim"    # known combos (offline)
    ./edhrec.py similar "Syr Konrad, the Grim"   # similar commanders (offline)
    ./edhrec.py list                              # what's already cached
"""

import argparse
import glob
import json
import os
import re
import sys
import time
import unicodedata
import urllib.error
import urllib.request

HERE = os.path.dirname(os.path.abspath(__file__))
CACHE = os.path.join(HERE, "data", "edhrec")
BASE = "https://json.edhrec.com/pages"
USER_AGENT = "SpellbookMTG/1.0 (https://github.com/ofelipenavarro/spellbook-mtg; on-demand personal lookup)"
MIN_INTERVAL = 0.5  # no official public API: be polite to their server

_last_request = 0.0


def slugify(name):
    """Converts a card/commander name to EDHREC's slug format."""
    name = unicodedata.normalize("NFKD", name).encode("ascii", "ignore").decode()
    name = name.lower()
    name = re.sub(r"[^a-z0-9]+", "-", name)
    return name.strip("-")


def _fetch_json(url):
    global _last_request
    elapsed = time.time() - _last_request
    if elapsed < MIN_INTERVAL:
        time.sleep(MIN_INTERVAL - elapsed)
    req = urllib.request.Request(url, headers={"User-Agent": USER_AGENT, "Accept": "application/json"})
    try:
        with urllib.request.urlopen(req, timeout=20) as resp:
            _last_request = time.time()
            return json.load(resp), None
    except urllib.error.HTTPError as e:
        _last_request = time.time()
        return None, f"HTTP {e.code}"
    except urllib.error.URLError as e:
        _last_request = time.time()
        return None, f"sem conexão ({e.reason})"


def _cache_path(kind, slug):
    return os.path.join(CACHE, kind, f"{slug}.json")


def cmd_fetch(args):
    slug = slugify(args.name)
    os.makedirs(os.path.join(CACHE, "commanders"), exist_ok=True)

    data, err = _fetch_json(f"{BASE}/commanders/{slug}.json")
    if err:
        print(f"ERRO ao buscar '{args.name}' (slug: {slug}): {err}")
        print("Sem cache local para esta carta ainda — precisa de internet na primeira vez.")
        return 1

    path = _cache_path("commanders", slug)
    with open(path, "w", encoding="utf-8") as fh:
        json.dump(data, fh, ensure_ascii=False)
    cardlists = data.get("container", {}).get("json_dict", {}).get("cardlists", [])
    n = sum(len(c.get("cardviews", [])) for c in cardlists)
    print(f"Cacheado: {args.name} -> {path}")
    print(f"  {len(cardlists)} listas, {n} recomendações de carta ao todo")
    similar = data.get("similar") or []
    if similar:
        print(f"  Comandantes parecidos: {', '.join(similar[:5])}")

    if args.combos:
        os.makedirs(os.path.join(CACHE, "combos"), exist_ok=True)
        cdata, cerr = _fetch_json(f"{BASE}/combos/{slug}.json")
        if cerr:
            print(f"  (combos não obtidos: {cerr})")
        else:
            cpath = _cache_path("combos", slug)
            with open(cpath, "w", encoding="utf-8") as fh:
                json.dump(cdata, fh, ensure_ascii=False)
            clists = cdata.get("container", {}).get("json_dict", {}).get("cardlists", [])
            print(f"  Combos cacheados: {len(clists)} combos conhecidos -> {cpath}")
    return 0


def _load_cached(kind, name):
    slug = slugify(name)
    path = _cache_path(kind, slug)
    if not os.path.exists(path):
        return None, slug
    with open(path, encoding="utf-8") as fh:
        return json.load(fh), slug


def cmd_recs(args):
    data, slug = _load_cached("commanders", args.name)
    if data is None:
        print(f"'{args.name}' (slug: {slug}) não está no cache.")
        print(f"  Rode com internet primeiro:  ./edhrec.py fetch \"{args.name}\"")
        return 1

    cardlists = data.get("container", {}).get("json_dict", {}).get("cardlists", [])
    if args.list:
        cardlists = [c for c in cardlists if args.list.lower() in c.get("header", "").lower()]
        if not cardlists:
            print(f"Nenhuma lista com '{args.list}' no nome. Listas disponíveis:")
            for c in data.get("container", {}).get("json_dict", {}).get("cardlists", []):
                print(f"  - {c.get('header')}")
            return 1

    for c in cardlists:
        views = c.get("cardviews", [])
        print(f"=== {c.get('header')} ({len(views)}) ===")
        for v in views[: args.limit]:
            syn = v.get("synergy")
            nd = v.get("num_decks")
            bits = []
            if syn is not None:
                bits.append(f"sinergia {syn:+.2f}")
            if nd is not None:
                bits.append(f"{nd:,} decks")
            extra = f"  ({', '.join(bits)})" if bits else ""
            print(f"  {v['name']}{extra}")
        print()
    return 0


def cmd_combos(args):
    data, slug = _load_cached("combos", args.name)
    if data is None:
        print(f"Combos de '{args.name}' (slug: {slug}) não estão no cache.")
        print(f"  Rode com internet primeiro:  ./edhrec.py fetch \"{args.name}\" --combos")
        return 1

    cardlists = data.get("container", {}).get("json_dict", {}).get("cardlists", [])
    if not cardlists:
        print("Nenhum combo catalogado para esta carta.")
        return 0

    print(f"{len(cardlists)} combo(s) conhecido(s) envolvendo {args.name}:\n")
    for c in cardlists[: args.limit]:
        combo = c.get("combo", {})
        names = [v["name"] for v in c.get("cardviews", [])]
        print(f"  {' + '.join(names)}")
        results = combo.get("results") or []
        if results:
            print(f"    Resultado: {'; '.join(results)}")
        if combo.get("count"):
            print(f"    Usado em {combo['count']:,} decks catalogados")
        print()
    return 0


def cmd_similar(args):
    data, slug = _load_cached("commanders", args.name)
    if data is None:
        print(f"'{args.name}' (slug: {slug}) não está no cache.")
        print(f"  Rode com internet primeiro:  ./edhrec.py fetch \"{args.name}\"")
        return 1
    similar = data.get("similar") or []
    if not similar:
        print("Nenhum comandante parecido listado.")
        return 0
    print(f"Comandantes com estilo parecido a {args.name}:")
    for s in similar:
        print(f"  - {s}")
    return 0


def cmd_list(args):
    for kind in ("commanders", "combos"):
        files = sorted(glob.glob(os.path.join(CACHE, kind, "*.json")))
        print(f"{kind} ({len(files)}):")
        for f in files:
            print(f"  - {os.path.splitext(os.path.basename(f))[0]}")
        print()
    return 0


def main():
    p = argparse.ArgumentParser(description="Cache offline de sinergia do EDHREC")
    sub = p.add_subparsers(dest="cmd", required=True)

    f = sub.add_parser("fetch", help="busca e cacheia um comandante (precisa de internet)")
    f.add_argument("name")
    f.add_argument("--combos", action="store_true", help="também buscar combos conhecidos")
    f.set_defaults(func=cmd_fetch)

    r = sub.add_parser("recs", help="recomendações de carta para um comandante (offline)")
    r.add_argument("name")
    r.add_argument("--list", help="filtra por nome de lista, ex: 'High Synergy'")
    r.add_argument("--limit", type=int, default=15)
    r.set_defaults(func=cmd_recs)

    c = sub.add_parser("combos", help="combos catalogados envolvendo a carta (offline)")
    c.add_argument("name")
    c.add_argument("--limit", type=int, default=20)
    c.set_defaults(func=cmd_combos)

    s = sub.add_parser("similar", help="comandantes com estilo parecido (offline)")
    s.add_argument("name")
    s.set_defaults(func=cmd_similar)

    sub.add_parser("list", help="mostra o que já está cacheado").set_defaults(func=cmd_list)

    args = p.parse_args()
    sys.exit(args.func(args))


if __name__ == "__main__":
    main()
