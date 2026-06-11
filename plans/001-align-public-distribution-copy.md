# Plan 001: Align Public Distribution Copy With The App Store Runtime

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving to the
> next step. If anything in the "STOP conditions" section occurs, stop and
> report. When done, update the status row for this plan in `plans/README.md`.
>
> **Drift check (run first)**:
> `git diff --stat d3646b8..HEAD -- README.md README.es.md README.pt-br.md README.zh-cn.md www/src/content.config.ts www/src/content www/src/lib/marketing-copy.json www/public/llms.txt www/public/agent-facts.json www/generate-badges.mjs www/public/badges .github/workflows/release.yml src-tauri/src/lib.rs docs`
>
> If any in-scope file changed since this plan was written, compare the
> "Current state" excerpts against the live code before proceeding; on a
> mismatch, treat it as a STOP condition.

## Status

- **Priority**: P1
- **Effort**: M
- **Risk**: MED
- **Depends on**: none
- **Category**: docs
- **Planned at**: commit `d3646b8`, 2026-06-11

## Why this matters

The public README and website currently tell users they can download a macOS
`.dmg` from GitHub Releases. The executable release workflow now creates a
source-only GitHub release after the App Store/TestFlight upload, and the macOS
runtime exits for the production bundle id unless the app is in the supported
App Store path and has a receipt. That makes the current public installation
copy actively misleading: users who follow the GitHub path will not get the
distribution the runtime is designed to support.

This plan makes the product promise match the executable release shape: Mac App
Store is the install path; GitHub Releases remain source/provenance only unless
the release workflow and runtime guard are intentionally changed later.

## Current state

- `README.md` and localized READMEs are loaded into the website home body by
  `www/src/content.config.ts`, so README install copy is also website copy.
- `.github/workflows/release.yml` creates a source-only GitHub release.
- `src-tauri/src/lib.rs` enforces the App Store/TestFlight receipt/runtime guard.
- `www/src/content/**/*.json`, `www/public/llms.txt`, and generated badge copy
  also need to be searched because FAQ and comparison pages are public
  marketing surfaces.

Important excerpts:

```text
README.md:24-25
<a href="https://apps.apple.com/app/better-resource-monitor/id6758237306" ...>
<a href="https://github.com/alexx855/better-resource-monitor/releases" ... alt="Download macOS on GitHub Releases" ...>

README.md:44
Get it from the Mac App Store ... or grab the `.dmg` from GitHub Releases ...
```

```text
www/src/content.config.ts:66-69
for (const locale of siteMarketingLocales) {
  const filename = locale === "en" ? "README.md" : `README.${locale}.md`;
  const raw = await readFile(resolve(rootDir, filename), "utf-8");
```

```text
.github/workflows/release.yml:246-270
- name: Create source-only GitHub release
...
gh release create "v${{ needs.version-bump.outputs.new_version }}" \
  --title "Better Resource Monitor v${{ needs.version-bump.outputs.new_version }}" \
  --notes-file release_notes.md
```

```text
src-tauri/src/lib.rs:178-183
const MACOS_BUNDLE_ID: &str = "dev.alexpedersen.better-resource-monitor";
const MACOS_SUPPORTED_EXECUTABLE_PATH: &str =
    "/Applications/Better Resource Monitor.app/Contents/MacOS/better-resource-monitor";

src-tauri/src/lib.rs:275-283
fn should_exit_unsupported_macos_bundle(...) -> bool {
    bundle_id == Some(MACOS_BUNDLE_ID)
        && (executable_path != std::path::Path::new(MACOS_SUPPORTED_EXECUTABLE_PATH)
            || !has_app_store_receipt)
}
```

```text
www/src/content/faq/en.json:20
"The Mac App Store and GitHub versions are the same app."

www/src/content/faq/en.json:30
"... available in both the App Store and GitHub versions ..."

www/public/llms.txt:8
- Availability: Mac App Store and GitHub releases

www/src/lib/marketing-copy.json:33-35
"macos": {
  "topText": "DOWNLOAD FROM",
  "bottomText": "GitHub Releases"
}
```

Repo conventions to preserve:

- Current locale keys are `en`, `es`, `pt-br`, and `zh-cn`.
- Website locales live in JSON content under `www/src/content` and
  `www/src/lib/marketing-copy.json`; app menu translations are separate and
  out of scope for this copy-only change.
- README changes affect the website home body through the README loader.
- Trust executable config and workflows over old prose docs.

## Commands you will need

| Purpose | Command | Expected on success |
| --- | --- | --- |
| Search current claims | `rg -n "GitHub Releases|GitHub versions|GitHub version|\\.dmg|Availability" README*.md www/src/content www/src/lib/marketing-copy.json www/public/llms.txt www/public/agent-facts.json docs` | Only source/provenance references remain after edits; no GitHub app-download claims |
| Regenerate badges if badge copy changes | `node www/generate-badges.mjs` | exit 0; generated badge paths logged |
| Build site | `pnpm build:www` | exit 0; Astro build completes |
| Check working tree scope | `git status --short` | Only in-scope files are modified |

## Scope

**In scope**:

- `README.md`
- `README.es.md`
- `README.pt-br.md`
- `README.zh-cn.md`
- All public content collection files under `www/src/content/**/*.json`,
  including FAQ files, comparison pages under `www/src/content/comparisons/`,
  and any other localized marketing content where stale distribution copy is
  found
- `www/src/lib/marketing-copy.json`
- `www/generate-badges.mjs`, only if badge labels or generated badge coverage change
- `www/public/badges/*`, only generated badge files affected by this copy change
- `www/public/llms.txt`
- `www/public/agent-facts.json`, only if it presents GitHub Releases as app availability rather than source/provenance
- `docs/*`, only to remove directly contradictory GitHub installer claims if found

