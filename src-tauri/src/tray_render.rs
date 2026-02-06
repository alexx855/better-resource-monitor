use std::collections::HashMap;

use image::{ImageBuffer, Rgba};
use rusttype::{Font, Scale};

const SVG_CPU: &str = include_str!("../assets/icons/svg/fill/cpu-fill.svg");
const SVG_MEMORY: &str = include_str!("../assets/icons/svg/fill/memory-fill.svg");
const SVG_GPU: &str = include_str!("../assets/icons/svg/fill/graphics-card-fill.svg");
const SVG_ARROW_UP: &str = include_str!("../assets/icons/svg/fill/cloud-arrow-up-fill.svg");
const SVG_ARROW_DOWN: &str = include_str!("../assets/icons/svg/fill/cloud-arrow-down-fill.svg");

const ALERT_THRESHOLD: f32 = 90.0;
const ALERT_COLOR: (u8, u8, u8) = (209, 71, 21); // #D14715

#[derive(Clone, Copy)]
pub struct Sizing {
    pub segment_width: u32,
    pub segment_width_net: u32,
    pub edge_padding: u32,
    pub segment_gap: u32,
    pub icon_height: u32,
    pub font_size: f32,
}

impl Sizing {
    pub fn scaled(self, scale: f32) -> Self {
        if !(scale > 0.0) {
            panic!("scale must be > 0");
        }

        let scale_u32 = |v: u32| -> u32 { ((v as f32) * scale).round().max(1.0) as u32 };
        Self {
            segment_width: scale_u32(self.segment_width),
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
    segment_width_net: 240,
    edge_padding: 16,
    segment_gap: 48,
    icon_height: 64,
    font_size: 56.0,
};

pub const SIZING_LINUX: Sizing = Sizing {
    segment_width: 58,
    segment_width_net: 75,
    edge_padding: 5,
    segment_gap: 18,
    icon_height: 22,
    font_size: 19.0,
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum IconType {
    Cpu,
    Memory,
    Gpu,
    ArrowDown,
    ArrowUp,
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
            if bb.min.y < min_y {
                min_y = bb.min.y;
            }
            if bb.max.y > max_y {
                max_y = bb.max.y;
            }
        }
    }

    if min_y < max_y {
        (icon_height as f32 / 2.0) - ((min_y + max_y) as f32 / 2.0)
    } else {
        (icon_height as f32 / 2.0) + (font.v_metrics(scale).ascent / 2.0)
    }
}

pub(crate) fn render_svg_icon(svg_data: &str, size: u32, color: (u8, u8, u8)) -> Vec<u8> {
    let color_hex = format!("#{:02x}{:02x}{:02x}", color.0, color.1, color.2);

    let svg_with_color = svg_data
        .replace("currentColor", &color_hex)
        .replace("<svg ", &format!("<svg fill=\"{color_hex}\" "));

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
    for chunk in pixels.chunks_exact_mut(4) {
        let alpha = chunk[3];
        if alpha > 0 && alpha < 255 {
            let a = alpha as u16;
            chunk[0] = ((chunk[0] as u16 * 255 / a).min(255)) as u8;
            chunk[1] = ((chunk[1] as u16 * 255 / a).min(255)) as u8;
            chunk[2] = ((chunk[2] as u16 * 255 / a).min(255)) as u8;
        }
    }
    pixels
}

struct IconCache {
    icons: HashMap<(IconType, (u8, u8, u8)), Vec<u8>>,
}

impl IconCache {
    fn new(size: u32) -> Self {
        let colors = [(255, 255, 255), (0, 0, 0), ALERT_COLOR];
        let icon_svgs = [
            (IconType::Cpu, SVG_CPU),
            (IconType::Memory, SVG_MEMORY),
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

    fn get(&self, icon_type: IconType, color: (u8, u8, u8)) -> &[u8] {
        self.icons.get(&(icon_type, color)).expect("icon cached")
    }
}

#[derive(Clone, Copy)]
pub struct Background {
    pub rgba: (u8, u8, u8, u8),
}

pub struct TrayRenderer {
    icon_caches: HashMap<u32, IconCache>,
    baseline_cache: Option<(u32, u32, f32)>,
}

impl TrayRenderer {
    pub fn new() -> Self {
        Self {
            icon_caches: HashMap::new(),
            baseline_cache: None,
        }
    }

    fn icon_cache_mut(&mut self, size: u32) -> &IconCache {
        if !self.icon_caches.contains_key(&size) {
            self.icon_caches.insert(size, IconCache::new(size));
        }
        self.icon_caches.get(&size).expect("icon cache exists")
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
        sizing: Sizing,
        cpu_usage: f32,
        mem_percent: f32,
        gpu_usage: f32,
        down_str: &str,
        up_str: &str,
        show_cpu: bool,
        show_mem: bool,
        show_gpu: bool,
        show_net: bool,
        show_alerts: bool,
        use_light_icons: bool,
        background: Option<Background>,
    ) -> (u32, u32, bool) {
        struct Segment {
            icon: IconType,
            value: String,
            width: u32,
            alert: bool,
        }

        let mut segments = Vec::with_capacity(5);
        let percent_segments = [
            (show_mem, IconType::Memory, mem_percent),
            (show_cpu, IconType::Cpu, cpu_usage),
            (show_gpu, IconType::Gpu, gpu_usage),
        ];
        for (show, icon, value) in percent_segments {
            if show {
                segments.push(Segment {
                    icon,
                    value: format!("{:.0}%", cap_percent(value)),
                    width: sizing.segment_width,
                    alert: value >= ALERT_THRESHOLD,
                });
            }
        }

        if show_net {
            segments.push(Segment {
                icon: IconType::ArrowDown,
                value: down_str.to_owned(),
                width: sizing.segment_width_net,
                alert: false,
            });
            segments.push(Segment {
                icon: IconType::ArrowUp,
                value: up_str.to_owned(),
                width: sizing.segment_width_net,
                alert: false,
            });
        }

        let has_active_alert = show_alerts && segments.iter().any(|s| s.alert);

        let total_width = sizing.edge_padding * 2
            + segments.iter().map(|s| s.width).sum::<u32>()
            + sizing.segment_gap * (segments.len() as u32).saturating_sub(1);

        let required_size = (total_width * sizing.icon_height * 4) as usize;
        buffer.clear();
        buffer.resize(required_size, 0);

        let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> =
            ImageBuffer::from_raw(total_width, sizing.icon_height, std::mem::take(buffer))
                .expect("buffer size matches dimensions");

        if let Some(bg) = background {
            let (r, g, b, a) = bg.rgba;
            for pixel in img.pixels_mut() {
                *pixel = Rgba([r, g, b, a]);
            }
        }

        let scale = Scale::uniform(sizing.font_size);
        let baseline = self.baseline(font, sizing);

        let icon_cache = self.icon_cache_mut(sizing.icon_height);

        let draw_text = |text: &str,
                         start_x: f32,
                         color: (u8, u8, u8),
                         background: Option<Background>,
                         img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>| {
            for glyph in font.layout(text, scale, rusttype::point(start_x, baseline)) {
                if let Some(bb) = glyph.pixel_bounding_box() {
                    glyph.draw(|gx, gy, v| {
                        let x = (bb.min.x + gx as i32) as u32;
                        let y = (bb.min.y + gy as i32) as u32;
                        if x < total_width && y < sizing.icon_height {
                            let alpha = (v * 255.0) as u8;
                            if alpha == 0 {
                                return;
                            }

                            if background.is_some() {
                                let dst = img.get_pixel_mut(x, y);
                                blend_over(dst, color, alpha);
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
             color: (u8, u8, u8),
             background: Option<Background>,
             img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>| {
                let icon_pixels = icon_cache.get(icon_type, color);
                let size = sizing.icon_height;

                for y in 0..size {
                    for x in 0..size {
                        let src_idx = ((y * size + x) * 4) as usize;
                        if src_idx + 3 < icon_pixels.len() {
                            let alpha = icon_pixels[src_idx + 3];
                            if alpha > 0 {
                                let dst_x = start_x + x;
                                if dst_x < total_width && y < size {
                                    if background.is_some() {
                                        let dst = img.get_pixel_mut(dst_x, y);
                                        blend_over(
                                            dst,
                                            (
                                                icon_pixels[src_idx],
                                                icon_pixels[src_idx + 1],
                                                icon_pixels[src_idx + 2],
                                            ),
                                            alpha,
                                        );
                                    } else {
                                        img.put_pixel(
                                            dst_x,
                                            y,
                                            Rgba([
                                                icon_pixels[src_idx],
                                                icon_pixels[src_idx + 1],
                                                icon_pixels[src_idx + 2],
                                                alpha,
                                            ]),
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            };

        let mut x_offset = sizing.edge_padding;
        for (i, segment) in segments.iter().enumerate() {
            if i > 0 {
                x_offset += sizing.segment_gap;
            }

            let segment_color = if has_active_alert {
                ALERT_COLOR
            } else if use_light_icons {
                (255, 255, 255)
            } else {
                (0, 0, 0)
            };

            draw_cached_icon(segment.icon, x_offset, segment_color, background, &mut img);

            let value_width: f32 = font
                .layout(&segment.value, scale, rusttype::point(0.0, 0.0))
                .map(|g| g.unpositioned().h_metrics().advance_width)
                .sum();
            let value_x = x_offset as f32 + segment.width as f32 - value_width;
            draw_text(&segment.value, value_x, segment_color, background, &mut img);

            x_offset += segment.width;
        }

        *buffer = img.into_raw();
        (total_width, sizing.icon_height, has_active_alert)
    }
}

fn blend_over(dst: &mut Rgba<u8>, src_rgb: (u8, u8, u8), src_alpha: u8) {
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
