use gtk4::prelude::*;
use gtk4::{self as gtk};
use std::f64::consts::PI;

/// Semantic "health" color for a temperature reading - green (normal), amber
/// (warm), red (hot). Returns `(r, g, b)` in `0.0..=1.0`.
pub fn temp_color(celsius: f64) -> (f64, f64, f64) {
    if celsius >= 85.0 {
        (0.93, 0.26, 0.22) // red
    } else if celsius >= 70.0 {
        (0.95, 0.62, 0.16) // amber
    } else {
        (0.24, 0.78, 0.45) // green
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

/// Just the dashed progress ring - no center number, no icon, no label under
/// it. For layouts that show the name/value as text elsewhere (e.g. a card
/// with a dedicated info column) and only want the ring as a plain visual
/// indicator on its own. `color` is the progress-arc color - pass
/// [`temp_color`] so the ring reads green/amber/red with the reading.
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
        let radius = (w.min(h) / 2.0) - 8.0;
        let line_width = 10.0;
        let dash_len = 6.0;
        let gap_len = 3.0;

        cr.set_line_width(line_width);
        cr.set_line_cap(gtk4::cairo::LineCap::Round);
        cr.set_dash(&[dash_len, gap_len], 0.0);
        cr.set_source_rgba(1.0, 1.0, 1.0, 0.08);
        cr.arc(cx, cy, radius, 0.0, 2.0 * PI);
        let _ = cr.stroke();

        if let Some(val) = value {
            let fraction = (val / max_value).clamp(0.0, 1.0);
            let start = -PI / 2.0;
            let end = start + fraction * 2.0 * PI;
            let (r, g, b) = color;
            cr.set_source_rgba(r, g, b, 1.0);
            cr.arc(cx, cy, radius, start, end);
            let _ = cr.stroke();
        }
    });

    drawing_area
}
