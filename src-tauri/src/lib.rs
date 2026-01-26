mod gpu;

use sysinfo::{Networks, System};
use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    AppHandle,
    Manager,
    image::Image,
};
use tauri_plugin_store::StoreExt;
use serde_json::json;

#[cfg(target_os = "macos")]
use tauri::ActivationPolicy;
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
use std::sync::Arc;

#[cfg(target_os = "linux")]
use std::sync::OnceLock;

#[cfg(target_os = "linux")]
static DETECTED_LIGHT_ICONS: OnceLock<bool> = OnceLock::new();

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
fn detect_light_icons() -> bool {
    *DETECTED_LIGHT_ICONS.get_or_init(|| {
        // Try gsettings (GNOME/GTK)
        if let Ok(output) = std::process::Command::new("gsettings")
            .args(["get", "org.gnome.desktop.interface", "gtk-application-prefer-dark-theme"])
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
    })
}


use std::thread;
use std::time::Duration;
use std::path::PathBuf;
use std::fs;
use image::{ImageBuffer, Rgba};
use rusttype::{Font, Scale};
use font_kit::source::SystemSource;
use font_kit::family_name::FamilyName;
use font_kit::properties::{Properties, Weight};
use font_kit::handle::Handle;

use gpu::GpuSampler;

#[cfg(desktop)]
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};

const SETTINGS_FILE: &str = "settings.json";

mod menu_id {
    pub const AUTOSTART: &str = "autostart";
    pub const SHOW_CPU: &str = "show_cpu";
    pub const SHOW_MEM: &str = "show_mem";
    pub const SHOW_GPU: &str = "show_gpu";
    pub const SHOW_NET: &str = "show_net";
    pub const LIGHT_ICONS: &str = "light_icons";
    pub const QUIT: &str = "quit";
}

const TRAY_ID: &str = "main";

fn load_settings(app: &AppHandle) -> (bool, bool, bool, bool, bool) {
    let store = match app.store(SETTINGS_FILE) {
        Ok(s) => Some(s),
        Err(e) => {
            eprintln!("Failed to load settings store, using defaults: {e}");
            None
        }
    };

    let get_bool = |key: &str| -> bool {
        store.as_ref()
            .and_then(|s| s.get(key))
            .and_then(|v| v.as_bool())
            .unwrap_or(true)
    };

    (
        get_bool("show_cpu"),
        get_bool("show_mem"),
        get_bool("show_gpu"),
        get_bool("show_net"),
        get_bool("dark_mode"),
    )
}

fn save_setting(app: &AppHandle, key: &str, value: bool) {
    if let Ok(store) = app.store(SETTINGS_FILE) {
        store.set(key, json!(value));
        let _ = store.save();
    }
}

const SVG_CPU: &str = include_str!("../assets/icons/svg/fill/cpu-fill.svg");
const SVG_MEMORY: &str = include_str!("../assets/icons/svg/fill/memory-fill.svg");
const SVG_GPU: &str = include_str!("../assets/icons/svg/fill/graphics-card-fill.svg");
const SVG_ARROW_UP: &str = include_str!("../assets/icons/svg/fill/cloud-arrow-up-fill.svg");
const SVG_ARROW_DOWN: &str = include_str!("../assets/icons/svg/fill/cloud-arrow-down-fill.svg");

const ALERT_THRESHOLD: f32 = 90.0;
const ALERT_COLOR_DARK: (u8, u8, u8) = (255, 149, 0);
const ALERT_COLOR_LIGHT: (u8, u8, u8) = (191, 54, 12);

fn get_alert_color(is_dark: bool) -> (u8, u8, u8) {
    if is_dark { ALERT_COLOR_DARK } else { ALERT_COLOR_LIGHT }
}

fn get_text_color(is_dark: bool) -> u8 {
    if is_dark { 255 } else { 0 }
}

fn cap_percent(value: f32) -> f32 {
    value.clamp(0.0, 99.0)
}

