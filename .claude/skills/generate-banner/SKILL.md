---
name: generate-banner
description: Regenerate www/public/better-resource-monitor.png using the exact same tray icon renderer as the app, stacked with an alert example.
argument-hint: "[optional: out-path]"
disable-model-invocation: true
allowed-tools:
  - Bash(cargo *)
  - Bash(sips *)
---

Regenerate the marketing banner image using the shared tray renderer (single source of truth).

Rules:
- Use the Rust CLI example in `src-tauri/examples/render_tray_icon.rs`.
- Do not reimplement rendering logic in scripts; the app and banner must share the same renderer in `src-tauri/src/tray_render.rs`.

## Default banner (matches repo README)

This generates `830x86`: the current non-alert banner on the first row, and the alert-colored version directly below it on the second row.

If `$ARGUMENTS` is provided, use it as the output path; otherwise write to `www/public/better-resource-monitor.png`.

```bash
cargo run --manifest-path src-tauri/Cargo.toml --example render_tray_icon -- \
  --preset macos \
  --scale 0.6666667 \
  --cpu 41 --mem 58 --gpu 27 \
  --down "1.5 MB" --up "0.2 MB" \
  --alert-cpu 93 --alert-mem 96 --alert-gpu 91 \
  --alert-down "12 MB" --alert-up "3.1 MB" \
  --show-alerts false \
  --include-alert-row true \
  --out "${ARGUMENTS:-www/public/better-resource-monitor.png}"
```

Verify dimensions:

```bash
sips -g pixelWidth -g pixelHeight "${ARGUMENTS:-www/public/better-resource-monitor.png}"
```

## High-res export

Use a larger scale factor and write to a temporary file:

```bash
cargo run --manifest-path src-tauri/Cargo.toml --example render_tray_icon -- \
  --preset macos \
  --scale 2.0 \
  --cpu 41 --mem 58 --gpu 27 \
  --down "1.5 MB" --up "0.2 MB" \
  --alert-cpu 93 --alert-mem 96 --alert-gpu 91 \
  --alert-down "12 MB" --alert-up "3.1 MB" \
  --show-alerts false \
  --include-alert-row true \
  --out /tmp/better-resource-monitor@2x.png
```
