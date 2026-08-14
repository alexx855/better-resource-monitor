mod gpu;
pub mod i18n;
#[cfg(target_os = "macos")]
mod macos_autostart;
mod storage;
#[cfg(target_os = "macos")]
pub mod thermal;
pub mod tray_render;

use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use font_kit::family_name::FamilyName;
use font_kit::handle::Handle;
use font_kit::properties::{Properties, Weight};
use font_kit::source::SystemSource;
use rusttype::Font;
use serde_json::json;
use sysinfo::{Networks, System};
use tauri::{
    image::Image,
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    AppHandle,
};
use tauri_plugin_store::StoreExt;

#[cfg(target_os = "macos")]
use tauri::ActivationPolicy;

#[cfg(target_os = "macos")]
use objc2::MainThreadMarker;
#[cfg(target_os = "macos")]
use objc2_app_kit::NSButtonCell;
#[cfg(target_os = "macos")]
use objc2_foundation::NSBundle;
#[cfg(target_os = "macos")]
use std::io::Write;
#[cfg(target_os = "macos")]
use std::os::fd::AsRawFd;

#[cfg(target_os = "linux")]
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};

use gpu::GpuSampler;

#[cfg(target_os = "linux")]
static LIGHT_ICONS: AtomicBool = AtomicBool::new(true);

#[cfg(target_os = "macos")]
const APP_SIZING: tray_render::Sizing = tray_render::SIZING_MACOS;

#[cfg(not(target_os = "macos"))]
const APP_SIZING: tray_render::Sizing = tray_render::SIZING_LINUX;

#[cfg(target_os = "linux")]
const THEME_POLL_INTERVAL_SECS: u64 = 5;

#[cfg(target_os = "linux")]
fn detect_light_icons() -> bool {
    LIGHT_ICONS.load(Relaxed)
}

#[cfg(target_os = "linux")]
fn start_theme_detection_thread() {
    // Initialize with actual value before spawning polling thread to avoid race condition
    LIGHT_ICONS.store(detect_light_icons_impl(), Relaxed);

    thread::spawn(|| loop {
        thread::sleep(Duration::from_secs(THEME_POLL_INTERVAL_SECS));
        let detected = detect_light_icons_impl();
        LIGHT_ICONS.store(detected, Relaxed);
    });
}

#[cfg(target_os = "linux")]
fn ensure_display_available() -> Result<(), String> {
    let has_x11 = std::env::var("DISPLAY").is_ok();
    let has_wayland = std::env::var("WAYLAND_DISPLAY").is_ok();

    if has_x11 || has_wayland {
        Ok(())
    } else {
        Err("No display server found. Please set DISPLAY or WAYLAND_DISPLAY.".to_string())
    }
}

#[cfg(target_os = "linux")]
fn detect_light_icons_impl() -> bool {
    // Try gsettings (GNOME/GTK)
    if let Ok(output) = std::process::Command::new("gsettings")
        .args([
            "get",
            "org.gnome.desktop.interface",
            "gtk-application-prefer-dark-theme",
        ])
        .output()
    {
        let result = String::from_utf8_lossy(&output.stdout);
        if result.contains("true") {
            return true; // Dark theme → light (white) icons
        }
    }

    // Check XDG_CURRENT_DESKTOP for common light-themed DEs
    if let Ok(desktop) = std::env::var("XDG_CURRENT_DESKTOP") {
        let lower = desktop.to_lowercase();
        if lower.contains("xfce") || lower.contains("elementary") || lower.contains("kde") {
            return false; // Often light themes → dark (black) icons
        }
    }

    // Default: most Linux panels are dark → use light (white) icons
    true
}

const SETTINGS_FILE: &str = "settings.json";

#[cfg(target_os = "macos")]
const DEFAULT_SHOW_THERMAL_STATUS: bool = false;

mod menu_id {
    pub const AUTOSTART: &str = "autostart";
    pub const SHOW_CPU: &str = "show_cpu";
    pub const SHOW_MEM: &str = "show_mem";
    pub const SHOW_STORAGE: &str = "show_storage";
    pub const SHOW_GPU: &str = "show_gpu";
    pub const SHOW_NET: &str = "show_net";
    #[cfg(target_os = "macos")]
    pub const SHOW_THERMAL: &str = "show_thermal_status";
    #[cfg(target_os = "macos")]
    pub const THERMAL_STATUS: &str = "thermal_status";
    pub const SHOW_ALERTS: &str = "show_alerts";
    pub const QUIT: &str = "quit";
}

const TRAY_ID: &str = "main";

#[derive(Clone)]
struct MetricToggles {
    show_cpu: Arc<AtomicBool>,
    show_mem: Arc<AtomicBool>,
    show_storage: Arc<AtomicBool>,
    show_gpu: Arc<AtomicBool>,
    show_net: Arc<AtomicBool>,
    show_alerts: Arc<AtomicBool>,
    #[cfg(target_os = "macos")]
    show_thermal: Arc<AtomicBool>,
}

struct LoadedSettings {
    show_cpu: bool,
    show_mem: bool,
    show_storage: bool,
    show_gpu: bool,
    show_net: bool,
    show_alerts: bool,
    autostart: bool,
    #[cfg(target_os = "macos")]
    show_thermal: bool,
}

fn load_settings(app: &AppHandle) -> LoadedSettings {
    let store = match app.store(SETTINGS_FILE) {
        Ok(s) => Some(s),
        Err(e) => {
            eprintln!("Failed to load settings store: {e}");
            None
        }
    };

    let get_bool_option = |key: &str| -> Option<bool> {
        store
            .as_ref()
            .and_then(|s| s.get(key))
            .and_then(|v| v.as_bool())
    };
    let get_bool = |key: &str, default: bool| -> bool { get_bool_option(key).unwrap_or(default) };

    let has_legacy_metric_settings = [
        menu_id::SHOW_CPU,
        menu_id::SHOW_MEM,
        menu_id::SHOW_GPU,
        menu_id::SHOW_NET,
    ]
    .iter()
    .any(|key| get_bool_option(key).is_some());
    let show_storage = migrate_storage_setting(
        get_bool_option(menu_id::SHOW_STORAGE),
        has_legacy_metric_settings,
    );

    LoadedSettings {
        show_cpu: get_bool("show_cpu", true),
        show_mem: get_bool("show_mem", true),
        show_storage,
        show_gpu: get_bool("show_gpu", true),
        show_net: get_bool("show_net", true),
        show_alerts: get_bool("show_alerts", true),
        autostart: get_bool(menu_id::AUTOSTART, false),
        #[cfg(target_os = "macos")]
        show_thermal: get_bool(menu_id::SHOW_THERMAL, DEFAULT_SHOW_THERMAL_STATUS),
    }
}

