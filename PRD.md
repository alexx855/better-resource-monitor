# PRD: Comprehensive Test Coverage

## Goal
Achieve high confidence in user-facing behavior by adding granular, meaningful tests. Focus on small, isolated tasks.

## Process
1.  Pick the next unchecked task from the list below.
2.  Implement **ONE** test (or small refactor to enable testing).
3.  Run `cargo test`.
4.  Update `test-coverage-progress.txt`.
5.  Commit.
6.  Repeat.

---

## Tasks

### Phase 1: Core Utilities (Pure Functions)

- [x] **Test `cap_percent`**
    - Verify inputs: `0.0`, `50.0`, `99.0`, `100.0` (should cap at 99), `150.0`.
- [x] **Test `get_alert_color`**
    - Verify output for `is_dark = true` (matches `ALERT_COLOR_DARK`).
    - Verify output for `is_dark = false` (matches `ALERT_COLOR_LIGHT`).
- [x] **Test `get_text_color`**
    - Verify output for `is_dark = true` (255).
    - Verify output for `is_dark = false` (0).
- [x] **Test `format_speed` (Edge cases)**
    - Verify extremely small numbers (e.g. `1e-10`).
    - Verify large Terabyte values (function might not handle TB, verifying behavior is good).

### Phase 2: Linux Display Logic (Refactor & Test)

- [x] **Refactor `ensure_display_available` behavior**
    - Change it to return `Result<(), String>` instead of calling `std::process::exit(1)`.
    - Update `run()` to handle the `Err` by printing and exiting generally.
    - *Why:* Enables unit testing without killing the test runner.
- [x] **Test `ensure_display_available` (No Display)**
    - Use `serial_test`.
    - Unset `DISPLAY` and `WAYLAND_DISPLAY`.
    - Assert incorrectly returns Error.
- [x] **Test `ensure_display_available` (X11)**
    - Set `DISPLAY` to `:0`.
    - Assert returns Ok.
- [ ] **Test `ensure_display_available` (Wayland)**
    - Set `WAYLAND_DISPLAY` to `wayland-0` (unset DISPLAY).
    - Assert returns Ok.

### Phase 3: Linux Icon Detection (Mock/Env)

- [ ] **Test `detect_light_icons` (Env Fallback)**
    - Mock/Set `XDG_CURRENT_DESKTOP` to "XFCE".
    - Verify returns `false` (dark icons for light theme).
    - Mock/Set `XDG_CURRENT_DESKTOP` to "GNOME".
    - Verify returns `true` (default).

### Phase 4: Integration / Rendering

- [ ] **Test `render_svg_icon`**
    - Pass a simple SVG string.
    - Verify it returns a non-empty `Vec<u8>`.
    - Verify it doesn't panic on invalid SVG (or check how it fails).

---

## Notes
- Use `#[cfg(target_os = "linux")]` for Linux-specific tests.
- Use `serial_test::serial` for environment variable tests to avoid race conditions.
