# shellcheck shell=bash
# sourced by script/capture-web, script/capture-ios and script/capture-android
# (task-77): what the three share. each captures one frame of the showcase on
# its target (page, theme, viewport and scale named the same way as the
# desktop's `regress capture`), then draws the desktop's frame of the same
# page, theme, viewport and scale through the headless path, then classifies
# the target's frame against it with the target's rows of the table of
# requested differences (crates/showcase/fixtures/determinism/requested.json).
#
# nothing here installs anything: a missing tool stops the script with the
# command that installs it.

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MISSING=""

# usage: the script's header comment.
usage() {
    awk 'NR > 1 && /^#/ { sub(/^# ?/, ""); print; next } NR > 1 { exit }' "$0"
}

# need <command> <how to get it>: remember what is missing.
need() {
    if ! command -v "$1" >/dev/null 2>&1; then
        MISSING="$MISSING
  $1: $2"
    fi
}

# need_path <path> <what> <how to get it>
need_path() {
    if [ ! -e "$1" ]; then
        MISSING="$MISSING
  $2 ($1): $3"
    fi
}

# need_rust_target <triple>
need_rust_target() {
    if ! rustup target list --installed 2>/dev/null | grep -qx "$1"; then
        MISSING="$MISSING
  rust target $1: rustup target add $1"
    fi
}

# stop, naming everything missing, when something is.
missing_stop() {
    if [ -n "$MISSING" ]; then
        echo "capture: this machine lacks what the $1 capture needs:$MISSING" >&2
        echo "nothing was captured. install the above (it needs the network) and run again." >&2
        exit 2
    fi
}

# regress <args...>: the showcase's regression harness, offline.
regress() {
    (cd "$REPO_ROOT" && cargo run --offline -q -p showcase --example regress -- "$@")
}

# to_srgb <png>: a capture tagged with a color profile other than sRGB (a
# display's, a wide gamut one) is converted to sRGB in place with sips, so the
# bytes compare with the desktop's sRGB frame; the profile found is printed.
to_srgb() {
    profile="$(sips -g profile "$1" 2>/dev/null | awk -F': ' '/profile:/ {print $2}')"
    case "$profile" in
        "" | "<nil>" | *sRGB*)
            echo ">> $1: color profile '${profile:-none}', compared as sRGB"
            ;;
        *)
            echo ">> $1: color profile '$profile', converted to sRGB before comparing"
            sips -m "/System/Library/ColorSync/Profiles/sRGB Profile.icc" "$1" >/dev/null
            ;;
    esac
}

# desktop_and_compare <target> <target.png> <page> <theme> <width> <height> <scale> <out>
# draws the desktop's frame of the same page, theme, viewport and scale, and
# classifies the target's frame against it.
desktop_and_compare() {
    target="$1"; shot="$2"; page="$3"; theme="$4"
    width="$5"; height="$6"; scale="$7"; out="$8"
    echo ">> the desktop's frame: $page / $theme at ${width}x${height}, scale $scale"
    regress capture --page "$page" --theme "$theme" --width "$width" --height "$height" \
        --scale "$scale" --out "$out/desktop" || true
    desktop="$(find "$out/desktop" -name '*.png' 2>/dev/null | head -n 1)"
    if [ -z "$desktop" ]; then
        echo "capture: the desktop reference was not drawn (see regress capture --help)" >&2
        exit 3
    fi
    to_srgb "$shot"
    regress compare "$desktop" "$shot" --scale "$scale" --target "$target"
}

# desktop_all <theme> <width> <height> <scale> <out>: the desktop's frame of
# every page of the gallery (`regress capture`), into <out>/desktop; prints
# the page titles, one per line, as its report names them.
desktop_all() {
    regress capture --theme "$1" --width "$2" --height "$3" --scale "$4" \
        --out "$5/desktop" >/dev/null || true
    awk 'col && (NF == 0 || /^\(/) { exit }
         col { title = substr($0, 1, col - 1); sub(/ +$/, "", title); print title }
         /^page +theme/ { col = index($0, "theme") }' "$5/desktop/report.txt"
}

# slug <title>: the name a page's png takes ("Alert Dialog" -> alert-dialog).
slug() {
    echo "$1" | tr '[:upper:]' '[:lower:]' | tr ' ' '-'
}

# the verdict of a `regress compare` line: what follows the parenthesis
# after " at <scale>x" (the target's rows), or the line when it has none.
VERDICT='
import sys
line = sys.stdin.read().strip()
at = line.find(" at ")
open_ = line.find("(", at) if at >= 0 else -1
depth, end = 0, -1
for i in range(max(open_, 0), len(line)):
    depth += {"(": 1, ")": -1}.get(line[i], 0)
    if open_ >= 0 and depth == 0:
        end = i
        break
print(line[end + 3:] if end >= 0 else line)
'

# compare_all <target> <titles file> <theme> <scale> <out>: each page's
# <out>/<target>/<slug>.png against the desktop's, one line per page in
# <out>/report.txt (the page, then the verdict of `regress compare`), and
# the count of each verdict.
compare_all() {
    target="$1"; titles="$2"; theme="$3"; scale="$4"; out="$5"
    : >"$out/report.txt"
    while IFS= read -r title; do
        name="$(slug "$title")"
        shot="$out/$target/$name.png"
        desktop="$(find "$out/desktop" -name "$name@$theme@*.png" | head -n 1)"
        if [ ! -f "$shot" ] || [ -z "$desktop" ]; then
            printf '%s\tmissing\n' "$title" >>"$out/report.txt"
            continue
        fi
        to_srgb "$shot" >/dev/null
        # compare exits 1 on a frame that did not hold: the verdict is the data
        verdict="$({ regress compare "$desktop" "$shot" --scale "$scale" --target "$target" 2>&1 || true; } \
            | tail -n 1 | python3 -c "$VERDICT")"
        printf '%s\t%s\n' "$title" "$verdict" >>"$out/report.txt"
    done <"$titles"
    echo ">> $out/report.txt:"
    cut -f 2 "$out/report.txt" | sed -E 's/^(equal|requested|silent|resized|missing).*/\1/' \
        | sort | uniq -c
}
