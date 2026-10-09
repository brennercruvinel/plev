#!/usr/bin/env python3
"""Rewrite repository paths written in text: docs, scripts, configs, comments.

Every path of today that migration.toml moves is replaced by its target,
longest first: whole file paths, then the directories whose files all
landed under one new directory (crates/showcase/fixtures/regress ->
tool/bench/regress/baselines), then the crate roots (crates/showcase ->
apps/uigallery). A match must start at a path boundary (not inside a
longer name). In .rs files only comments are rewritten: string literals
were handled by litfix.py and the ones left are data (a demo table's
file names), not paths of this repository.

    python3 xtask/scripts/textfix.py [--check] [--list] [files...]
"""
import os
import re
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import plevmap  # noqa: E402
import pathfix  # noqa: E402

ROOT = plevmap.ROOT
SKIP_PREFIX = ("boiler/", "xtask/", "migration.toml", "data/", "tool/bench/regress/baselines/")
TEXT_EXT = {".md", ".toml", ".sh", ".py", ".yml", ".yaml", ".html", ".kts", ".kt", ".xml",
            ".json", ".m", ".plist", ".txt", ".rs", ".properties", ""}


def table():
    moves = {}
    for e in plevmap.load():
        if e["kind"] in ("move", "merge") and e["from"] and e["to"] and e["from"] != e["to"]:
            moves[e["from"]] = e["to"]
    pairs = dict(moves)
    # directories: where their files went. all under one new directory with
    # the same suffix -> that directory; otherwise the directory most of them
    # went to, when it holds at least half of them
    dirs = {}
    for f, t in moves.items():
        d = os.path.dirname(f)
        while d:
            rest = f[len(d) + 1:]
            nd = t[: -len(rest) - 1] if t.endswith("/" + rest) else None
            dirs.setdefault(d, []).append((nd, t, rest))
            d = os.path.dirname(d)
    for d, items in dirs.items():
        news = {nd for nd, _, _ in items}
        if len(news) == 1 and None not in news:
            nd = news.pop()
        else:
            # the new directory of each direct child, by count
            counts = {}
            for _, t, rest in items:
                depth = rest.count("/")
                nd_child = os.path.dirname(t)
                for _ in range(depth):
                    nd_child = os.path.dirname(nd_child)
                counts[nd_child] = counts.get(nd_child, 0) + 1
            nd, n = max(counts.items(), key=lambda kv: kv[1])
            if n * 2 < len(items):
                continue
        if nd and nd != d and "/" in d:
            pairs[d + "/"] = nd + "/"
            pairs.setdefault(d, nd)
    # crate roots, even when some files inside were renamed
    for old, new in [
        ("crates/claude-model", "rust/claude"), ("crates/comps", "rust/engine/src/widgets"),
        ("crates/engine", "rust/engine"), ("crates/git", "rust/gitdrv"),
        ("crates/ide", "apps/idestudio"), ("crates/inspect", "rust/replay"),
        ("crates/inspector", "apps/inspector"), ("crates/llm", "rust/llmrun"),
        ("crates/lot", "rust/import/src/formats/lottie"), ("crates/macros", "rust/macros"),
        ("crates/monster", "rust/motion"), ("crates/narrate-macro", "rust/macros/src/markup"),
        ("crates/narrate", "rust/markup"), ("crates/parser", "rust/import"),
        ("crates/prime", "demo/primes"), ("crates/propose", "rust/offers"),
        ("crates/rec", "rust/record"), ("crates/rope", "rust/editor"),
        ("crates/sem", "rust/accord"), ("crates/showcase", "apps/uigallery"),
        ("crates/surface", "rust/screen"), ("crates/svg", "rust/import/src/formats/svg"),
        ("crates/uir", "rust/plevir"), ("crates/urnaui", "apps/urnadbapp"),
        ("crates/wire", "rust/socket"), ("refs/lottie", "data/media/lottie"),
        ("crates/showcase/src/app", "apps/uigallery/src/shell"),
        ("crates/showcase/examples", "apps/uigallery/examples"),
        ("crates/engine/examples", "demo"),
        ("docs/ADR", "docs/adr"), ("docs/mission", "docs/log"), ("docs/refs", "docs/ref"),
        ("docs/how-to", "docs/ref"), ("ios/showcase", "apps/uigallery/ios"),
        ("android", "apps/uigallery/android"), ("web/urnaui", "apps/urnadbapp/web"),
    ]:
        # these say where the directory went, over the count of its files
        pairs[old + "/"] = new + "/"
        if "/" in old:
            # a bare one-word root ("android") is a word too: only with its slash
            pairs[old] = new
    # merged scripts: the former script is now a subcommand of the new one
    for old, new in [
        ("script/capture-web", "tool/tasks/screencap.sh web"),
        ("script/capture-ios", "tool/tasks/screencap.sh ios"),
        ("script/capture-android", "tool/tasks/screencap.sh android"),
        ("script/web-urnaui", "tool/tasks/webtarget.sh urnadbapp"),
        ("android/build_android.sh", "tool/tasks/mobileapp.sh android"),
        ("ios/showcase/build_ios.sh", "tool/tasks/mobileapp.sh ios"),
        ("ios/showcase/run_ios.sh", "tool/tasks/mobileapp.sh ios-run"),
        ("crates/engine/examples/lot2monsters", "rust/import/src/bin/convert.rs"),
        ("crates/engine/examples/svg2monster", "rust/import/src/bin/convert.rs"),
    ]:
        pairs[old] = new
    return sorted(pairs.items(), key=lambda kv: -len(kv[0]))


