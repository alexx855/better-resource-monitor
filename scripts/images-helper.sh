#!/bin/bash

# App Store Screenshot Helper for Better Resource Monitor
# Generates tray banners and localized marketing screenshots for all supported languages.
# Usage: pnpm build:screenshots

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
DIST_DIR="$PROJECT_DIR/www/dist/client/images"
OUTPUT_DIR="$PROJECT_DIR/images/appstore"

LANGS=("en" "es" "pt-BR" "zh-Hans")
SCREENSHOTS=("simplicity" "performance" "privacy")

echo "==========================================="
echo "   App Store Screenshot Generator"
echo "==========================================="
echo ""
echo "Languages: ${LANGS[*]}"
echo "Screenshots: ${SCREENSHOTS[*]}"
echo "Output: $OUTPUT_DIR"
echo ""

cd "$PROJECT_DIR"

echo "--> Rendering tray banners..."
cargo run --manifest-path src-tauri/Cargo.toml --example render_tray_icon -- \
  --out www/public/better-resource-monitor.png \
  --preset macos \
  --scale 0.6666667 \
  --include-alert-row true
cargo run --manifest-path src-tauri/Cargo.toml --example render_tray_icon -- \
  --out www/public/better-resource-monitor-alert.png \
  --preset macos \
  --scale 0.6666667 \
  --cpu 93 \
  --mem 96 \
  --storage 92 \
  --gpu 91 \
  --down "12 MB" \
  --up "3.1 MB"

echo ""
echo "--> Building website (generates all screenshots)..."
pnpm --filter www build

if [ ! -d "$DIST_DIR" ]; then
  echo "ERROR: Build output not found at $DIST_DIR"
  exit 1
fi

echo ""
echo "--> Organizing screenshots by language..."

for lang in "${LANGS[@]}"; do
  mkdir -p "$OUTPUT_DIR/$lang"
  for screenshot in "${SCREENSHOTS[@]}"; do
    src="$DIST_DIR/${screenshot}-${lang}.png"
    dst="$OUTPUT_DIR/$lang/${screenshot}.png"
    if [ ! -f "$src" ]; then
      echo "ERROR: Missing $src"
      exit 1
    fi
    cp "$src" "$dst"
    echo "  $dst"
  done
done

TOTAL=$(( ${#LANGS[@]} * ${#SCREENSHOTS[@]} ))

echo ""
echo "==========================================="
echo "Done! $TOTAL screenshots saved to: $OUTPUT_DIR"
echo ""
echo "Upload these to App Store Connect per language."
echo "==========================================="
