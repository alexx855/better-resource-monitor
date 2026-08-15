# Plan 013: Refresh Website Advisory Floors

## Status

- **State**: IN PROGRESS
- **Scope**: Raise only the three transitive JavaScript packages currently
  blocking the required CI audit: `undici`, `js-yaml`, and `nanoid`.
- **Out of scope**: Rust code, Tauri behavior, release workflows, framework
  migrations, package-manager changes, and audit suppression.

## Why this is needed

The required `Rust checks` workflow currently fails at `pnpm audit` because the
website dependency graph resolves vulnerable versions of these packages:

- `undici` `<7.29.0`
- `js-yaml` `<4.3.1`
- `nanoid` `<3.3.18`

The direct Astro and Cloudflare packages already satisfy their current advisory
floors, so a narrow root `pnpm.overrides` update is safer than a framework
migration.

## Done criteria

- [ ] Add exact patched floors to the existing root `pnpm.overrides` block.
- [ ] Regenerate `pnpm-lock.yaml` with the frozen workspace manifest.
- [ ] `pnpm audit --prod --audit-level high` exits successfully with no high or
      moderate advisories.
- [ ] `pnpm --filter www check` and `pnpm build:www` pass.
- [ ] No files outside `package.json`, `pnpm-lock.yaml`, and this plan are
      changed for the remediation.
