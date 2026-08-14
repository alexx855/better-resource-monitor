# Plan 009: Make CI enforce every documented check (clippy, website build, astro check)

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving to the
> next step. If anything in the "STOP conditions" section occurs, stop and
> report — do not improvise. When done, update the status row for this plan
> in `plans/README.md` — unless a reviewer dispatched you and told you they
> maintain the index.
>
> **Drift check (run first)**:
> `git diff --stat 85c3e82..HEAD -- .github/workflows/ci.yml www/package.json www/astro.config.mjs package.json CONTRIBUTING.md .github/PULL_REQUEST_TEMPLATE.md`
> If any in-scope file changed since this plan was written, compare the
> "Current state" excerpts against the live code before proceeding; on a
> mismatch, treat it as a STOP condition.

## Status

- **Priority**: P1
- **Effort**: S
- **Risk**: LOW
- **Depends on**: none
- **Category**: dx / tests
- **Planned at**: commit `85c3e82`, 2026-07-08
- **Supersedes**: `plans/archive/002-add-website-build-to-ci.md` (same goal, extended
  to also cover clippy and astro check; read it only if you want extra context)

## Why this matters

Three checks are documented as mandatory but never run in CI, so violations
merge green:

- `cargo clippy` — listed in `AGENTS.md:9` and `CONTRIBUTING.md` as a standard
  Rust check; CI (`.github/workflows/ci.yml`) runs fmt/test/check/audits but
  no clippy step.
- `pnpm build:www` — the PR template and CONTRIBUTING tell contributors to run
  it for website changes; CI never does. The website is a real product surface
  (localized pages, generated OG images, README-derived home pages), and a
  broken `content.config.ts` or image route merges silently.
- `astro check` — `@astrojs/check` is installed in `www/devDependencies` but
  wired to nothing; TypeScript errors in `.astro`/`.ts` files ship undetected.

Baselines verified at the planned-at commit (2026-07-08):

| Command | Result |
|---|---|
| `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` | exit 0, clean |
| `pnpm build:www` | exit 0, 33 pages in ~20s (needs network for Google Fonts) |
| `cd www && pnpm exec astro check` | **3 pre-existing errors, all in `www/astro.config.mjs`** (see Step 2) |

So clippy and the site build can be gated immediately; astro check needs a
small pre-fix.

## Current state

- `.github/workflows/ci.yml` — single `rust` job on `macos-15-intel`. Relevant
  step order: checkout → Rust toolchain → rust-cache → pnpm/node setup →
  `pnpm install --frozen-lockfile` → `pnpm audit --prod --audit-level high` →
  script `bash -n` checks → cargo-audit → `cargo fmt --check` (line 71-72) →
  `cargo test` (line 74-75) → `cargo check --target x86_64-apple-darwin`
  (line 77-78) → `pnpm tauri build --bundles app` + bundle-layout assertions
  (lines 80-91).
- `www/package.json` scripts: `dev`, `build`, `preview`, `astro`, `deploy`,
  `cf-typegen` — no `check` script. `@astrojs/check` and `typescript` are in
  `devDependencies`.
- Root `package.json` scripts include `build:www` (`pnpm --filter www build`).
- The three `astro check` errors, verbatim:
  1. `astro.config.mjs:6:33` ts(7006) — `const externalNativeRenderer = (id) => ...` has implicit-any `id`.
  2. `astro.config.mjs:27:5` ts(2353) — `platformProxy` not a known option of the `@astrojs/cloudflare` adapter's `Options` type.
  3. `astro.config.mjs:39:18` ts(2322) — `external: [/^@resvg\/resvg-js(?:-.+)?$/, ...]` — RegExp in a `string[]`-typed field.
- `AGENTS.md:17` warns: `www/src/lib/renderer.ts` fetches fonts from Google
  during build, so the website build needs network access. GitHub-hosted
  runners have network, so this is fine today; `plans/archive/005-vendor-generated-image-fonts.md`
  removes the dependency later.
- Convention: CI steps use SHA-pinned actions with version comments; job runs
  on `macos-15-intel`.

## Commands you will need

| Purpose | Command (repo root) | Expected on success |
|---|---|---|
| Clippy | `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` | exit 0 |
| Site build | `pnpm build:www` | exit 0 |
| Type check | `pnpm --filter www check` (exists after Step 3) | exit 0 |
| Workflow lint | `actionlint .github/workflows/ci.yml` (skip if unavailable) | no errors |

## Scope

**In scope**:
- `.github/workflows/ci.yml`
- `www/package.json` (add `check` script)
- `www/astro.config.mjs` (fix the 3 type errors — types only, no behavior change)
- Root `package.json` (optional `check:www` convenience script)
- `CONTRIBUTING.md`, `.github/PULL_REQUEST_TEMPLATE.md` (state what CI now enforces)
- `plans/README.md` (status rows)

**Out of scope** (do NOT touch):
- `www/src/**` — if `astro check` reports errors outside `astro.config.mjs`,
  that's drift; see STOP conditions.
- The existing Rust steps and the tauri bundle-layout check in ci.yml — do not
  reorder or remove them.
- `testflight.yml`, `release.yml`, `direct-download.yml` (plans 006/007).
- Vendoring fonts (plan 005).

## Git workflow

