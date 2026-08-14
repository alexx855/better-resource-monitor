# Plans

This directory contains only active work. Completed and superseded plan
specifications were removed after their outcomes were recorded here.

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
| [012 — App Store-safe Thermal Status](012-add-app-store-safe-temperature-metric.md) | TODO — qualitative implementation | The numeric Celsius requirement is blocked by the App Sandbox/public-API boundary. A qualitative `ProcessInfo.thermalState` feature is feasible, but implementation has not started. |
| Plan 003 follow-up | TODO — refresh dependency remediation | The original advisory cleanup landed, but the current audit reports 8 transitive advisories: 3 high and 5 moderate. Create a new narrowly scoped remediation plan before changing dependencies. |

## Removed plan history

The old specifications are intentionally gone. Their outcomes were:

| Plan | Outcome |
| --- | --- |
| 001 | Superseded by 006; removed. |
| 002 | Superseded by 009; removed. |
| 003 | Original advisory cleanup landed; the current dependency audit follow-up remains open; removed. |
| 004 | Superseded by 011; removed. |
| 005–011 | Implemented in `main` by the later audit stack; removed. |

Plan 006's implementation was merged in PR #142 / commit `18c36cc`. The
remaining evidence gaps are operational smoke tests, not active plan files.

## Current verification snapshot

Verified 2026-08-14 in the current worktree:

- `cargo fmt --manifest-path src-tauri/Cargo.toml --check` — passed.
- `cargo test --manifest-path src-tauri/Cargo.toml` — passed, 28 tests.
- `node scripts/check-locale-parity.mjs` — passed.
- `pnpm --filter www check` — passed, 0 errors/warnings/hints.
- `pnpm build:www` — passed, 29 pages.
- `pnpm audit --prod --audit-level high` — currently fails with 8 transitive advisories; this is the Plan 003 follow-up above.

Status vocabulary: `TODO` means active work is not implemented; `IMPLEMENTED`
means the code is present in `main`; `SUPERSEDED` means do not execute the old
plan; `BLOCKED` is reserved for a hard technical or product boundary.
