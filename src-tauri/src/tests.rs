use super::*;
use crate::i18n;
use std::sync::{Mutex, OnceLock};

const UPDATE_INTERVAL_ENV: &str = "SILICON_UPDATE_INTERVAL";

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

struct EnvVarRestore {
    key: &'static str,
    previous: Option<std::ffi::OsString>,
}

impl Drop for EnvVarRestore {
    fn drop(&mut self) {
        if let Some(value) = &self.previous {
            std::env::set_var(self.key, value);
        } else {
            std::env::remove_var(self.key);
        }
    }
}

fn with_update_interval_env<R>(value: Option<&str>, assertion: impl FnOnce() -> R) -> R {
    let _guard = env_lock().lock().expect("env lock poisoned");
    let _restore = EnvVarRestore {
        key: UPDATE_INTERVAL_ENV,
        previous: std::env::var_os(UPDATE_INTERVAL_ENV),
    };

    if let Some(value) = value {
        std::env::set_var(UPDATE_INTERVAL_ENV, value);
    } else {
        std::env::remove_var(UPDATE_INTERVAL_ENV);
    }

    assertion()
}

fn base_render_config<'a>() -> tray_render::RenderConfig<'a> {
    tray_render::RenderConfig {
        sizing: APP_SIZING,
        cpu_usage: 50.0,
        mem_percent: 50.0,
        storage_available_str: "0.0 KB",
        storage_available_bytes: None,
        gpu_usage: 0.0,
        down_str: "0 KB",
        up_str: "0 KB",
        show_cpu: true,
        show_mem: true,
        show_storage: false,
        show_gpu: false,
        show_net: false,
        show_alerts: true,
        use_light_icons: true,
        background: None,
    }
}

fn assert_render_size(buffer: &[u8], width: u32, height: u32, expected_width: u32) {
    assert_eq!(width, expected_width);
    assert_eq!(height, APP_SIZING.icon_height);
    assert_eq!(buffer.len(), (width * height * 4) as usize);
}

fn visible_gap_between_icon_and_text(
    buffer: &[u8],
    image_width: u32,
    image_height: u32,
    segment_left: u32,
    segment_width: u32,
    icon_width: u32,
) -> u32 {
    let icon_right = (segment_left..segment_left + icon_width)
        .rev()
        .find(|&x| (0..image_height).any(|y| buffer[((y * image_width + x) * 4 + 3) as usize] > 0))
        .expect("rendered icon must contain visible pixels");
    let text_left = (segment_left + icon_width..segment_left + segment_width)
        .find(|&x| (0..image_height).any(|y| buffer[((y * image_width + x) * 4 + 3) as usize] > 0))
        .expect("rendered label must contain visible pixels");

    text_left
        .checked_sub(icon_right + 1)
        .expect("rendered label must not overlap its icon")
}

fn assert_sizing(sizing: tray_render::Sizing, expected: (u32, u32, u32, u32, u32, u32, f32)) {
    let (
        segment_width,
        segment_width_storage,
        segment_width_net,
        edge_padding,
        segment_gap,
        icon_height,
        font_size,
    ) = expected;

    assert_eq!(sizing.segment_width, segment_width);
    assert_eq!(sizing.segment_width_storage, segment_width_storage);
    assert_eq!(sizing.segment_width_net, segment_width_net);
    assert_eq!(sizing.edge_padding, edge_padding);
    assert_eq!(sizing.segment_gap, segment_gap);
    assert_eq!(sizing.icon_height, icon_height);
    assert_eq!(sizing.font_size, font_size);
}

#[test]
fn test_cap_percent() {
    for (input, expected) in [
        (0.0, 0.0),
        (50.0, 50.0),
        (99.0, 99.0),
        (100.0, 99.0),
        (150.0, 99.0),
        (-10.0, 0.0),
    ] {
        assert_eq!(tray_render::cap_percent(input), expected, "input={input}");
    }
}

#[test]
fn test_metric_display_update_uses_visibility_threshold_and_alert_state() {
    for (visible, previous, current, show_alerts, expected) in [
        (true, 10.0, 12.0, true, true),
        (true, 10.0, 8.0, true, true),
        (true, 10.0, 12.001, true, true),
        (true, 10.0, 11.9, true, false),
        (true, 10.0, 9.1, true, false),
        (true, 10.0, 10.0, true, false),
        // A hidden metric must not force repeated redraws from its initial or old value.
        (false, -100.0, 0.0, true, false),
        (false, 60.0, 0.0, true, false),
        (true, -100.0, 0.0, true, true),
        // Warning transitions redraw even when the value change is below hysteresis.
        (true, 80.5, 81.0, true, true),
        (true, 81.0, 80.5, true, true),
        (true, 80.5, 81.0, false, false),
        (true, 81.0, 80.5, false, false),
        (false, 80.5, 81.0, true, false),
    ] {
        assert_eq!(
            metric_display_needs_update(visible, previous, current, show_alerts),
            expected,
            "visible={visible}, previous={previous}, current={current}, show_alerts={show_alerts}"
        );
    }
}

