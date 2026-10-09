#!/usr/bin/env python3
"""Find Rust paths inside string literals that still name the old modules.

Generators write Rust source as strings (`"use engine::builder::{..}"`);
pathfix rewrote code and textfix comments, neither of them strings. A path
in a string whose crate is an old lib name and whose module prefix is an
old module that is no module of the target layout is printed with its
translation; `--write` rewrites it.

    python3 xtask/scripts/strpaths.py [--write]
"""
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import plevmap  # noqa: E402

TABLE = plevmap.module_table()
NEWMODS = set()
for d in plevmap.NEW_CRATES:
    for f in plevmap.rs_files(plevmap.ROOT):
        rel = os.path.relpath(f, plevmap.ROOT)
        if rel.startswith(d + "/"):
            m = plevmap.lib_module(rel, d)
            if m is not None:
                for n in range(len(m) + 1):
                    NEWMODS.add((plevmap.NEW_CRATES[d], tuple(m[:n])))
OLD_LIBS = set(plevmap.OLD_CRATES.values())
STR = re.compile(r'"(?:[^"\\\n]|\\.)*"')
PATH = re.compile(r"(?<![A-Za-z0-9_:])([a-z_][a-z0-9_]*)((?:::[A-Za-z_][A-Za-z0-9_]*)+)")


def fix_path(m):
    lib, rest = m.group(1), m.group(2)
    if lib not in OLD_LIBS:
        return None
    segs = rest[2:].split("::")
    mods = []
    for s in segs:
        if not re.match(r"^[a-z_][a-z0-9_]*$", s):
            break
        mods.append(s)
    # how far the path stays a module of the new layout: then it is new already
    nlib = plevmap.CRATE_RENAMES.get(lib, lib)
    k = 0
    while k < len(mods) and (nlib, tuple(mods[:k + 1])) in NEWMODS:
        k += 1
    if lib == nlib and k == len(mods):
        return None
    # the old module it named, at least one segment deep
    hit = None
    for n in range(len(mods), 0, -1):
        if (lib, tuple(mods[:n])) in TABLE:
            hit = n
            break
    if hit is None and lib == nlib:
        return None
    new_lib, new_segs = plevmap.translate(TABLE, lib, segs)
    new = new_lib + "::" + "::".join(new_segs) if new_segs else new_lib
    old = lib + rest
    return None if new == old else new


def main(argv):
    write = "--write" in argv
    total = 0
    for f in plevmap.rs_files(plevmap.ROOT):
        rel = os.path.relpath(f, plevmap.ROOT)
        src = open(f, encoding="utf-8").read()
        hits = []

        def sub_str(sm):
            s = sm.group(0)

            def sub_path(pm):
                new = fix_path(pm)
                if new is None:
                    return pm.group(0)
                hits.append((pm.group(0), new))
                return new
            return PATH.sub(sub_path, s)
        out = STR.sub(sub_str, src)
        for old, new in hits:
            line = src[: src.find(old)].count("\n") + 1
            print(f"{rel}:{line}: {old} -> {new}")
        total += len(hits)
        if write and out != src:
            open(f, "w", encoding="utf-8").write(out)
    print(total, "paths")


if __name__ == "__main__":
    main(sys.argv[1:])
