# macOS Autostart

Better Resource Monitor is a greenfield TestFlight/App Store app. macOS
autostart has one supported implementation: the main app registers itself with
`SMAppService.mainAppService()`.

## Runtime Contract

- The Start at login menu item reflects the live `SMAppService` main-app status.
- TestFlight/App Store builds launch from
  `/Applications/Better Resource Monitor.app`.
- Production app launches require the App Store/TestFlight receipt at
  `Contents/_MASReceipt/receipt`.
- The app bundle must not embed `Contents/Library/LaunchAgents`.
- Local unsigned builds and `tauri dev` do not register the main app as a login
  item.

## Packaging Guardrails

Use the GitHub Actions release and TestFlight workflows for App Store packaging.
Both workflows call the shared `.github/actions/upload-appstore` action. It
builds with `src-tauri/tauri.appstore.conf.json` and the `app-store` Cargo
feature, then verifies the expected team/application identifier entitlements,
`com.apple.security.app-sandbox=true`, no private GPU sampling API references,
and no bundled LaunchAgents.

The old local installed-app autostart verifier has been removed because the
Start at login flow has passed validation. Future autostart changes should be
verified manually from an installed TestFlight/App Store build: enable Start at
login, log out and back in, then confirm the app starts and the menu item still
matches macOS Login Items state.
