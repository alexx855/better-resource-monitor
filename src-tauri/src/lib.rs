mod gpu;
pub mod tray_render;

// std
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

// external crates
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
    AppHandle, Manager,
};
use tauri_plugin_store::StoreExt;

#[cfg(target_os = "macos")]
use tauri::ActivationPolicy;

#[cfg(desktop)]
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};

// internal
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

fn load_settings(app: &AppHandle) -> (bool, bool, bool, bool, bool) {
    let store = match app.store(SETTINGS_FILE) {
        Ok(s) => Some(s),
        Err(e) => {
            eprintln!("Failed to load settings store: {e}");
            None
        }
    };

    let get_bool = |key: &str| -> bool {
        store
            .as_ref()
            .and_then(|s| s.get(key))
            .and_then(|v| v.as_bool())
            .unwrap_or(true)
    };

    (
        get_bool("show_cpu"),
        get_bool("show_mem"),
        get_bool("show_gpu"),
        get_bool("show_net"),
        get_bool("show_alerts"),
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

const UPDATE_INTERVAL_MS: u64 = 800;
const CPU_STABILIZE_MS: u64 = 200;

/// Minimum change threshold to trigger icon update (prevents compositor leak on Linux)
const HYSTERESIS_THRESHOLD: f32 = 2.0;

/// Minimum network speed change (bytes/sec) to trigger an update.
/// Reduces tray icon churn that can accumulate compositor resources on Linux.
const NET_HYSTERESIS_BPS: f64 = 50_000.0;

/// Minimum interval between network-driven updates.
const NET_MIN_UPDATE_INTERVAL_SECS: u64 = 2;

/// Returns true if the new value differs from previous by at least the threshold
fn should_update(prev: f32, new: f32, threshold: f32) -> bool {
    (new - prev).abs() >= threshold
}

/// Get update interval from environment variable or use default.
/// Set SILICON_UPDATE_INTERVAL=2000 to reduce icon updates (helps debug compositor leaks).
fn get_update_interval_ms() -> u64 {
    std::env::var("SILICON_UPDATE_INTERVAL")
        .ok()
        .and_then(|s| s.parse().ok())
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

// Rendering is centralized in tray_render.rs

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

#[cfg(test)]
mod tests;

// render_tray_icon_into moved to tray_render.rs

fn toggle_setting(app: &AppHandle, key: &str, flag: &AtomicBool, all_flags: [&AtomicBool; 4]) {
    let current = flag.load(Relaxed);
    let enabled_count = all_flags.iter().filter(|v| v.load(Relaxed)).count();
    if !current || enabled_count > 1 {
        flag.store(!current, Relaxed);
        save_setting(app, key, !current);
    }
}

fn setup_tray(
    app: &AppHandle,
    font: &Font,
    show_cpu: Arc<AtomicBool>,
    show_mem: Arc<AtomicBool>,
    show_gpu: Arc<AtomicBool>,
    show_net: Arc<AtomicBool>,
    show_alerts: Arc<AtomicBool>,
    gpu_available: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(desktop)]
    let autostart_manager = app.autolaunch();

    #[cfg(desktop)]
    let is_autostart_enabled = {
        let marker_path: PathBuf = app
            .path()
            .app_data_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(".autostart_configured");

        if !marker_path.exists() {
            let _ = autostart_manager.enable();
            if let Some(parent) = marker_path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let _ = fs::write(&marker_path, "1");
            true
        } else if autostart_manager.is_enabled().unwrap_or(false) {
            // Re-register to repair macOS Login Items desync
            let _ = autostart_manager.enable();
            true
        } else {
            false
        }
    };
    #[cfg(not(desktop))]
    let is_autostart_enabled = false;

    let autostart_item = CheckMenuItem::with_id(
        app,
        menu_id::AUTOSTART,
        "Start at Login",
        true,
        is_autostart_enabled,
        None::<&str>,
    )?;

    let separator1 = PredefinedMenuItem::separator(app)?;

    let show_mem_item = CheckMenuItem::with_id(
        app,
        menu_id::SHOW_MEM,
        "Show Memory",
        true,
        show_mem.load(Relaxed),
        None::<&str>,
    )?;

    let show_cpu_item = CheckMenuItem::with_id(
        app,
        menu_id::SHOW_CPU,
        "Show CPU",
        true,
        show_cpu.load(Relaxed),
        None::<&str>,
    )?;

    let show_net_item = CheckMenuItem::with_id(
        app,
        menu_id::SHOW_NET,
        "Show Network",
        true,
        show_net.load(Relaxed),
        None::<&str>,
    )?;

    let separator2 = PredefinedMenuItem::separator(app)?;

    let show_alerts_item = CheckMenuItem::with_id(
        app,
        menu_id::SHOW_ALERTS,
        "Show Alert Colors",
        true,
        show_alerts.load(Relaxed),
        None::<&str>,
    )?;

    let separator3 = PredefinedMenuItem::separator(app)?;
    let quit_item = MenuItem::with_id(app, menu_id::QUIT, "Quit", true, None::<&str>)?;

    let menu = Menu::new(app)?;
    menu.append(&autostart_item)?;
    menu.append(&separator1)?;
    menu.append(&show_mem_item)?;
    menu.append(&show_cpu_item)?;
    if gpu_available {
        let show_gpu_item = CheckMenuItem::with_id(
            app,
            menu_id::SHOW_GPU,
            "Show GPU",
            true,
            show_gpu.load(Relaxed),
            None::<&str>,
        )?;
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
        APP_SIZING,
        0.0,
        0.0,
        0.0,
        "0 KB",
        "0 KB",
        show_cpu.load(Relaxed),
        show_mem.load(Relaxed),
        show_gpu.load(Relaxed) && gpu_available,
        show_net.load(Relaxed),
        show_alerts.load(Relaxed),
        use_light_icons,
        None,
    );
    let initial_icon = Image::new_owned(initial_buffer, width, height);

    let tray_builder = TrayIconBuilder::with_id(TRAY_ID).icon(initial_icon);

    // Use template mode by default - macOS will handle light/dark adaptation
    #[cfg(target_os = "macos")]
    let tray_builder = tray_builder.icon_as_template(true);

    let _tray = tray_builder
        .menu(&menu)
        .show_menu_on_left_click(true)
        .tooltip("System Monitor")
        .on_menu_event(move |app, event| {
            let flags = [
                show_cpu.as_ref(),
                show_mem.as_ref(),
                show_gpu.as_ref(),
                show_net.as_ref(),
            ];
            match event.id.as_ref() {
                menu_id::AUTOSTART => {
                    #[cfg(desktop)]
                    {
                        let manager = app.autolaunch();
                        if manager.is_enabled().unwrap_or(false) {
                            let _ = manager.disable();
                        } else {
                            let _ = manager.enable();
                        }
                    }
                }
                menu_id::SHOW_CPU => toggle_setting(app, menu_id::SHOW_CPU, &show_cpu, flags),
                menu_id::SHOW_MEM => toggle_setting(app, menu_id::SHOW_MEM, &show_mem, flags),
                menu_id::SHOW_GPU => toggle_setting(app, menu_id::SHOW_GPU, &show_gpu, flags),
                menu_id::SHOW_NET => toggle_setting(app, menu_id::SHOW_NET, &show_net, flags),
                menu_id::SHOW_ALERTS => {
                    let new_value = !show_alerts.load(Relaxed);
                    show_alerts.store(new_value, Relaxed);
                    save_setting(app, menu_id::SHOW_ALERTS, new_value);
                }
                menu_id::QUIT => app.exit(0),
                _ => {}
            }
        })
        .build(app)?;

    Ok(())
}

fn start_monitoring(
    app: AppHandle,
    font: Font<'static>,
    show_cpu: Arc<AtomicBool>,
    show_mem: Arc<AtomicBool>,
    show_gpu: Arc<AtomicBool>,
    show_net: Arc<AtomicBool>,
    show_alerts: Arc<AtomicBool>,
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
        let mut prev_down = String::new();
        let mut prev_up = String::new();
        let mut prev_down_speed: f64 = -1.0;
        let mut prev_up_speed: f64 = -1.0;
        let mut last_net_update =
            std::time::Instant::now() - Duration::from_secs(NET_MIN_UPDATE_INTERVAL_SECS);
        let mut prev_flags: (bool, bool, bool, bool, bool, bool) =
            (false, false, false, false, false, false);
        let update_interval = get_update_interval_ms();

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
            sys.refresh_cpu_usage();
            sys.refresh_memory();
            // Only refresh counters, not the interface list
            networks.refresh(false);

            let cpu_usage = sys.global_cpu_usage();

            let used_mem = sys.used_memory() as f64;
            let total_mem = sys.total_memory() as f64;
            let mem_percent = if total_mem > 0.0 {
                (used_mem / total_mem * 100.0) as f32
            } else {
                0.0
            };

            let (total_rx, total_tx) = sum_network_totals(&networks);
            let down_speed = total_rx.saturating_sub(prev_rx) as f64 / dt;
            let up_speed = total_tx.saturating_sub(prev_tx) as f64 / dt;
            (prev_rx, prev_tx) = (total_rx, total_tx);

            let sc = show_cpu.load(Relaxed);
            let sm = show_mem.load(Relaxed);
            let show_gpu_enabled = show_gpu.load(Relaxed);
            let sg = show_gpu_enabled && gpu_sampler.is_some();
            let sn = show_net.load(Relaxed);
            let sa = show_alerts.load(Relaxed);

            if sg {
                if let Some(ref mut sampler) = gpu_sampler {
                    gpu_usage = sampler.sample().unwrap_or(0.0);
                }
            } else {
                gpu_usage = 0.0;
            }

            let down_str = format_speed(down_speed);
            let up_str = format_speed(up_speed);

            // Hysteresis: only update if values change by meaningful threshold
            // This dramatically reduces icon updates, preventing compositor resource
            // accumulation that causes cursor slowdown on Ubuntu/GNOME
            let cpu_changed = should_update(prev_cpu, cpu_usage, HYSTERESIS_THRESHOLD);
            let mem_changed = should_update(prev_mem, mem_percent, HYSTERESIS_THRESHOLD);
            let gpu_changed = should_update(prev_gpu, gpu_usage, HYSTERESIS_THRESHOLD);
            let net_display_changed = prev_down != down_str || prev_up != up_str;
            let down_diff = (down_speed - prev_down_speed).abs();
            let up_diff = (up_speed - prev_up_speed).abs();
            let net_value_changed =
                down_diff >= NET_HYSTERESIS_BPS || up_diff >= NET_HYSTERESIS_BPS;
            let net_interval_elapsed = now.duration_since(last_net_update)
                >= Duration::from_secs(NET_MIN_UPDATE_INTERVAL_SECS);
            let net_changed =
                sn && (net_value_changed || (net_display_changed && net_interval_elapsed));

            #[cfg(target_os = "linux")]
            let current_flags = (sc, sm, sg, sn, sa, detect_light_icons());
            #[cfg(not(target_os = "linux"))]
            let current_flags = (sc, sm, sg, sn, sa, false);

            let flags_changed = prev_flags != current_flags;

            if cpu_changed || mem_changed || gpu_changed || net_changed || flags_changed {
                prev_cpu = cpu_usage;
                prev_mem = mem_percent;
                prev_gpu = gpu_usage;
                prev_down = down_str.clone();
                prev_up = up_str.clone();
                prev_down_speed = down_speed;
                prev_up_speed = up_speed;
                if sn && (net_value_changed || net_display_changed) {
                    last_net_update = now;
                }
                prev_flags = current_flags;

                let (width, height, _has_active_alert) = renderer.render_tray_icon_into(
                    &font,
                    &mut render_buffer,
                    APP_SIZING,
                    cpu_usage,
                    mem_percent,
                    gpu_usage,
                    &down_str,
                    &up_str,
                    sc,
                    sm,
                    sg,
                    sn,
                    sa,
                    current_flags.5, // Pass the detected theme flag
                    None,
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

    let show_cpu = Arc::new(AtomicBool::new(true));
    let show_mem = Arc::new(AtomicBool::new(true));
    let show_gpu = Arc::new(AtomicBool::new(true));
    let show_net = Arc::new(AtomicBool::new(true));
    let show_alerts = Arc::new(AtomicBool::new(true));

    let show_cpu_tray = show_cpu.clone();
    let show_mem_tray = show_mem.clone();
    let show_gpu_tray = show_gpu.clone();
    let show_net_tray = show_net.clone();
    let show_alerts_tray = show_alerts.clone();

    let gpu_sampler = GpuSampler::new();
    let gpu_available = gpu_sampler.is_some();

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|_app, _args, _cwd| {
            // No-op: tray-only app, nothing to focus
        }))
        .plugin(tauri_plugin_store::Builder::new().build());

    builder
        .setup(move |app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(ActivationPolicy::Accessory);

            #[cfg(desktop)]
            {
                app.handle().plugin(tauri_plugin_autostart::init(
                    MacosLauncher::LaunchAgent,
                    None,
                ))?;
            }

            #[cfg(target_os = "linux")]
            start_theme_detection_thread();

            // Load persisted settings
            let (cpu, mem, gpu, net, alerts) = load_settings(app.handle());
            show_cpu_tray.store(cpu, Relaxed);
            show_mem_tray.store(mem, Relaxed);
            show_gpu_tray.store(gpu, Relaxed);
            show_net_tray.store(net, Relaxed);
            show_alerts_tray.store(alerts, Relaxed);

            let font =
                load_system_font().map_err(|e| format!("Font required for tray icon: {e}"))?;

            setup_tray(
                app.handle(),
                &font,
                show_cpu_tray,
                show_mem_tray,
                show_gpu_tray,
                show_net_tray,
                show_alerts_tray,
                gpu_available,
            )?;

            start_monitoring(
                app.handle().clone(),
                font,
                show_cpu,
                show_mem,
                show_gpu,
                show_net,
                show_alerts,
                gpu_sampler,
            );

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
