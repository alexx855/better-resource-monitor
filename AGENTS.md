# AGENTS.md

## Repo Shape
- This repo has two codebases: the Tauri app at the root/`src-tauri/` and the Astro marketing site in `www/`. `pnpm-workspace.yaml` only includes `www`.
- The Tauri app has no web frontend bundle: `src-tauri/tauri.conf.json` sets `"frontendDist": null` and `"windows": []`. `src-tauri/src/main.rs` only calls `better_resource_monitor_lib::run()`; the real app wiring is tray/menu code in `src-tauri/src/lib.rs`, with rendering in `src-tauri/src/tray_render.rs`.

## Commands
- Root app commands: `pnpm install`, `pnpm tauri dev` or `pnpm dev`, `pnpm tauri build` or `pnpm build`.
- Rust checks run from `src-tauri/`: `cargo fmt`, `cargo test`, `cargo test test_name`, `cargo clippy`, `cargo llvm-cov --lib --html --output-dir coverage/`.
- Site commands run from root via workspace filters: `pnpm dev:www`, `pnpm build:www`, `pnpm preview:www`, `pnpm build:screenshots`.
- Do not guess `pnpm test` or `pnpm lint` at repo root; they do not exist. For site validation, use `pnpm build:www`.

## Wiring And Gotchas
- `www/src/content.config.ts` loads the localized root `README*.md` files into the website home body. README edits change the site, not just GitHub docs.
- Website locales come from `www/src/lib/marketing-copy.json`; app menu translations are separate in `src-tauri/src/i18n.rs`. Keep both sides aligned when changing locales. Current locale keys are `en`, `es`, `pt-br`, and `zh-cn`; screenshot language tags are mapped separately in `www/src/lib/translations.ts`.
- `www/src/pages/images/[id].png.ts` is `prerender = true` build-time code. It depends on native `@resvg/resvg-js`, which `www/astro.config.mjs` externalizes for SSR. Treat image generation as Node build-time logic, not Cloudflare runtime logic.
- `www/src/lib/renderer.ts` fetches fonts from Google during build. `pnpm build:www` and `pnpm build:screenshots` need network access.
- `pnpm build:screenshots` first builds `www/`, then copies generated PNGs from `www/dist/images` into `images/appstore/<lang>/`.
- If you change tray visuals, regenerate marketing tray art from `src-tauri/examples/render_tray_icon.rs`; do not hand-redraw `www/public/better-resource-monitor*.png` separately from the app renderer.
- macOS App Store builds use the repo Cargo feature `app-store`, which disables GPU sampling in `src-tauri/src/gpu.rs`. `sysinfo` separately enables its dependency feature `apple-app-store` in `Cargo.toml`.

## Website UX Decisions
- Do not add a skip-to-content link to the current marketing site by default. Pages start directly with their main content, and the extra localized copy, CSS, focus target, and DOM add complexity without clear value here. Revisit only if persistent navigation or other repeated chrome is added before the content.

## Release Notes
- Manual version bumps touch `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`. `.github/workflows/release.yml` is the executable reference.
- Hardcoded DMG download links also exist in `README.es.md`, `README.pt-br.md`, and `README.zh-cn.md`; the current release workflow only rewrites `README.md`, so localized READMEs are easy to leave stale.
- App Store packaging is driven by GitHub Actions plus `scripts/package-for-store.sh`, `scripts/setup-appstore-signing.sh`, `src-tauri/tauri.appstore.conf.json`, `src-tauri/Entitlements.appstore.plist`, and `src-tauri/embedded.provisionprofile`. `.github/workflows/testflight.yml` uploads build-number-only TestFlight builds without repo version changes; `.github/workflows/release.yml` bumps/tags/uploads and then creates the GitHub release. Local `scripts/.env` packaging remains a fallback only.

## Trust Code Over Docs
- Prefer `Cargo.toml`, `tauri*.json`, scripts, and source over prose docs. Current drift: `docs/app-store-guide.md` still shows the old repo feature name `apple-app-store` and macOS `minimumSystemVersion` `11.0`; current executable config uses feature `app-store` and minimum macOS `13.0`.
