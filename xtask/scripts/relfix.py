#!/usr/bin/env python3
"""Rewrite relative module paths whose root moved or was renamed.

pathfix.py translates paths rooted at a crate, `crate::` or `super::`. A
path can also start at a name the file itself binds: a child module it
declares (`builder::Element` in lib.rs) or a module it imports
(`use crate::ops; ... ops::apply`). When that child moved elsewhere, or
the import now binds another name (`operations`), the path breaks. For
each such root of the file as it was, this finds where the module went
and writes the path again from the file's new place: a child path when it
is still below, `crate::...` when it is elsewhere in the crate, the
crate's name when it left for another one.

    MIGRATION_BASE=<rev before the moves> python3 xtask/scripts/relfix.py [--check] [files...]
"""
import os
import re
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import plevmap  # noqa: E402
import pathfix  # noqa: E402

ROOT = plevmap.ROOT
BASE = os.environ.get("MIGRATION_BASE", "HEAD")
TABLE = plevmap.module_table()
REV = plevmap.reverse_moves()
DECL = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+([a-z_][a-z0-9_]*)\s*;", re.M)
USE = re.compile(r"\buse\s+([^;]+);", re.S)
CHILDREN = set()


def git_show(path):
    r = subprocess.run(["git", "show", f"{BASE}:{path}"], cwd=ROOT, capture_output=True, text=True)
    return r.stdout if r.returncode == 0 else None


def strip(src):
    """The source without comments and string literals (same length)."""
    out = []
    for kind, text, _ in pathfix.tokenize(src):
        out.append(re.sub(r"[^\n]", " ", text) if kind == "skip" else text)
    return "".join(out)


def bindings(src):
    """name -> path segments, for every `use` binding of a module-ish name."""
    out = {}
    for m in USE.finditer(strip(src)):
        tree = re.sub(r"\s+", "", m.group(1))
        expand(tree, [], out)
    return out


def expand(tree, prefix, out):
    # split a use-tree: a::b::{c, d::e, self} / a::b as c / a::b
    i = tree.find("{")
    if i >= 0 and tree.endswith("}"):
        head = tree[:i].rstrip(":")
        segs = prefix + ([s for s in head.split("::") if s] if head else [])
        depth, cur, parts = 0, "", []
        for ch in tree[i + 1:-1]:
            if ch == "," and depth == 0:
                parts.append(cur)
                cur = ""
                continue
            depth += ch == "{"
            depth -= ch == "}"
            cur += ch
        parts.append(cur)
        for p in parts:
            if p == "self":
                if segs:
                    out[segs[-1]] = segs
            elif p:
                expand(p, segs, out)
        return
    name = None
    if " as " in tree or "as" in tree.split("::")[-1:]:
        pass
    m = re.match(r"^(.*?)as([A-Za-z_][A-Za-z0-9_]*)$", tree)
    if m and m.group(1).endswith(tuple("abcdefghijklmnopqrstuvwxyz0123456789_")) is False:
        pass
    parts = tree.split("::")
    last = parts[-1]
    am = re.match(r"^([A-Za-z_][A-Za-z0-9_]*)as([A-Za-z_][A-Za-z0-9_]*)$", last)
    if am and am.group(1) and am.group(2):
        parts[-1] = am.group(1)
        name = am.group(2)
    if parts[-1] == "*":
        return
    segs = prefix + parts
    if name:
        return  # an alias keeps its own name whatever the path becomes
    name = segs[-1]
    if name[:1].islower():
        out[name] = segs


def absolute(segs, lib, mod):
    """A use path of a file of module `mod` in crate `lib` -> (lib, segments)."""
    if not segs:
        return None
    root = segs[0]
    if root == "crate":
        return lib, segs[1:]
    if root == "self":
        return lib, mod + segs[1:]
    if root == "super":
        k = 0
        while k < len(segs) and segs[k] == "super":
            k += 1
        if k > len(mod):
            return None
        return lib, mod[: len(mod) - k] + segs[k:]
    if root in plevmap.OLD_CRATES.values():
        return root, segs[1:]
    if root in CHILDREN:
        # a path relative to the module: one of its children
        return lib, mod + segs
    return None  # std, core and the other external crates


def express(new_lib, target, file_lib, file_mod, lib_file):
    if new_lib != file_lib or not lib_file:
        return [new_lib] + target
    if target[: len(file_mod)] == file_mod and len(target) > len(file_mod):
        return target[len(file_mod):]
    return ["crate"] + target


def fix(path, check):
    old = REV.get(path)
    if not old:
        return False
    oc = plevmap.crate_of(old, plevmap.OLD_CRATES)
    nc = plevmap.crate_of(path, plevmap.NEW_CRATES)
    if not oc or not nc:
        return False
    old_mod = plevmap.lib_module(old, oc)
    new_mod = plevmap.lib_module(path, nc)
    if old_mod is None or new_mod is None:
        return False
    old_src = git_show(old)
    if old_src is None:
        return False
    old_lib, new_lib = plevmap.OLD_CRATES[oc], plevmap.NEW_CRATES[nc]
    src = open(os.path.join(ROOT, path), encoding="utf-8").read()
    # what the file binds now: it keeps a root that still resolves
    now = set(DECL.findall(strip(src))) | set(bindings(src))
    roots = {}
    CHILDREN.clear()
    CHILDREN.update(DECL.findall(strip(old_src)))
    for name in DECL.findall(strip(old_src)):
        roots[name] = (old_lib, old_mod + [name])
    for name, segs in bindings(old_src).items():
        a = absolute(segs, old_lib, old_mod)
        if a:
            roots[name] = a
    renames = {}
    for name, (lib, segs) in roots.items():
        nl, nt = plevmap.translate(TABLE, lib, segs)
        if name in now:
            # still bound: only a binding whose name changed is rewritten
            newname = nt[-1] if nt else name
            if newname != name and newname in now and name not in DECL.findall(strip(src)):
                renames[name] = [newname]
            continue
        new_name_bound = nt and nt[-1] in now and nt[-1] != name
        if new_name_bound:
            renames[name] = [nt[-1]]
        else:
            renames[name] = express(nl, nt, new_lib, new_mod, True)
    if not renames:
        return False
    out = src
    toks = pathfix.tokenize(src)
    edits = []
    code = [t for t in toks if t[0] not in ("ws",)]
    for i, (k, text, pos) in enumerate(code):
        if k != "ident" or text not in renames:
            continue
        prev = code[i - 1] if i else None
        nxt = code[i + 1] if i + 1 < len(code) else None
        if not nxt or nxt[0] != "sep":
            continue
        if prev and (prev[0] == "sep" or prev[1] in (".", "use") and False):
            continue
        if prev and prev[0] == "sep":
            continue
        # not inside a `use` statement's own binding position: `use a::ops;`
        new = "::".join(renames[text])
        if new != text:
            edits.append((pos, pos + len(text), new))
    if not edits:
        return False
    parts, last = [], 0
    for s, e, t in sorted(edits):
        parts.append(out[last:s])
        parts.append(t)
        last = e
    parts.append(out[last:])
    out = "".join(parts)
    print("changed", path, " ".join(f"{a}->{'::'.join(b)}" for a, b in renames.items()))
    if not check:
        open(os.path.join(ROOT, path), "w", encoding="utf-8").write(out)
    return True


def main(argv):
    check = "--check" in argv
    files = [a for a in argv if not a.startswith("--")] or plevmap.rs_files(ROOT)
    n = sum(fix(f, check) for f in files)
    print(f"{n} files changed")


if __name__ == "__main__":
    main(sys.argv[1:])
