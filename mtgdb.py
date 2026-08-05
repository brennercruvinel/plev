#!/usr/bin/env python3
"""
Base de dados local de Magic, construída a partir do bulk data do Scryfall.

Lê os arquivos .jsonl.gz em streaming (sem descomprimir) e monta um índice SQLite,
permitindo consulta instantânea e offline de todas as cartas já lançadas —
inclusive busca pelo nome em português, útil porque a coleção física é em PT-BR.

Uso:
    ./mtgdb.py build                      # constrói/reconstrói o índice
    ./mtgdb.py card "Vona's Hunger"       # consulta (aceita nome em PT ou EN)
    ./mtgdb.py card "Fome de Vona"
    ./mtgdb.py search "sacrifice" --type Instant --color B
    ./mtgdb.py pt "Vento Pestilento"      # traduz PT -> EN
    ./mtgdb.py stats
"""

import argparse
import glob
import gzip
import json
import os
import sqlite3
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
DATA = os.path.join(HERE, "data")
DB = os.path.join(DATA, "mtg.sqlite")


def _newest(pattern):
    files = sorted(glob.glob(os.path.join(DATA, pattern)))
    return files[-1] if files else None


def _connect():
    if not os.path.exists(DB):
        sys.exit(f"Índice não encontrado. Rode primeiro:  {sys.argv[0]} build")
    con = sqlite3.connect(DB)
    con.row_factory = sqlite3.Row
    return con


