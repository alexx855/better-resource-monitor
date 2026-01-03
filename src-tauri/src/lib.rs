mod gpu;

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

use gpu::GpuSampler;

#[cfg(desktop)]
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};

/// Font data embedded at compile time
const FONT_DATA: &[u8] = include_bytes!("../fonts/Inter-Medium.ttf");

/// Fixed widths for each segment (in pixels at 2x scale for Retina)
const SEGMENT_CPU: u32 = 95;
const SEGMENT_MEM: u32 = 100;
const SEGMENT_GPU: u32 = 95;
const SEGMENT_NET: u32 = 160;
const EDGE_PADDING: u32 = 8;
const SEPARATOR_GAP: u32 = 10;
const SEPARATOR_LINE: u32 = 2;
const ICON_HEIGHT: u32 = 32;
const FONT_SIZE: f32 = 24.0;
const LABEL_VALUE_GAP: u32 = 6; // Minimum gap between label and value

/// Format bytes per second - compact format (max 4 chars like "999M")
fn format_speed(bytes_per_sec: f64) -> String {
    if bytes_per_sec >= 1_000_000_000.0 {
        let val = (bytes_per_sec / 1_000_000_000.0).min(99.0);
        format!("{:.0}G", val)
    } else if bytes_per_sec >= 1_000_000.0 {
        format!("{:.0}M", bytes_per_sec / 1_000_000.0)
    } else if bytes_per_sec >= 1_000.0 {
        format!("{:.0}K", bytes_per_sec / 1_000.0)
    } else {
        format!("{:.0}B", bytes_per_sec)
    }
}

/// Cap percentage at 99% when > 99% (indicates "maxed out")
fn cap_percent(value: f32) -> f32 {
    if value > 99.0 { 99.0 } else { value }
}

