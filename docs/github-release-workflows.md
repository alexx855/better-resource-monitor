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

## Local Fallback

Manual packaging remains available for emergency use:

```bash
BUILD_NUMBER=$(date -u +%Y%m%d%H%M) scripts/package-for-store.sh
```

Local runs require `scripts/.env`, locally installed Apple signing identities,
and `src-tauri/embedded.provisionprofile`.