- Branch: `fix/ci-enforce-documented-checks`
- Conventional commits, e.g. `ci: enforce clippy and website build` — matches
  history (`ci: harden testflight dispatch`).
- Do NOT push or open a PR unless the operator instructed it.

## Steps

### Step 1: Add a clippy step to CI

In `.github/workflows/ci.yml`, after "Run tests" (line 74-75) and before
"Check Intel macOS target", add:

```yaml
      - name: Run clippy
        run: cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
```

**Verify**: locally, `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings`
→ exit 0 (verified clean at planned-at commit; if it now fails, fix only what
clippy names, or STOP if a fix would change behavior).

### Step 2: Fix the three type errors in `www/astro.config.mjs`

Minimal, behavior-preserving fixes:

1. Line 6: annotate the parameter with JSDoc:
   ```js
   /** @param {string} id */
   const externalNativeRenderer = (id) => /@resvg[\\/]/.test(id) || id.endsWith('.node');
   ```
2. Line 27 `platformProxy`: check the installed `@astrojs/cloudflare` (v13)
   adapter options type (`node_modules/@astrojs/cloudflare/dist/index.d.ts`
   or its `Options` export). If the option was renamed in v13, rename to the
   current equivalent; if it is genuinely unsupported dead config, delete the
   block **only after** confirming `pnpm build:www` and `pnpm --filter www dev`
   still start (the option affects `wrangler` dev proxying, not builds).
   If unsure after reading the types, suppress with a `// @ts-expect-error`
   plus a one-line comment naming the adapter version, and note it in the PR.
3. Line 39: the Vite `ssr.external` type is `string[] | true`. The RegExp
   entry does work at runtime for rollup `external`, but here it's in the typed
   `ssr.external` array. Preserve exact behavior: move the RegExp matching into
   the existing function form if one is used nearby, or suppress with
   `// @ts-expect-error resvg native module matched by regex` — do NOT change
   which modules get externalized (AGENTS.md: resvg externalization is
   load-bearing for the image routes).

**Verify**: `cd www && pnpm exec astro check` → `0 errors`; then
`pnpm build:www` → exit 0 (behavior unchanged).

### Step 3: Wire the `check` script

In `www/package.json` scripts, add:

```json
"check": "astro check"
```

Optionally add to root `package.json`: `"check:www": "pnpm --filter www check"`.

**Verify**: `pnpm --filter www check` → exit 0, `0 errors`.

### Step 4: Add website build + check steps to CI

In `.github/workflows/ci.yml`, after the "Install frontend dependencies" step
(so it reuses the same install), add:

```yaml
      - name: Check website types
        run: pnpm --filter www check

      - name: Build website
        run: pnpm build:www
```

Placement note: putting these before the Rust steps gives faster failure on
site-only PRs; anywhere after `pnpm install` is acceptable. Keep the existing
steps untouched.

**Verify**: `actionlint .github/workflows/ci.yml` → no errors (or careful YAML
review); `pnpm build:www` → exit 0 locally.

### Step 5: Update the contributor docs to match

- `CONTRIBUTING.md`: in the validation section, note that CI enforces
  `cargo fmt`, `cargo test`, `cargo clippy -D warnings`, `cargo check`
  (Intel), `pnpm audit`, `cargo audit`, `astro check`, `pnpm build:www`, and
  the bundle-layout check.
- `.github/PULL_REQUEST_TEMPLATE.md`: add clippy to the checklist so the
  documented list matches CI.

**Verify**: `rg -n "clippy" CONTRIBUTING.md .github/PULL_REQUEST_TEMPLATE.md` → matches in both.

## Test plan

No new unit tests. Verification is the CI run itself:
- All local commands in "Commands you will need" exit 0.
- After pushing the branch (only if operator instructed), the CI workflow run
  shows the three new steps green.

## Done criteria

- [ ] `ci.yml` contains clippy, `pnpm --filter www check`, and `pnpm build:www` steps
- [ ] `cd www && pnpm exec astro check` → 0 errors locally
- [ ] `pnpm build:www` → exit 0 locally (same page count as before, 33 pages)
- [ ] `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` → exit 0
- [ ] CONTRIBUTING.md and PR template mention clippy/CI enforcement
- [ ] No files outside the in-scope list are modified (`git status`)
- [ ] `plans/README.md` status rows updated (this plan DONE; plan 002 marked SUPERSEDED)

## STOP conditions

Stop and report back (do not improvise) if:

- `astro check` reports errors in files other than `www/astro.config.mjs`
  (codebase drifted; fixing site source is out of scope).
- Fixing the `platformProxy` error appears to require changing dev-server or
  deploy behavior (not just types).
- Clippy fails with warnings whose fix would alter runtime behavior (not just
  style).
- `pnpm build:www` fails locally for network/font reasons — report; do not
  vendor fonts here (that's plan 005).

## Maintenance notes

- CI build time on `macos-15-intel` grows by roughly the site build (~20s
  local; expect 1–2 min on CI with cold caches). If that becomes painful,
  split website checks into a separate `ubuntu-latest` job — deferred because
  a single job keeps the workflow simple today.
- Once plan 005 (vendored fonts) lands, the website CI step stops depending on
  Google Fonts availability.
- If a locale-parity check script is added (plan 010), wire it next to the
  website steps added here.
