# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Better Resource Monitor — a lightweight macOS/Linux menu bar system monitor built with Rust + Tauri v2. Companion marketing website in `www/` built with Astro + Cloudflare Pages.

## Commands

```bash
# Tauri app
pnpm install                    # install all deps (root + www workspace)
pnpm tauri dev                  # run app with hot-reload
pnpm tauri build                # production build

# Rust (run from src-tauri/)
cargo fmt                       # format before committing
cargo test                      # unit tests
cargo clippy                    # lint

# Website (from root)
pnpm dev:www                    # astro dev server
pnpm build:www                  # build for Cloudflare Pages
pnpm preview:www                # build + wrangler local preview
```

## Architecture

### Tauri App (`src-tauri/src/`)

- **`lib.rs`** — App entry: Tauri setup, menu construction, settings persistence (tauri-plugin-store), and the main monitoring loop (1s interval thread reading CPU/Memory/GPU/Network via `sysinfo`)
- **`gpu.rs`** — Platform-specific GPU monitoring. macOS: IOAccelerator public IOKit API (`IOServiceMatching("IOAccelerator")` → `PerformanceStatistics` → `Device Utilization %`). Linux: NVML for NVIDIA GPUs
- **`tray_render.rs`** — Renders the menu bar icon: composites SVG icons (Phosphor fill variants in `assets/icons/`) + percentage text onto an `ImageBuffer`. Alert color (#D14715) at >90%. Platform-specific `Sizing` constants (macOS vs Linux)
- **`tests.rs`** — Unit tests for tray rendering

Key patterns:
- `#[cfg(target_os = "macos")]` / `#[cfg(target_os = "linux")]` for platform splits
- Settings stored via `tauri-plugin-store` as JSON (visibility toggles per metric, autostart)
- macOS runs as accessory app (no dock icon): `ActivationPolicy::Accessory`
- Hysteresis thresholds on metric changes to avoid excessive tray redraws

### Website (`www/`)

- **`src/pages/images/[id].png.ts`** — Dynamic image generation endpoint (`prerender = true`, runs at build time in Node.js). Generates App Store screenshots (2778×1284) and OG images (1200×630)
- **`src/lib/renderer.ts`** — Shared satori + @resvg/resvg-js renderer. Design tokens: `#181818` bg, `#edbc63` accent, `#c5c5c5` dim text. Fetches JetBrains Mono from Google Fonts at build time
- **`src/lib/competitors.ts`** — Data for comparison pages
- **`src/pages/*.astro`** — Landing, FAQ, comparison pages (vs-stats, vs-istat-menus, vs-eul), privacy/terms
- **`src/layouts/Layout.astro`** — Base layout with `ogImage` prop for per-page OG images

`@resvg/resvg-js` uses a native `.node` addon — it's in `vite.ssr.external` in astro config so it doesn't get bundled. Image endpoints use `prerender = true` to run in Node.js at build time, not on Cloudflare Workers.

## Code Style

- Rust: `cargo fmt` required. Standard Rust conventions
- TypeScript/Astro: 2-space indentation
- Release profile: `opt-level = "z"`, `lto = "thin"`, `strip = "symbols"` for minimal binary size
