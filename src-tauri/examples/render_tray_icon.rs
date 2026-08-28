use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::path::PathBuf;

use image::codecs::png::PngEncoder;
use image::ColorType;
use image::ImageEncoder;

use better_resource_monitor_lib::{load_system_font, tray_render};

fn usage() -> &'static str {
    "render_tray_icon\n\nUSAGE:\n  cargo run --manifest-path src-tauri/Cargo.toml --example render_tray_icon -- [args]\n\nARGS:\n  --out <path>                     Output PNG path (required)\n  --preset <macos|linux>           Sizing preset (default: host OS)\n  --scale <float>                  Uniform scale factor (default: 1.0)\n\n  --cpu <float>                    CPU percent (default: 45)\n  --mem <float>                    Memory percent (default: 57)\n  --storage <string>               Available storage display (default: 19.5 GB)\n  --storage-available-bytes <u64>  Available storage bytes for alert coloring (default: 19500000000)\n  --gpu <float>                    GPU percent (default: 32)\n  --down <string>                  Download display (default: 1.5 MB)\n  --up <string>                    Upload display (default: 0.2 MB)\n\n  --alert-cpu <float>              Alert row CPU percent (default: 93)\n  --alert-mem <float>              Alert row memory percent (default: 96)\n  --alert-storage-bytes <u64>      Alert row available storage bytes (default: 5000000000)\n  --alert-gpu <float>              Alert row GPU percent (default: 91)\n  --alert-down <string>            Alert row download display (default: 12 MB)\n  --alert-up <string>              Alert row upload display (default: 3.1 MB)\n\n  --show-cpu <true|false>          (default: true)\n  --show-mem <true|false>          (default: true)\n  --show-storage <true|false>      (default: true)\n  --show-gpu <true|false>          (default: true)\n  --show-net <true|false>          (default: true)\n  --show-alerts <true|false>       (default: true)\n  --use-light-icons <true|false>   (default: true)\n  --include-alert-row <true|false> (default: false)\n\n  --bg <transparent|#RRGGBB|#RRGGBBAA> (default: transparent)\n  --help\n"
}

#[derive(Clone, Copy)]
enum Preset {
    Macos,
    Linux,
}

fn default_preset() -> Preset {
    #[cfg(target_os = "macos")]
    {
        Preset::Macos
    }

    #[cfg(not(target_os = "macos"))]
    {
        Preset::Linux
    }
}

fn parse_bool(s: &str, key: &str) -> bool {
    match s {
        "true" => true,
        "false" => false,
        _ => panic!("{key} must be 'true' or 'false'"),
    }
}

fn parse_f32(s: &str, key: &str) -> f32 {
    s.parse::<f32>()
        .unwrap_or_else(|_| panic!("{key} must be a number"))
}

fn parse_u64(s: &str, key: &str) -> u64 {
    s.parse::<u64>()
        .unwrap_or_else(|_| panic!("{key} must be a non-negative integer"))
}

fn parse_bg_hex(s: &str) -> Option<tray_render::Background> {
    let hex = s.strip_prefix('#').unwrap_or(s);
    let bytes = match hex.len() {
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            (r, g, b, 255)
        }
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
            (r, g, b, a)
        }
        _ => return None,
    };

    Some(tray_render::Background { rgba: bytes })
}

fn parse_args() -> HashMap<String, String> {
    let mut args = env::args().skip(1);
    let mut map = HashMap::new();

    while let Some(arg) = args.next() {
        if arg == "--help" {
            print!("{}", usage());
            std::process::exit(0);
        }

        if !arg.starts_with("--") {
            panic!("Unexpected arg '{arg}'. Use --help.");
        }

        let Some(value) = args.next() else {
            panic!("Missing value for '{arg}'");
        };

        if map.insert(arg, value).is_some() {
            panic!("Duplicate argument");
        }
    }

    map
}

