---
name: generate-banner
description: Regenerate www/public/better-resource-monitor.png using the exact same tray icon renderer as the app.
argument-hint: "[optional: out-path]"
disable-model-invocation: true
allowed-tools:
  - Bash(cargo *)
  - Bash(sips *)
---

Regenerate the marketing banner image using the shared tray renderer (single source of truth).

Rules:
- Use the Rust CLI in `src-tauri/src/bin/render_tray_icon.rs`.
- Do not reimplement rendering logic in scripts; the app and banner must share the same renderer in `src-tauri/src/tray_render.rs`.

## Default banner (matches repo README)

This generates `830x43` with the macOS sizing preset scaled to 2/3, and disables alert coloring so high values do not turn orange.

If `$ARGUMENTS` is provided, use it as the output path; otherwise write to `www/public/better-resource-monitor.png`.

```bash
cargo run --manifest-path src-tauri/Cargo.toml --bin render_tray_icon -- \
  --preset macos \
  --scale 0.6666667 \
  --cpu 45 --mem 99 --gpu 78 \
  --down "1.5 MB" --up "0.2 MB" \
  --show-alerts false \
  --out "${ARGUMENTS:-www/public/better-resource-monitor.png}"
```

Verify dimensions:

```bash
sips -g pixelWidth -g pixelHeight "${ARGUMENTS:-www/public/better-resource-monitor.png}"
```

## High-res export

Use a larger scale factor and write to a temporary file:

```bash
cargo run --manifest-path src-tauri/Cargo.toml --bin render_tray_icon -- \
  --preset macos \
  --scale 2.0 \
  --cpu 45 --mem 99 --gpu 78 \
  --down "1.5 MB" --up "0.2 MB" \
  --show-alerts false \
  --out /tmp/better-resource-monitor@2x.png
```