#[test]
fn test_tray_update_retries_failed_frames_until_applied() {
    let mut updates = TrayUpdateTracker::default();
    assert!(!updates.begin_if_needed(false));
    assert!(updates.begin_if_needed(true));

    // Failed frames remain pending even after the sampled values stop changing.
    assert!(updates.begin_if_needed(false));
    assert!(updates.begin_if_needed(false));

    updates.mark_applied();
    assert!(!updates.begin_if_needed(false));
    assert!(updates.begin_if_needed(true));
    updates.mark_applied();
    assert!(!updates.begin_if_needed(false));
}

#[test]
fn test_normalize_metric_flags() {
    for (input, expected) in [
        (
            (false, false, true, false, false, true),
            (false, false, true, false, false),
        ),
        (
            (false, false, false, true, false, true),
            (false, false, false, true, false),
        ),
        (
            (false, false, false, true, false, false),
            (true, false, false, false, false),
        ),
        (
            (false, true, false, true, false, false),
            (false, true, false, false, false),
        ),
        (
            (false, false, false, false, false, true),
            (true, false, false, false, false),
        ),
        (
            (false, false, false, false, false, false),
            (true, false, false, false, false),
        ),
    ] {
        let (cpu, mem, storage, gpu, net, gpu_available) = input;
        assert_eq!(
            normalize_metric_flags(cpu, mem, storage, gpu, net, gpu_available),
            expected,
            "input={input:?}"
        );
    }
}

#[test]
fn test_storage_setting_migration() {
    for (stored_storage, has_legacy_metric_settings, expected) in [
        (Some(true), true, true),
        (Some(false), true, false),
        (Some(true), false, true),
        (Some(false), false, false),
        (None, false, true),
        (None, true, false),
    ] {
        assert_eq!(
            migrate_storage_setting(stored_storage, has_legacy_metric_settings),
            expected,
            "stored_storage={stored_storage:?}, has_legacy_metric_settings={has_legacy_metric_settings}"
        );
    }
}

#[test]
fn test_storage_sample_validates_and_clamps_available_bytes() {
    let sample = storage::sample_from_bytes(1_000, 250).expect("valid sample");
    assert_eq!(sample.available_bytes, 250);

    assert_eq!(storage::sample_from_bytes(0, 0), None);
    assert_eq!(storage::sample_from_bytes(0, 1_000), None);

    let clamped = storage::sample_from_bytes(1_000, 2_000).expect("clamped sample");
    assert_eq!(clamped.available_bytes, 1_000);

    let full = storage::sample_from_bytes(1_000, 0).expect("full filesystem");
    assert_eq!(full.available_bytes, 0);

    let huge = storage::sample_from_bytes(u128::MAX, u128::MAX).expect("bounded sample");
    assert_eq!(huge.available_bytes, u64::MAX);
}

