#!/usr/bin/env python3
"""Rewrite path literals in Rust sources after the moves.

Two shapes are rewritten through migration.toml, the ones whose meaning is
fixed by the file that holds them:

- `include_str!("rel")` / `include_bytes!("rel")`: `rel` is relative to the
  file's own directory, before and after;
- a string literal starting with `../` or naming a path under the crate on a
  line that mentions `CARGO_MANIFEST_DIR` (or the next two lines): relative to
  the crate root, before and after.

A literal whose old target is a directory is translated by the files under
it (all of them must land under one new directory). What cannot be resolved
is printed as `manual:`.

    python3 xtask/scripts/litfix.py [--check] [files...]
"""
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import plevmap  # noqa: E402

ROOT = plevmap.ROOT
REV = plevmap.reverse_moves()
MOVES = {}
for _e in plevmap.load():
    if _e["kind"] in ("move", "merge") and _e["from"] and _e["to"]:
        MOVES[_e["from"]] = _e["to"]
MANUAL = []


def old_dir_target(old):
    """New location of an old directory: the common new parent of its files."""
    files = [(f, t) for f, t in MOVES.items() if f.startswith(old.rstrip("/") + "/")]
    if not files:
        return None
    news = set()
    for f, t in files:
        rest = f[len(old.rstrip("/")) + 1:]
        if not t.endswith(rest):
            # renamed inside: fall back to the common prefix of all targets
            news.add(None)
            continue
        news.add(t[: len(t) - len(rest)].rstrip("/"))
    if len(news) == 1 and None not in news:
        return news.pop()
    common = os.path.commonpath([t for _, t in files])
    return common


def new_of(old_path):
    old_path = os.path.normpath(old_path)
    if old_path in MOVES:
        return MOVES[old_path]
    return old_dir_target(old_path) or None


def rel(target, base_dir):
    r = os.path.relpath(target, base_dir)
    return r


def fix(path, check):
    old = REV.get(path)
    if not old:
        return False
    full = os.path.join(ROOT, path)
    src = open(full, encoding="utf-8").read()
    out = src
    old_dir, new_dir = os.path.dirname(old), os.path.dirname(path)
    oc = plevmap.crate_of(old, plevmap.OLD_CRATES)
    nc = plevmap.crate_of(path, plevmap.NEW_CRATES)

    def inc(m):
        lit = m.group(2)
        tgt = new_of(os.path.join(old_dir, lit))
        if tgt is None:
            MANUAL.append(f"{path}: include {lit}")
            return m.group(0)
        new = rel(tgt, new_dir)
        if new != lit:
            print(f"  {path}: include \"{lit}\" -> \"{new}\"")
        return f'{m.group(1)}"{new}"' if new != lit else m.group(0)

    out = re.sub(r'(include_(?:str|bytes)!\(\s*)"([^"]+)"', inc, out)

    if oc and nc:
        lines = out.split("\n")
        for i, line in enumerate(lines):
            window = "\n".join(lines[max(0, i - 2): i + 1])
            if "CARGO_MANIFEST_DIR" not in window:
                continue

            def man(m):
                lit = m.group(1)
                if lit.startswith("/"):
                    lit2 = lit[1:]
                else:
                    lit2 = lit
                if not lit2 or "{" in lit2 or not re.match(r"^[\w.@-]+(/[\w.@ -]+)*/?$", lit2):
                    return m.group(0)
                old_abs = os.path.normpath(os.path.join(oc, lit2))
                if not (os.path.exists(os.path.join(ROOT, "boiler")) and True):
                    pass
                tgt = new_of(old_abs)
                if tgt is None:
                    # a path that does not exist in the map (tmp output, etc.)
                    if old_abs.startswith(oc + "/") and lit2.split("/")[0] in ("tests", "src", "assets", "fixtures", "examples", "benches"):
                        MANUAL.append(f"{path}:{i + 1}: manifest-relative {lit}")
                    return m.group(0)
                new = rel(tgt, nc)
                if lit.startswith("/"):
                    new = "/" + new
                if new != lit:
                    print(f"  {path}:{i + 1}: \"{lit}\" -> \"{new}\"")
                return f'"{new}"' if new != lit else m.group(0)

            lines[i] = re.sub(r'"([^"\n]*)"', man, line)
        out = "\n".join(lines)
    if out != src:
        print("changed", path)
        if not check:
            open(full, "w", encoding="utf-8").write(out)
        return True
    return False


def main(argv):
    check = "--check" in argv
    files = [a for a in argv if not a.startswith("--")] or plevmap.rs_files(ROOT)
    n = sum(fix(f, check) for f in files)
    for m in MANUAL:
        print("manual:", m)
    print(f"{n} files changed, {len(MANUAL)} manual")


if __name__ == "__main__":
    main(sys.argv[1:])
