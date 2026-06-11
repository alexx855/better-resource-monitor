#!/bin/bash
set -euo pipefail

# Prepares Developer ID signing and notarization credentials for direct-download
# macOS release artifacts.

TEMP_DIR="${RUNNER_TEMP:-${TMPDIR:-/tmp}}"
KEYCHAIN_PATH="$TEMP_DIR/brm-developer-id-signing.keychain-db"
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
  DEVELOPER_ID_APPLICATION_CERT_P12_BASE64
  DEVELOPER_ID_APPLICATION_CERT_PASSWORD
  APPLE_API_KEY_ID
  APPLE_API_ISSUER
  APPSTORE_CONNECT_API_KEY_P8
)

for var in "${required_vars[@]}"; do
  if [ -z "${!var:-}" ]; then
    echo "Error: $var is not set"
    exit 1
  fi
done

mkdir -p "$HOME/.appstoreconnect/private_keys"
printf '%s' "$APPSTORE_CONNECT_API_KEY_P8" > "$HOME/.appstoreconnect/private_keys/AuthKey_${APPLE_API_KEY_ID}.p8"
chmod 600 "$HOME/.appstoreconnect/private_keys/AuthKey_${APPLE_API_KEY_ID}.p8"

security create-keychain -p "$KEYCHAIN_PASSWORD" "$KEYCHAIN_PATH"
security set-keychain-settings -lut 21600 "$KEYCHAIN_PATH"
security unlock-keychain -p "$KEYCHAIN_PASSWORD" "$KEYCHAIN_PATH"
security list-keychains -d user -s "$KEYCHAIN_PATH" $(security list-keychains -d user | sed 's/[ "]//g')
security default-keychain -d user -s "$KEYCHAIN_PATH"

APPLICATION_CERT="$TEMP_DIR/developer-id-application.p12"
decode_base64_to_file "$DEVELOPER_ID_APPLICATION_CERT_P12_BASE64" "$APPLICATION_CERT"

security import "$APPLICATION_CERT" -k "$KEYCHAIN_PATH" -P "$DEVELOPER_ID_APPLICATION_CERT_PASSWORD" -T /usr/bin/codesign
rm -f "$APPLICATION_CERT"

security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k "$KEYCHAIN_PASSWORD" "$KEYCHAIN_PATH"
security find-identity -v -p codesigning "$KEYCHAIN_PATH"
