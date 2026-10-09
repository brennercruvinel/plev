#!/usr/bin/env python3
"""Rewrite Rust paths after the moves: crate renames and module moves.

For every .rs file of the target layout, each path rooted at a crate name
of today (`sem::`, `comps::`, ...) or at `crate::`/`$crate::` is translated
through migration.toml: the crate root by the package rename, the module
part by the longest module prefix that moved. Use-trees are rewritten in
place, group by group. Strings and comments are left alone; a root that
the file shadows (`mod git;`, `use x::git;`) is left alone; `self::` and
`super::` are left to the compiler.

    python3 xtask/scripts/pathfix.py [--check] [files...]

prints one line per file changed and, for paths it cannot express inside a
use-group, a `manual:` line to fix by hand.
"""
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import plevmap  # noqa: E402

TABLE = plevmap.module_table()
REV = plevmap.reverse_moves()
OLD_LIBS = set(plevmap.OLD_CRATES.values())
MANUAL = []


def tokenize(src):
    """(kind, text, start) with kind in ident, path_sep, brace, comma, skip, other."""
    toks = []
    i, n = 0, len(src)
    while i < n:
        c = src[i]
        if src.startswith("//", i):
            j = src.find("\n", i)
            j = n if j < 0 else j
            toks.append(("skip", src[i:j], i))
            i = j
        elif src.startswith("/*", i):
            depth, j = 1, i + 2
            while j < n and depth:
                if src.startswith("/*", j):
                    depth, j = depth + 1, j + 2
                elif src.startswith("*/", j):
                    depth, j = depth - 1, j + 2
                else:
                    j += 1
            toks.append(("skip", src[i:j], i))
            i = j
        elif c == '"' or (c in "rb" and re.match(r'(b?r#*"|b")', src[i:i + 4])):
            m = re.match(r'b?r(#*)"', src[i:])
            if m:
                end = src.find('"' + m.group(1), i + len(m.group(0)))
                j = n if end < 0 else end + 1 + len(m.group(1))
            else:
                j = i + (2 if c == "b" else 1)
                while j < n and src[j] != '"':
                    j += 2 if src[j] == "\\" else 1
                j += 1
            toks.append(("skip", src[i:j], i))
            i = j
        elif c == "'":
            m = re.match(r"'(\\.[^']*|[^\\'])'", src[i:])
            j = i + (len(m.group(0)) if m else 1)
            toks.append(("skip" if m else "other", src[i:j], i))
            i = j
        elif c == "$" and src.startswith("$crate", i):
            toks.append(("ident", "$crate", i))
            i += 6
        elif c.isalpha() or c == "_":
            m = plevmap.IDENT.match(src, i)
            toks.append(("ident", m.group(0), i))
            i = m.end()
        elif src.startswith("::", i):
            toks.append(("sep", "::", i))
            i += 2
        elif c in "{}":
            toks.append(("brace", c, i))
            i += 1
        elif c == ",":
            toks.append(("comma", c, i))
            i += 1
        elif c.isspace():
            j = i
            while j < n and src[j].isspace():
                j += 1
            toks.append(("ws", src[i:j], i))
            i = j
        else:
            toks.append(("other", c, i))
            i += 1
    return toks