#[test]
fn test_storage_block_math() {
    let sample = storage::sample_from_blocks(10, 4, 1024, 512).expect("valid statvfs sample");
    assert_eq!(sample.available_bytes, 4 * 1024);

    let fallback = storage::sample_from_blocks(10, 4, 0, 512).expect("block size fallback");
    assert_eq!(fallback.available_bytes, 4 * 512);

    assert_eq!(storage::sample_from_blocks(10, 4, 0, 0), None);
    assert_eq!(storage::sample_from_blocks(0, 4, 1024, 512), None);

    let clamped = storage::sample_from_blocks(10, 20, 1024, 512).expect("clamped blocks");
    assert_eq!(clamped.available_bytes, 10 * 1024);

    let huge = storage::sample_from_blocks(u128::MAX, u128::MAX / 2, u128::MAX, 0)
        .expect("huge values saturate");
    assert_eq!(huge.available_bytes, u64::MAX);
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
#[test]
fn test_storage_sample_reads_home_filesystem() {
    storage::sample().expect("home filesystem storage sample");
}

#[cfg(target_os = "macos")]
#[test]
fn test_macos_process_lock_path_is_per_user() {
    assert_eq!(
        macos_process_lock_path(501),
        std::path::PathBuf::from("/tmp/dev.alexpedersen.better-resource-monitor.501.lock")
    );
}

#[cfg(target_os = "macos")]
#[test]
fn test_supported_macos_bundle_runtime() {
    const TRASH_PATH: &str = "/Users/xeliapedersen/.Trash/Better Resource Monitor.app/Contents/MacOS/better-resource-monitor";

    for (bundle_id, executable_path, has_receipt, is_direct, should_exit) in [
        // App Store build: receipt at the supported path.
        (
            Some(MACOS_BUNDLE_ID),
            MACOS_SUPPORTED_EXECUTABLE_PATH,
            true,
            false,
            false,
        ),
        // Stray copy in the Trash exits even with a receipt.
        (Some(MACOS_BUNDLE_ID), TRASH_PATH, true, false, true),
        // Non-direct build without a receipt exits (existing behavior).
        (
            Some(MACOS_BUNDLE_ID),
            MACOS_SUPPORTED_EXECUTABLE_PATH,
            false,
            false,
            true,
        ),
        // Direct-distribution build: no receipt, supported path — allowed.
        (
            Some(MACOS_BUNDLE_ID),
            MACOS_SUPPORTED_EXECUTABLE_PATH,
            false,
            true,
            false,
        ),
        // Direct-distribution build in the Trash still exits.
        (Some(MACOS_BUNDLE_ID), TRASH_PATH, false, true, true),
        // Other bundle ids are never the guard's business.
        (None, "/tmp/better-resource-monitor", false, false, false),
    ] {
        assert_eq!(
            should_exit_unsupported_macos_bundle(
                bundle_id,
                std::path::Path::new(executable_path),
                has_receipt,
                is_direct
            ),
            should_exit,
            "executable_path={executable_path} has_receipt={has_receipt} is_direct={is_direct}"
        );
    }
}

#[cfg(target_os = "macos")]
#[test]
fn test_macos_receipt_path_for_executable_uses_bundle_contents() {
    assert_eq!(
        macos_receipt_path_for_executable(std::path::Path::new(MACOS_SUPPORTED_EXECUTABLE_PATH))
            .as_deref(),
        Some(std::path::Path::new(
            "/Applications/Better Resource Monitor.app/Contents/_MASReceipt/receipt"
        ))
    );
}

#[test]
fn test_format_speed() {
    for (input, expected) in [
        (0.0, "0.0 KB"),
        (500.0, "0.5 KB"),
        (1_500.0, "1.5 KB"),
        (9_000.0, "9.0 KB"),
        (9_900.0, "9.9 KB"),
        (9_950.0, "9.9 KB"),
        (100_000.0, "100 KB"),
        (500_000.0, "500 KB"),
        (999_000.0, "999 KB"),
        (999_500.0, "1.0 MB"),
        (1_500_000.0, "1.5 MB"),
        (9_900_000.0, "9.9 MB"),
        (9_950_000.0, "9.9 MB"),
        (10_000_000.0, "10 MB"),
        (100_000_000.0, "100 MB"),
        (500_000_000.0, "500 MB"),
        (999_000_000.0, "999 MB"),
        (999_500_000.0, "1.0 GB"),
        (1_500_000_000.0, "1.5 GB"),
        (9_900_000_000.0, "9.9 GB"),
        (50_000_000_000.0, "50 GB"),
        (1e-10, "0.0 KB"),
        (0.001, "0.0 KB"),
        (0.5, "0.0 KB"),
        (999_000_000_000.0, "999 GB"),
        (999_500_000_000.0, "1.0 TB"),
        (1_500_000_000_000.0, "1.5 TB"),
        (999_500_000_000_000.0, "1.0 PB"),
        (1_500_000_000_000_000.0, "1.5 PB"),
        (1e18, "1000 PB"),
        (1e20, ">99999 PB"),
        (-100.0, "-0.1 KB"),
    ] {
        assert_eq!(format_speed(input), expected, "input={input}");
    }

    assert_eq!(format_storage_available(0), "0.0 KB");
    assert_eq!(format_storage_available(9_949_999_999), "9.9 GB");
    assert_eq!(format_storage_available(9_950_000_000), "9.9 GB");
    assert_eq!(format_storage_available(9_999_999_999), "<10 GB");
    assert_eq!(format_storage_available(10_000_000_000), "10.0 GB");
    assert_eq!(format_storage_available(19_500_000_000), "19.5 GB");
    assert_eq!(format_storage_available(u64::MAX), "18.4 EB");
}

#[test]
fn test_storage_display_update_uses_label_and_alert_state() {
    for (previous_label, previous_bytes, current_label, current_bytes, expected) in [
        (None, None, "19.5 GB", Some(19_500_000_000), true),
        (
            Some("19.5 GB"),
            Some(19_500_000_000),
            "19.5 GB",
            Some(19_500_000_000),
            false,
        ),
        (
            Some("10.0 GB"),
            Some(10_010_000_000),
            "10.0 GB",
            Some(9_990_000_000),
            true,
        ),
        (
            Some("9.9 GB"),
            Some(9_950_000_000),
            "9.9 GB",
            Some(9_940_000_000),
            false,
        ),
        (
            Some("19.5 GB"),
            Some(19_500_000_000),
            "20 GB",
            Some(20_000_000_000),
            true,
        ),
    ] {
        assert_eq!(
            storage_display_needs_update(
                previous_label,
                previous_bytes,
                current_label,
                current_bytes,
            ),
            expected,
            "previous_label={previous_label:?}, previous_bytes={previous_bytes:?}, current_label={current_label}, current_bytes={current_bytes:?}"
        );
    }
}

#[test]
fn test_render_svg_icon_valid() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><circle cx="12" cy="12" r="10" fill="currentColor"/></svg>"#;
    let result = tray_render::render_svg_icon(svg, 16, (255, 255, 255));

    assert!(!result.is_empty());
    assert_eq!(result.len(), 16 * 16 * 4);
}

