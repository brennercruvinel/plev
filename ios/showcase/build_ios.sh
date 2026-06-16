#!/bin/bash
# Build the plev showcase iOS demo for the simulator.
#
#   ./build_ios.sh            # debug, aarch64-apple-ios-sim
#   PROFILE=release ./build_ios.sh
#
# Produces ShowcaseDemo.app under build/Build/Products/Debug-iphonesimulator.
# Run it with run_ios.sh. Requires Xcode (selected via DEVELOPER_DIR), the
# aarch64-apple-ios-sim Rust target, and xcodegen.
set -euo pipefail

export DEVELOPER_DIR="${DEVELOPER_DIR:-/Applications/Xcode.app/Contents/Developer}"
cd "$(dirname "$0")"
REPO_ROOT="$(cd ../.. && pwd)"

TRIPLE="aarch64-apple-ios-sim"
PROFILE="${PROFILE:-debug}"
CARGO_PROFILE_FLAG=""
XCODE_CONFIG="Debug"
if [ "$PROFILE" = "release" ]; then
  CARGO_PROFILE_FLAG="--release"
  XCODE_CONFIG="Release"
fi

echo ">> building Rust staticlib ($TRIPLE, $PROFILE)"
( cd "$REPO_ROOT" && cargo build -p showcase --lib --target "$TRIPLE" $CARGO_PROFILE_FLAG )

LIBDIR="$REPO_ROOT/target/$TRIPLE/$PROFILE"
echo ">> staticlib: $LIBDIR/libshowcase.a"

echo ">> generating Xcode project"
xcodegen generate

echo ">> building app ($XCODE_CONFIG, iphonesimulator)"
xcodebuild \
  -project ShowcaseDemo.xcodeproj \
  -scheme ShowcaseDemo \
  -configuration "$XCODE_CONFIG" \
  -sdk iphonesimulator \
  -destination 'generic/platform=iOS Simulator' \
  -derivedDataPath build \
  ARCHS=arm64 \
  EXCLUDED_ARCHS=x86_64 \
  ONLY_ACTIVE_ARCH=YES \
  OTHER_LDFLAGS="$LIBDIR/libshowcase.a -lc++" \
  CODE_SIGNING_ALLOWED=NO \
  build

echo ">> app: $(pwd)/build/Build/Products/$XCODE_CONFIG-iphonesimulator/ShowcaseDemo.app"
