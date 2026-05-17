# macOS Autostart Handoff

This branch fixes the Intel macOS Ventura case where Better Resource Monitor is
enabled in Login Items / Background Items but does not visibly start after login.
The reproduced failure was a login-started process stuck in `T` state before app
setup ran; reopening the app or sending `SIGCONT` resumed it and made the tray
appear.

## Build Commit

Run the signed Intel build from the merged fix commit. If the PR is
squash-merged or the workflow is dispatched from another ref, use that actual
signed-build commit. The app logs this short commit through `BRM_BUILD_COMMIT`,
so post-login verification should use the same short value:

```bash
EXPECTED_BUILD_COMMIT=<signed-build-short-commit>
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
EXPECTED_BUILD_COMMIT=<signed-build-short-commit> scripts/install-signed-macos-app.sh ~/Downloads/Better-Resource-Monitor-x86_64.app.zip
```

The installer checks the optional `.sha256` sidecar, bundle id, version,
`x86_64` architecture, bundled autostart LaunchAgent, code signature, and
Gatekeeper before replacing the app in `/Applications`.

## Verify After Login

Enable Start at Login in the app menu, then log out and back in. After the new
login session starts, run:

```bash
EXPECTED_VERSION=1.1.3 EXPECTED_BUILD_COMMIT=<signed-build-short-commit> scripts/verify-macos-autostart.sh
```

The verifier requires the installed app to be signed by team `G76YQZM2FU`, the
autostart LaunchAgent or Background Item to be registered, the process to be
running and not stopped/suspended, and the startup log to contain the expected
build commit.
