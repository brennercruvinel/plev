#!/bin/bash
# build and run the gallery as a mobile app (apps/uigallery/android, ios).
#
#   tool/tasks/mobileapp.sh android          # release cdylib, debug APK
#   PROFILE=debug tool/tasks/mobileapp.sh android
#   tool/tasks/mobileapp.sh ios              # debug, aarch64-apple-ios-sim
#   PROFILE=release tool/tasks/mobileapp.sh ios
#   tool/tasks/mobileapp.sh ios-run          # iPhone 17, Debug
#   DEVICE="iPhone 17 Pro" tool/tasks/mobileapp.sh ios-run
#
# android: compiles the gallery cdylib for arm64-v8a + x86_64 via cargo-ndk
# into app/src/main/jniLibs, then assembles the APK with Gradle. Requires
# the Android SDK/NDK, a JDK 17, cargo-ndk, and the android Rust targets.
#
# ios: produces UIGallery.app under build/Build/Products/Debug-iphonesimulator.
# Requires Xcode (selected via DEVELOPER_DIR), the aarch64-apple-ios-sim
# Rust target, and xcodegen.
#
# ios-run: boots a simulator, installs the built UIGallery.app, launches it
# and grabs a screenshot. Run `ios` first.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
APP_DIR="$REPO_ROOT/apps/uigallery"

android() {
    export ANDROID_HOME="${ANDROID_HOME:-$HOME/Library/Android/sdk}"
    export ANDROID_NDK_HOME="${ANDROID_NDK_HOME:-$ANDROID_HOME/ndk/27.2.12479018}"
    export JAVA_HOME="${JAVA_HOME:-$HOME/jdk17/Contents/Home}"
    export PATH="$JAVA_HOME/bin:$PATH"

    cd "$APP_DIR/android"
    JNILIBS="$(pwd)/app/src/main/jniLibs"

    PROFILE="${PROFILE:-release}"
    CARGO_PROFILE_FLAG="--release"
    [ "$PROFILE" = "debug" ] && CARGO_PROFILE_FLAG=""

    echo ">> building uigallery cdylib via cargo-ndk (arm64-v8a, x86_64, $PROFILE)"
    ( cd "$REPO_ROOT" && cargo ndk \
        -t arm64-v8a -t x86_64 \
        -o "$JNILIBS" \
        build -p uigallery --lib $CARGO_PROFILE_FLAG )

    # cargo-ndk also emits engine's own cdylib (libengine.so); it is dead weight —
    # libuigallery.so statically links engine and GameActivity loads only "uigallery".
    find "$JNILIBS" -name 'libengine.so' -delete

    echo ">> jniLibs:"; find "$JNILIBS" -name 'libuigallery.so' -exec ls -la {} \;

    echo ">> assembling debug APK"
    ./gradlew assembleDebug --no-daemon

    echo ">> APK: $(pwd)/app/build/outputs/apk/debug/app-debug.apk"
}

ios() {
    export DEVELOPER_DIR="${DEVELOPER_DIR:-/Applications/Xcode.app/Contents/Developer}"
    cd "$APP_DIR/ios"

    TRIPLE="aarch64-apple-ios-sim"
    PROFILE="${PROFILE:-debug}"
    CARGO_PROFILE_FLAG=""
    XCODE_CONFIG="Debug"
    if [ "$PROFILE" = "release" ]; then
        CARGO_PROFILE_FLAG="--release"
        XCODE_CONFIG="Release"
    fi

    echo ">> building Rust staticlib ($TRIPLE, $PROFILE)"
    ( cd "$REPO_ROOT" && cargo build -p uigallery --lib --target "$TRIPLE" $CARGO_PROFILE_FLAG )

    LIBDIR="$REPO_ROOT/target/$TRIPLE/$PROFILE"
    echo ">> staticlib: $LIBDIR/libuigallery.a"

    echo ">> generating Xcode project"
    xcodegen generate

    echo ">> building app ($XCODE_CONFIG, iphonesimulator)"
    xcodebuild \
        -project UIGallery.xcodeproj \
        -scheme UIGallery \
        -configuration "$XCODE_CONFIG" \
        -sdk iphonesimulator \
        -destination 'generic/platform=iOS Simulator' \
        -derivedDataPath build \
        ARCHS=arm64 \
        EXCLUDED_ARCHS=x86_64 \
        ONLY_ACTIVE_ARCH=YES \
        OTHER_LDFLAGS="$LIBDIR/libuigallery.a -lc++" \
        CODE_SIGNING_ALLOWED=NO \
        build

    echo ">> app: $(pwd)/build/Build/Products/$XCODE_CONFIG-iphonesimulator/UIGallery.app"
}

ios_run() {
    export DEVELOPER_DIR="${DEVELOPER_DIR:-/Applications/Xcode.app/Contents/Developer}"
    cd "$APP_DIR/ios"

    XCODE_CONFIG="${XCODE_CONFIG:-Debug}"
    APP="build/Build/Products/$XCODE_CONFIG-iphonesimulator/UIGallery.app"
    BUNDLE_ID="com.plev.uigallery"
    DEVICE="${DEVICE:-iPhone 17}"

    echo ">> booting $DEVICE"
    xcrun simctl boot "$DEVICE" 2>/dev/null || true
    xcrun simctl bootstatus "$DEVICE" || true

    echo ">> installing $APP"
    xcrun simctl install "$DEVICE" "$APP"

    echo ">> launching $BUNDLE_ID"
    xcrun simctl launch "$DEVICE" "$BUNDLE_ID" || true

    sleep 5
    xcrun simctl io "$DEVICE" screenshot ios_uigallery.png
    echo ">> screenshot: $(pwd)/ios_uigallery.png"
}

case "${1:-}" in
    android) android ;;
    ios) ios ;;
    ios-run) ios_run ;;
    *) echo "usage: tool/tasks/mobileapp.sh android | ios | ios-run" >&2; exit 2 ;;
esac
