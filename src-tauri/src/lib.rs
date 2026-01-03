use sysinfo::{Networks, System};
use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    ActivationPolicy,
    AppHandle,
    image::Image,
};
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use image::{ImageBuffer, Rgba};
use rusttype::{Font, Scale};

#[cfg(desktop)]
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};

/// Font data embedded at compile time
const FONT_DATA: &[u8] = include_bytes!("../fonts/Inter-Medium.ttf");

/// Fixed widths for each segment (in pixels at 2x scale for Retina)
const SEGMENT_CPU: u32 = 105;
const SEGMENT_MEM: u32 = 110;
const SEGMENT_NET: u32 = 100;
const SEPARATOR_WIDTH: u32 = 24;
const ICON_HEIGHT: u32 = 32;
const FONT_SIZE: f32 = 24.0;

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

/// Render tray icon as a fixed-width template image (black with alpha)
fn render_tray_icon(
    font: &Font,
    cpu_usage: f32,
    mem_percent: f32,
    down_speed: f64,
    up_speed: f64,
    show_cpu: bool,
    show_mem: bool,
    show_net_down: bool,
    show_net_up: bool,
) -> (Vec<u8>, u32, u32) {
    // Calculate total width based on enabled segments
    let mut segments: Vec<(String, u32)> = Vec::new();

    if show_cpu {
        segments.push((format!("CPU {:>3.0}%", cpu_usage), SEGMENT_CPU));
    }
    if show_mem {
        segments.push((format!("MEM {:>3.0}%", mem_percent), SEGMENT_MEM));
    }
    if show_net_down {
        segments.push((format!("D {:>5}", format_speed(down_speed)), SEGMENT_NET));
    }
    if show_net_up {
        segments.push((format!("U {:>5}", format_speed(up_speed)), SEGMENT_NET));
    }

    // Calculate total width
    let total_width = if segments.is_empty() {
        50 // Minimum width
    } else {
        segments.iter().map(|(_, w)| w).sum::<u32>()
            + SEPARATOR_WIDTH * (segments.len() as u32).saturating_sub(1)
    };

    // Create image buffer with transparent background
    let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(total_width, ICON_HEIGHT);

    // Fill with transparent
    for pixel in img.pixels_mut() {
        *pixel = Rgba([0, 0, 0, 0]);
    }

    let scale = Scale::uniform(FONT_SIZE);
    let v_metrics = font.v_metrics(scale);
    let baseline = (ICON_HEIGHT as f32 / 2.0) + (v_metrics.ascent / 2.0) - 2.0;

    // Draw each segment at fixed positions
    let mut x_offset = 0u32;
    for (i, (text, width)) in segments.iter().enumerate() {
        // Draw separator before segment (except first)
        if i > 0 {
            let sep_x = x_offset + SEPARATOR_WIDTH / 2;
            // Draw a thicker vertical line as separator (2px wide)
            for y in (ICON_HEIGHT / 4)..(ICON_HEIGHT * 3 / 4) {
                for dx in 0..2 {
                    let x = sep_x + dx;
                    if x < total_width {
                        img.put_pixel(x, y, Rgba([0, 0, 0, 200]));
                    }
                }
            }
            x_offset += SEPARATOR_WIDTH;
        }

        // Left-align text with padding
        let text_start_x = x_offset as f32 + 8.0;

        for glyph in font.layout(&text, scale, rusttype::point(text_start_x, baseline)) {
            if let Some(bb) = glyph.pixel_bounding_box() {
                glyph.draw(|gx, gy, v| {
                    let x = (bb.min.x + gx as i32) as u32;
                    let y = (bb.min.y + gy as i32) as u32;
                    if x < total_width && y < ICON_HEIGHT {
                        let alpha = (v * 255.0) as u8;
                        img.put_pixel(x, y, Rgba([0, 0, 0, alpha]));
                    }
                });
            }
        }

        x_offset += width;
    }

    (img.into_raw(), total_width, ICON_HEIGHT)
}

