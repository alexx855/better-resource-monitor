# Plan 002: Add The Marketing Site Build To CI

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving to the
> next step. If anything in the "STOP conditions" section occurs, stop and
> report. When done, update the status row for this plan in `plans/README.md`.
>
> **Drift check (run first)**:
> `git diff --stat d3646b8..HEAD -- .github/workflows/ci.yml package.json www/package.json AGENTS.md www/astro.config.mjs www/src`
>
> If any in-scope file changed since this plan was written, compare the
> "Current state" excerpts against the live code before proceeding; on a
> mismatch, treat it as a STOP condition.

## Status

- **Priority**: P1
- **Effort**: S
- **Risk**: LOW
- **Depends on**: none
- **Category**: tests
- **Planned at**: commit `d3646b8`, 2026-06-11

## Why this matters

This repository has two codebases: the root Tauri app and the Astro marketing
site in `www/`. CI currently checks Rust and the macOS app bundle layout, but it
does not run the site build. That means README-loaded home content, localized
content JSON, the prerendered image route, Cloudflare adapter config, and
generated Open Graph images can break without a pull request turning red.

Adding `pnpm build:www` to CI gives every later website, dependency, and copy
change a real gate.

## Current state

```text
AGENTS.md:4-10
This repo has two codebases: the Tauri app at the root/src-tauri/ and the Astro
marketing site in www/.
...
Site commands run from root via workspace filters: pnpm dev:www, pnpm build:www,
pnpm preview:www, pnpm build:screenshots.
```

```text
package.json:40-47
"scripts": {
  "tauri": "tauri",
  "dev": "tauri dev",
  "build": "tauri build",
  "dev:www": "pnpm --filter www dev",
  "build:www": "pnpm --filter www build",
  "preview:www": "pnpm --filter www preview",
  "build:screenshots": "bash scripts/images-helper.sh"
}
```

```text
www/package.json:5-11
"scripts": {
  "dev": "astro dev",
  "build": "astro build",
  "preview": "astro build && wrangler pages dev",
  "astro": "astro",
  "deploy": "astro build && wrangler pages deploy",
  "cf-typegen": "wrangler types"
}
```

```text
.github/workflows/ci.yml:20-69
jobs:
  rust:
    name: Rust checks
    runs-on: macos-15-intel
    ...
      - name: Install frontend dependencies
        run: pnpm install
      ...
      - name: Verify macOS app bundle layout
        run: |
          pnpm tauri build --bundles app --target x86_64-apple-darwin
          APP_PATH="src-tauri/target/x86_64-apple-darwin/release/bundle/macos/Better Resource Monitor.app"
          test ! -d "$APP_PATH/Contents/Library/LaunchAgents"
```

Observed baseline during the advisor audit:

- `pnpm build:www` exited 0.
- Build generated static image routes such as `/images/og-index-en.png`.
- Build printed a Cloudflare adapter warning about sessions/KV and sharp, but
  completed successfully. That warning is not a failure for this plan.

## Commands you will need

| Purpose | Command | Expected on success |
| --- | --- | --- |
| Local site build | `pnpm build:www` | exit 0; Astro build completes |
| Workflow syntax check | `ruby -e 'require "yaml"; YAML.load_file(".github/workflows/ci.yml"); puts "ok"'` | prints `ok` |
| Check working tree scope | `git status --short` | Only `.github/workflows/ci.yml` is modified |

## Scope

**In scope**:

- `.github/workflows/ci.yml`

**Out of scope**:

- Do not modify source files under `www/src` or `src-tauri/src`.
- Do not add root `pnpm test` or `pnpm lint`; repo instructions say those do
  not exist.
- Do not change release or TestFlight workflows.
- Do not combine this with dependency upgrades from Plan 003.

## Git workflow

- Branch: `codex/002-site-build-ci`
- Commit style: conventional commits. Example from repo history:
  `ci: update app store release workflow`
- Do not push or open a PR unless the operator instructed it.

## Steps

### Step 1: Add a dedicated website job

In `.github/workflows/ci.yml`, add a separate job named `site` or
`website` alongside the existing `rust` job.

Recommended shape:

- `runs-on: ubuntu-latest`
- checkout with `actions/checkout@v5`
- setup pnpm with `pnpm/action-setup@v5` and `version: 10.33.2`, matching the
  existing workflow and `package.json`
- setup Node with `actions/setup-node@v5`, `node-version: lts/*`, and
  `cache: 'pnpm'`
- run `pnpm install`
- run `pnpm build:www`

Keep the existing macOS `rust` job intact. A separate Ubuntu site job keeps the
site gate fast and avoids consuming the macOS runner for static site work.

**Verify**:
`ruby -e 'require "yaml"; YAML.load_file(".github/workflows/ci.yml"); puts "ok"'`
prints `ok`.

### Step 2: Run the exact site build locally

Run:

```bash
pnpm build:www
```

Expected: exit 0. Existing Cloudflare adapter warnings are acceptable if the
build completes.

### Step 3: Confirm no accidental script changes

Run:

```bash
git diff -- package.json www/package.json
```

Expected: no diff. This plan should use the existing `pnpm build:www` script,
not add or rename scripts.

## Test plan

- No application tests are needed for this workflow-only change.
- Local verification is `pnpm build:www` plus YAML parse.
- The pull request should show a new GitHub Actions job for the marketing site.

## Done criteria

- [ ] `.github/workflows/ci.yml` contains a site/website job that runs
      `pnpm build:www`.
- [ ] Existing Rust/macOS job behavior remains present.
- [ ] `ruby -e 'require "yaml"; YAML.load_file(".github/workflows/ci.yml"); puts "ok"'`
      prints `ok`.
- [ ] `pnpm build:www` exits 0 locally.
- [ ] No files outside `.github/workflows/ci.yml` are modified.
- [ ] `plans/README.md` status row updated.

## STOP conditions

Stop and report if:

- The workflow already gained a `pnpm build:www` gate since commit `d3646b8`.
- `pnpm build:www` fails locally before any CI edit.
- Adding the job requires secrets or Cloudflare deployment credentials. A static
  build should not need them.
- The operator asks to validate screenshots or deploys in CI; that is a larger
  pipeline plan.

## Maintenance notes

- Plan 003, Plan 004, and Plan 005 all benefit from this gate because they
  touch the marketing site or its build-time dependencies.
- If Plan 005 later removes remote font fetching, this CI job becomes less
  network-sensitive. Until then, CI still needs outbound network for Google
  font fetches during image rendering.
