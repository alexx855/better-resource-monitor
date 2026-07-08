# Plan 008: Stop tray-update failures from silently killing the monitoring thread

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving to the
> next step. If anything in the "STOP conditions" section occurs, stop and
> report — do not improvise. When done, update the status row for this plan
> in `plans/README.md` — unless a reviewer dispatched you and told you they
> maintain the index.
>
> **Drift check (run first)**:
> `git diff --stat 85c3e82..HEAD -- src-tauri/src/lib.rs src-tauri/src/tests.rs`
> If either file changed since this plan was written, compare the
> "Current state" excerpts against the live code before proceeding; on a
> mismatch, treat it as a STOP condition.

## Status

- **Priority**: P1
- **Effort**: S–M
- **Risk**: MED
- **Depends on**: none
- **Category**: bug
- **Planned at**: commit `85c3e82`, 2026-07-08

## Why this matters

The entire product is one background thread (`start_monitoring` in
`src-tauri/src/lib.rs`) that samples metrics and redraws the tray icon. On the
macOS update path, four calls use `.expect(...)`. A panic in a spawned thread
terminates only that thread: the app process stays alive, the tray icon stays
visible, but the numbers freeze forever with no error surfaced to the user or
to diagnostics. Additionally, `panic = "abort"` in the release profile
(`src-tauri/Cargo.toml:65`) means in release builds a panic here kills the
whole app instead — silently, from the user's perspective. Either failure mode
is the worst possible behavior for a monitoring tool: it looks like it's
working while reporting stale data, or vanishes.

