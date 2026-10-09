#!/bin/bash
# screencap.sh: capture one frame of the gallery on a target and classify it
# against the desktop's frame of the same page, theme, viewport and scale
# (task-77). one target per call:
#
#   tool/tasks/screencap.sh web [options]
#   tool/tasks/screencap.sh ios [options]
#   tool/tasks/screencap.sh android [options]
#
# ---- web
# capture one frame of the showcase's web build and classify it against the
# desktop's frame of the same page, theme, viewport and scale (task-77).
#
#   script/capture-web [--page <title> | --all] [--theme <name>] [--width <px>]
#                      [--height <px>] [--scale <s>] [--out <dir>] [--port <n>]
#
# --all: every page of the gallery in one browser, each against the
# desktop's, one line per page in <out>/report.txt.
#
# defaults: the launch page (Theme) in core, 1280x800 at scale 1, into
# target/determinism/web-<time>/. the build is `script/web build --release`
# (task-20's path, web/dist); the page is served on 127.0.0.1 and opened by
# python playwright in the headless chromium shell with WebGPU on Metal
# (`--enable-unsafe-webgpu --use-angle=metal`; without the second flag the
# adapter is SwiftShader, software, and the capture says so). the viewport is
# the page's size and the scale its deviceScaleFactor.
#
# the browser has no launch arguments: the page and the theme go in the
# URL's query (`?page=Data%20Table&theme=hoff`), which the web build reads
# as the desktop reads its command line (`engine::window::launch_args`).
#
# ---- ios
# capture one frame of the showcase on the iOS simulator and classify it
# against the desktop's frame of the same page, theme, viewport and scale
# (task-77).
#
#   script/capture-ios [--page <title> | --all] [--theme <name>] [--device <name>]
#                      [--out <dir>]
#
# --all: every page of the gallery, launched in turn on the one booted
# simulator, each against the desktop's, one line per page in
# <out>/report.txt.
#
# defaults: the launch page (Theme) in core on the "iPhone 17 Pro", into
# target/determinism/ios-<time>/. the build is task-19's
# ios/showcase/build_ios.sh; the launch is here and not in run_ios.sh because
# the page and the theme go as launch arguments (App::launch_view reads
# `[page] [theme]`), which run_ios.sh does not pass. the viewport and the
# scale are the device type's screen (its profile.plist): 402x874 at 3 on
# the iPhone 17 Pro. the status bar is pinned (9:41, full battery) so the
# band iOS draws over the app is the same every run; the table declares it
# (status-bar, home-indicator) as the platform's.
#
# ---- android
# capture one frame of the showcase on the android emulator and classify it
# against the desktop's frame of the same page, theme, viewport and scale
# (task-77).
#
#   script/capture-android [--page <title>] [--theme <name>] [--avd <name>]
#                          [--out <dir>]
#
# defaults: the launch page (Theme) in core on the AVD "plev35" (android 35,
# google_apis, arm64-v8a), into target/determinism/android-<time>/. the build
# is task-18's android/build_android.sh (cargo-ndk, then gradle; the first
# gradle run downloads its distribution and dependencies). the emulator runs
# without a window. the viewport and the scale come from the running device:
# `wm size` is the physical screen, `wm density` / 160 the scale (420 dpi is
# 2.625), the logical size their quotient, as winit gives the app.
#
# android has no launch arguments the gallery reads (std::env::args is
# empty there): the page and the theme go through the keyboard, as a
# person's: the digits 1 to 9 and 0 select the first ten pages, T cycles
# core, hoff, dark, light. a page past the tenth needs a launch channel (an
# intent extra) the android build does not have yet.
#
set -euo pipefail

# ---- what the three targets share
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

# ---- web (bodies unindented: they hold heredocs)
capture_web() {

PAGE="Theme"; ALL=""; THEME="core"; WIDTH=1280; HEIGHT=800; SCALE=1; PORT=8090
OUT="$REPO_ROOT/target/determinism/web-$(date +%Y%m%d-%H%M%S)"
while [ $# -gt 0 ]; do
    case "$1" in
        --page) PAGE="$2"; shift 2 ;;
        --all) ALL=1; shift ;;
        --theme) THEME="$2"; shift 2 ;;
        --width) WIDTH="$2"; shift 2 ;;
        --height) HEIGHT="$2"; shift 2 ;;
        --scale) SCALE="$2"; shift 2 ;;
        --out) OUT="$2"; shift 2 ;;
        --port) PORT="$2"; shift 2 ;;
        -h | --help) usage; exit 0 ;;
        *) echo "capture-web: unknown argument '$1'" >&2; exit 2 ;;
    esac