fn migrate_storage_setting(stored_storage: Option<bool>, has_legacy_metric_settings: bool) -> bool {
    stored_storage.unwrap_or(!has_legacy_metric_settings)
}

fn save_setting(app: &AppHandle, key: &str, value: bool) {
    if let Ok(store) = app.store(SETTINGS_FILE) {
        store.set(key, json!(value));
        if let Err(e) = store.save() {
            eprintln!("Failed to save setting {key}: {e}");
        }
    }
}

const UPDATE_INTERVAL_MS: u64 = 2000;
const CPU_STABILIZE_MS: u64 = 200;
#[cfg(target_os = "macos")]
const MACOS_TRAY_SETUP_ATTEMPTS: usize = 5;
#[cfg(target_os = "macos")]
const MACOS_TRAY_SETUP_RETRY_MS: u64 = 1_500;
#[cfg(target_os = "macos")]
const MACOS_BUNDLE_ID: &str = "dev.alexpedersen.better-resource-monitor";
#[cfg(target_os = "macos")]
const MACOS_SUPPORTED_EXECUTABLE_PATH: &str =
    "/Applications/Better Resource Monitor.app/Contents/MacOS/better-resource-monitor";
#[cfg(target_os = "macos")]
const MACOS_PROCESS_LOCK_PREFIX: &str = "/tmp/dev.alexpedersen.better-resource-monitor";
/// Set at compile time by the direct-download CI build
/// (`.github/actions/build-direct-download/action.yml`). Direct-distribution
/// builds have no Mac App Store receipt, so the runtime guard accepts them at
/// the supported install path instead of requiring a receipt. App Store
/// builds must never set this.
#[cfg(target_os = "macos")]
const MACOS_DIRECT_DISTRIBUTION: bool = option_env!("BRM_DIRECT_DISTRIBUTION").is_some();

/// Minimum change threshold to trigger icon update (prevents compositor leak on Linux)
const HYSTERESIS_THRESHOLD: f32 = 2.0;

/// Minimum network speed change (bytes/sec) to trigger an update.
/// Reduces tray icon churn that can accumulate compositor resources on Linux.
const NET_HYSTERESIS_BPS: f64 = 50_000.0;

/// Returns true if the new value differs from previous by at least the threshold
fn should_update(prev: f32, new: f32, threshold: f32) -> bool {
    (new - prev).abs() >= threshold
}

/// Get update interval from environment variable or use default.
/// Set SILICON_UPDATE_INTERVAL to override the default cadence.
fn get_update_interval_ms() -> u64 {
    std::env::var("SILICON_UPDATE_INTERVAL")
        .ok()
        .and_then(|s| s.parse().ok())
        .filter(|interval| *interval > 0)
        .unwrap_or(UPDATE_INTERVAL_MS)
}

pub fn load_system_font() -> Result<Font<'static>, String> {
    let source = SystemSource::new();

    let handle = source
        .select_best_match(
            &[FamilyName::SansSerif],
            Properties::new().weight(Weight::NORMAL),
        )
        .or_else(|_| source.select_best_match(&[FamilyName::SansSerif], &Properties::new()))
        .map_err(|e| format!("Failed to select a system font: {e}"))?;

    let font_data = match &handle {
        Handle::Path { path, .. } => {
            std::fs::read(path).map_err(|e| format!("Failed to read font file: {e}"))?
        }
        Handle::Memory { bytes, .. } => bytes.to_vec(),
    };

    Font::try_from_vec(font_data).ok_or_else(|| "Error constructing font".to_string())
}

fn format_speed(bytes_per_sec: f64) -> String {
    const THRESHOLD_KB: f64 = 999_500.0;
    const THRESHOLD_MB: f64 = 999_500_000.0;

    let (value, unit) = if bytes_per_sec >= THRESHOLD_MB {
        (bytes_per_sec / 1_000_000_000.0, "GB")
    } else if bytes_per_sec >= THRESHOLD_KB {
        (bytes_per_sec / 1_000_000.0, "MB")
    } else {
        (bytes_per_sec / 1_000.0, "KB")
    };

    if value >= 10.0 {
        format!("{value:.0} {unit}")
    } else {
        format!("{value:.1} {unit}")
    }
}

fn sum_network_totals(networks: &Networks) -> (u64, u64) {
    networks.iter().fold((0, 0), |(rx, tx), (_, data)| {
        (rx + data.total_received(), tx + data.total_transmitted())
    })
}

fn normalize_metric_flags(
    show_cpu: bool,
    show_mem: bool,
    show_storage: bool,
    show_gpu: bool,
    show_net: bool,
    gpu_available: bool,
) -> (bool, bool, bool, bool, bool) {
    let show_gpu = show_gpu && gpu_available;

    if show_cpu || show_mem || show_storage || show_gpu || show_net {
        (show_cpu, show_mem, show_storage, show_gpu, show_net)
    } else {
        (true, show_mem, show_storage, show_gpu, show_net)
    }
}

#[cfg(target_os = "macos")]
struct MacosProcessLock {
    _file: std::fs::File,
}

#[cfg(target_os = "macos")]
fn should_exit_unsupported_macos_bundle(
    bundle_id: Option<&str>,
    executable_path: &std::path::Path,
    has_app_store_receipt: bool,
    is_direct_distribution: bool,
) -> bool {
    if bundle_id != Some(MACOS_BUNDLE_ID) {
        return false;
    }

    // Stray copies (Trash, duplicates, dev builds with the production bundle
    // id) are rejected regardless of distribution channel.
    if executable_path != std::path::Path::new(MACOS_SUPPORTED_EXECUTABLE_PATH) {
        return true;
    }

    // At the supported install path: App Store builds prove themselves with a
    // receipt; direct-download (Developer ID) builds carry the compile-time
    // marker instead.
    !has_app_store_receipt && !is_direct_distribution
}

#[cfg(target_os = "macos")]
fn macos_receipt_path_for_executable(
    executable_path: &std::path::Path,
) -> Option<std::path::PathBuf> {
    Some(
        executable_path
            .parent()?
            .parent()?
            .join("_MASReceipt")
            .join("receipt"),
    )
}

#[cfg(target_os = "macos")]
fn current_macos_bundle_id() -> Option<String> {
    NSBundle::mainBundle()
        .bundleIdentifier()
        .map(|id| id.to_string())
}

#[cfg(target_os = "macos")]
fn prevent_macos_tray_image_dimming(tray: &tray_icon::TrayIcon) {
    // Dimming is cosmetic: if any AppKit lookup fails (e.g. transient status
    // item unavailability during display sleep/wake), skip silently rather
    // than panic — this runs on the main thread on every tray update.
    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    let Some(ns_status_item) = tray.ns_status_item() else {
        return;
    };
    let Some(button) = ns_status_item.button(mtm) else {
        return;
    };
    let Some(cell) = button.cell() else {
        return;
    };
    let Some(button_cell) = cell.downcast_ref::<NSButtonCell>() else {
        return;
    };

    // Keep template tinting for theme adaptation while avoiding button-cell image dimming.
    button_cell.setImageDimsWhenDisabled(false);
}

