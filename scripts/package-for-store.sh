#!/bin/bash
set -e

# Build number is required for TestFlight uploads
BUILD_NUMBER="${1:?Error: Build number required. Usage: $0 <build_number>}"

# Get script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
ENV_FILE="$SCRIPT_DIR/.env"

# Extract version from tauri.conf.json (single source of truth)
VERSION=$(jq -r '.version' "$PROJECT_ROOT/src-tauri/tauri.conf.json")

# Fail fast if .env not found
if [ ! -f "$ENV_FILE" ]; then
  echo "Error: .env file not found at $ENV_FILE"
  echo "Create scripts/.env with required variables (see scripts/.env.example)"
  exit 1
fi

# Load environment variables (handles spaces in values)
set -a
source "$ENV_FILE"
set +a

# Required variables - fail fast if missing
required_vars=("APPLE_TEAM_ID" "APPLE_DISTRIBUTION_IDENTITY" "APPLE_INSTALLER_IDENTITY")
for var in "${required_vars[@]}"; do
  if [ -z "${!var}" ]; then
    echo "Error: $var is not set in $ENV_FILE"
    exit 1
  fi
done

echo "=== App Store Packaging Script ==="
echo "Version: $VERSION"
echo "Build: $BUILD_NUMBER"
echo ""

# Configuration from environment
APP_NAME="Better Resource Monitor"
ENTITLEMENTS_PATH="src-tauri/Entitlements.plist"

# Build paths
TARGET_DIR="src-tauri/target/universal-apple-darwin/release/bundle/macos"
APP_PATH="${TARGET_DIR}/${APP_NAME}.app"
PKG_PATH="${TARGET_DIR}/${APP_NAME}.pkg"

# Check provisioning profile exists
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
pnpm tauri build --bundles app --target universal-apple-darwin

# Check if App exists
if [ ! -d "$APP_PATH" ]; then
  echo "Error: App bundle not found at $APP_PATH"
  exit 1
fi

echo "App bundle found at: $APP_PATH"

# Verify provisioning profile was embedded by Tauri
if [ ! -f "$APP_PATH/Contents/embedded.provisionprofile" ]; then
  echo "Error: Provisioning profile was not embedded by Tauri"
  echo "Check that 'files' config in tauri.conf.json is correct"
  exit 1
fi
echo "Provisioning profile embedded successfully"

# Inject build number into Info.plist (required for TestFlight)
APP_PLIST="$APP_PATH/Contents/Info.plist"
echo "Setting CFBundleVersion to $BUILD_NUMBER"
/usr/libexec/PlistBuddy -c "Add :CFBundleVersion string $BUILD_NUMBER" "$APP_PLIST" 2>/dev/null || \
/usr/libexec/PlistBuddy -c "Set :CFBundleVersion $BUILD_NUMBER" "$APP_PLIST"

# Re-sign the app with Distribution certificate and entitlements
echo "Signing app with: $APPLE_DISTRIBUTION_IDENTITY"
codesign --deep --force --options runtime \
  --sign "$APPLE_DISTRIBUTION_IDENTITY" \
  --entitlements "$ENTITLEMENTS_PATH" \
  "$APP_PATH"

# Verify signature
echo "Verifying signature..."
codesign --verify --deep --strict "$APP_PATH"
echo "Signature verified."

# Show entitlements for verification
echo ""
echo "Embedded entitlements:"
codesign -d --entitlements :- "$APP_PATH" 2>/dev/null | head -20

# Create installer package
echo ""
echo "Creating installer package with: $APPLE_INSTALLER_IDENTITY"
productbuild --component "$APP_PATH" /Applications \
  --sign "$APPLE_INSTALLER_IDENTITY" \
  "$PKG_PATH"

echo ""
echo "=== Package Created ==="
echo "Location: $PKG_PATH"
echo ""

# Upload option
if [ -n "$APPLE_API_KEY_ID" ] && [ -n "$APPLE_API_ISSUER" ]; then
  echo "API Key detected. Upload with:"
  echo "  xcrun altool --upload-app -f \"$PKG_PATH\" --type macos \\"
  echo "    --apiKey $APPLE_API_KEY_ID --apiIssuer $APPLE_API_ISSUER"
  echo ""
  read -p "Upload now? (y/N) " -n 1 -r
  echo
  if [[ $REPLY =~ ^[Yy]$ ]]; then
    xcrun altool --upload-app -f "$PKG_PATH" --type macos \
      --apiKey "$APPLE_API_KEY_ID" --apiIssuer "$APPLE_API_ISSUER"
  fi
else
  echo "Next steps:"
  echo "1. Open Transporter app (download from Mac App Store)"
  echo "2. Drag and drop the .pkg file into Transporter"
  echo "3. Click 'Deliver'"
  echo ""
  echo "Or set APPLE_API_KEY_ID and APPLE_API_ISSUER in .env for CLI upload"
fi
