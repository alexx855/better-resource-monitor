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
```

## Architecture

**Rust app** (`src-tauri/src/lib.rs`) - app lifecycle + sampling + tray wiring:
- System monitoring via `sysinfo` crate (CPU, memory, network)
- Dynamic tray icon rendering via shared renderer (`src-tauri/src/tray_render.rs`) using `rusttype` (text) + `resvg` (SVG icons)
- Menu bar only (no window) - uses `ActivationPolicy::Accessory` to hide dock icon
- Background thread updates tray icon every ~1s
- Toggle visibility of CPU/memory/network via right-click menu
- Autostart support via `tauri-plugin-autostart`

**Tray renderer (single source of truth)** (`src-tauri/src/tray_render.rs`)
- Pure rendering logic (layout + SVG rasterization + text baseline + RGBA buffer output)
- Used by both the running app and the banner generator CLI

**Banner generator CLI** (`src-tauri/src/bin/render_tray_icon.rs`)
- Renders a PNG using the exact same implementation as the app

**Frontend** (`src/`) - minimal, exists only to satisfy Tauri build requirements. No actual UI.

## Key Dependencies

- `sysinfo` - cross-platform system info
- `rusttype` + `image` - text rendering onto tray icon
- `resvg` + `tiny-skia` - SVG icon rendering
- `tauri-plugin-autostart` - launch at login

## Verification

- Always add a final verification step for changes (run a relevant command or manual check).
- On Linux, minimize tray icon updates to avoid compositor resource accumulation (cursor lag).

## Regenerate Marketing Banner

The repo includes a generator for `www/public/better-resource-monitor.png` that renders using the same code path as the app tray icon renderer.

Generate the banner (830x43) from the macOS sizing preset scaled to 2/3, with alert colors disabled (avoids orange when values exceed threshold):

```bash
cargo run --manifest-path src-tauri/Cargo.toml --bin render_tray_icon -- \
  --preset macos \
  --scale 0.6666667 \
  --cpu 45 --mem 99 --gpu 78 \
  --down "1.5 MB" --up "0.2 MB" \
  --show-alerts false \
  --out www/public/better-resource-monitor.png
```

Verify output dimensions:

```bash
sips -g pixelWidth -g pixelHeight www/public/better-resource-monitor.png
```

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
