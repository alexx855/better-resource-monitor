# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Better Resource Monitor - A lightweight menu bar/tray system monitor for macOS and Linux. Built with Tauri 2 + Rust, renders CPU/memory/GPU/network stats directly in the menu bar tray icon.

## Commands

```bash
# Development (runs frontend + Rust backend with hot reload)
pnpm tauri dev

# Production build
pnpm tauri build

# Rust-only commands (from src-tauri/)
cargo build
cargo check
cargo clippy

# Tests
cargo test --manifest-path src-tauri/Cargo.toml

# Test coverage (requires cargo-llvm-cov)
cd src-tauri && cargo llvm-cov --lib --html --output-dir coverage/

# Landing page (Astro on Cloudflare Pages)
pnpm --filter www dev
pnpm --filter www build

# Regenerate download badges (WebP, saved to www/public/badges/)
node www/generate-badges.mjs
```

## Architecture

**Rust app** (`src-tauri/src/lib.rs`) - app lifecycle + sampling + tray wiring:
- System monitoring via `sysinfo` crate (CPU, memory, network)
- Dynamic tray icon rendering via shared renderer (`src-tauri/src/tray_render.rs`) using `rusttype` (text) + `resvg` (SVG icons)
- Menu bar only (no window) - uses `ActivationPolicy::Accessory` to hide dock icon
- Background thread updates tray icon every ~1s
- Toggle visibility of CPU/memory/network via right-click menu
- Settings persistence via `tauri-plugin-store`
- Autostart support via `tauri-plugin-autostart`

