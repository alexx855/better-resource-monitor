# Plan 010: Enforce locale parity across the app and website, and delete drifted duplicates

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving to the
> next step. If anything in the "STOP conditions" section occurs, stop and
> report — do not improvise. When done, update the status row for this plan
> in `plans/README.md` — unless a reviewer dispatched you and told you they
> maintain the index.
>
> **Drift check (run first)**:
> `git diff --stat 85c3e82..HEAD -- www/src/lib/marketing-copy.json www/src/lib/translations.ts www/src/content www/src/content.config.ts src-tauri/src/i18n.rs scripts/`
> If any in-scope file changed since this plan was written, compare the
> "Current state" excerpts against the live code before proceeding; on a
> mismatch, treat it as a STOP condition.

## Status

- **Priority**: P2
- **Effort**: S–M
- **Risk**: LOW
- **Depends on**: plans/archive/009-enforce-documented-checks-in-ci.md recommended first (its CI website steps are where the new check gets wired)
- **Category**: tests / tech-debt
- **Planned at**: commit `85c3e82`, 2026-07-08

## Why this matters

`AGENTS.md` documents a manual invariant: website locales
(`www/src/lib/marketing-copy.json`) and app menu translations
(`src-tauri/src/i18n.rs`) must be kept aligned. Nothing enforces it, and
drift has already happened once at a smaller scale: `marketing-copy.json`
contains per-locale `home.title`/`home.description` blocks that **nothing
reads** (verified: `www/src/lib/translations.ts` — the only importer — never
references the `home` key), while the *actual* home meta lives in
`www/src/content/home/*.json` and has diverged (marketing-copy's English title
is "Better Resource Monitor - Menu Bar Monitor for macOS"; the live one in
`content/home/en.json` is "iStat Menus Alternative for macOS | Better Resource
Monitor"). Editors updating the dead copy see no effect and can't tell which
file is canonical.

Separately, adding a locale requires touching many files (README.<locale>.md,
four content collections, marketing-copy.json, i18n.rs) with no automated
check that all of them were updated — a missing file either breaks the build
opaquely or silently omits pages.

Current parity state (verified 2026-07-08): all four locales
(`en`, `es`, `pt-br`, `zh-cn`) are present and structurally consistent across
faq (9 identical item ids), home, ui, comparisons (11-row tables everywhere).
So the new check passes immediately — this plan is prevention plus deleting
the one dead duplicate.

## Current state

- `www/src/lib/marketing-copy.json` — per-locale objects with keys
  `badges`, `home`, `meta`, `ogTitles`, `screenshotLang`, `screenshots`
  (same shape in all four locales). The `home` key is dead (see above).
- `www/src/lib/translations.ts` — sole importer of marketing-copy.json;
  derives `siteMarketingLocales` from `Object.keys(marketingCopy)`; exports
  `localeMeta`, `ogTitles`, `localizedBadges`, screenshot metadata. No `home`
  reference anywhere in the file.
- `www/src/content/` — collections `home`, `faq`, `ui`, `comparisons/<slug>`,
  `legal`, each with one file per locale; schemas in
  `www/src/content.config.ts` validate files individually (plain Zod objects,
  no cross-locale or cross-array refinements). The `homeBody` loader
  (content.config.ts:59-98) reads `README.md` / `README.<locale>.md` per
  locale — a missing README fails the build loudly, good.
- `src-tauri/src/i18n.rs` — `enum Language { English, Spanish, Portuguese, Chinese }`,
  and:

```80:90:src-tauri/src/i18n.rs
fn language_for_locale(locale: &str) -> Option<Language> {
    let prefix = locale.split(['-', '_']).next()?.to_ascii_lowercase();

    match prefix.as_str() {
        "en" => Some(Language::English),
        "es" => Some(Language::Spanish),
        "pt" => Some(Language::Portuguese),
        "zh" => Some(Language::Chinese),
    }
}
```

  (plus `_ => None`). App locale keys are prefixes; site keys are
  `en`/`es`/`pt-br`/`zh-cn`. The invariant to encode: every site locale's
  prefix must map to a `Language`, and every `Language` must be reachable
  from at least one site locale.
- `scripts/` — bash scripts only today; a Node `.mjs` script is acceptable
  (the repo already has `www/generate-badges.mjs`, `www/legacy-redirects.mjs`
  as plain-Node ESM exemplars).
- CI: after plan 009, `.github/workflows/ci.yml` runs
  `pnpm --filter www check` and `pnpm build:www`.

## Commands you will need

| Purpose | Command (repo root) | Expected on success |
|---|---|---|
| Site build | `pnpm build:www` | exit 0, 33 pages |
| Type check | `pnpm --filter www check` (after plan 009) or `cd www && pnpm exec astro check` | 0 errors |
| New parity check | `node scripts/check-locale-parity.mjs` (created in Step 2) | exit 0 |
| Rust tests | `cargo test --manifest-path src-tauri/Cargo.toml` | exit 0 |

## Scope

**In scope**:
- `www/src/lib/marketing-copy.json` (delete the dead `home` keys, all four locales)
- `scripts/check-locale-parity.mjs` (create)
- `www/src/content.config.ts` (only if you choose schema refinements over the script — prefer the script; see Step 2)
- `.github/workflows/ci.yml` (one step, next to the website steps)
- `AGENTS.md` (one sentence pointing at the check)
- `plans/README.md` (status row)

**Out of scope** (do NOT touch):
- `src-tauri/src/i18n.rs` — read-only input to the check; menu strings stay as they are.
- `www/src/content/**` translations content — no copy edits.
- `www/src/lib/embedded-faqs.ts` — moving hardcoded FAQ copy into collections
  is a separate refactor, deliberately not in this plan.
- `README*.md` content (plan 006 owns distribution copy).

## Git workflow

- Branch: `fix/locale-parity-check`
- Conventional commits, e.g. `test: add cross-locale parity check` / `chore(www): remove dead home meta from marketing-copy`
- Do NOT push or open a PR unless the operator instructed it.

## Steps

### Step 1: Delete the dead `home` blocks from `marketing-copy.json`

Remove the `"home": { "title": ..., "description": ... }` object from each of
the four locale objects (`en`, `es`, `pt-br`, `zh-cn`). Canonical home meta
remains `www/src/content/home/<locale>.json` (consumed by
`HomePage.astro` via `getEntry("home", locale)`).

**Verify**:
- `rg -n '"home"' www/src/lib/marketing-copy.json` → no matches
- `pnpm --filter www check` (or `cd www && pnpm exec astro check`) → 0 errors
  (this proves nothing typed against the `home` key — `translations.ts` derives
  types from the JSON, so a stale reference would fail here)
- `pnpm build:www` → exit 0, 33 pages

### Step 2: Create `scripts/check-locale-parity.mjs`

A plain Node ESM script (no dependencies) that exits non-zero with a clear
message on the first violation. Checks, in order:

1. **Canonical locale set**: read `www/src/lib/marketing-copy.json`; its
   top-level keys are the canonical locales. Assert they equal
   `["en", "es", "pt-br", "zh-cn"]`? No — do NOT hardcode; treat the JSON as
   canonical and check everything else against it.
2. **Uniform marketing-copy shape**: every locale object has exactly the same
   key set as `en` (recursively one level deep for `badges`, `meta`,
   `ogTitles`, `screenshots`).
3. **Content collections complete**: for each of
   `www/src/content/{home,faq,ui}` — one `<locale>.json` per canonical locale,
   no extra locale files. For `www/src/content/comparisons/<slug>/` — same,
   per slug. For `www/src/content/legal/` — mirror however files are organized
   (list the directory first; it may be per-locale subdirs or `.md` suffixes —
   adapt, and assert one file per locale per document).
4. **READMEs**: `README.md` exists, and `README.<locale>.md` exists for each
   non-`en` locale.
5. **FAQ id parity**: `items[].id` arrays in `faq/<locale>.json` are identical
   (same ids, same order) to `en`.
6. **Comparison table row parity**: for every comparison file,
   `table.features.length === table.productValues.length === table.competitorValues.length`,
   and each locale's `features.length` equals the `en` file's for the same slug.
7. **App i18n coverage**: read `src-tauri/src/i18n.rs` as text. Extract the
   quoted prefixes from the `match prefix.as_str()` block in
   `language_for_locale` (regex on lines between `match prefix.as_str()` and
   the closing brace: `/"([a-z-]+)" => Some\(/g`). Assert every canonical
   locale's prefix (`locale.split("-")[0]`) appears in that set. This is a
   source-text check — brittle by design; if the regex finds zero prefixes,
   fail with "could not parse i18n.rs — update check-locale-parity.mjs" rather
   than passing vacuously.

Style: mirror `www/legacy-redirects.mjs` / `www/generate-badges.mjs` (plain
ESM, `node:fs`, no deps). Print one line per passed section and a final
`Locale parity OK (<n> locales)`.

**Verify**: `node scripts/check-locale-parity.mjs` → exit 0, prints OK line.
Then prove it can fail: temporarily rename `www/src/content/faq/es.json`,
rerun → non-zero exit with a message naming the missing file; restore the file.

### Step 3: Wire the check into CI

In `.github/workflows/ci.yml`, next to the website steps (added by plan 009 —
if plan 009 hasn't landed, place it after "Install frontend dependencies"):

```yaml
      - name: Check locale parity
        run: node scripts/check-locale-parity.mjs
```

**Verify**: YAML parses (`python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml'))"`) → exit 0.

### Step 4: Document the invariant's enforcement

In `AGENTS.md`, in the "Wiring And Gotchas" bullet about keeping
`marketing-copy.json` and `i18n.rs` aligned, append one sentence:
"`scripts/check-locale-parity.mjs` (run in CI) enforces the structural half of
this; translated wording still needs human review."

**Verify**: `rg -n "check-locale-parity" AGENTS.md` → 1 match.

## Test plan

The script is itself the test. Beyond the rename-a-file negative test in
Step 2, run these mutations (each: mutate → run script → expect failure →
revert):
- Remove one id from `www/src/content/faq/zh-cn.json` items → fails on id parity.
- Remove one entry from `table.competitorValues` in
  `www/src/content/comparisons/vs-stats/pt-br.json` → fails on row parity.
- Add a bogus locale key `"fr": {}` to `marketing-copy.json` → fails on
  README or content-collection completeness.
- `cargo test --manifest-path src-tauri/Cargo.toml` still exits 0 (nothing in
  Rust changed).

## Done criteria

- [ ] `rg -n '"home"' www/src/lib/marketing-copy.json` → no matches
- [ ] `node scripts/check-locale-parity.mjs` → exit 0 on the clean tree
- [ ] All four negative mutations above make the script exit non-zero with a
      message naming the violation, and are reverted
- [ ] `pnpm build:www` → exit 0; `astro check` → 0 errors
- [ ] CI workflow contains the parity step
- [ ] AGENTS.md mentions the script
- [ ] No files outside the in-scope list are modified (`git status`)
- [ ] `plans/README.md` status row updated

## STOP conditions

Stop and report back (do not improvise) if:

- `rg -n '"home"' www/src/lib/marketing-copy.json`'s deletion breaks
  `astro check` or the build — that means something started consuming the key
  since this plan was written; report the consumer.
- The `language_for_locale` match in `src-tauri/src/i18n.rs` no longer looks
  like the excerpt (structure changed) — adapt the regex only if the mapping
  is still a literal match block; otherwise report.
- The legal content collection's file layout doesn't map cleanly to
  one-file-per-locale — describe the actual layout and propose the check
  shape instead of guessing.

## Maintenance notes

- When a new locale is added, the script tells the contributor every file they
  must create — that's its main value. Keep its error messages actionable.
- The check intentionally does not diff translated *wording* between the app
  menu and the site (marketing register differs from menu labels); don't
  extend it that way without a maintainer decision.
- Deferred: moving `www/src/lib/embedded-faqs.ts` hardcoded copy into content
  collections (would let the parity check cover it); consolidating the
  redirect locale lists (`www/legacy-redirects.mjs`, `www/public/_redirects`,
  `www/src/lib/site.ts`) — see the original audit notes in Git history at
  commit `18c36cc`.
