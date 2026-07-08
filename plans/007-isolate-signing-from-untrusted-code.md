# Plan 007: Isolate Apple signing credentials from untrusted code in CI

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving to the
> next step. If anything in the "STOP conditions" section occurs, stop and
> report — do not improvise. When done, update the status row for this plan
> in `plans/README.md` — unless a reviewer dispatched you and told you they
> maintain the index.
>
> **Drift check (run first)**:
> `git diff --stat 85c3e82..HEAD -- .github/workflows/testflight.yml .github/workflows/direct-download.yml .github/actions/build-direct-download/action.yml scripts/setup-appstore-signing.sh scripts/setup-developer-id-signing.sh scripts/upload-appstore-package.sh`
> If any in-scope file changed since this plan was written, compare the
> "Current state" excerpts against the live code before proceeding; on a
> mismatch, treat it as a STOP condition.

## Status

- **Priority**: P1
- **Effort**: M
- **Risk**: MED
- **Depends on**: none (coordinate with plan 006 if both touch `direct-download.yml`; land this one first)
- **Category**: security
- **Planned at**: commit `85c3e82`, 2026-07-08

## Why this matters

Two workflows execute code an attacker can influence in the same runner
process lifetime that later holds Apple signing credentials:

1. **TestFlight** (`testflight.yml`): the repo's documented pre-merge gate has
   the maintainer dispatch this workflow with `source_ref=refs/pull/N/head` —
   i.e. deliberately running PR code. The job runs `pnpm install` (lifecycle
   scripts), `cargo test` (PR-controlled `build.rs` and test code), and the
   full app build from that untrusted checkout — and **then**, in the same
   job, writes the Apple distribution certificate, installer certificate,
   provisioning profile, and App Store Connect API key onto the same runner.
   GitHub Actions does not isolate processes between steps: a malicious PR can
   leave a background process during `cargo test` that reads the credentials
   once `setup-appstore-signing.sh` writes them.
2. **Direct Download** (`direct-download.yml`): the workflow checks out an
   arbitrary `v*` tag and then runs the composite action **from the tag
   checkout** (`uses: ./source/.github/actions/build-direct-download`), which
   in turn resolves `setup-developer-id-signing.sh` relative to itself — also
   from the tag. Anyone who can push a tag controls the exact shell that
   receives the Developer ID certificate and ASC API key. This contradicts the
   pattern `testflight.yml` already established (trusted `main` checkout for
   scripts, separate `source/` checkout for the code being built).

The credentials at stake sign every macOS release of this app. Exfiltration
means an attacker can ship signed malware as "Better Resource Monitor".

## Current state

- `.github/workflows/testflight.yml` — single job `upload-testflight` on
  `macos-15`, environment `testflight`. Step order (all one job):
  - `:68-79` two checkouts: trusted `main` at root, untrusted `source_ref` into `source/`
  - `:103-105` `pnpm install --frozen-lockfile` in `source/` (untrusted lifecycle scripts run here)
  - `:113-114` `cargo test --manifest-path source/src-tauri/Cargo.toml` (untrusted `build.rs` + tests)
  - `:143-148` `bash scripts/build-appstore-bundle.sh source ...` (trusted script, but it compiles untrusted Rust)
  - `:171-181` `bash scripts/setup-appstore-signing.sh` with all Apple secrets in env — **after** untrusted code ran
  - `:183-196` `bash scripts/upload-appstore-package.sh ...`
- `scripts/setup-appstore-signing.sh:42-59` — writes the ASC `.p8` to
  `$HOME/.appstoreconnect/private_keys/`, imports both `.p12`s into a temp
  keychain, deletes the `.p12` files after import (keychain remains).
- `.github/workflows/direct-download.yml`:
  - `:38-42` checks out `inputs.tag` into `source/` (no trusted-main checkout of `scripts/` is used for signing)
  - `:97` `uses: ./source/.github/actions/build-direct-download` ← composite action loaded **from the tag**
  - `:69-74` the "Validate scripts" step does `bash -n` on the **root** (main) copies of the scripts, but that validation is cosmetic — the action executes the tag's copy.
- `.github/actions/build-direct-download/action.yml`:
  - `:135` `SETUP_SCRIPT="$GITHUB_ACTION_PATH/../../../scripts/setup-developer-id-signing.sh"` — resolves inside `source/`, i.e. tag-controlled
  - `:141` `bash "$SETUP_SCRIPT"` with Developer ID P12 + password + ASC key in env
  - `:88` `pnpm tauri build ...` builds untrusted (tag) code in the same job that signs.
