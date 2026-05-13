mod gpu;
pub mod i18n;
#[cfg(target_os = "macos")]
mod macos_autostart;
pub mod tray_render;

use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
use std::sync::Arc;
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
use std::io::Write;

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

mod menu_id {
    pub const AUTOSTART: &str = "autostart";
    pub const SHOW_CPU: &str = "show_cpu";
    pub const SHOW_MEM: &str = "show_mem";
    pub const SHOW_GPU: &str = "show_gpu";
    pub const SHOW_NET: &str = "show_net";
    pub const SHOW_ALERTS: &str = "show_alerts";
    pub const QUIT: &str = "quit";
}

const TRAY_ID: &str = "main";

#[derive(Clone)]
struct MetricToggles {
    show_cpu: Arc<AtomicBool>,
    show_mem: Arc<AtomicBool>,
    show_gpu: Arc<AtomicBool>,
    show_net: Arc<AtomicBool>,
    show_alerts: Arc<AtomicBool>,
}

fn load_settings(app: &AppHandle) -> (bool, bool, bool, bool, bool, bool) {
    let store = match app.store(SETTINGS_FILE) {
        Ok(s) => Some(s),
        Err(e) => {
            eprintln!("Failed to load settings store: {e}");
            None
        }
    };

    let get_bool = |key: &str, default: bool| -> bool {
        store
            .as_ref()
            .and_then(|s| s.get(key))
            .and_then(|v| v.as_bool())
            .unwrap_or(default)
    };

    (
        get_bool("show_cpu", true),
        get_bool("show_mem", true),
        get_bool("show_gpu", true),
        get_bool("show_net", true),
        get_bool("show_alerts", true),
        get_bool(menu_id::AUTOSTART, false),
    )
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
    show_gpu: bool,
    show_net: bool,
    gpu_available: bool,
) -> (bool, bool, bool, bool) {
    let show_gpu = show_gpu && gpu_available;

    if show_cpu || show_mem || show_gpu || show_net {
        (show_cpu, show_mem, show_gpu, show_net)
    } else {
        (true, show_mem, show_gpu, show_net)
    }
}

