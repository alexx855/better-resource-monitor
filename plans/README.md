# Plans

This directory now contains only active work. Historical implementation plans
and superseded alternatives are preserved in [`archive/`](archive/).

## Why there were so many plans

Two audits were combined here without separating active work from history:

- Plans 001–005 came from the 2026-06-11 audit.
- Plans 006–011 came from the deeper 2026-07-08 audit and included successors
  for three earlier plans.

The implementation work was later stacked and merged into `main` by commit
`18c36cc` (2026-07-17), but the individual specifications stayed in this
directory. That made completed work look active.

## Active plans and follow-ups

| Item | Status | Current meaning |
| --- | --- | --- |
| 012 — App Store-safe Thermal Status | IMPLEMENTED — menu-only qualitative status | The numeric Celsius requirement remains blocked by the App Sandbox/public-API boundary. The replacement uses public qualitative APIs and exposes a dimmed localized status row in the macOS tray menu (`src-tauri/src/thermal.rs`). |
| Plan 003 follow-up | TODO — refresh dependency remediation | The original advisory cleanup landed, but the current audit reports 8 transitive advisories: 3 high and 5 moderate. Create a new narrowly scoped remediation plan before changing dependencies. |

## Implemented and archived

These plans' source changes are present in the current `main` history:

| Plan | Result | Evidence / remaining proof |
| --- | --- | --- |
| 005 | Implemented vendored generated-image fonts | Local website build passes; fonts are under `www/src/assets/fonts`. |
| 006 | Implemented Path A: signed direct-download DMGs | Merged in PR #142 / commit `18c36cc`; a real end-to-end DMG install smoke test remains useful. |
| 007 | Implemented signing isolation | Trusted workflow split is in `main`; post-merge TestFlight dispatch proof remains an operational gate. |
| 008 | Implemented tray-update error handling | Current Rust tests pass; a real macOS sleep/wake smoke test remains useful. |
| 009 | Implemented documented CI checks | CI contains locale parity, Astro check, site build, and clippy; local Astro check and site build pass. |
| 010 | Implemented locale parity enforcement | `node scripts/check-locale-parity.mjs` passes for all four locales. |
| 011 | Implemented ClientRouter removal | The strict `connect-src 'none'` CSP remains; the local site build passes without ClientRouter. |

## Superseded and archived

- Plan 001 is superseded by Plan 006.
- Plan 002 is superseded by Plan 009.
- Plan 004 is superseded by Plan 011.

## Current verification snapshot

Verified 2026-08-14 in the current worktree:

- `cargo fmt --manifest-path src-tauri/Cargo.toml --check` — passed.
- `cargo test --manifest-path src-tauri/Cargo.toml` — passed, 37 tests (38 with the `app-store` feature).
- `node scripts/check-locale-parity.mjs` — passed.
- `pnpm --filter www check` — passed, 0 errors/warnings/hints.
- `pnpm build:www` — passed, 29 pages.
- `pnpm audit --prod --audit-level high` — currently fails with 8 transitive advisories; this is the Plan 003 follow-up above.

Status vocabulary: `TODO` means active work is not implemented; `IMPLEMENTED`
means the code is present in `main`; `SUPERSEDED` means do not execute the old
plan; `BLOCKED` is reserved for a hard technical or product boundary.
