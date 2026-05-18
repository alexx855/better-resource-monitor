# macOS Autostart Handoff

This branch fixes the Intel macOS Ventura case where Better Resource Monitor is
enabled in Login Items / Background Items but does not visibly start after login.
The reproduced failure was a login-started process stuck in `T` state before app
setup ran; reopening the app or sending `SIGCONT` resumed it and made the tray
appear.

On the current affected machine, the installed TestFlight app also reproduces a
stale preference/system-state split: the app container settings contain
`"autostart": true`, while macOS has no durable Better Resource Monitor
LaunchAgent or Background Task Management entry. That installed app still lacks:

```text
Contents/Library/LaunchAgents/dev.alexpedersen.better-resource-monitor.autostart.plist
```

The currently installed `/Applications` app is a valid old TestFlight build, not
the fixed build:

```text
CFBundleVersion=202605151931
TeamIdentifier=G76YQZM2FU
Gatekeeper source=Testflight
embedded build marker=1c51c4a61981
```

## Current Fix Shape

The current implementation uses a bundled `SMAppService` LaunchAgent instead of
the old `SMAppService.mainAppService` login item path. The LaunchAgent plist is
embedded at:

```text
Contents/Library/LaunchAgents/dev.alexpedersen.better-resource-monitor.autostart.plist
```

The app removes the old main-app login item during enable/repair flows and then
registers the bundled LaunchAgent only when it is not already enabled. This is
important because registering a LaunchAgent can immediately bootstrap it; normal
startup must not unregister/re-register an already-enabled agent and spawn a
second instance.

The source, CI, installer, verifier, and App Store packaging path all validate
the LaunchAgent metadata through:

```text
scripts/verify-macos-autostart-agent-plist.sh
```

## Build Commit

Run the signed Intel build from the merged fix commit. If the PR is
squash-merged or the workflow is dispatched from another ref, use that actual
signed-build commit. The app logs this short commit through `BRM_BUILD_COMMIT`,
so post-login verification should use the same short value:

```bash
EXPECTED_BUILD_COMMIT=<signed-build-short-commit>
```

## Signed Artifact Path

After this PR is merged, run the manual `Signed macOS Build` workflow on the
merged commit. It builds `--bundles app --target x86_64-apple-darwin`, verifies
the signed app with `codesign` and `spctl`, then uploads:

```text
Better-Resource-Monitor-x86_64.app.zip
Better-Resource-Monitor-x86_64.app.zip.sha256
```

## Install

Download both files into the same directory, then install the signed app:

```bash
EXPECTED_BUILD_COMMIT=<signed-build-short-commit> scripts/install-signed-macos-app.sh ~/Downloads/Better-Resource-Monitor-x86_64.app.zip
```

The installer checks the optional `.sha256` sidecar, bundle id, version,
`x86_64` architecture, bundled autostart LaunchAgent metadata, code signature,
and Gatekeeper before replacing the app in `/Applications`.

For TestFlight/App Store packaging, `scripts/package-for-store.sh` also asserts
that the signed app keeps the expected team/application identifier entitlements
and `com.apple.security.app-sandbox=true` before upload.

## TestFlight Local State

If the fix is shipped through TestFlight, verify that TestFlight actually
installed the fixed build before using the app as proof. The current TestFlight
databases store app bundle rows in `ZTFAPPBUNDLEMODEL`:

```bash
for db in "$HOME/Library/Containers/com.apple.TestFlight/Data/Library/Application Support/TestFlight/TestFlight.sqlite" \
  "$HOME/Library/Containers/com.apple.TestFlight.ServiceExtension/Data/Library/Application Support/TestFlightServiceExtension/TestFlight.sqlite"; do
  [ -f "$db" ] || continue
  echo "db=$db"
  sqlite3 "$db" <<'SQL'
select
  b.ZSHORTVERSION,
  b.ZBUNDLEVERSION,
  b.ZINSTALLSTATUSRAW,
  case
    when b.ZBUNDLEURL is null or b.ZBUNDLEURL = '' then 'no_url'
    else 'has_url'
  end
from ZTFAPPBUNDLEMODEL b
left join ZTFAPPMODEL a on b.ZAPP = a.Z_PK
where b.ZBUNDLEID = 'dev.alexpedersen.better-resource-monitor'
  or a.ZBUNDLEID = 'dev.alexpedersen.better-resource-monitor'
order by b.ZBUNDLEVERSION;
SQL
done
```

Only a row for the fixed build with an installed status is useful evidence. A
newer metadata row without a bundle URL, or an installed row from an older build,
does not satisfy the acceptance gate.

## Verify After Login

Enable Start at Login in the app menu, then log out and back in. After the new
login session starts, run:

```bash
EXPECTED_VERSION=1.1.3 EXPECTED_BUILD_COMMIT=<signed-build-short-commit> scripts/verify-macos-autostart.sh
```

The verifier requires the installed app to be signed by team `G76YQZM2FU`, the
installed executable to embed the expected build commit, the autostart
LaunchAgent or Background Item to be registered, the running process to come
from the installed `/Applications` bundle and not be stopped/suspended, and the
startup log to contain the expected build commit.

The fix is not proven by local unsigned bundles or green source CI alone. The
acceptance gate is the real signed app installed at
`/Applications/Better Resource Monitor.app` passing the verifier after a fresh
login session, with the menu bar item visible without manually reopening the app.
