# Plan 011: Remove Astro ClientRouter — resolve the CSP conflict by deleting its cause

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving to the
> next step. If anything in the "STOP conditions" section occurs, stop and
> report — do not improvise. When done, update the status row for this plan
> in `plans/README.md` — unless a reviewer dispatched you and told you they
> maintain the index.
>
> **Drift check (run first)**:
> `git diff --stat 85c3e82..HEAD -- www/src/layouts/Layout.astro www/public/_headers www/src/components`
> If any in-scope file changed since this plan was written, compare the
> "Current state" excerpts against the live code before proceeding; on a
> mismatch, treat it as a STOP condition.

## Status

- **Priority**: P2
- **Effort**: S
- **Risk**: LOW
- **Depends on**: plans/009 recommended first (so CI catches any build breakage)
- **Category**: perf / security-config
- **Planned at**: commit `85c3e82`, 2026-07-08
- **Supersedes**: `plans/004-fix-website-csp-client-router.md` — that plan
  proposed loosening CSP (`connect-src 'none'` → `'self'`) so ClientRouter's
  same-origin fetches work. This plan takes the opposite, strictly better
  trade: remove ClientRouter, keep the maximally strict CSP.

## Why this matters

Every page of the marketing site ships Astro's ClientRouter script
(verified in the built output: `www/dist/client/index.html` loads
`/_astro/ClientRouter.astro_astro_type_script_index_0_lang.*.js` — it is the
**only** JavaScript file on the site). What it buys today:

- Soft (fetch-based) navigation between pages — which the site's own CSP
  (`connect-src 'none'` in `www/public/_headers:6`) **blocks**, so the router
  silently falls back to full page loads anyway, logging CSP violations.
- One cosmetic view-transition (`view-transition-name: site-title` on the
  header title, `Layout.astro:384`) that can only ever fire when soft
  navigation works — i.e. never, under the current CSP.

So the site pays JS download/parse/execute on every page for a feature the
CSP disables. Removing ClientRouter makes the site 100% JavaScript-free
(structured-data `<script type="application/ld+json">` blocks are inert data,
unaffected), keeps the strict CSP intact, and closes plan 004 without touching
security headers. A static four-page marketing site loses nothing user-visible:
navigation becomes normal MPA loads, which is what users effectively get today.

Verified before planning: no code in `www/src` depends on router lifecycle —
`rg "astro:page-load|astro:after-swap|transition:persist|transition:animate|navigate\(" www/src`
returns nothing, and the only `<script>` tags in components are JSON-LD.

## Current state

- `www/src/layouts/Layout.astro:3` — `import { ClientRouter } from "astro:transitions";`
- `www/src/layouts/Layout.astro:88` — `<ClientRouter />` in `<head>`.
- `www/src/layouts/Layout.astro:384` — inside the global style block:
  ```css
  view-transition-name: site-title;
  ```
  (dead once the router is gone; browsers ignore it without a view transition).
- `www/public/_headers:6` — CSP with `connect-src 'none'` — **unchanged by this plan**.
- Built output at the planned-at commit: `index.html` has exactly one external
  script (the ClientRouter bundle) and one CSS file (`Layout.*.css`).
- Build command: `pnpm build:www` from repo root (33 pages, exit 0 at planning time).

## Commands you will need

| Purpose | Command (repo root) | Expected on success |
|---|---|---|
| Site build | `pnpm build:www` | exit 0, 33 pages |
| Type check | `cd www && pnpm exec astro check` | 0 errors (after plan 009's config fixes; 3 pre-existing `astro.config.mjs` errors before that — ignore those if 009 hasn't landed) |
| JS-free proof | `rg -l "ClientRouter" www/dist/client` | no matches (after rebuild) |

## Scope

**In scope**:
- `www/src/layouts/Layout.astro` (remove import, tag, and the dead
  `view-transition-name` declaration and its containing rule if empty)
- `plans/README.md` (status rows: this plan, and 004 → SUPERSEDED)

**Out of scope** (do NOT touch):
- `www/public/_headers` — the CSP stays exactly as is; that's the point.
- Any component/page files — none reference transitions (verified above); if
  you find one that does, that's a STOP condition, not a fix-it-here.
- `www/astro.config.mjs` — no transition config exists there.

## Git workflow

- Branch: `fix/www-remove-client-router`
- Conventional commit, e.g. `perf(www): drop ClientRouter to keep strict CSP and ship zero JS`
- Do NOT push or open a PR unless the operator instructed it.

## Steps

### Step 1: Remove ClientRouter from the layout

In `www/src/layouts/Layout.astro`:
- Delete line 3 (`import { ClientRouter } from "astro:transitions";`).
- Delete line 88 (`<ClientRouter />`).
- Find the rule containing `view-transition-name: site-title;` (~line 384) and
  remove that declaration; if the rule becomes empty, remove the rule.

**Verify**: `rg -n "ClientRouter|view-transition" www/src` → no matches.

### Step 2: Rebuild and confirm zero JavaScript

```
pnpm build:www
rg -l "ClientRouter" www/dist/client        # expect: no matches
rg -o 'src="[^"]*\.js"' www/dist/client/index.html   # expect: no output
```

Also confirm page count is still 33 in the build log, and spot-check that
`www/dist/client/_astro/` contains only CSS (no `.js`).

**Verify**: all three commands produce the expected results above.

### Step 3: Manual navigation smoke check

Run `pnpm preview:www` (builds + serves via wrangler) and click through:
home → FAQ → comparison index → a comparison page → back home, plus one
localized page (`/es/`). Every navigation is a normal page load; no console
errors.

**Verify**: no console errors; all links navigate correctly.

## Test plan

No unit tests apply. The verification gates in Steps 2–3 are the test:
build output contains zero `.js`, and manual navigation works. If plan 009
landed, CI's `pnpm build:www` step guards regressions.

## Done criteria

- [ ] `rg -n "ClientRouter|view-transition" www/src` → no matches
- [ ] `pnpm build:www` → exit 0, 33 pages
- [ ] Built site contains no JavaScript files (`rg -o 'src="[^"]*\.js"' www/dist/client/index.html` → empty; no `.js` in `www/dist/client/_astro/`)
- [ ] `www/public/_headers` is unmodified (`git diff --stat -- www/public/_headers` → empty)
- [ ] No files outside the in-scope list are modified (`git status`)
- [ ] `plans/README.md` updated: this plan DONE, plan 004 SUPERSEDED

## STOP conditions

Stop and report back (do not improvise) if:

- Any file in `www/src` besides `Layout.astro` matches
  `ClientRouter|astro:transitions|transition:` — a transition consumer was
  added since planning; the remove-vs-keep decision needs revisiting.
- The build output still contains a `.js` file after removal — identify what
  emits it and report (something else started shipping client JS).
- The maintainer has expressed (in a newer commit message or doc) intent to
  actually use view transitions — then execute old plan 004 (CSP
  `connect-src 'self'`) instead, and mark THIS plan rejected.

## Maintenance notes

- If soft navigation/view transitions are ever wanted for real, re-adding
  `<ClientRouter />` requires the CSP change from superseded plan 004
  (`connect-src 'none'` → `'self'` in `www/public/_headers`) or navigation
  silently degrades — leave a comment in `_headers` at that time.
- The site being zero-JS is now a property worth advertising and guarding;
  a reviewer seeing any new `<script>` (other than `type="application/ld+json"`)
  in `www/src` should ask why.