fn should_repair_macos_autostart(stored_autostart: bool, system_autostart: bool) -> bool {
    stored_autostart || system_autostart
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
    all_flags: [bool; 4],
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

fn setup_tray(
    app: &AppHandle,
    font: &Font,
    metrics: MetricToggles,
    gpu_available: bool,
    is_autostart_enabled: bool,
    translations: &i18n::Translations,
) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "macos")]
    {
        macos_diag_log(format!(
            "setup_tray autostart_requested={is_autostart_enabled} status_before={}",
            macos_autostart::status_label()
        ));
        if is_autostart_enabled {
            match macos_autostart::repair() {
                Ok(()) => macos_diag_log(format!(
                    "autostart repair ok status_after={}",
                    macos_autostart::status_label()
                )),
                Err(e) => {
                    eprintln!("Failed to repair autostart registration: {e}");
                    macos_diag_log(format!(
                        "autostart repair failed status_after={} error={e}",
                        macos_autostart::status_label()
                    ));
                }
            }
        } else {
            match macos_autostart::disable() {
                Ok(()) => macos_diag_log(format!(
                    "autostart disable ok status_after={}",
                    macos_autostart::status_label()
                )),
                Err(e) => {
                    eprintln!("Failed to disable autostart: {e}");
                    macos_diag_log(format!(
                        "autostart disable failed status_after={} error={e}",
                        macos_autostart::status_label()
                    ));
                }
            }
        }
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

    let show_net_item = CheckMenuItem::with_id(
        app,
        menu_id::SHOW_NET,
        translations.show_network,
        true,
        metrics.show_net.load(Relaxed),
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
    if gpu_available {
        menu.append(&show_gpu_item)?;
    }
    menu.append(&show_net_item)?;
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
            gpu_usage: 0.0,
            down_str: "0 KB",
            up_str: "0 KB",
            show_cpu: metrics.show_cpu.load(Relaxed),
            show_mem: metrics.show_mem.load(Relaxed),
            show_gpu: metrics.show_gpu.load(Relaxed) && gpu_available,
            show_net: metrics.show_net.load(Relaxed),
            show_alerts: metrics.show_alerts.load(Relaxed),
            use_light_icons,
            background: None,
        },
    );
    let initial_icon = Image::new_owned(initial_buffer, width, height);

    let tray_builder = TrayIconBuilder::with_id(TRAY_ID).icon(initial_icon);

    #[cfg(target_os = "macos")]
    let tray_builder = tray_builder.icon_as_template(true);

    let cpu_item = show_cpu_item.clone();
    let mem_item = show_mem_item.clone();
    let gpu_item = show_gpu_item.clone();
    let net_item = show_net_item.clone();
    let autostart_menu_item = autostart_item.clone();

    let _tray = tray_builder
        .menu(&menu)
        .show_menu_on_left_click(true)
        .tooltip(translations.system_monitor)
        .on_menu_event(move |app, event| {
            let flags = [
                metrics.show_cpu.load(Relaxed),
                metrics.show_mem.load(Relaxed),
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
                menu_id::SHOW_GPU => {
                    toggle_setting(app, menu_id::SHOW_GPU, &metrics.show_gpu, flags, &gpu_item)
                }
                menu_id::SHOW_NET => {
                    toggle_setting(app, menu_id::SHOW_NET, &metrics.show_net, flags, &net_item)
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
    macos_diag_log("tray build ok");

    Ok(())
}

fn setup_initial_tray(
    app: &AppHandle,
    metrics: MetricToggles,
    gpu_available: bool,
    autostart: bool,
    translations: &'static i18n::Translations,
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
                setup_tray(
                    app,
                    &font,
                    metrics.clone(),
                    gpu_available,
                    autostart,
                    translations,
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

fn handle_second_instance_launch(app: &AppHandle, metrics: MetricToggles, gpu_available: bool) {
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

        let (_, _, _, _, _, stored_autostart) = load_settings(app);
        let autostart =
            should_repair_macos_autostart(stored_autostart, macos_autostart::is_enabled());
        macos_diag_log(format!(
            "second_instance stored_autostart={stored_autostart} effective_autostart={autostart} status_before={}",
            macos_autostart::status_label()
        ));
        if autostart {
            match macos_autostart::repair() {
                Ok(()) => macos_diag_log(format!(
                    "second_instance autostart repair ok status_after={}",
                    macos_autostart::status_label()
                )),
                Err(e) => {
                    eprintln!("Failed to repair autostart registration on second launch: {e}");
                    macos_diag_log(format!(
                        "second_instance autostart repair failed status_after={} error={e}",
                        macos_autostart::status_label()
                    ));
                }
            }
        }

        if let Some(tray) = app.tray_by_id(TRAY_ID) {
            if let Err(e) = tray.set_visible(true) {
                eprintln!("Failed to show tray icon on second launch: {e}");
                macos_diag_log(format!("second_instance tray show failed error={e}"));
            } else {
                macos_diag_log("second_instance tray show ok");
            }
        } else {
            macos_diag_log("second_instance tray missing; rebuilding");
            let (cpu, mem, gpu, net, alerts, _) = load_settings(app);
            let (cpu, mem, gpu, net) = normalize_metric_flags(cpu, mem, gpu, net, gpu_available);
            metrics.show_cpu.store(cpu, Relaxed);
            metrics.show_mem.store(mem, Relaxed);
            metrics.show_gpu.store(gpu, Relaxed);
            metrics.show_net.store(net, Relaxed);
            metrics.show_alerts.store(alerts, Relaxed);

            let translations = i18n::detect_language().translations();
            if let Err(e) = setup_initial_tray(app, metrics, gpu_available, autostart, translations)
            {
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
        let mut prev_gpu: f32 = -100.0;
        let mut prev_down_speed: f64 = -1.0;
        let mut prev_up_speed: f64 = -1.0;
        let mut prev_flags: (bool, bool, bool, bool, bool, bool) =
            (false, false, false, false, false, false);
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
            let full_tick = tick_count % 2 == 0;
            tick_count = tick_count.wrapping_add(1);

            let sc = metrics.show_cpu.load(Relaxed);
            let sm = metrics.show_mem.load(Relaxed);
            let show_gpu_enabled = metrics.show_gpu.load(Relaxed);
            let sg = show_gpu_enabled && gpu_sampler.is_some();
            let sn = metrics.show_net.load(Relaxed);
            let sa = metrics.show_alerts.load(Relaxed);

            #[cfg(target_os = "linux")]
            let current_flags = (sc, sm, sg, sn, sa, detect_light_icons());
            #[cfg(not(target_os = "linux"))]
            let current_flags = (sc, sm, sg, sn, sa, false);

            let flags_changed = prev_flags != current_flags;
            let net_was_enabled = prev_flags.3;

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
            let gpu_changed = should_update(prev_gpu, gpu_usage, HYSTERESIS_THRESHOLD);
            let down_diff = (down_speed - prev_down_speed).abs();
            let up_diff = (up_speed - prev_up_speed).abs();
            let net_value_changed =
                down_diff >= NET_HYSTERESIS_BPS || up_diff >= NET_HYSTERESIS_BPS;
            let net_changed = sn && net_value_changed;

            if cpu_changed || mem_changed || gpu_changed || net_changed || flags_changed {
                // Defer string formatting to render time only
                let down_str = format_speed(down_speed);
                let up_str = format_speed(up_speed);

                if sc {
                    prev_cpu = cpu_usage;
                }
                if sm {
                    prev_mem = mem_percent;
                }
                if sg {
                    prev_gpu = gpu_usage;
                }
                if sn {
                    prev_down_speed = down_speed;
                    prev_up_speed = up_speed;
                }
                prev_flags = current_flags;

                let (width, height, _has_active_alert) = renderer.render_tray_icon_into(
                    &font,
                    &mut render_buffer,
                    &tray_render::RenderConfig {
                        sizing: APP_SIZING,
                        cpu_usage,
                        mem_percent,
                        gpu_usage,
                        down_str: &down_str,
                        up_str: &up_str,
                        show_cpu: sc,
                        show_mem: sm,
                        show_gpu: sg,
                        show_net: sn,
                        show_alerts: sa,
                        use_light_icons: current_flags.5,
                        background: None,
                    },
                );

                if let Some(tray) = app.tray_by_id(TRAY_ID) {
                    #[cfg(target_os = "macos")]
                    {
                        let use_template = !_has_active_alert;
                        let icon = tray_icon::Icon::from_rgba(render_buffer.clone(), width, height)
                            .expect("Failed to create icon");
                        let _ = tray.with_inner_tray_icon(move |inner| {
                            inner.set_icon_with_as_template(Some(icon), use_template)
                        });
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
    #[cfg(target_os = "linux")]
    if let Err(e) = ensure_display_available() {
        eprintln!("{e}");
        std::process::exit(1);
    }

    let metrics = MetricToggles {
        show_cpu: Arc::new(AtomicBool::new(true)),
        show_mem: Arc::new(AtomicBool::new(true)),
        show_gpu: Arc::new(AtomicBool::new(true)),
        show_net: Arc::new(AtomicBool::new(true)),
        show_alerts: Arc::new(AtomicBool::new(true)),
    };
    let tray_metrics = metrics.clone();
    let second_instance_metrics = metrics.clone();

    let gpu_sampler = GpuSampler::new();
    let gpu_available = gpu_sampler.is_some();

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(
            move |app, _args, _cwd| {
                handle_second_instance_launch(app, second_instance_metrics.clone(), gpu_available);
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

            let (cpu, mem, gpu, net, alerts, stored_autostart) = load_settings(app.handle());
            let (cpu, mem, gpu, net) = normalize_metric_flags(cpu, mem, gpu, net, gpu_available);
            #[cfg(target_os = "macos")]
            let autostart =
                should_repair_macos_autostart(stored_autostart, macos_autostart::is_enabled());
            #[cfg(not(target_os = "macos"))]
            let autostart = stored_autostart;
            #[cfg(target_os = "macos")]
            macos_diag_log(format!(
                "settings loaded stored_autostart={stored_autostart} effective_autostart={autostart} metrics cpu={cpu} mem={mem} gpu={gpu} net={net} alerts={alerts} gpu_available={gpu_available}"
            ));
            tray_metrics.show_cpu.store(cpu, Relaxed);
            tray_metrics.show_mem.store(mem, Relaxed);
            tray_metrics.show_gpu.store(gpu, Relaxed);
            tray_metrics.show_net.store(net, Relaxed);
            tray_metrics.show_alerts.store(alerts, Relaxed);

            let translations = i18n::detect_language().translations();

            let font = setup_initial_tray(
                app.handle(),
                tray_metrics,
                gpu_available,
                autostart,
                translations,
            )?;

            start_monitoring(app.handle().clone(), font, metrics, gpu_sampler);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
