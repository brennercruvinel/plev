#!/usr/bin/env python3
"""Find (and with --write rewrite) Rust paths inside doc-comment code blocks.

A ``` block in `///` or `//!` is compiled as a doctest; pathfix rewrote
code and textfix comments, neither the code in a comment. A path there
whose root is an old crate is translated through the module table
(xtask/tests/modpaths.txt, the rule the proofs use); a `use` tree is
expanded one level so `use engine::{theme::Theme, builder::div}` has
each branch translated, then written back in the same shape.

    python3 xtask/scripts/docpaths.py [--write]
"""
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import apidiff  # noqa: E402
import plevmap  # noqa: E402

OLD = set(plevmap.OLD_CRATES.values()) - {"engine", "macros", "inspector"}
DOC = re.compile(r"^(\s*//[/!] ?)(.*)$")
USE_TREE = re.compile(r"\b([a-z_][a-z0-9_]*)::\{([^{}]*)\}")


def fix_line(code):
    def tree(m):
        root, inner = m.group(1), m.group(2)
        if root not in plevmap.CRATE_RENAMES:
            return m.group(0)
        parts = [p.strip() for p in inner.split(",") if p.strip()]
        out = []
        for p in parts:
            t = apidiff.translate(f"{root}::{p}", bare=False)
            out.append(t)
        new_root = None
        branches = []
        for t in out:
            r, _, rest = t.partition("::")
            new_root = new_root or r
            if r != new_root:
                return m.group(0)
            branches.append(rest)
        # a common module prefix stays outside the braces
        return f"{new_root}::{{{', '.join(branches)}}}"
    s = USE_TREE.sub(tree, code)
    return apidiff.translate(s, bare=False)


def main(argv):
    write = "--write" in argv
    total = 0
    for f in plevmap.rs_files(plevmap.ROOT):
        rel = os.path.relpath(f, plevmap.ROOT)
        lines = open(f, encoding="utf-8").read().split("\n")
        fence, changed = False, False
        for i, line in enumerate(lines):
            m = DOC.match(line)
            if not m:
                fence = False
                continue
            body = m.group(2)
            if body.lstrip().startswith("```"):
                fence = not fence
                continue
            if not fence:
                continue
            new = fix_line(body)
            if new != body:
                print(f"{rel}:{i + 1}: {body.strip()}\n    -> {new.strip()}")
                lines[i] = m.group(1) + new
                total += 1
                changed = True
        if write and changed:
            open(f, "w", encoding="utf-8").write("\n".join(lines))
    print(total, "lines")


if __name__ == "__main__":
    main(sys.argv[1:])
