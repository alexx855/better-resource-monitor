# Plan 006: Make direct-download distribution real or retire its public promises

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving to the
> next step. If anything in the "STOP conditions" section occurs, stop and
> report — do not improvise. When done, update the status row for this plan
> in `plans/README.md` — unless a reviewer dispatched you and told you they
> maintain the index.
>
> **Drift check (run first)**:
> `git diff --stat 85c3e82..HEAD -- src-tauri/src/lib.rs src-tauri/src/tests.rs README.md README.es.md README.pt-br.md README.zh-cn.md www/src/content/faq www/public/llms.txt www/public/agent-facts.json .github/workflows/release.yml .github/workflows/direct-download.yml`
> If any in-scope file changed since this plan was written, compare the
> "Current state" excerpts against the live code before proceeding; on a
> mismatch, treat it as a STOP condition.

## Status

- **Priority**: P1
- **Effort**: M
- **Risk**: MED
- **Depends on**: none (but read plan 007 first if you touch `direct-download.yml`; it changes the same file)
- **Category**: bug / direction
- **Planned at**: commit `85c3e82`, 2026-07-08
- **Supersedes**: `plans/001-align-public-distribution-copy.md` (written before the
  `direct-download.yml` workflow and the runtime-guard incompatibility were known)

## Decision gate — read this first

This plan has two mutually exclusive paths. **Before executing, the maintainer
must pick one.** If you were dispatched without an explicit choice, STOP and ask.

- **Path A — ship DMGs for real**: teach the runtime guard to accept
  Developer ID (notarized, non-App-Store) builds, and wire the existing
  `direct-download.yml` workflow into the release flow so GitHub releases
  actually carry a DMG.
- **Path B — App Store only**: remove the `.dmg` promise from all public copy
  and machine-readable facts, and mark the direct-download workflow as
  dormant/experimental.

Steps are labeled `[A]`, `[B]`, or `[both]`.

## Why this matters

The repo currently promises an install path that cannot work:

1. `README.md` tells users to "grab the `.dmg` from GitHub Releases", but the
   latest release (v1.1.4) has **zero binary assets** — verified via
   `gh release view v1.1.4` on 2026-07-08. `release.yml` explicitly creates
   "source-only" releases, and `direct-download.yml` (which builds signed DMGs)
   is manual-dispatch only and has never been run for a public release.
2. Worse: even if a DMG were published today, the app inside it would **silently
   exit at launch**. The macOS runtime guard exits any binary with the production
   bundle ID that lacks a Mac App Store receipt — and the direct-download build
   uses the production bundle ID (`tauri.conf.json`) with no receipt. Installing
   the DMG to `/Applications` produces an app that `exit(0)`s immediately with
   no user-visible error. Nothing in CI launches the built app, so this ships
   unnoticed.

Users following the documented install path today get nothing; after a naive
"just publish the DMG" fix they'd get a broken app. Either outcome erodes trust
in a product whose marketing leans on openness.

## Current state

Relevant files:

- `src-tauri/src/lib.rs` — the runtime guard (lines ~301–381) and its constants (lines ~203–208).
- `src-tauri/src/tests.rs` — guard tests `test_supported_macos_bundle_runtime` (lines ~225–259).
- `.github/workflows/release.yml` — automated release; job near line 249 creates a source-only GitHub release.
- `.github/workflows/direct-download.yml` — manual workflow that builds, signs, notarizes, and optionally uploads DMG/ZIP artifacts to an existing release.
- `.github/actions/build-direct-download/action.yml` — composite action doing the actual build (`pnpm tauri build --bundles app --target universal-apple-darwin` at line 88, using plain `tauri.conf.json`, i.e. production bundle ID `dev.alexpedersen.better-resource-monitor`).
- Public copy claiming DMG/GitHub distribution:
  - `README.md:25` — "Download macOS on GitHub Releases" badge.
  - `README.md:50` — "The Mac App Store build and GitHub build are the same app."
  - `README.md:64` — "grab the `.dmg` from GitHub Releases".
  - `README.es.md`, `README.pt-br.md`, `README.zh-cn.md` — localized equivalents (search each for `releases` and `dmg`).
  - `www/src/content/faq/en.json:20` — "The Mac App Store and GitHub versions are the same app." (also `:30` mentions "GitHub versions"); same items exist in `es.json`, `pt-br.json`, `zh-cn.json`.
  - `www/public/llms.txt:8` — "Availability: Mac App Store and GitHub source releases".
  - `www/public/agent-facts.json` — `agentPolicy.notes[1]`: "GitHub Releases currently provide source releases; use the Mac App Store for the signed app until direct-download artifacts are published."
  - Note: the README bodies are rendered into the website home pages by `www/src/content.config.ts` (readme-loader), so README edits change the live site.

