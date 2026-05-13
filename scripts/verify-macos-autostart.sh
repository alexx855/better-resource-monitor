#!/usr/bin/env bash
set -euo pipefail

APP_PATH="${APP_PATH:-/Applications/Better Resource Monitor.app}"
BUNDLE_ID="${BUNDLE_ID:-dev.alexpedersen.better-resource-monitor}"
EXPECTED_VERSION="${EXPECTED_VERSION:-}"
EXPECTED_BUILD_COMMIT="${EXPECTED_BUILD_COMMIT:-}"
EXPECTED_TEAM_ID="${EXPECTED_TEAM_ID:-G76YQZM2FU}"
PROCESS_NAME="${PROCESS_NAME:-better-resource-monitor}"

fail() {
  echo "FAIL: $*" >&2
  exit 1
}

note() {
  echo "==> $*"
}

print_recent_startup_log() {
  local found=0
  local log_paths=(
    "$HOME/Library/Logs/Better Resource Monitor/autostart.log"
    "$HOME/Library/Containers/$BUNDLE_ID/Data/Library/Logs/Better Resource Monitor/autostart.log"
  )

  for log_path in "${log_paths[@]}"; do
    if [[ -f "$log_path" ]]; then
      found=1
      note "Recent app startup log: $log_path"
      tail -40 "$log_path"
    fi
  done

  if [[ "$found" -eq 0 ]]; then
    note "App startup log not found in standard or sandbox container log paths"
  fi
}

verify_expected_build_commit() {
  if [[ -z "$EXPECTED_BUILD_COMMIT" ]]; then
    return
  fi

  local found=0
  local log_paths=(
    "$HOME/Library/Logs/Better Resource Monitor/autostart.log"
    "$HOME/Library/Containers/$BUNDLE_ID/Data/Library/Logs/Better Resource Monitor/autostart.log"
  )

  for log_path in "${log_paths[@]}"; do
    if [[ -f "$log_path" ]] && grep -q "build_commit=$EXPECTED_BUILD_COMMIT" "$log_path"; then
      found=1
    fi
  done

  [[ "$found" -eq 1 ]] || fail "Expected startup log to contain build_commit=$EXPECTED_BUILD_COMMIT"
}

if [[ "$(uname -s)" != "Darwin" ]]; then
  fail "This verifier must run on macOS"
fi

if [[ ! -d "$APP_PATH" ]]; then
  fail "App bundle not found: $APP_PATH"
fi

INFO_PLIST="$APP_PATH/Contents/Info.plist"
EXECUTABLE="$APP_PATH/Contents/MacOS/$PROCESS_NAME"

[[ -f "$INFO_PLIST" ]] || fail "Info.plist not found: $INFO_PLIST"
[[ -x "$EXECUTABLE" ]] || fail "Executable not found: $EXECUTABLE"

note "Installed app"
INSTALLED_BUNDLE_ID=$(/usr/libexec/PlistBuddy -c 'Print :CFBundleIdentifier' "$INFO_PLIST")
INSTALLED_VERSION=$(/usr/libexec/PlistBuddy -c 'Print :CFBundleShortVersionString' "$INFO_PLIST")
INSTALLED_BUILD=$(/usr/libexec/PlistBuddy -c 'Print :CFBundleVersion' "$INFO_PLIST" 2>/dev/null || true)
echo "path=$APP_PATH"
echo "bundle_id=$INSTALLED_BUNDLE_ID"
echo "version=$INSTALLED_VERSION"
echo "build=${INSTALLED_BUILD:-<missing>}"

[[ "$INSTALLED_BUNDLE_ID" == "$BUNDLE_ID" ]] || fail "Expected bundle id $BUNDLE_ID, got $INSTALLED_BUNDLE_ID"
if [[ -n "$EXPECTED_VERSION" && "$INSTALLED_VERSION" != "$EXPECTED_VERSION" ]]; then
  fail "Expected version $EXPECTED_VERSION, got $INSTALLED_VERSION"
fi

note "Architecture"
LIPO_INFO=$(lipo -info "$EXECUTABLE")
echo "$LIPO_INFO"
if [[ "$(uname -m)" == "x86_64" ]]; then
  [[ "$LIPO_INFO" == *"x86_64"* ]] || fail "Installed executable does not contain x86_64"
fi

note "Code signature"
codesign --verify --deep --strict --verbose=2 "$APP_PATH"
CODESIGN_DETAILS=$(codesign -dv --verbose=4 "$APP_PATH" 2>&1)
echo "$CODESIGN_DETAILS" | sed -n '/^Identifier=/p;/^Authority=/p;/^TeamIdentifier=/p'
TEAM_ID=$(echo "$CODESIGN_DETAILS" | awk -F= '/^TeamIdentifier=/{print $2; exit}')
[[ "$TEAM_ID" == "$EXPECTED_TEAM_ID" ]] || fail "Expected TeamIdentifier $EXPECTED_TEAM_ID, got ${TEAM_ID:-<missing>}"

note "Gatekeeper assessment"
spctl -a -vv -t execute "$APP_PATH"

note "Background Item registration"
BTM_MATCH=$(sfltool dumpbtm | grep -A 12 -B 4 "$BUNDLE_ID" || true)
[[ -n "$BTM_MATCH" ]] || fail "No Background Item found for $BUNDLE_ID"
echo "$BTM_MATCH"
echo "$BTM_MATCH" | grep -q "enabled" || fail "Background Item for $BUNDLE_ID is not enabled"

note "Running process"
if pgrep -x "$PROCESS_NAME" >/dev/null; then
  pgrep -ax "$PROCESS_NAME"
else
  print_recent_startup_log
  fail "$PROCESS_NAME is not running. If you just installed the app, log out/in or reboot, then run this verifier again."
fi

print_recent_startup_log
verify_expected_build_commit

note "Verification passed"
