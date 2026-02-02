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

#[test]
fn test_icon_buffer_reuse() {
    // This test verifies the buffer reuse optimization is working.
    // After the first render, the buffer should be reused (same capacity).
    let font = load_system_font();

    // First render - allocates buffer
    let (pixels1, w1, h1) = render_tray_icon(
        &font, 50.0, 60.0, 0.0, 1000.0, 500.0,
        true, true, false, true, true,
    );
    assert!(!pixels1.is_empty());
    assert!(w1 > 0 && h1 > 0);

    // Get buffer capacity after first render
    let capacity_after_first = ICON_BUFFER
        .get()
        .map(|m| m.lock().unwrap().capacity())
        .unwrap_or(0);

    // Second render - should reuse buffer
    let (pixels2, w2, h2) = render_tray_icon(
        &font, 75.0, 80.0, 0.0, 2000.0, 1000.0,
        true, true, false, true, true,
    );
    assert!(!pixels2.is_empty());
    assert_eq!(w2, w1); // Same config = same width
    assert_eq!(h2, h1);

    // Buffer capacity should remain the same (reused, not reallocated)
    let capacity_after_second = ICON_BUFFER
        .get()
        .map(|m| m.lock().unwrap().capacity())
        .unwrap_or(0);

    assert_eq!(
        capacity_after_first, capacity_after_second,
        "Buffer should be reused, not reallocated"
    );

    // Third render with different visible segments (different width)
    let (pixels3, w3, _) = render_tray_icon(
        &font, 50.0, 60.0, 0.0, 0.0, 0.0,
        true, false, false, false, true, // Only CPU visible
    );
    assert!(!pixels3.is_empty());
    assert!(w3 < w1); // Fewer segments = narrower

    // Buffer capacity should still be sufficient (>= required)
    let capacity_after_third = ICON_BUFFER
        .get()
        .map(|m| m.lock().unwrap().capacity())
        .unwrap_or(0);

    println!("Capacities: first={}, second={}, third={}",
        capacity_after_first, capacity_after_second, capacity_after_third);

    // Capacity shouldn't decrease (Vec doesn't shrink)
    assert!(
        capacity_after_third >= capacity_after_first,
        "Buffer capacity should not decrease: {} < {}",
        capacity_after_third, capacity_after_first
    );
}