The guard as it exists today:

```203:208:src-tauri/src/lib.rs
const MACOS_BUNDLE_ID: &str = "dev.alexpedersen.better-resource-monitor";
#[cfg(target_os = "macos")]
const MACOS_SUPPORTED_EXECUTABLE_PATH: &str =
    "/Applications/Better Resource Monitor.app/Contents/MacOS/better-resource-monitor";
```

```301:309:src-tauri/src/lib.rs
fn should_exit_unsupported_macos_bundle(
    bundle_id: Option<&str>,
    executable_path: &std::path::Path,
    has_app_store_receipt: bool,
) -> bool {
    bundle_id == Some(MACOS_BUNDLE_ID)
        && (executable_path != std::path::Path::new(MACOS_SUPPORTED_EXECUTABLE_PATH)
            || !has_app_store_receipt)
}
```

`enforce_supported_macos_runtime()` (lib.rs ~352–381) computes
`has_app_store_receipt` by checking for
`<App>.app/Contents/_MASReceipt/receipt` on disk, and calls
`std::process::exit(0)` when the guard returns true. It runs unconditionally at
the top of `run()` (lib.rs ~1108). The guard exists to stop stray copies (e.g.
Trash, dev builds with the production bundle ID) from running; the tests at
`src-tauri/src/tests.rs:225-259` encode the intended matrix.

Direct-download builds embed a commit via `BRM_BUILD_COMMIT` (see
`build_commit()` at lib.rs ~438 and the verification in
`.github/actions/build-direct-download/action.yml:95-99`) — that mechanism is
available if a build-time marker is needed.

Repo conventions: Rust changes need `cargo fmt`, `cargo test`, `cargo clippy`
from `src-tauri/`. Guard logic is pure-function + tests — follow the existing
pattern of `should_exit_unsupported_macos_bundle` + table-driven test.
Commit style is conventional commits (e.g. `fix: avoid nested tray main-thread dispatch`).

## Commands you will need

| Purpose | Command (from repo root) | Expected on success |
|---|---|---|
| Rust fmt | `cargo fmt --manifest-path src-tauri/Cargo.toml --check` | exit 0 |
| Rust tests | `cargo test --manifest-path src-tauri/Cargo.toml` | exit 0, all pass |
| Rust lint | `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` | exit 0 |
| Website build | `pnpm install && pnpm build:www` | exit 0 (needs network for fonts) |
| Workflow syntax | `actionlint .github/workflows/*.yml` (if installed; otherwise skip) | no errors |

## Scope

**In scope**:
- `src-tauri/src/lib.rs` (Path A only — guard logic)
- `src-tauri/src/tests.rs` (Path A only — guard tests)
- `README.md`, `README.es.md`, `README.pt-br.md`, `README.zh-cn.md`
- `www/src/content/faq/{en,es,pt-br,zh-cn}.json`
- `www/public/llms.txt`, `www/public/agent-facts.json`
- `.github/workflows/release.yml` (Path A only — dispatch direct-download)
- `docs/github-release-workflows.md` (update prose to match)
- `plans/README.md` (status row)

**Out of scope** (do NOT touch):
- `.github/workflows/testflight.yml` and signing scripts — plan 007 owns workflow security changes.
- `scripts/build-appstore-bundle.sh`, `tauri.appstore.conf.json`, App Store entitlements — App Store path is working; don't disturb it.
- `www/src/lib/marketing-copy.json` badges — badge asset changes are a separate concern (see plan 010 for locale copy).
- Version numbers anywhere.

## Git workflow