#[test]
#[should_panic(expected = "Failed to parse SVG")]
fn test_render_svg_icon_invalid_panics() {
    tray_render::render_svg_icon("not valid svg", 16, (255, 255, 255));
}

#[test]
fn test_icon_buffer_reuse() {
    let font = load_system_font().expect("test font required");
    let mut renderer = tray_render::TrayRenderer::new();
    let mut buffer: Vec<u8> = Vec::with_capacity(4 * 800 * APP_SIZING.icon_height as usize);
    let initial_capacity = buffer.capacity();

    let (width1, height1, _) = renderer.render_tray_icon_into(
        &font,
        &mut buffer,
        &tray_render::RenderConfig {
            mem_percent: 60.0,
            down_str: "1.0 KB",
            up_str: "0.5 KB",
            show_net: true,
            show_alerts: false,
            ..base_render_config()
        },
    );
    assert!(width1 > 0);
    assert_eq!(height1, APP_SIZING.icon_height);
    assert!(!buffer.is_empty());

    let capacity_after_first = buffer.capacity();
    assert!(capacity_after_first >= initial_capacity);

    let (width2, height2, _) = renderer.render_tray_icon_into(
        &font,
        &mut buffer,
        &tray_render::RenderConfig {
            cpu_usage: 70.0,
            mem_percent: 80.0,
            down_str: "2.0 KB",
            up_str: "1.0 KB",
            show_net: true,
            show_alerts: false,
            ..base_render_config()
        },
    );
    assert!(width2 > 0);
    assert_eq!(height2, APP_SIZING.icon_height);
    assert!(buffer.capacity() >= capacity_after_first);
}

#[test]
fn test_alert_colors_all_segments() {
    let font = load_system_font().expect("test font required");
    let mut buffer: Vec<u8> = Vec::new();
    let mut renderer = tray_render::TrayRenderer::new();

    let assert_alert_pixels = |buffer: &[u8], expected_alert: bool| {
        let (background, foreground) = if expected_alert {
            ([209, 71, 21, 255], [255, 255, 255, 255])
        } else {
            ([0, 0, 0, 0], [0, 0, 0, 255])
        };
        assert_eq!(&buffer[..4], &background);
        assert!(buffer.as_chunks::<4>().0.contains(&foreground));
    };

    for (show_cpu, show_mem, show_gpu) in [
        (true, false, false),
        (false, true, false),
        (false, false, true),
    ] {
        for (usage, visible, show_alerts, expected_alert) in [
            (50.0, true, true, false),
            (80.0, true, true, false),
            (81.0, true, true, true),
            (81.0, true, false, false),
            (81.0, false, true, false),
        ] {
            let (_, _, has_alert) = renderer.render_tray_icon_into(
                &font,
                &mut buffer,
                &tray_render::RenderConfig {
                    cpu_usage: usage,
                    mem_percent: usage,
                    gpu_usage: usage,
                    show_cpu: show_cpu && visible,
                    show_mem: show_mem && visible,
                    show_gpu: show_gpu && visible,
                    show_net: true,
                    show_alerts,
                    use_light_icons: false,
                    ..base_render_config()
                },
            );
            assert_eq!(
                has_alert, expected_alert,
                "show_cpu={show_cpu}, show_mem={show_mem}, show_gpu={show_gpu}, usage={usage}, visible={visible}, show_alerts={show_alerts}"
            );
            assert_alert_pixels(&buffer, expected_alert);
        }
    }

    for (storage_available_bytes, show_storage, show_alerts, expected_alert) in [
        (Some(10_000_000_001), true, true, false),
        (Some(10_000_000_000), true, true, false),
        (Some(9_999_999_999), true, true, true),
        (Some(9_999_999_999), true, false, false),
        (Some(9_999_999_999), false, true, false),
        (None, true, true, false),
    ] {
        let (_, _, has_alert) = renderer.render_tray_icon_into(
            &font,
            &mut buffer,
            &tray_render::RenderConfig {
                cpu_usage: 0.0,
                mem_percent: 0.0,
                storage_available_bytes,
                show_cpu: false,
                show_mem: false,
                show_storage,
                show_net: true,
                show_alerts,
                use_light_icons: false,
                ..base_render_config()
            },
        );
        assert_eq!(
            has_alert, expected_alert,
            "storage_available_bytes={storage_available_bytes:?}, show_storage={show_storage}, show_alerts={show_alerts}"
        );
        assert_alert_pixels(&buffer, expected_alert);
    }
}

