# Plan 005: Vendor Generated-Image Fonts For Deterministic Builds

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving to the
> next step. If anything in the "STOP conditions" section occurs, stop and
> report. When done, update the status row for this plan in `plans/README.md`.
>
> **Drift check (run first)**:
> `git diff --stat d3646b8..HEAD -- www/src/lib/renderer.ts www/generate-badges.mjs www/src/assets www/public AGENTS.md scripts package.json www/package.json`
>
> If any in-scope file changed since this plan was written, compare the
> "Current state" excerpts against the live code before proceeding; on a
> mismatch, treat it as a STOP condition.

## Status

- **Priority**: P2
- **Effort**: M
- **Risk**: MED
- **Depends on**: plans/002-add-website-build-to-ci.md recommended first
- **Category**: dx
- **Planned at**: commit `d3646b8`, 2026-06-11

## Why this matters

The website build and badge generator fetch Google Fonts at build time. That
makes `pnpm build:www`, `pnpm build:screenshots`, and badge regeneration depend
on external network availability and on Google font URLs staying stable. The
site already treats generated images as build-time Node work, so vendoring the
exact font files makes builds deterministic and removes a known CI/release
fragility point.

This plan should not change tray rendering or app runtime behavior. It only
changes website/badge image generation inputs.

## Current state

`www/src/lib/renderer.ts` already imports `readFileSync` and `join`, but still
fetches fonts remotely:

```text
www/src/lib/renderer.ts:23-36
// Font cache - fetched once per build
let fontData: ArrayBuffer | null = null;

async function fetchFont(url: string): Promise<ArrayBuffer> {
  const res = await fetch(url);
  if (!res.ok) throw new Error(`Failed to fetch font ${url}: ${res.status} ${res.statusText}`);
  return res.arrayBuffer();
}

async function loadFont(): Promise<ArrayBuffer> {
  if (fontData) return fontData;
  fontData = await fetchFont(
    "https://fonts.gstatic.com/s/jetbrainsmono/v24/tDbY2o-flEEny0FZhsfKu5WU4zr3E_BX0PnT8RD8yKxjPQ.ttf"
  );
```

```text
www/src/lib/renderer.ts:42-68
loadFontBold() fetches JetBrains Mono bold from fonts.gstatic.com
loadNotoJP() fetches Noto Sans JP from fonts.gstatic.com
loadNotoSC() fetches Noto Sans SC from fonts.gstatic.com
```

`www/generate-badges.mjs` also fetches fonts remotely:

```text
www/generate-badges.mjs:42-51
async function loadFonts() {
  const [bold, notoSC] = await Promise.all([
    fetch("https://fonts.gstatic.com/s/jetbrainsmono/...L6tjPQ.ttf").then((r) => r.arrayBuffer()),
    fetch("https://fonts.gstatic.com/s/notosanssc/...GzjCnYw.ttf").then((r) => r.arrayBuffer()),
  ]);
```

Repo-local instructions currently document the fragility:

```text
AGENTS.md:17
www/src/lib/renderer.ts fetches fonts from Google during build. pnpm build:www
and pnpm build:screenshots need network access.
```

Repo conventions to preserve:

- `www/src/pages/images/[id].png.ts` is prerendered Node build-time code.
- `@resvg/resvg-js` is a build-time native dependency, not Cloudflare runtime.
- If tray visuals change, marketing tray art comes from the Rust tray renderer;
  do not hand-redraw those assets here.

## Commands you will need

| Purpose | Command | Expected on success |
| --- | --- | --- |
| Download font files | `curl -L --fail -o <target> <font-url>` | exit 0 for each font |
| Search for remote font fetches | `rg -n "fonts\\.gstatic\\.com|fetchFont|Failed to fetch font" www/src/lib/renderer.ts www/generate-badges.mjs` | no matches after edits |
| Regenerate badges | `node www/generate-badges.mjs` | exit 0; badge files logged |
| Build site | `pnpm build:www` | exit 0 |
| Build screenshots | `pnpm build:screenshots` | exit 0; images copied to `images/appstore/<lang>/` |
| Check scope | `git status --short` | only in-scope files are modified |

