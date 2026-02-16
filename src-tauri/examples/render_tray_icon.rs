use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::path::PathBuf;

use image::codecs::png::PngEncoder;
use image::ColorType;
use image::ImageEncoder;

use better_resource_monitor_lib::{load_system_font, tray_render};

fn usage() -> &'static str {
    "render_tray_icon\n\nUSAGE:\n  cargo run --manifest-path src-tauri/Cargo.toml --bin render_tray_icon -- [args]\n\nARGS:\n  --out <path>                 Output PNG path (required)\n  --preset <macos|linux>       Sizing preset (default: host OS)\n  --scale <float>              Uniform scale factor (default: 1.0)\n\n  --cpu <float>                CPU percent (default: 45)\n  --mem <float>                Memory percent (default: 99)\n  --gpu <float>                GPU percent (default: 78)\n  --down <string>              Download display (default: 1.5 MB)\n  --up <string>                Upload display (default: 0.2 MB)\n\n  --show-cpu <true|false>       (default: true)\n  --show-mem <true|false>       (default: true)\n  --show-gpu <true|false>       (default: true)\n  --show-net <true|false>       (default: true)\n  --show-alerts <true|false>   (default: true)\n  --use-light-icons <true|false> (default: true)\n\n  --bg <transparent|#RRGGBB|#RRGGBBAA> (default: transparent)\n  --help\n"
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

    if !(scale > 0.0) {
        panic!("--scale must be > 0");
    }

    let cpu = args
        .get("--cpu")
        .map(|v| parse_f32(v, "--cpu"))
        .unwrap_or(45.0);
    let mem = args
        .get("--mem")
        .map(|v| parse_f32(v, "--mem"))
        .unwrap_or(99.0);
    let gpu = args
        .get("--gpu")
        .map(|v| parse_f32(v, "--gpu"))
        .unwrap_or(78.0);

    let down = args
        .get("--down")
        .cloned()
        .unwrap_or_else(|| "1.5 MB".to_string());
    let up = args
        .get("--up")
        .cloned()
        .unwrap_or_else(|| "0.2 MB".to_string());

    let show_cpu = args
        .get("--show-cpu")
        .map(|v| parse_bool(v, "--show-cpu"))
        .unwrap_or(true);
    let show_mem = args
        .get("--show-mem")
        .map(|v| parse_bool(v, "--show-mem"))
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
    let mut buffer = Vec::new();

    let (width, height, _has_alert) = renderer.render_tray_icon_into(
        &font,
        &mut buffer,
        sizing,
        cpu,
        mem,
        gpu,
        &down,
        &up,
        show_cpu,
        show_mem,
        show_gpu,
        show_net,
        show_alerts,
        use_light_icons,
        background,
    );

    let Some(parent) = out.parent() else {
        panic!("Invalid output path");
    };
    if !parent.as_os_str().is_empty() {
        std::fs::create_dir_all(parent).expect("failed to create output directory");
    }

    let file = File::create(&out).expect("failed to create output file");
    let encoder = PngEncoder::new(file);
    encoder
        .write_image(&buffer, width, height, ColorType::Rgba8)
        .expect("failed to encode PNG");

    println!("Wrote {} ({}x{})", out.display(), width, height);
}