#[cfg(target_os = "macos")]
fn enforce_supported_macos_runtime() {
    let bundle_id = current_macos_bundle_id();
    let executable_path = std::env::current_exe().ok();
    let receipt_path = executable_path
        .as_deref()
        .and_then(macos_receipt_path_for_executable);
    let has_app_store_receipt = receipt_path.as_deref().is_some_and(|path| path.is_file());
    let should_exit = executable_path
        .as_deref()
        .map(|path| {
            should_exit_unsupported_macos_bundle(
                bundle_id.as_deref(),
                path,
                has_app_store_receipt,
                MACOS_DIRECT_DISTRIBUTION,
            )
        })
        .unwrap_or(bundle_id.as_deref() == Some(MACOS_BUNDLE_ID));

    if should_exit {
        macos_diag_log(format!(
            "unsupported_runtime bundle_id={} executable={} receipt={}; exiting",
            bundle_id.as_deref().unwrap_or("<unknown>"),
            executable_path
                .as_deref()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "<unknown>".to_string()),
            receipt_path
                .as_deref()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "<unknown>".to_string())
        ));
        std::process::exit(0);
    }
}

#[cfg(target_os = "macos")]
fn macos_process_lock_path(uid: u32) -> std::path::PathBuf {
    std::path::PathBuf::from(format!("{MACOS_PROCESS_LOCK_PREFIX}.{uid}.lock"))
}

#[cfg(target_os = "macos")]
fn acquire_macos_process_lock() -> Option<MacosProcessLock> {
    let uid = unsafe { libc::geteuid() } as u32;
    let path = macos_process_lock_path(uid);
    let file = match std::fs::OpenOptions::new()
        .create(true)
        .read(true)
        .truncate(false)
        .write(true)
        .open(&path)
    {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Failed to open macOS process lock {}: {e}", path.display());
            macos_diag_log(format!(
                "process_lock open failed path={} error={e}; continuing",
                path.display()
            ));
            return None;
        }
    };

    let lock_result = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };

    if lock_result == 0 {
        macos_diag_log(format!("process_lock acquired path={}", path.display()));
        return Some(MacosProcessLock { _file: file });
    }

    let error = std::io::Error::last_os_error();
    if error.kind() == std::io::ErrorKind::WouldBlock {
        macos_diag_log(format!(
            "process_lock already held path={}; exiting duplicate",
            path.display()
        ));
        std::process::exit(0);
    }

    eprintln!(
        "Failed to acquire macOS process lock {}: {error}",
        path.display()
    );
    macos_diag_log(format!(
        "process_lock acquire failed path={} error={error}; continuing",
        path.display()
    ));
    None
}

#[cfg(target_os = "macos")]
fn build_commit() -> &'static str {
    option_env!("BRM_BUILD_COMMIT")
        .or(option_env!("GITHUB_SHA"))
        .unwrap_or("unknown")
}

#[cfg(target_os = "macos")]
fn macos_diag_log(event: impl AsRef<str>) {
    let Some(home) = std::env::var_os("HOME") else {
        return;
    };

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();
    let line = format!("[{now}] {}\n", event.as_ref());
    let home = std::path::PathBuf::from(home);
    let log_dirs = [
        home.join("Library")
            .join("Logs")
            .join("Better Resource Monitor"),
        home.join("Library")
            .join("Containers")
            .join(MACOS_BUNDLE_ID)
            .join("Data")
            .join("Library")
            .join("Logs")
            .join("Better Resource Monitor"),
    ];

    for log_dir in log_dirs {
        if std::fs::create_dir_all(&log_dir).is_err() {
            continue;
        }

        let log_path = log_dir.join("autostart.log");
        let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)
        else {
            continue;
        };
        let _ = file.write_all(line.as_bytes());
    }
}

#[cfg(test)]
mod tests;

fn toggle_setting(
    app: &AppHandle,
    key: &str,
    flag: &AtomicBool,
    all_flags: [bool; 5],
    item: &CheckMenuItem<tauri::Wry>,
) {
    let current = flag.load(Relaxed);
    let enabled_count = all_flags.iter().filter(|v| **v).count();
    if !current || enabled_count > 1 {
        flag.store(!current, Relaxed);
        save_setting(app, key, !current);
    } else {
        let _ = item.set_checked(true);
    }
}

#[cfg(target_os = "macos")]
#[derive(Clone)]
struct ThermalUiHandle(Arc<Mutex<Option<ThermalUi>>>);

#[cfg(target_os = "macos")]
impl Default for ThermalUiHandle {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(None)))
    }
}

#[cfg(target_os = "macos")]
struct ThermalUi {
    menu: Menu<tauri::Wry>,
    status_item: MenuItem<tauri::Wry>,
    status_position: usize,
    present: bool,
}

#[cfg(target_os = "macos")]
#[derive(Clone)]
struct ThermalRuntime {
    status: Arc<Mutex<thermal::ThermalStatus>>,
    ui: ThermalUiHandle,
}

#[cfg(target_os = "macos")]
impl ThermalRuntime {
    fn new() -> Self {
        Self {
            status: Arc::new(Mutex::new(thermal::ThermalStatus::Unavailable)),
            ui: ThermalUiHandle::default(),
        }
    }

    fn status(&self) -> thermal::ThermalStatus {
        *self
            .status
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn set_status(&self, status: thermal::ThermalStatus) {
        *self
            .status
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = status;
    }
}

#[cfg(target_os = "macos")]
fn thermal_copy(
    translations: &i18n::Translations,
    status: thermal::ThermalStatus,
) -> Option<(&'static str, &'static str)> {
    match status {
        thermal::ThermalStatus::Nominal => Some((
            translations.thermal_nominal,
            translations.thermal_nominal_explanation,
        )),
        thermal::ThermalStatus::Fair => Some((
            translations.thermal_fair,
            translations.thermal_fair_explanation,
        )),
        thermal::ThermalStatus::Serious => Some((
            translations.thermal_serious,
            translations.thermal_serious_explanation,
        )),
        thermal::ThermalStatus::Critical => Some((
            translations.thermal_critical,
            translations.thermal_critical_explanation,
        )),
        thermal::ThermalStatus::Unavailable => None,
    }
}

#[cfg(target_os = "macos")]
fn thermal_tray_label(
    translations: &i18n::Translations,
    status: thermal::ThermalStatus,
) -> &'static str {
    match status {
        thermal::ThermalStatus::Nominal => translations.thermal_nominal_short,
        thermal::ThermalStatus::Fair => translations.thermal_fair_short,
        thermal::ThermalStatus::Serious => translations.thermal_serious_short,
        thermal::ThermalStatus::Critical => translations.thermal_critical_short,
        thermal::ThermalStatus::Unavailable => "",
    }
}

