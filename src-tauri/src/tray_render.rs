use std::collections::HashMap;

use image::{ImageBuffer, Rgba};
use rusttype::{Font, Scale};

const SVG_CPU: &str = include_str!("../assets/icons/svg/fill/cpu-fill.svg");
const SVG_MEMORY: &str = include_str!("../assets/icons/svg/fill/memory-fill.svg");
const SVG_STORAGE: &str = include_str!("../assets/icons/svg/fill/disc-fill.svg");
const SVG_GPU: &str = include_str!("../assets/icons/svg/fill/graphics-card-fill.svg");
const SVG_ARROW_UP: &str = include_str!("../assets/icons/svg/fill/cloud-arrow-up-fill.svg");
const SVG_ARROW_DOWN: &str = include_str!("../assets/icons/svg/fill/cloud-arrow-down-fill.svg");
type Color = (u8, u8, u8);

const ALERT_THRESHOLD: f32 = 81.0;
const ALERT_COLOR: Color = (209, 71, 21); // #D14715
const ALERT_FOREGROUND: Color = (255, 255, 255);

#[derive(Clone, Copy)]
pub struct Sizing {
    pub segment_width: u32,
    pub segment_width_storage: u32,
    pub segment_width_net: u32,
    pub edge_padding: u32,
    pub segment_gap: u32,
    pub icon_height: u32,
    pub font_size: f32,
}

impl Sizing {
    pub fn scaled(self, scale: f32) -> Self {
        assert!(scale > 0.0, "scale must be > 0");

        let scale_u32 = |v: u32| -> u32 { ((v as f32) * scale).round().max(1.0) as u32 };
        Self {
            segment_width: scale_u32(self.segment_width),
            segment_width_storage: scale_u32(self.segment_width_storage),
            segment_width_net: scale_u32(self.segment_width_net),
            edge_padding: scale_u32(self.edge_padding),
            segment_gap: scale_u32(self.segment_gap),
            icon_height: scale_u32(self.icon_height),
            font_size: self.font_size * scale,
        }
    }
}

pub const SIZING_MACOS: Sizing = Sizing {
    segment_width: 180,
    segment_width_storage: 320,
    segment_width_net: 240,
    edge_padding: 16,
    segment_gap: 48,
    icon_height: 64,
    font_size: 56.0,
};

