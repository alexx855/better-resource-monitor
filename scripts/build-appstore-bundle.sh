#!/bin/bash
set -euo pipefail

# Builds the app bundle from an arbitrary source checkout without Apple signing
# credentials present in the environment.

SOURCE_DIR="${1:?source directory is required}"
BUILD_NUMBER_INPUT="${2:?build number is required}"
BRM_BUILD_COMMIT_INPUT="${3:?build commit is required}"
APPSTORE_CONFIG="$(cd "$(dirname "${BASH_SOURCE[0]}")/../src-tauri" && pwd)/tauri.appstore.conf.json"

source "$(dirname "${BASH_SOURCE[0]}")/validate-appstore-metadata.sh"
validate_appstore_metadata "$BRM_BUILD_COMMIT_INPUT" "$BUILD_NUMBER_INPUT"
BRM_BUILD_COMMIT="${BRM_BUILD_COMMIT_INPUT:0:12}"

if [ ! -d "$SOURCE_DIR" ]; then
  echo "Error: source directory not found: $SOURCE_DIR"
  exit 1
fi

cd "$SOURCE_DIR"

VERSION="$(jq -r '.version' src-tauri/tauri.conf.json)"
echo "=== App Store Bundle Build ==="
echo "Version: $VERSION"
echo "Build: $BUILD_NUMBER_INPUT"
echo "Commit: $BRM_BUILD_COMMIT"
echo ""

echo "Using --bundles app to create only .app bundle"
if grep -R "IO[R]eport" src-tauri/src src-tauri/Cargo.toml .github scripts; then
  echo "Error: App Store source, workflows, or scripts must not reference private GPU sampling APIs"
  exit 1
fi
echo "No private GPU sampling source references found"

export BRM_BUILD_COMMIT
pnpm tauri build \
  --bundles app \
  --target universal-apple-darwin \
  --config "$APPSTORE_CONFIG" \
  --features app-store

APP_PATH="src-tauri/target/universal-apple-darwin/release/bundle/macos/Better Resource Monitor.app"
APP_EXECUTABLE="$APP_PATH/Contents/MacOS/better-resource-monitor"
if [ ! -d "$APP_PATH" ]; then
  echo "Error: App bundle not found at $APP_PATH"
  exit 1
fi

if ! strings -a "$APP_EXECUTABLE" | awk -v needle="$BRM_BUILD_COMMIT" 'index($0, needle) { found = 1 } END { exit found ? 0 : 1 }'; then
  echo "Error: App executable does not contain expected build commit $BRM_BUILD_COMMIT"
  exit 1
fi
echo "Build commit embedded successfully"

if strings -a "$APP_EXECUTABLE" | grep "IO[R]eport"; then
  echo "Error: App executable contains private GPU sampling references"
  exit 1
fi
echo "No private GPU sampling executable references found"
