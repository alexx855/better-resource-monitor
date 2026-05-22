#!/bin/bash
set -e

# Builds, signs, packages, and uploads a universal macOS binary to App Store Connect.
# Local runs may load Apple credentials from scripts/.env. GitHub Actions should
# provide the same values as environment variables.
# Build number can be set with BUILD_NUMBER. Otherwise it auto-increments from
# scripts/.build-number while never falling below a timestamp-scale value.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
ENV_FILE="$SCRIPT_DIR/.env"

cd "$PROJECT_ROOT"

VERSION=$(jq -r '.version' "$PROJECT_ROOT/src-tauri/tauri.conf.json")
BRM_BUILD_COMMIT="${BRM_BUILD_COMMIT:-$(git -C "$PROJECT_ROOT" rev-parse --short=12 HEAD 2>/dev/null || echo unknown)}"
export BRM_BUILD_COMMIT
case "$BRM_BUILD_COMMIT" in
  [0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f])
    ;;
  *)
    echo "Error: BRM_BUILD_COMMIT must be the 12-character lowercase git commit prefix, got '$BRM_BUILD_COMMIT'"
    exit 1
    ;;
esac

if [ -f "$ENV_FILE" ]; then
  set -a
  source "$ENV_FILE"
  set +a
elif [ -z "${CI:-}" ]; then
  echo "Error: .env file not found at $ENV_FILE"
  echo "Create scripts/.env with required variables (see scripts/.env.example)"
  echo "CI may provide these variables directly in the environment."
  exit 1
fi

required_vars=("APPLE_TEAM_ID" "APPLE_DISTRIBUTION_IDENTITY" "APPLE_INSTALLER_IDENTITY" "APPLE_API_KEY_ID" "APPLE_API_ISSUER")
for var in "${required_vars[@]}"; do
  if [ -z "${!var}" ]; then
    echo "Error: $var is not set"
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
  echo "$BUILD_NUMBER" > "$BUILD_NUMBER_FILE"
fi

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

APP_EXECUTABLE="$APP_PATH/Contents/MacOS/better-resource-monitor"
if ! strings -a "$APP_EXECUTABLE" | awk -v needle="$BRM_BUILD_COMMIT" 'index($0, needle) { found = 1 } END { exit found ? 0 : 1 }'; then
  echo "Error: App executable does not contain expected build commit $BRM_BUILD_COMMIT"
  exit 1
fi
echo "Build commit embedded successfully"

if [ ! -f "$APP_PATH/Contents/embedded.provisionprofile" ]; then
  echo "Error: Provisioning profile was not embedded by Tauri"
  echo "Check that 'files' config in tauri.appstore.conf.json is correct"
  exit 1
fi
echo "Provisioning profile embedded successfully"

if [ -d "$APP_PATH/Contents/Library/LaunchAgents" ]; then
  echo "Error: App bundle must not embed LaunchAgents"
  exit 1
fi

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
ENTITLEMENTS_OUT="$(mktemp "${TMPDIR:-/tmp}/brm-entitlements.XXXXXX.plist")"
trap 'rm -f "$ENTITLEMENTS_OUT"' EXIT
codesign -d --entitlements :- "$APP_PATH" > "$ENTITLEMENTS_OUT" 2>/dev/null
cat "$ENTITLEMENTS_OUT" | head -20

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
