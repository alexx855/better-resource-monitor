#!/usr/bin/env bash
set -euo pipefail

ZIP_PATH="${1:-}"
APP_PATH="${APP_PATH:-/Applications/Better Resource Monitor.app}"
BUNDLE_ID="${BUNDLE_ID:-dev.alexpedersen.better-resource-monitor}"
EXPECTED_VERSION="${EXPECTED_VERSION:-1.1.3}"
EXPECTED_TEAM_ID="${EXPECTED_TEAM_ID:-G76YQZM2FU}"
EXPECTED_BUILD_COMMIT="${EXPECTED_BUILD_COMMIT:-}"
PROCESS_NAME="${PROCESS_NAME:-better-resource-monitor}"

fail() {
  echo "FAIL: $*" >&2
  exit 1
}

note() {
  echo "==> $*"
}

usage() {
  cat <<EOF
Usage: EXPECTED_BUILD_COMMIT=<commit> $0 <Better-Resource-Monitor-x86_64.app.zip>

Installs a signed Better Resource Monitor .app zip produced by the signed Intel
GitHub Actions workflow, then prints the post-login verification command.
EOF
}

if [[ "$(uname -s)" != "Darwin" ]]; then
  fail "This installer must run on macOS"
fi

if [[ -z "$ZIP_PATH" ]]; then
  usage
  exit 2
fi

if [[ ! -f "$ZIP_PATH" ]]; then
  fail "Artifact zip not found: $ZIP_PATH"
fi

TMP_DIR=$(mktemp -d "${TMPDIR:-/tmp}/brm-install.XXXXXX")
cleanup() {
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

note "Extracting artifact"
ditto -x -k "$ZIP_PATH" "$TMP_DIR"

SOURCE_APP=$(find "$TMP_DIR" -maxdepth 2 -name "Better Resource Monitor.app" -type d -print -quit)
if [[ -z "$SOURCE_APP" ]]; then
  fail "Better Resource Monitor.app not found inside $ZIP_PATH"
fi

INFO_PLIST="$SOURCE_APP/Contents/Info.plist"
EXECUTABLE="$SOURCE_APP/Contents/MacOS/$PROCESS_NAME"
[[ -f "$INFO_PLIST" ]] || fail "Info.plist not found: $INFO_PLIST"
[[ -x "$EXECUTABLE" ]] || fail "Executable not found: $EXECUTABLE"

note "Artifact identity"
ARTIFACT_BUNDLE_ID=$(/usr/libexec/PlistBuddy -c 'Print :CFBundleIdentifier' "$INFO_PLIST")
ARTIFACT_VERSION=$(/usr/libexec/PlistBuddy -c 'Print :CFBundleShortVersionString' "$INFO_PLIST")
echo "path=$SOURCE_APP"
echo "bundle_id=$ARTIFACT_BUNDLE_ID"
echo "version=$ARTIFACT_VERSION"
lipo -info "$EXECUTABLE"

[[ "$ARTIFACT_BUNDLE_ID" == "$BUNDLE_ID" ]] || fail "Expected bundle id $BUNDLE_ID, got $ARTIFACT_BUNDLE_ID"
[[ "$ARTIFACT_VERSION" == "$EXPECTED_VERSION" ]] || fail "Expected version $EXPECTED_VERSION, got $ARTIFACT_VERSION"

note "Artifact signature"
codesign --verify --deep --strict --verbose=2 "$SOURCE_APP"
CODESIGN_DETAILS=$(codesign -dv --verbose=4 "$SOURCE_APP" 2>&1)
echo "$CODESIGN_DETAILS" | sed -n '/^Identifier=/p;/^Authority=/p;/^TeamIdentifier=/p'
TEAM_ID=$(echo "$CODESIGN_DETAILS" | awk -F= '/^TeamIdentifier=/{print $2; exit}')
[[ "$TEAM_ID" == "$EXPECTED_TEAM_ID" ]] || fail "Expected TeamIdentifier $EXPECTED_TEAM_ID, got ${TEAM_ID:-<missing>}"

note "Gatekeeper assessment"
spctl -a -vv -t install "$SOURCE_APP"

note "Stopping running app"
pkill -x "$PROCESS_NAME" 2>/dev/null || true

INSTALL_PARENT=$(dirname "$APP_PATH")
STAGED_APP="$INSTALL_PARENT/.Better Resource Monitor.app.installing.$$"
rm -rf "$STAGED_APP"

note "Staging install at $STAGED_APP"
ditto "$SOURCE_APP" "$STAGED_APP"
codesign --verify --deep --strict --verbose=2 "$STAGED_APP"
spctl -a -vv -t install "$STAGED_APP"

note "Installing to $APP_PATH"
rm -rf "$APP_PATH"
mv "$STAGED_APP" "$APP_PATH"

note "Installed app signature"
codesign --verify --deep --strict --verbose=2 "$APP_PATH"
spctl -a -vv -t install "$APP_PATH"

note "Installed. Log out and back in, then run:"
if [[ -n "$EXPECTED_BUILD_COMMIT" ]]; then
  echo "EXPECTED_VERSION=$EXPECTED_VERSION EXPECTED_BUILD_COMMIT=$EXPECTED_BUILD_COMMIT scripts/verify-macos-autostart.sh"
else
  echo "EXPECTED_VERSION=$EXPECTED_VERSION scripts/verify-macos-autostart.sh"
fi