## Scope

**In scope**:

- `www/src/assets/fonts/*` or another clearly named source-controlled font
  directory under `www/src/assets`
- `www/src/lib/renderer.ts`
- `www/generate-badges.mjs`
- generated badge/image assets changed by running the repo's existing
  generation commands
- `AGENTS.md`, only to update the now-stale note that site builds need Google
  font network access

**Out of scope**:

- Do not change `src-tauri/src/tray_render.rs`.
- Do not hand-edit `www/public/better-resource-monitor*.png`.
- Do not change marketing copy, locales, or route structure.
- Do not add a new font package dependency unless direct vendored files are
  blocked by licensing or size concerns.

## Git workflow

- Branch: `codex/005-vendor-image-fonts`
- Commit style: conventional commits. Example from repo history:
  `fix(www): restore FAQ pages`
- Do not push or open a PR unless the operator instructed it.

## Steps

### Step 1: Add the font files

Create a source-controlled font directory, for example:

```bash
mkdir -p www/src/assets/fonts
```

Download the exact fonts currently used:

```bash
curl -L --fail -o www/src/assets/fonts/JetBrainsMono-Regular.ttf \
  "https://fonts.gstatic.com/s/jetbrainsmono/v24/tDbY2o-flEEny0FZhsfKu5WU4zr3E_BX0PnT8RD8yKxjPQ.ttf"
curl -L --fail -o www/src/assets/fonts/JetBrainsMono-Bold.ttf \
  "https://fonts.gstatic.com/s/jetbrainsmono/v24/tDbY2o-flEEny0FZhsfKu5WU4zr3E_BX0PnT8RD8L6tjPQ.ttf"
curl -L --fail -o www/src/assets/fonts/NotoSansJP-Bold.ttf \
  "https://fonts.gstatic.com/s/notosansjp/v56/-F6jfjtqLzI2JPCgQBnw7HFyzSD-AsregP8VFPYk75s.ttf"
curl -L --fail -o www/src/assets/fonts/NotoSansSC-Bold.ttf \
  "https://fonts.gstatic.com/s/notosanssc/v40/k3kCo84MPvpLmixcA63oeAL7Iqp5IZJF9bmaGzjCnYw.ttf"
```

Before committing binary font files, verify their license permits repository
distribution. Google Fonts families are normally under the SIL Open Font
License, but do not assume: check the upstream metadata and add a short
`www/src/assets/fonts/README.md` or license note if needed.

**Verify**:
`ls -lh www/src/assets/fonts` shows all four font files and any license note.

### Step 2: Replace remote fetches in the image renderer

In `www/src/lib/renderer.ts`, replace `fetchFont(url)` with a local file helper.
Target shape:

```ts
const FONT_MAP = {
  "JetBrainsMono-Regular.ttf": join(process.cwd(), "src", "assets", "fonts", "JetBrainsMono-Regular.ttf"),
  "JetBrainsMono-Bold.ttf": join(process.cwd(), "src", "assets", "fonts", "JetBrainsMono-Bold.ttf"),
  "NotoSansJP-Bold.ttf": join(process.cwd(), "src", "assets", "fonts", "NotoSansJP-Bold.ttf"),
  "NotoSansSC-Bold.ttf": join(process.cwd(), "src", "assets", "fonts", "NotoSansSC-Bold.ttf"),
};

function readFontAsset(filename: keyof typeof FONT_MAP): Uint8Array {
  return readFileSync(FONT_MAP[filename]);
}
```

Then update the cached loaders:

- `loadFont()` reads `JetBrainsMono-Regular.ttf`
- `loadFontBold()` reads `JetBrainsMono-Bold.ttf`
- `loadNotoJP()` reads `NotoSansJP-Bold.ttf`
- `loadNotoSC()` reads `NotoSansSC-Bold.ttf`

Keep the existing cache behavior and `renderImage()` font array shape, but
update the related TypeScript types to match the local-file helper:

- cache variables such as `fontData`, `fontBoldData`, `notoJPData`, and
  `notoSCData` should be `Uint8Array | null`
