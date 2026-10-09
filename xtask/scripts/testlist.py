#!/usr/bin/env python3
"""Compare two `cargo nextest list --message-format json` outputs across
the migration, the "before" side translated to the names of the target
layout through migration.toml.

A test is (package, target, test path). For the before side the target's
root file (src/lib.rs, tests/x.rs, tests/x/main.rs, a [[bin]] or an
example's main.rs) is translated to its new file, which names the new
package and target; the test path's module prefix is translated through
the files it names below that root (`theme::tests_contrast::x` ->
`styling::theme::tests::contrast::x`). The two sets are then compared and
every difference is printed.

    python3 xtask/scripts/testlist.py <before.json> <after.json>
"""
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import plevmap  # noqa: E402

MOVES = {}
for _e in plevmap.load():
    if _e["kind"] in ("move", "merge") and _e["from"] and _e["to"]:
        MOVES[_e["from"]] = _e["to"]
OLD_FILES = set(MOVES) | {e["from"] for e in plevmap.load() if e["from"]}


def rel(p, base):
    return os.path.relpath(p, base)


def root_file(suite, base):
    """The repository path of the target's root file, from its binary-id and cwd."""
    crate_dir = rel(suite["cwd"], base)
    kind = suite["kind"]
    bid = suite["binary-id"]
    name = bid.split("::", 1)[1] if "::" in bid else None
    if kind in ("lib", "proc-macro", "cdylib", "staticlib", "rlib"):
        return crate_dir, os.path.join(crate_dir, "src/lib.rs")
    if kind == "test":
        for cand in (f"tests/{name}.rs", f"tests/{name}/main.rs"):
            if os.path.join(crate_dir, cand) in OLD_FILES or os.path.exists(os.path.join(base, crate_dir, cand)):
                return crate_dir, os.path.join(crate_dir, cand)
    if kind == "bin":
        bname = name[4:] if name and name.startswith("bin/") else (name or suite["package-name"])
        declared = old_bins(crate_dir).get(bname)
        if declared:
            return crate_dir, os.path.join(crate_dir, declared)
        for cand in ("src/main.rs", f"src/bin/{bname}.rs", f"src/bin/{bname}/main.rs",
                     f"src/bin/{bname.replace('-', '_')}.rs", "src/main_web.rs"):
            p = os.path.join(crate_dir, cand)
            if p in OLD_FILES or os.path.exists(os.path.join(base, p)):
                if cand == "src/main.rs" and bname not in (suite["package-name"],) and \
                        os.path.exists(os.path.join(base, crate_dir, f"src/bin/{bname}.rs")):
                    continue
                return crate_dir, p
    if kind == "example":
        ename = name[8:] if name and name.startswith("example/") else name
        for cand in (f"examples/{ename}/main.rs", f"examples/{ename}.rs"):
            p = os.path.join(crate_dir, cand)
            if p in OLD_FILES:
                return crate_dir, p
    return crate_dir, None


_BINS = {}


def old_bins(crate_dir):
    """[[bin]] name -> path in the old manifest of `crate_dir`."""
    if crate_dir not in _BINS:
        import subprocess
        import tomllib
        r = subprocess.run(["git", "show", f"{os.environ.get('MIGRATION_BASE', 'HEAD')}:{crate_dir}/Cargo.toml"],
                           cwd=plevmap.ROOT, capture_output=True, text=True)
        bins = {}
        if r.returncode == 0:
            for b in tomllib.loads(r.stdout).get("bin", []):
                if "path" in b:
                    bins[b["name"]] = b["path"]
        _BINS[crate_dir] = bins
    return _BINS[crate_dir]


def module_file(root, segs):
    """The old file of module `segs` under the target root file, or None."""
    d = os.path.dirname(root)
    if os.path.basename(root) not in ("lib.rs", "main.rs", "mod.rs"):
        d = os.path.join(d, os.path.basename(root)[:-3])
    path = os.path.join(d, *segs)
    cands = [path + ".rs", os.path.join(path, "mod.rs")]
    if len(segs) >= 2 and segs[-1].startswith(segs[-2] + "_"):
        # `#[path = "tests_x.rs"] mod tests_x;` inside `tests.rs`: the file
        # sits beside its parent's
        cands.append(os.path.join(d, *segs[:-2], segs[-1]) + ".rs")
    for cand in cands:
        if cand in OLD_FILES:
            return cand
    return None


def new_module(new_root, new_file):
    """Module segments of `new_file` below the target root `new_root`."""
    d = os.path.dirname(new_root)
    r = os.path.relpath(new_file, d)[:-3].split("/")
    if r[-1] in ("mod", "lib", "main"):
        r = r[:-1]
    return r


