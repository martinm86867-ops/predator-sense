use gtk4::prelude::*;
use gtk4::{self as gtk};
use std::f64::consts::PI;

/// Semantic "health" color for a temperature reading - green (normal), amber
/// (warm), red (hot). Returns `(r, g, b)` in `0.0..=1.0`.
pub fn temp_color(celsius: f64) -> (f64, f64, f64) {
    if celsius >= 85.0 {
        (0.93, 0.26, 0.22) // red
    } else if celsius >= 70.0 {
        (0.85, 0.50, 0.05) // rich amber
    } else {
        if crate::ui::brand_theme::is_light_mode() {
            (0.08, 0.58, 0.35) // deeper contrast green for light mode
        } else {
            (0.24, 0.78, 0.45) // bright green
        }
    }
}

/// Same thresholds as [`temp_color`], as a CSS hex string for text markup.
/// Derived from [`temp_color`] so the two can never drift apart.
pub fn temp_color_hex(celsius: f64) -> String {
    let (r, g, b) = temp_color(celsius);
    format!(
        "#{:02x}{:02x}{:02x}",
        (r * 255.0).round() as u8,
        (g * 255.0).round() as u8,
        (b * 255.0).round() as u8,
    )
}

/// An open 240° telemetry gauge: smooth recessed groove, calibration ticks,
/// ambient neon glow, and a glowing needle pip at the reading's head.
pub fn create_bare_ring(
    value: Option<f64>,
    max_value: f64,
    size: i32,
    color: (f64, f64, f64),
) -> gtk::DrawingArea {
    let drawing_area = gtk::DrawingArea::new();
    drawing_area.set_size_request(size, size);
    drawing_area.set_halign(gtk::Align::Center);
    drawing_area.set_valign(gtk::Align::Center);

    drawing_area.set_draw_func(move |_area, cr, width, height| {
        let w = width as f64;
        let h = height as f64;
        let cx = w / 2.0;
        let cy = h / 2.0;
        let radius = (w.min(h) / 2.0) - 10.0;
        let line_width = 7.5;

        // 240° open sweep from 150° (bottom-left) to 390° (bottom-right)
        let deg2rad = |d: f64| d * (PI / 180.0);
        let start_angle = deg2rad(150.0);
        let total_sweep = deg2rad(240.0);
        let end_angle = start_angle + total_sweep;

        cr.set_dash(&[], 0.0);
        cr.set_line_cap(gtk4::cairo::LineCap::Round);

        // 1. Recessed background groove
        cr.set_line_width(line_width);
        if crate::ui::brand_theme::is_light_mode() {
            cr.set_source_rgba(0.0, 0.0, 0.0, 0.09);
        } else {
            cr.set_source_rgba(1.0, 1.0, 1.0, 0.08);
        }
        cr.arc(cx, cy, radius, start_angle, end_angle);
        let _ = cr.stroke();

        // 2. Calibration tick pips (0%, 25%, 50%, 75%, 100%)
        let tick_count = 5;
        for i in 0..tick_count {
            let frac = i as f64 / (tick_count - 1) as f64;
            let a = start_angle + frac * total_sweep;
            let r_inner = radius - 8.0;
            let r_outer = radius - 4.5;
            cr.set_line_width(1.2);
            if crate::ui::brand_theme::is_light_mode() {
                cr.set_source_rgba(0.0, 0.0, 0.0, 0.22);
            } else {
                cr.set_source_rgba(1.0, 1.0, 1.0, 0.22);
            }
            cr.move_to(cx + a.cos() * r_inner, cy + a.sin() * r_inner);
            cr.line_to(cx + a.cos() * r_outer, cy + a.sin() * r_outer);
            let _ = cr.stroke();
        }

        // 3. Active progress arc
        if let Some(val) = value {
            let fraction = (val / max_value).clamp(0.0, 1.0);
            if fraction > 0.005 {
                let arc_end = start_angle + fraction * total_sweep;
                let (r, g, b) = color;

                // Ambient neon glow underlay
                cr.set_line_width(line_width + 5.0);
                cr.set_source_rgba(r, g, b, 0.22);
                cr.arc(cx, cy, radius, start_angle, arc_end);
                let _ = cr.stroke();

                // Crisp solid foreground arc
                cr.set_line_width(line_width);
                cr.set_source_rgba(r, g, b, 1.0);
                cr.arc(cx, cy, radius, start_angle, arc_end);
                let _ = cr.stroke();

                // Glowing head pip
                let px = cx + arc_end.cos() * radius;
                let py = cy + arc_end.sin() * radius;
                cr.set_source_rgba(1.0, 1.0, 1.0, 0.95);
                cr.arc(px, py, 2.2, 0.0, 2.0 * PI);
                let _ = cr.fill();
            }
        }
    });

    drawing_area
}
