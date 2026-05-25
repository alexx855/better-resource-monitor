# macOS Autostart Verification

Better Resource Monitor is a greenfield TestFlight/App Store app. macOS
autostart has one supported implementation: the main app registers itself with
`SMAppService.mainAppService()`.

## Invariants

- Production runtime distribution is TestFlight/App Store only.
- The installed app must live at
  `/Applications/Better Resource Monitor.app`.
- The app bundle must include an App Store/TestFlight receipt at
  `Contents/_MASReceipt/receipt`.
- The app bundle must not embed `Contents/Library/LaunchAgents`.
- A production process with bundle id
  `dev.alexpedersen.better-resource-monitor` exits unless it is launched from
  `/Applications/Better Resource Monitor.app/Contents/MacOS/better-resource-monitor`
  with a receipt.

## Packaging

Use the GitHub Actions release and TestFlight workflows for App Store packaging.
Both workflows call `scripts/package-for-store.sh`. There is no supported local
App Store packaging fallback; keep uploads on GitHub Actions so builds stay tied
to a commit, runner log, and passing checks. The script builds with
`src-tauri/tauri.appstore.conf.json` and the `app-store` Cargo feature, then
verifies the expected team/application identifier entitlements,
`com.apple.security.app-sandbox=true`, and no bundled LaunchAgents.

## Verification

Install the TestFlight/App Store build, enable Start at Login in the app menu,
then log out and back in. After the new login session starts, run:

```bash
EXPECTED_VERSION="<version>" EXPECTED_BUILD_COMMIT="<short-commit>" scripts/verify-macos-autostart.sh
```

The verifier checks the installed `/Applications` app, signature authority,
receipt, embedded build marker, absence of bundled LaunchAgents, enabled main-app
Background Item, exactly one running process from the installed app executable,
and startup log entry for the expected build commit.

Local unsigned builds, local bundle copies, and source-only CI do not satisfy
this gate.
