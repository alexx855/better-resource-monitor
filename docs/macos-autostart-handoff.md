# macOS Autostart Handoff

This branch fixes the Intel macOS Ventura case where Better Resource Monitor is
enabled in Login Items / Background Items but does not visibly start after login.

## Fixed Commit

Use a signed Intel build from:

```text
7d7a1015f4582728365a347c17436890688e5bc7
```

The app logs this short commit through `BRM_BUILD_COMMIT`, so post-login
verification should use:

```bash
EXPECTED_BUILD_COMMIT=7d7a101
```

## Signed Artifact Path

After this PR is merged, run the manual `Signed macOS Build` workflow on the
merged commit. It builds `--bundles app --target x86_64-apple-darwin`, verifies
the signed app with `codesign` and `spctl`, then uploads:

```text
Better-Resource-Monitor-x86_64.app.zip
Better-Resource-Monitor-x86_64.app.zip.sha256
```

## Install

Download both files into the same directory, then install the signed app:

```bash
EXPECTED_BUILD_COMMIT=7d7a101 scripts/install-signed-macos-app.sh ~/Downloads/Better-Resource-Monitor-x86_64.app.zip
```

The installer checks the optional `.sha256` sidecar, bundle id, version,
`x86_64` architecture, code signature, and Gatekeeper before replacing the app
in `/Applications`.

## Verify After Login

Enable Start at Login in the app menu, then log out and back in. After the new
login session starts, run:

```bash
EXPECTED_VERSION=1.1.3 EXPECTED_BUILD_COMMIT=7d7a101 scripts/verify-macos-autostart.sh
```

The verifier requires the installed app to be signed by team `G76YQZM2FU`, the
Background Item to be enabled, the process to be running, and the startup log to
contain the expected build commit.