#[cfg(target_os = "macos")]
fn thermal_status_text(
    translations: &i18n::Translations,
    status: thermal::ThermalStatus,
) -> Option<String> {
    let (state, explanation) = thermal_copy(translations, status)?;
    Some(format!(
        "{}: {} — {}",
        translations.thermal_status, state, explanation
    ))
}

#[cfg(target_os = "macos")]
fn update_thermal_ui(
    app: &AppHandle,
    ui_handle: &ThermalUiHandle,
    translations: &'static i18n::Translations,
    enabled: bool,
    status: thermal::ThermalStatus,
) {
    let available = enabled && status.is_available();
    let status_text = thermal_status_text(translations, status);
    let mut guard = ui_handle
        .0
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let Some(ui) = guard.as_mut() else {
        return;
    };

    if available != ui.present {
        let result = if available {
            ui.menu.insert(&ui.status_item, ui.status_position)
        } else {
            ui.menu.remove(&ui.status_item)
        };
        if let Err(error) = result {
            eprintln!("Failed to update thermal status menu row: {error}");
        } else {
            ui.present = available;
        }
    }

    if let Some(text) = status_text.as_deref() {
        if let Err(error) = ui.status_item.set_text(text) {
            eprintln!("Failed to update thermal status menu text: {error}");
        }
    }

    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let tooltip = if available {
            status_text.as_deref()
        } else {
            Some(translations.system_monitor)
        };
        if let Err(error) = tray.set_tooltip(tooltip) {
            eprintln!("Failed to update thermal status tooltip: {error}");
        }
    }
}