done

need trunk "brew install trunk"
need wasm-bindgen "cargo install --locked wasm-bindgen-cli --version 0.2.123 (the version Cargo.lock pins)"
need python3 "brew install python"
need sips "part of macOS"
need_rust_target wasm32-unknown-unknown
if command -v python3 >/dev/null 2>&1 && ! python3 -c "import playwright" 2>/dev/null; then
    MISSING="$MISSING
  python playwright: python3 -m pip install playwright"
fi
if ! ls -d "$HOME"/Library/Caches/ms-playwright/chromium* >/dev/null 2>&1; then
    MISSING="$MISSING
  a playwright chromium: python3 -m playwright install chromium-headless-shell"
fi
missing_stop web

mkdir -p "$OUT"
echo ">> building the web target (script/web build --release)"
"$REPO_ROOT/script/web" build --release

echo ">> serving web/dist on 127.0.0.1:$PORT"
python3 -m http.server "$PORT" --bind 127.0.0.1 --directory "$REPO_ROOT/web/dist" >/dev/null 2>&1 &
SERVER=$!
trap 'kill "$SERVER" 2>/dev/null || true' EXIT
sleep 1

# the pages to open: "<title>\t<png>" per line
PAGES="$OUT/pages.tsv"
if [ -n "$ALL" ]; then
    echo ">> the desktop's frame of every page: $THEME at ${WIDTH}x${HEIGHT}, scale $SCALE"
    desktop_all "$THEME" "$WIDTH" "$HEIGHT" "$SCALE" "$OUT" >"$OUT/titles.txt"
    mkdir -p "$OUT/web"
    while IFS= read -r title; do
        printf '%s\t%s\n' "$title" "$OUT/web/$(slug "$title").png"
    done <"$OUT/titles.txt" >"$PAGES"
else
    SHOT="$OUT/web.png"
    printf '%s\t%s\n' "$PAGE" "$SHOT" >"$PAGES"
fi

echo ">> capturing $(wc -l <"$PAGES" | tr -d ' ') page(s) in $THEME at ${WIDTH}x${HEIGHT}, scale $SCALE"
python3 - "$PORT" "$WIDTH" "$HEIGHT" "$SCALE" "$THEME" "$PAGES" <<'PY'
import asyncio, sys
from urllib.parse import urlencode
from playwright.async_api import async_playwright

port, width, height, scale, theme, pages = sys.argv[1:]
pages = [line.rstrip("\n").split("\t") for line in open(pages) if line.strip()]

async def main():
    async with async_playwright() as p:
        # the emulated deviceScaleFactor alone moves devicePixelRatio but
        # not a ResizeObserver's device-pixel-content-box, which winit
        # sizes the canvas by: at scale 2 the app got the CSS size as its
        # physical size and laid out at half width. the flag moves both.
        browser = await p.chromium.launch(
            headless=True, args=["--enable-unsafe-webgpu", "--use-angle=metal",
                                 f"--force-device-scale-factor={scale}"])
        for title, shot in pages:
            page = await browser.new_page(
                viewport={"width": round(float(width)), "height": round(float(height))},
                device_scale_factor=float(scale))
            ready = asyncio.get_running_loop().create_future()
            def console(msg, ready=ready):
                if "GPU context ready" in msg.text and not ready.done():
                    ready.set_result(True)
            page.on("console", console)
            query = urlencode({"page": title, "theme": theme})
            await page.goto(f"http://127.0.0.1:{port}/?{query}")
            await asyncio.wait_for(ready, timeout=60)
            if title == pages[0][0]:
                adapter = await page.evaluate("""async () => {
                    const a = await navigator.gpu.requestAdapter();
                    const i = (a && a.info) || {};
                    return [i.vendor, i.architecture].join(' / ');
                }""")
                print(f">> WebGPU adapter: {adapter}")
                if "swiftshader" in adapter:
                    print(">> warning: a software adapter; this is not the GPU's frame", file=sys.stderr)
            await page.wait_for_timeout(1500)
            await page.screenshot(path=shot)
            await page.close()
        await browser.close()

