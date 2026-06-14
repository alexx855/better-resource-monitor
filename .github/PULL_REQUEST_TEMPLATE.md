## What changed

-

## Why

-

## Validation

- [ ] `cd src-tauri && cargo fmt --check`
- [ ] `cargo test --manifest-path src-tauri/Cargo.toml`
- [ ] Pre-merge TestFlight passed for this PR head: `gh workflow run testflight.yml --ref "$(git branch --show-current)" -f pr_number="$(gh pr view --json number -q .number)"`
- [ ] `pnpm build:www`

## Notes

-
