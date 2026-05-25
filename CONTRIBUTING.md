# Contributing to Better Resource Monitor

Thanks for helping improve Better Resource Monitor. This guide is for human
contributors; agent-specific repo notes live in [AGENTS.md](AGENTS.md), and
release mechanics are owned by the workflow files, scripts, and Tauri config.

## Getting Started

1. Fork the repository and clone your fork.
2. Install dependencies with `pnpm install`.
3. Run the app with `pnpm tauri dev`.
4. For the website, use `pnpm dev:www`, `pnpm build:www`,
   `pnpm preview:www`, and `pnpm build:screenshots` from the repo root.

Rust checks run from `src-tauri/` unless a workflow says otherwise:

```bash
cd src-tauri
cargo fmt
cargo test
cargo clippy
cargo llvm-cov --lib --html --output-dir coverage/
```

## Development Workflow

Keep branches focused and short-lived. Use an intent prefix such as `docs/`,
`fix/`, `feat/`, or `chore/` so reviewers can scan the branch purpose. If you
are working from a dirty local checkout, create a temporary clone or worktree for
the pull request so unrelated changes do not leak into the diff.

Pull requests should explain:

- what changed and why;
- which user, contributor, or release path is affected;
- which checks were run, or why a narrower check is enough;
- follow-up work that is intentionally left out of scope.

## Validation

Docs-only changes still need enough proof for the claims they make. A wording
fix can usually be reviewed with a focused diff check. A docs change that
describes Rust/Tauri behavior, release packaging, CI gates, App Store signing,
Start at Login, GPU sampling, or bundle layout should wait for the relevant CI
or Rust proof before merge.

For website changes, run `pnpm build:www`. For Rust or app behavior changes,
run the focused Rust command first, then broaden to the full checks when the
change touches shared behavior or release packaging.

## Release Workflow Changes

Treat executable files as the source of truth. Release and TestFlight behavior
is defined by `.github/workflows/release.yml`,
`.github/workflows/testflight.yml`, `scripts/package-for-store.sh`,
`scripts/setup-appstore-signing.sh`, `src-tauri/tauri.appstore.conf.json`,
`src-tauri/Entitlements.appstore.plist`, and
`src-tauri/embedded.provisionprofile`.

Release documentation should explain how those files fit together without
copying long YAML or shell fragments into prose. If a branch temporarily changes
the intended workflow, describe that in the pull request body or a temporary
handoff note, then fold only durable guidance into mainline docs.

## Localization And Marketing Copy

When a change affects a user-facing promise, update the related localized and
marketing surfaces together. This includes supported platforms, install paths,
release availability, sandboxing/no-root claims, privacy/no-telemetry claims,
pricing, and comparison language.

Internal contributor wording can usually change in one file. Product positioning
should move across the matching root `README*.md` files, which feed the website
home body, `www/src/lib/marketing-copy.json`, and `src-tauri/src/i18n.rs` in
the same pull request, or call out the deferred localization follow-up
explicitly.

## Discussions vs Issues

Use GitHub Discussions for questions, design tradeoffs, validation policy,
contributor workflow clarification, and support-style conversations where the
answer may shape later work. Start uncertain process questions there, then open
an issue once the desired change is concrete.

Use GitHub Issues for reproducible bugs, accepted implementation tasks, or
tracked docs/product changes with clear acceptance criteria. If a discussion
reveals a real defect or durable docs gap, link the follow-up issue back to the
discussion so the decision trail is easy to find.

## Reporting Issues

When reporting bugs, please include:

- macOS version and chip (Intel/Apple Silicon)
- steps to reproduce
- expected vs actual behavior
- screenshots if applicable

## Questions?

If you are not sure where something belongs, start with GitHub Discussions.
