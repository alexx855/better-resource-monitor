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


const FONT_DATA: &[u8] = include_bytes!("../fonts/Inter-Medium.ttf");


const SEGMENT_WIDTH_CPU: u32 = 110;  // "CPU" + value
const SEGMENT_WIDTH_MEM: u32 = 114; // "MEM" is wider
const SEGMENT_WIDTH_GPU: u32 = 108;  // "GPU" + value
const SEGMENT_WIDTH_NET: u32 = 102;  // Arrow + network speed
const EDGE_PADDING: u32 = 8;
const SEPARATOR_GAP: u32 = 10;
const SEPARATOR_LINE: u32 = 2;
const ICON_HEIGHT: u32 = 32;
const FONT_SIZE: f32 = 26.0;
const LABEL_VALUE_GAP: u32 = 6;


fn format_speed(bytes_per_sec: f64) -> String {
    const THRESHOLD_KB: f64 = 99_500.0;
    const THRESHOLD_MB: f64 = 99_500_000.0;
    const THRESHOLD_GB: f64 = 99_500_000_000.0;
    
    let (value, unit) = if bytes_per_sec >= THRESHOLD_GB {
        let val = bytes_per_sec / 1_000_000_000_000.0;
        if val > 9.9 {
             (9.9, "TB")
        } else {
             (val, "TB")
        }
    } else if bytes_per_sec >= THRESHOLD_MB {
        (bytes_per_sec / 1_000_000_000.0, "GB")
    } else if bytes_per_sec >= THRESHOLD_KB {
        (bytes_per_sec / 1_000_000.0, "MB")
    } else {
        (bytes_per_sec / 1_000.0, "KB")
    };

    let value_str = if value < 10.0 {
        format!("{:.1}", value)
    } else {
        format!("{:.0}", value.round().min(99.0))
    };

    format!("{} {}", value_str, unit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_speed_strict() {
        // KB range (0 - 99)
        assert_eq!(format_speed(0.0),        "0.0 KB");
        assert_eq!(format_speed(500.0),      "0.5 KB");
        assert_eq!(format_speed(1_500.0),    "1.5 KB");
        assert_eq!(format_speed(10_000.0),   "10 KB");
        assert_eq!(format_speed(99_000.0),   "99 KB");
        assert_eq!(format_speed(99_400.0),   "99 KB"); // Rounds down

        // Endpoint: 99.5 KB -> 0.1 MB
        assert_eq!(format_speed(99_500.0),   "0.1 MB");
        
        // MB range
        assert_eq!(format_speed(100_000.0),  "0.1 MB");
        assert_eq!(format_speed(1_500_000.0), "1.5 MB");
        assert_eq!(format_speed(10_000_000.0), "10 MB");
        
        // Endpoint: 99.5 MB -> 0.1 GB
        assert_eq!(format_speed(99_500_000.0), "0.1 GB");

        // GB range
        assert_eq!(format_speed(100_000_000.0), "0.1 GB");
        assert_eq!(format_speed(1_500_000_000.0), "1.5 GB");
        assert_eq!(format_speed(10_000_000_000.0), "10 GB");

        // TB range and Cap
        // 99.5 GB -> 0.1 TB
        assert_eq!(format_speed(99_500_000_000.0), "0.1 TB");
        assert_eq!(format_speed(1_000_000_000_000.0), "1.0 TB");
        assert_eq!(format_speed(9_900_000_000_000.0), "9.9 TB");
        assert_eq!(format_speed(15_000_000_000_000.0), "9.9 TB"); // Cap
    }
}

fn cap_percent(value: f32) -> f32 {
    if value > 99.0 { 99.0 } else { value }
}

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
    struct Segment {
        label: &'static str,
        value: String,
        width: u32,
    }

    let mut segments: Vec<Segment> = Vec::new();

    if show_cpu {
        segments.push(Segment {
            label: "CPU",
            value: format!("{:.0}%", cap_percent(cpu_usage)),
            width: SEGMENT_WIDTH_CPU,
        });
    }
    if show_mem {
        segments.push(Segment {
            label: "MEM",
            value: format!("{:.0}%", cap_percent(mem_percent)),
            width: SEGMENT_WIDTH_MEM,
        });
    }
    if show_gpu {
        segments.push(Segment {
            label: "GPU",
            value: format!("{:.0}%", cap_percent(gpu_usage)),
            width: SEGMENT_WIDTH_GPU,
        });
    }
    if show_net {
        segments.push(Segment {
            label: "↓",
            value: format_speed(down_speed),
            width: SEGMENT_WIDTH_NET,
        });
        segments.push(Segment {
            label: "↑",
            value: format_speed(up_speed),
            width: SEGMENT_WIDTH_NET,
        });
    }

    let separator_total = SEPARATOR_GAP * 2 + SEPARATOR_LINE;
    let segment_widths: u32 = segments.iter().map(|s| s.width).sum();
    let total_width = if segments.is_empty() {
        50 // Minimum width
    } else {
        EDGE_PADDING
            + segment_widths
            + separator_total * (segments.len() as u32).saturating_sub(1)
            + EDGE_PADDING
    };

    let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(total_width, ICON_HEIGHT);

    for pixel in img.pixels_mut() {
        *pixel = Rgba([0, 0, 0, 0]);
    }

    let scale = Scale::uniform(FONT_SIZE);
    let v_metrics = font.v_metrics(scale);
    let baseline = (ICON_HEIGHT as f32 / 2.0) + (v_metrics.ascent / 2.0) - 2.0;

    let measure_text = |text: &str| -> f32 {
        font.layout(text, scale, rusttype::point(0.0, 0.0))
            .map(|g| g.unpositioned().h_metrics().advance_width)
            .sum()
    };

    let draw_text = |text: &str, start_x: f32, img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>| {
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

    let mut x_offset = EDGE_PADDING;
    for (i, segment) in segments.iter().enumerate() {
        if i > 0 {
            x_offset += SEPARATOR_GAP;
            let sep_x = x_offset;
            for y in (ICON_HEIGHT / 4)..(ICON_HEIGHT * 3 / 4) {
                for dx in 0..SEPARATOR_LINE {
                    if sep_x + dx < total_width {
                        img.put_pixel(sep_x + dx, y, Rgba([0, 0, 0, 200]));
                    }
                }
            }
            x_offset += SEPARATOR_LINE + SEPARATOR_GAP;
        }

        let label_width = measure_text(&segment.label);
        draw_text(&segment.label, x_offset as f32, &mut img);
        
        let value_width = measure_text(&segment.value);
        let segment_end = x_offset as f32 + segment.width as f32;
        let right_aligned_x = segment_end - value_width;
        let min_gap_x = x_offset as f32 + label_width + LABEL_VALUE_GAP as f32;
        let value_x = right_aligned_x.max(min_gap_x);
        draw_text(&segment.value, value_x, &mut img);

        x_offset += segment.width;
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