def cmd_build(args):
    oracle_file = _newest("oracle-cards-*.jsonl.gz")
    all_file = _newest("all-cards-*.jsonl.gz")
    if not oracle_file:
        sys.exit("Arquivo oracle-cards-*.jsonl.gz não encontrado em data/")

    os.makedirs(DATA, exist_ok=True)
    if os.path.exists(DB):
        os.remove(DB)
    con = sqlite3.connect(DB)

    con.executescript("""
        CREATE TABLE cards (
            oracle_id TEXT PRIMARY KEY,
            name TEXT, mana_cost TEXT, cmc REAL, type_line TEXT,
            oracle_text TEXT, power TEXT, toughness TEXT, loyalty TEXT,
            colors TEXT, color_identity TEXT, rarity TEXT, set_code TEXT,
            keywords TEXT, commander_legal TEXT, price_usd TEXT,
            reserved INTEGER, edhrec_rank INTEGER, uri TEXT, image_uri TEXT
        );
        CREATE TABLE names_pt (
            printed_name TEXT, oracle_id TEXT, set_code TEXT
        );
        CREATE INDEX idx_name ON cards(name COLLATE NOCASE);
        CREATE INDEX idx_type ON cards(type_line);
        CREATE INDEX idx_pt ON names_pt(printed_name COLLATE NOCASE);
    """)

    def face(c, key):
        if c.get(key) is not None:
            return c[key]
        faces = c.get("card_faces") or []
        vals = [f.get(key) for f in faces if f.get(key)]
        return " // ".join(str(v) for v in vals) if vals else None

    def image_uri(c):
        iu = c.get("image_uris")
        if iu:
            return iu.get("normal") or iu.get("large") or iu.get("small")
        faces = c.get("card_faces") or []
        if faces and faces[0].get("image_uris"):
            fiu = faces[0]["image_uris"]
            return fiu.get("normal") or fiu.get("large") or fiu.get("small")
        return None

    print(f"Lendo {os.path.basename(oracle_file)} ...")
    rows, n = [], 0
    with gzip.open(oracle_file, "rt", encoding="utf-8") as fh:
        for line in fh:
            line = line.strip().rstrip(",")
            if not line or line in "[]":
                continue
            try:
                c = json.loads(line)
            except json.JSONDecodeError:
                continue
            if c.get("object") != "card":
                continue
            rows.append((
                c.get("oracle_id"), c.get("name"), face(c, "mana_cost"), c.get("cmc"),
                c.get("type_line"), face(c, "oracle_text"), face(c, "power"),
                face(c, "toughness"), c.get("loyalty"),
                "".join(c.get("colors") or []), "".join(c.get("color_identity") or []),
                c.get("rarity"), c.get("set"), ",".join(c.get("keywords") or []),
                (c.get("legalities") or {}).get("commander"),
                (c.get("prices") or {}).get("usd"),
                1 if c.get("reserved") else 0, c.get("edhrec_rank"),
                c.get("scryfall_uri"), image_uri(c),
            ))
            n += 1
            if len(rows) >= 5000:
                con.executemany(
                    "INSERT OR REPLACE INTO cards VALUES (%s)" % ",".join("?" * 20), rows)
                rows.clear()
    if rows:
        con.executemany(
            "INSERT OR REPLACE INTO cards VALUES (%s)" % ",".join("?" * 20), rows)
    con.commit()
    print(f"  {n:,} cartas únicas indexadas")

    if all_file:
        print(f"Lendo {os.path.basename(all_file)} para nomes em português ...")
        rows, n_pt, seen = [], 0, set()
        with gzip.open(all_file, "rt", encoding="utf-8") as fh:
            for line in fh:
                line = line.strip().rstrip(",")
                if not line or line in "[]":
                    continue
                try:
                    c = json.loads(line)
                except json.JSONDecodeError:
                    continue
                if c.get("lang") != "pt":
                    continue
                pn = c.get("printed_name")
                if not pn and c.get("card_faces"):
                    parts = [f.get("printed_name") for f in c["card_faces"] if f.get("printed_name")]
                    pn = " // ".join(parts) if parts else None
                oid = c.get("oracle_id")
                if not pn or not oid:
                    continue
                key = (pn.lower(), oid)
                if key in seen:
                    continue
                seen.add(key)
                rows.append((pn, oid, c.get("set")))
                n_pt += 1
                if len(rows) >= 5000:
                    con.executemany("INSERT INTO names_pt VALUES (?,?,?)", rows)
                    rows.clear()
        if rows:
            con.executemany("INSERT INTO names_pt VALUES (?,?,?)", rows)
        con.commit()
        print(f"  {n_pt:,} nomes em português indexados")
    else:
        print("  (all-cards não encontrado — busca por nome em PT indisponível)")

    con.execute("VACUUM")
    con.close()
    size = os.path.getsize(DB) / 1024 / 1024
    print(f"\nÍndice pronto: {DB} ({size:.1f} MB)")
    return 0


def fmt(r, verbose=True):
    out = [f"{r['name']}  {r['mana_cost'] or ''}".rstrip(), f"  {r['type_line']}"]
    if r["oracle_text"] and verbose:
        out.extend("  " + l for l in r["oracle_text"].split("\n"))
    if r["power"] is not None:
        out.append(f"  {r['power']}/{r['toughness']}")
    if r["loyalty"]:
        out.append(f"  Lealdade: {r['loyalty']}")
    if verbose:
        out.append(f"  Commander: {r['commander_legal']}  |  ~US$ {r['price_usd'] or '—'}"
                   f"  |  {r['rarity']}")
        if r["edhrec_rank"]:
            out.append(f"  EDHREC rank: {r['edhrec_rank']:,}")
        out.append(f"  {r['uri']}")
    return "\n".join(out)


def official_pt(con, oracle_id):
    """Todos os nomes oficiais em PT de uma carta, com as edições onde saíram.

    Vem do campo printed_name do Scryfall — o texto impresso na carta física.
    Nunca é tradução gerada; se não há impressão em PT, retorna lista vazia.
    """
    rows = con.execute(
        "SELECT printed_name, GROUP_CONCAT(DISTINCT set_code) sets FROM names_pt "
        "WHERE oracle_id = ? GROUP BY printed_name ORDER BY COUNT(*) DESC",
        (oracle_id,)).fetchall()
    return [(r["printed_name"], r["sets"]) for r in rows]


