# macOS Start at login

[`macos_autostart.rs`](../src-tauri/src/macos_autostart.rs) registers the bundled
app with `SMAppService.mainAppService()`. The menu reflects the live macOS
registration status, refreshed on the monitoring cadence (every two seconds by
default) and immediately after a menu action. Pending approval remains unchecked
until macOS reports the service enabled. Raw development binaries do not register
a login item.

App bundles must not contain `Contents/Library/LaunchAgents`.
[CI](../.github/workflows/ci.yml) checks this packaging requirement.

To verify a change, install the exact TestFlight build, enable **Start at login**,
log out and back in, and confirm the app starts and its menu agrees with macOS
Login Items. Also change the item in System Settings while the app is running:
confirm approval checks the menu item and external disabling unchecks it within
one monitoring interval, then confirm the next menu click performs the displayed
action. Use the [TestFlight workflow](../.github/workflows/testflight.yml)
to upload the build.
