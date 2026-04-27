# AGENTS.md

## Repo Shape
- This repo has two codebases: the Tauri app at the root/`src-tauri/` and the Astro marketing site in `www/`. `pnpm-workspace.yaml` only includes `www`.
- The Tauri app has no web frontend bundle: `src-tauri/tauri.conf.json` sets `"frontendDist": null` and `"windows": []`. The real app entrypoint is Rust tray/menu code in `src-tauri/src/lib.rs`, with rendering in `src-tauri/src/tray_render.rs`.

## Commands
- Root app commands: `pnpm install`, `pnpm tauri dev`, `pnpm tauri build`.
- Rust checks run from `src-tauri/`: `cargo fmt`, `cargo test`, `cargo test test_name`, `cargo clippy`, `cargo llvm-cov --lib --html --output-dir coverage/`.
- Site commands run from root: `pnpm dev:www`, `pnpm build:www`, `pnpm preview:www`, `pnpm build:screenshots`.
- Do not guess `pnpm test` or `pnpm lint` at repo root; they do not exist. For site validation, use `pnpm build:www`.

## Wiring And Gotchas
- `www/src/content.config.ts` loads the localized root `README*.md` files into the website home body. README edits change the site, not just GitHub docs.
- Website locales come from `www/src/lib/marketing-copy.json`; app menu translations are separate in `src-tauri/src/i18n.rs`. Keep both sides aligned when changing locales. Current locale keys are `en`, `es`, `pt-br`, and `zh-cn`; screenshot language tags are mapped separately in `www/src/lib/translations.ts`.
- `www/src/pages/images/[id].png.ts` is `prerender = true` build-time code. It depends on native `@resvg/resvg-js`, which `www/astro.config.mjs` externalizes for SSR. Treat image generation as Node build-time logic, not Cloudflare runtime logic.
- `www/src/lib/renderer.ts` fetches fonts from Google during build. `pnpm build:www` and `pnpm build:screenshots` need network access.
- `pnpm build:screenshots` first builds `www/`, then copies generated PNGs from `www/dist/images` into `images/appstore/<lang>/`.
- If you change tray visuals, regenerate `www/public/better-resource-monitor.png` from `src-tauri/examples/render_tray_icon.rs` (or the repo-local `generate-banner` skill). Do not hand-redraw marketing tray art separately from the app renderer.

## Release Notes
- Manual version bumps touch `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`. `.github/workflows/release.yml` is the executable reference.
- Hardcoded DMG download links also exist in `README.es.md`, `README.pt-br.md`, and `README.zh-cn.md`; the current release workflow only rewrites `README.md`, so localized READMEs are easy to leave stale.
- App Store packaging is driven by `scripts/package-for-store.sh` plus `src-tauri/tauri.appstore.conf.json`, `src-tauri/Entitlements.appstore.plist`, `scripts/.env`, and `src-tauri/embedded.provisionprofile`.

## Trust Code Over Docs
- Prefer `Cargo.toml`, `tauri*.json`, scripts, and source over prose docs. Current drift: `CLAUDE.md` still says 7 app languages; `docs/app-store-guide.md` mentions a repo Cargo `apple-app-store` feature that does not exist; READMEs and FAQ copy say Ventura 13+ while `src-tauri/tauri.conf.json` sets `minimumSystemVersion` to `11.0`.
