# Better Resource Monitor roadmap

A lightweight, sandbox-friendly system monitor for the macOS menu bar.

This roadmap reflects the repo as it exists today, including the current `intel-support` branch work. It is intentionally conservative: if something is only partially wired, manually tested, or implied by branch changes, it is described that way.

## Platforms

| Platform | Status | Notes |
| --- | --- | --- |
| macOS 11+ | ✅ Shipping | Primary product. Tauri tray app, DMG packaging, Mac App Store configuration, and the main README all target macOS first. |
| Apple Silicon Macs | ✅ Shipping | Public download/install docs and release links clearly support Apple Silicon today. |
| Intel Macs | In Progress | This branch is explicitly focused on Intel support. The repo now shows active x86_64/macOS work and native macOS autostart changes, but Intel distribution is not yet documented as fully released. |
| Linux | Experimental | Linux-specific code paths and packaging config exist, including tray/theme handling and Linux bundle metadata, but there is still no clear supported release story or user documentation. |
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
| Start at Login tied to real macOS Login Items state | In Progress | The branch adds a dedicated autostart module, native ServiceManagement integration, Objective-C bridge code, status mapping tests, and a manual macOS test plan. README copy now describes this as current work, but it still depends on manual verification. |
| Intel compatibility | In Progress | The branch name, local x86_64/macOS build artifacts, and broader compatibility framing point to active work, but the public docs do not yet claim Intel support as fully shipped. |
| Marketing website | Partial | The repo includes a real Astro marketing site under `www/`, but roadmap and product docs inside the app repo are still fairly lean. |
| Linux packaging/productization | Experimental | Linux bundle metadata exists in Tauri config, but there is no supported installer, release process, or end-user positioning for Linux yet. |
| Release automation and packaging polish | Planned | DMG and App Store packaging are configured, but there is no strong evidence here of a finished automated multi-platform release pipeline. |

## Near-term focus

- Finish validating the native macOS `Start at Login` flow, especially approval-required and fallback behavior.
- Land Intel support with public documentation only when distribution and testing are genuinely ready.
- Keep the tray renderer lean under long-running use.
- Decide whether Linux stays experimental or gets a real supported release path.
- Continue polishing packaging and release workflow without giving up the sandboxed, no-root model.