def _lookup(con, name):
    """Resolve um nome (EN ou PT oficial) para uma carta.

    Retorna (carta, rótulo_da_correspondência). Correspondências aproximadas são
    sempre rotuladas como tal — nunca devolvidas como se fossem exatas.
    """
    r = con.execute("SELECT * FROM cards WHERE name = ? COLLATE NOCASE", (name,)).fetchone()
    if r:
        return r, "exata (nome em inglês)"

    rows = con.execute(
        "SELECT DISTINCT oracle_id FROM names_pt WHERE printed_name = ? COLLATE NOCASE",
        (name,)).fetchall()
    if len(rows) == 1:
        r = con.execute("SELECT * FROM cards WHERE oracle_id = ?", (rows[0]["oracle_id"],)).fetchone()
        if r:
            return r, "exata (nome oficial em português)"
    elif len(rows) > 1:
        return None, f"AMBÍGUO: {len(rows)} cartas diferentes usam esse nome em português"

    # Correspondência parcial — sempre sinalizada, nunca silenciosa
    r = con.execute("SELECT * FROM cards WHERE name LIKE ? COLLATE NOCASE LIMIT 1",
                    (f"%{name}%",)).fetchone()
    if r:
        return r, "APROXIMADA (parcial, inglês) — confirme se é a carta certa"
    row = con.execute(
        "SELECT oracle_id, printed_name FROM names_pt WHERE printed_name LIKE ? COLLATE NOCASE LIMIT 1",
        (f"%{name}%",)).fetchone()
    if row:
        r = con.execute("SELECT * FROM cards WHERE oracle_id = ?", (row["oracle_id"],)).fetchone()
        if r:
            return r, f"APROXIMADA (parcial, português: '{row['printed_name']}') — confirme"
    return None, None


def cmd_card(args):
    con = _connect()
    r, how = _lookup(con, args.name)
    if not r:
        print(f"Carta não encontrada: {args.name}")
        if how:
            print(f"  {how}")
        return 1
    if how and not how.startswith("exata"):
        print(f"!! Correspondência {how}")
    print(fmt(r))

    pts = official_pt(con, r["oracle_id"])
    if not pts:
        print("  PT: sem impressão oficial em português "
              "(não existe nome oficial em PT para esta carta)")
    elif len(pts) == 1:
        print(f"  PT oficial: {pts[0][0]}")
    else:
        print("  PT oficial: MÚLTIPLOS nomes conforme a edição —")
        for nm, sets in pts:
            print(f"     '{nm}'  (sets: {sets})")
    return 0


def cmd_pt(args):
    """Traduz nome em português -> inglês, usando só os nomes oficiais impressos."""
    con = _connect()
    exact = con.execute(
        "SELECT DISTINCT n.printed_name, c.name, c.type_line, c.mana_cost, c.oracle_id "
        "FROM names_pt n JOIN cards c ON c.oracle_id = n.oracle_id "
        "WHERE n.printed_name = ? COLLATE NOCASE",
        (args.name,)).fetchall()
    rows = exact
    label = "exata"
    if not rows:
        rows = con.execute(
            "SELECT DISTINCT n.printed_name, c.name, c.type_line, c.mana_cost, c.oracle_id "
            "FROM names_pt n JOIN cards c ON c.oracle_id = n.oracle_id "
            "WHERE n.printed_name LIKE ? COLLATE NOCASE LIMIT 20",
            (f"%{args.name}%",)).fetchall()
        label = "APROXIMADA"

    if not rows:
        print(f"Nenhum nome OFICIAL em português corresponde a: {args.name}")
        print("  Ou a carta nunca foi impressa em PT, ou o nome está grafado diferente.")
        print("  Não invente uma tradução — confirme a grafia na carta física ou no Scryfall.")
        return 1

    if label != "exata":
        print(f"!! Sem correspondência exata; mostrando resultados APROXIMADOS — confirme:\n")
    if len(rows) > 1 and label == "exata":
        print(f"!! ATENÇÃO: {len(rows)} cartas distintas compartilham esse nome em PT\n")

    for r in rows:
        print(f"{r['printed_name']}  ->  {r['name']}  {r['mana_cost'] or ''}")
        print(f"    {r['type_line']}")
        outros = [nm for nm, _ in official_pt(con, r["oracle_id"]) if nm != r["printed_name"]]
        if outros:
            print(f"    (esta carta também saiu em PT como: {', '.join(outros)})")
    return 0