/// Render tray icon as a fixed-width template image (black with alpha)
fn render_tray_icon(
    font: &Font,
    cpu_usage: f32,
    mem_percent: f32,
    gpu_usage: f32,
    down_speed: f64,
    up_speed: f64,
    show_cpu: bool,
    show_mem: bool,
    show_gpu: bool,
    show_net: bool,
) -> (Vec<u8>, u32, u32) {
    // Segment types: Regular (label + value) or Network (two label+value pairs)
    enum Segment {
        Regular { label: &'static str, value: String, width: u32 },
        Network { down_label: &'static str, down_value: String, up_label: &'static str, up_value: String, width: u32 },
    }

    // Calculate total width based on enabled segments
    let mut segments: Vec<Segment> = Vec::new();

    if show_cpu {
        segments.push(Segment::Regular {
            label: "CPU",
            value: format!("{:.0}%", cap_percent(cpu_usage)),
            width: SEGMENT_CPU,
        });
    }
    if show_mem {
        segments.push(Segment::Regular {
            label: "MEM",
            value: format!("{:.0}%", cap_percent(mem_percent)),
            width: SEGMENT_MEM,
        });
    }
    if show_gpu {
        segments.push(Segment::Regular {
            label: "GPU",
            value: format!("{:.0}%", cap_percent(gpu_usage)),
            width: SEGMENT_GPU,
        });
    }
    if show_net {
        segments.push(Segment::Network {
            down_label: "↓",
            down_value: format_speed(down_speed),
            up_label: "↑",
            up_value: format_speed(up_speed),
            width: SEGMENT_NET,
        });
    }

    // Calculate total width with symmetric edge padding and separator gaps
    let separator_total = SEPARATOR_GAP * 2 + SEPARATOR_LINE;
    let segment_widths: u32 = segments.iter().map(|s| match s {
        Segment::Regular { width, .. } => *width,
        Segment::Network { width, .. } => *width,
    }).sum();
    let total_width = if segments.is_empty() {
        50 // Minimum width
    } else {
        EDGE_PADDING
            + segment_widths
            + separator_total * (segments.len() as u32).saturating_sub(1)
            + EDGE_PADDING
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

    // Helper to measure text width
    let measure_text = |text: &str| -> f32 {
        let glyphs: Vec<_> = font.layout(text, scale, rusttype::point(0.0, 0.0)).collect();
        glyphs.last()
            .and_then(|g| g.pixel_bounding_box())
            .map(|bb| bb.max.x as f32)
            .unwrap_or(0.0)
    };

    // Helper to draw text at a position
    let mut draw_text = |text: &str, start_x: f32, img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>| {
        for glyph in font.layout(text, scale, rusttype::point(start_x, baseline)) {
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
    };

    // Draw each segment at fixed positions
    let mut x_offset = EDGE_PADDING;
    for (i, segment) in segments.iter().enumerate() {
        let width = match segment {
            Segment::Regular { width, .. } => *width,
            Segment::Network { width, .. } => *width,
        };

        // Draw separator before segment (except first)
        if i > 0 {
            x_offset += SEPARATOR_GAP;
            let sep_x = x_offset;
            // Draw separator line
            for y in (ICON_HEIGHT / 4)..(ICON_HEIGHT * 3 / 4) {
                for dx in 0..SEPARATOR_LINE {
                    if sep_x + dx < total_width {
                        img.put_pixel(sep_x + dx, y, Rgba([0, 0, 0, 200]));
                    }
                }
            }
            x_offset += SEPARATOR_LINE + SEPARATOR_GAP;
        }

        match segment {
            Segment::Regular { label, value, width: seg_width } => {
                // Label at left, value right-aligned (but respecting min gap from label)
                draw_text(label, x_offset as f32, &mut img);
                let label_width = measure_text(label);
                let value_width = measure_text(value);
                let min_value_x = x_offset as f32 + label_width + LABEL_VALUE_GAP as f32;
                let right_aligned_x = x_offset as f32 + *seg_width as f32 - value_width;
                let value_x = right_aligned_x.max(min_value_x);
                draw_text(value, value_x, &mut img);
            }
            Segment::Network { down_label, down_value, up_label, up_value, width: seg_width } => {
                // Split segment in half for download and upload
                let half_width = seg_width / 2;
                
                // Download: label at left, value right-aligned (with min gap)
                draw_text(down_label, x_offset as f32, &mut img);
                let down_label_width = measure_text(down_label);
                let down_value_width = measure_text(down_value);
                let down_min_x = x_offset as f32 + down_label_width + LABEL_VALUE_GAP as f32;
                let down_right_x = x_offset as f32 + half_width as f32 - down_value_width - LABEL_VALUE_GAP as f32;
                let down_value_x = down_right_x.max(down_min_x);
                draw_text(down_value, down_value_x, &mut img);
                
                // Upload: label at left of second half, value right-aligned (with min gap)
                let up_start = x_offset + half_width;
                draw_text(up_label, up_start as f32, &mut img);
                let up_label_width = measure_text(up_label);
                let up_value_width = measure_text(up_value);
                let up_min_x = up_start as f32 + up_label_width + LABEL_VALUE_GAP as f32;
                let up_right_x = up_start as f32 + half_width as f32 - up_value_width;
                let up_value_x = up_right_x.max(up_min_x);
                draw_text(up_value, up_value_x, &mut img);
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
    show_gpu: Arc<AtomicBool>,
    show_net: Arc<AtomicBool>,
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

    let show_gpu_item = CheckMenuItem::with_id(
        app, "show_gpu", "Show GPU", true, show_gpu.load(Relaxed), None::<&str>,
    )?;

    let show_net_item = CheckMenuItem::with_id(
        app, "show_net", "Show Network", true, show_net.load(Relaxed), None::<&str>,
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
            &show_gpu_item,
            &show_net_item,
            &separator2,
            &quit_item,
        ],
    )?;

    // Render initial icon as template
    let font = Font::try_from_bytes(FONT_DATA).expect("Failed to load font");
    let (pixels, width, height) = render_tray_icon(
        &font,
        0.0, 0.0, 0.0, 0.0, 0.0,
        show_cpu.load(Relaxed),
        show_mem.load(Relaxed),
        show_gpu.load(Relaxed),
        show_net.load(Relaxed),
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
                "show_gpu" => {
                    let current = show_gpu.load(Relaxed);
                    show_gpu.store(!current, Relaxed);
                }
                "show_net" => {
                    let current = show_net.load(Relaxed);
                    show_net.store(!current, Relaxed);
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
    show_gpu: Arc<AtomicBool>,
    show_net: Arc<AtomicBool>,
) {
    thread::spawn(move || {
        // Load font once at thread start
        let font = Font::try_from_bytes(FONT_DATA).expect("Failed to load font");

        let mut sys = System::new();
        let mut networks = Networks::new_with_refreshed_list();
        let mut gpu_sampler = GpuSampler::new();

        let mut prev_rx: u64 = 0;
        let mut prev_tx: u64 = 0;
        let mut first_run = true;
        let mut prev_display: Option<String> = None;
        let mut gpu_usage: f32 = 0.0;

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

            // Sample GPU usage
            if let Some(ref mut sampler) = gpu_sampler {
                gpu_usage = sampler.sample();
            }

            let sc = show_cpu.load(Relaxed);
            let sm = show_mem.load(Relaxed);
            let sg = show_gpu.load(Relaxed);
            let sn = show_net.load(Relaxed);

            let display_key = format!(
                "{:.0}|{:.0}|{:.0}|{}|{}|{}{}{}{}",
                cpu_usage, mem_percent, gpu_usage,
                format_speed(down_speed), format_speed(up_speed),
                sc, sm, sg, sn
            );

            // Only update icon if display changed
            if prev_display.as_ref() != Some(&display_key) {
                prev_display = Some(display_key);

                let (pixels, width, height) = render_tray_icon(
                    &font,
                    cpu_usage,
                    mem_percent,
                    gpu_usage,
                    down_speed,
                    up_speed,
                    sc, sm, sg, sn,
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
    let show_gpu = Arc::new(AtomicBool::new(true));
    let show_net = Arc::new(AtomicBool::new(true));

    let show_cpu_tray = show_cpu.clone();
    let show_mem_tray = show_mem.clone();
    let show_gpu_tray = show_gpu.clone();
    let show_net_tray = show_net.clone();

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
                show_gpu_tray.clone(),
                show_net_tray.clone(),
            )?;

            start_monitoring(
                app.handle().clone(),
                show_cpu,
                show_mem,
                show_gpu,
                show_net,
            );

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
