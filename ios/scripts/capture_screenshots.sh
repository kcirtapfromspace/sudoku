#!/bin/bash

# =============================================================================
# App Store Screenshot Capture Script
# =============================================================================
# This script automates capturing screenshots for App Store submission
#
# Requirements:
#   - Xcode 15.0+
#   - XcodeGen installed (brew install xcodegen)
#   - iOS Simulator
#
# Usage:
#   ./scripts/capture_screenshots.sh
#
# Screenshots will be saved to: ./screenshots/
# =============================================================================

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCREENSHOTS_DIR="${PROJECT_DIR}/screenshots"
SCHEME="Sudoku"
UI_TEST_SCHEME="SudokuScreenshots"

# Device configurations for App Store
# iPhone 6.5" (required): iPhone 14 Pro Max or iPhone 15 Pro Max
# iPhone 6.7" (new): iPhone 15 Pro Max
# iPad Pro 12.9" (required)
IPHONE_DEVICE="iPhone 17 Pro Max"
IPAD_DEVICE="iPad Pro 13-inch (M5)"

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  App Store Screenshot Capture${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Create screenshots directory
mkdir -p "${SCREENSHOTS_DIR}/iPhone"
mkdir -p "${SCREENSHOTS_DIR}/iPad"

cd "${PROJECT_DIR}/Sudoku"

# Step 1: Regenerate Xcode project
echo -e "${YELLOW}Step 1: Regenerating Xcode project...${NC}"
if command -v xcodegen &> /dev/null; then
    xcodegen generate
    echo -e "${GREEN}Project regenerated.${NC}"
else
    echo -e "${RED}XcodeGen not found. Please install: brew install xcodegen${NC}"
    exit 1
fi

# Step 2: Build the app
echo ""
echo -e "${YELLOW}Step 2: Building app for simulator...${NC}"
xcodebuild clean build \
    -scheme "${SCHEME}" \
    -destination "platform=iOS Simulator,name=${IPHONE_DEVICE}" \
    -configuration Debug \
    -quiet \
    2>&1 | grep -E "(error:|warning:)" || true

echo -e "${GREEN}Build complete.${NC}"

# Step 3: Boot simulator
echo ""
echo -e "${YELLOW}Step 3: Booting iPhone simulator...${NC}"

# Kill existing simulators
killall "Simulator" 2>/dev/null || true
sleep 2

# Boot the device
xcrun simctl boot "${IPHONE_DEVICE}" 2>/dev/null || true
open -a Simulator
sleep 5

# Set appearance for first set (Light Mode)
echo -e "${YELLOW}Setting Light mode...${NC}"
xcrun simctl ui booted appearance light

# Step 4: Install and launch the app
echo ""
echo -e "${YELLOW}Step 4: Installing and launching app...${NC}"

# Find the app bundle
APP_PATH=$(find ~/Library/Developer/Xcode/DerivedData -name "Sudoku.app" -path "*/Debug-iphonesimulator/*" 2>/dev/null | head -1)

if [ -z "$APP_PATH" ]; then
    echo -e "${RED}Could not find Sudoku.app. Make sure build succeeded.${NC}"
    exit 1
fi

xcrun simctl install booted "${APP_PATH}"
xcrun simctl launch booted com.ukodus.app
sleep 3

# Step 5: Capture screenshots manually (automated test had issues)
echo ""
echo -e "${YELLOW}Step 5: Capturing screenshots...${NC}"

capture_screenshot() {
    local name=$1
    local mode=$2
    xcrun simctl io booted screenshot "${SCREENSHOTS_DIR}/${mode}/${name}.png"
    echo -e "  ${GREEN}Captured: ${name}${NC}"
}

# Function to wait for user to set up screen
wait_for_screen() {
    local screen_name=$1
    echo ""
    echo -e "${BLUE}>>> Navigate to: ${screen_name}${NC}"
    echo -e "${YELLOW}Press Enter when ready to capture...${NC}"
    read -r
}

echo ""
echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  SEMI-AUTOMATED CAPTURE MODE${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""
echo "The app is now running in the simulator."
echo "Follow the prompts to capture each screenshot."
echo ""

# iPhone Light Mode Screenshots
echo -e "${YELLOW}--- iPhone (Light Mode) ---${NC}"

wait_for_screen "Main Menu"
capture_screenshot "01_MainMenu" "iPhone"

wait_for_screen "Difficulty Selection (tap New Game)"
capture_screenshot "02_DifficultyPicker" "iPhone"

wait_for_screen "Gameplay (select a cell to show highlighting)"
capture_screenshot "03_Gameplay_Highlighted" "iPhone"

wait_for_screen "Notes Mode (toggle to Notes mode)"
capture_screenshot "04_NotesMode" "iPhone"

wait_for_screen "Settings (open from menu)"
capture_screenshot "05_Settings" "iPhone"

wait_for_screen "Statistics (open from menu)"
capture_screenshot "06_Statistics" "iPhone"

# Switch to Dark Mode
echo ""
echo -e "${YELLOW}Switching to Dark Mode...${NC}"
xcrun simctl ui booted appearance dark
sleep 2

wait_for_screen "Gameplay (Dark Mode)"
capture_screenshot "07_Gameplay_Dark" "iPhone"

# Win Screen (requires completing a puzzle or using debug menu)
echo ""
echo -e "${BLUE}--- OPTIONAL: Win Screen ---${NC}"
echo "To capture the Win Screen:"
echo "1. Long-press the grid for 2 seconds to open Debug Menu"
echo "2. Select 'Fill All (leave 1 cell) - Win Test'"
echo "3. Enter the last number to trigger the Win Screen"
echo ""
wait_for_screen "Win Screen (with confetti - OPTIONAL, press Enter to skip)"
if [ -f "${SCREENSHOTS_DIR}/iPhone/temp_check" ]; then
    capture_screenshot "08_WinScreen" "iPhone"
fi
xcrun simctl io booted screenshot "${SCREENSHOTS_DIR}/iPhone/08_WinScreen.png" 2>/dev/null || echo "  Skipped Win Screen"

# Celebration Toast (requires timing)
echo ""
echo -e "${BLUE}--- OPTIONAL: Celebration Toast ---${NC}"
echo "To capture the Celebration Toast:"
echo "1. Use Debug Menu to 'Fill Row 1 (except 1 cell)'"
echo "2. Complete the row"
echo "3. Quickly press Enter to capture the toast"
echo ""
wait_for_screen "Celebration Toast (OPTIONAL, press Enter to skip)"
xcrun simctl io booted screenshot "${SCREENSHOTS_DIR}/iPhone/09_Celebration.png" 2>/dev/null || echo "  Skipped Celebration"

echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  Screenshot Capture Complete!${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo "Screenshots saved to: ${SCREENSHOTS_DIR}"
echo ""
echo "Files captured:"
ls -la "${SCREENSHOTS_DIR}/iPhone/" 2>/dev/null || echo "No iPhone screenshots"
echo ""

# Provide resize instructions
echo -e "${BLUE}--- Post-Processing ---${NC}"
echo ""
echo "App Store requires these exact sizes:"
echo "  iPhone 6.5\":  1242 x 2688 or 1284 x 2778"
echo "  iPhone 6.7\":  1290 x 2796"
echo "  iPad 12.9\":   2048 x 2732"
echo ""
echo "Your iPhone 15 Pro Max screenshots are 1290 x 2796."
echo "For 6.5\" display, resize with:"
echo ""
echo "  sips -z 2778 1284 screenshots/iPhone/*.png"
echo ""
echo "Or use the exact dimensions from the simulator."
echo ""
