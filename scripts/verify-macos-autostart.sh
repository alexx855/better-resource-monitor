#!/usr/bin/env bash
set -euo pipefail

APP_PATH="${APP_PATH:-/Applications/Better Resource Monitor.app}"
BUNDLE_ID="${BUNDLE_ID:-dev.alexpedersen.better-resource-monitor}"
EXPECTED_VERSION="${EXPECTED_VERSION:-}"
EXPECTED_BUILD="${EXPECTED_BUILD:-}"
EXPECTED_BUILD_COMMIT="${EXPECTED_BUILD_COMMIT:-}"
EXPECTED_TEAM_ID="${EXPECTED_TEAM_ID:-G76YQZM2FU}"
PROCESS_NAME="${PROCESS_NAME:-better-resource-monitor}"
AUTOSTART_AGENT_LABEL="${AUTOSTART_AGENT_LABEL:-dev.alexpedersen.better-resource-monitor.autostart}"
AUTOSTART_AGENT_PLIST="${AUTOSTART_AGENT_PLIST:-dev.alexpedersen.better-resource-monitor.autostart.plist}"
SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)

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

verify_executable_contains_build_commit() {
  if ! strings -a "$EXECUTABLE" | awk -v needle="$EXPECTED_BUILD_COMMIT" 'index($0, needle) { found = 1 } END { exit found ? 0 : 1 }'; then
    fail "Installed executable does not contain expected build commit $EXPECTED_BUILD_COMMIT"
  fi
}

process_uses_installed_executable() {
  local pid="$1"
  local command="$2"

  if [[ "$command" == "$EXECUTABLE"* ]]; then
    return 0
  fi

  lsof -p "$pid" -a -d txt -Fn 2>/dev/null \
    | sed -n 's/^n//p' \
    | grep -Fxq "$EXECUTABLE"
}

validate_expected_build_commit() {
  if [[ ! "$EXPECTED_BUILD_COMMIT" =~ ^[0-9a-f]{12}$ ]]; then
    fail "EXPECTED_BUILD_COMMIT must be the 12-character lowercase git commit prefix"
  fi
}

if [[ "$(uname -s)" != "Darwin" ]]; then
  fail "This verifier must run on macOS"
fi

validate_expected_build_commit

if [[ ! -d "$APP_PATH" ]]; then
  fail "App bundle not found: $APP_PATH"
fi

INFO_PLIST="$APP_PATH/Contents/Info.plist"
EXECUTABLE="$APP_PATH/Contents/MacOS/$PROCESS_NAME"
AUTOSTART_AGENT="$APP_PATH/Contents/Library/LaunchAgents/$AUTOSTART_AGENT_PLIST"

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
if [[ -n "$EXPECTED_BUILD" && "$INSTALLED_BUILD" != "$EXPECTED_BUILD" ]]; then
  fail "Expected build $EXPECTED_BUILD, got ${INSTALLED_BUILD:-<missing>}"
fi

note "Architecture"
LIPO_INFO=$(lipo -info "$EXECUTABLE")
echo "$LIPO_INFO"
if [[ "$(uname -m)" == "x86_64" ]]; then
  [[ "$LIPO_INFO" == *"x86_64"* ]] || fail "Installed executable does not contain x86_64"
fi
verify_executable_contains_build_commit
[[ -f "$AUTOSTART_AGENT" ]] || fail "Autostart LaunchAgent not found: $AUTOSTART_AGENT"
"$SCRIPT_DIR/verify-macos-autostart-agent-plist.sh" "$AUTOSTART_AGENT"

note "Code signature"
codesign --verify --deep --strict --verbose=2 "$APP_PATH"
CODESIGN_DETAILS=$(codesign -dv --verbose=4 "$APP_PATH" 2>&1)
echo "$CODESIGN_DETAILS" | sed -n '/^Identifier=/p;/^Authority=/p;/^TeamIdentifier=/p'
TEAM_ID=$(echo "$CODESIGN_DETAILS" | awk -F= '/^TeamIdentifier=/{print $2; exit}')
[[ "$TEAM_ID" == "$EXPECTED_TEAM_ID" ]] || fail "Expected TeamIdentifier $EXPECTED_TEAM_ID, got ${TEAM_ID:-<missing>}"

note "Gatekeeper assessment"
spctl -a -vv -t execute "$APP_PATH"

note "Autostart registration"
AGENT_MATCH=$(launchctl print "gui/$(id -u)/$AUTOSTART_AGENT_LABEL" 2>/dev/null || true)
if [[ -n "$AGENT_MATCH" ]]; then
  echo "$AGENT_MATCH" | sed -n '1,80p'
else
  BTM_MATCH=$(sfltool dumpbtm 2>/dev/null | grep -A 12 -B 4 "$BUNDLE_ID" || true)
  [[ -n "$BTM_MATCH" ]] || fail "No LaunchAgent or Background Item found for $BUNDLE_ID"
  echo "$BTM_MATCH"
  echo "$BTM_MATCH" | grep -q "enabled" || fail "Background Item for $BUNDLE_ID is not enabled"
fi

note "Running process"
PIDS=$(pgrep -x "$PROCESS_NAME" || true)
if [[ -n "$PIDS" ]]; then
  INSTALLED_PROCESS_COUNT=0
  for pid in $PIDS; do
    PROCESS_STATE=$(ps -p "$pid" -o stat= | awk '{print $1}')
    PROCESS_COMMAND=$(ps -p "$pid" -o command=)
    ps -p "$pid" -o pid,ppid,lstart,etime,stat,command
    if process_uses_installed_executable "$pid" "$PROCESS_COMMAND"; then
      ((INSTALLED_PROCESS_COUNT += 1))
    else
      fail "$PROCESS_NAME pid $pid is not running from installed app path $EXECUTABLE"
    fi
    [[ "$PROCESS_STATE" != *T* ]] || fail "$PROCESS_NAME pid $pid is stopped/suspended"
  done
  if [[ "$INSTALLED_PROCESS_COUNT" -gt 1 ]]; then
    fail "Expected one $PROCESS_NAME process from installed app path $EXECUTABLE, found $INSTALLED_PROCESS_COUNT"
  fi
else
  print_recent_startup_log
  fail "$PROCESS_NAME is not running. If you just installed the app, log out/in or reboot, then run this verifier again."
fi

print_recent_startup_log
verify_expected_build_commit

note "Verification passed"
