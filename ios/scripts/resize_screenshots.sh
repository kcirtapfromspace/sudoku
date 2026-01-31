#!/bin/bash

# =============================================================================
# Resize Screenshots for App Store
# =============================================================================
# Resizes screenshots to exact App Store dimensions
# =============================================================================

INPUT_DIR="${1:-/Users/thinkstudio/tui/sudoku-workspace/ios/screenshots}"
OUTPUT_DIR="${INPUT_DIR}/resized"

mkdir -p "$OUTPUT_DIR"

echo "Resizing screenshots for App Store..."
echo "Input:  $INPUT_DIR"
echo "Output: $OUTPUT_DIR"
echo ""

# App Store dimensions:
# iPhone 6.5": 1284 x 2778 (or 1242 x 2688)
# iPhone 6.7": 1290 x 2796
# iPad 12.9": 2048 x 2732

for file in "$INPUT_DIR"/*.png; do
    if [ -f "$file" ]; then
        filename=$(basename "$file")

        # Resize to 6.7" (1290 x 2796) - maintains aspect ratio better
        sips -z 2796 1290 "$file" --out "$OUTPUT_DIR/${filename%.png}_6.7.png" 2>/dev/null

        # Resize to 6.5" (1284 x 2778)
        sips -z 2778 1284 "$file" --out "$OUTPUT_DIR/${filename%.png}_6.5.png" 2>/dev/null

        echo "Resized: $filename"
    fi
done

echo ""
echo "Done! Resized screenshots in: $OUTPUT_DIR"
ls -la "$OUTPUT_DIR"
