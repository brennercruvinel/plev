#!/usr/bin/env python3
"""Widen what a move made unreachable, by the compiler's own report.

When a child module becomes a sibling (resolve_figma/look.rs ->
figma/look.rs) or a file leaves the module that owned a type (the ide's
workbench/generated.rs -> generative/), the private items it used are no
longer in reach. This runs `cargo check` with JSON diagnostics and, for
every E0603 (private item), E0616 (private field) and E0624 (private
method), opens the item's definition one step: private -> pub(super),
pub(super) -> pub(crate). It repeats until the errors stop changing.

    python3 xtask/scripts/viswiden.py -p <package> [-p ...]
"""
import json
import os
import re
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
CODES = {"E0603", "E0616", "E0624"}
ITEM = re.compile(r"^(\s*)((?:pub(?:\([^)]*\))?\s+)?)((?:const\s+|async\s+|unsafe\s+)*(?:fn|struct|enum|const|static|type|trait|mod|union)\b)")
FIELD = re.compile(r"^(\s*)((?:pub(?:\([^)]*\))?\s+)?)([A-Za-z_][A-Za-z0-9_]*\s*:)")


def defs(pkgs):
    cmd = ["cargo", "check", "--all-targets", "--message-format", "json"]
    for p in pkgs:
        cmd += ["-p", p]
    env = dict(os.environ, CARGO_BUILD_JOBS="4")
    out = subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True, env=env).stdout
    found = set()
    for line in out.splitlines():
        try:
            m = json.loads(line)
        except ValueError:
            continue
        if m.get("reason") != "compiler-message":
            continue
        msg = m["message"]
        tm = re.search(r"type `([\w:]+)` is (?:private|more private than)", msg["message"])
        if tm and msg.get("spans"):
            hit = item_def(msg["spans"][0]["file_name"], tm.group(1))
            if hit:
                found.add(hit)
            continue
        if not msg.get("code") or msg["code"]["code"] not in CODES:
            continue
        if msg["code"]["code"] == "E0616":
            fm = re.search(r"field `(\w+)` of struct `([\w:]+)`", msg["message"])
            span = (msg.get("spans") or [{}])[0]
            if fm and span.get("file_name"):
                hit = field_def(span["file_name"], fm.group(2), fm.group(1))
                if hit:
                    found.add(hit)
            continue
        for c in [msg] + msg["children"]:
            for s in c.get("spans", []):
                label = s.get("label") or ""
                if "defined here" in label or "is defined here" in c.get("message", ""):
                    found.add((s["file_name"], s["line_start"]))
    return found


def field_def(use_file, struct_path, field):
    """(file, line) of `field` in the struct `a::b::Name`, searched in the
    using file's crate, in the file whose path ends like the module path."""
    segs = struct_path.split("::")
    struct, mods = segs[-1], segs[:-1]
    d = os.path.dirname(os.path.join(ROOT, use_file))
    while d != ROOT and not os.path.isfile(os.path.join(d, "Cargo.toml")):
        d = os.path.dirname(d)
    cands = []
    for dirpath, _, files in os.walk(os.path.join(d, "src")):
        for f in files:
            if f.endswith(".rs"):
                cands.append(os.path.join(dirpath, f))
    tail = "/".join(mods)

    def rank(p):
        rel = p[:-3]
        if rel.endswith("/mod"):
            rel = rel[:-4]
        return 0 if tail and rel.endswith("/" + tail) else 1

    for p in sorted(cands, key=rank):
        if True:
            lines = open(p, encoding="utf-8").read().split("\n")
            for i, l in enumerate(lines):
                if re.match(rf"\s*(pub(\([^)]*\))?\s+)?struct\s+{struct}\b", l):
                    for j in range(i + 1, min(i + 200, len(lines))):
                        if re.match(r"^\s*\}", lines[j]) and not lines[j].startswith("    "):
                            break
                        if re.match(rf"^\s*(pub(\([^)]*\))?\s+)?{field}\s*:", lines[j]):
                            return (os.path.relpath(p, ROOT), j + 1)
    return None


def item_def(use_file, path):
    """(file, line) of the struct/enum/type named by `a::b::Name`."""
    segs = path.split("::")
    name, mods = segs[-1], segs[:-1]
    d = os.path.dirname(os.path.join(ROOT, use_file))
    while d != ROOT and not os.path.isfile(os.path.join(d, "Cargo.toml")):
        d = os.path.dirname(d)
    cands = []
    for dirpath, _, files in os.walk(os.path.join(d, "src")):
        cands += [os.path.join(dirpath, f) for f in files if f.endswith(".rs")]
    tail = "/".join(mods)

    def rank(p):
        rel = p[:-3]
        rel = rel[:-4] if rel.endswith("/mod") else rel
        return 0 if tail and rel.endswith("/" + tail) else (1 if not tail else 2)

    rx = re.compile(rf"^\s*(pub(\([^)]*\))?\s+)?(struct|enum|type|trait)\s+{name}\b")
    for p in sorted(cands, key=rank):
        for i, l in enumerate(open(p, encoding="utf-8").read().split("\n")):
            if rx.match(l):
                return (os.path.relpath(p, ROOT), i + 1)
    return None


def widen(file, line):
    path = os.path.join(ROOT, file)
    if not os.path.isfile(path) or "/.cargo/" in path or "/rustlib/" in path:
        return False
    lines = open(path, encoding="utf-8").read().split("\n")
    i = line - 1
    # the definition may start on an attribute or doc line: find the item
    for j in range(i, min(i + 6, len(lines))):
        for rx in (ITEM, FIELD):
            m = rx.match(lines[j])
            if m:
                vis = m.group(2).strip()
                new = {"": "pub(super) ", "pub(super)": "pub(crate) "}.get(vis)
                if new is None:
                    return False
                lines[j] = m.group(1) + new + lines[j][m.end(2):]
                open(path, "w", encoding="utf-8").write("\n".join(lines))
                print(f"{file}:{j + 1}: {vis or 'private'} -> {new.strip()}")
                return True
    return False


def main(argv):
    pkgs = [argv[i + 1] for i, a in enumerate(argv) if a == "-p"]
    seen = None
    for _ in range(6):
        found = defs(pkgs)
        if not found or found == seen:
            break
        seen = found
        changed = sum(widen(f, l) for f, l in sorted(found))
        if not changed:
            break


if __name__ == "__main__":
    main(sys.argv[1:])