asyncio.run(main())
PY

if [ -n "$ALL" ]; then
    compare_all web "$OUT/titles.txt" "$THEME" "$SCALE" "$OUT"
else
    desktop_and_compare web "$SHOT" "$PAGE" "$THEME" "$WIDTH" "$HEIGHT" "$SCALE" "$OUT"
fi
}

# ---- ios (bodies unindented: they hold heredocs)
capture_ios() {

PAGE="Theme"; ALL=""; THEME="core"; DEVICE="iPhone 17 Pro"
OUT="$REPO_ROOT/target/determinism/ios-$(date +%Y%m%d-%H%M%S)"
while [ $# -gt 0 ]; do
    case "$1" in
        --page) PAGE="$2"; shift 2 ;;
        --all) ALL=1; shift ;;
        --theme) THEME="$2"; shift 2 ;;
        --device) DEVICE="$2"; shift 2 ;;
        --out) OUT="$2"; shift 2 ;;
        -h | --help) usage; exit 0 ;;
        *) echo "capture-ios: unknown argument '$1'" >&2; exit 2 ;;
    esac
done

export DEVELOPER_DIR="${DEVELOPER_DIR:-/Applications/Xcode.app/Contents/Developer}"
need xcrun "Xcode from the App Store, then xcode-select -s /Applications/Xcode.app"
need xcodebuild "Xcode from the App Store"
need xcodegen "brew install xcodegen"
need plutil "part of macOS"
need sips "part of macOS"
need_rust_target aarch64-apple-ios-sim
missing_stop ios

TYPES="$(xcrun simctl list devicetypes -j)"
PROFILE="$(python3 -c 'import json,sys; t=[d for d in json.loads(sys.argv[1])["devicetypes"] if d["name"]==sys.argv[2]]; print(t[0]["bundlePath"] if t else "")' "$TYPES" "$DEVICE")"
if [ -z "$PROFILE" ] || ! xcrun simctl list devices available | grep -q "    $DEVICE ("; then
    echo "capture-ios: no available simulator named '$DEVICE' (xcrun simctl list devices available)" >&2
    exit 2
fi
# the screen: `mainScreen<Width|Height|Scale>` of profile.plist on older
# Xcodes; the Xcode 26 profiles keep it in capabilities.plist instead
# (`ScreenDimensionsCapability`, `main-screen-<width|height|scale>`)
screen() {
    local lower
    lower="$(echo "$1" | tr '[:upper:]' '[:lower:]')"
    plutil -extract "mainScreen$1" raw "$PROFILE/Contents/Resources/profile.plist" 2>/dev/null \
        || plutil -extract "capabilities.ScreenDimensionsCapability.main-screen-$lower" raw \
            "$PROFILE/Contents/Resources/capabilities.plist" 2>/dev/null
}
PX_W="$(screen Width)"; PX_H="$(screen Height)"; SCALE="$(screen Scale)"
# capabilities.plist writes the scale as a real ("3.000000"): as the desktop names it
SCALE="$(python3 -c "print('%g' % float('${SCALE}'))")"
WIDTH="$(python3 -c "print(repr(${PX_W} / ${SCALE}))")"
HEIGHT="$(python3 -c "print(repr(${PX_H} / ${SCALE}))")"

mkdir -p "$OUT"
echo ">> building the showcase for the simulator (ios/showcase/build_ios.sh)"
# build_ios.sh links the staticlib from target/<triple>/<profile> of the
# repository: a CARGO_TARGET_DIR elsewhere would leave it looking at nothing
(unset CARGO_TARGET_DIR; "$REPO_ROOT/ios/showcase/build_ios.sh")
APP="$REPO_ROOT/ios/showcase/build/Build/Products/Debug-iphonesimulator/ShowcaseDemo.app"

echo ">> booting $DEVICE"
xcrun simctl boot "$DEVICE" 2>/dev/null || true
xcrun simctl bootstatus "$DEVICE" -b
xcrun simctl status_bar "$DEVICE" override --time "9:41" --dataNetwork wifi \
    --wifiMode active --wifiBars 3 --cellularMode active --cellularBars 4 \
    --batteryState charged --batteryLevel 100