- Minor related gap: `testflight.yml:49-66` accepts free-form `source_ref` and
  writes it to `GITHUB_OUTPUT` without a delimiter block or allowlist, while
  `build_number` and `pr_number` are strictly validated.

Conventions to preserve: actions are SHA-pinned with version comments;
workflows use `set -euo pipefail` in run blocks; secrets flow through `env:`
not inline interpolation; the `testflight` environment gates the secrets.

## Commands you will need

| Purpose | Command | Expected on success |
|---|---|---|
| Workflow lint | `actionlint .github/workflows/testflight.yml .github/workflows/direct-download.yml` (skip if actionlint unavailable; do careful YAML review instead) | no errors |
| Shell syntax | `bash -n scripts/setup-appstore-signing.sh scripts/setup-developer-id-signing.sh` | exit 0 |
| YAML parse | `python3 -c "import yaml,sys; yaml.safe_load(open('.github/workflows/testflight.yml'))"` | exit 0 |

There is no way to fully test these workflows locally; correctness is
established by review plus a manual dispatch after merge (see Maintenance notes).

## Scope

**In scope**:
- `.github/workflows/testflight.yml`
- `.github/workflows/direct-download.yml`
- `.github/actions/build-direct-download/action.yml`
- `docs/github-release-workflows.md` (describe the new job split)
- `plans/README.md` (status row)

**Out of scope** (do NOT touch):
- `scripts/setup-appstore-signing.sh`, `scripts/setup-developer-id-signing.sh`,
  `scripts/upload-appstore-package.sh`, `scripts/build-appstore-bundle.sh` —
  their contents are fine; the problem is *where they run*, not what they do.
- `.github/workflows/ci.yml` and `release.yml` — plan 009 and plan 006 own those.
- Repository secret values or environment protection rules (GitHub settings, not code).

## Git workflow

- Branch: `fix/ci-signing-isolation`
- Conventional commits, e.g. `ci: split testflight build from signing` — matches
  existing history (`ci: isolate PR TestFlight signing`, `ci: harden testflight dispatch`).
- Do NOT push or open a PR unless the operator instructed it.

## Steps

### Step 1: Split `testflight.yml` into build and sign/upload jobs

Restructure the single `upload-testflight` job into two:

**Job 1 `build` (no Apple secrets, no `environment: testflight`):**
- Keep: require-main-dispatch check, resolve-source-ref, both checkouts, Rust
  toolchain + cache, pnpm/node setup, `pnpm install`, `cargo test`,
  build-metadata resolution, `bash scripts/build-appstore-bundle.sh ...`
  (script from the trusted `main` checkout at the workflow root, as today).
- New final step: upload the built `.app` bundle as a workflow artifact
  (`actions/upload-artifact`, SHA-pinned like the one already used in
  `direct-download.yml:109`). Find the bundle path produced by
  `build-appstore-bundle.sh` (read the script to confirm; it builds under
  `source/src-tauri/target/universal-apple-darwin/release/bundle/macos/`).
- Expose `build_number` and `sha` as job outputs.

**Job 2 `sign-upload` (`needs: build`, `environment: testflight`, holds all secrets):**
- Checkout `main` only (trusted scripts). Do NOT check out `source_ref`.
- Download the artifact into the path layout `upload-appstore-package.sh`
  expects (read the script first; if it expects the bundle inside
  `source/src-tauri/target/...`, recreate that directory shape).
- Run `setup-appstore-signing.sh`, then `upload-appstore-package.sh`, then the
  existing PR status/comment steps (pending status can stay in job 1 or move
  here; keep the failure handler in job 2 so a signing failure marks the PR).
- The commit-embedding check (the app must contain the expected commit prefix)
  should be re-verified in job 2 on the downloaded artifact with
  `strings -a <executable> | grep <sha-prefix>` so job 1 cannot substitute a
  different binary silently.

Key invariant: **no step in job 2 executes any file that came from the
`source_ref` checkout or the artifact** — the artifact is data to be signed,
never executed.

**Verify**: `actionlint .github/workflows/testflight.yml` → no errors; manual
review confirms job 2 has no `source/` checkout and job 1 has no
`secrets.APPLE_*` references.

### Step 2: Tighten `source_ref` validation while you're in the file

In the "Resolve source ref" step (`testflight.yml:49-66`):
- Allowlist the ref: accept only `main`, `v[0-9]*` tags, or
  `refs/pull/<digits>/head`. Reject anything else with a clear error.
- Write the output with a delimiter block:
  ```bash
  {
    echo "ref<<BRM_EOF"
    echo "$source_ref"
    echo "BRM_EOF"
  } >> "$GITHUB_OUTPUT"
  ```

