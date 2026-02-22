use super::*;
use std::sync::{Mutex, OnceLock};

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

#[test]
fn test_cap_percent() {
    assert_eq!(tray_render::cap_percent(0.0), 0.0);
    assert_eq!(tray_render::cap_percent(50.0), 50.0);
    assert_eq!(tray_render::cap_percent(99.0), 99.0);
    assert_eq!(tray_render::cap_percent(100.0), 99.0);
    assert_eq!(tray_render::cap_percent(150.0), 99.0);
    assert_eq!(tray_render::cap_percent(-10.0), 0.0);
}

#[test]
fn test_should_update_threshold() {
    assert!(should_update(10.0, 12.0, 2.0));
    assert!(should_update(10.0, 8.0, 2.0));
    assert!(should_update(10.0, 12.001, 2.0));
    assert!(!should_update(10.0, 11.9, 2.0));
    assert!(!should_update(10.0, 9.1, 2.0));
    assert!(!should_update(10.0, 10.0, 2.0));
}

#[test]
fn test_format_speed() {
    // KB range (0.0 - 999.5)
    assert_eq!(format_speed(0.0), "0.0 KB");
    assert_eq!(format_speed(500.0), "0.5 KB");
    assert_eq!(format_speed(1_500.0), "1.5 KB");
    assert_eq!(format_speed(9_000.0), "9.0 KB");
    assert_eq!(format_speed(9_900.0), "9.9 KB");
    assert_eq!(format_speed(9_950.0), "9.9 KB"); // Still KB (threshold raised to ~1 MB)
    assert_eq!(format_speed(100_000.0), "100 KB"); // No decimal for >= 10
    assert_eq!(format_speed(500_000.0), "500 KB"); // No decimal for >= 10
    assert_eq!(format_speed(999_000.0), "999 KB"); // No decimal for >= 10
    assert_eq!(format_speed(999_500.0), "1.0 MB"); // Boundary: KB -> MB

    // MB range (1.0 - 999.5)
    assert_eq!(format_speed(1_500_000.0), "1.5 MB");
    assert_eq!(format_speed(9_900_000.0), "9.9 MB");
    assert_eq!(format_speed(9_950_000.0), "9.9 MB"); // Still MB (threshold raised to ~1 GB)
    assert_eq!(format_speed(10_000_000.0), "10 MB"); // No decimal for >= 10
    assert_eq!(format_speed(100_000_000.0), "100 MB"); // No decimal for >= 10
    assert_eq!(format_speed(500_000_000.0), "500 MB"); // No decimal for >= 10
    assert_eq!(format_speed(999_000_000.0), "999 MB"); // No decimal for >= 10
    assert_eq!(format_speed(999_500_000.0), "1.0 GB"); // Boundary: MB -> GB

    // GB range
    assert_eq!(format_speed(1_500_000_000.0), "1.5 GB");
    assert_eq!(format_speed(9_900_000_000.0), "9.9 GB");
    assert_eq!(format_speed(50_000_000_000.0), "50 GB"); // No decimal for >= 10

    // Edge cases
    assert_eq!(format_speed(1e-10), "0.0 KB");
    assert_eq!(format_speed(0.001), "0.0 KB");
    assert_eq!(format_speed(0.5), "0.0 KB");
    assert_eq!(format_speed(1_000_000_000_000.0), "1000 GB"); // No decimal for >= 10
    assert_eq!(format_speed(1e15), "1000000 GB"); // No decimal for >= 10
    assert_eq!(format_speed(-100.0), "-0.1 KB");
}

#[test]
fn test_render_svg_icon_valid() {
    // Simple valid SVG
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><circle cx="12" cy="12" r="10" fill="currentColor"/></svg>"#;
    let result = tray_render::render_svg_icon(svg, 16, (255, 255, 255));

    // Should return non-empty pixel data
    assert!(!result.is_empty());

    // 16x16 RGBA = 1024 bytes
    assert_eq!(result.len(), 16 * 16 * 4);
}