fn load_system_font() -> Font<'static> {
    let source = SystemSource::new();

    let handle = source.select_best_match(
        &[FamilyName::SansSerif],
        Properties::new().weight(Weight::NORMAL)
    ).or_else(|_| {
        source.select_best_match(&[FamilyName::SansSerif], &Properties::new())
    }).expect("Failed to select a system font");

    let font_data = match &handle {
        Handle::Path { path, .. } => std::fs::read(path).expect("Failed to read font file"),
        Handle::Memory { bytes, .. } => bytes.to_vec(),
    };

    Font::try_from_vec(font_data).expect("Error constructing font")
}

fn render_svg_icon(svg_data: &str, size: u32, color: (u8, u8, u8)) -> Vec<u8> {
    let color_hex = format!("#{:02x}{:02x}{:02x}", color.0, color.1, color.2);

    let svg_with_color = svg_data
        .replace("currentColor", &color_hex)
        .replace("<svg ", &format!("<svg fill=\"{color_hex}\" "));
 
    let opts = resvg::usvg::Options::default();
    let tree = resvg::usvg::Tree::from_str(&svg_with_color, &opts)
        .expect("Failed to parse SVG");

    let svg_size = tree.size();
    let scale = size as f32 / svg_size.width().max(svg_size.height());

    let scaled_width = (svg_size.width() * scale).ceil() as u32;
    let scaled_height = (svg_size.height() * scale).ceil() as u32;

    let mut pixmap = resvg::tiny_skia::Pixmap::new(scaled_width, scaled_height)
        .expect("Failed to create pixmap");

    let transform = resvg::tiny_skia::Transform::from_scale(scale, scale);
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    pixmap.take()
}


#[cfg(target_os = "macos")]
mod sizing {
    pub const SEGMENT_WIDTH: u32 = 180;
    pub const SEGMENT_WIDTH_NET: u32 = 240;
    pub const EDGE_PADDING: u32 = 16;
    pub const SEGMENT_GAP: u32 = 48;
    pub const ICON_HEIGHT: u32 = 64;
    pub const FONT_SIZE: f32 = 56.0;
}

#[cfg(target_os = "linux")]
mod sizing {
    // Tuned for 22px GNOME tray height
    pub const SEGMENT_WIDTH: u32 = 58;
    pub const SEGMENT_WIDTH_NET: u32 = 75;
    pub const EDGE_PADDING: u32 = 5;
    pub const SEGMENT_GAP: u32 = 18;
    pub const ICON_HEIGHT: u32 = 22;
    pub const FONT_SIZE: f32 = 19.0;
}

