#!/usr/bin/env python3
"""Compare `cargo metadata` before and after the move, through the map.

The external packages (name, version, source) must be the same set. The
workspace crates are grouped in families: an old crate and a new one are
in the same family when a file moved from one to the other (a rename is a
family of two; comps and engine one; showcase with uigallery, the benches
and the demos it held another). Per family, the dependencies on external
packages (name, requirement, features) and on other families must be the
same; a dependency that only changed kind (dev -> normal when examples
became crates) is printed as such.

    python3 xtask/scripts/metacmp.py <before metadata.json> <after metadata.json>
"""
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import plevmap  # noqa: E402

OLD_PKG = {}  # old crate dir -> package name
NEW_PKG = {}


def load(path, root_dirs):
    d = json.load(open(path))
    ws = {}
    ext = set()
    for p in d["packages"]:
        if p["source"]:
            ext.add((p["name"], p["version"], p["source"]))
            continue
        ws[p["name"]] = p
    return ws, ext


def families():
    parent = {}

    def find(x):
        while parent.setdefault(x, x) != x:
            parent[x] = parent[parent[x]]
            x = parent[x]
        return x
    for e in plevmap.load():
        if e["kind"] not in ("move", "merge") or not e["from"] or not e["to"]:
            continue
        oc = plevmap.crate_of(e["from"], plevmap.OLD_CRATES)
        nc = plevmap.crate_of(e["to"], plevmap.NEW_CRATES)
        if oc and nc:
            parent[find("old:" + oc)] = find("new:" + nc)
    groups = {}
    for x in list(parent):
        groups.setdefault(find(x), set()).add(x)
    return groups


def main(argv):
    bws, bext = load(argv[0], None)
    aws, aext = load(argv[1], None)
    print(f"external packages: before {len(bext)}, after {len(aext)}, same {len(bext & aext)}")
    status = 0
    for x in sorted(bext ^ aext):
        print("  ", "-" if x in bext else "+", x[:2])
        status = 1
    dir_of = lambda p: os.path.relpath(os.path.dirname(p["manifest_path"]), plevmap.ROOT)
    bdir = {dir_of(p): n for n, p in bws.items()}
    adir = {dir_of(p): n for n, p in aws.items()}
    fam = families()
    fam_of = {}
    for k, members in fam.items():
        label = "+".join(sorted(m.split(":", 1)[1].split("/")[-1] for m in members if m.startswith("old:")))
        for m in members:
            side, d = m.split(":", 1)
            names = bdir if side == "old" else adir
            if d in names:
                fam_of[(side, names[d])] = label
    for n in aws:  # crates with no old file (tool/tests): their own family
        fam_of.setdefault(("new", n), "new:" + n)

    def deps(ws, side):
        out = {}
        for n, p in ws.items():
            f = fam_of.get((side, n), side + ":" + n)
            s = out.setdefault(f, {})
            for dep in p["dependencies"]:
                if dep["source"] is None and dep.get("path"):
                    target = fam_of.get((side, dep["name"]), dep["name"])
                    if target == f:
                        continue
                    key = ("family", target, "", "")
                else:
                    key = ("ext", dep["name"], dep["req"], ",".join(sorted(dep["features"])))
                s.setdefault(key, set()).add(dep["kind"] or "normal")
        return out
    bd, ad = deps(bws, "old"), deps(aws, "new")
    for f in sorted(set(bd) | set(ad)):
        b, a = bd.get(f, {}), ad.get(f, {})
        gone = sorted(set(b) - set(a))
        new = sorted(set(a) - set(b))
        kind = sorted(k for k in set(a) & set(b) if a[k] != b[k])
        print(f"{f}: {len(b)} deps before, {len(a)} after"
              + ("" if gone or new or kind else ", the same"))
        for k in gone:
            print(f"   - {k[0]} {k[1]} {k[2]} {k[3]} ({','.join(sorted(b[k]))})")
            status = 1
        for k in new:
            print(f"   + {k[0]} {k[1]} {k[2]} {k[3]} ({','.join(sorted(a[k]))})")
            status = 1
        for k in kind:
            print(f"   ~ {k[0]} {k[1]}: {','.join(sorted(b[k]))} -> {','.join(sorted(a[k]))}")
    return status


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
