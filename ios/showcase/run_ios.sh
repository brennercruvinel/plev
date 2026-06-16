#!/bin/bash
# Boot a simulator, install the built ShowcaseDemo.app, launch it and grab a
# screenshot. Run build_ios.sh first.
#
#   ./run_ios.sh                 # iPhone 17, Debug
#   DEVICE="iPhone 17 Pro" ./run_ios.sh
set -euo pipefail

export DEVELOPER_DIR="${DEVELOPER_DIR:-/Applications/Xcode.app/Contents/Developer}"
cd "$(dirname "$0")"

XCODE_CONFIG="${XCODE_CONFIG:-Debug}"
APP="build/Build/Products/$XCODE_CONFIG-iphonesimulator/ShowcaseDemo.app"
BUNDLE_ID="com.plev.showcase"
DEVICE="${DEVICE:-iPhone 17}"

echo ">> booting $DEVICE"
xcrun simctl boot "$DEVICE" 2>/dev/null || true
xcrun simctl bootstatus "$DEVICE" || true

echo ">> installing $APP"
xcrun simctl install "$DEVICE" "$APP"

echo ">> launching $BUNDLE_ID"
xcrun simctl launch "$DEVICE" "$BUNDLE_ID" || true

sleep 5
xcrun simctl io "$DEVICE" screenshot ios_showcase.png
echo ">> screenshot: $(pwd)/ios_showcase.png"
