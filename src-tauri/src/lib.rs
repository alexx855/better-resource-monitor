use sysinfo::{Networks, System};
use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    AppHandle,
};

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[cfg(desktop)]
use tauri_plugin_autostart::MacosLauncher;

/// Format bytes per second into a human-readable string
fn format_speed(bytes_per_sec: f64) -> String {
    if bytes_per_sec >= 1_000_000.0 {
        format!("{:>4.1} MB/s", bytes_per_sec / 1_000_000.0)
    } else if bytes_per_sec >= 1_000.0 {
        format!("{:>4.0} KB/s", bytes_per_sec / 1_000.0)
    } else {
        format!("{:>4.0}  B/s", bytes_per_sec)
    }
}

/// Create the tray title string with fixed-width formatting
fn create_tray_title(cpu: f32, mem: f32, down: f64, up: f64) -> String {
    format!(
        "CPU:{:>3.0}%  MEM:{:>3.0}%  ↓{}  ↑{}",
        cpu,
        mem,
        format_speed(down),
        format_speed(up)
    )
}

/// Set up the system tray with menu
fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(desktop)]
    let autostart_manager = {
        use tauri_plugin_autostart::ManagerExt;
        app.autolaunch()
    };

    // Check initial autostart state
    #[cfg(desktop)]
    let is_autostart_enabled = autostart_manager.is_enabled().unwrap_or(false);
    #[cfg(not(desktop))]
    let is_autostart_enabled = false;

    // Create menu items
    let autostart_item = CheckMenuItem::with_id(
        app,
        "autostart",
        "Start at Login",
        true,
        is_autostart_enabled,
        None::<&str>,
    )?;

    let separator = PredefinedMenuItem::separator(app)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    // Build the menu
    let menu = Menu::with_items(app, &[&autostart_item, &separator, &quit_item])?;

    // Build the tray icon with ID so we can reference it later
    let _tray = TrayIconBuilder::with_id("main")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .title("CPU:  0%  MEM:  0%  ↓   0  B/s  ↑   0  B/s")
        .tooltip("SiliconMonitor")
        .on_menu_event(move |app, event| {
            match event.id.as_ref() {
                "autostart" => {
                    #[cfg(desktop)]
                    {
                        use tauri_plugin_autostart::ManagerExt;
                        let manager = app.autolaunch();
                        let is_enabled = manager.is_enabled().unwrap_or(false);
                        
                        if is_enabled {
                            let _ = manager.disable();
                        } else {
                            let _ = manager.enable();
                        }
                    }
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .build(app)?;

    Ok(())
}

/// Start the monitoring thread that updates the tray title
fn start_monitoring(app: AppHandle, running: Arc<AtomicBool>) {
    thread::spawn(move || {
        let mut sys = System::new();
        let mut networks = Networks::new_with_refreshed_list();
        
        // Track previous network bytes for speed calculation
        let mut prev_rx: u64 = 0;
        let mut prev_tx: u64 = 0;
        let mut first_run = true;

        while running.load(Ordering::SeqCst) {
            // Refresh CPU info
            sys.refresh_cpu_usage();
            
            // Wait a bit for accurate CPU measurement
            thread::sleep(Duration::from_millis(200));
            sys.refresh_cpu_usage();
            
            // Refresh memory and network
            sys.refresh_memory();
            networks.refresh();

            // Calculate CPU usage (global average)
            let cpu_usage = sys.global_cpu_usage();

            // Calculate memory usage percentage
            let used_mem = sys.used_memory() as f64;
            let total_mem = sys.total_memory() as f64;
            let mem_percent = if total_mem > 0.0 {
                (used_mem / total_mem * 100.0) as f32
            } else {
                0.0
            };

            // Calculate network speeds
            let mut total_rx: u64 = 0;
            let mut total_tx: u64 = 0;
            
            for (_interface_name, data) in networks.iter() {
                total_rx += data.total_received();
                total_tx += data.total_transmitted();
            }

            // Calculate bytes per second (skip first run since we need delta)
            let (down_speed, up_speed) = if first_run {
                prev_rx = total_rx;
                prev_tx = total_tx;
                first_run = false;
                (0.0, 0.0)
            } else {
                let rx_delta = total_rx.saturating_sub(prev_rx) as f64;
                let tx_delta = total_tx.saturating_sub(prev_tx) as f64;
                prev_rx = total_rx;
                prev_tx = total_tx;
                // Adjust for our ~1 second interval (with 200ms CPU wait)
                (rx_delta, tx_delta)
            };

            // Create and set the tray title
            let title = create_tray_title(cpu_usage, mem_percent, down_speed, up_speed);
            
            // Update tray title
            if let Some(tray) = app.tray_by_id("main") {
                let _ = tray.set_title(Some(&title));
            }

            // Sleep for the remainder of the second
            thread::sleep(Duration::from_millis(800));
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(move |app| {
            // Initialize autostart plugin
            #[cfg(desktop)]
            {
                app.handle().plugin(tauri_plugin_autostart::init(
                    MacosLauncher::LaunchAgent,
                    None,
                ))?;
            }

            // Set up the tray
            setup_tray(app.handle())?;

            // Start the monitoring thread
            start_monitoring(app.handle().clone(), running_clone.clone());

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    // Signal the monitoring thread to stop
    running.store(false, Ordering::SeqCst);
}
