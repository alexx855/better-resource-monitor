use super::*;

#[test]
fn test_cap_percent() {
    assert_eq!(cap_percent(0.0), 0.0);
    assert_eq!(cap_percent(50.0), 50.0);
    assert_eq!(cap_percent(99.0), 99.0);
    assert_eq!(cap_percent(100.0), 99.0);
    assert_eq!(cap_percent(150.0), 99.0);
    assert_eq!(cap_percent(-10.0), 0.0);
}

#[test]
fn test_get_text_color() {
    assert_eq!(get_text_color(true), (255, 255, 255));
    assert_eq!(get_text_color(false), (0, 0, 0));
}

#[test]
fn test_format_speed() {
    // KB range (0.0 - 9.9)
    assert_eq!(format_speed(0.0), "0.0 KB");
    assert_eq!(format_speed(500.0), "0.5 KB");
    assert_eq!(format_speed(1_500.0), "1.5 KB");
    assert_eq!(format_speed(9_000.0), "9.0 KB");
    assert_eq!(format_speed(9_900.0), "9.9 KB");
    assert_eq!(format_speed(9_950.0), "0.0 MB"); // Boundary: KB -> MB

    // MB range (0.0 - 9.9)
    assert_eq!(format_speed(100_000.0), "0.1 MB");
    assert_eq!(format_speed(1_500_000.0), "1.5 MB");
    assert_eq!(format_speed(9_900_000.0), "9.9 MB");
    assert_eq!(format_speed(9_950_000.0), "0.0 GB"); // Boundary: MB -> GB

    // GB range (capped at 9.9)
    assert_eq!(format_speed(100_000_000.0), "0.1 GB");
    assert_eq!(format_speed(1_500_000_000.0), "1.5 GB");
    assert_eq!(format_speed(9_900_000_000.0), "9.9 GB");
    assert_eq!(format_speed(50_000_000_000.0), "9.9 GB");

    // Edge cases
    assert_eq!(format_speed(1e-10), "0.0 KB");
    assert_eq!(format_speed(0.001), "0.0 KB");
    assert_eq!(format_speed(0.5), "0.0 KB");
    assert_eq!(format_speed(1_000_000_000_000.0), "9.9 GB");
    assert_eq!(format_speed(1e15), "9.9 GB");
    assert_eq!(format_speed(-100.0), "-0.1 KB");
}

#[cfg(target_os = "linux")]
#[test]
fn test_detect_light_icons_from_desktop() {
    // DEs with typically light themes -> dark (black) icons
    for de in ["XFCE", "xfce", "Xfce", "elementary", "Pantheon:elementary", "KDE", "kde"] {
        assert_eq!(detect_light_icons_from_desktop(de), Some(false), "failed for: {de}");
    }

    // DEs that return None (use default)
    for de in ["GNOME", "gnome", "ubuntu:GNOME", "i3", "sway", ""] {
        assert_eq!(detect_light_icons_from_desktop(de), None, "failed for: {de}");
    }
}

#[test]
fn test_render_svg_icon_valid() {
    // Simple valid SVG
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><circle cx="12" cy="12" r="10" fill="currentColor"/></svg>"#;
    let result = render_svg_icon(svg, 16, (255, 255, 255));

    // Should return non-empty pixel data
    assert!(!result.is_empty());

    // 16x16 RGBA = 1024 bytes
    assert_eq!(result.len(), 16 * 16 * 4);
}

#[test]
#[should_panic(expected = "Failed to parse SVG")]
fn test_render_svg_icon_invalid_panics() {
    // Invalid SVG should panic (current behavior uses .expect())
    render_svg_icon("not valid svg", 16, (255, 255, 255));
}