#[test]
fn test_storage_alert_uses_available_space_instead_of_percentage() {
    let font = load_system_font().expect("test font required");
    let mut buffer = Vec::new();
    let mut renderer = tray_render::TrayRenderer::new();

    for (available, available_bytes, expected_alert) in [
        ("100 GB", Some(100_000_000_000), false),
        ("10.0 GB", Some(10_000_000_000), false),
        ("9.9 GB", Some(9_900_000_000), true),
        ("0.0 KB", None, false),
    ] {
        let (_, _, has_alert) = renderer.render_tray_icon_into(
            &font,
            &mut buffer,
            &tray_render::RenderConfig {
                cpu_usage: 0.0,
                mem_percent: 0.0,
                storage_available_str: available,
                storage_available_bytes: available_bytes,
                show_cpu: false,
                show_mem: false,
                show_storage: true,
                show_gpu: false,
                show_net: false,
                show_alerts: true,
                ..base_render_config()
            },
        );
        assert_eq!(
            has_alert, expected_alert,
            "available={available}, available_bytes={available_bytes:?}"
        );
    }
}

#[test]
fn test_sizing_scaled() {
    for (scale, expected) in [
        (2.0, (116, 256, 150, 10, 36, 44, 38.0)),
        (0.5, (29, 64, 38, 3, 9, 11, 9.5)),
        (0.333, (19, 43, 25, 2, 6, 7, 19.0 * 0.333)),
    ] {
        assert_sizing(tray_render::SIZING_LINUX.scaled(scale), expected);
    }
}

#[test]
#[should_panic(expected = "scale must be finite and between 0 and 4")]
fn test_sizing_scaled_panics_on_zero() {
    let _ = tray_render::SIZING_LINUX.scaled(0.0);
}

#[test]
#[should_panic(expected = "scale must be finite and between 0 and 4")]
fn test_sizing_scaled_panics_on_infinity() {
    let _ = tray_render::SIZING_LINUX.scaled(f32::INFINITY);
}

#[test]
#[should_panic(expected = "scale must be finite and between 0 and 4")]
fn test_sizing_scaled_panics_above_safe_limit() {
    let _ = tray_render::SIZING_LINUX.scaled(4.01);
}

#[test]
fn test_get_update_interval_ms() {
    for (value, expected) in [
        (None, UPDATE_INTERVAL_MS),
        (Some("1234"), 1234),
        (Some("abc"), UPDATE_INTERVAL_MS),
        (Some("0"), UPDATE_INTERVAL_MS),
    ] {
        with_update_interval_env(value, || {
            assert_eq!(get_update_interval_ms(), expected, "value={value:?}");
        });
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
        &tray_render::RenderConfig {
            gpu_usage: 50.0,
            show_cpu: false,
            show_mem: false,
            show_gpu: false,
            show_net: false,
            ..base_render_config()
        },
    );

    assert!(!has_alert);
    assert_render_size(&buffer, width, height, APP_SIZING.edge_padding * 2);
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
        &tray_render::RenderConfig {
            cpu_usage: 0.0,
            mem_percent: 0.0,
            down_str: &long_down,
            up_str: &long_up,
            show_cpu: false,
            show_mem: false,
            show_net: true,
            ..base_render_config()
        },
    );

    let expected_width =
        APP_SIZING.edge_padding * 2 + (APP_SIZING.segment_width_net * 2) + APP_SIZING.segment_gap;

    assert!(!has_alert);
    assert_render_size(&buffer, width, height, expected_width);
}