pub const SIZING_LINUX: Sizing = Sizing {
    segment_width: 58,
    segment_width_storage: 128,
    segment_width_net: 75,
    edge_padding: 5,
    segment_gap: 18,
    icon_height: 22,
    font_size: 19.0,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum IconType {
    Cpu,
    Memory,
    Storage,
    Gpu,
    ArrowDown,
    ArrowUp,
}

const METRIC_ICON_ORDER: [IconType; 4] = [
    IconType::Memory,
    IconType::Cpu,
    IconType::Gpu,
    IconType::Storage,
];

#[cfg(test)]
pub(crate) fn metric_icon_order_for_tests() -> [IconType; 4] {
    METRIC_ICON_ORDER
}

pub(crate) fn alert_active(value: f32) -> bool {
    value >= ALERT_THRESHOLD
}

pub(crate) fn cap_percent(value: f32) -> f32 {
    value.clamp(0.0, 99.0)
}

fn calculate_font_baseline(font: &Font, icon_height: u32, scale: Scale) -> f32 {
    let reference_text = "0123456789% KMGTP";
    let mut min_y = i32::MAX;
    let mut max_y = i32::MIN;

    for glyph in font.layout(reference_text, scale, rusttype::point(0.0, 0.0)) {
        if let Some(bb) = glyph.pixel_bounding_box() {
            min_y = min_y.min(bb.min.y);
            max_y = max_y.max(bb.max.y);
        }
    }

    if min_y < max_y {
        (icon_height as f32 / 2.0) - ((min_y + max_y) as f32 / 2.0)
    } else {
        (icon_height as f32 / 2.0) + (font.v_metrics(scale).ascent / 2.0)
    }
}

fn text_width(font: &Font, text: &str, scale: Scale) -> f32 {
    font.layout(text, scale, rusttype::point(0.0, 0.0))
        .map(|glyph| glyph.unpositioned().h_metrics().advance_width)
        .sum()
}

pub(crate) fn fit_text_scale(font: &Font, text: &str, font_size: f32, max_width: f32) -> Scale {
    let base_scale = Scale::uniform(font_size);
    let width = text_width(font, text, base_scale);

    if width > max_width && width > 0.0 {
        Scale::uniform((font_size * max_width / width).max(1.0))
    } else {
        base_scale
    }
}

fn storage_segment_width(font: &Font, text: &str, sizing: Sizing) -> u32 {
    let scale = Scale::uniform(sizing.font_size);
    let network_text_width = text_width(font, "0.0 KB", scale);
    let network_inner_gap =
        (sizing.segment_width_net as f32 - sizing.icon_height as f32 - network_text_width).max(0.0);
    let desired_width =
        (sizing.icon_height as f32 + network_inner_gap + text_width(font, text, scale)).ceil()
            as u32;

    desired_width.clamp(sizing.segment_width_net, sizing.segment_width_storage)
}

pub(crate) fn render_svg_icon(svg_data: &str, size: u32, color: Color) -> Vec<u8> {
    let (r, g, b) = color;
    let color_hex = format!("#{r:02x}{g:02x}{b:02x}");

    let svg_with_color = svg_data.replace("currentColor", &color_hex);
    let svg_with_color = if svg_with_color
        .split_once('>')
        .map(|(svg_tag, _)| svg_tag.contains("fill="))
        .unwrap_or(false)
    {
        svg_with_color
    } else {
        svg_with_color.replace("<svg ", &format!("<svg fill=\"{color_hex}\" "))
    };

    let opts = resvg::usvg::Options::default();
    let tree = resvg::usvg::Tree::from_str(&svg_with_color, &opts).expect("Failed to parse SVG");

    let svg_size = tree.size();
    let scale = size as f32 / svg_size.width().max(svg_size.height());

    let scaled_width = svg_size.width() * scale;
    let scaled_height = svg_size.height() * scale;

    let mut pixmap = resvg::tiny_skia::Pixmap::new(size, size).expect("Failed to create pixmap");

    let tx = (size as f32 - scaled_width) / 2.0;
    let ty = (size as f32 - scaled_height) / 2.0;
    let transform = resvg::tiny_skia::Transform::from_translate(tx, ty).post_scale(scale, scale);
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    let mut pixels = pixmap.take();
    // Un-premultiply alpha so image crate gets straight-alpha pixels
    for chunk in pixels.as_chunks_mut::<4>().0 {
        let a = chunk[3] as u16;
        if a > 0 && a < 255 {
            for c in &mut chunk[..3] {
                *c = ((*c as u16 * 255 / a).min(255)) as u8;
            }
        }
    }
    pixels
}

struct IconCache {
    icons: HashMap<(IconType, Color), Vec<u8>>,
}

impl IconCache {
    fn new(size: u32) -> Self {
        let colors = [(255, 255, 255), (0, 0, 0)];
        let icon_svgs = [
            (IconType::Cpu, SVG_CPU),
            (IconType::Memory, SVG_MEMORY),
            (IconType::Storage, SVG_STORAGE),
            (IconType::Gpu, SVG_GPU),
            (IconType::ArrowDown, SVG_ARROW_DOWN),
            (IconType::ArrowUp, SVG_ARROW_UP),
        ];

        let mut icons = HashMap::new();
        for (icon_type, svg) in icon_svgs {
            for color in colors {
                icons.insert((icon_type, color), render_svg_icon(svg, size, color));
            }
        }

        Self { icons }
    }

    fn get(&self, icon_type: IconType, color: Color) -> &[u8] {
        self.icons.get(&(icon_type, color)).expect("icon cached")
    }
}

#[derive(Clone, Copy)]
pub struct Background {
    pub rgba: (u8, u8, u8, u8),
}

pub struct RenderConfig<'a> {
    pub sizing: Sizing,
    pub cpu_usage: f32,
    pub mem_percent: f32,
    pub storage_available_str: &'a str,
    pub storage_used_percent: f32,
    pub gpu_usage: f32,
    pub down_str: &'a str,
    pub up_str: &'a str,
    pub show_cpu: bool,
    pub show_mem: bool,
    pub show_storage: bool,
    pub show_gpu: bool,
    pub show_net: bool,
    pub show_alerts: bool,
    pub use_light_icons: bool,
    pub background: Option<Background>,
}

#[derive(Default)]
pub struct TrayRenderer {
    icon_caches: HashMap<u32, IconCache>,
    baseline_cache: Option<(u32, u32, f32)>,
}

impl TrayRenderer {
    pub fn new() -> Self {
        Self::default()
    }

    fn icon_cache(&mut self, size: u32) -> &IconCache {
        self.icon_caches
            .entry(size)
            .or_insert_with(|| IconCache::new(size))
    }