The plausible triggers are transient AppKit conditions (status item briefly
unavailable during display sleep/wake, screen locking, or main-thread dispatch
contention) — exactly the conditions a long-running menu-bar app hits over
weeks of uptime. Recent commits (`fdf1aac` "avoid nested tray main-thread
dispatch", `cbc8ac3` "keep tray icon bright on inactive displays") show this
area already produces edge cases.

## Current state

- `src-tauri/src/lib.rs:897-1104` — `start_monitoring` spawns the loop with
  `thread::spawn`; no `catch_unwind`, no restart, no panic hook.
- The macOS tray-update path inside the loop:

```1080:1100:src-tauri/src/lib.rs
                if let Some(tray) = app.tray_by_id(TRAY_ID) {
                    #[cfg(target_os = "macos")]
                    {
                        let use_template = !_has_active_alert;
                        let icon = tray_icon::Icon::from_rgba(render_buffer.clone(), width, height)
                            .expect("Failed to create icon");
                        tray.with_inner_tray_icon(move |inner| {
                            inner
                                .set_icon_with_as_template(Some(icon), use_template)
                                .expect("failed to set macOS tray icon template image");
                            prevent_macos_tray_image_dimming(inner);
                        })
                        .expect("failed to update macOS tray icon on main thread");
                    }

                    #[cfg(not(target_os = "macos"))]
                    {
                        let icon = Image::new_owned(render_buffer.clone(), width, height);
                        let _ = tray.set_icon(Some(icon));
                    }
                }
```

  Note the asymmetry: the Linux path already ignores errors
  (`let _ = tray.set_icon(...)`); macOS panics instead.

- `prevent_macos_tray_image_dimming` (`src-tauri/src/lib.rs:332-349`) contains
  four more `.expect(...)` calls on AppKit lookups (`MainThreadMarker::new`,
  `ns_status_item`, `button(mtm)`, `cell()`, `downcast_ref`). It runs inside
  the `with_inner_tray_icon` closure, i.e. on the main thread — a panic there
  propagates through the main-thread dispatch.
- Diagnostics convention: this file already has a macOS diagnostic logger,
  `macos_diag_log(...)` (used at e.g. `lib.rs:367`, `lib.rs:402`, `lib.rs:413`)
  — use it for failure logging on macOS; `eprintln!` is the cross-platform
  fallback used elsewhere (e.g. `lib.rs:401`, `lib.rs:426`).
- Also relevant (same failure class, smaller blast radius):
  `src-tauri/src/tray_render.rs:124` `.expect("Failed to parse SVG")` and
  `:132` `.expect("Failed to create pixmap")` run once per `IconCache::new`
  (constructed at loop start, `lib.rs:930`) — on bundled, compile-time-known
  assets. These are acceptable as `expect` (startup, deterministic input);
  leave them.
- Tests live in `src-tauri/src/tests.rs`, table-driven style, run with
  `cargo test` from `src-tauri/`.

Repo conventions: `cargo fmt`/`clippy -D warnings` must stay clean (both pass
at the planned-at commit). Comments explain intent, not mechanics.

## Commands you will need

| Purpose | Command (repo root) | Expected on success |
|---|---|---|
| Format | `cargo fmt --manifest-path src-tauri/Cargo.toml --check` | exit 0 |
| Tests | `cargo test --manifest-path src-tauri/Cargo.toml` | exit 0 |
| Lint | `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` | exit 0 |
| Smoke run | `pnpm tauri dev` (manual; tray appears, values update) | visual check |

## Scope

**In scope**:
- `src-tauri/src/lib.rs` — the tray update block (~1080–1100),
  `prevent_macos_tray_image_dimming` (~332–349), and a small extracted helper
- `src-tauri/src/tests.rs` — tests for the extracted helper
- `plans/README.md` (status row)

**Out of scope** (do NOT touch):
- `src-tauri/src/tray_render.rs` — startup `expect`s on bundled assets are fine.
- The hysteresis/sampling logic in the loop (lines 934–1078) — behavior must not change.
- `Cargo.toml` `panic = "abort"` — changing the panic strategy is a separate decision.
- Thread-restart machinery — do not add a supervisor/respawn loop; converting
  panics to logged skips is sufficient and simpler (next tick retries anyway).

## Git workflow

- Branch: `fix/tray-update-panic-hardening`
- Conventional commits, e.g. `fix: log tray icon update failures instead of panicking`
- Do NOT push or open a PR unless the operator instructed it.

## Steps

### Step 1: Convert the macOS tray-update panics to logged skips

Rework the macOS block (lib.rs ~1081–1093) so every fallible call logs and
continues instead of panicking. Target shape:

```rust
#[cfg(target_os = "macos")]
{
    let use_template = !_has_active_alert;
    match tray_icon::Icon::from_rgba(render_buffer.clone(), width, height) {
        Ok(icon) => {
            let result = tray.with_inner_tray_icon(move |inner| {
                if let Err(e) = inner.set_icon_with_as_template(Some(icon), use_template) {
                    eprintln!("Failed to set macOS tray icon: {e}");
                }
                prevent_macos_tray_image_dimming(inner);
            });
            if let Err(e) = result {
                macos_diag_log(format!("tray_update main-thread dispatch failed: {e}"));
            }
        }
        Err(e) => {
            macos_diag_log(format!("tray_update icon creation failed: {e}"));
        }
    }
}
```

Adjust to the actual API types (check what `with_inner_tray_icon` returns —
if its closure return type or error type differs, keep the intent: no
`expect`/`unwrap` on this path). The next loop iteration will retry because
`prev_*` values were already updated; that means one skipped frame on
transient failure, which is acceptable — do not restructure the
`prev_*` bookkeeping.

**Verify**: `rg -n "expect\(" src-tauri/src/lib.rs` → no matches between
lines ~1080–1100 (the monitor-loop tray block).

### Step 2: Make `prevent_macos_tray_image_dimming` non-panicking

Change it to return early (with one `macos_diag_log` at most per failure kind)
instead of `expect`-ing, e.g.:

```rust
#[cfg(target_os = "macos")]
fn prevent_macos_tray_image_dimming(tray: &tray_icon::TrayIcon) {
    let Some(mtm) = MainThreadMarker::new() else { return };
    let Ok(ns_status_item) = tray.ns_status_item() else { return };
    // ... same pattern for button, cell, downcast_ref ...
    button_cell.setImageDimsWhenDisabled(false);
}
```

Match the real return types (`Option` vs `Result`) of each call; the current
code at lib.rs:332-349 shows which is which (`.expect` on both). Silent return
is acceptable here — dimming is cosmetic; do not log on every tick (this runs
per update). If you want visibility, log once via a `std::sync::Once`.

**Verify**: `rg -n "expect\(" src-tauri/src/lib.rs | rg -v "tests"` → remaining
matches are only outside the monitoring/tray-update paths (e.g. none in
`prevent_macos_tray_image_dimming`).

### Step 3: Add a defensive log for oversized/empty render output

`Icon::from_rgba` fails when `render_buffer.len() != 4 * width * height`.
Before the icon creation, add a debug assertion of that invariant as a plain
runtime check that logs-and-skips (not `debug_assert!`, which vanishes in
release):

```rust
if render_buffer.len() != (4 * width * height) as usize {
    macos_diag_log(format!(
        "tray_update buffer size mismatch len={} expected={}",
        render_buffer.len(),
        4 * width * height
    ));
    continue;
}
```

Place it after `render_tray_icon_into` returns and before the
`if let Some(tray)` block (it protects both platform paths); note the Linux
path's `Image::new_owned` has the same size expectation.

**Verify**: `cargo test --manifest-path src-tauri/Cargo.toml` → all pass.

### Step 4: Run the full check suite

**Verify**:
- `cargo fmt --manifest-path src-tauri/Cargo.toml --check` → exit 0
- `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` → exit 0
- `cargo test --manifest-path src-tauri/Cargo.toml` → exit 0

## Test plan

The tray/AppKit calls themselves cannot be unit-tested without a runtime, but
the buffer-size invariant can:

- In `src-tauri/src/tests.rs`, add a test that renders a known config via
  `TrayRenderer::render_tray_icon_into` (existing tests at
  `tests.rs:452-532` show how to construct `RenderConfig`; model after
  `assert_render_size`) and asserts `buffer.len() == 4 * width * height` —
  this pins the invariant Step 3 relies on.
- Manual verification (documented in the PR): run `pnpm tauri dev`, confirm
  tray values update; lock the screen / sleep the display and confirm updates
  resume afterwards.

## Done criteria

- [ ] No `.expect(` or `.unwrap(` remains in the monitor loop's tray-update
      block or in `prevent_macos_tray_image_dimming`
- [ ] Buffer-size mismatch is checked at runtime and skips the frame with a diagnostic log
- [ ] `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` all exit 0
- [ ] New invariant test exists in `src-tauri/src/tests.rs` and passes
- [ ] No files outside the in-scope list are modified (`git status`)
- [ ] `plans/README.md` status row updated

## STOP conditions

Stop and report back (do not improvise) if:

- `with_inner_tray_icon`'s signature doesn't allow returning/observing an
  error from the closure in a way that compiles cleanly — report the actual
  signature rather than swallowing errors with a bare `let _ =`.
- The code at lib.rs ~1080–1100 no longer matches the excerpt (drift).
- Fixing this appears to require changes to `tray_render.rs` beyond reading it.
- Clippy flags the new code and the fix would alter loop behavior.

## Maintenance notes

- This plan makes failures visible in the macOS diagnostic log
  (`macos_diag_log` writes under the app container — see `diag_log_path()` in
  lib.rs ~460). If freeze reports persist after this lands, that log is now
  the first place to look.
- The pre-merge TestFlight gate applies (app behavior change): dispatch
  TestFlight per AGENTS.md before merging.
- Deferred deliberately: thread respawn/supervision, and revisiting
  `panic = "abort"` — reconsider only if evidence shows panics from other
  code paths (sampling, sysinfo) actually occur.