#[test]
#[should_panic(expected = "Failed to parse SVG")]
fn test_render_svg_icon_invalid_panics() {
    // Invalid SVG should panic (current behavior uses .expect())
    tray_render::render_svg_icon("not valid svg", 16, (255, 255, 255));
}

#[test]
fn test_icon_buffer_reuse() {
    let font = load_system_font().expect("test font required");

    let mut renderer = tray_render::TrayRenderer::new();

    // Create buffer with known capacity
    let mut buffer: Vec<u8> = Vec::with_capacity(4 * 800 * APP_SIZING.icon_height as usize);
    let initial_capacity = buffer.capacity();

    // First render
    let (width1, height1, _) = renderer.render_tray_icon_into(
        &font,
        &mut buffer,
        APP_SIZING,
        50.0,
        60.0,
        0.0,
        "1.0 KB",
        "0.5 KB",
        true,
        true,
        false,
        true,
        false,
        true,
        None,
    );
    assert!(width1 > 0);
    assert_eq!(height1, APP_SIZING.icon_height);
    assert!(!buffer.is_empty());

    // Capacity should be preserved or grown, never shrunk
    let capacity_after_first = buffer.capacity();
    assert!(capacity_after_first >= initial_capacity);

    // Second render with different values - buffer should be reused
    let (width2, height2, _) = renderer.render_tray_icon_into(
        &font,
        &mut buffer,
        APP_SIZING,
        70.0,
        80.0,
        0.0,
        "2.0 KB",
        "1.0 KB",
        true,
        true,
        false,
        true,
        false,
        true,
        None,
    );
    assert!(width2 > 0);
    assert_eq!(height2, APP_SIZING.icon_height);

    // Capacity should still be preserved (key test: no reallocation for same-size renders)
    assert!(buffer.capacity() >= capacity_after_first);
}

#[test]
fn test_alert_colors_all_segments() {
    let font = load_system_font().expect("test font required");
    let mut buffer: Vec<u8> = Vec::new();

    let mut renderer = tray_render::TrayRenderer::new();

    // No alerts - has_active_alert should be false
    let (_, _, has_alert_no) = renderer.render_tray_icon_into(
        &font,
        &mut buffer,
        APP_SIZING,
        50.0,
        50.0,
        0.0,
        "0 KB",
        "0 KB",
        true,
        true,
        false,
        false,
        true, // alerts enabled
        true,
        None,
    );
    assert!(!has_alert_no);

    // CPU at 95% with alerts enabled - has_active_alert should be true
    let (_, _, has_alert_yes) = renderer.render_tray_icon_into(
        &font,
        &mut buffer,
        APP_SIZING,
        95.0,
        50.0,
        0.0,
        "0 KB",
        "0 KB",
        true,
        true,
        false,
        false,
        true, // alerts enabled
        true,
        None,
    );
    assert!(has_alert_yes);

    // CPU at 95% but alerts disabled - has_active_alert should be false
    let (_, _, has_alert_disabled) = renderer.render_tray_icon_into(
        &font,
        &mut buffer,
        APP_SIZING,
        95.0,
        50.0,
        0.0,
        "0 KB",
        "0 KB",
        true,
        true,
        false,
        false,
        false, // alerts disabled
        true,
        None,
    );
    assert!(!has_alert_disabled);
}

#[test]
fn test_sizing_scaled_up() {
    let scaled = tray_render::SIZING_LINUX.scaled(2.0);

    assert_eq!(scaled.segment_width, 116);
    assert_eq!(scaled.segment_width_net, 150);
    assert_eq!(scaled.edge_padding, 10);
    assert_eq!(scaled.segment_gap, 36);
    assert_eq!(scaled.icon_height, 44);
    assert_eq!(scaled.font_size, 38.0);
}

#[test]
fn test_sizing_scaled_down() {
    let scaled = tray_render::SIZING_LINUX.scaled(0.5);

    assert_eq!(scaled.segment_width, 29);
    assert_eq!(scaled.segment_width_net, 38);
    assert_eq!(scaled.edge_padding, 3);
    assert_eq!(scaled.segment_gap, 9);
    assert_eq!(scaled.icon_height, 11);
    assert_eq!(scaled.font_size, 9.5);
}

