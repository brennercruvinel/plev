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
