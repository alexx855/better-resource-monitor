# Plan 013: Refresh Website Advisory Floors

## Status

- **State**: IMPLEMENTED
- **Scope**: Raise only the four transitive JavaScript packages currently
  below their patched advisory floors: `undici`, `js-yaml`, `nanoid`, and
  `postcss`.
- **Out of scope**: Rust code, Tauri behavior, release workflows, framework
  migrations, package-manager changes, and audit suppression.

## Why this is needed

The required `Rust checks` workflow currently fails at `pnpm audit` because the
website dependency graph resolves vulnerable versions of these packages:

- `undici` `<7.29.0`
- `js-yaml` `<4.3.1`
- `nanoid` `<3.3.18`
- `postcss` `<8.5.23`

The direct Astro and Cloudflare packages already satisfy their current advisory
floors, so a narrow root `pnpm.overrides` update is safer than a framework
migration.

## Done criteria

- [x] Add exact patched floors to the existing root `pnpm.overrides` block.
- [x] Regenerate `pnpm-lock.yaml` with the frozen workspace manifest.
- [x] `pnpm audit --prod --audit-level high` exits successfully with no high or
      moderate advisories.
- [x] `pnpm --filter www check` and `pnpm build:www` pass.
- [x] No Rust, Tauri, or release workflow files changed for the remediation;
      package-manager tooling remains pnpm.

## Verification

Verified 2026-08-15 in the feature worktree:

- `pnpm audit --prod --audit-level low` — passed, no known vulnerabilities.
- `pnpm --filter www check` — passed, 0 errors/warnings/hints.
- `pnpm build:www` — passed, 29 pages.
