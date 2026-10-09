#!/usr/bin/env python3
"""`super::super::...` chains that climb to the crate root become `crate::`.

pathfix.py re-expresses a moved module's super:: paths relative to its new
place; when the only module both share is the crate root, the chain climbs
all the way up. `crate::` says the same thing in one word.

    python3 xtask/scripts/supertidy.py [--check] [files...]
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import plevmap  # noqa: E402
import pathfix  # noqa: E402


def tidy(path, check):
    nc = plevmap.crate_of(path, plevmap.NEW_CRATES)
    mod = plevmap.lib_module(path, nc) if nc else None
    if mod is None:
        return False
    full = os.path.join(plevmap.ROOT, path)
    src = open(full, encoding="utf-8").read()
    toks = [t for t in pathfix.tokenize(src) if t[0] not in ("ws", "skip")]
    stack, edits, i = [], [], 0
    while i < len(toks):
        k, text, pos = toks[i]
        if k == "brace":
            if text == "{":
                stack.append(i >= 2 and toks[i - 2][1] == "mod" and toks[i - 1][0] == "ident")
            elif stack:
                stack.pop()
            i += 1
            continue
        prev = toks[i - 1] if i else None
        if text == "super" and not (prev and prev[0] == "sep"):
            j, n = i, 0
            while j < len(toks) and toks[j][1] == "super" and j + 1 < len(toks) and toks[j + 1][0] == "sep":
                n += 1
                j += 2
            if n >= 2 and n == len(mod) + sum(stack):
                end = toks[j - 1][2] + 2  # through the last `::`
                edits.append((pos, end, "crate::"))
            i = max(j, i + 1)
            continue
        i += 1
    if not edits:
        return False
    out, last = [], 0
    for s, e, t in edits:
        out += [src[last:s], t]
        last = e
    out.append(src[last:])
    print("changed", path, len(edits))
    if not check:
        open(full, "w", encoding="utf-8").write("".join(out))
    return True


def main(argv):
    check = "--check" in argv
    files = [a for a in argv if not a.startswith("--")] or plevmap.rs_files(plevmap.ROOT)
    print(sum(tidy(f, check) for f in files), "files changed")


if __name__ == "__main__":
    main(sys.argv[1:])