- Branch: `fix/direct-download-distribution` (repo uses `fix/**` branches; CI runs on them)
- Conventional commits; one commit per step is fine.
- Do NOT push or open a PR unless the operator instructed it.

## Steps — Path A (ship DMGs)

### Step A1: Give Developer ID builds a legitimate way past the guard

In `src-tauri/src/lib.rs`, extend the guard so a notarized direct-download
build is accepted. Recommended mechanism, consistent with the existing
`BRM_BUILD_COMMIT` pattern: a compile-time env marker.

- Add `const DIRECT_DISTRIBUTION: bool = option_env!("BRM_DIRECT_DISTRIBUTION").is_some();`
  (or read it inside the guard via a parameter, keeping
  `should_exit_unsupported_macos_bundle` pure — add a `is_direct_distribution: bool`
  parameter and pass the constant from `enforce_supported_macos_runtime`).
- New guard semantics: exit iff `bundle_id == MACOS_BUNDLE_ID` AND NOT
  (App-Store-shaped: supported path + receipt) AND NOT
  (direct-distribution build: `is_direct_distribution && executable_path == MACOS_SUPPORTED_EXECUTABLE_PATH`).
  Keep rejecting Trash/duplicate paths for direct builds too — that's the
  guard's original purpose.
- Then set `BRM_DIRECT_DISTRIBUTION=1` in the build environment of
  `.github/actions/build-direct-download/action.yml` (the `Build app bundle`
  step, next to `BRM_BUILD_COMMIT`). App Store builds
  (`scripts/build-appstore-bundle.sh`) must NOT set it.

**Verify**: `cargo test --manifest-path src-tauri/Cargo.toml` → all pass
(after Step A2 adds the new cases).

### Step A2: Extend the guard test matrix

In `src-tauri/src/tests.rs`, extend `test_supported_macos_bundle_runtime` with:
- direct build (`is_direct_distribution = true`), supported path, no receipt → do NOT exit
- direct build, Trash path, no receipt → exit
- non-direct build, supported path, no receipt → exit (existing behavior preserved)
- App Store build (receipt), supported path → do NOT exit (existing)

**Verify**: `cargo test --manifest-path src-tauri/Cargo.toml test_supported_macos_bundle_runtime` → passes.

### Step A3: Chain Direct Download into the release flow

In `.github/workflows/release.yml`, after the source-only release is created
(job containing "Create source-only GitHub release", ~line 249), add a step or
job that dispatches the Direct Download workflow for the new tag:

```yaml
- name: Dispatch direct-download artifacts
  env:
    GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
  run: |
    gh workflow run direct-download.yml --ref main \
      -f tag="v${{ needs.version-bump.outputs.new_version }}" \
      -f upload_to_release=true
```

Note: the default `GITHUB_TOKEN` may not be permitted to dispatch workflows
depending on repo settings; if the dispatch step fails with a permissions
error, this is a STOP condition (a PAT or `workflow` permission decision
belongs to the maintainer). Also update the release body text that says
"source-only" if it would now be inaccurate.

**Verify**: `actionlint .github/workflows/release.yml` (or careful YAML review)
→ no syntax errors. Do not trigger an actual release.

### Step A4: Align the copy with reality

Update the copy so it matches "App Store (auto-updates) or notarized DMG from
GitHub Releases (manual updates)":
- `README.md:64` is already accurate for Path A — keep, but verify the
  "same app" claims at `README.md:50` are still true (they are, once A1 lands).
- `www/public/llms.txt:8` → "Availability: Mac App Store and GitHub releases (signed DMG)".
- `www/public/agent-facts.json` `agentPolicy.notes[1]` → drop the "until
  direct-download artifacts are published" clause once the first DMG release
  exists; if executing before that, rephrase to "signed DMG artifacts are
  attached to releases starting v<next>".
- Check the localized READMEs and FAQ JSONs mention the same story.

**Verify**: `pnpm build:www` → exit 0. `rg -n "source releases" www/public/llms.txt` → no match.

## Steps — Path B (App Store only)

### Step B1: Remove the DMG promise from the README family

- `README.md:25` — remove the GitHub-releases download badge line.
- `README.md:50` — rewrite the "same app" FAQ answer to "Yes. MIT-licensed and
  free. Install it from the Mac App Store; the full source is on GitHub."
