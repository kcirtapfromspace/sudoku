#!/bin/bash

# =============================================================================
# Quick Screenshot Capture (Fully Automated)
# =============================================================================
# Takes basic screenshots without UI automation
# For full App Store set, use capture_screenshots.sh
# =============================================================================

set -e

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCREENSHOTS_DIR="${PROJECT_DIR}/screenshots"
DEVICE="iPhone 17 Pro Max"

mkdir -p "${SCREENSHOTS_DIR}"

cd "${PROJECT_DIR}/Sudoku"

echo "Regenerating project..."
xcodegen generate 2>/dev/null || echo "XcodeGen not found, using existing project"

echo "Building..."
xcodebuild build \
    -scheme Sudoku \
    -destination "platform=iOS Simulator,name=${DEVICE}" \
    -configuration Debug \
    -quiet 2>&1 | grep -E "error:" || true

echo "Starting simulator..."
killall "Simulator" 2>/dev/null || true
sleep 2
xcrun simctl boot "${DEVICE}" 2>/dev/null || true
open -a Simulator
sleep 4

echo "Installing app..."
APP_PATH=$(find ~/Library/Developer/Xcode/DerivedData -name "Sudoku.app" -path "*/Debug-iphonesimulator/*" 2>/dev/null | head -1)
if [ -n "$APP_PATH" ]; then
    xcrun simctl install booted "${APP_PATH}"

    # Light mode
    xcrun simctl ui booted appearance light
    xcrun simctl launch booted com.ukodus.app
    sleep 3
    xcrun simctl io booted screenshot "${SCREENSHOTS_DIR}/01_MainMenu_Light.png"
    echo "Captured: Main Menu (Light)"

    # Terminate and relaunch for dark mode
    xcrun simctl terminate booted com.ukodus.app
    xcrun simctl ui booted appearance dark
    sleep 1
    xcrun simctl launch booted com.ukodus.app
    sleep 3
    xcrun simctl io booted screenshot "${SCREENSHOTS_DIR}/02_MainMenu_Dark.png"
    echo "Captured: Main Menu (Dark)"

    echo ""
    echo "Basic screenshots saved to: ${SCREENSHOTS_DIR}"
    echo ""
    echo "For gameplay screenshots, open the app manually and use:"
    echo "  xcrun simctl io booted screenshot filename.png"
    echo ""
    echo "Or press Cmd+S in the Simulator window."
else
    echo "Error: Could not find built app"
    exit 1
fi