#[test]
fn test_sizing_scaled_rounding() {
    let scaled = tray_render::SIZING_LINUX.scaled(0.333);

    assert_eq!(scaled.segment_width, 19);
    assert_eq!(scaled.segment_width_net, 25);
    assert_eq!(scaled.edge_padding, 2);
    assert_eq!(scaled.segment_gap, 6);
    assert_eq!(scaled.icon_height, 7);
    assert_eq!(scaled.font_size, 19.0 * 0.333);
}

#[test]
#[should_panic(expected = "scale must be > 0")]
fn test_sizing_scaled_panics_on_zero() {
    let _ = tray_render::SIZING_LINUX.scaled(0.0);
}

#[test]
fn test_get_update_interval_ms_default_when_unset() {
    let _guard = env_lock().lock().expect("env lock poisoned");
    let previous = std::env::var("SILICON_UPDATE_INTERVAL").ok();
    std::env::remove_var("SILICON_UPDATE_INTERVAL");

    assert_eq!(get_update_interval_ms(), UPDATE_INTERVAL_MS);

    if let Some(value) = previous {
        std::env::set_var("SILICON_UPDATE_INTERVAL", value);
    }
}

#[test]
fn test_get_update_interval_ms_valid_env() {
    let _guard = env_lock().lock().expect("env lock poisoned");
    let previous = std::env::var("SILICON_UPDATE_INTERVAL").ok();
    std::env::set_var("SILICON_UPDATE_INTERVAL", "1234");

    assert_eq!(get_update_interval_ms(), 1234);

    if let Some(value) = previous {
        std::env::set_var("SILICON_UPDATE_INTERVAL", value);
    } else {
        std::env::remove_var("SILICON_UPDATE_INTERVAL");
    }
}

#[test]
fn test_get_update_interval_ms_invalid_env_falls_back() {
    let _guard = env_lock().lock().expect("env lock poisoned");
    let previous = std::env::var("SILICON_UPDATE_INTERVAL").ok();
    std::env::set_var("SILICON_UPDATE_INTERVAL", "abc");

    assert_eq!(get_update_interval_ms(), UPDATE_INTERVAL_MS);

    if let Some(value) = previous {
        std::env::set_var("SILICON_UPDATE_INTERVAL", value);
    } else {
        std::env::remove_var("SILICON_UPDATE_INTERVAL");
    }
}

#[test]
fn test_render_with_all_segments_disabled() {
    let font = load_system_font().expect("test font required");
    let mut buffer = Vec::new();
    let mut renderer = tray_render::TrayRenderer::new();

    let (width, height, has_alert) = renderer.render_tray_icon_into(
        &font,
        &mut buffer,
        APP_SIZING,
        50.0,
        50.0,
        50.0,
        "0 KB",
        "0 KB",
        false,
        false,
        false,
        false,
        true,
        true,
        None,
    );

    assert_eq!(width, APP_SIZING.edge_padding * 2);
    assert_eq!(height, APP_SIZING.icon_height);
    assert!(!has_alert);
    assert_eq!(buffer.len(), (width * height * 4) as usize);
}

#[test]
fn test_render_with_long_network_strings() {
    let font = load_system_font().expect("test font required");
    let mut buffer = Vec::new();
    let mut renderer = tray_render::TrayRenderer::new();
    let long_down = "9".repeat(512);
    let long_up = "8".repeat(512);

    let (width, height, has_alert) = renderer.render_tray_icon_into(
        &font,
        &mut buffer,
        APP_SIZING,
        0.0,
        0.0,
        0.0,
        &long_down,
        &long_up,
        false,
        false,
        false,
        true,
        true,
        true,
        None,
    );

    let expected_width = APP_SIZING.edge_padding * 2
        + (APP_SIZING.segment_width_net * 2)
        + APP_SIZING.segment_gap;

    assert_eq!(width, expected_width);
    assert_eq!(height, APP_SIZING.icon_height);
    assert!(!has_alert);
    assert_eq!(buffer.len(), (width * height * 4) as usize);
}
