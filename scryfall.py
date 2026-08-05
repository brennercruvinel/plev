#!/usr/bin/env python3
"""
Command-line client for the Scryfall API.

No external dependencies (stdlib only) and no API key — Scryfall's API is
public. Honors Scryfall's requested usage limits: identifiable User-Agent,
Accept: application/json, and a minimum interval between requests.

Usage (CLI output is in Portuguese, matching the owner's daily workflow):
    ./scryfall.py card "Syr Konrad, the Grim"      # exact lookup (falls back to fuzzy)
    ./scryfall.py search "c:b t:instant cmc<=2"    # Scryfall search syntax
    ./scryfall.py rulings "Syr Konrad, the Grim"   # official rulings for the card
    ./scryfall.py deck lista.md                    # validates a whole decklist
    ./scryfall.py bulk                             # lists available bulk data files
    ./scryfall.py bulk oracle_cards --download     # downloads a bulk data file

Human-readable output by default; use --json for the raw Scryfall object.
"""

import argparse
import json
import re
import sys
import time
import urllib.error
import urllib.parse
import urllib.request

API = "https://api.scryfall.com"
USER_AGENT = "FelipeNavarroMTGVault/1.0"
MIN_INTERVAL = 0.1  # Scryfall asks for 50-100ms between requests

_last_request = 0.0


def _get(path, params=None, base=API):
    """GET with rate limiting and the headers Scryfall requires."""
    global _last_request
    elapsed = time.time() - _last_request
    if elapsed < MIN_INTERVAL:
        time.sleep(MIN_INTERVAL - elapsed)

    url = f"{base}{path}"
    if params:
        url += "?" + urllib.parse.urlencode(params)
    req = urllib.request.Request(
        url, headers={"User-Agent": USER_AGENT, "Accept": "application/json"}
    )
    try:
        with urllib.request.urlopen(req, timeout=30) as resp:
            _last_request = time.time()
            return json.load(resp)
    except urllib.error.HTTPError as e:
        _last_request = time.time()
        try:
            return json.load(e)
        except Exception:
            return {"object": "error", "status": e.code, "details": str(e)}


def _post(path, payload):
    """POST with rate limiting (used by the /cards/collection endpoint)."""
    global _last_request
    elapsed = time.time() - _last_request
    if elapsed < MIN_INTERVAL:
        time.sleep(MIN_INTERVAL - elapsed)

    req = urllib.request.Request(
        f"{API}{path}",
        data=json.dumps(payload).encode(),
        headers={
            "User-Agent": USER_AGENT,
            "Accept": "application/json",
            "Content-Type": "application/json",
        },
    )
    try:
        with urllib.request.urlopen(req, timeout=30) as resp:
            _last_request = time.time()
            return json.load(resp)
    except urllib.error.HTTPError as e:
        _last_request = time.time()
        try:
            return json.load(e)
        except Exception:
            return {"object": "error", "status": e.code, "details": str(e)}


def fmt_card(c, verbose=True):
    """Formats a card object for terminal reading."""
    if c.get("object") == "error":
        return f"  ERRO: {c.get('details', 'desconhecido')}"

    parts = [f"{c['name']}  {c.get('mana_cost', '')}".rstrip()]
    parts.append(f"  {c.get('type_line', '')}")

    text = c.get("oracle_text")
    if not text and "card_faces" in c:
        text = "\n---\n".join(
            f"{f.get('name','')} {f.get('mana_cost','')}\n{f.get('type_line','')}\n{f.get('oracle_text','')}"
            for f in c["card_faces"]
        )
    if text and verbose:
        parts.extend("  " + line for line in text.split("\n"))

    if c.get("power") is not None:
        parts.append(f"  {c['power']}/{c['toughness']}")
    if c.get("loyalty"):
        parts.append(f"  Lealdade: {c['loyalty']}")

    if verbose:
        pr = c.get("prices", {}) or {}
        price_bits = [f"{k}: {v}" for k, v in pr.items() if v]
        if price_bits:
            parts.append("  Preços — " + " | ".join(price_bits))
        legal = c.get("legalities", {}) or {}
        parts.append(f"  Commander: {legal.get('commander', '?')}")
        parts.append(f"  {c.get('scryfall_uri', '')}")

    return "\n".join(parts)


def cmd_card(args):
    key = "exact" if not args.fuzzy else "fuzzy"
    data = _get("/cards/named", {key: args.name})
    if data.get("object") == "error" and not args.fuzzy:
        data = _get("/cards/named", {"fuzzy": args.name})
    if args.json:
        print(json.dumps(data, indent=2, ensure_ascii=False))
    else:
        print(fmt_card(data))
    return 0 if data.get("object") != "error" else 1


def cmd_search(args):
    data = _get("/cards/search", {"q": args.query, "order": args.order})
    if data.get("object") == "error":
        print(f"ERRO: {data.get('details')}")
        return 1
    if args.json:
        print(json.dumps(data, indent=2, ensure_ascii=False))
        return 0

    total = data.get("total_cards", 0)
    print(f"{total} carta(s) encontrada(s)\n")
    for c in data.get("data", [])[: args.limit]:
        print(fmt_card(c, verbose=not args.brief))
        print()
    if total > args.limit:
        print(f"... mostrando {args.limit} de {total}. Use --limit para ver mais.")
    return 0


def cmd_rulings(args):
    card = _get("/cards/named", {"fuzzy": args.name})
    if card.get("object") == "error":
        print(f"ERRO: {card.get('details')}")
        return 1
    data = _get(f"/cards/{card['id']}/rulings")
    print(f"Rulings — {card['name']}\n")
    rulings = data.get("data", [])
    if not rulings:
        print("  (nenhum ruling oficial)")
    for r in rulings:
        print(f"  [{r['published_at']}] {r['comment']}\n")
    return 0