fn main() {
    let args = parse_args();

    let out = args
        .get("--out")
        .map(PathBuf::from)
        .unwrap_or_else(|| panic!("--out is required"));

    let preset = match args.get("--preset").map(String::as_str) {
        None => default_preset(),
        Some("macos") => Preset::Macos,
        Some("linux") => Preset::Linux,
        Some(v) => panic!("--preset must be 'macos' or 'linux', got '{v}'"),
    };

    let scale = args
        .get("--scale")
        .map(|v| {
            v.parse::<f32>()
                .unwrap_or_else(|_| panic!("--scale must be a number"))
        })
        .unwrap_or(1.0);

    if !scale.is_finite() || scale <= 0.0 || scale > tray_render::MAX_RENDER_SCALE {
        panic!("--scale must be finite and between 0 and 4");
    }

    let cpu = args
        .get("--cpu")
        .map(|v| parse_f32(v, "--cpu"))
        .unwrap_or(45.0);
    let mem = args
        .get("--mem")
        .map(|v| parse_f32(v, "--mem"))
        .unwrap_or(57.0);
    let storage_available = args
        .get("--storage")
        .cloned()
        .unwrap_or_else(|| "19.5 GB".to_string());
    let storage_available_bytes = args
        .get("--storage-available-bytes")
        .map(|v| parse_u64(v, "--storage-available-bytes"))
        .unwrap_or(19_500_000_000);
    let gpu = args
        .get("--gpu")
        .map(|v| parse_f32(v, "--gpu"))
        .unwrap_or(32.0);

    let down = args
        .get("--down")
        .cloned()
        .unwrap_or_else(|| "1.5 MB".to_string());
    let up = args
        .get("--up")
        .cloned()
        .unwrap_or_else(|| "0.2 MB".to_string());

    let alert_cpu = args
        .get("--alert-cpu")
        .map(|v| parse_f32(v, "--alert-cpu"))
        .unwrap_or(93.0);
    let alert_mem = args
        .get("--alert-mem")
        .map(|v| parse_f32(v, "--alert-mem"))
        .unwrap_or(96.0);
    let alert_storage_available_bytes = args
        .get("--alert-storage-bytes")
        .map(|v| parse_u64(v, "--alert-storage-bytes"))
        .unwrap_or(5_000_000_000);
    let alert_gpu = args
        .get("--alert-gpu")
        .map(|v| parse_f32(v, "--alert-gpu"))
        .unwrap_or(91.0);
    let alert_down = args
        .get("--alert-down")
        .cloned()
        .unwrap_or_else(|| "12 MB".to_string());
    let alert_up = args
        .get("--alert-up")
        .cloned()
        .unwrap_or_else(|| "3.1 MB".to_string());

    let show_cpu = args
        .get("--show-cpu")
        .map(|v| parse_bool(v, "--show-cpu"))
        .unwrap_or(true);
    let show_mem = args
        .get("--show-mem")
        .map(|v| parse_bool(v, "--show-mem"))
        .unwrap_or(true);
    let show_storage = args
        .get("--show-storage")
        .map(|v| parse_bool(v, "--show-storage"))
        .unwrap_or(true);
    let show_gpu = args
        .get("--show-gpu")
        .map(|v| parse_bool(v, "--show-gpu"))
        .unwrap_or(true);
    let show_net = args
        .get("--show-net")
        .map(|v| parse_bool(v, "--show-net"))
        .unwrap_or(true);
    let show_alerts = args
        .get("--show-alerts")
        .map(|v| parse_bool(v, "--show-alerts"))
        .unwrap_or(true);
    let use_light_icons = args
        .get("--use-light-icons")
        .map(|v| parse_bool(v, "--use-light-icons"))
        .unwrap_or(true);
    let include_alert_row = args
        .get("--include-alert-row")
        .map(|v| parse_bool(v, "--include-alert-row"))
        .unwrap_or(false);

    let background = match args.get("--bg").map(String::as_str) {
        None => None,
        Some("transparent") => None,
        Some(v) => Some(
            parse_bg_hex(v)
                .unwrap_or_else(|| panic!("--bg must be 'transparent', '#RRGGBB', or '#RRGGBBAA'")),
        ),
    };

    let sizing = match preset {
        Preset::Macos => tray_render::SIZING_MACOS,
        Preset::Linux => tray_render::SIZING_LINUX,
    }
    .scaled(scale);

    let font = load_system_font().expect("font required");
    let mut renderer = tray_render::TrayRenderer::new();
    let mut primary_buffer = Vec::new();

    let primary_config = tray_render::RenderConfig {
        sizing,
        cpu_usage: cpu,
        mem_percent: mem,
        storage_available_str: &storage_available,
        storage_available_bytes: Some(storage_available_bytes),
        gpu_usage: gpu,
        down_str: &down,
        up_str: &up,
        show_cpu,
        show_mem,
        show_storage,
        show_gpu,
        show_net,
        show_alerts,
        use_light_icons,
        background,
    };

    let (width, height, _has_alert) =
        renderer.render_tray_icon_into(&font, &mut primary_buffer, &primary_config);

    let (output_width, output_height, output_buffer) = if include_alert_row {
        let mut alert_buffer = Vec::new();
        let alert_config = tray_render::RenderConfig {
            cpu_usage: alert_cpu,
            mem_percent: alert_mem,
            storage_available_bytes: Some(alert_storage_available_bytes),
            gpu_usage: alert_gpu,
            down_str: &alert_down,
            up_str: &alert_up,
            show_alerts: true,
            ..primary_config
        };
        let (alert_width, alert_height, _has_alert) =
            renderer.render_tray_icon_into(&font, &mut alert_buffer, &alert_config);

        assert_eq!(width, alert_width, "stacked rows must share width");
        assert_eq!(height, alert_height, "stacked rows must share height");

        primary_buffer.extend_from_slice(&alert_buffer);
        (width, height * 2, primary_buffer)
    } else {
        (width, height, primary_buffer)
    };

    let Some(parent) = out.parent() else {
        panic!("Invalid output path");
    };
    if !parent.as_os_str().is_empty() {
        std::fs::create_dir_all(parent).expect("failed to create output directory");
    }

    let file = File::create(&out).expect("failed to create output file");
    let encoder = PngEncoder::new(file);
    encoder
        .write_image(
            &output_buffer,
            output_width,
            output_height,
            ColorType::Rgba8.into(),
        )
        .expect("failed to encode PNG");

    println!(
        "Wrote {} ({}x{})",
        out.display(),
        output_width,
        output_height
    );
}
