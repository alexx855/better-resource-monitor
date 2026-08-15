# Plan 004: Make The Website CSP Compatible With Astro ClientRouter

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving to the
> next step. If anything in the "STOP conditions" section occurs, stop and
> report. When done, update the status row for this plan in `plans/README.md`.
>
> **Drift check (run first)**:
> `git diff --stat d3646b8..HEAD -- www/src/layouts/Layout.astro www/public/_headers package.json www/package.json`
>
> If any in-scope file changed since this plan was written, compare the
> "Current state" excerpts against the live code before proceeding; on a
> mismatch, treat it as a STOP condition.

## Status

- **Priority**: P2
- **Effort**: S
- **Risk**: LOW
- **Depends on**: plans/archive/002-add-website-build-to-ci.md recommended first
- **Category**: correctness
- **Planned at**: commit `d3646b8`, 2026-06-11

## Why this matters

The marketing site includes Astro's `ClientRouter`, which intercepts same-origin
link clicks and fetches the next page HTML. The Cloudflare Pages headers set
`connect-src 'none'`, which blocks fetches from JavaScript, including
same-origin fetches. The router catches failures and falls back quietly, so this
can look like "navigation works" while still producing CSP violations and
disabling the intended client-side transition behavior.

The narrow fix is to allow same-origin connections with `connect-src 'self'`
while continuing to block third-party network connections.

## Current state

```text
www/src/layouts/Layout.astro:3
import { ClientRouter } from "astro:transitions";

www/src/layouts/Layout.astro:86
<ClientRouter />
```

```text
www/public/_headers:6
Content-Security-Policy: default-src 'self'; base-uri 'self'; connect-src 'none'; font-src 'self'; form-action 'self'; frame-ancestors 'none'; img-src 'self' data:; manifest-src 'self'; object-src 'none'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; upgrade-insecure-requests
```

Installed Astro 5.16.8 `ClientRouter.astro` intercepts same-origin clicks:

```text
node_modules/.../astro/components/ClientRouter.astro:84-107
// same-origin navigation ...
if (... origin !== location.origin ...) return;
ev.preventDefault();
navigate(href, { ... });
```

Installed Astro router fetches the target HTML:

```text
node_modules/.../astro/dist/transitions/router.js:51-58
async function fetchHTML(href, init) {
  try {
    const headers = new Headers(init?.headers);
    ...
    const res = await fetch(href, { ...init, headers });
```

Repo conventions to preserve:

- Site validation is `pnpm build:www`.
- The site currently has a strict CSP and no analytics/telemetry. Preserve that
  stance; only allow same-origin fetches.
- Do not add a skip-to-content link as part of this fix.

## Commands you will need

| Purpose | Command | Expected on success |
| --- | --- | --- |
| Static CSP check | `rg -n "connect-src 'self'" www/public/_headers` | one match in CSP |
| Site build | `pnpm build:www` | exit 0 |
| Optional preview | `pnpm preview:www` | local Wrangler Pages dev server starts |
| Check scope | `git status --short` | only in-scope files are modified |

## Scope

**In scope**:

- `www/public/_headers`

**Optional in scope only if the primary fix is rejected**:

- `www/src/layouts/Layout.astro`, only to remove `ClientRouter` entirely if the
  maintainer decides the site should not use client-side navigation.

**Out of scope**:

- Do not add analytics, telemetry, or third-party `connect-src` origins.
- Do not weaken unrelated CSP directives.
- Do not redesign navigation, routing, layouts, or content.
- Do not change Cloudflare deployment settings outside this headers file.

## Git workflow

- Branch: `codex/004-csp-client-router`
- Commit style: conventional commits. Example from repo history:
  `fix(www): restore FAQ pages`
- Do not push or open a PR unless the operator instructed it.

## Steps

### Step 1: Allow same-origin connections

Edit `www/public/_headers` and change only the CSP `connect-src` directive:

```text
connect-src 'none'
```

to:

```text
connect-src 'self'
```

Keep every other directive unchanged.

**Verify**:
`rg -n "connect-src 'self'" www/public/_headers` returns one match.

### Step 2: Build the site

Run:

```bash
pnpm build:www
```

Expected: exit 0. Existing Cloudflare adapter warnings are acceptable if the
build completes.

### Step 3: Smoke same-origin navigation when practical

If a browser is available, run:

```bash
pnpm preview:www
```

Open the local preview, click a same-origin navigation link, and check the
browser console. Expected: no CSP error saying the page refused to connect
because of `connect-src 'none'`.

If no browser is available in the execution environment, record that the
runtime smoke was skipped and rely on the static CSP check plus build.

## Test plan

- No new automated tests are necessary for this one-line header fix.
- Regression coverage is `pnpm build:www` plus static verification that the CSP
  now uses `connect-src 'self'`.
- Browser smoke is recommended because the bug is runtime CSP behavior.

## Done criteria

- [ ] `www/public/_headers` uses `connect-src 'self'`.
- [ ] No other CSP directive changed unless explicitly justified.
- [ ] `pnpm build:www` exits 0.
- [ ] Browser smoke confirms no `connect-src 'none'` errors, or the skip is
      documented if no browser is available.
- [ ] `plans/README.md` status row updated.

## STOP conditions

Stop and report if:

- `ClientRouter` has already been removed since commit `d3646b8`.
- The maintainer wants `connect-src 'none'` preserved exactly. In that case the
  correct alternative is to remove `ClientRouter`, which should be a deliberate
  product/UX decision.
- A build or preview failure appears unrelated to this header change.

## Maintenance notes

- If future features add API calls, analytics, or remote image probes, review
  CSP intentionally rather than expanding `connect-src` ad hoc.
- If Plan 003 upgrades Astro and removes or changes the transitions router,
  re-run the drift check before applying this plan.
