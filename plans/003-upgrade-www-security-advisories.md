# Plan 003: Clear High And Moderate Website Dependency Advisories

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving to the
> next step. If anything in the "STOP conditions" section occurs, stop and
> report. When done, update the status row for this plan in `plans/README.md`.
>
> **Drift check (run first)**:
> `git diff --stat d3646b8..HEAD -- www/package.json pnpm-lock.yaml www/astro.config.mjs www/src www/public package.json`
>
> If any in-scope file changed since this plan was written, compare the
> "Current state" excerpts against the live code before proceeding; on a
> mismatch, treat it as a STOP condition.

## Status

- **Priority**: P1
- **Effort**: M
- **Risk**: MED
- **Depends on**: plans/002-add-website-build-to-ci.md recommended first
- **Category**: security
- **Planned at**: commit `d3646b8`, 2026-06-11

## Why this matters

`pnpm audit --prod` currently reports 36 production dependency advisories in
the website dependency graph, including 14 high severity advisories. Several
are in build/deploy tooling used by this repo: Astro, `@astrojs/cloudflare`,
Vite, Rollup, Wrangler, Undici, H3, and related transitive packages. The site is
small enough that keeping the framework/toolchain current is cheaper than
carrying known advisory noise, and it reduces risk for Cloudflare Pages deploys
and local build tooling.

This plan is specifically for the `www` JavaScript/Astro dependency tree. Rust
dependency auditing is not included because `cargo audit` was not installed in
the advisor environment.

## Current state

```text
www/package.json:13-25
"dependencies": {
  "@astrojs/cloudflare": "^12.6.12",
  "@astrojs/sitemap": "^3.6.1",
  "@resvg/resvg-js": "^2.6.2",
  "astro": "^5.16.8",
  "satori": "^0.19.1"
},
"devDependencies": {
  "@astrojs/check": "^0.9.8",
  "@types/node": "^25.0.5",
  "sharp": "^0.34.5",
  "typescript": "^5.9.3",
  "wrangler": "^4.58.0"
}
```

Advisor audit command:

```bash
pnpm audit --prod --json | node -e '...summary script...'
```

Observed summary on 2026-06-11:

```text
counts high=14 moderate=15 low=7 critical=0
1x high defu >=6.1.5
2x high devalue >=5.6.2
1x high h3 >=1.15.5
1x high h3 >=1.15.6
2x high picomatch >=2.3.2/>=4.0.4
1x high rollup >=4.59.0
1x high svgo >=4.0.1
3x high undici >=7.24.0
1x high vite >=6.4.2
1x high wrangler >=4.59.1
1x low @astrojs/cloudflare >=13.1.10
1x moderate astro >=6.1.6
...
```

`pnpm --filter www exec astro --version` reported:

```text
astro  v5.16.8
```

`cargo audit` from `src-tauri/` reported:

```text
error: no such command: `audit`
```

Repo conventions to preserve:

- `pnpm` is the package manager, with `packageManager` set to `pnpm@10.33.2`.
- Site validation command is `pnpm build:www`.
- Do not invent root `pnpm test` or `pnpm lint`.

## Commands you will need

| Purpose | Command | Expected on success |
| --- | --- | --- |
| Inspect current advisories | `pnpm audit --prod` | currently exits non-zero; use before/after comparison |
| Update Astro packages | `pnpm --filter www update astro @astrojs/cloudflare @astrojs/sitemap @astrojs/check --latest` | exit 0; framework packages updated |
| Update remaining website tooling | `pnpm --filter www update wrangler @types/node typescript sharp satori @resvg/resvg-js --latest` | exit 0; remaining direct packages updated |
| Build site | `pnpm build:www` | exit 0 |
| Check remaining advisories | `pnpm audit --prod` | preferably exit 0; at minimum no high or moderate advisories |
| Check scope | `git status --short` | only in-scope files are modified |

## Scope

**In scope**:

- `www/package.json`
- `pnpm-lock.yaml`
- `www/astro.config.mjs`, only if Astro or Cloudflare adapter migration requires it
- `www/src/**`, only for minimal migration fixes required by the package update
- `www/public/**`, only if build output expectations require static asset metadata changes

**Out of scope**:

- Do not change `src-tauri/**` Rust code.
- Do not change release/TestFlight workflows.
- Do not switch package managers.
- Do not silence advisories with audit ignore files unless the operator
  explicitly accepts a documented residual risk.

