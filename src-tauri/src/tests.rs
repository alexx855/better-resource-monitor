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
        gpu_usage: 0.0,
        down_str: "0 KB",
        up_str: "0 KB",
        show_cpu: true,
        show_mem: true,
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

fn assert_sizing(sizing: tray_render::Sizing, expected: (u32, u32, u32, u32, u32, f32)) {
    let (segment_width, segment_width_net, edge_padding, segment_gap, icon_height, font_size) =
        expected;

    assert_eq!(sizing.segment_width, segment_width);
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
fn test_should_update_threshold() {
    for (previous, new, threshold, expected) in [
        (10.0, 12.0, 2.0, true),
        (10.0, 8.0, 2.0, true),
        (10.0, 12.001, 2.0, true),
        (10.0, 11.9, 2.0, false),
        (10.0, 9.1, 2.0, false),
        (10.0, 10.0, 2.0, false),
    ] {
        assert_eq!(
            should_update(previous, new, threshold),
            expected,
            "previous={previous}, new={new}, threshold={threshold}"
        );
    }
}

#[test]
fn test_normalize_metric_flags() {
    for (input, expected) in [
        (
            (false, false, true, false, true),
            (false, false, true, false),
        ),
        (
            (false, false, true, false, false),
            (true, false, false, false),
        ),
        (
            (false, true, true, false, false),
            (false, true, false, false),
        ),
        (
            (false, false, false, false, true),
            (true, false, false, false),
        ),
        (
            (false, false, false, false, false),
            (true, false, false, false),
        ),
    ] {
        let (cpu, mem, gpu, net, gpu_available) = input;
        assert_eq!(
            normalize_metric_flags(cpu, mem, gpu, net, gpu_available),
            expected,
            "input={input:?}"
        );
    }
}

#[cfg(all(target_os = "macos", feature = "app-store"))]
#[test]
fn test_app_store_feature_uses_macos_gpu_sampler() {
    assert_eq!(crate::gpu::IMPLEMENTATION, "macos_ioaccelerator");
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
    for (bundle_id, executable_path, has_receipt, should_exit) in [
        (
            Some(MACOS_BUNDLE_ID),
            MACOS_SUPPORTED_EXECUTABLE_PATH,
            true,
            false,
        ),
        (
            Some(MACOS_BUNDLE_ID),
            "/Users/xeliapedersen/.Trash/Better Resource Monitor.app/Contents/MacOS/better-resource-monitor",
            true,
            true,
        ),
        (
            Some(MACOS_BUNDLE_ID),
            MACOS_SUPPORTED_EXECUTABLE_PATH,
            false,
            true,
        ),
        (None, "/tmp/better-resource-monitor", false, false),
    ] {
        assert_eq!(
            should_exit_unsupported_macos_bundle(
                bundle_id,
                std::path::Path::new(executable_path),
                has_receipt
            ),
            should_exit,
            "executable_path={executable_path}"
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
        (1_000_000_000_000.0, "1000 GB"),
        (1e15, "1000000 GB"),
        (-100.0, "-0.1 KB"),
    ] {
        assert_eq!(format_speed(input), expected, "input={input}");
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

    for (cpu_usage, show_alerts, expected_alert) in [
        (50.0, true, false),
        (80.0, true, false),
        (81.0, true, true),
        (81.0, false, false),
    ] {
        let (_, _, has_alert) = renderer.render_tray_icon_into(
            &font,
            &mut buffer,
            &tray_render::RenderConfig {
                cpu_usage,
                show_alerts,
                ..base_render_config()
            },
        );
        assert_eq!(
            has_alert, expected_alert,
            "cpu_usage={cpu_usage}, show_alerts={show_alerts}"
        );
    }
}

#[test]
fn test_sizing_scaled() {
    for (scale, expected) in [
        (2.0, (116, 150, 10, 36, 44, 38.0)),
        (0.5, (29, 38, 3, 9, 11, 9.5)),
        (0.333, (19, 25, 2, 6, 7, 19.0 * 0.333)),
    ] {
        assert_sizing(tray_render::SIZING_LINUX.scaled(scale), expected);
    }
}

#[test]
#[should_panic(expected = "scale must be > 0")]
fn test_sizing_scaled_panics_on_zero() {
    let _ = tray_render::SIZING_LINUX.scaled(0.0);
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
fn test_all_languages_have_translations() {
    let languages = [
        i18n::Language::English,
        i18n::Language::Spanish,
        i18n::Language::Portuguese,
        i18n::Language::Chinese,
    ];
    for lang in languages {
        let t = lang.translations();
        assert!(!t.start_at_login.is_empty());
        assert!(!t.show_memory.is_empty());
        assert!(!t.show_cpu.is_empty());
        assert!(!t.show_gpu.is_empty());
        assert!(!t.show_network.is_empty());
        assert!(!t.show_alert_colors.is_empty());
        assert!(!t.quit.is_empty());
        assert!(!t.system_monitor.is_empty());
    }
}

#[test]
fn test_english_defaults() {
    let t = i18n::Language::English.translations();
    assert_eq!(t.quit, "Quit");
    assert_eq!(t.system_monitor, "System Monitor");
}