DECKLINE = re.compile(r"^\s*(?:(\d+)\s*x?\s+)?(.+?)\s*$")


def parse_decklist(path):
    """Reads a text decklist: '1 Card Name', '1x Name', or just 'Name'."""
    entries = []
    with open(path, encoding="utf-8") as fh:
        for raw in fh:
            line = raw.strip().rstrip("\\")
            if not line or line.startswith(("#", "//", "---", "=")):
                continue
            m = DECKLINE.match(line)
            if not m:
                continue
            qty, name = m.group(1), m.group(2).strip()
            name = re.sub(r"\s*\([^)]*\)\s*$", "", name)  # strip trailing "(set)"
            if name:
                entries.append((int(qty) if qty else 1, name))
    return entries


def cmd_deck(args):
    entries = parse_decklist(args.file)
    if not entries:
        print("Nenhuma carta encontrada no arquivo.")
        return 1

    print(f"Verificando {len(entries)} entrada(s) de {args.file}\n")
    found, missing = [], []

    # /cards/collection accepts up to 75 identifiers per request
    for i in range(0, len(entries), 75):
        chunk = entries[i : i + 75]
        payload = {"identifiers": [{"name": n} for _, n in chunk]}
        data = _post("/cards/collection", payload)
        if data.get("object") == "error":
            print(f"ERRO: {data.get('details')}")
            return 1
        found.extend(data.get("data", []))
        missing.extend(data.get("not_found", []))

    qty_by_name = {}
    for qty, name in entries:
        qty_by_name[name.lower()] = qty

    by_type = {}
    total_cards = 0
    for c in found:
        qty = qty_by_name.get(c["name"].lower(), 1)
        total_cards += qty
        main_type = c.get("type_line", "?").split("—")[0].strip()
        for t in ("Land", "Creature", "Instant", "Sorcery", "Artifact",
                  "Enchantment", "Planeswalker", "Battle"):
            if t in main_type:
                main_type = t
                break
        by_type.setdefault(main_type, []).append((qty, c))

    for t in sorted(by_type):
        cards = by_type[t]
        count = sum(q for q, _ in cards)
        print(f"=== {t} ({count}) ===")
        for qty, c in sorted(cards, key=lambda x: x[1]["name"]):
            price = (c.get("prices") or {}).get("usd") or "—"
            print(f"  {qty}x {c['name']:<38} {c.get('mana_cost',''):<14} ${price}")
        print()

    print(f"TOTAL: {total_cards} cartas ({len(found)} nomes distintos reconhecidos)")

    if missing:
        print(f"\n!! NÃO ENCONTRADAS ({len(missing)}) — verifique a grafia:")
        for m in missing:
            print(f"   - {m.get('name')}")
        return 1
    return 0


def cmd_bulk(args):
    data = _get("/bulk-data")
    items = data.get("data", [])

    if not args.type:
        print("Arquivos de bulk data disponíveis no Scryfall:\n")
        for x in items:
            mb = x.get("compressed_size", 0) / 1024 / 1024
            print(f"  {x['type']:<16} {mb:>7.1f} MB (gz)  atualizado {x['updated_at'][:10]}")
            print(f"      {x['description'][:100]}")
        print("\nUse: scryfall.py bulk <tipo> --download")
        return 0

    match = next((x for x in items if x["type"] == args.type), None)
    if not match:
        print(f"Tipo desconhecido: {args.type}")
        return 1

    uri = match.get("download_uri") or match.get("jsonl_download_uri")
    mb = match.get("compressed_size", 0) / 1024 / 1024
    if not args.download:
        print(f"{match['name']} — {mb:.1f} MB comprimido")
        print(f"Atualizado: {match['updated_at']}")
        print(f"URL: {uri}")
        print("\nAdicione --download para baixar.")
        return 0

    dest = args.output or uri.split("/")[-1]
    print(f"Baixando {match['name']} ({mb:.1f} MB) para {dest} ...")
    req = urllib.request.Request(uri, headers={"User-Agent": USER_AGENT})
    with urllib.request.urlopen(req, timeout=600) as resp, open(dest, "wb") as out:
        while True:
            block = resp.read(1 << 20)
            if not block:
                break
            out.write(block)
    print(f"Pronto: {dest}")
    return 0


def main():
    p = argparse.ArgumentParser(description="Cliente da API do Scryfall")
    sub = p.add_subparsers(dest="cmd", required=True)

    c = sub.add_parser("card", help="busca uma carta pelo nome")
    c.add_argument("name")
    c.add_argument("--fuzzy", action="store_true", help="busca aproximada direto")
    c.add_argument("--json", action="store_true")
    c.set_defaults(func=cmd_card)

    s = sub.add_parser("search", help="busca com a sintaxe do Scryfall")
    s.add_argument("query")
    s.add_argument("--limit", type=int, default=10)
    s.add_argument("--order", default="name")
    s.add_argument("--brief", action="store_true", help="só nome, custo e tipo")
    s.add_argument("--json", action="store_true")
    s.set_defaults(func=cmd_search)

    r = sub.add_parser("rulings", help="rulings oficiais de uma carta")
    r.add_argument("name")
    r.set_defaults(func=cmd_rulings)

    d = sub.add_parser("deck", help="verifica uma decklist em arquivo texto")
    d.add_argument("file")
    d.set_defaults(func=cmd_deck)

    b = sub.add_parser("bulk", help="lista/baixa arquivos de bulk data")
    b.add_argument("type", nargs="?")
    b.add_argument("--download", action="store_true")
    b.add_argument("--output")
    b.set_defaults(func=cmd_bulk)

    args = p.parse_args()
    sys.exit(args.func(args))


if __name__ == "__main__":
    main()