- `README.md:64` — rewrite Installation to point only at the App Store, keeping
  "Build from Source" below as the open-source path.
- Apply the same edits to `README.es.md`, `README.pt-br.md`, `README.zh-cn.md`
  (find the corresponding lines by searching each file for `releases` / `dmg` /
  `apps.apple.com`). Translate in the same register as the surrounding text.

**Verify**: `rg -n "dmg" README*.md -i` → no matches referring to downloads.
`pnpm build:www` → exit 0 (README changes flow into the site build).

### Step B2: Fix the machine-readable and FAQ surfaces

- `www/public/llms.txt:8` → "Availability: Mac App Store (source code on GitHub)".
- `www/public/agent-facts.json` `agentPolicy.notes[1]` → "GitHub Releases are
  source-only; install the app from the Mac App Store."
- `www/src/content/faq/{en,es,pt-br,zh-cn}.json` — update the "same app"
  answers (en.json items with ids around lines 20 and 30) to remove "GitHub
  versions" phrasing.

**Verify**: `pnpm build:www` → exit 0. `python3 -m json.tool www/public/agent-facts.json` → valid JSON.

### Step B3: Mark direct-download dormant

Add a comment block at the top of `.github/workflows/direct-download.yml`:
"Dormant: direct-download DMGs are not currently published. The runtime guard
in src-tauri/src/lib.rs (should_exit_unsupported_macos_bundle) exits non-App-Store
builds with the production bundle ID; see plans/006 for the reactivation path."
Do not delete the workflow. Update `docs/github-release-workflows.md` to note
the dormant status.

**Verify**: `rg -n "Dormant" .github/workflows/direct-download.yml` → 1 match.

## Test plan

- Path A: extended table-driven cases in `test_supported_macos_bundle_runtime`
  (see Step A2), modeled on the existing tuples at `src-tauri/src/tests.rs:228-247`.
- Path B: no Rust changes; verification is `pnpm build:www` plus greps above.
- Both: `cargo fmt --check`, `cargo test`, `cargo clippy -- -D warnings` all exit 0.

## Done criteria

- [ ] Decision path chosen and recorded in the plan's status row (`DONE (Path A)` or `DONE (Path B)`)
- [ ] `cargo test --manifest-path src-tauri/Cargo.toml` exits 0
- [ ] `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` exits 0
- [ ] `pnpm build:www` exits 0
- [ ] Path A: guard accepts direct builds at the supported path without a receipt (test proves it); `release.yml` dispatches `direct-download.yml`
- [ ] Path B: `rg -in "grab the .?dmg" README*.md` returns no matches; `llms.txt` and `agent-facts.json` no longer promise binary GitHub downloads
- [ ] No files outside the in-scope list are modified (`git status`)
- [ ] `plans/README.md` status rows updated (this plan, and 001 marked SUPERSEDED)

## STOP conditions

Stop and report back (do not improvise) if:

- No decision (Path A vs B) was provided with the dispatch.
- The guard code at `src-tauri/src/lib.rs:301-309` no longer matches the excerpt above.
- Path A: the `gh workflow run` dispatch in release.yml would require new
  repository secrets or permissions you cannot verify.
- Path A: you find any other code path that assumes "production bundle ID
  implies App Store" (search `MACOS_BUNDLE_ID` usages) beyond the guard —
  autostart and diagnostics use the bundle ID too; if their behavior would
  change for direct builds, report before proceeding.
- The localized READMEs have structurally diverged from `README.md` (missing
  the Installation section) — report rather than restructure them.

## Maintenance notes

- The pre-merge TestFlight gate (AGENTS.md "Release Notes") still applies to
  Path A's Rust change — the guard runs at startup in the App Store build, and
  a regression here bricks the shipped app. Reviewer must scrutinize the guard
  truth table.
- If Path A lands, the first real release should be manually smoke-tested:
  download the DMG from the release page, install to /Applications, launch,
  confirm the tray appears (this is exactly the failure mode CI cannot see).
- `www/public/agent-facts.json` and `llms.txt` are hand-maintained duplicates
  of distribution facts; a future improvement (see direction findings in
  `plans/README.md`) is generating both from one manifest.
