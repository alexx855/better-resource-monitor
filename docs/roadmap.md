# Better Resource Monitor roadmap

A lightweight, sandbox-friendly system monitor for the macOS menu bar.

This roadmap reflects the current product direction: a greenfield macOS menu bar
app distributed through TestFlight and the Mac App Store.

## Platforms

| Platform | Status | Notes |
| --- | --- | --- |
| macOS 13+ | ✅ Shipping | Primary product. The Tauri tray app, App Store configuration, and public docs target macOS first. |
| Apple Silicon Macs | ✅ Shipping | Supported through the App Store/TestFlight distribution path. |
| Intel Macs | In Progress | Supported by source checks and App Store packaging work; mark fully shipped only after TestFlight/App Store verification passes on Intel hardware. |
| Linux | Experimental | Linux-specific code paths and bundle metadata exist, but there is no supported installer, release process, or end-user positioning. |
| Windows | Not targeted | Tauri can compile cross-platform in theory, but this repo does not ship or document a Windows product. |

## Current product status

| Area | Status | Notes |
| --- | --- | --- |
| Core tray monitoring | ✅ Shipping | CPU, memory, GPU, and network metrics are rendered in the tray app and are central to the current product. |
| Sandboxed, no-root monitoring | ✅ Shipping | Still a core differentiator in the README, and the sysinfo/App Store setup supports that claim. |
| Mac App Store distribution | ✅ Shipping | App Store listing, App Store-specific config, and sandbox entitlements are present in the repo. |
| Localized UI | ✅ Shipping | English, Spanish, Portuguese (Brazil), and Simplified Chinese are present, and localization behavior is covered by tests. |
| Persistent tray preferences | ✅ Shipping | Visibility toggles and alert-color preference are stored locally through the Tauri store plugin. |
| Alert-color tray states | ✅ Shipping | Implemented in the renderer and covered by tests. |
| Tray rendering performance work | Partial | Hysteresis thresholds, buffer reuse, and renderer-focused tests exist, but this still reads like ongoing optimization rather than a finished performance story. |
| Start at Login tied to real macOS Login Items state | In Progress | TestFlight/App Store builds use `SMAppService.mainAppService()` and verify the main-app Background Item after login. |
| Intel compatibility | In Progress | Keep Intel marked in progress until the signed TestFlight/App Store app passes verification on Intel hardware. |
| Marketing website | Partial | The repo includes a real Astro marketing site under `www/`, but roadmap and product docs inside the app repo are still fairly lean. |
| Linux packaging/productization | Experimental | Linux bundle metadata exists in Tauri config, but there is no supported installer, release process, or end-user positioning for Linux yet. |
| Release automation and packaging polish | Planned | App Store packaging is configured; keep polishing the TestFlight/App Store release path. |

## Near-term focus

- Finish validating the native macOS `Start at Login` flow through the TestFlight/App Store path.
- Mark Intel support as shipped only when signed TestFlight/App Store verification passes on Intel hardware.
- Keep the tray renderer lean under long-running use.
- Decide whether Linux stays experimental or gets a real supported release path.
- Continue polishing the TestFlight/App Store packaging workflow without giving up the sandboxed, no-root model.