## Git workflow

- Branch: `codex/003-www-security-upgrades`
- Commit style: conventional commits. Example from repo history:
  `fix(www): restore FAQ pages`
- Do not push or open a PR unless the operator instructed it.

## Steps

### Step 1: Capture the before state

Run:

```bash
pnpm audit --prod
pnpm --filter www exec astro --version
pnpm build:www
```

Expected:

- `pnpm audit --prod` exits non-zero before the update.
- Astro reports `v5.16.8` unless drift already occurred.
- `pnpm build:www` exits 0 before the dependency change.

If the build fails before changes, stop. Dependency upgrades should not be
mixed with pre-existing build repair unless the operator approves.

### Step 2: Upgrade the website toolchain

Run targeted website updates in phases so breaking changes are easier to
isolate:

```bash
pnpm --filter www update astro @astrojs/cloudflare @astrojs/sitemap @astrojs/check --latest
pnpm build:www
pnpm --filter www update wrangler @types/node typescript sharp satori @resvg/resvg-js --latest
```

The first phase moves Astro and its integrations together. The second phase
updates deploy/build tooling and renderer dependencies. Together they should
update direct website dependencies and the transitive lockfile. The
important minimums from the audit are:

- `astro` must move past the vulnerable 5.x line. Audit listed patched Astro
  versions at least `>=6.1.10` for the newer low advisory and `>=6.1.6` for the
  moderate advisory.
- `@astrojs/cloudflare` must be at least `>=13.1.10`.
- `wrangler` must be at least `>=4.59.1`.
- Transitives such as Vite, Rollup, H3, Undici, Devalue, Defu, Picomatch, SVGO,
  PostCSS, `ws`, `yaml`, and `smol-toml` should resolve to patched versions
  through the direct updates.

**Verify**:
`git diff -- www/package.json pnpm-lock.yaml` shows direct package updates and
lockfile changes only at this step.

### Step 3: Apply minimal Astro migration fixes if needed

Run:

```bash
pnpm build:www
```

If it fails because of an Astro or Cloudflare adapter breaking change, make the
smallest migration edit in `www/astro.config.mjs` or `www/src/**`. Keep the site
shape, routes, content collections, and generated image route behavior intact.

Do not rewrite the site, change copy, or alter the deployment target. This plan
is only for dependency security and migration compatibility.

**Verify**:
`pnpm build:www` exits 0.

### Step 4: Confirm advisory reduction

Run:

```bash
pnpm audit --prod
```

Expected: exit 0. If new low severity advisories remain because no patched
version is available, document them in the commit/PR body and in
`plans/README.md` when marking the plan done. Do not accept remaining high or
moderate advisories without operator approval.

## Test plan

- Primary verification is `pnpm build:www`.
- Add no tests unless a required Astro migration changes local helper logic in
  `www/src/lib/**`. If it does, add a small TypeScript or build-time check only
  if the repo already has a matching pattern; otherwise rely on the Astro build.
- Run `pnpm audit --prod` after the update and record remaining counts.

## Done criteria

- [ ] `www/package.json` direct dependencies are upgraded enough to pull patched
      versions for the advisories listed above.
- [ ] `pnpm-lock.yaml` is updated and committed.
- [ ] `pnpm build:www` exits 0.
- [ ] `pnpm audit --prod` exits 0, or has no high/moderate advisories and any
      residual low advisories are explicitly documented.
- [ ] No Rust code, release workflow, or package-manager migration is included.
- [ ] `plans/README.md` status row updated.

## STOP conditions

Stop and report if:

- Astro 6 or the Cloudflare adapter upgrade requires broad route/content
  rewrites rather than small compatibility edits.
- `pnpm build:www` fails before any dependency changes.
- `pnpm audit --prod` still reports high or moderate advisories after updating
  to current direct dependencies.
- Clearing the advisories requires pinning transitive dependencies with
  overrides that conflict with Astro or Wrangler peer dependencies.

## Maintenance notes

- Keep Plan 002's CI site build in place so future advisory upgrades are gated.
- If the repo wants Rust advisory coverage, add a separate plan to install or
  run `cargo audit` in CI. Do not fold that into this JavaScript dependency
  migration.
- Reviewers should inspect any Astro migration edits carefully; the dependency
  update is the goal, not a redesign.
