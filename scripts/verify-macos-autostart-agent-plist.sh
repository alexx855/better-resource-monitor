#!/usr/bin/env bash
set -euo pipefail

PLIST_PATH="${1:-}"
BUNDLE_ID="${BUNDLE_ID:-dev.alexpedersen.better-resource-monitor}"
PROCESS_NAME="${PROCESS_NAME:-better-resource-monitor}"
AUTOSTART_AGENT_LABEL="${AUTOSTART_AGENT_LABEL:-dev.alexpedersen.better-resource-monitor.autostart}"

fail() {
  echo "FAIL: $*" >&2
  exit 1
}

if [[ -z "$PLIST_PATH" ]]; then
  echo "Usage: $0 <path-to-autostart-agent.plist>" >&2
  exit 2
fi

[[ -f "$PLIST_PATH" ]] || fail "Autostart LaunchAgent not found: $PLIST_PATH"
plutil -lint "$PLIST_PATH"

LABEL=$(/usr/libexec/PlistBuddy -c 'Print :Label' "$PLIST_PATH")
BUNDLE_PROGRAM=$(/usr/libexec/PlistBuddy -c 'Print :BundleProgram' "$PLIST_PATH")
ASSOCIATED_BUNDLE_ID=$(/usr/libexec/PlistBuddy -c 'Print :AssociatedBundleIdentifiers:0' "$PLIST_PATH")
RUN_AT_LOAD=$(/usr/libexec/PlistBuddy -c 'Print :RunAtLoad' "$PLIST_PATH")
KEEP_ALIVE=$(/usr/libexec/PlistBuddy -c 'Print :KeepAlive' "$PLIST_PATH")
SESSION_TYPE=$(/usr/libexec/PlistBuddy -c 'Print :LimitLoadToSessionType' "$PLIST_PATH")

[[ "$LABEL" == "$AUTOSTART_AGENT_LABEL" ]] || fail "Expected LaunchAgent Label $AUTOSTART_AGENT_LABEL, got $LABEL"
[[ "$BUNDLE_PROGRAM" == "Contents/MacOS/$PROCESS_NAME" ]] || fail "Expected BundleProgram Contents/MacOS/$PROCESS_NAME, got $BUNDLE_PROGRAM"
[[ "$ASSOCIATED_BUNDLE_ID" == "$BUNDLE_ID" ]] || fail "Expected AssociatedBundleIdentifiers[0] $BUNDLE_ID, got $ASSOCIATED_BUNDLE_ID"
[[ "$RUN_AT_LOAD" == "true" ]] || fail "Expected RunAtLoad=true, got $RUN_AT_LOAD"
[[ "$KEEP_ALIVE" == "false" ]] || fail "Expected KeepAlive=false, got $KEEP_ALIVE"
[[ "$SESSION_TYPE" == "Aqua" ]] || fail "Expected LimitLoadToSessionType=Aqua, got $SESSION_TYPE"