**Out of scope**:

- Do not change `src-tauri/src/lib.rs`, release workflows, signing, entitlements,
  or App Store packaging behavior.
- Do not add a GitHub DMG upload path as part of this plan.
- Do not change app menu translations in `src-tauri/src/i18n.rs`; this is public
  installation/crawl copy only.
- Do not add new locales.

## Git workflow

- Branch: `codex/001-align-distribution-copy`
- Commit style: conventional commits. Example from repo history:
  `docs: align App Store release references`
- Do not push or open a PR unless the operator instructed it.

## Steps

### Step 1: Make the README installation path App Store-only

Update all four root READMEs so the installation section says the installable
signed app is available from the Mac App Store. Keep a separate source or
release-provenance sentence if useful, but do not describe GitHub Releases as a
place to download a `.dmg` or installable app.

Remove the GitHub "Download macOS on GitHub Releases" badge anchors from the
README badge blocks, or replace them with non-download source/provenance copy.
If you keep a GitHub badge, its alt text and rendered label must not say
"Download".

Preserve the App Store badge and all existing locale links.

**Verify**:
`rg -n "Download.*GitHub Releases|Descargar.*GitHub Releases|Baixar.*GitHub Releases|GitHub Releases.*\\.dmg|\\.dmg" README*.md`
returns no matches.

### Step 2: Align content collection copy in every locale

Update the pricing and GPU monitoring FAQ answers in:

- `www/src/content/faq/en.json`
- `www/src/content/faq/es.json`
- `www/src/content/faq/pt-br.json`
- `www/src/content/faq/zh-cn.json`

Target meaning:

- The app is free and MIT-licensed.
- The Mac App Store build is the supported installable app.
- GitHub contains source code and release provenance.
- GPU monitoring stays available in the App Store build through the public
  IOAccelerator API.

Do not claim there are matching "App Store and GitHub versions" unless plan
owners have separately changed the workflow and runtime guard to ship a GitHub
installer.

Then search all content collections under `www/src/content`, including
comparison pages under `www/src/content/comparisons/`, and update any stale
distribution claims found there. Keep copy changes limited to the same App
Store install vs GitHub source/provenance distinction.

**Verify**:
`rg -n "GitHub versions|GitHub version|App Store and GitHub versions|both the App Store and GitHub" www/src/content`
returns no matches.

### Step 3: Fix machine-readable and badge copy

Update `www/public/llms.txt` so `Availability` means install availability, not
source availability. A good target is:

```text
- Availability: Mac App Store
```

Keep the GitHub and Releases URLs in the "Primary URLs" or "Preferred sources"
sections if they are presented as source/provenance references.

Then handle `www/src/lib/marketing-copy.json`:

- If the GitHub release badges are no longer used anywhere, remove the stale
  `macos` and `ubuntu` download-badge definitions and delete the generated
  `www/public/badges/macos*` and `www/public/badges/ubuntu*` assets.
- If a source/provenance badge is intentionally kept, change both `topText` and
  `bottomText` so the badge cannot be read as an app download. Then run
  `node www/generate-badges.mjs` and commit the updated generated badge assets.

**Verify**:
`rg -n "\"DOWNLOAD FROM\"|\"GitHub Releases\"" www/src/lib/marketing-copy.json`
returns no matches unless the remaining text is explicitly source/provenance
copy and not a download prompt.

### Step 4: Build the website and inspect generated home content

Run the site build from the repo root:

```bash
pnpm build:www
```

Expected: exit 0. Astro may print the existing Cloudflare adapter warning about
sessions/KV, but the build must complete.

Then search the generated site output for stale install claims:

```bash
rg -u -n "GitHub versions|Download macOS on GitHub Releases|\\.dmg|Availability: Mac App Store and GitHub" www/dist README*.md
```

Expected: no stale app-download claims. GitHub source/provenance links may
remain if the surrounding text is correct.

## Test plan

- This is copy and generated-asset work, so there are no Rust unit tests to add.
- The regression test is the combination of targeted `rg` searches and
  `pnpm build:www`.
- If badge assets are changed, run `node www/generate-badges.mjs` before
  `pnpm build:www` and review the generated image file names in `git status`.

## Done criteria

- [ ] README install sections in all four locales present Mac App Store as the
      supported install path.
- [ ] README badge blocks no longer contain a GitHub app-download badge.
- [ ] FAQ answers in all four locales no longer refer to "GitHub versions" of
      the installable app.
- [ ] `www/public/llms.txt` no longer says availability is "Mac App Store and
      GitHub releases" unless that phrase is changed to source/provenance.
- [ ] Generated badge definitions/assets do not advertise GitHub Releases as an
      app download.
- [ ] `pnpm build:www` exits 0.
- [ ] `git status --short` shows only in-scope files.
- [ ] `plans/README.md` status row updated.

## STOP conditions

Stop and report if:

- The release workflow has been changed to upload signed macOS installer assets
  to GitHub Releases since commit `d3646b8`.
- `src-tauri/src/lib.rs` no longer exits for non-receipted production macOS
  bundles.
- The operator wants GitHub installer distribution restored instead of removing
  stale claims. That is a product/release plan, not this copy plan.
- A localized README has diverged so much that the target install copy cannot be
  confidently translated.

## Maintenance notes

- Future release-workflow changes should update the README, FAQ, `llms.txt`,
  `agent-facts.json`, and generated badges in the same PR.
- Reviewers should scrutinize the distinction between "source releases on
  GitHub" and "install the app from GitHub"; the former can remain, the latter
  is currently false.
- If a GitHub DMG distribution is intentionally reintroduced, it needs its own
  implementation plan covering signing, runtime receipt/path behavior,
  notarization, release asset upload, and update expectations.
