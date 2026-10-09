"""The migration map seen as crates and modules.

Shared by the rewriting scripts: which crate a path belongs to, before and
after, and how a module path of today translates into the target layout.
Everything is derived from migration.toml; nothing here is hand-kept.
"""
import os
import re
import tomllib

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

# old crate dir -> lib name of today (package name with - -> _)
OLD_CRATES = {
    "crates/claude-model": "claude_model", "crates/comps": "comps", "crates/engine": "engine",
    "crates/git": "git", "crates/ide": "ide", "crates/inspect": "inspect",
    "crates/inspector": "inspector", "crates/llm": "llm", "crates/lot": "lot",
    "crates/macros": "macros", "crates/monster": "monster", "crates/narrate": "narrate",
    "crates/narrate-macro": "narrate_macro", "crates/parser": "parser", "crates/prime": "prime",
    "crates/propose": "propose", "crates/rec": "rec", "crates/rope": "rope", "crates/sem": "sem",
    "crates/showcase": "showcase", "crates/surface": "surface", "crates/svg": "svg",
    "crates/uir": "uir", "crates/urnaui": "urnaui", "crates/wire": "wire",
}
# new crate dir -> lib name of the target layout (the directory's name)
NEW_CRATES = {d: os.path.basename(d) for d in [
    "rust/accord", "rust/claude", "rust/editor", "rust/engine", "rust/gitdrv", "rust/import",
    "rust/llmrun", "rust/macros", "rust/markup", "rust/motion", "rust/offers", "rust/plevir",
    "rust/record", "rust/replay", "rust/screen", "rust/socket",
    "apps/agenteval", "apps/idestudio", "apps/inspector", "apps/mirroring", "apps/uigallery",
    "apps/urnadbapp",
    "demo/arcade", "demo/camera", "demo/commit", "demo/inputs", "demo/intent", "demo/layers",
    "demo/lottie", "demo/mobile", "demo/mockup", "demo/player", "demo/primes", "demo/shapes",
    "demo/ticker",
    "tool/bench/measure", "tool/bench/regress", "tool/tests",
]}
# package renames, old lib name -> new lib name (merged crates map to the host)
CRATE_RENAMES = {
    "claude_model": "claude", "comps": "engine", "git": "gitdrv", "ide": "idestudio",
    "inspect": "replay", "llm": "llmrun", "lot": "import", "svg": "import", "parser": "import",
    "monster": "motion", "narrate": "markup", "narrate_macro": "macros", "prime": "primes",
    "propose": "offers", "rec": "record", "rope": "editor", "sem": "accord",
    "showcase": "uigallery", "surface": "screen", "uir": "plevir", "urnaui": "urnadbapp",
    "wire": "socket", "engine": "engine", "macros": "macros", "inspector": "inspector",
}


def load():
    with open(os.path.join(ROOT, "migration.toml"), "rb") as f:
        return tomllib.load(f)["file"]


def crate_of(path, crates):
    best = None
    for d in crates:
        if path.startswith(d + "/") and (best is None or len(d) > len(best)):
            best = d
    return best


# modules a binary declares (its main.rs) in a crate that also has a library:
# part of the binary, not of the library, wherever their file sits
BIN_MODULES = {
    "crates/ide/src/actions.rs", "crates/ide/src/keys.rs", "crates/ide/src/pace.rs",
    "crates/ide/src/recording.rs", "crates/ide/src/renderer.rs", "crates/ide/src/watcher.rs",
    "crates/prime/src/scene.rs",
    "apps/idestudio/src/interfaces/window/keymap.rs",
    "apps/idestudio/src/interfaces/window/keyboard.rs",
    "apps/idestudio/src/interfaces/window/pace.rs",
    "apps/idestudio/src/interfaces/window/record/shell.rs",
    "apps/idestudio/src/interfaces/window/renderer.rs",
    "apps/idestudio/src/repository/watcher.rs",
    "demo/primes/src/scene.rs",
}


def lib_module(path, crate_dir):
    """Module path (list of segments) of a lib source file, or None when the
    file is not part of the crate's library tree (bins, tests, examples)."""
    if path in BIN_MODULES:
        return None
    rel = path[len(crate_dir) + 1:]
    if not rel.startswith("src/") or not rel.endswith(".rs"):
        return None
    rel = rel[4:]
    if rel.startswith("bin/") or rel in ("main.rs", "main_web.rs"):
        return None
    parts = rel[:-3].split("/")
    if parts[-1] in ("mod", "lib"):
        parts = parts[:-1]
    return parts


def module_table():
    """(old lib, old module segments) -> (new lib, new module segments)."""
    table = {}
    for e in load():
        if e["kind"] not in ("move", "merge") or not e["from"].endswith(".rs"):
            continue
        oc = crate_of(e["from"], OLD_CRATES)
        nc = crate_of(e["to"], NEW_CRATES)
        if not oc or not nc:
            continue
        om = lib_module(e["from"], oc)
        nm = lib_module(e["to"], nc)
        if om is None or nm is None:
            continue
        table[(OLD_CRATES[oc], tuple(om))] = (NEW_CRATES[nc], tuple(nm))
    # crate roots of renamed crates even when lib.rs is not a lib module move
    for old, new in CRATE_RENAMES.items():
        table.setdefault((old, ()), (new, ()))
    return table


# exact paths that name something other than the module: `engine::component`
# is also the attribute macro re-exported from `macros`
EXACT_SKIP = {("engine", ("component",))}


def translate(table, lib, segs):
    """Longest module prefix of `segs` under crate `lib` -> (new lib, new segs)."""
    if (lib, tuple(segs)) in EXACT_SKIP:
        return CRATE_RENAMES.get(lib, lib), list(segs)
    for n in range(len(segs), -1, -1):
        hit = table.get((lib, tuple(segs[:n])))
        if hit:
            return hit[0], list(hit[1]) + list(segs[n:])
    return CRATE_RENAMES.get(lib, lib), list(segs)


def reverse_moves():
    """target path -> source path (moves and the primary of merges)."""
    return {e["to"]: e["from"] for e in load() if e["kind"] == "move"}


def rs_files(base):
    out = []
    for d in ("rust", "apps", "demo", "tool"):
        for dirpath, _, files in os.walk(os.path.join(base, d)):
            if "/target" in dirpath or "/android/" in dirpath:
                continue
            for f in files:
                if f.endswith(".rs"):
                    out.append(os.path.relpath(os.path.join(dirpath, f), base))
    return sorted(out)


IDENT = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")
