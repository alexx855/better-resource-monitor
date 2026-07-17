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
- Website locales come from `www/src/lib/marketing-copy.json`; app menu translations are separate in `src-tauri/src/i18n.rs`. Keep both sides aligned when changing locales. Current locale keys are `en`, `es`, `pt-br`, and `zh-cn`; screenshot language tags are mapped separately in `www/src/lib/translations.ts`. `scripts/check-locale-parity.mjs` (run in CI) enforces the structural half of this; translated wording still needs human review.
- `www/src/pages/images/[id].png.ts` is `prerender = true` build-time code. It depends on native `@resvg/resvg-js`, which `www/astro.config.mjs` externalizes for SSR. Treat image generation as Node build-time logic, not Cloudflare runtime logic.
- `www/src/lib/renderer.ts` and `www/generate-badges.mjs` read vendored fonts from `www/src/assets/fonts` (OFL-licensed; see the README there). Generated image routes still run at build time in Node, but builds no longer need network access for Google font fetches.
- `pnpm build:screenshots` first builds `www/`, then copies generated PNGs from `www/dist/images` into `images/appstore/<lang>/`.
- If you change tray visuals, regenerate marketing tray art from `src-tauri/examples/render_tray_icon.rs`; do not hand-redraw `www/public/better-resource-monitor*.png` separately from the app renderer.
- macOS App Store builds use `--features app-store`, but GPU sampling must stay enabled through the public IOAccelerator path in `src-tauri/src/gpu.rs`. Do not reintroduce an App Store GPU stub; the old private IOReport workaround is obsolete. `sysinfo` separately enables its dependency feature `apple-app-store` in `Cargo.toml`.

## Website UX Decisions
- Do not add a skip-to-content link to the current marketing site by default. Pages start directly with their main content, and the extra localized copy, CSS, focus target, and DOM add complexity without clear value here. Revisit only if persistent navigation or other repeated chrome is added before the content.
- Marketing link styles (see `DESIGN.md` and `www/src/layouts/Layout.astro`): all text links share one brand-filled pill style (white text, inset bottom edge, no underline) — same-origin `/`, outbound `http(s)`, language switcher, and footer nav alike. Keep `a:has(img)` badge links transparent with no box-shadow. `.comparison-card` keeps its own card treatment.

## Release Notes
- Manual version bumps touch `package.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, and `src-tauri/tauri.conf.json`. `.github/workflows/release.yml` is the executable reference.
- Hardcoded DMG download links also exist in `README.es.md`, `README.pt-br.md`, and `README.zh-cn.md`; the current release workflow only rewrites `README.md`, so localized READMEs are easy to leave stale.
- App Store packaging is driven by GitHub Actions plus trusted `main` workflow scripts: `scripts/build-appstore-bundle.sh`, `scripts/setup-appstore-signing.sh`, `scripts/upload-appstore-package.sh`, `src-tauri/tauri.appstore.conf.json`, `src-tauri/Entitlements.appstore.plist`, and the CI-provided provisioning profile. `.github/workflows/testflight.yml` is the only App Store upload path and owns `CFBundleVersion`; it uses the GitHub run number with a UTC timestamp floor for same-version monotonicity. `.github/workflows/release.yml` opens and merges a version-bump PR, tags the merge commit on `main`, dispatches `TestFlight` for the tag, waits for the App Store Connect upload to pass, then creates the GitHub release. There is no local App Store packaging fallback.
- Before merging any PR, run TestFlight through the `main` workflow while passing the exact PR head ref as source, and require real tester confirmation when the change affects app behavior or tray visuals. Use `pr_number="$(gh pr view --json number -q .number)" && gh workflow run testflight.yml --ref main -f source_ref="refs/pull/${pr_number}/head" -f pr_number="$pr_number"` after the PR exists; the workflow writes the `Pre-merge TestFlight` status back to the PR head commit. Do not substitute a post-merge TestFlight upload for this gate.

## Trust Code Over Docs
- Prefer `Cargo.toml`, `tauri*.json`, workflows, scripts, and source over prose docs. The current executable config uses the repo feature `app-store`, the sysinfo dependency feature `apple-app-store`, and minimum macOS `13.0`; old notes that blur those names or mention macOS `11.0` are stale.
