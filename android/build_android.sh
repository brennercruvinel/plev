#!/bin/bash
# Build the plev showcase Android APK.
#
#   ./build_android.sh                 # release cdylib, debug APK
#   PROFILE=debug ./build_android.sh
#
# Compiles the showcase cdylib for arm64-v8a + x86_64 via cargo-ndk into
# app/src/main/jniLibs, then assembles the APK with Gradle. Requires the
# Android SDK/NDK, a JDK 17, cargo-ndk, and the android Rust targets.
set -euo pipefail

export ANDROID_HOME="${ANDROID_HOME:-$HOME/Library/Android/sdk}"
export ANDROID_NDK_HOME="${ANDROID_NDK_HOME:-$ANDROID_HOME/ndk/27.2.12479018}"
export JAVA_HOME="${JAVA_HOME:-$HOME/jdk17/Contents/Home}"
export PATH="$JAVA_HOME/bin:$PATH"

cd "$(dirname "$0")"
REPO_ROOT="$(cd .. && pwd)"
JNILIBS="$(pwd)/app/src/main/jniLibs"

PROFILE="${PROFILE:-release}"
CARGO_PROFILE_FLAG="--release"
[ "$PROFILE" = "debug" ] && CARGO_PROFILE_FLAG=""

echo ">> building showcase cdylib via cargo-ndk (arm64-v8a, x86_64, $PROFILE)"
( cd "$REPO_ROOT" && cargo ndk \
    -t arm64-v8a -t x86_64 \
    -o "$JNILIBS" \
    build -p showcase --lib $CARGO_PROFILE_FLAG )

# cargo-ndk also emits plev's own cdylib (libplev.so); it is dead weight —
# libshowcase.so statically links plev and GameActivity loads only "showcase".
find "$JNILIBS" -name 'libplev.so' -delete

echo ">> jniLibs:"; find "$JNILIBS" -name 'libshowcase.so' -exec ls -la {} \;

echo ">> assembling debug APK"
./gradlew assembleDebug --no-daemon

echo ">> APK: $(pwd)/app/build/outputs/apk/debug/app-debug.apk"