#[test]
fn test_render_all_default_visible_metrics_width() {
    let font = load_system_font().expect("test font required");
    let mut buffer = Vec::new();
    let mut renderer = tray_render::TrayRenderer::new();

    let (storage_only_width, _, _) = renderer.render_tray_icon_into(
        &font,
        &mut buffer,
        &tray_render::RenderConfig {
            show_cpu: false,
            show_mem: false,
            show_storage: true,
            ..base_render_config()
        },
    );
    let rendered_storage_segment_width = storage_only_width - APP_SIZING.edge_padding * 2;

    let (width, height, has_alert) = renderer.render_tray_icon_into(
        &font,
        &mut buffer,
        &tray_render::RenderConfig {
            show_storage: true,
            show_gpu: true,
            show_net: true,
            ..base_render_config()
        },
    );

    let percent_segments = 3;
    let network_segments = 2;
    let segment_count = percent_segments + 1 + network_segments;
    let expected_width = APP_SIZING.edge_padding * 2
        + (APP_SIZING.segment_width * percent_segments)
        + rendered_storage_segment_width
        + (APP_SIZING.segment_width_net * network_segments)
        + (APP_SIZING.segment_gap * (segment_count - 1));

    assert!(!has_alert);
    assert_render_size(&buffer, width, height, expected_width);
}

#[test]
fn test_storage_visual_geometry_matches_network_and_fits_extremes() {
    let font = load_system_font().expect("test font required");
    let labels = [
        "0.0 KB", "999 KB", "1.0 MB", "999 MB", "1.0 GB", "<10 GB", "38.7 GB", "999 GB", "1.0 TB",
        "999 TB", "1.0 PB", "999 PB", "1.0 EB", "18.4 EB",
    ];

    for sizing in [tray_render::SIZING_MACOS, tray_render::SIZING_LINUX] {
        let mut renderer = tray_render::TrayRenderer::new();
        let mut buffer = Vec::new();

        let (network_width, network_height, _) = renderer.render_tray_icon_into(
            &font,
            &mut buffer,
            &tray_render::RenderConfig {
                sizing,
                show_cpu: false,
                show_mem: false,
                show_storage: false,
                show_gpu: false,
                show_net: true,
                show_alerts: false,
                down_str: "0.0 KB",
                up_str: "0.0 KB",
                ..base_render_config()
            },
        );
        let rendered_network_segment_width =
            (network_width - sizing.edge_padding * 2 - sizing.segment_gap) / 2;
        assert_eq!(network_height, sizing.icon_height);
        let network_visible_gap = visible_gap_between_icon_and_text(
            &buffer,
            network_width,
            network_height,
            sizing.edge_padding,
            rendered_network_segment_width,
            sizing.icon_height,
        );
        for label in labels {
            let (width, height, has_alert) = renderer.render_tray_icon_into(
                &font,
                &mut buffer,
                &tray_render::RenderConfig {
                    sizing,
                    storage_available_str: label,
                    show_cpu: false,
                    show_mem: false,
                    show_storage: true,
                    show_gpu: false,
                    show_net: false,
                    show_alerts: false,
                    ..base_render_config()
                },
            );
            assert!(!has_alert);
            assert_eq!(height, sizing.icon_height);
            assert_eq!(buffer.len(), (width * height * 4) as usize);

            let rendered_storage_segment_width = width - sizing.edge_padding * 2;
            let storage_available_width =
                (rendered_storage_segment_width - sizing.icon_height) as f32;
            let storage_scale = tray_render::fit_text_scale(
                &font,
                label,
                sizing.font_size,
                storage_available_width,
            );
            let storage_label_width: f32 = font
                .layout(label, storage_scale, rusttype::point(0.0, 0.0))
                .map(|glyph| glyph.unpositioned().h_metrics().advance_width)
                .sum();
            assert!(storage_label_width <= storage_available_width + 0.01);
            let storage_visible_gap = visible_gap_between_icon_and_text(
                &buffer,
                width,
                height,
                sizing.edge_padding,
                rendered_storage_segment_width,
                sizing.icon_height,
            );
            assert!(
                storage_visible_gap.abs_diff(network_visible_gap) <= 1,
                "storage label {label:?} visible gap {storage_visible_gap} must match network gap {network_visible_gap} within one raster pixel"
            );

            let ink_columns = buffer
                .as_chunks::<4>()
                .0
                .iter()
                .enumerate()
                .filter_map(|(index, pixel)| (pixel[3] > 0).then_some(index as u32 % width));
            let (leftmost, rightmost) =
                ink_columns.fold((width, 0), |(left, right), x| (left.min(x), right.max(x)));
            assert!(leftmost >= sizing.edge_padding);
            assert!(rightmost < width - sizing.edge_padding);
        }
    }
}

