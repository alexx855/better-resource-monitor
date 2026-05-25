# GitHub Release Workflows

Better Resource Monitor uses GitHub Actions for both public verification and
App Store Connect uploads.

## Workflows

- `CI` validates pull requests and pushes with Rust checks and a macOS app
  bundle layout check.
- `TestFlight` uploads a build to App Store Connect without changing repository
  version files. It only sets a unique `CFBundleVersion` for the uploaded build.
- `Release` bumps the marketing version, commits and tags it, uploads the App
  Store package to App Store Connect, then creates the GitHub release.

Both upload workflows use `scripts/package-for-store.sh` as the packaging
entrypoint. Keep signing, entitlement, provisioning-profile, embedded commit,
and LaunchAgent checks there instead of duplicating packaging logic in YAML.

## Release Path FAQ

### Which workflow owns version bumping and tagging?

`.github/workflows/release.yml` owns public version bumps. Its `version-bump`
job updates `package.json`, `src-tauri/tauri.conf.json`, and
`src-tauri/Cargo.toml`, commits those changes to `main`, and pushes the
matching `v<version>` tag.

### Which workflow uploads to TestFlight/App Store Connect?

`.github/workflows/testflight.yml` uploads manual validation builds without
changing repository version files. `.github/workflows/release.yml` also uploads
the App Store package as part of a public release, after the version bump and tag
exist. Both paths call `scripts/setup-appstore-signing.sh` and
`scripts/package-for-store.sh` for signing, packaging, embedded-commit checks,
and App Store Connect upload behavior.

### Is there a local App Store packaging fallback?

No supported release fallback exists outside GitHub Actions. The scripts are kept
as workflow entrypoints and can be syntax-checked locally, but TestFlight and
release uploads should be dispatched through GitHub Actions so the build is tied
to a GitHub commit, runner log, and passing checks.

### Which files and workflows govern the release process?

Use `.github/workflows/release.yml` for version bumps, tags, and GitHub
Releases; `.github/workflows/testflight.yml` for manual App Store Connect
uploads; `scripts/setup-appstore-signing.sh` for certificate and provisioning
setup; `scripts/package-for-store.sh` for packaging and upload behavior; and
`src-tauri/tauri.conf.json`, `src-tauri/tauri.appstore.conf.json`,
`src-tauri/Entitlements.appstore.plist`, and
`src-tauri/embedded.provisionprofile` for App Store bundle inputs.

## Required GitHub Secrets

Add these repository secrets before running `TestFlight` or `Release`:

- `APPLE_TEAM_ID`
- `APPLE_DISTRIBUTION_IDENTITY`
- `APPLE_INSTALLER_IDENTITY`
- `APPLE_DISTRIBUTION_CERT_P12_BASE64`
- `APPLE_DISTRIBUTION_CERT_PASSWORD`
- `APPLE_INSTALLER_CERT_P12_BASE64`
- `APPLE_INSTALLER_CERT_PASSWORD`
- `APPLE_PROVISION_PROFILE_BASE64`
- `APPLE_API_KEY_ID`
- `APPLE_API_ISSUER`
- `APPSTORE_CONNECT_API_KEY_P8`

The certificate secrets are base64-encoded `.p12` files. The provisioning
profile secret is the base64-encoded Mac App Store provisioning profile for
`dev.alexpedersen.better-resource-monitor`. The App Store Connect key secret is
the raw private key content for `AuthKey_<APPLE_API_KEY_ID>.p8`.

## Transparency

The package script embeds the 12-character Git commit in the executable and
fails if the built app does not contain it. To verify a TestFlight/App Store
install, compare the app's embedded commit with the GitHub commit and its
passing GitHub Actions checks.

Run the local installed-app verifier after installing from TestFlight:

```bash
EXPECTED_VERSION=1.1.3 EXPECTED_BUILD_COMMIT=<short-commit> scripts/verify-macos-autostart.sh
```