fn format_speed(bytes_per_sec: f64) -> String {
    // Switch units at 9.95 to keep values in 0.0-9.9 range (max 2 digits)
    // GB is the final unit, capped at 9.9 GB
    const THRESHOLD_KB: f64 = 9_950.0;
    const THRESHOLD_MB: f64 = 9_950_000.0;

    let (value, unit) = if bytes_per_sec >= THRESHOLD_MB {
        ((bytes_per_sec / 1_000_000_000.0).min(9.9), "GB")
    } else if bytes_per_sec >= THRESHOLD_KB {
        (bytes_per_sec / 1_000_000.0, "MB")
    } else {
        (bytes_per_sec / 1_000.0, "KB")
    };

    format!("{value:.1} {unit}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cap_percent() {
        assert_eq!(cap_percent(0.0), 0.0);
        assert_eq!(cap_percent(50.0), 50.0);
        assert_eq!(cap_percent(99.0), 99.0);
        assert_eq!(cap_percent(100.0), 99.0);
        assert_eq!(cap_percent(150.0), 99.0);
        // Also test negative values are clamped to 0
        assert_eq!(cap_percent(-10.0), 0.0);
    }

    #[test]
    fn test_get_alert_color() {
        // Dark mode should return ALERT_COLOR_DARK (orange)
        assert_eq!(get_alert_color(true), (255, 149, 0));
        // Light mode should return ALERT_COLOR_LIGHT (red-orange)
        assert_eq!(get_alert_color(false), (191, 54, 12));
    }

    #[test]
    fn test_get_text_color() {
        // Dark mode should return 255 (white text)
        assert_eq!(get_text_color(true), 255);
        // Light mode should return 0 (black text)
        assert_eq!(get_text_color(false), 0);
    }

    #[test]
    fn test_format_speed_strict() {
        // KB range (0.0 - 9.9)
        assert_eq!(format_speed(0.0),        "0.0 KB");
        assert_eq!(format_speed(500.0),      "0.5 KB");
        assert_eq!(format_speed(1_500.0),    "1.5 KB");
        assert_eq!(format_speed(9_000.0),    "9.0 KB");
        assert_eq!(format_speed(9_900.0),    "9.9 KB");

        // Endpoint: 9.95 KB -> 0.0 MB
        assert_eq!(format_speed(9_950.0),    "0.0 MB");

        // MB range (0.0 - 9.9)
        assert_eq!(format_speed(100_000.0),  "0.1 MB");
        assert_eq!(format_speed(1_500_000.0), "1.5 MB");
        assert_eq!(format_speed(9_900_000.0), "9.9 MB");

        // Endpoint: 9.95 MB -> 0.0 GB
        assert_eq!(format_speed(9_950_000.0), "0.0 GB");

        // GB range (0.0 - 9.9, capped)
        assert_eq!(format_speed(100_000_000.0), "0.1 GB");
        assert_eq!(format_speed(1_500_000_000.0), "1.5 GB");
        assert_eq!(format_speed(9_900_000_000.0), "9.9 GB");
        assert_eq!(format_speed(50_000_000_000.0), "9.9 GB"); // Cap
    }

    #[test]
    fn test_format_speed_edge_cases() {
        // Extremely small numbers - should show as 0.0 KB
        assert_eq!(format_speed(1e-10), "0.0 KB");
        assert_eq!(format_speed(0.001), "0.0 KB");
        assert_eq!(format_speed(0.5), "0.0 KB");

        // Terabyte values - function caps at 9.9 GB
        assert_eq!(format_speed(1_000_000_000_000.0), "9.9 GB"); // 1 TB
        assert_eq!(format_speed(5_000_000_000_000.0), "9.9 GB"); // 5 TB
        assert_eq!(format_speed(1e15), "9.9 GB"); // 1 PB - still caps at 9.9 GB

        // Negative values - handled by division (becomes negative KB)
        assert_eq!(format_speed(-100.0), "-0.1 KB");
    }
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
    is_dark_mode: bool,
) -> (Vec<u8>, u32, u32) {
    enum SegmentLabel {
        IconCpu,
        IconMem,
        IconGpu,
        IconDown,
        IconUp,
    }

    struct Segment {
        label: SegmentLabel,
        value: String,
        width: u32,
        alert: bool,
    }

    let cpu_alert = cpu_usage >= ALERT_THRESHOLD;
    let mem_alert = mem_percent >= ALERT_THRESHOLD;
    let gpu_alert = gpu_usage >= ALERT_THRESHOLD;

    let mut segments: Vec<Segment> = Vec::new();

    if show_cpu {
        segments.push(Segment {
            label: SegmentLabel::IconCpu,
            value: format!("{:.0}%", cap_percent(cpu_usage)),
            width: sizing::SEGMENT_WIDTH,
            alert: cpu_alert,
        });
    }
    if show_mem {
        segments.push(Segment {
            label: SegmentLabel::IconMem,
            value: format!("{:.0}%", cap_percent(mem_percent)),
            width: sizing::SEGMENT_WIDTH,
            alert: mem_alert,
        });
    }
    if show_gpu {
        segments.push(Segment {
            label: SegmentLabel::IconGpu,
            value: format!("{:.0}%", cap_percent(gpu_usage)),
            width: sizing::SEGMENT_WIDTH,
            alert: gpu_alert,
        });
    }
    if show_net {
        segments.push(Segment {
            label: SegmentLabel::IconDown,
            value: format_speed(down_speed),
            width: sizing::SEGMENT_WIDTH_NET,
            alert: false,
        });
        segments.push(Segment {
            label: SegmentLabel::IconUp,
            value: format_speed(up_speed),
            width: sizing::SEGMENT_WIDTH_NET,
            alert: false,
        });
    }

    let segment_widths: u32 = segments.iter().map(|s| s.width).sum();
    let total_width = sizing::EDGE_PADDING
        + segment_widths
        + sizing::SEGMENT_GAP * (segments.len() as u32).saturating_sub(1)
        + sizing::EDGE_PADDING;

    let icon_height = sizing::ICON_HEIGHT;
    let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(total_width, icon_height);

    for pixel in img.pixels_mut() {
        *pixel = Rgba([0, 0, 0, 0]);
    }

    let scale = Scale::uniform(sizing::FONT_SIZE);
    let v_metrics = font.v_metrics(scale);

    let reference_text = "0123456789% KMGTP";
    let mut min_y = i32::MAX;
    let mut max_y = i32::MIN;

    for glyph in font.layout(reference_text, scale, rusttype::point(0.0, 0.0)) {
        if let Some(bb) = glyph.pixel_bounding_box() {
            if bb.min.y < min_y { min_y = bb.min.y; }
            if bb.max.y > max_y { max_y = bb.max.y; }
        }
    }

    let baseline = if min_y < max_y {
        (icon_height as f32 / 2.0) - ((min_y + max_y) as f32 / 2.0)
    } else {
        (icon_height as f32 / 2.0) + (v_metrics.ascent / 2.0)
    };

    let measure_text = |text: &str| -> f32 {
        font.layout(text, scale, rusttype::point(0.0, 0.0))
            .map(|g| g.unpositioned().h_metrics().advance_width)
            .sum()
    };

    let base_color_val = get_text_color(is_dark_mode);
    let base_color: (u8, u8, u8) = (base_color_val, base_color_val, base_color_val);

    let draw_text = |text: &str, start_x: f32, color: (u8, u8, u8), img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>| {
        for glyph in font.layout(text, scale, rusttype::point(start_x, baseline)) {
            if let Some(bb) = glyph.pixel_bounding_box() {
                glyph.draw(|gx, gy, v| {
                    let x = (bb.min.x + gx as i32) as u32;
                    let y = (bb.min.y + gy as i32) as u32;
                    if x < total_width && y < icon_height {
                        let alpha = (v * 255.0) as u8;
                        img.put_pixel(x, y, Rgba([color.0, color.1, color.2, alpha]));
                    }
                });
            }
        }
    };

    let draw_svg_icon = |svg_data: &str, start_x: u32, color: (u8, u8, u8), img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>| {
        let icon_pixels = render_svg_icon(svg_data, icon_height, color);

        for y in 0..icon_height {
            for x in 0..icon_height {
                let src_idx = ((y * icon_height + x) * 4) as usize;
                if src_idx + 3 < icon_pixels.len() {
                    let alpha = icon_pixels[src_idx + 3];
                    if alpha > 0 {
                        let dst_x = start_x + x;
                        if dst_x < total_width && y < icon_height {
                            img.put_pixel(dst_x, y, Rgba([
                                icon_pixels[src_idx],
                                icon_pixels[src_idx + 1],
                                icon_pixels[src_idx + 2],
                                alpha,
                            ]));
                        }
                    }
                }
            }
        }
    };

    let mut x_offset = sizing::EDGE_PADDING;
    for (i, segment) in segments.iter().enumerate() {
        if i > 0 {
            x_offset += sizing::SEGMENT_GAP;
        }

        let segment_color = if segment.alert { get_alert_color(is_dark_mode) } else { base_color };

        match segment.label {
            SegmentLabel::IconCpu => draw_svg_icon(SVG_CPU, x_offset, segment_color, &mut img),
            SegmentLabel::IconMem => draw_svg_icon(SVG_MEMORY, x_offset, segment_color, &mut img),
            SegmentLabel::IconGpu => draw_svg_icon(SVG_GPU, x_offset, segment_color, &mut img),
            SegmentLabel::IconDown => draw_svg_icon(SVG_ARROW_DOWN, x_offset, segment_color, &mut img),
            SegmentLabel::IconUp => draw_svg_icon(SVG_ARROW_UP, x_offset, segment_color, &mut img),
        }

        let value_width = measure_text(&segment.value);
        let segment_end = x_offset as f32 + segment.width as f32;
        let value_x = segment_end - value_width;
        draw_text(&segment.value, value_x, segment_color, &mut img);

        x_offset += segment.width;
    }

    (img.into_raw(), total_width, icon_height)
}

