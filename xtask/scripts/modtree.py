#!/usr/bin/env python3
"""Reconcile `mod` declarations with the files after the moves.

For every module file of the target layout (lib.rs, main.rs, mod.rs, and
`x.rs` files that own an `x/` directory), the children its directory holds
must be declared and nothing else. A child that is missing is declared
with the declaration it had in its old parent (visibility, cfg and other
attributes carried over, under its new name); a declaration whose file is
gone is removed with its attributes; a `#[path]` attribute is dropped when
the file now sits where the module system looks for it. A child renamed in
place (`types` -> `events`) is renamed in its parent's paths too.

    python3 xtask/scripts/modtree.py [--check] [crate dirs...]
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
    if _e["kind"] in ("move", "merge") and _e["from"]:
        MOVES[_e["from"]] = _e["to"]

DECL = re.compile(
    r"(?P<attrs>(?:[ \t]*#\[[^\]]*(?:\[[^\]]*\][^\]]*)*\][ \t]*\n)*)"
    r"(?P<indent>[ \t]*)(?P<vis>pub(?:\([^)]*\))?\s+)?mod\s+(?P<name>[a-z_][a-z0-9_]*)\s*;[ \t]*\n",
    re.M,
)


def module_dir(path):
    """Directory whose files are the children of module file `path`."""
    d, b = os.path.split(path)
    if b in ("lib.rs", "main.rs", "mod.rs"):
        return d
    return os.path.join(d, b[:-3])


def children(full_dir, is_root, own=None):
    out = set()
    if not os.path.isdir(full_dir):
        return out
    for f in os.listdir(full_dir):
        p = os.path.join(full_dir, f)
        if f.endswith(".rs") and f != "mod.rs" and f != own:
            if is_root and f in ("lib.rs", "main.rs", "main_web.rs"):
                continue
            out.add(f[:-3])
        elif os.path.isdir(p) and os.path.isfile(os.path.join(p, "mod.rs")):
            if is_root and f == "bin":
                continue
            out.add(f)
    return out


def old_decl(new_child_path):
    """The declaration of this child in its old parent, with its old name."""
    old = REV.get(new_child_path)
    if not old:
        return None, None
    d, b = os.path.split(old)
    name = os.path.basename(d) if b == "mod.rs" else b[:-3]
    parent_dir = os.path.dirname(d) if b == "mod.rs" else d
    for cand in (os.path.join(parent_dir, "mod.rs"), parent_dir + ".rs",
                 os.path.join(parent_dir, "lib.rs"), os.path.join(parent_dir, "main.rs")):
        try:
            src = old_source(cand)
        except KeyError:
            continue
        for m in DECL.finditer(src):
            if m.group("name") == name:
                return m, name
        # #[path = "tests_x.rs"] mod x;
        for m in DECL.finditer(src):
            if f'"{b}"' in m.group("attrs"):
                return m, m.group("name")
    return None, name


_OLD = {}


def old_source(path):
    if path not in _OLD:
        import subprocess
        r = subprocess.run(["git", "show", f"{BASE}:{path}"], cwd=ROOT,
                           capture_output=True, text=True)
        if r.returncode:
            raise KeyError(path)
        _OLD[path] = r.stdout
    return _OLD[path]


BASE = os.environ.get("MIGRATION_BASE", "HEAD")


def child_path(mod_dir, name):
    a = os.path.join(mod_dir, name + ".rs")
    b = os.path.join(mod_dir, name, "mod.rs")
    return a if os.path.isfile(os.path.join(ROOT, a)) else b


def fix_module(path, check):
    full = os.path.join(ROOT, path)
    src = open(full, encoding="utf-8").read()
    mdir = module_dir(path)
    is_root = os.path.basename(path) in ("lib.rs", "main.rs") and mdir.endswith("/src")
    own = os.path.basename(path) if os.path.basename(path) in ("lib.rs", "main.rs") else None
    want = children(os.path.join(ROOT, mdir), is_root, own)
    if is_root and os.path.basename(path) == "main.rs" and os.path.isfile(os.path.join(ROOT, mdir, "lib.rs")):
        return False  # the binary uses the library; the library declares
    have = {m.group("name"): m for m in DECL.finditer(src)}
    out = src
    renames = {}
    # drop declarations of files that are gone, and stale #[path] attributes
    for name, m in sorted(have.items(), key=lambda kv: -kv[1].start()):
        attrs = m.group("attrs")
        if name not in want:
            out = out[:m.start()] + out[m.end():]
            continue
        if "#[path" in attrs:
            new_attrs = re.sub(r"[ \t]*#\[path\s*=\s*\"[^\"]*\"\][ \t]*\n", "", attrs)
            out = out[:m.start()] + new_attrs + out[m.start() + len(attrs):]
    # declare what is missing, in the shape it had before
    added = []
    for name in sorted(want - set(have)):
        cpath = child_path(mdir, name)
        m, old_name = old_decl(cpath)
        if m is not None:
            attrs = re.sub(r"[ \t]*#\[path\s*=\s*\"[^\"]*\"\][ \t]*\n", "", m.group("attrs"))
            vis = m.group("vis") or ""
            if old_name and old_name != name:
                renames[old_name] = name
        else:
            attrs, vis = "", "pub "
            if name == "tests":
                attrs, vis = "#[cfg(test)]\n", ""
        added.append(f"{attrs}{vis}mod {name};\n")
    if added:
        # after the last remaining declaration, or after the inner doc/attrs
        decls = list(DECL.finditer(out))
        if decls:
            pos = decls[-1].end()
        else:
            pos = 0
            for m in re.finditer(r"^(//!.*|#!\[.*\])\n", out, re.M):
                if m.start() == pos:
                    pos = m.end()
            if pos:
                out = out[:pos] + "\n" + out[pos:]
                pos += 1
        out = out[:pos] + "".join(added) + out[pos:]
    for old, new in renames.items():
        out = re.sub(rf"(?<![A-Za-z0-9_:]){old}::", f"{new}::", out)
        out = re.sub(rf"(\buse\s+(?:self::)?){old}\b", rf"\g<1>{new}", out)
    if out != src:
        print("changed", path, f"(+{len(added)})" if added else "",
              " ".join(f"{a}->{b}" for a, b in renames.items()))
        if not check:
            open(full, "w", encoding="utf-8").write(out)
        return True
    return False


def module_files(crate_dir):
    out = []
    for sub in ("src", "tests", "examples", "benches"):
        out += _module_files(os.path.join(ROOT, crate_dir, sub), sub == "src")
    return sorted(set(out))


def _module_files(src, is_src):
    out = []
    for dirpath, _, files in os.walk(src):
        for f in files:
            p = os.path.relpath(os.path.join(dirpath, f), ROOT)
            if f in ("lib.rs", "main.rs", "mod.rs"):
                if f == "mod.rs" or is_src or os.path.dirname(dirpath) != os.path.dirname(src) or True:
                    out.append(p)
            elif f.endswith(".rs") and os.path.isdir(os.path.join(dirpath, f[:-3])):
                out.append(p)
    return sorted(out)


def main(argv):
    check = "--check" in argv
    crates = [a.rstrip("/") for a in argv if not a.startswith("--")] or list(plevmap.NEW_CRATES)
    n = 0
    for c in crates:
        for p in module_files(c):
            n += fix_module(p, check)
    print(f"{n} module files changed")


if __name__ == "__main__":
    main(sys.argv[1:])
