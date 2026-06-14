#!/bin/bash
set -euo pipefail

# Prepares App Store signing and upload credentials for hosted CI.

TEMP_DIR="${RUNNER_TEMP:-${TMPDIR:-/tmp}}"
KEYCHAIN_PATH="$TEMP_DIR/brm-appstore-signing.keychain-db"
KEYCHAIN_PASSWORD="$(uuidgen)"

decode_base64_to_file() {
  local value="$1"
  local path="$2"

  if printf '%s' "$value" | base64 --decode > "$path" 2>/dev/null; then
    return 0
  fi

  printf '%s' "$value" | base64 -D > "$path"
}

required_vars=(
  APPLE_DISTRIBUTION_CERT_P12_BASE64
  APPLE_DISTRIBUTION_CERT_PASSWORD
  APPLE_INSTALLER_CERT_P12_BASE64
  APPLE_INSTALLER_CERT_PASSWORD
  APPLE_PROVISION_PROFILE_BASE64
  APPLE_API_KEY_ID
  APPSTORE_CONNECT_API_KEY_P8
)

for var in "${required_vars[@]}"; do
  if [ -z "${!var:-}" ]; then
    echo "Error: $var is not set"
    exit 1
  fi
done

PROVISION_PROFILE_OUTPUT="${PROVISION_PROFILE_OUTPUT:-src-tauri/embedded.provisionprofile}"
mkdir -p "$(dirname "$PROVISION_PROFILE_OUTPUT")"
decode_base64_to_file "$APPLE_PROVISION_PROFILE_BASE64" "$PROVISION_PROFILE_OUTPUT"

mkdir -p "$HOME/.appstoreconnect/private_keys"
printf '%s' "$APPSTORE_CONNECT_API_KEY_P8" > "$HOME/.appstoreconnect/private_keys/AuthKey_${APPLE_API_KEY_ID}.p8"
chmod 600 "$HOME/.appstoreconnect/private_keys/AuthKey_${APPLE_API_KEY_ID}.p8"

security create-keychain -p "$KEYCHAIN_PASSWORD" "$KEYCHAIN_PATH"
security set-keychain-settings -lut 21600 "$KEYCHAIN_PATH"
security unlock-keychain -p "$KEYCHAIN_PASSWORD" "$KEYCHAIN_PATH"
security list-keychains -d user -s "$KEYCHAIN_PATH" $(security list-keychains -d user | sed 's/[ "]//g')
security default-keychain -d user -s "$KEYCHAIN_PATH"

DIST_CERT="$TEMP_DIR/apple-distribution.p12"
INSTALLER_CERT="$TEMP_DIR/apple-installer.p12"
decode_base64_to_file "$APPLE_DISTRIBUTION_CERT_P12_BASE64" "$DIST_CERT"
decode_base64_to_file "$APPLE_INSTALLER_CERT_P12_BASE64" "$INSTALLER_CERT"

security import "$DIST_CERT" -k "$KEYCHAIN_PATH" -P "$APPLE_DISTRIBUTION_CERT_PASSWORD" -T /usr/bin/codesign -T /usr/bin/productbuild
security import "$INSTALLER_CERT" -k "$KEYCHAIN_PATH" -P "$APPLE_INSTALLER_CERT_PASSWORD" -T /usr/bin/codesign -T /usr/bin/productbuild
rm -f "$DIST_CERT" "$INSTALLER_CERT"

security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k "$KEYCHAIN_PASSWORD" "$KEYCHAIN_PATH"
security find-identity -v -p codesigning "$KEYCHAIN_PATH"
