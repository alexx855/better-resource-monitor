# GitHub Release Workflows

Better Resource Monitor uses GitHub Actions for both public verification and
App Store Connect uploads. GitHub direct-download artifacts use a separate
Developer ID workflow so App Store packaging and outside-the-store packaging do
not share signing contracts.

## Workflows

- `CI` validates pull requests and pushes with Rust checks and a macOS app
  bundle layout check.
- `TestFlight` is manually dispatched and uploads a build to App Store Connect
  without changing repository version files. It only sets a unique
  `CFBundleVersion` for the uploaded build.
- `Release` is manually dispatched with a version bump type. It opens and
  merges a version-bump pull request, tags the merge commit on `main`, dispatches
  `TestFlight` for that tag, waits for the App Store Connect upload to pass,
  then creates the GitHub release.
- `Direct Download` is manually dispatched for an existing tag. It builds a
  universal macOS app, signs it with Developer ID, notarizes it, packages both a
  `.dmg` and `.zip`, and can upload those artifacts to the matching GitHub
  release.

The `TestFlight` workflow uses `.github/actions/upload-appstore` as the shared
packaging entrypoint, and `Release` dispatches `TestFlight` instead of
duplicating upload logic. Keep signing, entitlement, provisioning-profile,
embedded commit, private-GPU-API, and LaunchAgent checks in that action.
The current macOS autostart contract is documented in
[`macos-autostart.md`](macos-autostart.md).

## Action Runtime

The workflows pin JavaScript actions to Node 24-compatible major versions:
`actions/checkout@v5`, `actions/setup-node@v5`, and `pnpm/action-setup@v5`.
They also set `FORCE_JAVASCRIPT_ACTIONS_TO_NODE24=true` at workflow scope so
any remaining JavaScript action runs on Node 24 while GitHub phases out older
runtimes.

Do not replace `.github/actions/upload-appstore` or
`scripts/setup-appstore-signing.sh` with `tauri-apps/tauri-action` just to
silence Node runtime warnings. The Tauri action is useful for Tauri builds and
GitHub Release artifacts, but this repo's App Store path also needs
provisioning-profile installation, installer certificate import, entitlement
checks, `CFBundleVersion` assignment, embedded commit verification, private API
scanning, `productbuild`, and `xcrun altool` upload behavior.

The direct-download workflow uses `.github/actions/build-direct-download` and
`scripts/setup-developer-id-signing.sh`. Keep it separate from the App Store
action because direct downloads need Developer ID Application signing,
notarization, stapling, and user-installable artifacts rather than App Store
provisioning profiles or App Store installer packages.

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

Add these additional repository secrets before running `Direct Download`:

- `DEVELOPER_ID_APPLICATION_IDENTITY`
- `DEVELOPER_ID_APPLICATION_CERT_P12_BASE64`
- `DEVELOPER_ID_APPLICATION_CERT_PASSWORD`

`DEVELOPER_ID_APPLICATION_IDENTITY` should be the full codesigning identity,
for example `Developer ID Application: Alex Pedersen (TEAMID)`. The certificate
secret is the base64-encoded Developer ID Application `.p12` file. Direct
download notarization reuses `APPLE_API_KEY_ID`, `APPLE_API_ISSUER`, and
`APPSTORE_CONNECT_API_KEY_P8`.

## Transparency

The upload action embeds the 12-character Git commit in the executable and
fails if the built app does not contain it. To verify a TestFlight/App Store
install, compare the app's embedded commit with the GitHub commit and its
passing GitHub Actions checks.

All App Store uploads go through `TestFlight`. That workflow uses the GitHub run
number as the baseline `CFBundleVersion`, with a UTC timestamp floor so manual
validation builds remain higher than earlier timestamp-scale uploads for the
same marketing version. Apple requires build numbers to increase within the same
marketing version, but a new marketing version can start again with a lower
build number.

## Dispatch Commands

Upload a TestFlight/App Store Connect build without changing versions:

```bash
gh workflow run testflight.yml --ref main
```

Create a release, upload the App Store package, and publish the GitHub release:

```bash
gh workflow run release.yml --ref main -f version_type=patch
```

Build signed and notarized direct-download artifacts for an existing tag and
upload them to the matching GitHub Release:

```bash
gh workflow run direct-download.yml --ref main -f tag=v1.1.4 -f upload_to_release=true
```

The upload paths are GitHub Actions only. There is no local App Store packaging
fallback; use TestFlight dispatch for validation builds and Direct Download
dispatch for public `.dmg` and `.zip` artifacts.
