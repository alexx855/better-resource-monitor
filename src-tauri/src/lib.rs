mod gpu;

use sysinfo::{Networks, System};
use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    AppHandle,
    Manager,
    image::Image,
};

#[cfg(target_os = "macos")]
use tauri::ActivationPolicy;
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
use std::sync::Arc;

#[cfg(target_os = "linux")]
use std::sync::OnceLock;
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

// Phosphor Icons SVGs (MIT license) - embedded at compile time
const SVG_CPU: &str = include_str!("../assets/icons/svg/fill/cpu-fill.svg");
const SVG_MEMORY: &str = include_str!("../assets/icons/svg/fill/memory-fill.svg");
const SVG_GPU: &str = include_str!("../assets/icons/svg/fill/graphics-card-fill.svg");
const SVG_ARROW_UP: &str = include_str!("../assets/icons/svg/fill/cloud-arrow-up-fill.svg");
const SVG_ARROW_DOWN: &str = include_str!("../assets/icons/svg/fill/cloud-arrow-down-fill.svg");

#[cfg(target_os = "linux")]
static DETECTED_TEXT_COLOR: OnceLock<u8> = OnceLock::new();

#[cfg(target_os = "linux")]
fn get_text_color() -> u8 {
    *DETECTED_TEXT_COLOR.get_or_init(|| {
        // Try to detect via gsettings (GNOME/GTK)
        if let Ok(output) = std::process::Command::new("gsettings")
            .args(&["get", "org.gnome.desktop.interface", "gtk-application-prefer-dark-theme"])
            .output()
        {
            let result = String::from_utf8_lossy(&output.stdout);
            if result.contains("true") {
                #[cfg(debug_assertions)]
                eprintln!("[INFO] Detected dark theme, using white icons");
                return 255;
            }
        }

        // Check XDG_CURRENT_DESKTOP for common light-themed DEs
        if let Ok(desktop) = std::env::var("XDG_CURRENT_DESKTOP") {
            let lower = desktop.to_lowercase();
            if lower.contains("xfce") || lower.contains("elementary") || lower.contains("kde") {
                // These DEs often default to light themes
                #[cfg(debug_assertions)]
                eprintln!("[INFO] Detected potentially light-themed desktop ({}), using black icons", desktop);
                return 0;
            }
        }

        // Default to white for dark themes (most Linux panels are dark)
        #[cfg(debug_assertions)]
        eprintln!("[INFO] Could not detect theme, defaulting to white icons");
        255
    })
}

#[cfg(not(target_os = "linux"))]
fn get_text_color() -> u8 {
    255 // macOS uses template icons, Windows/other default to white
}

fn load_system_font() -> Font<'static> {
    let source = SystemSource::new();

    let handle = source.select_best_match(
        &[FamilyName::SansSerif],
        Properties::new().weight(Weight::MEDIUM)
    ).or_else(|_| {
        source.select_best_match(&[FamilyName::SansSerif], &Properties::new())
    }).expect("Failed to select a system font");

    let font_data = match &handle {
        Handle::Path { path, .. } => std::fs::read(path).expect("Failed to read font file"),
        Handle::Memory { bytes, .. } => bytes.to_vec(),
    };

    Font::try_from_vec(font_data).expect("Error constructing font")
}