xcrun simctl install "$DEVICE" "$APP"

# shoot <page> <png>: the app launched on the page and the theme, its frame
shoot() {
    xcrun simctl terminate "$DEVICE" com.plev.showcase 2>/dev/null || true
    xcrun simctl launch "$DEVICE" com.plev.showcase "$1" "$THEME" >/dev/null
    sleep "$2"
    xcrun simctl io "$DEVICE" screenshot --type=png "$3" >/dev/null 2>&1
}

if [ -n "$ALL" ]; then
    echo ">> the desktop's frame of every page: $THEME at ${WIDTH}x${HEIGHT}, scale $SCALE"
    desktop_all "$THEME" "$WIDTH" "$HEIGHT" "$SCALE" "$OUT" >"$OUT/titles.txt"
    mkdir -p "$OUT/ios"
    echo ">> launching $(wc -l <"$OUT/titles.txt" | tr -d ' ') pages in $THEME"
    while IFS= read -r title; do
        shoot "$title" 4 "$OUT/ios/$(slug "$title").png"
    done <"$OUT/titles.txt"
    compare_all ios "$OUT/titles.txt" "$THEME" "$SCALE" "$OUT"
    exit 0
fi

echo ">> launching $PAGE / $THEME"
SHOT="$OUT/ios.png"
shoot "$PAGE" 5 "$SHOT"
echo ">> $SHOT: ${PX_W}x${PX_H}, ${WIDTH}x${HEIGHT} logical at $SCALE"

desktop_and_compare ios "$SHOT" "$PAGE" "$THEME" "$WIDTH" "$HEIGHT" "$SCALE" "$OUT"
}

# ---- android (bodies unindented: they hold heredocs)
capture_android() {

PAGE="Theme"; THEME="core"; AVD="plev35"
OUT="$REPO_ROOT/target/determinism/android-$(date +%Y%m%d-%H%M%S)"
while [ $# -gt 0 ]; do
    case "$1" in
        --page) PAGE="$2"; shift 2 ;;
        --theme) THEME="$2"; shift 2 ;;
        --avd) AVD="$2"; shift 2 ;;
        --out) OUT="$2"; shift 2 ;;
        -h | --help) usage; exit 0 ;;
        *) echo "capture-android: unknown argument '$1'" >&2; exit 2 ;;
    esac
done

# the paths build_android.sh reads, unless the environment says otherwise
export ANDROID_HOME="${ANDROID_HOME:-$HOME/Library/Android/sdk}"
export JAVA_HOME="${JAVA_HOME:-$HOME/jdk17/Contents/Home}"
ADB="$ANDROID_HOME/platform-tools/adb"
EMULATOR="$ANDROID_HOME/emulator/emulator"
SDK_HINT="brew install --cask android-commandlinetools, then sdkmanager \"platform-tools\" \"emulator\" \"platforms;android-35\" \"system-images;android-35;google_apis;arm64-v8a\" \"ndk;27.2.12479018\" (with ANDROID_HOME at that sdk)"
SDK_MISSING=""
[ -x "$ADB" ] || SDK_MISSING="$SDK_MISSING adb"
[ -x "$EMULATOR" ] || SDK_MISSING="$SDK_MISSING emulator"
[ -d "$ANDROID_HOME/ndk/27.2.12479018" ] || SDK_MISSING="$SDK_MISSING ndk-27.2"
if [ -n "$SDK_MISSING" ]; then
    MISSING="$MISSING
  the android sdk at $ANDROID_HOME (missing:$SDK_MISSING): $SDK_HINT"
fi
need_path "$JAVA_HOME/bin/java" "a JDK 17" "brew install --cask temurin@17, then JAVA_HOME=\$(/usr/libexec/java_home -v 17)"
need cargo-ndk "cargo install cargo-ndk"
need sips "part of macOS"
need_rust_target aarch64-linux-android
need_rust_target x86_64-linux-android
if [ -x "$EMULATOR" ] && ! "$EMULATOR" -list-avds 2>/dev/null | grep -qx "$AVD"; then
    MISSING="$MISSING
  the AVD $AVD: avdmanager create avd -n $AVD -k \"system-images;android-35;google_apis;arm64-v8a\""
