# Contributing to Better Resource Monitor

Thanks for helping improve Better Resource Monitor. Keep contributions focused,
reviewable, and tied to the macOS menu bar product this repo currently ships.

## Getting started

1. Fork the repository and create a short-lived branch for the change.
2. Install dependencies with `pnpm install`.
3. Use `pnpm tauri dev` or `pnpm dev` for the root app.
4. Run Rust checks from `src-tauri/` when changing app behavior.
5. Run site commands through the workspace scripts when changing `www/`.

## Branch and PR scope

- Keep one concern per branch. Separate docs cleanup, workflow changes, UI copy,
  and runtime behavior unless they need to move together.
- Preserve unrelated dirty work. Use a temporary clone when testing or preparing
  a contribution from a machine that already has local changes.
- Write PR bodies that name the user-facing behavior, the source files that own
  it, and the verification that proves the change.

## Checks and validation

Docs-only changes still need stronger proof when they describe executable
behavior. If a Markdown change makes claims about Rust/Tauri code, CI gates,
release packaging, App Store signing, Start at Login, GPU sampling, or bundle
layout, wait for the same CI/Rust checks that protect those claims.

For narrow copy or prose-only edits, `git diff --check` plus a focused review is
usually enough. Broaden validation when the wording affects user promises or
release decisions.

## Release workflow docs

Treat executable files as the source of truth. When updating release docs, point
contributors to the workflow, script, config, entitlement, and provisioning files
that own the behavior instead of copying long YAML or shell snippets into prose.

Useful anchors include:

- `.github/workflows/release.yml` for version bumps, tags, and GitHub Releases.
- `.github/workflows/testflight.yml` for manual App Store Connect uploads.
- `scripts/package-for-store.sh` and `scripts/setup-appstore-signing.sh` for
  packaging and signing behavior on the current default branch.
- `src-tauri/tauri.appstore.conf.json`, `src-tauri/Entitlements.appstore.plist`,
  and `src-tauri/embedded.provisionprofile` for App Store packaging inputs.

## Localized copy and marketing surfaces

Update localized README files and website copy together when changing a
user-facing promise: supported platforms, install path, privacy/no-telemetry
claims, sandboxing/no-root behavior, release availability, pricing, or comparison
language.

Internal contributor wording can usually change in one docs file. If user-facing
copy must be deferred for localization, call that out in the PR body.

## Reporting issues

When reporting bugs, include:

- macOS version and chip, such as Intel or Apple Silicon.
- App version, install source, and whether the build came from TestFlight, the
  Mac App Store, a GitHub release, or source.
- Steps to reproduce.
- Expected and actual behavior.
- Screenshots or logs when they clarify the problem.
