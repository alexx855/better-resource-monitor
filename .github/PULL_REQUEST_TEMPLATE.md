## What changed

-

## Why

-

## Validation

- [ ] `cd src-tauri && cargo fmt --check`
- [ ] `cargo test --manifest-path src-tauri/Cargo.toml`
- [ ] Pre-merge TestFlight passed for this PR head: `pr_number="$(gh pr view --json number -q .number)" && gh workflow run testflight.yml --ref main -f source_ref="refs/pull/${pr_number}/head" -f pr_number="$pr_number"`
- [ ] `pnpm build:www`

## Notes

-
