#!/bin/bash

# App Store Screenshot Helper for Better Resource Monitor
# Generates localized marketing screenshots for all supported languages.
# Usage: ./scripts/screenshot-helper.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
DIST_DIR="$PROJECT_DIR/www/dist/images"
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
