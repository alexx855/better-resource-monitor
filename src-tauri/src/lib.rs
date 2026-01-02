use sysinfo::{Networks, System};
use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    ActivationPolicy,
    AppHandle,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[cfg(desktop)]
use tauri_plugin_autostart::MacosLauncher;

/// Format bytes per second - compact format
fn format_speed(bytes_per_sec: f64) -> String {
    if bytes_per_sec >= 1_000_000_000.0 {
        format!("{:.0}G", bytes_per_sec / 1_000_000_000.0)
    } else if bytes_per_sec >= 1_000_000.0 {
        format!("{:.0}M", bytes_per_sec / 1_000_000.0)
    } else if bytes_per_sec >= 1_000.0 {
        format!("{:.0}K", bytes_per_sec / 1_000.0)
    } else {
        format!("{:.0}B", bytes_per_sec)
    }
}


/// Format the tray title text
fn format_tray_title(
    cpu_usage: f32,
    mem_percent: f32,
    down_speed: f64,
    up_speed: f64,
    show_cpu: bool,
    show_mem: bool,
    show_net_down: bool,
    show_net_up: bool,
) -> String {
    let mut parts = Vec::new();
    
    if show_cpu {
        // Fixed width: icon + space + 3-char value
        parts.push(format!("◎ {:>3.0}%", cpu_usage));
    }
    if show_mem {
        parts.push(format!("▣ {:>3.0}%", mem_percent));
    }
    if show_net_down {
        parts.push(format!("▼ {:>5}", format_speed(down_speed)));
    }
    if show_net_up {
        parts.push(format!("▲ {:>5}", format_speed(up_speed)));
    }
    
    // Use vertical bar as separator for consistent visual spacing
    parts.join("  │  ")
}

fn setup_tray(
    app: &AppHandle,
    show_cpu: Arc<AtomicBool>,
    show_mem: Arc<AtomicBool>,
    show_net_down: Arc<AtomicBool>,
    show_net_up: Arc<AtomicBool>,
) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(desktop)]
    let autostart_manager = {
        use tauri_plugin_autostart::ManagerExt;
        app.autolaunch()
    };

    #[cfg(desktop)]
    let is_autostart_enabled = autostart_manager.is_enabled().unwrap_or(false);
    #[cfg(not(desktop))]
    let is_autostart_enabled = false;

    let autostart_item = CheckMenuItem::with_id(
        app, "autostart", "Start at Login", true, is_autostart_enabled, None::<&str>,
    )?;

    let separator1 = PredefinedMenuItem::separator(app)?;

    let show_cpu_item = CheckMenuItem::with_id(
        app, "show_cpu", "Show CPU", true, show_cpu.load(Ordering::SeqCst), None::<&str>,
    )?;

    let show_mem_item = CheckMenuItem::with_id(
        app, "show_mem", "Show Memory", true, show_mem.load(Ordering::SeqCst), None::<&str>,
    )?;

    let show_net_down_item = CheckMenuItem::with_id(
        app, "show_net_down", "Show Download", true, show_net_down.load(Ordering::SeqCst), None::<&str>,
    )?;

    let show_net_up_item = CheckMenuItem::with_id(
        app, "show_net_up", "Show Upload", true, show_net_up.load(Ordering::SeqCst), None::<&str>,
    )?;

    let separator2 = PredefinedMenuItem::separator(app)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[
            &autostart_item,
            &separator1,
            &show_cpu_item,
            &show_mem_item,
            &show_net_down_item,
            &show_net_up_item,
            &separator2,
            &quit_item,
        ],
    )?;

    let _tray = TrayIconBuilder::with_id("main")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .title("")
        .tooltip("System Monitor")
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
                "show_cpu" => {
                    let current = show_cpu.load(Ordering::SeqCst);
                    show_cpu.store(!current, Ordering::SeqCst);
                }
                "show_mem" => {
                    let current = show_mem.load(Ordering::SeqCst);
                    show_mem.store(!current, Ordering::SeqCst);
                }
                "show_net_down" => {
                    let current = show_net_down.load(Ordering::SeqCst);
                    show_net_down.store(!current, Ordering::SeqCst);
                }
                "show_net_up" => {
                    let current = show_net_up.load(Ordering::SeqCst);
                    show_net_up.store(!current, Ordering::SeqCst);
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

fn start_monitoring(
    app: AppHandle,
    running: Arc<AtomicBool>,
    show_cpu: Arc<AtomicBool>,
    show_mem: Arc<AtomicBool>,
    show_net_down: Arc<AtomicBool>,
    show_net_up: Arc<AtomicBool>,
) {
    thread::spawn(move || {
        let mut sys = System::new();
        let mut networks = Networks::new_with_refreshed_list();
        
        let mut prev_rx: u64 = 0;
        let mut prev_tx: u64 = 0;
        let mut first_run = true;

        while running.load(Ordering::SeqCst) {
            sys.refresh_cpu_usage();
            thread::sleep(Duration::from_millis(200));
            sys.refresh_cpu_usage();
            
            sys.refresh_memory();
            networks.refresh(true);

            let cpu_usage = sys.global_cpu_usage();

            let used_mem = sys.used_memory() as f64;
            let total_mem = sys.total_memory() as f64;
            let mem_percent = if total_mem > 0.0 {
                (used_mem / total_mem * 100.0) as f32
            } else {
                0.0
            };

            let mut total_rx: u64 = 0;
            let mut total_tx: u64 = 0;
            
            for (_interface_name, data) in networks.iter() {
                total_rx += data.total_received();
                total_tx += data.total_transmitted();
            }

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
                (rx_delta, tx_delta)
            };

            let title = format_tray_title(
                cpu_usage,
                mem_percent,
                down_speed,
                up_speed,
                show_cpu.load(Ordering::SeqCst),
                show_mem.load(Ordering::SeqCst),
                show_net_down.load(Ordering::SeqCst),
                show_net_up.load(Ordering::SeqCst),
            );
            
            if let Some(tray) = app.tray_by_id("main") {
                let _ = tray.set_title(Some(&title));
            }

            thread::sleep(Duration::from_millis(800));
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    let show_cpu = Arc::new(AtomicBool::new(true));
    let show_mem = Arc::new(AtomicBool::new(true));
    let show_net_down = Arc::new(AtomicBool::new(true));
    let show_net_up = Arc::new(AtomicBool::new(true));

    let show_cpu_tray = show_cpu.clone();
    let show_mem_tray = show_mem.clone();
    let show_net_down_tray = show_net_down.clone();
    let show_net_up_tray = show_net_up.clone();

    let show_cpu_mon = show_cpu.clone();
    let show_mem_mon = show_mem.clone();
    let show_net_down_mon = show_net_down.clone();
    let show_net_up_mon = show_net_up.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
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

            setup_tray(
                app.handle(),
                show_cpu_tray.clone(),
                show_mem_tray.clone(),
                show_net_down_tray.clone(),
                show_net_up_tray.clone(),
            )?;

            start_monitoring(
                app.handle().clone(),
                running_clone.clone(),
                show_cpu_mon.clone(),
                show_mem_mon.clone(),
                show_net_down_mon.clone(),
                show_net_up_mon.clone(),
            );

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    running.store(false, Ordering::SeqCst);
}