fn setup_tray(
    app: &AppHandle,
    font: &Font,
    metrics: MetricToggles,
    gpu_available: bool,
    is_autostart_enabled: bool,
    translations: &i18n::Translations,
    #[cfg(target_os = "macos")] thermal_runtime: &ThermalRuntime,
) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "macos")]
    let thermal_status = thermal_runtime.status();
    #[cfg(target_os = "macos")]
    let thermal_label = thermal_tray_label(translations, thermal_status);
    #[cfg(target_os = "macos")]
    {
        macos_diag_log(format!(
            "setup_tray autostart_requested={is_autostart_enabled} status={}",
            macos_autostart::status_label()
        ));
    }
    #[cfg(target_os = "linux")]
    {
        let manager = app.autolaunch();
        if is_autostart_enabled {
            if let Err(e) = manager.enable() {
                eprintln!("Failed to enable autostart: {e}");
            }
        } else if let Err(e) = manager.disable() {
            eprintln!("Failed to disable autostart: {e}");
        }
    }

    let autostart_item = CheckMenuItem::with_id(
        app,
        menu_id::AUTOSTART,
        translations.start_at_login,
        true,
        is_autostart_enabled,
        None::<&str>,
    )?;

    let separator1 = PredefinedMenuItem::separator(app)?;

    let show_mem_item = CheckMenuItem::with_id(
        app,
        menu_id::SHOW_MEM,
        translations.show_memory,
        true,
        metrics.show_mem.load(Relaxed),
        None::<&str>,
    )?;

    let show_cpu_item = CheckMenuItem::with_id(
        app,
        menu_id::SHOW_CPU,
        translations.show_cpu,
        true,
        metrics.show_cpu.load(Relaxed),
        None::<&str>,
    )?;

    let show_storage_item = CheckMenuItem::with_id(
        app,
        menu_id::SHOW_STORAGE,
        translations.show_storage,
        true,
        metrics.show_storage.load(Relaxed),
        None::<&str>,
    )?;

    let show_net_item = CheckMenuItem::with_id(
        app,
        menu_id::SHOW_NET,
        translations.show_network,
        true,
        metrics.show_net.load(Relaxed),
        None::<&str>,
    )?;

    #[cfg(target_os = "macos")]
    let show_thermal_item = CheckMenuItem::with_id(
        app,
        menu_id::SHOW_THERMAL,
        translations.show_thermal_status,
        true,
        metrics.show_thermal.load(Relaxed),
        None::<&str>,
    )?;

    #[cfg(target_os = "macos")]
    let thermal_status_item = MenuItem::with_id(
        app,
        menu_id::THERMAL_STATUS,
        thermal_status_text(translations, thermal_status).unwrap_or_default(),
        false,
        None::<&str>,
    )?;

    let separator2 = PredefinedMenuItem::separator(app)?;

    let show_alerts_item = CheckMenuItem::with_id(
        app,
        menu_id::SHOW_ALERTS,
        translations.show_alert_colors,
        true,
        metrics.show_alerts.load(Relaxed),
        None::<&str>,
    )?;

    let separator3 = PredefinedMenuItem::separator(app)?;
    let quit_item = MenuItem::with_id(app, menu_id::QUIT, translations.quit, true, None::<&str>)?;

    let show_gpu_item = CheckMenuItem::with_id(
        app,
        menu_id::SHOW_GPU,
        translations.show_gpu,
        true,
        metrics.show_gpu.load(Relaxed),
        None::<&str>,
    )?;

    let menu = Menu::new(app)?;
    menu.append(&autostart_item)?;
    menu.append(&separator1)?;
    menu.append(&show_mem_item)?;
    menu.append(&show_cpu_item)?;
    menu.append(&show_storage_item)?;
    if gpu_available {
        menu.append(&show_gpu_item)?;
    }
    menu.append(&show_net_item)?;
    #[cfg(target_os = "macos")]
    menu.append(&show_thermal_item)?;
    #[cfg(target_os = "macos")]
    let thermal_status_position = 7 + usize::from(gpu_available);
    #[cfg(target_os = "macos")]
    let thermal_status_present =
        metrics.show_thermal.load(Relaxed) && thermal_status.is_available();
    #[cfg(target_os = "macos")]
    if thermal_status_present {
        menu.append(&thermal_status_item)?;
    }
    menu.append(&separator2)?;
    menu.append(&show_alerts_item)?;
    menu.append(&separator3)?;
    menu.append(&quit_item)?;

    #[cfg(target_os = "linux")]
    let use_light_icons = detect_light_icons();
    #[cfg(not(target_os = "linux"))]
    let use_light_icons = true;

    let mut renderer = tray_render::TrayRenderer::new();
    let mut initial_buffer = Vec::with_capacity(4 * 800 * APP_SIZING.icon_height as usize);
    let (width, height, _has_alert) = renderer.render_tray_icon_into(
        font,
        &mut initial_buffer,
        &tray_render::RenderConfig {
            sizing: APP_SIZING,
            cpu_usage: 0.0,
            mem_percent: 0.0,
            storage_percent: 0.0,
            gpu_usage: 0.0,
            down_str: "0 KB",
            up_str: "0 KB",
            show_cpu: metrics.show_cpu.load(Relaxed),
            show_mem: metrics.show_mem.load(Relaxed),
            show_storage: metrics.show_storage.load(Relaxed),
            show_gpu: metrics.show_gpu.load(Relaxed) && gpu_available,
            show_net: metrics.show_net.load(Relaxed),
            show_alerts: metrics.show_alerts.load(Relaxed),
            use_light_icons,
            background: None,
            #[cfg(target_os = "macos")]
            show_thermal: metrics.show_thermal.load(Relaxed),
            #[cfg(target_os = "macos")]
            thermal_status,
            #[cfg(target_os = "macos")]
            thermal_label,
        },
    );
    let initial_icon = Image::new_owned(initial_buffer, width, height);

    #[cfg(target_os = "macos")]
    {
        thermal_runtime.set_status(thermal_status);
        *thermal_runtime
            .ui
            .0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(ThermalUi {
            menu: menu.clone(),
            status_item: thermal_status_item.clone(),
            status_position: thermal_status_position,
            present: thermal_status_present,
        });
    }

    let tray_builder = TrayIconBuilder::with_id(TRAY_ID).icon(initial_icon);

    #[cfg(target_os = "macos")]
    let tray_builder = tray_builder.icon_as_template(true);

    let cpu_item = show_cpu_item.clone();
    let mem_item = show_mem_item.clone();
    let storage_item = show_storage_item.clone();
    let gpu_item = show_gpu_item.clone();
    let net_item = show_net_item.clone();
    let autostart_menu_item = autostart_item.clone();

    #[cfg(target_os = "macos")]
    let initial_tooltip = if metrics.show_thermal.load(Relaxed) {
        thermal_status_text(translations, thermal_status)
            .unwrap_or_else(|| translations.system_monitor.to_string())
    } else {
        translations.system_monitor.to_string()
    };
    #[cfg(not(target_os = "macos"))]
    let initial_tooltip = translations.system_monitor.to_string();

    let _tray = tray_builder
        .menu(&menu)
        .show_menu_on_left_click(true)
        .tooltip(initial_tooltip)
        .on_menu_event(move |app, event| {
            let flags = [
                metrics.show_cpu.load(Relaxed),
                metrics.show_mem.load(Relaxed),
                metrics.show_storage.load(Relaxed),
                metrics.show_gpu.load(Relaxed) && gpu_available,
                metrics.show_net.load(Relaxed),
            ];
            match event.id.as_ref() {
                menu_id::AUTOSTART => {
                    #[cfg(target_os = "macos")]
                    {
                        let enabled = macos_autostart::is_enabled();
                        if enabled {
                            match macos_autostart::disable() {
                                Ok(()) => {
                                    save_setting(app, menu_id::AUTOSTART, false);
                                    let _ = autostart_menu_item.set_checked(false);
                                    macos_diag_log(format!(
                                        "menu autostart disabled status_after={}",
                                        macos_autostart::status_label()
                                    ));
                                }
                                Err(e) => {
                                    eprintln!("Failed to disable autostart: {e}");
                                    let _ = autostart_menu_item.set_checked(true);
                                    macos_diag_log(format!(
                                        "menu autostart disable failed status_after={} error={e}",
                                        macos_autostart::status_label()
                                    ));
                                }
                            }
                        } else {
                            match macos_autostart::enable() {
                                Ok(()) => {
                                    save_setting(app, menu_id::AUTOSTART, true);
                                    let _ = autostart_menu_item.set_checked(true);
                                    macos_diag_log(format!(
                                        "menu autostart enabled status_after={}",
                                        macos_autostart::status_label()
                                    ));
                                }
                                Err(e) => {
                                    eprintln!("Failed to enable autostart: {e}");
                                    let _ = autostart_menu_item.set_checked(false);
                                    macos_diag_log(format!(
                                        "menu autostart enable failed status_after={} error={e}",
                                        macos_autostart::status_label()
                                    ));
                                }
                            }
                        }
                    }
                    #[cfg(target_os = "linux")]
                    {
                        let manager = app.autolaunch();
                        let enabled = manager.is_enabled().unwrap_or(false);
                        if enabled {
                            match manager.disable() {
                                Ok(()) => {
                                    save_setting(app, menu_id::AUTOSTART, false);
                                    let _ = autostart_menu_item.set_checked(false);
                                }
                                Err(e) => {
                                    eprintln!("Failed to disable autostart: {e}");
                                    let _ = autostart_menu_item.set_checked(true);
                                }
                            }
                        } else {
                            match manager.enable() {
                                Ok(()) => {
                                    save_setting(app, menu_id::AUTOSTART, true);
                                    let _ = autostart_menu_item.set_checked(true);
                                }
                                Err(e) => {
                                    eprintln!("Failed to enable autostart: {e}");
                                    let _ = autostart_menu_item.set_checked(false);
                                }
                            }
                        }
                    }
                }
                menu_id::SHOW_CPU => {
                    toggle_setting(app, menu_id::SHOW_CPU, &metrics.show_cpu, flags, &cpu_item)
                }
                menu_id::SHOW_MEM => {
                    toggle_setting(app, menu_id::SHOW_MEM, &metrics.show_mem, flags, &mem_item)
                }
                menu_id::SHOW_STORAGE => toggle_setting(
                    app,
                    menu_id::SHOW_STORAGE,
                    &metrics.show_storage,
                    flags,
                    &storage_item,
                ),
                menu_id::SHOW_GPU => {
                    toggle_setting(app, menu_id::SHOW_GPU, &metrics.show_gpu, flags, &gpu_item)
                }
                menu_id::SHOW_NET => {
                    toggle_setting(app, menu_id::SHOW_NET, &metrics.show_net, flags, &net_item)
                }
                #[cfg(target_os = "macos")]
                menu_id::SHOW_THERMAL => {
                    let new_value = !metrics.show_thermal.load(Relaxed);
                    metrics.show_thermal.store(new_value, Relaxed);
                    save_setting(app, menu_id::SHOW_THERMAL, new_value);
                }
                menu_id::SHOW_ALERTS => {
                    let new_value = !metrics.show_alerts.load(Relaxed);
                    metrics.show_alerts.store(new_value, Relaxed);
                    save_setting(app, menu_id::SHOW_ALERTS, new_value);
                }
                menu_id::QUIT => app.exit(0),
                _ => {}
            }
        })
        .build(app)?;

    #[cfg(target_os = "macos")]
    {
        _tray
            .with_inner_tray_icon(prevent_macos_tray_image_dimming)
            .expect("failed to apply macOS tray image dimming hook");
        macos_diag_log("tray build ok");
    }

    Ok(())
}

