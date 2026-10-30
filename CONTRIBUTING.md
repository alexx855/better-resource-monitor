# Contributing to Better Resource Monitor

## Development

Install [Tauri v2 prerequisites](https://v2.tauri.app/start/prerequisites/)
and pnpm, then run from the repository root:

```sh
pnpm install
pnpm dev
pnpm build
```

Use `pnpm dev` for local testing. Installed macOS releases must be built through
the distribution workflows and run from `/Applications/Better Resource Monitor.app`.

See [www/README.md](www/README.md) for website development.

## Validation

For app changes, run from `src-tauri/`:

```sh
cargo fmt --check
cargo test
cargo clippy -- -D warnings
```

For website changes, run `pnpm check:www` and `pnpm build:www` from the root.
[CI](.github/workflows/ci.yml) defines the complete checks.

TestFlight uploads are optional and start only when a maintainer manually runs
[the workflow](.github/workflows/testflight.yml) from `main`. Set `source_ref`
to `main`, a `v*` release tag, or `refs/pull/<number>/head` to choose what to test:

```sh
gh workflow run testflight.yml --ref main -f source_ref=main
```

PRs and releases do not trigger TestFlight or require it before merging.

## Pull requests

Keep changes focused. Explain the problem, the change, and how you checked it.

- Regenerate affected screenshots with `pnpm build:screenshots` and include
  the resulting diff or before/after images. Generate tray art from the app
  renderer, as that command does.
- Update matching localized content when product claims change.
  Root `README*.md` files also supply the website home page body.
- App Store uploads go through the [TestFlight workflow](.github/workflows/testflight.yml).
  The [release workflow](.github/workflows/release.yml) handles releases.

## Questions and reports

Use [Discussions](https://github.com/alexx855/better-resource-monitor/discussions)
for questions and proposals, and
[Issues](https://github.com/alexx855/better-resource-monitor/issues)
for bugs and agreed work. Bug reports should include the macOS version, chip,
steps to reproduce, and expected versus actual behavior.

Report vulnerabilities as described in [SECURITY.md](SECURITY.md).
