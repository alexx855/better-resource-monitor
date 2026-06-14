#!/bin/bash
set -euo pipefail

# Signs, packages, and uploads an already-built app bundle. This script is run
# from trusted workflow tooling after PR-controlled build/test code has finished.

SOURCE_DIR="${1:?source directory is required}"
BUILD_NUMBER_INPUT="${2:?build number is required}"
BRM_BUILD_COMMIT_INPUT="${3:?build commit is required}"
PROVISION_PROFILE_PATH="${4:?provisioning profile path is required}"

case "$BUILD_NUMBER_INPUT" in
  ''|*[!0-9]*)
    echo "Error: build number must be numeric, got '$BUILD_NUMBER_INPUT'"
    exit 1
    ;;
esac

BRM_BUILD_COMMIT="$(printf '%s' "$BRM_BUILD_COMMIT_INPUT" | cut -c1-12)"
case "$BRM_BUILD_COMMIT" in
  [0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f])
    ;;
  *)
    echo "Error: build commit must resolve to a 12-character lowercase git commit prefix, got '$BRM_BUILD_COMMIT'"
    exit 1
    ;;
esac

required_vars=(
  APPLE_TEAM_ID
  APPLE_DISTRIBUTION_IDENTITY
  APPLE_INSTALLER_IDENTITY
  APPLE_API_KEY_ID
  APPLE_API_ISSUER
)
for var in "${required_vars[@]}"; do
  if [ -z "${!var:-}" ]; then
    echo "Error: $var is not set"
    exit 1
  fi
done

if [ ! -d "$SOURCE_DIR" ]; then
  echo "Error: source directory not found: $SOURCE_DIR"
  exit 1
fi
if [ ! -f "$PROVISION_PROFILE_PATH" ]; then
  echo "Error: provisioning profile not found at $PROVISION_PROFILE_PATH"
  exit 1
fi

cd "$SOURCE_DIR"

VERSION="$(jq -r '.version' src-tauri/tauri.conf.json)"
BUILD_NUMBER="$BUILD_NUMBER_INPUT"
APP_NAME="Better Resource Monitor"
ENTITLEMENTS_PATH="src-tauri/Entitlements.appstore.plist"
TARGET_DIR="src-tauri/target/universal-apple-darwin/release/bundle/macos"
APP_PATH="${TARGET_DIR}/${APP_NAME}.app"
PKG_PATH="${TARGET_DIR}/${APP_NAME}.pkg"
APP_EXECUTABLE="$APP_PATH/Contents/MacOS/better-resource-monitor"
APP_PLIST="$APP_PATH/Contents/Info.plist"

echo "=== App Store Package Upload ==="
echo "Version: $VERSION"
echo "Build: $BUILD_NUMBER"
echo "Commit: $BRM_BUILD_COMMIT"
echo ""

if [ ! -d "$APP_PATH" ]; then
  echo "Error: App bundle not found at $APP_PATH"
  exit 1
fi
echo "App bundle found at: $APP_PATH"

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

cp "$PROVISION_PROFILE_PATH" "$APP_PATH/Contents/embedded.provisionprofile"
echo "Provisioning profile embedded successfully"

if [ -d "$APP_PATH/Contents/Library/LaunchAgents" ]; then
  echo "Error: App bundle must not embed LaunchAgents"
  exit 1
fi

PRIVACY_MANIFEST="$APP_PATH/Contents/Resources/PrivacyInfo.xcprivacy"
if [ ! -f "$PRIVACY_MANIFEST" ]; then
  echo "Error: PrivacyInfo.xcprivacy was not embedded in Contents/Resources"
  exit 1
fi
plutil -lint "$PRIVACY_MANIFEST"
DISK_CATEGORY=$(/usr/libexec/PlistBuddy -c "Print :NSPrivacyAccessedAPITypes:0:NSPrivacyAccessedAPIType" "$PRIVACY_MANIFEST" 2>/dev/null || true)
DISK_REASON=$(/usr/libexec/PlistBuddy -c "Print :NSPrivacyAccessedAPITypes:0:NSPrivacyAccessedAPITypeReasons:0" "$PRIVACY_MANIFEST" 2>/dev/null || true)
if [ "$DISK_CATEGORY" != "NSPrivacyAccessedAPICategoryDiskSpace" ]; then
  echo "Error: Privacy manifest missing disk-space API category"
  exit 1
fi
if [ "$DISK_REASON" != "85F4.1" ]; then
  echo "Error: Privacy manifest missing disk-space reason 85F4.1"
  exit 1
fi
echo "Privacy manifest disk-space reason verified"

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
ENTITLEMENTS_OUT="$(mktemp "${TMPDIR:-/tmp}/brm-entitlements.XXXXXX")"
trap 'rm -f "$ENTITLEMENTS_OUT"' EXIT
codesign -d --entitlements :- "$APP_PATH" > "$ENTITLEMENTS_OUT" 2>/dev/null
head -20 "$ENTITLEMENTS_OUT" || true

SANDBOX_ENTITLEMENT=$(/usr/libexec/PlistBuddy -c 'Print :com.apple.security.app-sandbox' "$ENTITLEMENTS_OUT" 2>/dev/null || true)
APP_IDENTIFIER_ENTITLEMENT=$(/usr/libexec/PlistBuddy -c 'Print :com.apple.application-identifier' "$ENTITLEMENTS_OUT" 2>/dev/null || true)
TEAM_IDENTIFIER_ENTITLEMENT=$(/usr/libexec/PlistBuddy -c 'Print :com.apple.developer.team-identifier' "$ENTITLEMENTS_OUT" 2>/dev/null || true)

if [ "$SANDBOX_ENTITLEMENT" != "true" ]; then
  echo "Error: Signed app is missing com.apple.security.app-sandbox=true"
  exit 1
fi
if [ "$APP_IDENTIFIER_ENTITLEMENT" != "$APPLE_TEAM_ID.dev.alexpedersen.better-resource-monitor" ]; then
  echo "Error: Signed app has unexpected application identifier entitlement: $APP_IDENTIFIER_ENTITLEMENT"
  exit 1
fi
if [ "$TEAM_IDENTIFIER_ENTITLEMENT" != "$APPLE_TEAM_ID" ]; then
  echo "Error: Signed app has unexpected team identifier entitlement: $TEAM_IDENTIFIER_ENTITLEMENT"
  exit 1
fi

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