class File:
    def __init__(self, path, src):
        self.path = path
        self.src = src
        self.toks = [t for t in tokenize(src) if t[0] not in ("ws", "skip")]
        self.edits = []  # (start, end, text)
        old = REV.get(path)
        oc = plevmap.crate_of(old, plevmap.OLD_CRATES) if old else None
        nc = plevmap.crate_of(path, plevmap.NEW_CRATES)
        self.new_lib = plevmap.NEW_CRATES.get(nc)
        self.old_lib = plevmap.OLD_CRATES.get(oc) if oc else None
        # `crate::` means the old crate only for files of a library tree
        self.lib_file = bool(oc and plevmap.lib_module(old, oc) is not None)
        self.shadow = set()
        for m in re.finditer(r"\bmod\s+([a-z_][a-z0-9_]*)", src):
            self.shadow.add(m.group(1))
        for m in re.finditer(r"\bas\s+([a-z_][a-z0-9_]*)\s*[;,}]", src):
            self.shadow.add(m.group(1))
        for m in re.finditer(r"::([a-z_][a-z0-9_]*)\s*;", src):
            # `use a::b::git;` brings a local `git` into scope
            if re.search(r"\buse\b[^;]*::" + m.group(1) + r"\s*;", src):
                self.shadow.add(m.group(1))

    def root_lib(self, root):
        if root in ("crate", "$crate"):
            return self.old_lib if self.lib_file else None
        if root in OLD_LIBS and root not in self.shadow:
            return root
        return None

    def emit_root(self, new_lib, root_tok):
        if root_tok == "$crate":
            return "$crate" if new_lib == self.new_lib else new_lib
        return "crate" if new_lib == self.new_lib else new_lib

    def translate(self, root, segs):
        lib = self.root_lib(root)
        if lib is None:
            return None
        new_lib, new_segs = plevmap.translate(TABLE, lib, segs)
        return [self.emit_root(new_lib, root)] + new_segs

    def run(self):
        t = self.toks
        i = 0
        while i < len(t):
            k, text, _ = t[i]
            prev = t[i - 1] if i else None
            if (k == "ident" and i + 1 < len(t) and t[i + 1][0] == "sep"
                    and not (prev and (prev[0] == "sep" or prev[1] == "."))):
                i = self.path_at(i)
            else:
                i += 1
        return self.apply()

    def read_path(self, i):
        """Segments starting at token i: (segs, idx after, group_start or None)."""
        segs = [i]
        j = i + 1
        while j + 1 < len(self.toks) and self.toks[j][0] == "sep":
            nk, nt, _ = self.toks[j + 1]
            if nk == "ident":
                segs.append(j + 1)
                j += 2
            elif nk == "brace" and nt == "{":
                return segs, j + 1, j + 1
            else:
                break
        return segs, j, None

    def path_at(self, i):
        segs, j, group = self.read_path(i)
        names = [self.toks[s][1] for s in segs]
        new = self.translate(names[0], names[1:])
        if new is not None and new != names:
            self.replace_segs(segs, new)
        if group is not None and self.root_lib(names[0]) is not None:
            return self.group(group, names, new or names)
        return j

    def replace_segs(self, segs, new):
        start = self.toks[segs[0]][2]
        last = self.toks[segs[-1]]
        self.edits.append((start, last[2] + len(last[1]), "::".join(new)))

    def group(self, g, old_prefix, new_prefix):
        """Rewrite a use-group `{ ... }` opened at token g; returns index after `}`."""
        j = g + 1
        while j < len(self.toks):
            k, text, _ = self.toks[j]
            if k == "brace" and text == "}":
                return j + 1
            if k == "comma":
                j += 1
                continue
            if k == "brace" and text == "{":
                j = self.group(j, old_prefix, new_prefix)
                continue
            if k != "ident" or text in ("self", "super"):
                j += 1
                continue
            segs, after, sub = self.read_path(j)
            names = [self.toks[s][1] for s in segs]
            full_old = old_prefix + names
            full_new = self.translate(full_old[0], full_old[1:])
            if full_new is None:
                full_new = new_prefix + names
            if full_new[:len(new_prefix)] == new_prefix:
                rel = full_new[len(new_prefix):]
                if rel != names and rel:
                    self.replace_segs(segs, rel)
                elif not rel:
                    MANUAL.append(f"{self.path}: {'::'.join(full_old)} -> "
                                  f"{'::'.join(full_new)} (empty relative path)")
            else:
                MANUAL.append(f"{self.path}: {'::'.join(full_old)} -> "
                              f"{'::'.join(full_new)} leaves its use-group")
            if sub is not None:
                j = self.group(sub, full_old, full_new)
            else:
                j = after
        return j

    def apply(self):
        if not self.edits:
            return self.src
        out, last = [], 0
        for s, e, text in sorted(set(self.edits)):
            if s < last:
                continue
            out.append(self.src[last:s])
            out.append(text)
            last = e
        out.append(self.src[last:])
        return "".join(out)


def main(argv):
    check = "--check" in argv
    files = [a for a in argv if not a.startswith("--")] or plevmap.rs_files(plevmap.ROOT)
    changed = 0
    for rel in files:
        full = os.path.join(plevmap.ROOT, rel)
        src = open(full, encoding="utf-8").read()
        out = File(rel, src).run()
        if out != src:
            changed += 1
            print("changed", rel)
            if not check:
                open(full, "w", encoding="utf-8").write(out)
    for m in MANUAL:
        print("manual:", m)
    print(f"{changed} files changed, {len(MANUAL)} manual")


if __name__ == "__main__":
    main(sys.argv[1:])