fn setup_tray(
    app: &AppHandle,
    show_cpu: Arc<AtomicBool>,
    show_mem: Arc<AtomicBool>,
    show_net_down: Arc<AtomicBool>,
    show_net_up: Arc<AtomicBool>,
) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(desktop)]
    let autostart_manager = app.autolaunch();

    #[cfg(desktop)]
    let is_autostart_enabled = autostart_manager.is_enabled().unwrap_or(false);
    #[cfg(not(desktop))]
    let is_autostart_enabled = false;

    let autostart_item = CheckMenuItem::with_id(
        app, "autostart", "Start at Login", true, is_autostart_enabled, None::<&str>,
    )?;

    let separator1 = PredefinedMenuItem::separator(app)?;

    let show_cpu_item = CheckMenuItem::with_id(
        app, "show_cpu", "Show CPU", true, show_cpu.load(Relaxed), None::<&str>,
    )?;

    let show_mem_item = CheckMenuItem::with_id(
        app, "show_mem", "Show Memory", true, show_mem.load(Relaxed), None::<&str>,
    )?;

    let show_net_down_item = CheckMenuItem::with_id(
        app, "show_net_down", "Show Download", true, show_net_down.load(Relaxed), None::<&str>,
    )?;

    let show_net_up_item = CheckMenuItem::with_id(
        app, "show_net_up", "Show Upload", true, show_net_up.load(Relaxed), None::<&str>,
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

    // Render initial icon as template
    let font = Font::try_from_bytes(FONT_DATA).expect("Failed to load font");
    let (pixels, width, height) = render_tray_icon(
        &font,
        0.0, 0.0, 0.0, 0.0,
        show_cpu.load(Relaxed),
        show_mem.load(Relaxed),
        show_net_down.load(Relaxed),
        show_net_up.load(Relaxed),
    );
    let initial_icon = Image::new_owned(pixels, width, height);

    let _tray = TrayIconBuilder::with_id("main")
        .icon(initial_icon)
        .icon_as_template(true)
        .menu(&menu)
        .show_menu_on_left_click(true)
        .tooltip("System Monitor")
        .on_menu_event(move |app, event| {
            match event.id.as_ref() {
                "autostart" => {
                    #[cfg(desktop)]
                    {
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
                    let current = show_cpu.load(Relaxed);
                    show_cpu.store(!current, Relaxed);
                }
                "show_mem" => {
                    let current = show_mem.load(Relaxed);
                    show_mem.store(!current, Relaxed);
                }
                "show_net_down" => {
                    let current = show_net_down.load(Relaxed);
                    show_net_down.store(!current, Relaxed);
                }
                "show_net_up" => {
                    let current = show_net_up.load(Relaxed);
                    show_net_up.store(!current, Relaxed);
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
    show_cpu: Arc<AtomicBool>,
    show_mem: Arc<AtomicBool>,
    show_net_down: Arc<AtomicBool>,
    show_net_up: Arc<AtomicBool>,
) {
    thread::spawn(move || {
        // Load font once at thread start
        let font = Font::try_from_bytes(FONT_DATA).expect("Failed to load font");

        let mut sys = System::new();
        let mut networks = Networks::new_with_refreshed_list();

        let mut prev_rx: u64 = 0;
        let mut prev_tx: u64 = 0;
        let mut first_run = true;
        let mut prev_display: Option<String> = None;

        loop {
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

            let sc = show_cpu.load(Relaxed);
            let sm = show_mem.load(Relaxed);
            let sd = show_net_down.load(Relaxed);
            let su = show_net_up.load(Relaxed);

            // Build display key for comparison (rounded values to reduce flickering)
            let display_key = format!(
                "{:.0}|{:.0}|{}|{}|{}{}{}{}",
                cpu_usage, mem_percent,
                format_speed(down_speed), format_speed(up_speed),
                sc, sm, sd, su
            );

            // Only update icon if display changed
            if prev_display.as_ref() != Some(&display_key) {
                prev_display = Some(display_key);

                let (pixels, width, height) = render_tray_icon(
                    &font,
                    cpu_usage,
                    mem_percent,
                    down_speed,
                    up_speed,
                    sc, sm, sd, su,
                );

                if let Some(tray) = app.tray_by_id("main") {
                    let _ = tray.with_inner_tray_icon(move |inner| {
                        let icon = tray_icon::Icon::from_rgba(pixels, width, height).ok();
                        inner.set_icon_with_as_template(icon, true)
                    });
                }
            }

            thread::sleep(Duration::from_millis(800));
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let show_cpu = Arc::new(AtomicBool::new(true));
    let show_mem = Arc::new(AtomicBool::new(true));
    let show_net_down = Arc::new(AtomicBool::new(true));
    let show_net_up = Arc::new(AtomicBool::new(true));

    let show_cpu_tray = show_cpu.clone();
    let show_mem_tray = show_mem.clone();
    let show_net_down_tray = show_net_down.clone();
    let show_net_up_tray = show_net_up.clone();

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
                show_cpu,
                show_mem,
                show_net_down,
                show_net_up,
            );

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
