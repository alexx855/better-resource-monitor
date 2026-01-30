#!/bin/bash

# App Store Screenshot Helper for Better Resource Monitor
# Usage: ./scripts/screenshot-helper.sh [mode] [size]
#   Modes: full, menu, all (default: full)
#   Sizes: 1280, 1440, 2560, 2880 (default: auto-detect best)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
OUTPUT_DIR="$PROJECT_DIR/screenshots/appstore"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# App Store valid dimensions (16:10 aspect ratio)
declare -A VALID_SIZES=(
    [1280]="800"
    [1440]="900"
    [2560]="1600"
    [2880]="1800"
)

show_help() {
    echo "==========================================="
    echo "   App Store Screenshot Helper"
    echo "==========================================="
    echo ""
    echo "Usage: $0 [mode] [target_width]"
    echo ""
    echo "Modes:"
    echo "  zoom     - Zoomed menu bar area (best for showcasing the app)"
    echo "  menu     - Zoomed with context menu open (10s delay)"
    echo "  full     - Full screen capture"
    echo "  all      - Generate all 4 App Store sizes from one capture"
    echo ""
    echo "Target widths (optional):"
    echo "  1280     - 1280x800 (standard)"
    echo "  1440     - 1440x900 (standard)"
    echo "  2560     - 2560x1600 (Retina)"
    echo "  2880     - 2880x1800 (Retina)"
    echo ""
    echo "If no target width specified, auto-selects best match."
    echo ""
    echo "Recommended workflow:"
    echo "  1. Run the app so it shows in your menu bar"
    echo "  2. ./screenshot-helper.sh zoom    # Clean menu bar shot"
    echo "  3. ./screenshot-helper.sh menu    # With context menu open"
    echo ""
    echo "Screenshots are saved to: screenshots/appstore/"
    echo ""
}

prepare_desktop() {
    echo "--> Hiding desktop icons..."
    defaults write com.apple.finder CreateDesktop false
    killall Finder

    echo "--> Setting clean wallpaper..."
    osascript -e 'tell application "System Events" to set picture of every desktop to "/System/Library/Desktop Pictures/Solid Colors/Space Gray Pro.png"'
}

restore_desktop() {
    echo "--> Restoring desktop icons..."
    defaults delete com.apple.finder CreateDesktop 2>/dev/null || true
    killall Finder
}

countdown() {
    local seconds=$1
    local message=$2
    echo ""
    echo "--> $message"
    for i in $(seq "$seconds" -1 1); do
        echo "$i..."
        sleep 1
    done
}

# Resize screenshot to exact App Store dimensions
resize_to_appstore() {
    local input_file=$1
    local target_width=$2
    local target_height=${VALID_SIZES[$target_width]}

    if [ -z "$target_height" ]; then
        echo "ERROR: Invalid target width. Use 1280, 1440, 2560, or 2880."
        return 1
    fi

    local basename=$(basename "$input_file" .png)
    local output_file="$OUTPUT_DIR/${basename}_${target_width}x${target_height}.png"

    # Get current dimensions
    local src_width=$(sips -g pixelWidth "$input_file" | tail -n1 | awk '{print $2}')
    local src_height=$(sips -g pixelHeight "$input_file" | tail -n1 | awk '{print $2}')

    echo "--> Resizing to ${target_width}x${target_height}..."

    # Calculate crop/scale to maintain 16:10 aspect ratio
    # Target aspect = 16:10 = 1.6
    local src_aspect=$(echo "scale=4; $src_width / $src_height" | bc)
    local target_aspect="1.6"

    # Create a temporary file for processing
    local temp_file="${input_file%.png}_temp.png"
    cp "$input_file" "$temp_file"

    # If source is wider than 16:10, crop sides
    # If source is taller than 16:10, crop top/bottom
    local crop_width=$src_width
    local crop_height=$src_height

    if (( $(echo "$src_aspect > $target_aspect" | bc -l) )); then
        # Source is wider - crop width
        crop_width=$(echo "scale=0; $src_height * $target_aspect / 1" | bc)
    else
        # Source is taller - crop height
        crop_height=$(echo "scale=0; $src_width / $target_aspect / 1" | bc)
    fi

    # Crop to 16:10 aspect ratio (centered)
    local crop_x=$(( (src_width - crop_width) / 2 ))
    local crop_y=$(( (src_height - crop_height) / 2 ))

    if [ "$crop_width" != "$src_width" ] || [ "$crop_height" != "$src_height" ]; then
        sips --cropToHeightWidth "$crop_height" "$crop_width" "$temp_file" --out "$temp_file" > /dev/null 2>&1
    fi

    # Resize to target dimensions
    sips --resampleHeightWidth "$target_height" "$target_width" "$temp_file" --out "$output_file" > /dev/null 2>&1

    # Cleanup
    rm -f "$temp_file"

    # Verify output
    local out_width=$(sips -g pixelWidth "$output_file" | tail -n1 | awk '{print $2}')
    local out_height=$(sips -g pixelHeight "$output_file" | tail -n1 | awk '{print $2}')

    if [ "$out_width" = "$target_width" ] && [ "$out_height" = "$target_height" ]; then
        echo "    Created: $output_file (${out_width}x${out_height})"
        return 0
    else
        echo "    WARNING: Output is ${out_width}x${out_height}, expected ${target_width}x${target_height}"
        return 1
    fi
}

