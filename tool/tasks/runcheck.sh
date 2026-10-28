#!/usr/bin/env bash
# every executable of the repository, run: the apps, the demos, the crates'
# command lines and the tools. the gate (fullcheck.sh) proves the code
# compiles and its tests pass; this proves each program starts and does its
# job on this machine.
#
#   tool/tasks/runcheck.sh [--out <dir>] [--wait <s>] [<name>...]
#
# a window program passes when it is still running after --wait seconds
# (default 6) with a window on screen; on macOS each of its windows is
# captured to <out>/<name>[-n].png. a command line passes on its exit code,
# or on the line it must print. names narrow the run (`runcheck.sh arcade
# lottie`). the report is <out>/report.md (default: target/runcheck).
#
# the inputs are the repository's own fixtures. two programs need what the
# repository does not hold, and say so instead of failing:
#   llm-bench  the TinyLlama 1.1B Chat directory in PLEV_LLM_MODEL, else
#              the one `hf download TinyLlama/TinyLlama-1.1B-Chat-v1.0` caches
#   urnadbapp  the sibling urna checkout (../urna), as fullcheck.sh does;
#              it opens URNA_SAMPLE, else the sample urna setup installs
# intent runs the rules only: PLEV_CLAUDE points at no program, so no
# model is asked (a real `claude -p` costs, and is not this check's).
set -u

cd "$(dirname "$0")/../.." || exit 1
ROOT=$(pwd)
OUT="$ROOT/target/runcheck"
WAIT=6
ONLY=""
while [ $# -gt 0 ]; do
    case "$1" in
        --out) OUT="$2"; shift 2 ;;
        --wait) WAIT="$2"; shift 2 ;;
        -h|--help) sed -n '2,22p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) ONLY="$ONLY $1"; shift ;;
    esac
done
mkdir -p "$OUT"
REPORT="$OUT/report.md"
BIN="$ROOT/target/debug"
FIX_REACT="rust/import/tests/fixtures/react"
SESSION="apps/idestudio/tests/fixtures/sessions/bug84.monster"
LOTTIE="data/media/lottie/buttontap/animation.json"
PASS=0
FAIL=0
SKIP=0
ROWS=""