    fn baseline(&mut self, font: &Font, sizing: Sizing) -> f32 {
        let font_size_key = (sizing.font_size * 1000.0).round() as u32;
        if let Some((h, fs, baseline)) = self.baseline_cache {
            if h == sizing.icon_height && fs == font_size_key {
                return baseline;
            }
        }

        let scale = Scale::uniform(sizing.font_size);
        let baseline = calculate_font_baseline(font, sizing.icon_height, scale);
        self.baseline_cache = Some((sizing.icon_height, font_size_key, baseline));
        baseline
    }

    pub fn render_tray_icon_into(
        &mut self,
        font: &Font,
        buffer: &mut Vec<u8>,
        config: &RenderConfig,
    ) -> (u32, u32, bool) {
        struct Segment {
            icon: IconType,
            value: String,
            width: u32,
            alert: bool,
        }

        let sizing = config.sizing;
        let storage_width = storage_segment_width(font, config.storage_available_str, sizing);

        let mut segments = Vec::with_capacity(6);
        for icon in METRIC_ICON_ORDER {
            let (show, value, value_text, width) = match icon {
                IconType::Memory => (
                    config.show_mem,
                    config.mem_percent,
                    format!("{:.0}%", cap_percent(config.mem_percent)),
                    sizing.segment_width,
                ),
                IconType::Cpu => (
                    config.show_cpu,
                    config.cpu_usage,
                    format!("{:.0}%", cap_percent(config.cpu_usage)),
                    sizing.segment_width,
                ),
                IconType::Gpu => (
                    config.show_gpu,
                    config.gpu_usage,
                    format!("{:.0}%", cap_percent(config.gpu_usage)),
                    sizing.segment_width,
                ),
                IconType::Storage => (
                    config.show_storage,
                    config.storage_used_percent,
                    config.storage_available_str.to_owned(),
                    storage_width,
                ),
                IconType::ArrowDown | IconType::ArrowUp => {
                    unreachable!("network icons are separate")
                }
            };

            if show {
                segments.push(Segment {
                    icon,
                    value: value_text,
                    width,
                    alert: alert_active(value),
                });
            }
        }

        if config.show_net {
            segments.push(Segment {
                icon: IconType::ArrowDown,
                value: config.down_str.to_owned(),
                width: sizing.segment_width_net,
                alert: false,
            });
            segments.push(Segment {
                icon: IconType::ArrowUp,
                value: config.up_str.to_owned(),
                width: sizing.segment_width_net,
                alert: false,
            });
        }

        let has_active_alert = config.show_alerts && segments.iter().any(|s| s.alert);

        let total_width = sizing.edge_padding * 2
            + segments.iter().map(|s| s.width).sum::<u32>()
            + sizing.segment_gap * (segments.len() as u32).saturating_sub(1);

        let required_size = (total_width * sizing.icon_height * 4) as usize;
        buffer.clear();
        buffer.resize(required_size, 0);

        let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> =
            ImageBuffer::from_raw(total_width, sizing.icon_height, std::mem::take(buffer))
                .expect("buffer size matches dimensions");

        let alert_bg = Background {
            rgba: (ALERT_COLOR.0, ALERT_COLOR.1, ALERT_COLOR.2, 255),
        };
        let effective_bg = if has_active_alert {
            Some(alert_bg)
        } else {
            config.background
        };

        if let Some(bg) = effective_bg {
            let (r, g, b, a) = bg.rgba;
            let pixel = Rgba([r, g, b, a]);
            for p in img.pixels_mut() {
                *p = pixel;
            }
        }

        let scale = Scale::uniform(sizing.font_size);
        let baseline = self.baseline(font, sizing);

        let icon_cache = self.icon_cache(sizing.icon_height);

        let has_bg = effective_bg.is_some();
        let min_alpha: u8 = if has_bg { 4 } else { 1 };

        let draw_text = |text: &str,
                         text_scale: Scale,
                         text_baseline: f32,
                         start_x: f32,
                         clip_left: u32,
                         clip_right: u32,
                         color: Color,
                         img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>| {
            for glyph in font.layout(text, text_scale, rusttype::point(start_x, text_baseline)) {
                if let Some(bb) = glyph.pixel_bounding_box() {
                    glyph.draw(|gx, gy, v| {
                        let x = (bb.min.x + gx as i32) as u32;
                        let y = (bb.min.y + gy as i32) as u32;
                        if x >= clip_left
                            && x < clip_right
                            && x < total_width
                            && y < sizing.icon_height
                        {
                            let alpha = (v * 255.0) as u8;
                            if alpha < min_alpha {
                                return;
                            }

                            if has_bg {
                                blend_over(img.get_pixel_mut(x, y), color, alpha);
                            } else {
                                img.put_pixel(x, y, Rgba([color.0, color.1, color.2, alpha]));
                            }
                        }
                    });
                }
            }
        };

        let draw_cached_icon =
            |icon_type: IconType,
             start_x: u32,
             color: Color,
             img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>| {
                let icon_pixels = icon_cache.get(icon_type, color);
                let size = sizing.icon_height;
                let stride = (size * 4) as usize;

                for y in 0..size {
                    let row_start = y as usize * stride;
                    for x in 0..size {
                        let src_idx = row_start + (x as usize * 4);
                        if src_idx + 3 >= icon_pixels.len() {
                            continue;
                        }
                        let alpha = icon_pixels[src_idx + 3];
                        if alpha < min_alpha {
                            continue;
                        }
                        let dst_x = start_x + x;
                        if dst_x >= total_width {
                            continue;
                        }
                        let rgb = (
                            icon_pixels[src_idx],
                            icon_pixels[src_idx + 1],
                            icon_pixels[src_idx + 2],
                        );
                        if has_bg {
                            blend_over(img.get_pixel_mut(dst_x, y), rgb, alpha);
                        } else {
                            img.put_pixel(dst_x, y, Rgba([rgb.0, rgb.1, rgb.2, alpha]));
                        }
                    }
                }
            };

        let segment_color = if has_active_alert {
            ALERT_FOREGROUND
        } else if config.use_light_icons {
            (255, 255, 255)
        } else {
            (0, 0, 0)
        };

        let mut x_offset = sizing.edge_padding;
        for (i, segment) in segments.iter().enumerate() {
            if i > 0 {
                x_offset += sizing.segment_gap;
            }

            draw_cached_icon(segment.icon, x_offset, segment_color, &mut img);

            let text_left = x_offset + sizing.icon_height;
            let text_right = x_offset + segment.width;
            let available_width = (text_right - text_left) as f32;
            let value_scale =
                fit_text_scale(font, &segment.value, sizing.font_size, available_width);
            let value_width = text_width(font, &segment.value, value_scale);
            let value_baseline = if value_scale == scale {
                baseline
            } else {
                calculate_font_baseline(font, sizing.icon_height, value_scale)
            };
            let value_x = x_offset as f32 + segment.width as f32 - value_width;
            draw_text(
                &segment.value,
                value_scale,
                value_baseline,
                value_x,
                text_left,
                text_right,
                segment_color,
                &mut img,
            );

            x_offset += segment.width;
        }

        *buffer = img.into_raw();
        (total_width, sizing.icon_height, has_active_alert)
    }
}