fn setup_initial_tray(
    app: &AppHandle,
    metrics: MetricToggles,
    gpu_available: bool,
    autostart: bool,
    translations: &'static i18n::Translations,
    #[cfg(target_os = "macos")] thermal_runtime: &ThermalRuntime,
    #[cfg(target_os = "macos")] thermal_status: thermal::ThermalStatus,
) -> Result<Font<'static>, Box<dyn std::error::Error>> {
    #[cfg(target_os = "macos")]
    let attempts = MACOS_TRAY_SETUP_ATTEMPTS;
    #[cfg(not(target_os = "macos"))]
    let attempts = 1;

    let mut last_error = String::new();

    for attempt in 1..=attempts {
        match load_system_font()
            .map_err(|e| format!("Font required for tray icon: {e}"))
            .and_then(|font| {
                #[cfg(target_os = "macos")]
                thermal_runtime.set_status(thermal_status);
                setup_tray(
                    app,
                    &font,
                    metrics.clone(),
                    gpu_available,
                    autostart,
                    translations,
                    #[cfg(target_os = "macos")]
                    thermal_runtime,
                )
                .map(|()| font)
                .map_err(|e| e.to_string())
            }) {
            Ok(font) => {
                #[cfg(target_os = "macos")]
                macos_diag_log(format!(
                    "setup_initial_tray attempt={attempt}/{attempts} ok"
                ));
                return Ok(font);
            }
            Err(error) => {
                last_error = error;

                #[cfg(target_os = "macos")]
                macos_diag_log(format!(
                    "setup_initial_tray attempt={attempt}/{attempts} failed error={last_error}"
                ));

                #[cfg(target_os = "macos")]
                if attempt < attempts {
                    eprintln!(
                        "Tray setup attempt {attempt}/{attempts} failed: {last_error}; retrying"
                    );
                    thread::sleep(Duration::from_millis(
                        MACOS_TRAY_SETUP_RETRY_MS * attempt as u64,
                    ));
                }
            }
        }
    }

    Err(last_error.into())
}

fn handle_second_instance_launch(
    app: &AppHandle,
    metrics: MetricToggles,
    gpu_available: bool,
    #[cfg(target_os = "macos")] thermal_runtime: ThermalRuntime,
) {
    #[cfg(target_os = "macos")]
    {
        macos_diag_log("second_instance launch received");
        if let Err(e) = app.set_activation_policy(ActivationPolicy::Accessory) {
            eprintln!("Failed to set activation policy on second launch: {e}");
            macos_diag_log(format!(
                "second_instance activation_policy failed error={e}"
            ));
        } else {
            macos_diag_log("second_instance activation_policy ok");
        }

        macos_diag_log(format!(
            "second_instance status={}",
            macos_autostart::status_label()
        ));

        if let Some(tray) = app.tray_by_id(TRAY_ID) {
            if let Err(e) = tray.set_visible(true) {
                eprintln!("Failed to show tray icon on second launch: {e}");
                macos_diag_log(format!("second_instance tray show failed error={e}"));
            } else {
                macos_diag_log("second_instance tray show ok");
            }
        } else {
            macos_diag_log("second_instance tray missing; rebuilding");
            let settings = load_settings(app);
            let (cpu, mem, storage, gpu, net) = normalize_metric_flags(
                settings.show_cpu,
                settings.show_mem,
                settings.show_storage,
                settings.show_gpu,
                settings.show_net,
                gpu_available,
            );
            metrics.show_cpu.store(cpu, Relaxed);
            metrics.show_mem.store(mem, Relaxed);
            metrics.show_storage.store(storage, Relaxed);
            metrics.show_gpu.store(gpu, Relaxed);
            metrics.show_net.store(net, Relaxed);
            metrics.show_alerts.store(settings.show_alerts, Relaxed);

            #[cfg(target_os = "macos")]
            metrics.show_thermal.store(settings.show_thermal, Relaxed);

            let translations = i18n::detect_language().translations();
            let autostart = macos_autostart::is_enabled();
            #[cfg(target_os = "macos")]
            let thermal_status = if settings.show_thermal {
                thermal::sample()
            } else {
                thermal::ThermalStatus::Unavailable
            };
            if let Err(e) = setup_initial_tray(
                app,
                metrics,
                gpu_available,
                autostart,
                translations,
                #[cfg(target_os = "macos")]
                &thermal_runtime,
                #[cfg(target_os = "macos")]
                thermal_status,
            ) {
                eprintln!("Failed to restore tray icon on second launch: {e}");
                macos_diag_log(format!("second_instance tray rebuild failed error={e}"));
            } else {
                macos_diag_log("second_instance tray rebuild ok");
            }
        }
    }
}