- loader return types should be `Promise<Uint8Array>`
- `fontLoads` should be typed as `Promise<Uint8Array>[]`

Do not leave these as `ArrayBuffer` if `readFontAsset()` returns the
`readFileSync` buffer directly.

Keep the `join` import in `www/src/lib/renderer.ts` because the static
`FONT_MAP` uses it to resolve filesystem paths.

**Verify**:
`rg -n "fonts\\.gstatic\\.com|fetchFont|Failed to fetch font" www/src/lib/renderer.ts`
returns no matches.

### Step 3: Replace remote fetches in badge generation

In `www/generate-badges.mjs`, make sure the existing named imports include
`readFileSync` from `node:fs` and `join` from `node:path`, then read the same
vendored font files with `readFileSync` instead of `fetch`. Use the existing
`ROOT` constant defined near the top of that file instead of mixing in
`import.meta.dirname`.

Recommended pattern:

```js
const FONT_DIR = join(ROOT, "src", "assets", "fonts");

function readFontAsset(filename) {
  return readFileSync(join(FONT_DIR, filename));
}
```

Use:

- `JetBrainsMono-Bold.ttf`
- `NotoSansSC-Bold.ttf`

**Verify**:
`rg -n "fonts\\.gstatic\\.com|fetch\\(" www/generate-badges.mjs`
returns no matches unless another non-font fetch already existed and is
documented.

### Step 4: Update stale agent guidance

Edit `AGENTS.md` to remove or replace the statement that `pnpm build:www` and
`pnpm build:screenshots` need network access for Google Fonts. Preserve the
rest of the build-time renderer guidance.

Target meaning:

- Generated image routes still run at build time in Node.
- The renderer uses vendored fonts from `www/src/assets/fonts`.
- Network should no longer be required specifically for Google font fetches.

**Verify**:
`rg -n "fonts from Google|need network access" AGENTS.md`
returns no stale statement about Google fonts.

### Step 5: Regenerate and build

Run:

```bash
node www/generate-badges.mjs
pnpm build:www
pnpm build:screenshots
```

Expected: all three commands exit 0.

Review generated asset diffs:

```bash
git diff --stat -- www/public/badges images/appstore www/src/lib/renderer.ts www/generate-badges.mjs AGENTS.md
```

Expected: only in-scope files. Some image binary diffs may occur because the
font bytes are now local but should be visually equivalent.

## Test plan

- No new unit tests are needed.
- Regression checks are:
  - `rg` confirms no remote font URLs remain in image generation code.
  - `node www/generate-badges.mjs` succeeds using local fonts.
  - `pnpm build:www` succeeds.
  - `pnpm build:screenshots` succeeds.
- If visual QA tooling is available, compare a few generated images before and
  after. Text should remain correctly rendered in English and Chinese assets.

## Done criteria

- [ ] All font files used by generated image rendering are vendored under
      `www/src/assets/fonts` or an equivalent source-controlled directory.
- [ ] `www/src/lib/renderer.ts` contains no `fonts.gstatic.com` URL and no font
      fetch helper.
- [ ] `www/generate-badges.mjs` contains no font network fetch.
- [ ] `AGENTS.md` no longer says site builds need network access for Google
      font fetches.
- [ ] `node www/generate-badges.mjs` exits 0.
- [ ] `pnpm build:www` exits 0.
- [ ] `pnpm build:screenshots` exits 0.
- [ ] `plans/README.md` status row updated.

## STOP conditions

Stop and report if:

- The font license cannot be verified as safe for repository redistribution.
- Vendored font files are too large for the repository policy.
- Generated Chinese or Japanese images lose glyph coverage.
- Build failures point to native `@resvg/resvg-js` or Rollup module issues
  unrelated to font loading.
- The fix requires changing tray renderer output or app runtime assets.

## Maintenance notes

- If future locales need additional scripts, add their fonts deliberately and
  keep this renderer local-file based.
- If Plan 003 upgrades Astro/Satori and changes font data expectations, re-run
  all generated image commands after resolving that migration.
- Reviewers should check generated binary diffs by viewing at least the English
  App Store badge, one Chinese badge, and one generated OG image.
