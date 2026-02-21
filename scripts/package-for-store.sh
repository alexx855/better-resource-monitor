#!/bin/bash
set -e

# Builds, signs, packages, and uploads a universal macOS binary to App Store Connect.
# Build number auto-increments from scripts/.build-number on each run.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
ENV_FILE="$SCRIPT_DIR/.env"

VERSION=$(jq -r '.version' "$PROJECT_ROOT/src-tauri/tauri.conf.json")

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

# Auto-increment build number
BUILD_NUMBER_FILE="$SCRIPT_DIR/.build-number"
BUILD_NUMBER=$(cat "$BUILD_NUMBER_FILE" 2>/dev/null || echo 0)
BUILD_NUMBER=$((BUILD_NUMBER + 1))
echo "$BUILD_NUMBER" > "$BUILD_NUMBER_FILE"

echo "=== App Store Packaging Script ==="
echo "Version: $VERSION"
echo "Build: $BUILD_NUMBER"
echo ""

APP_NAME="Better Resource Monitor"
ENTITLEMENTS_PATH="src-tauri/Entitlements.plist"

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
echo "Using --features apple-app-store to strip private IOReport APIs (Guideline 2.5.1)"
pnpm tauri build --features apple-app-store --bundles app --target universal-apple-darwin

if [ ! -d "$APP_PATH" ]; then
  echo "Error: App bundle not found at $APP_PATH"
  exit 1
fi

echo "App bundle found at: $APP_PATH"

if [ ! -f "$APP_PATH/Contents/embedded.provisionprofile" ]; then
  echo "Error: Provisioning profile was not embedded by Tauri"
  echo "Check that 'files' config in tauri.conf.json is correct"
  exit 1
fi
echo "Provisioning profile embedded successfully"

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

echo "Uploading to App Store Connect..."
xcrun altool --upload-app -f "$PKG_PATH" --type macos \
  --apiKey "$APPLE_API_KEY_ID" --apiIssuer "$APPLE_API_ISSUER"
