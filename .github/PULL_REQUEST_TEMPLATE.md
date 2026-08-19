## What changed

-

## Why

-

## Validation

- [ ] `cd src-tauri && cargo fmt --check`
- [ ] `cargo test --manifest-path src-tauri/Cargo.toml`
- [ ] `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings`
- [ ] Pre-merge TestFlight passed for this PR head when native/app-impacting files changed: `pr_number="$(gh pr view --json number -q .number)" && gh workflow run testflight.yml --ref main -f source_ref="refs/pull/${pr_number}/head" -f pr_number="$pr_number"` (website, documentation, and CI-only PRs receive an automatic skipped success)
- [ ] `pnpm build:www`

## Notes

-