**Tray renderer (single source of truth)** (`src-tauri/src/tray_render.rs`)
- Pure rendering logic (layout + SVG rasterization + text baseline + RGBA buffer output)
- Used by both the running app and the banner generator CLI
- Pre-caches SVG icons at init in 3 colors: white, black, alert orange (#D14715)
- Two sizing presets: `SIZING_MACOS` and `SIZING_LINUX`, scalable via `Sizing::scaled()`
- Buffer reuse: monitoring thread owns persistent `render_buffer`, critical for preventing Linux compositor texture leaks

**GPU monitoring** (`src-tauri/src/gpu.rs`) - unified `GpuSampler` interface:
- **macOS**: IOAccelerator via public IOKit APIs for Apple Silicon device utilization
- **Linux**: NVML via `nvml-wrapper` for NVIDIA utilization
- GPU menu item only shown when hardware is detected; missing GPU doesn't prevent app from running

**Banner generator CLI** (`src-tauri/examples/render_tray_icon.rs`)
- Renders a PNG using the exact same `TrayRenderer` as the app

**Tests** (`src-tauri/src/tests.rs`)
- Unit tests for value capping, hysteresis, speed formatting, SVG rendering, buffer reuse, alert colors

**Frontend** (`src/`) - minimal, exists only to satisfy Tauri build requirements. No actual UI.

**Landing page** (`www/`) - Astro 5 site deployed to Cloudflare Pages. Contains homepage, privacy policy, terms.

## Platform-Specific Patterns

All platform logic uses `#[cfg(target_os = "...")]` — no custom Cargo features.

- **macOS**: `ActivationPolicy::Accessory`, template icon mode (OS handles light/dark), IOAccelerator GPU
- **Linux**: gsettings/env var theme detection, `AtomicBool` polling for light/dark, NVML GPU, hysteresis throttling

## Update Throttling (Hysteresis)

To mitigate Linux compositor texture leaks, tray icon only updates when:
- CPU/MEM/GPU changes by ≥2.0%
- Network changes by ≥50KB/s OR ≥2s since last update

This logic lives in `should_update()` in `lib.rs`.

## Key Dependencies

- `sysinfo` - cross-platform system info
- `rusttype` + `image` - text rendering onto tray icon
- `resvg` + `tiny-skia` - SVG icon rendering
- `font-kit` - system font loading
- `tauri-plugin-autostart` - launch at login
- `tauri-plugin-store` - settings persistence
- macOS: `core-foundation` (IOAccelerator FFI)
- Linux: `nvml-wrapper` (NVIDIA GPU)

## Release Profile

Binary size is optimized: `opt-level = "z"`, `lto = "thin"`, `strip = "symbols"`, `panic = "abort"`.

## CI/CD

`.github/workflows/release.yml` - manual dispatch with version bump (patch/minor/major):
1. Auto-increments version across `package.json`, `tauri.conf.json`, `Cargo.toml`
2. Builds for macOS (aarch64) and Linux (amd64) with code signing + notarization on macOS
3. Creates GitHub release with DMG/DEB artifacts

## Verification

- Always add a final verification step for changes (run a relevant command or manual check).
- On Linux, minimize tray icon updates to avoid compositor resource accumulation (cursor lag).

## Regenerate Marketing Banner

The repo includes a generator for `www/public/better-resource-monitor.png` that renders using the same code path as the app tray icon renderer.

Generate the banner (2488x128) from the macOS sizing preset at 2x scale, with alert colors disabled (avoids orange when values exceed threshold). Higher resolution needed for crisp App Store screenshots:

```bash
cargo run --manifest-path src-tauri/Cargo.toml --example render_tray_icon -- \
  --preset macos \
  --scale 2.0 \
  --cpu 45 --mem 99 --gpu 78 \
  --down "1.5 MB" --up "0.2 MB" \
  --show-alerts false \
  --out www/public/better-resource-monitor.png
```

Verify output dimensions:

```bash
sips -g pixelWidth -g pixelHeight www/public/better-resource-monitor.png
```

## Regenerate Marketing Images

Builds the Astro site (which prerenders App Store images at 2778×1284 and OG images at 1200×630 via satori + resvg-js) and copies them:

```bash
pnpm --filter www build
cp www/dist/images/{simplicity,performance,privacy}.png images/appstore/
mkdir -p images/og
cp www/dist/images/og-*.png images/og/
```

Source template: `www/src/pages/images/[id].png.ts`
Shared renderer: `www/src/lib/renderer.ts`
Output: `images/appstore/{simplicity,performance,privacy}.png` (2778×1284), `images/og/og-{index,faq,privacy,terms}.png` (1200×630)

## Known Issues

### Linux: Cursor Lag After Extended Use (Ubuntu/GNOME Wayland)

**Symptom**: Cursor lag and system slowdown after 2-4 hours of app running. Lag persists even after app exits. Can freeze video playback overnight.

**Root Cause**: Bug in Ubuntu's `gnome-shell-extension-appindicator` (Ubuntu Bug #2130726). The extension leaks GPU textures when tray icons update frequently. The leak occurs in GNOME Shell's compositor memory, not our application process.

**Status**: Waiting for Tauri upstream fix via KSNI migration

**Track These PRs** (click to view progress):
- [Tauri PR #12319 - Add linux-ksni feature](https://github.com/tauri-apps/tauri/pull/12319)
- [tray-icon PR #201 - Replace libappindicator with ksni](https://github.com/tauri-apps/tray-icon/pull/201)
- [Tauri Issue #11293 - Use ksni for tray icons](https://github.com/tauri-apps/tauri/issues/11293)

**Technical Details**:
- Current stack: Tauri → libappindicator → D-Bus StatusNotifierItem → ubuntu-appindicators extension → Mutter (Wayland compositor)
- libappindicator is abandoned (last meaningful commit ~15 years ago) and doesn't properly manage icon lifecycle
- The extension creates GPU textures for each icon update but never releases old ones

**Workarounds**:
- Disable ubuntu-appindicators extension (loses all tray icon functionality)
- Use KDE Plasma instead of GNOME (handles StatusNotifierItem natively without leak)
- Wait for Tauri 2.x with KSNI support

**References**:
- https://bugs.launchpad.net/ubuntu/+source/gnome-shell-extension-appindicator/+bug/2130726
- https://github.com/tauri-apps/tauri/issues/11293