/// Renders an SVG icon to an RGBA pixel buffer at the specified size
fn render_svg_icon(svg_data: &str, size: u32, color: u8) -> Vec<u8> {
    let color_hex = format!("#{:02x}{:02x}{:02x}", color, color, color);

    // Inject fill color into SVG - handle both currentColor and default black fills
    let svg_with_color = svg_data
        .replace("currentColor", &color_hex)
        .replace("<svg ", &format!("<svg fill=\"{}\" ", color_hex));

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


// Base sizing constants (macOS Retina 2x values)
// Linux applies a scale factor since it uses 1x displays
mod base_sizing {
    pub const SEGMENT_WIDTH: u32 = 108;      // For CPU, MEM, GPU (displays "XX%")
    pub const SEGMENT_WIDTH_NET: u32 = 118;  // For network (displays "X.X KB")
    pub const EDGE_PADDING: u32 = 8;
    pub const SEPARATOR_GAP: u32 = 10;
    pub const SEPARATOR_LINE: u32 = 2;
    pub const ICON_HEIGHT: u32 = 32;
    pub const FONT_SIZE: f32 = 26.0;
    pub const LABEL_VALUE_GAP: u32 = 6;

    pub const MIN_WIDTH: u32 = 50;
}

#[cfg(target_os = "macos")]
mod sizing {
    use super::base_sizing::*;
    pub const SEGMENT_WIDTH: u32 = base_sizing::SEGMENT_WIDTH;
    pub const SEGMENT_WIDTH_NET: u32 = base_sizing::SEGMENT_WIDTH_NET;
    pub const EDGE_PADDING: u32 = base_sizing::EDGE_PADDING;
    pub const SEPARATOR_GAP: u32 = base_sizing::SEPARATOR_GAP;
    pub const SEPARATOR_LINE: u32 = base_sizing::SEPARATOR_LINE;
    pub const ICON_HEIGHT: u32 = base_sizing::ICON_HEIGHT;
    pub const FONT_SIZE: f32 = base_sizing::FONT_SIZE;
    pub const LABEL_VALUE_GAP: u32 = base_sizing::LABEL_VALUE_GAP;

    pub const MIN_WIDTH: u32 = base_sizing::MIN_WIDTH;
}

#[cfg(target_os = "linux")]
mod sizing {
    use super::base_sizing;

    // Scale factor for Linux (1x display vs macOS 2x Retina)
    const SCALE: f32 = 0.70;

    const fn scale_u32(val: u32) -> u32 { (val as f32 * SCALE) as u32 }
    const fn scale_f32(val: f32) -> f32 { val * SCALE }

    pub const SEGMENT_WIDTH: u32 = scale_u32(base_sizing::SEGMENT_WIDTH);
    pub const SEGMENT_WIDTH_NET: u32 = scale_u32(base_sizing::SEGMENT_WIDTH_NET);
    pub const EDGE_PADDING: u32 = scale_u32(base_sizing::EDGE_PADDING);
    pub const SEPARATOR_GAP: u32 = scale_u32(base_sizing::SEPARATOR_GAP);
    pub const SEPARATOR_LINE: u32 = 1; // Minimum 1px line
    pub const ICON_HEIGHT: u32 = scale_u32(base_sizing::ICON_HEIGHT);
    pub const FONT_SIZE: f32 = scale_f32(base_sizing::FONT_SIZE);
    pub const LABEL_VALUE_GAP: u32 = scale_u32(base_sizing::LABEL_VALUE_GAP);

    pub const MIN_WIDTH: u32 = scale_u32(base_sizing::MIN_WIDTH);
}

use sizing::*;


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
        format!("{value:.1}")
    } else {
        format!("{:.0}", value.round().min(99.0))
    };

    format!("{value_str} {unit}")
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
    }

    let mut segments: Vec<Segment> = Vec::new();

    if show_cpu {
        segments.push(Segment {
            label: SegmentLabel::IconCpu,
            value: format!("{:.0}%", cap_percent(cpu_usage)),
            width: SEGMENT_WIDTH,
        });
    }
    if show_mem {
        segments.push(Segment {
            label: SegmentLabel::IconMem,
            value: format!("{:.0}%", cap_percent(mem_percent)),
            width: SEGMENT_WIDTH,
        });
    }
    if show_gpu {
        segments.push(Segment {
            label: SegmentLabel::IconGpu,
            value: format!("{:.0}%", cap_percent(gpu_usage)),
            width: SEGMENT_WIDTH,
        });
    }
    if show_net {
        segments.push(Segment {
            label: SegmentLabel::IconDown,
            value: format_speed(down_speed),
            width: SEGMENT_WIDTH_NET,
        });
        segments.push(Segment {
            label: SegmentLabel::IconUp,
            value: format_speed(up_speed),
            width: SEGMENT_WIDTH_NET,
        });
    }

    let separator_total = SEPARATOR_GAP * 2 + SEPARATOR_LINE;
    let segment_widths: u32 = segments.iter().map(|s| s.width).sum();
    let total_width = if segments.is_empty() {
        MIN_WIDTH
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
    
    // Calculate baseline to center text vertically based on actual glyph heights
    // Use a reference string containing common characters to determine visual center
    let reference_text = "0123456789% KMGTP"; 
    let mut min_y = i32::MAX;
    let mut max_y = i32::MIN;
    
    // We layout at 0,0 to find relative bounds
    for glyph in font.layout(reference_text, scale, rusttype::point(0.0, 0.0)) {
         if let Some(bb) = glyph.pixel_bounding_box() {
             if bb.min.y < min_y { min_y = bb.min.y; }
             if bb.max.y > max_y { max_y = bb.max.y; }
         }
    }
    
    // If we can't measure (e.g. empty font?), fallback to previous logic
    let baseline = if min_y < max_y {
        // visual_center relative to baseline = (min_y + max_y) / 2.0
        // We want baseline + visual_center = ICON_HEIGHT / 2.0
        // baseline = (ICON_HEIGHT / 2.0) - visual_center
        (ICON_HEIGHT as f32 / 2.0) - ((min_y + max_y) as f32 / 2.0)
    } else {
        (ICON_HEIGHT as f32 / 2.0) + (v_metrics.ascent / 2.0)
    };

    let measure_text = |text: &str| -> f32 {
        font.layout(text, scale, rusttype::point(0.0, 0.0))
            .map(|g| g.unpositioned().h_metrics().advance_width)
            .sum()
    };

    let text_color: u8 = get_text_color();

    let draw_text = |text: &str, start_x: f32, img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>| {
        for glyph in font.layout(text, scale, rusttype::point(start_x, baseline)) {
            if let Some(bb) = glyph.pixel_bounding_box() {
                glyph.draw(|gx, gy, v| {
                    let x = (bb.min.x + gx as i32) as u32;
                    let y = (bb.min.y + gy as i32) as u32;
                    if x < total_width && y < ICON_HEIGHT {
                        let alpha = (v * 255.0) as u8;
                        img.put_pixel(x, y, Rgba([text_color, text_color, text_color, alpha]));
                    }
                });
            }
        }
    };

    // Render SVG icon and blit to image buffer
    let icon_size = ICON_HEIGHT;
    // Center icon vertically in the available height (should be 0 if size == height)
    let icon_y_offset = 0;

    let draw_svg_icon = |svg_data: &str, start_x: u32, img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>| {
        let icon_pixels = render_svg_icon(svg_data, icon_size, text_color);
        let icon_width = icon_size; // Square icons

        for y in 0..icon_size {
            for x in 0..icon_width {
                let src_idx = ((y * icon_width + x) * 4) as usize;
                if src_idx + 3 < icon_pixels.len() {
                    let alpha = icon_pixels[src_idx + 3];
                    if alpha > 0 {
                        let dst_x = start_x + x;
                        let dst_y = icon_y_offset + y;
                        if dst_x < total_width && dst_y < ICON_HEIGHT {
                            img.put_pixel(dst_x, dst_y, Rgba([
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

    let mut x_offset = EDGE_PADDING;
    for (i, segment) in segments.iter().enumerate() {
        if i > 0 {
            x_offset += SEPARATOR_GAP * 2 + SEPARATOR_LINE;
        }

        let label_width: f32 = icon_size as f32; // All labels are now icons

        match segment.label {
            SegmentLabel::IconCpu => draw_svg_icon(SVG_CPU, x_offset, &mut img),
            SegmentLabel::IconMem => draw_svg_icon(SVG_MEMORY, x_offset, &mut img),
            SegmentLabel::IconGpu => draw_svg_icon(SVG_GPU, x_offset, &mut img),
            SegmentLabel::IconDown => draw_svg_icon(SVG_ARROW_DOWN, x_offset, &mut img),
            SegmentLabel::IconUp => draw_svg_icon(SVG_ARROW_UP, x_offset, &mut img),
        }
        
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
    gpu_available: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(desktop)]
    let autostart_manager = app.autolaunch();

    // Check if this is the first run by looking for a marker file.
    // If it doesn't exist, enable autostart by default and create the marker.
    #[cfg(desktop)]
    let is_autostart_enabled = {
        let marker_path: PathBuf = app
            .path()
            .app_data_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(".autostart_configured");
        
        if !marker_path.exists() {
            // First run: enable autostart by default
            let _ = autostart_manager.enable();
            // Create parent directory if needed and write the marker file
            if let Some(parent) = marker_path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let _ = fs::write(&marker_path, "1");
            true
        } else {
            // Subsequent run: read current state (respects user's choice)
            autostart_manager.is_enabled().unwrap_or(false)
        }
    };
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

    let show_net_item = CheckMenuItem::with_id(
        app, "show_net", "Show Network", true, show_net.load(Relaxed), None::<&str>,
    )?;

    let separator2 = PredefinedMenuItem::separator(app)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    // Build menu - only include GPU option if GPU monitoring is available
    let menu = if gpu_available {
        let show_gpu_item = CheckMenuItem::with_id(
            app, "show_gpu", "Show GPU", true, show_gpu.load(Relaxed), None::<&str>,
        )?;
        Menu::with_items(
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
        )?
    } else {
        Menu::with_items(
            app,
            &[
                &autostart_item,
                &separator1,
                &show_cpu_item,
                &show_mem_item,
                &show_net_item,
                &separator2,
                &quit_item,
            ],
        )?
    };

    let font = load_system_font();
    let (pixels, width, height) = render_tray_icon(
        &font,
        0.0, 0.0, 0.0, 0.0, 0.0,
        show_cpu.load(Relaxed),
        show_mem.load(Relaxed),
        show_gpu.load(Relaxed) && gpu_available,
        show_net.load(Relaxed),
    );
    let initial_icon = Image::new_owned(pixels, width, height);

    // Template icons let macOS adapt colors to menu bar theme automatically
    #[cfg(target_os = "macos")]
    let tray_builder = TrayIconBuilder::with_id("main")
        .icon(initial_icon)
        .icon_as_template(true);
    
    #[cfg(target_os = "linux")]
    let tray_builder = TrayIconBuilder::with_id("main")
        .icon(initial_icon);

    let _tray = tray_builder
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
                "show_cpu" => { show_cpu.fetch_xor(true, Relaxed); }
                "show_mem" => { show_mem.fetch_xor(true, Relaxed); }
                "show_gpu" => { show_gpu.fetch_xor(true, Relaxed); }
                "show_net" => { show_net.fetch_xor(true, Relaxed); }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .build(app)?;

    #[cfg(debug_assertions)]
    eprintln!("[INFO] Tray icon builder completed");

    // Try to verify the icon exists (tray-icon doesn't expose this well,
    // so this is a best-effort check)
    #[cfg(all(debug_assertions, target_os = "linux"))]
    {
        // Check if AppIndicator service should be registered
        // Note: This is informational only, can't easily verify from Rust
        eprintln!("[INFO] If tray icon doesn't appear, check:");
        eprintln!("  1. GNOME Shell extension: ubuntu-appindicators@ubuntu.com is enabled");
        eprintln!("  2. Display server connection is active");
        eprintln!("  3. Check system tray/notification area");
    }

    Ok(())
}

fn start_monitoring(
    app: AppHandle,
    show_cpu: Arc<AtomicBool>,
    show_mem: Arc<AtomicBool>,
    show_gpu: Arc<AtomicBool>,
    show_net: Arc<AtomicBool>,
    gpu_available: bool,
) {
    thread::spawn(move || {
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
            let sg = show_gpu.load(Relaxed) && gpu_available;
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
                    #[cfg(target_os = "macos")]
                    {
                        let result = tray.with_inner_tray_icon(move |inner| {
                            let icon = tray_icon::Icon::from_rgba(pixels, width, height).ok();
                            inner.set_icon_with_as_template(icon, true)
                        });
                        #[cfg(debug_assertions)]
                        if let Err(e) = result {
                            eprintln!("Failed to update tray icon: {:?}", e);
                        }
                        #[cfg(not(debug_assertions))]
                        let _ = result;
                    }

                    #[cfg(target_os = "linux")]
                    {
                        let result = tray.with_inner_tray_icon(move |inner| {
                            let icon = tray_icon::Icon::from_rgba(pixels, width, height).ok();
                            inner.set_icon(icon)
                        });
                        #[cfg(debug_assertions)]
                        if let Err(e) = result {
                            eprintln!("Failed to update tray icon: {:?}", e);
                        }
                        #[cfg(not(debug_assertions))]
                        let _ = result;
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

    let show_cpu_tray = show_cpu.clone();
    let show_mem_tray = show_mem.clone();
    let show_gpu_tray = show_gpu.clone();
    let show_net_tray = show_net.clone();

    // Check if GPU monitoring is available
    let gpu_available = GpuSampler::new().is_some();

    // Validate display environment on Linux before attempting tray icon creation
    #[cfg(target_os = "linux")]
    {
        use std::env;

        let has_display = env::var("DISPLAY").is_ok()
            || env::var("WAYLAND_DISPLAY").is_ok();

        if !has_display {
            eprintln!("Warning: No display server detected (DISPLAY/WAYLAND_DISPLAY not set).");
            eprintln!("The app may fail to create the tray icon.");
            eprintln!("If running via SSH, consider: ssh -X for X11 forwarding");
        }

        // Suppress GTK debug messages on Linux
        std::env::set_var("G_MESSAGES_DEBUG", "");
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|_app, _args, _cwd| {
            // Silent: tray-only app has no window to focus
        }))
        .plugin(tauri_plugin_shell::init())
        .setup(move |app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(ActivationPolicy::Accessory);

            // Platform-specific autostart initialization
            // Note: MacosLauncher is ignored on non-macOS platforms
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
                gpu_available,
            )?;

            start_monitoring(
                app.handle().clone(),
                show_cpu,
                show_mem,
                show_gpu,
                show_net,
                gpu_available,
            );

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