PAIRS = table()
LOOKUP = dict(PAIRS)
# one regex, alternatives longest first, at a path boundary. a key of one
# segment ("android/") must also not follow a slash: inside a longer path
# (web/android/ios, apps/uigallery/android) it is not the root directory
_multi = [k for k, _ in PAIRS if "/" in k.rstrip("/")]
_single = [k for k, _ in PAIRS if "/" not in k.rstrip("/")]
RX = re.compile(
    # never inside a longer path: not after a name or after `name/` (a
    # relative `../` still leads in)
    r"(?<![A-Za-z0-9_\-.])(?<![A-Za-z0-9_\-]/)(" + "|".join(re.escape(k) for k in _multi) + r")(?![A-Za-z0-9_\-])"
    + (r"|(?<![A-Za-z0-9_\-./])(" + "|".join(re.escape(k) for k in _single) + r")" if _single else "")
)


_OLD = set(subprocess.run(["git", "ls-tree", "-r", "--name-only",
                           os.environ.get("MIGRATION_BASE", "HEAD")], cwd=ROOT,
                          capture_output=True, text=True).stdout.split())
_OLD_DIRS = {os.path.dirname(f) for f in _OLD}


def _repl(m):
    key = m.group(1) or m.group(2)
    if m.group(2):
        # a one-word root only when what follows is a path of the old tree
        rest = re.match(r"[A-Za-z0-9_.@-]+(?:/[A-Za-z0-9_.@-]+)*", m.string[m.end():])
        tail = rest.group(0) if rest else ""
        cand = (key + tail).rstrip("/.")
        if cand not in _OLD and cand not in _OLD_DIRS:
            return key
    return LOOKUP[key]


def sub_text(text):
    return RX.sub(_repl, text)


def fix_rs(src):
    out, last = [], 0
    for kind, text, pos in pathfix.tokenize(src):
        if kind == "skip" and (text.startswith("//") or text.startswith("/*")):
            new = sub_text(text)
            if new != text:
                out.append(src[last:pos])
                out.append(new)
                last = pos + len(text)
    out.append(src[last:])
    return "".join(out)


def files():
    out = subprocess.run(["git", "ls-files", "-co", "--exclude-standard"], cwd=ROOT,
                         capture_output=True, text=True).stdout.split("\n")
    return [f for f in out if f and not f.startswith(SKIP_PREFIX)
            and os.path.splitext(f)[1] in TEXT_EXT and os.path.isfile(os.path.join(ROOT, f))]


def main(argv):
    check = "--check" in argv
    if "--list" in argv:
        for k, v in PAIRS:
            print(f"{k} -> {v}")
        return
    todo = [a for a in argv if not a.startswith("--")] or files()
    n = 0
    for f in todo:
        p = os.path.join(ROOT, f)
        try:
            src = open(p, encoding="utf-8").read()
        except UnicodeDecodeError:
            continue
        out = fix_rs(src) if f.endswith(".rs") else sub_text(src)
        if out != src:
            n += 1
            print("changed", f)
            if not check:
                open(p, "w", encoding="utf-8").write(out)
    print(n, "files changed")


if __name__ == "__main__":
    main(sys.argv[1:])
