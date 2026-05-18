#!/usr/bin/env bash
set -euo pipefail

ZIP_PATH="${1:-}"
APP_PATH="${APP_PATH:-/Applications/Better Resource Monitor.app}"
BUNDLE_ID="${BUNDLE_ID:-dev.alexpedersen.better-resource-monitor}"
EXPECTED_VERSION="${EXPECTED_VERSION:-1.1.3}"
EXPECTED_TEAM_ID="${EXPECTED_TEAM_ID:-G76YQZM2FU}"
EXPECTED_BUILD_COMMIT="${EXPECTED_BUILD_COMMIT:-}"
PROCESS_NAME="${PROCESS_NAME:-better-resource-monitor}"
AUTOSTART_AGENT_PLIST="${AUTOSTART_AGENT_PLIST:-dev.alexpedersen.better-resource-monitor.autostart.plist}"
SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)

fail() {
  echo "FAIL: $*" >&2
  exit 1
}

note() {
  echo "==> $*"
}

verify_executable_contains_build_commit() {
  if ! strings -a "$EXECUTABLE" | awk -v needle="$EXPECTED_BUILD_COMMIT" 'index($0, needle) { found = 1 } END { exit found ? 0 : 1 }'; then
    fail "Artifact executable does not contain expected build commit $EXPECTED_BUILD_COMMIT"
  fi
}

validate_expected_build_commit() {
  if [[ ! "$EXPECTED_BUILD_COMMIT" =~ ^[0-9a-f]{12}$ ]]; then
    fail "EXPECTED_BUILD_COMMIT must be the 12-character lowercase git commit prefix"
  fi
}

usage() {
  cat <<EOF
Usage: EXPECTED_BUILD_COMMIT=<commit> $0 <Better-Resource-Monitor-x86_64.app.zip>

Installs a signed Better Resource Monitor .app zip produced by the signed Intel
GitHub Actions workflow. You can pass either the inner
Better-Resource-Monitor-x86_64.app.zip file or the outer downloaded GitHub
artifact zip that contains it.
EOF
}

if [[ "$(uname -s)" != "Darwin" ]]; then
  fail "This installer must run on macOS"
fi

validate_expected_build_commit

if [[ -z "$ZIP_PATH" ]]; then
  usage
  exit 2
fi

if [[ ! -f "$ZIP_PATH" ]]; then
  fail "Artifact zip not found: $ZIP_PATH"
fi

SHASUM_PATH="${ZIP_PATH}.sha256"
if [[ -f "$SHASUM_PATH" ]]; then
  note "Verifying artifact checksum"
  EXPECTED_SHA=$(awk '{print $1; exit}' "$SHASUM_PATH")
  ACTUAL_SHA=$(shasum -a 256 "$ZIP_PATH" | awk '{print $1; exit}')
  [[ -n "$EXPECTED_SHA" ]] || fail "No checksum found in $SHASUM_PATH"
  [[ "$ACTUAL_SHA" == "$EXPECTED_SHA" ]] || fail "Checksum mismatch for $ZIP_PATH"
fi

TMP_DIR=$(mktemp -d "${TMPDIR:-/tmp}/brm-install.XXXXXX")
cleanup() {
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

note "Extracting artifact"
EXTRACT_DIR="$TMP_DIR/extracted"
mkdir -p "$EXTRACT_DIR"
ditto -x -k "$ZIP_PATH" "$EXTRACT_DIR"

SOURCE_APP=$(find "$EXTRACT_DIR" -maxdepth 3 -name "Better Resource Monitor.app" -type d -print -quit)
if [[ -z "$SOURCE_APP" ]]; then
  INNER_ZIP=$(find "$EXTRACT_DIR" -maxdepth 3 -name "Better-Resource-Monitor-x86_64.app.zip" -type f -print -quit)
  if [[ -n "$INNER_ZIP" ]]; then
    INNER_SHA="${INNER_ZIP}.sha256"
    if [[ -f "$INNER_SHA" ]]; then
      note "Verifying nested artifact checksum"
      EXPECTED_SHA=$(awk '{print $1; exit}' "$INNER_SHA")
      ACTUAL_SHA=$(shasum -a 256 "$INNER_ZIP" | awk '{print $1; exit}')
      [[ -n "$EXPECTED_SHA" ]] || fail "No checksum found in $INNER_SHA"
      [[ "$ACTUAL_SHA" == "$EXPECTED_SHA" ]] || fail "Checksum mismatch for $INNER_ZIP"
    fi

    note "Extracting nested app zip"
    NESTED_DIR="$TMP_DIR/nested"
    mkdir -p "$NESTED_DIR"
    ditto -x -k "$INNER_ZIP" "$NESTED_DIR"
    SOURCE_APP=$(find "$NESTED_DIR" -maxdepth 3 -name "Better Resource Monitor.app" -type d -print -quit)
  fi
fi
if [[ -z "$SOURCE_APP" ]]; then
  fail "Better Resource Monitor.app not found inside $ZIP_PATH"
fi

INFO_PLIST="$SOURCE_APP/Contents/Info.plist"
EXECUTABLE="$SOURCE_APP/Contents/MacOS/$PROCESS_NAME"
AUTOSTART_AGENT="$SOURCE_APP/Contents/Library/LaunchAgents/$AUTOSTART_AGENT_PLIST"
[[ -f "$INFO_PLIST" ]] || fail "Info.plist not found: $INFO_PLIST"
[[ -x "$EXECUTABLE" ]] || fail "Executable not found: $EXECUTABLE"
[[ -f "$AUTOSTART_AGENT" ]] || fail "Autostart LaunchAgent not found: $AUTOSTART_AGENT"

note "Artifact identity"
ARTIFACT_BUNDLE_ID=$(/usr/libexec/PlistBuddy -c 'Print :CFBundleIdentifier' "$INFO_PLIST")
ARTIFACT_VERSION=$(/usr/libexec/PlistBuddy -c 'Print :CFBundleShortVersionString' "$INFO_PLIST")
echo "path=$SOURCE_APP"
echo "bundle_id=$ARTIFACT_BUNDLE_ID"
echo "version=$ARTIFACT_VERSION"
LIPO_INFO=$(lipo -info "$EXECUTABLE")
echo "$LIPO_INFO"
"$SCRIPT_DIR/verify-macos-autostart-agent-plist.sh" "$AUTOSTART_AGENT"

[[ "$ARTIFACT_BUNDLE_ID" == "$BUNDLE_ID" ]] || fail "Expected bundle id $BUNDLE_ID, got $ARTIFACT_BUNDLE_ID"
[[ "$ARTIFACT_VERSION" == "$EXPECTED_VERSION" ]] || fail "Expected version $EXPECTED_VERSION, got $ARTIFACT_VERSION"
[[ "$LIPO_INFO" == *"x86_64"* ]] || fail "Artifact executable does not contain x86_64"
verify_executable_contains_build_commit

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
  echo "EXPECTED_VERSION=$EXPECTED_VERSION EXPECTED_BUILD_COMMIT=$EXPECTED_BUILD_COMMIT \"$SCRIPT_DIR/verify-macos-autostart.sh\""
else
  echo "EXPECTED_VERSION=$EXPECTED_VERSION \"$SCRIPT_DIR/verify-macos-autostart.sh\""
fi