wanted() { [ -z "$ONLY" ] || case " $ONLY " in *" $1 "*) return 0 ;; *) return 1 ;; esac; }
# a timeout every unix has: perl's alarm ends the program with SIGALRM.
bounded() { local s="$1"; shift; perl -e 'alarm shift; exec @ARGV' "$s" "$@"; }
row() { ROWS="$ROWS| $1 | $2 | $3 | $4 |
"; printf '%-15s %-4s %s\n' "$1" "$3" "$4"; }
pass() { PASS=$((PASS + 1)); row "$1" "$2" "ok" "$3"; }
fail() { FAIL=$((FAIL + 1)); row "$1" "$2" "FAIL" "$3"; }
skip() { SKIP=$((SKIP + 1)); row "$1" "$2" "skip" "$3"; }
last_line() { grep -v '^[[:space:]]*$' "$1" | tail -1 | cut -c1-110 | tr '|' '/'; }

# macOS helper, built once: `winid <pid>` lists the pid's window ids and
# areas; `winid --colors <png>` counts the distinct colours on a grid below
# the title bar, so a window that never drew (one flat colour) is not passed.
WINID=""
if [ "$(uname)" = Darwin ] && command -v swiftc >/dev/null 2>&1; then
    WINID="$OUT/winid"
    if [ ! -x "$WINID" ]; then
        cat > "$OUT/winid.swift" <<'SWIFT'
import CoreGraphics
import ImageIO
import Foundation
let args = CommandLine.arguments
if args[1] == "--colors" {
    let url = URL(fileURLWithPath: args[2]) as CFURL
    guard let src = CGImageSourceCreateWithURL(url, nil),
          let img = CGImageSourceCreateImageAtIndex(src, 0, nil),
          let data = img.dataProvider?.data, let px = CFDataGetBytePtr(data) else { print(0); exit(0) }
    let bpp = img.bitsPerPixel / 8, row = img.bytesPerRow
    var seen = Set<UInt32>()
    for gy in 0..<24 { for gx in 0..<24 {
        let x = img.width * (2 * gx + 1) / 48
        let y = 80 + (img.height - 80) * (2 * gy + 1) / 48
        let o = y * row + x * bpp
        seen.insert(UInt32(px[o]) << 16 | UInt32(px[o + 1]) << 8 | UInt32(px[o + 2]))
    } }
    print(seen.count)
    exit(0)
}
let pid = Int32(args[1])!
let list = CGWindowListCopyWindowInfo([.optionOnScreenOnly], kCGNullWindowID) as! [[String: Any]]
for w in list where (w[kCGWindowOwnerPID as String] as? Int32) == pid {
    let b = w[kCGWindowBounds as String] as! [String: Double]
    guard b["Width"]! > 1 && b["Height"]! > 1 else { continue }
    print(w[kCGWindowNumber as String]!, Int(b["Width"]! * b["Height"]!))
}
SWIFT
        swiftc -O "$OUT/winid.swift" -o "$WINID" 2>/dev/null || WINID=""
    fi
fi

# gui <name> <cmd...>: started, alive after WAIT, its windows captured.
gui() {
    local name="$1"; shift
    wanted "$name" || return 0
    local log="$OUT/$name.log"
    "$@" > "$log" 2>&1 &
    local pid=$!
    sleep "$WAIT"
    if ! kill -0 "$pid" 2>/dev/null; then
        wait "$pid"; fail "$name" window "exited $? before ${WAIT}s: $(last_line "$log")"
        return
    fi
    local shots="" n=0
    if [ -n "$WINID" ]; then
        for id in $("$WINID" "$pid" | sort -k2 -rn | awk '{print $1}'); do
            local png="$OUT/$name.png" _
            [ $n -gt 0 ] && png="$OUT/$name-$n.png"
            # a window still settling can refuse a capture, or show its first
            # frame late: up to three tries, three seconds apart.
            for _ in 1 2 3; do
                screencapture -x -o -l "$id" "$png" 2>/dev/null && [ -s "$png" ] &&
                    [ "$("$WINID" --colors "$png")" -ge 2 ] && break
                sleep 3
            done
            [ -s "$png" ] && shots="$shots $(basename "$png")"
            n=$((n + 1))
        done
    fi
    kill "$pid" 2>/dev/null; wait "$pid" 2>/dev/null
    if [ -z "$WINID" ]; then
        pass "$name" window "running after ${WAIT}s (no window capture off macOS)"
    elif [ $n -eq 0 ]; then
        fail "$name" window "running, but no window on screen"
    elif [ -z "$shots" ]; then
        fail "$name" window "running with $n window(s), none captured"
    elif [ "$("$WINID" --colors "$OUT/$name.png")" -lt 2 ]; then
        fail "$name" window "running, but its window is one flat colour: it never drew"
    else
        pass "$name" window "running after ${WAIT}s;$shots"
    fi
}

# cli <name> <expect> <cmd...>: passes on exit 0, or on a line matching
# <expect> when one is given (a program whose last step is left out).
cli() {
    local name="$1" expect="$2"; shift 2
    wanted "$name" || return 0
    local log="$OUT/$name.log"
    bounded 300 "$@" > "$log" 2>&1
    local code=$?
    if [ -n "$expect" ] && grep -q -- "$expect" "$log"; then
        pass "$name" cli "printed '$expect'"
    elif [ -z "$expect" ] && [ $code -eq 0 ]; then
        pass "$name" cli "$(last_line "$log")"
    else
        fail "$name" cli "exit $code: $(last_line "$log")"
    fi
}

echo ">> building every executable (workspace, urnadbapp, llmrun)"
cargo build --workspace --bins || { echo "the workspace does not build" >&2; exit 1; }
URNA_OK=0
if [ -f ../urna/rust/format/Cargo.toml ]; then
    cargo build --manifest-path apps/urnadbapp/Cargo.toml && URNA_OK=1
fi
LLM_OK=0
LLM_FEATURES="candle"
[ "$(uname)" = Darwin ] && LLM_FEATURES="candle,metal"
cargo build --release --manifest-path rust/llmrun/Cargo.toml --features "$LLM_FEATURES" --bin llm-bench && LLM_OK=1
echo

echo ">> crates and tools (command line)"
cli commit "" "$BIN/commit"
cli text_probe "" "$BIN/text_probe" "$OUT/text_probe.png"
cli convert "" "$BIN/convert" "$LOTTIE" "$OUT/buttontap.monster"
cli transpile "" "$BIN/transpile" "$FIX_REACT/index.tsx" "$FIX_REACT/NoteCard.module.sass" "$FIX_REACT/note-card-variables.sass"
cli plev-inspect "" "$BIN/plev-inspect" "$SESSION"
cli player-frame "" "$BIN/player" "$OUT/buttontap.monster" --frame 0.5 --out "$OUT/player-frame.png"
cli intent "the rules: a verified surface" env PLEV_CLAUDE=/nonexistent/claude "$BIN/intent" --app todo "show only the active items"
cli measure "" "$BIN/measure" --frames 10 --out "$OUT/measure.md"
cli agenteval "" "$BIN/agenteval" --arm scripted --scripted observe --out "$OUT/agenteval" --timeout 30
cli regress "" "$BIN/regress" check
if wanted uigallery-web; then
    if cargo build --target wasm32-unknown-unknown -p uigallery --bin uigallery-web >"$OUT/uigallery-web.log" 2>&1; then
        pass uigallery-web wasm "builds for wasm32 (the browser runs it: tool/tasks/webtarget.sh)"
    else
        fail uigallery-web wasm "$(last_line "$OUT/uigallery-web.log")"
    fi
fi
if wanted llm-bench; then
    # unset: the snapshot `hf download TinyLlama/TinyLlama-1.1B-Chat-v1.0` leaves.
    if [ -z "${PLEV_LLM_MODEL:-}" ]; then
        for dir in "${HF_HOME:-$HOME/.cache/huggingface}"/hub/models--TinyLlama--TinyLlama-1.1B-Chat-v1.0/snapshots/*/; do
            [ -f "$dir/model.safetensors" ] && export PLEV_LLM_MODEL="$dir"
        done
    fi
    if [ $LLM_OK -ne 1 ]; then
        fail llm-bench cli "does not build"
    elif [ -z "${PLEV_LLM_MODEL:-}" ]; then
        skip llm-bench cli "PLEV_LLM_MODEL not set (the TinyLlama 1.1B Chat directory)"
    else
        cli llm-bench "" rust/llmrun/target/release/llm-bench --impl candle --rounds 1 --new 16
    fi
fi
echo

echo ">> apps and demos (windows)"
gui uigallery "$BIN/uigallery"
gui idestudio "$BIN/idestudio"
gui plev-inspector "$BIN/plev-inspector" "$SESSION"
for demo in arcade camera inputs layers mobile primes shapes ticker; do
    gui "$demo" "$BIN/$demo"
done
gui lottie "$BIN/lottie" "$LOTTIE"
gui player "$BIN/player" "$OUT/buttontap.monster"
gui mockup "$BIN/mockup" "$FIX_REACT/index.tsx" "$FIX_REACT/NoteCard.module.sass" "$FIX_REACT/note-card-variables.sass"
if wanted urnadbapp; then
    SAMPLE="${URNA_SAMPLE:-$HOME/.local/share/urna/sample/fakenews.urna}"
    if [ $URNA_OK -ne 1 ]; then
        skip urnadbapp window "no sibling urna checkout at ../urna"
    elif [ -f "$SAMPLE" ]; then
        gui urnadbapp apps/urnadbapp/target/debug/urnadbapp "$SAMPLE"
    else
        gui urnadbapp apps/urnadbapp/target/debug/urnadbapp
    fi
fi

# the agent's side needs an app with its port on: the gallery, its windows
# shared with a watcher (PLEV_MIRROR=1). not PLEV_BACKGROUND: a window kept
# below every other one is never asked to draw, and the mirror shows what it
# draws. its port still carries the name it had before the rename, `showcase`.
if wanted mirroring || wanted mirroring-window || wanted plev; then
    PLEV_AGENT=1 PLEV_MIRROR=1 "$BIN/uigallery" > "$OUT/uigallery-agent.log" 2>&1 &
    APP=$!
    sleep "$WAIT"
    cli plev "" "$BIN/plev" observe --app showcase --timeout 5000
    cli mirroring "" "$BIN/mirroring" --frames 1 --png "$OUT/mirroring-frame.png"
    gui mirroring-window "$BIN/mirroring" --show
    kill "$APP" 2>/dev/null; wait "$APP" 2>/dev/null
fi

{
    echo "# runcheck: $(date '+%Y-%m-%d %H:%M') on $(uname -sm)"
    echo
    echo "$PASS ok, $FAIL failed, $SKIP skipped. Logs and captures beside this file."
    echo
    echo "| program | kind | result | evidence |"
    echo "| --- | --- | --- | --- |"
    printf '%s' "$ROWS"
} > "$REPORT"
echo
echo "$PASS ok, $FAIL failed, $SKIP skipped: $REPORT"
[ $FAIL -eq 0 ]