#[test]
fn test_render_buffer_matches_rgba_dimensions() {
    // The monitor loop skips a frame when the rendered buffer is not exactly
    // 4 bytes per pixel; this pins the renderer invariant that check relies on.
    let font = load_system_font().expect("test font required");
    let mut buffer = Vec::new();
    let mut renderer = tray_render::TrayRenderer::new();

    for config in [
        base_render_config(),
        tray_render::RenderConfig {
            show_storage: true,
            show_gpu: true,
            show_net: true,
            ..base_render_config()
        },
    ] {
        let (width, height, _) = renderer.render_tray_icon_into(&font, &mut buffer, &config);
        assert_eq!(
            buffer.len(),
            (4 * width * height) as usize,
            "render buffer must be 4 bytes per pixel for {width}x{height}"
        );
    }
}

#[test]
fn test_rendered_metric_order_is_memory_cpu_gpu_storage() {
    let font = load_system_font().expect("test font required");
    let mut renderer = tray_render::TrayRenderer::new();
    let config = tray_render::RenderConfig {
        mem_percent: 23.0,
        cpu_usage: 45.0,
        gpu_usage: 67.0,
        storage_available_str: "19.5 GB",
        show_mem: true,
        show_cpu: true,
        show_gpu: true,
        show_storage: true,
        ..base_render_config()
    };
    let mut combined = Vec::new();
    let (combined_width, height, _) = renderer.render_tray_icon_into(&font, &mut combined, &config);
    let mut segment = Vec::new();
    let mut segment_left = APP_SIZING.edge_padding;

    for (show_mem, show_cpu, show_gpu, show_storage) in [
        (true, false, false, false),
        (false, true, false, false),
        (false, false, true, false),
        (false, false, false, true),
    ] {
        let (width, segment_height, _) = renderer.render_tray_icon_into(
            &font,
            &mut segment,
            &tray_render::RenderConfig {
                show_mem,
                show_cpu,
                show_gpu,
                show_storage,
                ..config
            },
        );
        assert_eq!(segment_height, height);
        let segment_width = width - APP_SIZING.edge_padding * 2;
        for y in 0..height {
            let combined_start = ((y * combined_width + segment_left) * 4) as usize;
            let segment_start = ((y * width + APP_SIZING.edge_padding) * 4) as usize;
            // Cached icons have exact pixels at every offset. Text layout uses
            // floating point positions whose rounding can vary with the offset.
            let row_bytes = APP_SIZING.icon_height as usize * 4;
            assert_eq!(
                &combined[combined_start..combined_start + row_bytes],
                &segment[segment_start..segment_start + row_bytes],
                "metric at x={segment_left}, row={y}"
            );
        }
        segment_left += segment_width + APP_SIZING.segment_gap;
    }

    assert_eq!(
        segment_left - APP_SIZING.segment_gap + APP_SIZING.edge_padding,
        combined_width
    );
}

#[test]
fn test_detect_language_does_not_panic() {
    let lang = i18n::detect_language();
    let t = lang.translations();
    assert!(!t.quit.is_empty());
}

#[test]
fn test_detect_language_from_locales_recognizes_supported_tags() {
    for (locales, expected) in [
        (&["es-ES"][..], i18n::Language::Spanish),
        (&["ES_mx"][..], i18n::Language::Spanish),
        (&["pt-BR"][..], i18n::Language::Portuguese),
        (&["zh-Hans"][..], i18n::Language::Chinese),
        (&["en-US"][..], i18n::Language::English),
    ] {
        assert_eq!(
            i18n::detect_language_from_locales(locales.iter().copied()),
            expected,
            "locales={locales:?}"
        );
    }
}

#[test]
fn test_detect_language_from_locales_uses_preference_order() {
    for (locales, expected) in [
        (&["fr-FR", "es-ES"][..], i18n::Language::Spanish),
        (&["en-US", "es-ES"][..], i18n::Language::English),
        (&["fr-FR", "de-DE", ""][..], i18n::Language::English),
    ] {
        assert_eq!(
            i18n::detect_language_from_locales(locales.iter().copied()),
            expected,
            "locales={locales:?}"
        );
    }
}

