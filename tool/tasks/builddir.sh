#!/bin/sh
# the build directories: what they weigh, a ceiling for them, and a warm
# start for a new checkout (task-106).
#
#   tool/tasks/builddir.sh size
#   tool/tasks/builddir.sh trim [--over <GB>] [--yes]
#   tool/tasks/builddir.sh seed <dir> [--from <dir>]
#
# size: what each build directory of the repository weighs: target/ by
# folder, every agent's under target/agents, and the two workspaces that
# keep their own (apps/urnadbapp/target, rust/llmrun/target). a seeded
# directory counts in full here: du cannot tell a clone from a copy.
#
# trim: when the build directory (CARGO_TARGET_DIR, else target/) weighs
# more than --over GB (default 12), removes what the workspace's own crates
# wrote there, for the host and for every target triple with a folder: the
# test executables, the incremental data and the libraries, nine tenths of
# the directory when it was measured (4.5 of 5.2 GB). the dependencies stay,
# so the next build compiles the workspace only. it is cargo's own `cargo
# clean --workspace`, so the folders cargo does not own (target/fonts-noto,
# target/determinism, target/agents) are not touched. without --yes it
# prints what would go (cargo's --dry-run) and removes nothing.
#
# seed: a build directory for another checkout (an agent's worktree),
# started from the dependencies this repository already compiled. <dir> is
# cloned from the main checkout's target/debug copy on write (`cp -c` on
# APFS, `cp --reflink=auto` on linux: no bytes are written until a file
# changes), and then the workspace's own artifacts are removed from the
# copy. that second step is not optional: cargo names a workspace crate the
# same in every checkout and judges it fresh by mtime, so a copy that kept
# them could hand a build an artifact compiled from another checkout's
# sources (docs/Research/Workflows/orchestrate-coding-agents-on-this-repo.md).
# run it from the checkout that will build in <dir>, and not while a build
# is writing the directory it clones. one build directory per checkout stays
# the rule; this spares the new one its cold start. measured on arcade in a
# new worktree: 192 crates and 87 s cold, 14 crates and 14 s seeded.
set -eu

cd "$(dirname "$0")/../.."
ROOT=$(pwd)

usage() {
    sed -n '2,7p' "$0" | sed 's/^# \{0,1\}//' >&2
    exit 2
}

# the size of a path in whole GB and in MB, as "GB MB"
weigh() {
    kb=$(du -sk "$1" 2>/dev/null | cut -f1)
    echo "$((kb / 1048576)) $((kb / 1024))"
}

size() {
    for dir in "$ROOT/target" "$ROOT/apps/urnadbapp/target" "$ROOT/rust/llmrun/target"; do
        [ -d "$dir" ] || continue
        set -- $(weigh "$dir")
        echo ">> ${dir#"$ROOT"/}: $2 MB"
        for part in "$dir"/*/ "$dir"/agents/*/; do
            [ -d "$part" ] || continue
            set -- $(weigh "$part")
            part=${part%/}
            echo "   ${part#"$dir"/}: $2 MB"
        done
    done
}

trim() {
    over=12
    dry="--dry-run"
    while [ $# -gt 0 ]; do
        case "$1" in
            --over) over="$2"; shift 2 ;;
            --yes) dry=""; shift ;;
            *) usage ;;
        esac
    done
    dir="${CARGO_TARGET_DIR:-$ROOT/target}"
    [ -d "$dir" ] || { echo ">> $dir: no build directory yet"; exit 0; }
    set -- $(weigh "$dir")
    if [ "$1" -lt "$over" ]; then
        echo ">> $dir: $2 MB, under $over GB: nothing to remove"
        exit 0
    fi
    echo ">> $dir: $2 MB, over $over GB: the workspace's own artifacts${dry:+ (dry run, --yes removes)}"
    # the host, then every target triple that has a folder (wasm32, the phones)
    cargo clean --workspace --target-dir "$dir" $dry
    for triple in "$dir"/*-*-*/; do
        [ -d "$triple" ] || continue
        cargo clean --workspace --target-dir "$dir" --target "$(basename "$triple")" $dry
    done
    [ -n "$dry" ] || { set -- $(weigh "$dir"); echo ">> $dir: $2 MB now"; }
}

seed() {
    [ $# -ge 1 ] || usage
    dest="$1"
    shift
    # the main checkout is the first entry of the worktree list
    from="$(git worktree list --porcelain | sed -n '1s/^worktree //p')/target"
    while [ $# -gt 0 ]; do
        case "$1" in
            --from) from="$2"; shift 2 ;;
            *) usage ;;
        esac
    done
    [ -d "$from/debug" ] || { echo ">> $from/debug: nothing to clone, build there first" >&2; exit 1; }
    if [ -e "$dest" ] && [ -n "$(ls -A "$dest" 2>/dev/null)" ]; then
        echo ">> $dest: not empty, a seed never overwrites a build directory" >&2
        exit 1
    fi
    mkdir -p "$dest"
    dest=$(cd "$dest" && pwd)
    case "$(uname)" in
        Darwin) cp -c -R "$from/debug" "$dest/debug" ;;
        *) cp -R --reflink=auto "$from/debug" "$dest/debug" ;;
    esac
    [ -f "$from/CACHEDIR.TAG" ] && cp "$from/CACHEDIR.TAG" "$dest/"
    # what the workspace's crates compiled there came from another checkout's sources
    cargo clean --workspace --target-dir "$dest"
    set -- $(weigh "$dest")
    echo ">> $dest: seeded with the dependencies ($2 MB, shared with $from until they change)"
    echo "   export CARGO_TARGET_DIR=$dest"
}

case "${1:-}" in
    size) size ;;
    trim) shift; trim "$@" ;;
    seed) shift; seed "$@" ;;
    *) usage ;;
esac