fn start_monitoring(
    app: AppHandle,
    font: Font<'static>,
    metrics: MetricToggles,
    mut gpu_sampler: Option<GpuSampler>,
    #[cfg(target_os = "macos")] thermal_runtime: ThermalRuntime,
    #[cfg(target_os = "macos")] translations: &'static i18n::Translations,
) {
    thread::spawn(move || {
        let mut sys = System::new();
        // Warm up CPU measurement before loop so first render has valid data
        sys.refresh_cpu_usage();
        thread::sleep(Duration::from_millis(CPU_STABILIZE_MS));

        let mut networks = Networks::new_with_refreshed_list();

        // Initialize network counters from current values to avoid spike on first iteration
        let (mut prev_rx, mut prev_tx) = sum_network_totals(&networks);
        let mut gpu_usage: f32 = 0.0;
        let mut last_update = std::time::Instant::now();

        // Track previous values for hysteresis-based updates (prevents compositor leak on Linux)
        let mut prev_cpu: f32 = -100.0; // Force initial update
        let mut prev_mem: f32 = -100.0;
        let mut prev_storage: f32 = -100.0;
        let mut prev_gpu: f32 = -100.0;
        let mut prev_down_speed: f64 = -1.0;
        let mut prev_up_speed: f64 = -1.0;
        let mut prev_flags: (bool, bool, bool, bool, bool, bool, bool) =
            (false, false, false, false, false, false, false);
        #[cfg(target_os = "macos")]
        let mut thermal_tracker = thermal::ThermalTracker::new(thermal_runtime.status());
        #[cfg(target_os = "macos")]
        let mut thermal_last_sample = if metrics.show_thermal.load(Relaxed) {
            Some(std::time::Instant::now())
        } else {
            None
        };
        #[cfg(target_os = "macos")]
        let mut prev_thermal_enabled = metrics.show_thermal.load(Relaxed);
        #[cfg(target_os = "macos")]
        let mut prev_thermal_status = thermal_tracker.displayed();
        let update_interval = get_update_interval_ms();
        let mut tick_count: u32 = 0;

        // Reusable buffer owned by monitoring thread - prevents compositor resource
        // accumulation on Linux that causes cursor slowdown
        let mut renderer = tray_render::TrayRenderer::new();
        let mut render_buffer: Vec<u8> =
            Vec::with_capacity(4 * 800 * APP_SIZING.icon_height as usize);

        loop {
            thread::sleep(Duration::from_millis(update_interval));

            let now = std::time::Instant::now();
            let dt = now.duration_since(last_update).as_secs_f64();
            last_update = now;
            let full_tick = tick_count.is_multiple_of(2);
            tick_count = tick_count.wrapping_add(1);

            let sc = metrics.show_cpu.load(Relaxed);
            let sm = metrics.show_mem.load(Relaxed);
            let ss = metrics.show_storage.load(Relaxed);
            let show_gpu_enabled = metrics.show_gpu.load(Relaxed);
            let sg = show_gpu_enabled && gpu_sampler.is_some();
            let sn = metrics.show_net.load(Relaxed);
            let sa = metrics.show_alerts.load(Relaxed);
            #[cfg(target_os = "macos")]
            let st = metrics.show_thermal.load(Relaxed);

            #[cfg(target_os = "linux")]
            let current_flags = (sc, sm, ss, sg, sn, sa, detect_light_icons());
            #[cfg(not(target_os = "linux"))]
            let current_flags = (sc, sm, ss, sg, sn, sa, false);

            let flags_changed = prev_flags != current_flags;
            let net_was_enabled = prev_flags.4;

            #[cfg(target_os = "macos")]
            let thermal_status = {
                let setting_changed = st != prev_thermal_enabled;
                if !st {
                    thermal_last_sample = None;
                } else {
                    if setting_changed {
                        thermal_last_sample = None;
                    }
                    if thermal::should_poll(st, thermal_last_sample, now) {
                        let sampled = thermal::sample();
                        thermal_last_sample = Some(now);
                        thermal_tracker.observe(sampled);
                        thermal_runtime.set_status(thermal_tracker.displayed());
                    } else {
                        let shared = thermal_runtime.status();
                        if shared != thermal_tracker.displayed() {
                            thermal_tracker = thermal::ThermalTracker::new(shared);
                        }
                    }
                }
                let status = thermal_tracker.displayed();
                if setting_changed || status != prev_thermal_status {
                    update_thermal_ui(&app, &thermal_runtime.ui, translations, st, status);
                }
                status
            };
            #[cfg(target_os = "macos")]
            let thermal_changed =
                st != prev_thermal_enabled || thermal_status != prev_thermal_status;
            #[cfg(not(target_os = "macos"))]
            let thermal_changed = false;

            // Refresh only metrics currently visible in the tray
            if sc {
                sys.refresh_cpu_usage();
            }
            if full_tick && sm {
                sys.refresh_memory();
            }
            if sn {
                networks.refresh(false);
            }

            let cpu_usage = if sc { sys.global_cpu_usage() } else { 0.0 };

            let mem_percent = if sm {
                let used_mem = sys.used_memory() as f64;
                let total_mem = sys.total_memory() as f64;
                if total_mem > 0.0 {
                    (used_mem / total_mem * 100.0) as f32
                } else {
                    0.0
                }
            } else {
                0.0
            };

            let storage_percent = if full_tick && ss {
                storage::sample().map(|s| s.used_percent).unwrap_or(0.0)
            } else if ss {
                prev_storage.max(0.0)
            } else {
                0.0
            };

            let (down_speed, up_speed) = if sn {
                let (total_rx, total_tx) = sum_network_totals(&networks);
                if net_was_enabled {
                    let down_speed = total_rx.saturating_sub(prev_rx) as f64 / dt;
                    let up_speed = total_tx.saturating_sub(prev_tx) as f64 / dt;
                    (prev_rx, prev_tx) = (total_rx, total_tx);
                    (down_speed, up_speed)
                } else {
                    (prev_rx, prev_tx) = (total_rx, total_tx);
                    (0.0, 0.0)
                }
            } else {
                (0.0, 0.0)
            };

            if sg && full_tick {
                if let Some(ref mut sampler) = gpu_sampler {
                    gpu_usage = sampler.sample().unwrap_or(0.0);
                }
            } else if !sg {
                gpu_usage = 0.0;
            }

            // Hysteresis: only update if values change by meaningful threshold
            // This dramatically reduces icon updates, preventing compositor resource
            // accumulation that causes cursor slowdown on Ubuntu/GNOME
            let cpu_changed = should_update(prev_cpu, cpu_usage, HYSTERESIS_THRESHOLD);
            let mem_changed = should_update(prev_mem, mem_percent, HYSTERESIS_THRESHOLD);
            let storage_changed =
                should_update(prev_storage, storage_percent, HYSTERESIS_THRESHOLD);
            let gpu_changed = should_update(prev_gpu, gpu_usage, HYSTERESIS_THRESHOLD);
            let down_diff = (down_speed - prev_down_speed).abs();
            let up_diff = (up_speed - prev_up_speed).abs();
            let net_value_changed =
                down_diff >= NET_HYSTERESIS_BPS || up_diff >= NET_HYSTERESIS_BPS;
            let net_changed = sn && net_value_changed;

            if cpu_changed
                || mem_changed
                || storage_changed
                || gpu_changed
                || net_changed
                || flags_changed
                || thermal_changed
            {
                // Defer string formatting to render time only
                let down_str = format_speed(down_speed);
                let up_str = format_speed(up_speed);

                if sc {
                    prev_cpu = cpu_usage;
                }
                if sm {
                    prev_mem = mem_percent;
                }
                if ss {
                    prev_storage = storage_percent;
                }
                if sg {
                    prev_gpu = gpu_usage;
                }
                if sn {
                    prev_down_speed = down_speed;
                    prev_up_speed = up_speed;
                }
                prev_flags = current_flags;
                #[cfg(target_os = "macos")]
                {
                    prev_thermal_enabled = st;
                    prev_thermal_status = thermal_status;
                }

                let (width, height, _has_active_alert) = renderer.render_tray_icon_into(
                    &font,
                    &mut render_buffer,
                    &tray_render::RenderConfig {
                        sizing: APP_SIZING,
                        cpu_usage,
                        mem_percent,
                        storage_percent,
                        gpu_usage,
                        down_str: &down_str,
                        up_str: &up_str,
                        show_cpu: sc,
                        show_mem: sm,
                        show_storage: ss,
                        show_gpu: sg,
                        show_net: sn,
                        show_alerts: sa,
                        use_light_icons: current_flags.6,
                        background: None,
                        #[cfg(target_os = "macos")]
                        show_thermal: st,
                        #[cfg(target_os = "macos")]
                        thermal_status,
                        #[cfg(target_os = "macos")]
                        thermal_label: thermal_tray_label(translations, thermal_status),
                    },
                );

                // Icon::from_rgba (and Image::new_owned on other platforms)
                // requires exactly 4 bytes per pixel; a mismatch here would
                // otherwise fail the icon update every tick.
                if render_buffer.len() != (4 * width * height) as usize {
                    eprintln!(
                        "tray_update buffer size mismatch len={} expected={}",
                        render_buffer.len(),
                        4 * width * height
                    );
                    #[cfg(target_os = "macos")]
                    macos_diag_log(format!(
                        "tray_update buffer size mismatch len={} expected={}",
                        render_buffer.len(),
                        4 * width * height
                    ));
                    continue;
                }

                if let Some(tray) = app.tray_by_id(TRAY_ID) {
                    // A transient failure here (e.g. AppKit contention during
                    // display sleep/wake) must not kill the monitor thread;
                    // log, skip this frame, and let the next tick retry.
                    #[cfg(target_os = "macos")]
                    {
                        let use_template = !_has_active_alert;
                        match tray_icon::Icon::from_rgba(render_buffer.clone(), width, height) {
                            Ok(icon) => {
                                let result = tray.with_inner_tray_icon(move |inner| {
                                    if let Err(e) =
                                        inner.set_icon_with_as_template(Some(icon), use_template)
                                    {
                                        eprintln!("Failed to set macOS tray icon: {e}");
                                    }
                                    prevent_macos_tray_image_dimming(inner);
                                });
                                if let Err(e) = result {
                                    macos_diag_log(format!(
                                        "tray_update main-thread dispatch failed: {e}"
                                    ));
                                }
                            }
                            Err(e) => {
                                macos_diag_log(format!("tray_update icon creation failed: {e}"));
                            }
                        }
                    }

                    #[cfg(not(target_os = "macos"))]
                    {
                        let icon = Image::new_owned(render_buffer.clone(), width, height);
                        let _ = tray.set_icon(Some(icon));
                    }
                }
            }
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(target_os = "macos")]
    enforce_supported_macos_runtime();

    #[cfg(target_os = "macos")]
    let _macos_process_lock = acquire_macos_process_lock();

    #[cfg(target_os = "linux")]
    if let Err(e) = ensure_display_available() {
        eprintln!("{e}");
        std::process::exit(1);
    }

    let metrics = MetricToggles {
        show_cpu: Arc::new(AtomicBool::new(true)),
        show_mem: Arc::new(AtomicBool::new(true)),
        show_storage: Arc::new(AtomicBool::new(true)),
        show_gpu: Arc::new(AtomicBool::new(true)),
        show_net: Arc::new(AtomicBool::new(true)),
        show_alerts: Arc::new(AtomicBool::new(true)),
        #[cfg(target_os = "macos")]
        show_thermal: Arc::new(AtomicBool::new(DEFAULT_SHOW_THERMAL_STATUS)),
    };
    let tray_metrics = metrics.clone();
    let second_instance_metrics = metrics.clone();

    #[cfg(target_os = "macos")]
    let thermal_runtime = ThermalRuntime::new();
    #[cfg(target_os = "macos")]
    let second_instance_thermal_runtime = thermal_runtime.clone();

    let gpu_sampler = GpuSampler::new();
    let gpu_available = gpu_sampler.is_some();

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(
            move |app, _args, _cwd| {
                handle_second_instance_launch(
                    app,
                    second_instance_metrics.clone(),
                    gpu_available,
                    #[cfg(target_os = "macos")]
                    second_instance_thermal_runtime.clone(),
                );
            },
        ))
        .plugin(tauri_plugin_store::Builder::new().build());

    builder
        .setup(move |app| {
            #[cfg(target_os = "macos")]
            {
                macos_diag_log(format!(
                    "setup start version={} build_commit={} status_before={}",
                    env!("CARGO_PKG_VERSION"),
                    build_commit(),
                    macos_autostart::status_label()
                ));
                app.set_activation_policy(ActivationPolicy::Accessory);
                macos_diag_log("activation_policy ok");
            }

            #[cfg(target_os = "linux")]
            {
                app.handle().plugin(tauri_plugin_autostart::init(
                    MacosLauncher::LaunchAgent,
                    None,
                ))?;
            }

            #[cfg(target_os = "linux")]
            start_theme_detection_thread();

            let settings = load_settings(app.handle());
            let (cpu, mem, storage, gpu, net) =
                normalize_metric_flags(
                    settings.show_cpu,
                    settings.show_mem,
                    settings.show_storage,
                    settings.show_gpu,
                    settings.show_net,
                    gpu_available,
                );
            #[cfg(target_os = "macos")]
            // The macOS menu reflects live SMAppService state only.
            let autostart = macos_autostart::is_enabled();
            #[cfg(not(target_os = "macos"))]
            let autostart = settings.autostart;
            #[cfg(target_os = "macos")]
            macos_diag_log(format!(
                "settings loaded stored_autostart={} system_autostart={autostart} metrics cpu={cpu} mem={mem} storage={storage} gpu={gpu} net={net} alerts={} thermal={} gpu_available={gpu_available}",
                settings.autostart,
                settings.show_alerts,
                settings.show_thermal,
            ));
            tray_metrics.show_cpu.store(cpu, Relaxed);
            tray_metrics.show_mem.store(mem, Relaxed);
            tray_metrics.show_storage.store(storage, Relaxed);
            tray_metrics.show_gpu.store(gpu, Relaxed);
            tray_metrics.show_net.store(net, Relaxed);
            tray_metrics.show_alerts.store(settings.show_alerts, Relaxed);
            #[cfg(target_os = "macos")]
            tray_metrics.show_thermal.store(settings.show_thermal, Relaxed);

            let translations = i18n::detect_language().translations();

            #[cfg(target_os = "macos")]
            let thermal_status = if settings.show_thermal {
                thermal::sample()
            } else {
                thermal::ThermalStatus::Unavailable
            };

            let font = setup_initial_tray(
                app.handle(),
                tray_metrics,
                gpu_available,
                autostart,
                translations,
                #[cfg(target_os = "macos")]
                &thermal_runtime,
                #[cfg(target_os = "macos")]
                thermal_status,
            )?;

            start_monitoring(
                app.handle().clone(),
                font,
                metrics,
                gpu_sampler,
                #[cfg(target_os = "macos")]
                thermal_runtime,
                #[cfg(target_os = "macos")]
                translations,
            );

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