fn blend_over(dst: &mut Rgba<u8>, src_rgb: Color, src_alpha: u8) {
    let (sr, sg, sb) = src_rgb;
    let sa = src_alpha as u32;

    let dr = dst[0] as u32;
    let dg = dst[1] as u32;
    let db = dst[2] as u32;
    let da = dst[3] as u32;

    let out_a = sa + (da * (255 - sa) + 127) / 255;
    if out_a == 0 {
        *dst = Rgba([0, 0, 0, 0]);
        return;
    }

    let src_r_p = (sr as u32 * sa + 127) / 255;
    let src_g_p = (sg as u32 * sa + 127) / 255;
    let src_b_p = (sb as u32 * sa + 127) / 255;

    let dst_r_p = (dr * da + 127) / 255;
    let dst_g_p = (dg * da + 127) / 255;
    let dst_b_p = (db * da + 127) / 255;

    let out_r_p = src_r_p + (dst_r_p * (255 - sa) + 127) / 255;
    let out_g_p = src_g_p + (dst_g_p * (255 - sa) + 127) / 255;
    let out_b_p = src_b_p + (dst_b_p * (255 - sa) + 127) / 255;

    let out_r = (out_r_p * 255 + out_a / 2) / out_a;
    let out_g = (out_g_p * 255 + out_a / 2) / out_a;
    let out_b = (out_b_p * 255 + out_a / 2) / out_a;

    *dst = Rgba([
        out_r.min(255) as u8,
        out_g.min(255) as u8,
        out_b.min(255) as u8,
        out_a.min(255) as u8,
    ]);
}