#[test]
fn test_menu_translations_match_supported_locales() {
    let expected = [
        (
            i18n::Language::English,
            [
                "Start at login",
                "Show memory",
                "Show CPU",
                "Show storage",
                "Show GPU",
                "Show network",
                "Show warning colors",
                "Quit",
                "System monitor",
            ],
        ),
        (
            i18n::Language::Spanish,
            [
                "Abrir al iniciar sesión",
                "Mostrar memoria",
                "Mostrar CPU",
                "Mostrar almacenamiento",
                "Mostrar GPU",
                "Mostrar uso de red",
                "Mostrar colores de advertencia",
                "Salir",
                "Monitor del sistema",
            ],
        ),
        (
            i18n::Language::Portuguese,
            [
                "Iniciar com o sistema",
                "Mostrar memória",
                "Mostrar CPU",
                "Mostrar armazenamento",
                "Mostrar GPU",
                "Mostrar rede",
                "Mostrar cores de aviso",
                "Sair",
                "Monitor do sistema",
            ],
        ),
        (
            i18n::Language::Chinese,
            [
                "登录时启动",
                "显示内存",
                "显示 CPU",
                "显示存储空间",
                "显示 GPU",
                "显示网络",
                "显示警告颜色",
                "退出",
                "系统监控器",
            ],
        ),
    ];

    for (language, expected) in expected {
        let translations = language.translations();
        let actual = [
            translations.start_at_login,
            translations.show_memory,
            translations.show_cpu,
            translations.show_storage,
            translations.show_gpu,
            translations.show_network,
            translations.show_alert_colors,
            translations.quit,
            translations.system_monitor,
        ];

        assert_eq!(actual, expected, "language={language:?}");
    }
}

#[test]
fn test_thermal_translations_match_supported_locales() {
    let expected = [
        (
            i18n::Language::English,
            [
                "Thermal status",
                "unavailable",
                "nominal",
                "fair",
                "serious",
                "critical",
            ],
        ),
        (
            i18n::Language::Spanish,
            [
                "Estado térmico",
                "no disponible",
                "normal",
                "elevado",
                "alto",
                "crítico",
            ],
        ),
        (
            i18n::Language::Portuguese,
            [
                "Estado térmico",
                "indisponível",
                "normal",
                "elevado",
                "alto",
                "crítico",
            ],
        ),
        (
            i18n::Language::Chinese,
            ["温度状态", "不可用", "正常", "偏高", "高", "危急"],
        ),
    ];

    for (language, expected) in expected {
        let translations = language.translations();
        let actual = [
            translations.thermal_status,
            translations.thermal_unavailable,
            translations.thermal_nominal,
            translations.thermal_fair,
            translations.thermal_serious,
            translations.thermal_critical,
        ];

        assert_eq!(actual, expected, "language={language:?}");
    }
}

#[cfg(target_os = "macos")]
#[test]
fn test_thermal_menu_status_text_is_localized_for_every_locale() {
    let expected = [
        (
            i18n::Language::English,
            [
                "Thermal status: nominal",
                "Thermal status: fair",
                "Thermal status: serious",
                "Thermal status: critical",
                "Thermal status: unavailable",
            ],
        ),
        (
            i18n::Language::Spanish,
            [
                "Estado térmico: normal",
                "Estado térmico: elevado",
                "Estado térmico: alto",
                "Estado térmico: crítico",
                "Estado térmico: no disponible",
            ],
        ),
        (
            i18n::Language::Portuguese,
            [
                "Estado térmico: normal",
                "Estado térmico: elevado",
                "Estado térmico: alto",
                "Estado térmico: crítico",
                "Estado térmico: indisponível",
            ],
        ),
        (
            i18n::Language::Chinese,
            [
                "温度状态: 正常",
                "温度状态: 偏高",
                "温度状态: 高",
                "温度状态: 危急",
                "温度状态: 不可用",
            ],
        ),
    ];

    for (language, expected) in expected {
        let translations = language.translations();
        let actual = [
            thermal_status_text(translations, thermal::ThermalStatus::Nominal),
            thermal_status_text(translations, thermal::ThermalStatus::Fair),
            thermal_status_text(translations, thermal::ThermalStatus::Serious),
            thermal_status_text(translations, thermal::ThermalStatus::Critical),
            thermal_status_text(translations, thermal::ThermalStatus::Unavailable),
        ];

        assert_eq!(actual, expected, "language={language:?}");
    }
}