fi
missing_stop android

KEYS=("Theme" "Typography" "Icons" "Effects" "Accordion" "Alert" "Alert Dialog" "Aspect Ratio" "Attachment" "Avatar")
KEY=""
for i in "${!KEYS[@]}"; do
    if [ "$(echo "${KEYS[$i]}" | tr '[:upper:]' '[:lower:]')" = "$(echo "$PAGE" | tr '[:upper:]' '[:lower:]')" ]; then
        KEY=$(( (i + 1) % 10 ))
        PAGE="${KEYS[$i]}"
    fi
done
if [ -z "$KEY" ]; then
    echo "capture-android: '$PAGE' is not one of the first ten pages, the only ones a key selects" >&2
    exit 2
fi
case "$THEME" in
    core) PRESSES=0 ;; hoff) PRESSES=1 ;; dark) PRESSES=2 ;; light) PRESSES=3 ;;
    *) echo "capture-android: the T key reaches core, hoff, dark and light, not '$THEME'" >&2; exit 2 ;;
esac

mkdir -p "$OUT"
echo ">> building the showcase apk (android/build_android.sh)"
"$REPO_ROOT/android/build_android.sh"
APK="$REPO_ROOT/android/app/build/outputs/apk/debug/app-debug.apk"

echo ">> starting the emulator $AVD without a window"
"$EMULATOR" -avd "$AVD" -no-window -no-audio -no-boot-anim -no-snapshot >"$OUT/emulator.log" 2>&1 &
EMU=$!
trap '"$ADB" emu kill >/dev/null 2>&1 || kill "$EMU" 2>/dev/null || true' EXIT
"$ADB" wait-for-device
until [ "$("$ADB" shell getprop sys.boot_completed 2>/dev/null | tr -d '\r')" = "1" ]; do
    sleep 2
done
# a fresh boot applies the system's color overlays a moment after
# boot_completed; an activity started before that is relaunched, and the
# gallery's never draws again (task-18, pending), so the app waits
sleep 20
echo ">> renderer: $("$ADB" shell getprop ro.hardware.egl 2>/dev/null | tr -d '\r') / vulkan $("$ADB" shell getprop ro.hardware.vulkan 2>/dev/null | tr -d '\r')"

"$ADB" install -r "$APK" >/dev/null
"$ADB" shell am force-stop com.plev.showcase
"$ADB" shell am start -W -n com.plev.showcase/.MainActivity >/dev/null
sleep 5
if [ "$KEY" != "1" ]; then
    "$ADB" shell input keyevent "KEYCODE_$KEY"
    sleep 1
fi
# not `seq 1 "$PRESSES"`: the BSD seq of macOS counts down, `seq 1 0` is "1 0"
for ((i = 0; i < PRESSES; i++)); do
    "$ADB" shell input keyevent KEYCODE_T
    sleep 1
done
sleep 2

SHOT="$OUT/android.png"
"$ADB" exec-out screencap -p >"$SHOT"
SIZE="$("$ADB" shell wm size | tr -d '\r' | awk -F': ' '/size/ {print $2}' | tail -n 1)"
DPI="$("$ADB" shell wm density | tr -d '\r' | awk -F': ' '/density/ {print $2}' | tail -n 1)"
PX_W="${SIZE%x*}"; PX_H="${SIZE#*x}"
SCALE="$(python3 -c "print(repr(${DPI} / 160))")"
WIDTH="$(python3 -c "print(repr(${PX_W} / (${DPI} / 160)))")"
HEIGHT="$(python3 -c "print(repr(${PX_H} / (${DPI} / 160)))")"
echo ">> $SHOT: ${PX_W}x${PX_H} at ${DPI} dpi, ${WIDTH}x${HEIGHT} logical at $SCALE"

desktop_and_compare android "$SHOT" "$PAGE" "$THEME" "$WIDTH" "$HEIGHT" "$SCALE" "$OUT"
}

case "${1:-}" in
    web) shift; capture_web "$@" ;;
    ios) shift; capture_ios "$@" ;;
    android) shift; capture_android "$@" ;;
    "" | -h | --help) usage; exit 0 ;;
    *) echo "screencap.sh: unknown target '$1' (web, ios, android)" >&2; exit 2 ;;
esac
