# Resource Monitor

The root Tauri app has no web frontend/window: app wiring is in `src-tauri/src/lib.rs`, rendering in `tray_render.rs`. `www/` is a separate Astro marketing site and the only pnpm workspace. Prefer executable config over stale prose about feature names or macOS versions.

## Development

- App: `pnpm dev` / `pnpm build`. Rust checks run in `src-tauri/`: `cargo fmt`, `cargo test [test_name]`, `cargo clippy`; coverage uses `cargo llvm-cov --lib --html --output-dir coverage/`.
- Site: `pnpm dev:www` / `pnpm build:www`. Root has no `test` or `lint` scripts.
- `pnpm build:screenshots` renders tray art, builds the site, and copies `www/dist/images` PNGs to `images/appstore/<lang>/`.

## Cross-surface contracts

- `www/src/content.config.ts` loads root localized `README*.md` into the website; README edits affect the site.
- Locale changes must align `www/src/lib/marketing-copy.json` and app `src-tauri/src/i18n.rs`. The marketing copy's `screenshotLang` fields drive screenshot tags. `scripts/check-locale-parity.mjs` checks structure, not translation quality.
- Image routes in `www/src/pages/images/[id].png.ts` run at build time in Node, not Cloudflare. Preserve native `@resvg/resvg-js` SSR externalization and vendored fonts in `www/src/assets/fonts`; builds need no Google Fonts fetches.
- For tray visual changes, regenerate marketing art through `src-tauri/examples/render_tray_icon.rs`; never hand-redraw `www/public/better-resource-monitor*.png` independently.
- App Store builds use repo feature `app-store`; sysinfo uses dependency feature `apple-app-store`. Keep GPU sampling through public IOAccelerator in `src-tauri/src/gpu.rs`; no GPU stub or obsolete private IOReport workaround.

## Website decisions

- Do not add skip-to-content unless persistent navigation/repeated chrome is added before main content.
- Follow `DESIGN.md` and `www/src/layouts/Layout.astro` for links: all text links, including language/footer navigation, use the same brand-filled style (white text, no underline). `a:has(img)` badges remain transparent without shadow; `.comparison-card` retains its card treatment.

## Releases

- `.github/workflows/release.yml` owns version-bump PR/merge, tagging, GitHub release, then Direct Download dispatch. Manual bumps touch `package.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, and `src-tauri/tauri.conf.json`.
- `.github/workflows/testflight.yml` is the only App Store upload path, using trusted `main` scripts/config and CI provisioning; no local packaging fallback. It owns monotonic `CFBundleVersion` via run number and UTC timestamp floor.
- TestFlight uploads are optional and manually dispatched from trusted `main`; PRs and releases do not launch them. Use `source_ref` to select `main`, a `v*` tag, or `refs/pull/<number>/head`. PRs have no TestFlight merge requirement.