fn toggle_setting(
    app: &AppHandle,
    key: &str,
    flag: &AtomicBool,
    all_flags: [&AtomicBool; 4],
) {
    let enabled_count = all_flags.iter().filter(|v| v.load(Relaxed)).count();
    if !flag.load(Relaxed) || enabled_count > 1 {
        flag.fetch_xor(true, Relaxed);
        save_setting(app, key, flag.load(Relaxed));
    }
}

fn setup_tray(
    app: &AppHandle,
    show_cpu: Arc<AtomicBool>,
    show_mem: Arc<AtomicBool>,
    show_gpu: Arc<AtomicBool>,
    show_net: Arc<AtomicBool>,
    dark_mode: Arc<AtomicBool>,
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
        } else {
            autostart_manager.is_enabled().unwrap_or(false)
        }
    };
    #[cfg(not(desktop))]
    let is_autostart_enabled = false;

    let autostart_item = CheckMenuItem::with_id(
        app, menu_id::AUTOSTART, "Start at Login", true, is_autostart_enabled, None::<&str>,
    )?;

    let separator1 = PredefinedMenuItem::separator(app)?;

    let show_cpu_item = CheckMenuItem::with_id(
        app, menu_id::SHOW_CPU, "Show CPU", true, show_cpu.load(Relaxed), None::<&str>,
    )?;

    let show_mem_item = CheckMenuItem::with_id(
        app, menu_id::SHOW_MEM, "Show Memory", true, show_mem.load(Relaxed), None::<&str>,
    )?;

    let show_net_item = CheckMenuItem::with_id(
        app, menu_id::SHOW_NET, "Show Network", true, show_net.load(Relaxed), None::<&str>,
    )?;

    #[cfg(target_os = "macos")]
    let separator2 = PredefinedMenuItem::separator(app)?;

    #[cfg(target_os = "macos")]
    let light_icons_item = CheckMenuItem::with_id(
        app, menu_id::LIGHT_ICONS, "Use Light Icons", true, dark_mode.load(Relaxed), None::<&str>,
    )?;

    let separator3 = PredefinedMenuItem::separator(app)?;
    let quit_item = MenuItem::with_id(app, menu_id::QUIT, "Quit", true, None::<&str>)?;

    let menu = Menu::new(app)?;
    menu.append(&autostart_item)?;
    menu.append(&separator1)?;
    menu.append(&show_cpu_item)?;
    menu.append(&show_mem_item)?;
    if gpu_available {
        let show_gpu_item = CheckMenuItem::with_id(
            app, menu_id::SHOW_GPU, "Show GPU", true, show_gpu.load(Relaxed), None::<&str>,
        )?;
        menu.append(&show_gpu_item)?;
    }
    menu.append(&show_net_item)?;
    #[cfg(target_os = "macos")]
    {
        menu.append(&separator2)?;
        menu.append(&light_icons_item)?;
    }
    menu.append(&separator3)?;
    menu.append(&quit_item)?;

    let font = load_system_font();

    #[cfg(target_os = "macos")]
    let is_light_icons = dark_mode.load(Relaxed);
    #[cfg(target_os = "linux")]
    let is_light_icons = detect_light_icons();

    let (pixels, width, height) = render_tray_icon(
        &font,
        0.0, 0.0, 0.0, 0.0, 0.0,
        show_cpu.load(Relaxed),
        show_mem.load(Relaxed),
        show_gpu.load(Relaxed) && gpu_available,
        show_net.load(Relaxed),
        is_light_icons,
    );
    let initial_icon = Image::new_owned(pixels, width, height);

    let tray_builder = TrayIconBuilder::with_id(TRAY_ID).icon(initial_icon);

    #[cfg(target_os = "macos")]
    let tray_builder = tray_builder.icon_as_template(false);

    let _tray = tray_builder
        .menu(&menu)
        .show_menu_on_left_click(true)
        .tooltip("System Monitor")
        .on_menu_event(move |app, event| {
            let flags = [show_cpu.as_ref(), show_mem.as_ref(), show_gpu.as_ref(), show_net.as_ref()];
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
                menu_id::LIGHT_ICONS => {
                    dark_mode.fetch_xor(true, Relaxed);
                    save_setting(app, "dark_mode", dark_mode.load(Relaxed));
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
    show_cpu: Arc<AtomicBool>,
    show_mem: Arc<AtomicBool>,
    show_gpu: Arc<AtomicBool>,
    show_net: Arc<AtomicBool>,
    dark_mode: Arc<AtomicBool>,
    gpu_available: bool,
) {
    thread::spawn(move || {
        // On Linux, dark_mode is detected via system theme, not passed in
        #[cfg(target_os = "linux")]
        let _ = &dark_mode;
        let font = load_system_font();

        let mut sys = System::new();
        let mut networks = Networks::new_with_refreshed_list();
        let mut gpu_sampler = if gpu_available { GpuSampler::new() } else { None };

        let mut prev_rx: u64 = 0;
        let mut prev_tx: u64 = 0;
        let mut first_run = true;
        let mut prev_display: Option<String> = None;
        let mut gpu_usage: f32 = 0.0;

        loop {
            // Double refresh needed on first run for accurate CPU readings
            if first_run {
                sys.refresh_cpu_usage();
                thread::sleep(Duration::from_millis(200));
            }
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
                first_run = false;
                prev_rx = total_rx;
                prev_tx = total_tx;
                (0.0, 0.0)
            } else {
                let rx_delta = total_rx.saturating_sub(prev_rx) as f64;
                let tx_delta = total_tx.saturating_sub(prev_tx) as f64;
                prev_rx = total_rx;
                prev_tx = total_tx;
                (rx_delta, tx_delta)
            };

            if let Some(ref mut sampler) = gpu_sampler {
                if let Some(usage) = sampler.sample() {
                    gpu_usage = usage;
                }
            }

            let sc = show_cpu.load(Relaxed);
            let sm = show_mem.load(Relaxed);
            let sg = show_gpu.load(Relaxed) && gpu_available;
            let sn = show_net.load(Relaxed);

            #[cfg(target_os = "macos")]
            let dm = dark_mode.load(Relaxed);
            #[cfg(target_os = "linux")]
            let dm = detect_light_icons();

            let display_key = format!(
                "{:.0}|{:.0}|{:.0}|{}|{}|{}{}{}{}{}",
                cpu_usage, mem_percent, gpu_usage,
                format_speed(down_speed), format_speed(up_speed),
                sc, sm, sg, sn, dm
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
                    sc, sm, sg, sn, dm,
                );

                if let Some(tray) = app.tray_by_id(TRAY_ID) {
                    let icon = Image::new_owned(pixels, width, height);
                    if let Err(_e) = tray.set_icon(Some(icon)) {
                        #[cfg(debug_assertions)]
                        eprintln!("Failed to set tray icon: {_e:?}");
                    }
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
    let dark_mode = Arc::new(AtomicBool::new(true));

    let show_cpu_tray = show_cpu.clone();
    let show_mem_tray = show_mem.clone();
    let show_gpu_tray = show_gpu.clone();
    let show_net_tray = show_net.clone();
    let dark_mode_tray = dark_mode.clone();

    let gpu_available = GpuSampler::new().is_some();

    #[cfg(target_os = "linux")]
    if let Err(e) = ensure_display_available() {
        eprintln!("{e}");
        std::process::exit(1);
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|_app, _args, _cwd| {}))
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

            // Load persisted settings
            let (cpu, mem, gpu, net, dark) = load_settings(app.handle());
            show_cpu_tray.store(cpu, Relaxed);
            show_mem_tray.store(mem, Relaxed);
            show_gpu_tray.store(gpu, Relaxed);
            show_net_tray.store(net, Relaxed);
            dark_mode_tray.store(dark, Relaxed);

            setup_tray(
                app.handle(),
                show_cpu_tray,
                show_mem_tray,
                show_gpu_tray,
                show_net_tray,
                dark_mode_tray,
                gpu_available,
            )?;

            start_monitoring(
                app.handle().clone(),
                show_cpu,
                show_mem,
                show_gpu,
                show_net,
                dark_mode,
                gpu_available,
            );

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
