#!/usr/bin/env python3
"""Compare the public API before and after the move, through the map.

Reads `cargo public-api -sss` listings: <before>/<old package>.txt and
<after>/<new package>.txt. Every path in a line before is translated
(xtask/tests/modpaths.txt, one pass, longest module first, the same rule
the behaviour probes use) and the line is filed under the crate it was
renamed or merged into; then each new crate's set is compared with its
listing after, and every difference is printed.

    python3 xtask/scripts/apidiff.py <before dir> <after dir>
"""
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import plevmap  # noqa: E402

MODS = []
for line in open(os.path.join(plevmap.ROOT, "xtask/tests/modpaths.txt")):
    if line.startswith("#") or not line.strip():
        continue
    a, b = line.split()
    MODS.append((a, b))
IDENT = re.compile(r"[A-Za-z0-9_]")


def translate(text):
    out, i = [], 0
    while i < len(text):
        if i == 0 or not (IDENT.match(text[i - 1]) or text[i - 1] == ":"):
            hit = None
            for old, new in MODS:
                if text.startswith(old, i):
                    hit = (old, new)
                    break
                bare = old[:-2]
                j = i + len(bare)
                if text.startswith(bare, i) and (j == len(text) or not (IDENT.match(text[j]) or text[j] == ":")):
                    hit = (bare, new[:-2])
                    break
            if hit:
                out.append(hit[1])
                i += len(hit[0])
                continue
        out.append(text[i])
        i += 1
    return "".join(out)


def listing(path):
    if not os.path.exists(path):
        return None
    return {l.rstrip("\n") for l in open(path) if l.strip()}


def main(argv):
    before_dir, after_dir = argv
    want = {}
    for old_lib, new_lib in plevmap.CRATE_RENAMES.items():
        pkg = old_lib.replace("_", "-")
        lines = listing(os.path.join(before_dir, pkg + ".txt")) or listing(os.path.join(before_dir, old_lib + ".txt"))
        if lines is None:
            print(f"no listing before for {old_lib}")
            continue
        want.setdefault(new_lib, set()).update(translate(l) for l in lines)
    status = 0
    for new_lib in sorted(want):
        got = listing(os.path.join(after_dir, new_lib + ".txt"))
        if got is None:
            print(f"{new_lib}: no listing after")
            status = 1
            continue
        gone = sorted(want[new_lib] - got)
        new = sorted(got - want[new_lib])
        print(f"{new_lib}: before {len(want[new_lib])}, after {len(got)}, same {len(want[new_lib] & got)}")
        for l in gone:
            print(f"  - {l}")
        for l in new:
            print(f"  + {l}")
        status |= bool(gone or new)
    return status


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