**Verify**: YAML parses; the case/regex covers the three allowed forms
(`main`, `v1.2.3`, `refs/pull/123/head`) and rejects `refs/heads/evil` —
trace each by hand.

### Step 3: Run the direct-download composite action and signing script from `main`

In `.github/workflows/direct-download.yml`:
- The first checkout (`:36`, workflow tooling at the root) is already `main`
  by virtue of `workflow_dispatch` from main — make that explicit with
  `ref: main` and `persist-credentials: false` for clarity.
- Change `:97` from `uses: ./source/.github/actions/build-direct-download`
  to `uses: ./.github/actions/build-direct-download` (the trusted root copy).
- In `.github/actions/build-direct-download/action.yml:135`, the
  `$GITHUB_ACTION_PATH/../../../scripts/...` resolution now points at the
  trusted root checkout's `scripts/` automatically once the action is loaded
  from the root — verify the relative path still resolves (it becomes
  `<workspace>/.github/actions/build-direct-download/../../../scripts/` =
  `<workspace>/scripts/`, which is the main checkout). Keep the existence
  check and error message.
- Apply the same build/sign job-split principle as Step 1 if feasible within
  effort: minimum acceptable outcome for this plan is that **all executed
  workflow logic (composite action + signing script) comes from `main`**, with
  only the Rust/tauri build consuming tag-controlled sources. Note in a YAML
  comment that the build step still compiles tag-controlled code
  (`build.rs`), so pushing tags must remain maintainer-only.

**Verify**: `rg -n "source/.github" .github/workflows/direct-download.yml` →
no matches. `actionlint .github/workflows/direct-download.yml` → no errors.

### Step 4: Update the release docs

In `docs/github-release-workflows.md`, describe the new two-job TestFlight
shape and the trusted-action rule for Direct Download (one short paragraph
each). Do not rewrite unrelated sections.

**Verify**: `rg -n "two-job\|trusted" docs/github-release-workflows.md` → matches present.

## Test plan

No automated tests exist for workflows. Verification is:
- `actionlint` (or careful manual YAML review) on both workflows.
- A checklist in the PR description mapping each secret to the job that uses
  it, demonstrating job 1 has none.
- Post-merge: maintainer dispatches TestFlight from `main` against `main`
  (`gh workflow run testflight.yml --ref main`) and confirms a successful
  upload end-to-end before relying on the gate for PRs again.

## Done criteria

- [ ] `testflight.yml` has ≥2 jobs; the job with `environment: testflight` /
      `secrets.APPLE_*` contains no checkout of `source_ref` and no
      `pnpm install` / `cargo` / build of untrusted code
- [ ] The signing job re-verifies the embedded commit prefix in the downloaded artifact
- [ ] `source_ref` is allowlisted and written via a heredoc delimiter
- [ ] `direct-download.yml` uses `./.github/actions/build-direct-download` (root/trusted), not `./source/...`
- [ ] `bash -n` passes on both signing scripts (unchanged)
- [ ] No files outside the in-scope list are modified (`git status`)
- [ ] `plans/README.md` status row updated

## STOP conditions

Stop and report back (do not improvise) if:

- `scripts/build-appstore-bundle.sh` or `scripts/upload-appstore-package.sh`
  turn out to interleave building and signing in a way that cannot be split at
  the `.app`-artifact boundary (e.g. the upload script re-runs the build) —
  read both scripts fully before starting Step 1, and report if the boundary
  isn't clean.
- The `.app` bundle exceeds GitHub's artifact size limits or the
  upload/download round-trip breaks code signatures (symlinks/xattrs). If
  signature-relevant metadata is lost, `ditto -c -k --keepParent` the bundle
  into a zip before upload and unzip in job 2 — if that still fails, report.
- Plan 006 (Path A) has already modified `release.yml`/`direct-download.yml`
  in ways that conflict with Step 3.
- Any step would require adding a new repository secret.

## Maintenance notes

- Future workflow edits must preserve the invariant: *jobs holding Apple
  secrets never execute code from a non-`main` ref.* Reviewers should check
  every new `uses: ./source/...` or `working-directory: source` line in those
  jobs.
- The `testflight` environment's protection rules (GitHub settings) are a
  second layer worth configuring (required reviewers for the environment),
  but that's a dashboard change outside this repo.
- Deliberately deferred: hardening `pnpm install` in the build job with
  `--ignore-scripts` (the tauri CLI may need its postinstall; investigate
  separately), and pinning the Rust toolchain (see plans/README.md rejected
  findings).
- Local hygiene reminder for the maintainer (not part of this plan): the
  working tree contains `Certificates.p12` and `private_keys/*.p8` at mode
  644 — move them out of the repo and `chmod 600`.