def cmd_search(args):
    con = _connect()
    sql = "SELECT * FROM cards WHERE 1=1"
    params = []
    if args.text:
        sql += " AND (oracle_text LIKE ? OR name LIKE ?)"
        params += [f"%{args.text}%", f"%{args.text}%"]
    if args.type:
        sql += " AND type_line LIKE ?"
        params.append(f"%{args.type}%")
    if args.color:
        sql += " AND color_identity = ?"
        params.append(args.color.upper())
    if args.commander_legal:
        sql += " AND commander_legal = 'legal'"
    if args.max_cmc is not None:
        sql += " AND cmc <= ?"
        params.append(args.max_cmc)
    sql += " ORDER BY CASE WHEN edhrec_rank IS NULL THEN 1 ELSE 0 END, edhrec_rank LIMIT ?"
    params.append(args.limit)

    rows = con.execute(sql, params).fetchall()
    print(f"{len(rows)} resultado(s)\n")
    for r in rows:
        print(fmt(r, verbose=not args.brief))
        print()
    return 0


def cmd_stats(args):
    con = _connect()
    n = con.execute("SELECT COUNT(*) FROM cards").fetchone()[0]
    pt = con.execute("SELECT COUNT(DISTINCT printed_name) FROM names_pt").fetchone()[0]
    print(f"Cartas únicas (Oracle IDs): {n:,}")
    print(f"Nomes distintos em português: {pt:,}")
    print(f"Banco: {DB} ({os.path.getsize(DB)/1024/1024:.1f} MB)")
    print("\nPor tipo principal:")
    for t in ("Land", "Creature", "Instant", "Sorcery", "Artifact",
              "Enchantment", "Planeswalker", "Battle"):
        c = con.execute("SELECT COUNT(*) FROM cards WHERE type_line LIKE ?",
                        (f"%{t}%",)).fetchone()[0]
        print(f"  {t:<14} {c:>7,}")
    return 0


def main():
    p = argparse.ArgumentParser(description="Base local de cartas de Magic (Scryfall bulk)")
    sub = p.add_subparsers(dest="cmd", required=True)

    b = sub.add_parser("build", help="constrói o índice SQLite a partir do bulk data")
    b.set_defaults(func=cmd_build)

    c = sub.add_parser("card", help="consulta uma carta (nome em inglês ou português)")
    c.add_argument("name")
    c.set_defaults(func=cmd_card)

    t = sub.add_parser("pt", help="traduz nome em português para o nome em inglês")
    t.add_argument("name")
    t.set_defaults(func=cmd_pt)

    s = sub.add_parser("search", help="busca local por texto/tipo/cor")
    s.add_argument("text", nargs="?")
    s.add_argument("--type")
    s.add_argument("--color", help="identidade de cor exata, ex: B")
    s.add_argument("--max-cmc", type=float)
    s.add_argument("--commander-legal", action="store_true")
    s.add_argument("--limit", type=int, default=15)
    s.add_argument("--brief", action="store_true")
    s.set_defaults(func=cmd_search)

    st = sub.add_parser("stats", help="estatísticas do índice local")
    st.set_defaults(func=cmd_stats)

    args = p.parse_args()
    sys.exit(args.func(args))


if __name__ == "__main__":
    main()