def translate(name, root, new_root):
    """(new root, test path): a module moved to another target's root file
    (a test that went to the app it measures) names that target."""
    parts = name.split("::")
    for n in range(len(parts) - 1, 0, -1):
        f = module_file(root, parts[:n])
        if f and f in plevmap.BIN_MODULES:
            # a binary's module, declared by `#[path]` under its old name
            return new_root, name
        if f and f in MOVES:
            nf = MOVES[f]
            if os.path.relpath(nf, os.path.dirname(new_root)).startswith(".."):
                return nf, "::".join(parts[n:])
            segs = new_module(new_root, nf)
            return new_root, "::".join(segs + parts[n:])
    return new_root, name


def package_of(new_root):
    nc = plevmap.crate_of(new_root, plevmap.NEW_CRATES)
    return plevmap.NEW_CRATES.get(nc, "?")


def target_of(new_root, nc_dir):
    r = os.path.relpath(new_root, nc_dir)
    if r == "src/lib.rs" or (r.startswith("src/") and plevmap.lib_module(new_root, nc_dir)):
        # a crate merged into this one: its lib is a module of the host's
        return "lib"
    if r.startswith("tests/"):
        return "test:" + r[6:].split("/")[0].removesuffix(".rs")
    if r == "src/main.rs":
        return "bin:" + os.path.basename(nc_dir)
    if r.startswith("src/bin/"):
        return "bin:" + r[8:].split("/")[0].removesuffix(".rs")
    if r.startswith("examples/"):
        return "example:" + r[9:].split("/")[0].removesuffix(".rs")
    if "/" not in r and r.endswith(".rs"):
        # a [[test]] at the crate's root (tool/tests)
        return "test:" + r.removesuffix(".rs")
    return "?" + r


def before_set(path):
    data = json.load(open(path))
    base = plevmap.ROOT
    out = {}
    for suite in data["rust-suites"].values():
        crate_dir, root = root_file(suite, base)
        if not root:
            print("unresolved target:", suite["binary-id"], file=sys.stderr)
            continue
        new_root = MOVES.get(root, root)
        nc = plevmap.crate_of(new_root, plevmap.NEW_CRATES)
        pkg = plevmap.NEW_CRATES.get(nc, "?")
        tgt = target_of(new_root, nc) if nc else "?"
        prefix = plevmap.lib_module(new_root, nc) if nc and tgt == "lib" else None
        for t in suite["testcases"]:
            r, name = translate(t, root, new_root)
            if r != new_root:
                rc = plevmap.crate_of(r, plevmap.NEW_CRATES)
                key = (plevmap.NEW_CRATES.get(rc, "?"), target_of(r, rc) if rc else "?", name)
            else:
                if prefix and not name.startswith("::".join(prefix) + "::"):
                    name = "::".join(prefix + [name])
                key = (pkg, tgt, name)
            out[key] = suite["binary-id"] + " " + t
    return out


def after_set(path):
    data = json.load(open(path))
    base = plevmap.ROOT
    out = {}
    for suite in data["rust-suites"].values():
        crate_dir = rel(suite["cwd"], base)
        kind = suite["kind"]
        bid = suite["binary-id"]
        name = bid.split("::", 1)[1] if "::" in bid else None
        if kind in ("lib", "proc-macro", "cdylib", "staticlib", "rlib"):
            tgt = "lib"
        elif kind == "test":
            tgt = "test:" + name
        elif kind == "bin":
            b = name[4:] if name and name.startswith("bin/") else (name or suite["package-name"])
            if os.path.exists(os.path.join(base, crate_dir, "src/main.rs")) and \
                    not os.path.exists(os.path.join(base, crate_dir, f"src/bin/{b}.rs")) and \
                    not os.path.exists(os.path.join(base, crate_dir, f"src/bin/{b}/main.rs")):
                b = os.path.basename(crate_dir)
            tgt = "bin:" + b
        elif kind == "example":
            tgt = "example:" + (name[8:] if name and name.startswith("example/") else name)
        else:
            tgt = kind
        pkg = os.path.basename(crate_dir)
        for t in suite["testcases"]:
            out[(pkg, tgt, t)] = bid + " " + t
    return out


def norm(d):
    return {(p, t.replace("-", "_"), n): v for (p, t, n), v in d.items()}


def main(argv):
    before = norm(before_set(argv[0]))
    after = norm(after_set(argv[1]))
    only_b = sorted(set(before) - set(after))
    only_a = sorted(set(after) - set(before))
    print(f"before: {len(before)} tests, after: {len(after)} tests, "
          f"same: {len(set(before) & set(after))}")
    for k in only_b:
        print("only before:", "|".join(k), "  <-", before[k])
    for k in only_a:
        print("only after: ", "|".join(k), "  <-", after[k])
    return 1 if (only_a or only_b) else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
