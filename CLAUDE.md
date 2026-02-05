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

**Single-file Rust app** (`src-tauri/src/lib.rs`) - all logic in one file:
- System monitoring via `sysinfo` crate (CPU, memory, network)
- Dynamic tray icon rendering with `rusttype` (text) + `resvg` (SVG icons)
- Menu bar only (no window) - uses `ActivationPolicy::Accessory` to hide dock icon
- Background thread updates tray icon every ~1s
- Toggle visibility of CPU/memory/network via right-click menu
- Autostart support via `tauri-plugin-autostart`

**Frontend** (`src/`) - minimal, exists only to satisfy Tauri build requirements. No actual UI.

## Key Dependencies

- `sysinfo` - cross-platform system info
- `rusttype` + `image` - text rendering onto tray icon
- `resvg` + `tiny-skia` - SVG icon rendering
- `tauri-plugin-autostart` - launch at login

## Verification

- Always add a final verification step for changes (run a relevant command or manual check).
- On Linux, minimize tray icon updates to avoid compositor resource accumulation (cursor lag).
