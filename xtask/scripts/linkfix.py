#!/usr/bin/env python3
"""Rewrite relative links in the docs after the moves.

A markdown link `[text](rel)` (or `<a href="rel">`) in a file that moved
resolved against the file's old directory; the target moved too. Both are
translated through migration.toml and the link is written again relative
to the file's new directory, the #fragment kept. Links that are URLs,
absolute, anchors only, or that never resolved to a tracked path are left
alone; `manual:` lines name what could not be translated.

    MIGRATION_BASE=<rev before the moves> python3 xtask/scripts/linkfix.py [--check] [files...]
"""
import os
import re
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import plevmap  # noqa: E402

ROOT = plevmap.ROOT
BASE = os.environ.get("MIGRATION_BASE", "HEAD")
REV = plevmap.reverse_moves()
MOVES = {}
for _e in plevmap.load():
    if _e["kind"] in ("move", "merge") and _e["from"] and _e["to"]:
        MOVES[_e["from"]] = _e["to"]
OLD_FILES = set(subprocess.run(["git", "ls-tree", "-r", "--name-only", BASE], cwd=ROOT,
                               capture_output=True, text=True).stdout.split())
OLD_DIRS = set()
for _f in OLD_FILES:
    _d = os.path.dirname(_f)
    while _d:
        OLD_DIRS.add(_d)
        _d = os.path.dirname(_d)
LINK = re.compile(r"(\]\(|href=\")([^)\"\s]+)")
MANUAL = []


def new_dir_of(old_dir):
    """Where an old directory went: the common directory of its files' targets."""
    kids = [t for f, t in MOVES.items() if f.startswith(old_dir + "/")]
    if not kids:
        return None
    return os.path.commonpath(kids) if len(kids) > 1 else os.path.dirname(kids[0])


def fix(path, check):
    old = REV.get(path)
    if not old or not path.endswith((".md", ".html", ".toml")):
        return False
    full = os.path.join(ROOT, path)
    src = open(full, encoding="utf-8").read()
    old_dir, new_dir = os.path.dirname(old), os.path.dirname(path)

    def sub(m):
        link = m.group(2)
        if re.match(r"^[a-z]+:", link) or link.startswith(("#", "/", "<")):
            return m.group(0)
        target, frag = (link.split("#", 1) + [""])[:2]
        if not target:
            return m.group(0)
        resolved = os.path.normpath(os.path.join(old_dir, target))
        if resolved in MOVES:
            new_target = MOVES[resolved]
        elif resolved in OLD_DIRS:
            new_target = new_dir_of(resolved)
        elif resolved in OLD_FILES:
            MANUAL.append(f"{path}: {link} (dropped)")
            return m.group(0)
        else:
            return m.group(0)  # never resolved: not ours to judge here
        if new_target is None:
            MANUAL.append(f"{path}: {link}")
            return m.group(0)
        rel = os.path.relpath(new_target, new_dir or ".")
        if target.endswith("/") and not rel.endswith("/"):
            rel += "/"
        new = rel + ("#" + frag if frag else "")
        return m.group(1) + new if new != link else m.group(0)

    out = LINK.sub(sub, src)
    if out != src:
        print("changed", path)
        if not check:
            open(full, "w", encoding="utf-8").write(out)
        return True
    return False


def main(argv):
    check = "--check" in argv
    files = [a for a in argv if not a.startswith("--")]
    if not files:
        files = [t for t in REV if t.endswith((".md", ".html", ".toml"))]
    n = sum(fix(f, check) for f in sorted(files))
    for m in MANUAL:
        print("manual:", m)
    print(n, "files changed,", len(MANUAL), "manual")


if __name__ == "__main__":
    main(sys.argv[1:])