# Auto-select best target size based on source resolution
select_best_size() {
    local src_width=$1

    # Choose the largest size that doesn't upscale
    if [ "$src_width" -ge 2880 ]; then
        echo "2880"
    elif [ "$src_width" -ge 2560 ]; then
        echo "2560"
    elif [ "$src_width" -ge 1440 ]; then
        echo "1440"
    else
        echo "1280"
    fi
}

# Zoom into menu bar area and scale to App Store size
capture_zoom() {
    local target_width=${1:-2880}
    local target_height=${VALID_SIZES[$target_width]}
    local raw_file="$OUTPUT_DIR/raw_zoom_${TIMESTAMP}.png"
    local output_file="$OUTPUT_DIR/zoom_${TIMESTAMP}_${target_width}x${target_height}.png"

    echo ""
    echo "=== Zoomed Menu Bar Capture ==="
    echo "This captures the top-right menu bar area and enlarges it."
    countdown 5 "GET READY! Ensure your app icon is visible in the menu bar."

    echo "--> Capturing full screen..."
    screencapture -m "$raw_file"

    local src_width=$(sips -g pixelWidth "$raw_file" | tail -n1 | awk '{print $2}')
    local src_height=$(sips -g pixelHeight "$raw_file" | tail -n1 | awk '{print $2}')
    echo "    Raw capture: ${src_width}x${src_height}"

    # Calculate crop region: top-right area with 16:10 aspect ratio
    # We want to show roughly 1/3 of screen width (right side where menu extras are)
    # and top portion including menu bar
    local crop_width=$(( src_width / 2 ))
    local crop_height=$(( crop_width * 10 / 16 ))  # 16:10 aspect ratio

    # Ensure we don't crop more than screen height
    if [ "$crop_height" -gt "$src_height" ]; then
        crop_height=$src_height
        crop_width=$(( crop_height * 16 / 10 ))
    fi

    echo "--> Cropping to top-right region (${crop_width}x${crop_height})..."

    # Crop from top-right corner
    local crop_x=$(( src_width - crop_width ))
    local crop_y=0

    # Create cropped version
    local temp_file="${raw_file%.png}_crop.png"
    sips --cropToHeightWidth "$crop_height" "$crop_width" --cropOffset "$crop_y" "$crop_x" "$raw_file" --out "$temp_file" > /dev/null 2>&1 || {
        # Fallback: crop centered then move
        cp "$raw_file" "$temp_file"
        sips --cropToHeightWidth "$crop_height" "$crop_width" "$temp_file" --out "$temp_file" > /dev/null 2>&1
    }

    echo "--> Scaling to ${target_width}x${target_height}..."
    sips --resampleHeightWidth "$target_height" "$target_width" "$temp_file" --out "$output_file" > /dev/null 2>&1

    rm -f "$temp_file"

    # Verify
    local out_width=$(sips -g pixelWidth "$output_file" | tail -n1 | awk '{print $2}')
    local out_height=$(sips -g pixelHeight "$output_file" | tail -n1 | awk '{print $2}')

    echo ""
    echo "    Created: $output_file"
    echo "    Size: ${out_width}x${out_height}"
    echo "    Raw file kept: $raw_file"
}

capture_full() {
    local target_width=$1
    local raw_file="$OUTPUT_DIR/raw_full_${TIMESTAMP}.png"

    echo ""
    echo "=== Full Screen Capture ==="
    countdown 5 "GET READY! Position your menu bar as desired."

    echo "--> Capturing full screen..."
    screencapture -m "$raw_file"

    local src_width=$(sips -g pixelWidth "$raw_file" | tail -n1 | awk '{print $2}')
    local src_height=$(sips -g pixelHeight "$raw_file" | tail -n1 | awk '{print $2}')
    echo "    Raw capture: ${src_width}x${src_height}"

    if [ -z "$target_width" ]; then
        target_width=$(select_best_size "$src_width")
        echo "    Auto-selected target: ${target_width}x${VALID_SIZES[$target_width]}"
    fi

    resize_to_appstore "$raw_file" "$target_width"

    # Keep raw file for reference
    echo "    Raw file kept: $raw_file"
}

