#!/bin/bash
set -e

# Builds, signs, packages, and uploads a universal macOS binary to App Store Connect.
# Build number can be set with BUILD_NUMBER, otherwise it auto-increments from
# scripts/.build-number while never falling below a timestamp-scale value.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
ENV_FILE="$SCRIPT_DIR/.env"

cd "$PROJECT_ROOT"

VERSION=$(jq -r '.version' "$PROJECT_ROOT/src-tauri/tauri.conf.json")
BRM_BUILD_COMMIT="${BRM_BUILD_COMMIT:-$(git -C "$PROJECT_ROOT" rev-parse --short=12 HEAD 2>/dev/null || echo unknown)}"
export BRM_BUILD_COMMIT

if [ ! -f "$ENV_FILE" ]; then
  echo "Error: .env file not found at $ENV_FILE"
  echo "Create scripts/.env with required variables (see scripts/.env.example)"
  exit 1
fi

set -a
source "$ENV_FILE"
set +a

required_vars=("APPLE_TEAM_ID" "APPLE_DISTRIBUTION_IDENTITY" "APPLE_INSTALLER_IDENTITY" "APPLE_API_KEY_ID" "APPLE_API_ISSUER")
for var in "${required_vars[@]}"; do
  if [ -z "${!var}" ]; then
    echo "Error: $var is not set in $ENV_FILE"
    exit 1
  fi
done

# Auto-increment build number. The timestamp floor avoids accidentally creating
# a build lower than an already-distributed TestFlight/App Store build when the
# ignored local counter file is missing or stale.
BUILD_NUMBER_FILE="$SCRIPT_DIR/.build-number"
if [ -n "${BUILD_NUMBER:-}" ]; then
  case "$BUILD_NUMBER" in
    ''|*[!0-9]*)
      echo "Error: BUILD_NUMBER must be numeric, got '$BUILD_NUMBER'"
      exit 1
      ;;
  esac
else
  CURRENT_BUILD_NUMBER=$(cat "$BUILD_NUMBER_FILE" 2>/dev/null || echo 0)
  case "$CURRENT_BUILD_NUMBER" in
    ''|*[!0-9]*)
      CURRENT_BUILD_NUMBER=0
      ;;
  esac
  TIMESTAMP_BUILD_NUMBER=$(date -u +%Y%m%d%H%M)
  BUILD_NUMBER=$((CURRENT_BUILD_NUMBER + 1))
  if [ "$BUILD_NUMBER" -lt "$TIMESTAMP_BUILD_NUMBER" ]; then
    BUILD_NUMBER="$TIMESTAMP_BUILD_NUMBER"
  fi
fi
echo "$BUILD_NUMBER" > "$BUILD_NUMBER_FILE"

echo "=== App Store Packaging Script ==="
echo "Version: $VERSION"
echo "Build: $BUILD_NUMBER"
echo "Commit: $BRM_BUILD_COMMIT"
echo ""

APP_NAME="Better Resource Monitor"
ENTITLEMENTS_PATH="src-tauri/Entitlements.appstore.plist"

TARGET_DIR="src-tauri/target/universal-apple-darwin/release/bundle/macos"
APP_PATH="${TARGET_DIR}/${APP_NAME}.app"
PKG_PATH="${TARGET_DIR}/${APP_NAME}.pkg"

PROFILE_PATH="src-tauri/embedded.provisionprofile"
if [ ! -f "$PROFILE_PATH" ]; then
  echo "Error: Provisioning profile not found at $PROFILE_PATH"
  echo ""
  echo "Download your Mac App Store provisioning profile from:"
  echo "https://developer.apple.com/account/resources/profiles/list"
  echo ""
  echo "Save it as: src-tauri/embedded.provisionprofile"
  exit 1
fi

echo "Building universal binary for App Store..."
echo "Using --bundles app to create only .app bundle"
pnpm tauri build --bundles app --target universal-apple-darwin --config src-tauri/tauri.appstore.conf.json --features app-store

if [ ! -d "$APP_PATH" ]; then
  echo "Error: App bundle not found at $APP_PATH"
  exit 1
fi

echo "App bundle found at: $APP_PATH"

if [ ! -f "$APP_PATH/Contents/embedded.provisionprofile" ]; then
  echo "Error: Provisioning profile was not embedded by Tauri"
  echo "Check that 'files' config in tauri.appstore.conf.json is correct"
  exit 1
fi
echo "Provisioning profile embedded successfully"

AUTOSTART_AGENT_PATH="$APP_PATH/Contents/Library/LaunchAgents/dev.alexpedersen.better-resource-monitor.autostart.plist"
if [ ! -f "$AUTOSTART_AGENT_PATH" ]; then
  echo "Error: Autostart LaunchAgent was not embedded by Tauri"
  echo "Check that 'files' config in tauri.appstore.conf.json is correct"
  exit 1
fi
"$SCRIPT_DIR/verify-macos-autostart-agent-plist.sh" "$AUTOSTART_AGENT_PATH"
echo "Autostart LaunchAgent embedded successfully"

APP_PLIST="$APP_PATH/Contents/Info.plist"
echo "Setting CFBundleVersion to $BUILD_NUMBER"
/usr/libexec/PlistBuddy -c "Add :CFBundleVersion string $BUILD_NUMBER" "$APP_PLIST" 2>/dev/null || \
/usr/libexec/PlistBuddy -c "Set :CFBundleVersion $BUILD_NUMBER" "$APP_PLIST"

echo "Signing app with: $APPLE_DISTRIBUTION_IDENTITY"
codesign --deep --force --options runtime \
  --sign "$APPLE_DISTRIBUTION_IDENTITY" \
  --entitlements "$ENTITLEMENTS_PATH" \
  "$APP_PATH"

echo "Verifying signature..."
codesign --verify --deep --strict "$APP_PATH"
echo "Signature verified."

echo ""
echo "Embedded entitlements:"
codesign -d --entitlements :- "$APP_PATH" 2>/dev/null | head -20

echo ""
echo "Creating installer package with: $APPLE_INSTALLER_IDENTITY"
productbuild --component "$APP_PATH" /Applications \
  --sign "$APPLE_INSTALLER_IDENTITY" \
  "$PKG_PATH"

echo ""
echo "=== Package Created ==="
echo "Location: $PKG_PATH"
echo ""

echo "Uploading to App Store Connect (validates automatically)..."
xcrun altool --upload-app -f "$PKG_PATH" --type macos \
  --apiKey "$APPLE_API_KEY_ID" --apiIssuer "$APPLE_API_ISSUER" \
  --transport DAV