capture_menu() {
    local target_width=${1:-2880}
    local target_height=${VALID_SIZES[$target_width]}
    local raw_file="$OUTPUT_DIR/raw_menu_${TIMESTAMP}.png"
    local output_file="$OUTPUT_DIR/menu_${TIMESTAMP}_${target_width}x${target_height}.png"

    echo ""
    echo "=== Zoomed Context Menu Capture ==="
    echo "This captures with context menu open and zooms the menu bar area."
    echo ""
    echo "Instructions:"
    echo "  1. Right-click on your app's menu bar icon"
    echo "  2. Keep the context menu open until capture completes"
    countdown 10 "RIGHT-CLICK your app icon NOW and keep the menu open!"

    echo "--> Capturing full screen..."
    screencapture -m "$raw_file"

    local src_width=$(sips -g pixelWidth "$raw_file" | tail -n1 | awk '{print $2}')
    local src_height=$(sips -g pixelHeight "$raw_file" | tail -n1 | awk '{print $2}')
    echo "    Raw capture: ${src_width}x${src_height}"

    # For menu capture, crop a taller region to include the dropdown menu
    # Use right half of screen, with more vertical space
    local crop_width=$(( src_width / 2 ))
    local crop_height=$(( crop_width * 10 / 16 ))

    if [ "$crop_height" -gt "$src_height" ]; then
        crop_height=$src_height
        crop_width=$(( crop_height * 16 / 10 ))
    fi

    echo "--> Cropping to top-right region (${crop_width}x${crop_height})..."

    local temp_file="${raw_file%.png}_crop.png"
    sips --cropToHeightWidth "$crop_height" "$crop_width" "$raw_file" --out "$temp_file" > /dev/null 2>&1 || {
        cp "$raw_file" "$temp_file"
        sips --cropToHeightWidth "$crop_height" "$crop_width" "$temp_file" --out "$temp_file" > /dev/null 2>&1
    }

    echo "--> Scaling to ${target_width}x${target_height}..."
    sips --resampleHeightWidth "$target_height" "$target_width" "$temp_file" --out "$output_file" > /dev/null 2>&1

    rm -f "$temp_file"

    local out_width=$(sips -g pixelWidth "$output_file" | tail -n1 | awk '{print $2}')
    local out_height=$(sips -g pixelHeight "$output_file" | tail -n1 | awk '{print $2}')

    echo ""
    echo "    Created: $output_file"
    echo "    Size: ${out_width}x${out_height}"
    echo "    Raw file kept: $raw_file"
}

capture_all_sizes() {
    local raw_file="$OUTPUT_DIR/raw_all_${TIMESTAMP}.png"

    echo ""
    echo "=== Capture for All App Store Sizes ==="
    echo ""
    echo "This will take ONE screenshot and generate ALL 4 sizes:"
    echo "  - 1280x800"
    echo "  - 1440x900"
    echo "  - 2560x1600"
    echo "  - 2880x1800"
    echo ""
    countdown 5 "GET READY! Position your menu bar as desired."

    echo "--> Capturing full screen..."
    screencapture -m "$raw_file"

    local src_width=$(sips -g pixelWidth "$raw_file" | tail -n1 | awk '{print $2}')
    local src_height=$(sips -g pixelHeight "$raw_file" | tail -n1 | awk '{print $2}')
    echo "    Raw capture: ${src_width}x${src_height}"
    echo ""

    echo "--> Generating all App Store sizes..."
    for width in 2880 2560 1440 1280; do
        resize_to_appstore "$raw_file" "$width"
    done

    echo ""
    echo "    Raw file kept: $raw_file"
}

# Main execution
MODE="${1:-zoom}"
TARGET_WIDTH="${2:-}"

if [ "$MODE" = "-h" ] || [ "$MODE" = "--help" ]; then
    show_help
    exit 0
fi

# Validate target width if provided
if [ -n "$TARGET_WIDTH" ]; then
    if [ -z "${VALID_SIZES[$TARGET_WIDTH]}" ]; then
        echo "ERROR: Invalid target width '$TARGET_WIDTH'"
        echo "Valid widths: 1280, 1440, 2560, 2880"
        exit 1
    fi
fi

# Create output directory
mkdir -p "$OUTPUT_DIR"

echo "==========================================="
echo "   App Store Screenshot Helper"
echo "==========================================="
echo ""
echo "Mode: $MODE"
if [ -n "$TARGET_WIDTH" ]; then
    echo "Target: ${TARGET_WIDTH}x${VALID_SIZES[$TARGET_WIDTH]}"
else
    echo "Target: auto-detect"
fi
echo "Output: $OUTPUT_DIR"
echo ""
echo "NOTE: Your original wallpaper will NOT be automatically restored."
echo "      You will need to change it back manually in System Settings."
echo ""

read -p "Press [Enter] to start or Ctrl+C to cancel..."

# Prepare desktop
prepare_desktop

# Trap to restore desktop on exit
trap restore_desktop EXIT

# Execute based on mode
case "$MODE" in
    zoom)
        capture_zoom "$TARGET_WIDTH"
        ;;
    menu)
        capture_menu "$TARGET_WIDTH"
        ;;
    full)
        capture_full "$TARGET_WIDTH"
        ;;
    all)
        capture_all_sizes
        ;;
    *)
        echo "Unknown mode: $MODE"
        show_help
        exit 1
        ;;
esac

echo ""
echo "==========================================="
echo "Done! Screenshots saved to: $OUTPUT_DIR"
echo ""
echo "Valid App Store sizes created. Upload these to App Store Connect."
echo "Don't forget to restore your wallpaper manually."
