//! Hand-drawn Cairo icons, colored by the live profile accent.
//!
//! GTK renders a file-loaded SVG once and never recolors it, so a static
//! `#00cce6` icon cannot follow the per-mode accent the rest of the app uses.
//! Drawing the same shapes with Cairo instead reads `brand_theme::accent()`
//! on every paint, so `window.rs`'s `queue_draw_recursive` on a profile change
//! recolors them for free - exactly like the sidebar chrome and header mark.

use gtk4::prelude::*;
use gtk4::{self as gtk};

/// Maps a feature icon filename (as passed to `icons/...` lookups) to its
/// draw function, or `None` for an unknown name.
pub fn icon(name: &str, size: i32) -> Option<gtk::DrawingArea> {
    let draw: fn(&gtk4::cairo::Context, f64, f64, (f64, f64, f64)) = match name {
        "cpu.svg" => draw_cpu,
        "gpu.svg" => draw_gpu,
        "memoria-ram.svg" => draw_ram,
        "ssd.svg" => draw_ssd,
        "bios.svg" => draw_bios,
        "internet.svg" => draw_wifi,
        "linux.svg" => draw_linux,
        "mode_quiet.svg" => draw_mode_quiet,
        "mode_balanced.svg" => draw_mode_balanced,
        "mode_performance.svg" => draw_mode_performance,
        "mode_turbo.svg" => draw_mode_turbo,
        "mode_eco.svg" => draw_mode_eco,
        _ => return None,
    };
    Some(accent_drawing_area(size, draw))
}

/// The Predator mark for the header bar (same geometry as `resources/logo.svg`),
/// colored by the live accent.
pub fn logo_mark(size: i32) -> gtk::DrawingArea {
    accent_drawing_area(size, draw_logo)
}

fn accent_drawing_area(
    size: i32,
    draw: fn(&gtk4::cairo::Context, f64, f64, (f64, f64, f64)),
) -> gtk::DrawingArea {
    let da = gtk::DrawingArea::new();
    da.set_size_request(size, size);
    da.set_draw_func(move |_a, cr, w, h| {
        let (r, g, b) = crate::ui::brand_theme::accent().bright;
        draw(cr, w as f64, h as f64, (r, g, b));
    });
    da
}

/// Design-to-canvas scale: every icon is authored on a 24x24 grid and scaled
/// (uniformly, centered) to whatever size the widget actually is.
fn grid(w: f64, h: f64) -> (f64, f64, f64) {
    let s = (w.min(h)) / 24.0;
    (s, (w - 24.0 * s) / 2.0, (h - 24.0 * s) / 2.0)
}

#[allow(clippy::too_many_arguments)]
fn rect(cr: &gtk4::cairo::Context, s: f64, ox: f64, oy: f64, x: f64, y: f64, w: f64, h: f64) {
    cr.rectangle(ox + x * s, oy + y * s, w * s, h * s);
}

#[allow(clippy::too_many_arguments)]
fn round(cr: &gtk4::cairo::Context, s: f64, ox: f64, oy: f64, x: f64, y: f64, w: f64, h: f64, r: f64) {
    let x = ox + x * s;
    let y = oy + y * s;
    let w = w * s;
    let h = h * s;
    let r = (r * s).min(w / 2.0).min(h / 2.0);
    cr.new_sub_path();
    cr.arc(x + w - r, y + r, r, -std::f64::consts::FRAC_PI_2, 0.0);
    cr.arc(x + w - r, y + h - r, r, 0.0, std::f64::consts::FRAC_PI_2);
    cr.arc(x + r, y + h - r, r, std::f64::consts::FRAC_PI_2, std::f64::consts::PI);
    cr.arc(x + r, y + r, r, std::f64::consts::PI, 3.0 * std::f64::consts::FRAC_PI_2);
    cr.close_path();
}

fn draw_cpu(cr: &gtk4::cairo::Context, w: f64, h: f64, c: (f64, f64, f64)) {
    let (s, ox, oy) = grid(w, h);
    cr.set_source_rgb(c.0, c.1, c.2);
    cr.set_fill_rule(gtk4::cairo::FillRule::EvenOdd);
    rect(cr, s, ox, oy, 6.5, 6.5, 11.0, 11.0); // body
    rect(cr, s, ox, oy, 10.8, 2.0, 2.4, 2.6); // top pin
    rect(cr, s, ox, oy, 10.8, 19.4, 2.4, 2.6); // bottom pin
    rect(cr, s, ox, oy, 2.0, 10.8, 2.6, 2.4); // left pin
    rect(cr, s, ox, oy, 19.4, 10.8, 2.6, 2.4); // right pin
    rect(cr, s, ox, oy, 4.6, 4.6, 2.0, 2.0); // tl pin
    rect(cr, s, ox, oy, 17.4, 4.6, 2.0, 2.0); // tr pin
    rect(cr, s, ox, oy, 4.6, 17.4, 2.0, 2.0); // bl pin
    rect(cr, s, ox, oy, 17.4, 17.4, 2.0, 2.0); // br pin
    rect(cr, s, ox, oy, 9.6, 9.6, 4.8, 4.8); // die (cutout)
    let _ = cr.fill();
}

fn draw_gpu(cr: &gtk4::cairo::Context, w: f64, h: f64, c: (f64, f64, f64)) {
    let (s, ox, oy) = grid(w, h);
    cr.set_source_rgb(c.0, c.1, c.2);
    cr.set_fill_rule(gtk4::cairo::FillRule::EvenOdd);
    round(cr, s, ox, oy, 1.5, 5.0, 21.0, 10.5, 1.5); // card
    rect(cr, s, ox, oy, 1.5, 15.0, 21.0, 3.0); // bracket
    // fan: a circular cutout with an accent hub in the middle
    let cx = ox + 12.0 * s;
    let cy = oy + 10.0 * s;
    cr.new_sub_path();
    cr.arc(cx, cy, 3.6 * s, 0.0, 2.0 * std::f64::consts::PI);
    cr.close_path();
    cr.new_sub_path();
    cr.arc(cx, cy, 1.2 * s, 0.0, 2.0 * std::f64::consts::PI);
    cr.close_path();
    let _ = cr.fill();
}

fn draw_ram(cr: &gtk4::cairo::Context, w: f64, h: f64, c: (f64, f64, f64)) {
    let (s, ox, oy) = grid(w, h);
    cr.set_source_rgb(c.0, c.1, c.2);
    cr.set_fill_rule(gtk4::cairo::FillRule::EvenOdd);
    round(cr, s, ox, oy, 2.0, 4.5, 20.0, 8.5, 1.2); // pcb
    // contact teeth
    for i in 0..8 {
        rect(cr, s, ox, oy, 3.0 + i as f64 * 2.3, 15.0, 1.1, 2.0);
    }
    // chips (cutout)
    rect(cr, s, ox, oy, 5.0, 6.5, 3.0, 2.4);
    rect(cr, s, ox, oy, 10.0, 6.5, 4.0, 2.4);
    rect(cr, s, ox, oy, 16.0, 6.5, 2.0, 2.4);
    let _ = cr.fill();
}

fn draw_ssd(cr: &gtk4::cairo::Context, w: f64, h: f64, c: (f64, f64, f64)) {
    let (s, ox, oy) = grid(w, h);
    cr.set_source_rgb(c.0, c.1, c.2);
    cr.set_fill_rule(gtk4::cairo::FillRule::EvenOdd);
    round(cr, s, ox, oy, 3.0, 7.5, 18.0, 9.0, 1.5); // drive
    rect(cr, s, ox, oy, 5.0, 9.5, 7.0, 2.2); // label (cutout)
    rect(cr, s, ox, oy, 5.0, 12.6, 9.0, 1.4); // label line (cutout)
    let cx = ox + 17.4 * s;
    let cy = oy + 9.6 * s;
    cr.new_sub_path();
    cr.arc(cx, cy, 0.5 * s, 0.0, 2.0 * std::f64::consts::PI);
    cr.new_sub_path();
    cr.arc(ox + 17.4 * s, oy + 14.4 * s, 0.5 * s, 0.0, 2.0 * std::f64::consts::PI);
    let _ = cr.fill();
}

fn draw_bios(cr: &gtk4::cairo::Context, w: f64, h: f64, c: (f64, f64, f64)) {
    let (s, ox, oy) = grid(w, h);
    cr.set_source_rgb(c.0, c.1, c.2);
    cr.set_fill_rule(gtk4::cairo::FillRule::EvenOdd);
    round(cr, s, ox, oy, 5.0, 6.0, 14.0, 12.0, 1.5); // chip
    rect(cr, s, ox, oy, 11.0, 2.0, 2.0, 2.2); // top pin
    rect(cr, s, ox, oy, 11.0, 19.8, 2.0, 2.2); // bottom pin
    rect(cr, s, ox, oy, 1.8, 11.0, 2.2, 2.0); // left pin
    rect(cr, s, ox, oy, 20.0, 11.0, 2.2, 2.0); // right pin
    rect(cr, s, ox, oy, 4.0, 4.4, 2.0, 2.0);
    rect(cr, s, ox, oy, 18.0, 4.4, 2.0, 2.0);
    rect(cr, s, ox, oy, 4.0, 17.6, 2.0, 2.0);
    rect(cr, s, ox, oy, 18.0, 17.6, 2.0, 2.0);
    rect(cr, s, ox, oy, 10.0, 6.0, 4.0, 1.6); // notch (cutout)
    rect(cr, s, ox, oy, 7.0, 9.0, 10.0, 1.6); // die line (cutout)
    let _ = cr.fill();
}

fn draw_wifi(cr: &gtk4::cairo::Context, w: f64, h: f64, c: (f64, f64, f64)) {
    let (s, ox, oy) = grid(w, h);
    cr.set_source_rgb(c.0, c.1, c.2);
    cr.set_line_width(1.9 * s);
    cr.set_line_cap(gtk4::cairo::LineCap::Round);
    let cx = ox + 12.0 * s;
    let cy = oy + 10.0 * s;
    let draw_arc = |r: f64| {
        cr.new_sub_path();
        cr.arc(cx, cy, r * s, std::f64::consts::PI + 0.55, 2.0 * std::f64::consts::PI - 0.55);
        let _ = cr.stroke();
    };
    draw_arc(9.0);
    draw_arc(6.2);
    draw_arc(3.4);
    // dot
    cr.new_sub_path();
    cr.arc(cx, oy + 17.4 * s, 1.25 * s, 0.0, 2.0 * std::f64::consts::PI);
    let _ = cr.fill();
}

fn draw_linux(cr: &gtk4::cairo::Context, w: f64, h: f64, c: (f64, f64, f64)) {
    let (s, ox, oy) = grid(w, h);
    cr.set_source_rgb(c.0, c.1, c.2);
    cr.set_fill_rule(gtk4::cairo::FillRule::EvenOdd);
    round(cr, s, ox, oy, 2.5, 4.0, 19.0, 16.0, 2.0); // terminal window
    rect(cr, s, ox, oy, 5.5, 7.5, 4.5, 1.5); // title bar line (cutout)
    rect(cr, s, ox, oy, 5.5, 14.2, 9.0, 1.5); // prompt line (cutout)
    let _ = cr.fill();
}

/// The Predator mark from the user's SVG (two mirrored angular glyphs),
/// scaled to fit the canvas. Original path coordinates are in a 700x900 box.
const LOGO_LEFT: [(f64, f64); 13] = [
    (2.0, 0.5),
    (1.0, 512.0),
    (150.0, 659.0),
    (150.5, 533.0),
    (102.5, 478.0),
    (102.5, 360.0),
    (179.0, 473.0),
    (181.0, 898.0),
    (250.0, 730.0),
    (331.5, 643.5),
    (331.0, 115.5),
    (239.5, 354.0),
    (88.5, 205.0),
];

const LOGO_RIGHT: [(f64, f64); 13] = [
    (698.0, 0.5),
    (699.0, 512.0),
    (550.0, 659.0),
    (549.5, 533.0),
    (597.5, 478.0),
    (597.5, 360.0),
    (521.0, 473.0),
    (519.0, 898.0),
    (450.0, 730.0),
    (368.5, 643.5),
    (369.0, 115.5),
    (460.5, 354.0),
    (611.5, 205.0),
];

fn draw_logo(cr: &gtk4::cairo::Context, w: f64, h: f64, c: (f64, f64, f64)) {
    // Original box (1..699 x 0.5..898), baked translate(139,50) scale(0.89).
    let bw = 699.0 - 1.0;
    let bh = 898.0 - 0.5;
    let scale = (w / bw).min(h / bh);
    let ox = (w - bw * scale) / 2.0;
    let oy = (h - bh * scale) / 2.0;
    let map = |x: f64, y: f64| (ox + (x - 1.0) * scale, oy + (y - 0.5) * scale);

    cr.set_source_rgb(c.0, c.1, c.2);
    for pts in [&LOGO_LEFT[..], &LOGO_RIGHT[..]] {
        let (x0, y0) = map(pts[0].0, pts[0].1);
        cr.move_to(x0, y0);
        for &(x, y) in &pts[1..] {
            let (px, py) = map(x, y);
            cr.line_to(px, py);
        }
        cr.close_path();
    }
    let _ = cr.fill();
}

fn draw_mode_quiet(cr: &gtk4::cairo::Context, w: f64, h: f64, c: (f64, f64, f64)) {
    let (s, ox, oy) = grid(w, h);
    cr.set_source_rgb(c.0, c.1, c.2);
    cr.set_line_width(2.0 * s);
    cr.set_line_cap(gtk4::cairo::LineCap::Round);
    // Leaf / feather minimal profile curve
    let cx = ox + 12.0 * s;
    let cy = oy + 12.0 * s;
    cr.arc(cx, cy, 7.0 * s, 0.0, std::f64::consts::PI * 1.5);
    let _ = cr.stroke();
    cr.arc(cx, cy, 3.5 * s, std::f64::consts::PI * 0.5, 2.0 * std::f64::consts::PI);
    let _ = cr.stroke();
}

fn draw_mode_balanced(cr: &gtk4::cairo::Context, w: f64, h: f64, c: (f64, f64, f64)) {
    let (s, ox, oy) = grid(w, h);
    cr.set_source_rgb(c.0, c.1, c.2);
    cr.set_line_width(2.0 * s);
    cr.set_line_cap(gtk4::cairo::LineCap::Round);
    // Twin balanced tachometer arcs
    let cx = ox + 12.0 * s;
    let cy = oy + 12.0 * s;
    cr.arc(cx, cy, 7.5 * s, std::f64::consts::PI * 0.75, std::f64::consts::PI * 2.25);
    let _ = cr.stroke();
    cr.move_to(cx, cy);
    cr.line_to(cx + 4.5 * s, cy - 4.5 * s);
    let _ = cr.stroke();
}

fn draw_mode_performance(cr: &gtk4::cairo::Context, w: f64, h: f64, c: (f64, f64, f64)) {
    let (s, ox, oy) = grid(w, h);
    cr.set_source_rgb(c.0, c.1, c.2);
    // Double chevron up / speed indicator
    cr.set_line_width(2.5 * s);
    cr.set_line_cap(gtk4::cairo::LineCap::Round);
    cr.set_line_join(gtk4::cairo::LineJoin::Round);
    cr.move_to(ox + 4.0 * s, oy + 13.0 * s);
    cr.line_to(ox + 12.0 * s, oy + 6.0 * s);
    cr.line_to(ox + 20.0 * s, oy + 13.0 * s);
    let _ = cr.stroke();
    cr.move_to(ox + 4.0 * s, oy + 19.0 * s);
    cr.line_to(ox + 12.0 * s, oy + 12.0 * s);
    cr.line_to(ox + 20.0 * s, oy + 19.0 * s);
    let _ = cr.stroke();
}

fn draw_mode_turbo(cr: &gtk4::cairo::Context, w: f64, h: f64, c: (f64, f64, f64)) {
    let (s, ox, oy) = grid(w, h);
    cr.set_source_rgb(c.0, c.1, c.2);
    // Lightning bolt / turbo charge
    cr.move_to(ox + 13.5 * s, oy + 2.0 * s);
    cr.line_to(ox + 6.0 * s, oy + 12.5 * s);
    cr.line_to(ox + 12.0 * s, oy + 12.5 * s);
    cr.line_to(ox + 10.5 * s, oy + 22.0 * s);
    cr.line_to(ox + 18.0 * s, oy + 10.5 * s);
    cr.line_to(ox + 12.5 * s, oy + 10.5 * s);
    cr.close_path();
    let _ = cr.fill();
}

fn draw_mode_eco(cr: &gtk4::cairo::Context, w: f64, h: f64, c: (f64, f64, f64)) {
    let (s, ox, oy) = grid(w, h);
    cr.set_source_rgb(c.0, c.1, c.2);
    cr.set_line_width(2.0 * s);
    cr.set_line_cap(gtk4::cairo::LineCap::Round);
    // Minimal battery eco cell
    round(cr, s, ox, oy, 6.0, 5.0, 12.0, 15.0, 2.0);
    let _ = cr.stroke();
    rect(cr, s, ox, oy, 9.5, 2.5, 5.0, 2.0);
    let _ = cr.fill();
    rect(cr, s, ox, oy, 8.5, 12.0, 7.0, 6.0);
    let _ = cr.fill();
}

pub fn draw_nav_by_name(name: &str, cr: &gtk4::cairo::Context, w: f64, h: f64, c: (f64, f64, f64)) {
    match name {
        "home" => draw_nav_home(cr, w, h, c),
        "cooling" => draw_nav_cooling(cr, w, h, c),
        "usage" => draw_nav_usage(cr, w, h, c),
        "network" => draw_wifi(cr, w, h, c),
        "lighting" => draw_nav_lighting(cr, w, h, c),
        "battery" => draw_nav_battery(cr, w, h, c),
        "tools" => draw_nav_tools(cr, w, h, c),
        "settings" => draw_nav_settings(cr, w, h, c),
        _ => draw_logo(cr, w, h, c),
    }
}

fn draw_nav_home(cr: &gtk4::cairo::Context, w: f64, h: f64, c: (f64, f64, f64)) {
    let (s, ox, oy) = grid(w, h);
    cr.set_source_rgb(c.0, c.1, c.2);
    cr.set_line_width(2.0 * s);
    cr.set_line_cap(gtk4::cairo::LineCap::Round);
    cr.set_line_join(gtk4::cairo::LineJoin::Round);

    // Roof
    cr.move_to(ox + 3.0 * s, oy + 11.5 * s);
    cr.line_to(ox + 12.0 * s, oy + 3.5 * s);
    cr.line_to(ox + 21.0 * s, oy + 11.5 * s);
    let _ = cr.stroke();

    // Body
    cr.move_to(ox + 5.5 * s, oy + 10.5 * s);
    cr.line_to(ox + 5.5 * s, oy + 20.0 * s);
    cr.line_to(ox + 18.5 * s, oy + 20.0 * s);
    cr.line_to(ox + 18.5 * s, oy + 10.5 * s);
    let _ = cr.stroke();

    // Door cutout
    round(cr, s, ox, oy, 10.0, 14.5, 4.0, 5.5, 1.0);
    let _ = cr.stroke();
}

fn draw_nav_cooling(cr: &gtk4::cairo::Context, w: f64, h: f64, c: (f64, f64, f64)) {
    let (s, ox, oy) = grid(w, h);
    cr.set_source_rgb(c.0, c.1, c.2);
    let cx = ox + 12.0 * s;
    let cy = oy + 12.0 * s;

    // Center hub
    cr.arc(cx, cy, 2.6 * s, 0.0, 2.0 * std::f64::consts::PI);
    let _ = cr.fill();

    // 4 Curved Fan blades
    cr.set_line_width(2.2 * s);
    cr.set_line_cap(gtk4::cairo::LineCap::Round);
    for i in 0..4 {
        let angle = i as f64 * std::f64::consts::FRAC_PI_2;
        let x1 = cx + angle.cos() * 3.5 * s;
        let y1 = cy + angle.sin() * 3.5 * s;
        let x2 = cx + (angle + 0.55).cos() * 8.5 * s;
        let y2 = cy + (angle + 0.55).sin() * 8.5 * s;
        cr.move_to(x1, y1);
        cr.curve_to(
            cx + (angle + 0.2).cos() * 6.0 * s,
            cy + (angle + 0.2).sin() * 6.0 * s,
            cx + (angle + 0.4).cos() * 7.5 * s,
            cy + (angle + 0.4).sin() * 7.5 * s,
            x2,
            y2,
        );
        let _ = cr.stroke();
    }
}

fn draw_nav_usage(cr: &gtk4::cairo::Context, w: f64, h: f64, c: (f64, f64, f64)) {
    let (s, ox, oy) = grid(w, h);
    cr.set_source_rgb(c.0, c.1, c.2);
    round(cr, s, ox, oy, 4.5, 12.0, 3.5, 8.5, 1.2);
    let _ = cr.fill();
    round(cr, s, ox, oy, 10.25, 5.0, 3.5, 15.5, 1.2);
    let _ = cr.fill();
    round(cr, s, ox, oy, 16.0, 8.5, 3.5, 12.0, 1.2);
    let _ = cr.fill();
}

fn draw_nav_lighting(cr: &gtk4::cairo::Context, w: f64, h: f64, c: (f64, f64, f64)) {
    let (s, ox, oy) = grid(w, h);
    cr.set_source_rgb(c.0, c.1, c.2);
    cr.set_line_width(1.8 * s);
    cr.set_line_cap(gtk4::cairo::LineCap::Round);
    round(cr, s, ox, oy, 3.0, 6.0, 18.0, 12.5, 2.0);
    let _ = cr.stroke();
    for r in 0..2 {
        for col in 0..4 {
            let kx = ox + (5.5 + col as f64 * 3.6) * s;
            let ky = oy + (9.0 + r as f64 * 4.0) * s;
            cr.arc(kx, ky, 0.9 * s, 0.0, 2.0 * std::f64::consts::PI);
            let _ = cr.fill();
        }
    }
}

fn draw_nav_battery(cr: &gtk4::cairo::Context, w: f64, h: f64, c: (f64, f64, f64)) {
    let (s, ox, oy) = grid(w, h);
    cr.set_source_rgb(c.0, c.1, c.2);
    cr.set_line_width(1.8 * s);
    cr.set_line_cap(gtk4::cairo::LineCap::Round);
    round(cr, s, ox, oy, 3.0, 7.0, 15.0, 10.0, 2.0);
    let _ = cr.stroke();
    round(cr, s, ox, oy, 18.5, 10.0, 2.0, 4.0, 0.8);
    let _ = cr.fill();
    cr.move_to(ox + 11.5 * s, oy + 8.5 * s);
    cr.line_to(ox + 8.5 * s, oy + 12.5 * s);
    cr.line_to(ox + 11.0 * s, oy + 12.5 * s);
    cr.line_to(ox + 9.5 * s, oy + 15.5 * s);
    cr.line_to(ox + 13.5 * s, oy + 11.5 * s);
    cr.line_to(ox + 11.0 * s, oy + 11.5 * s);
    cr.close_path();
    let _ = cr.fill();
}

fn draw_nav_tools(cr: &gtk4::cairo::Context, w: f64, h: f64, c: (f64, f64, f64)) {
    let (s, ox, oy) = grid(w, h);
    cr.set_source_rgb(c.0, c.1, c.2);
    cr.set_line_width(2.0 * s);
    cr.set_line_cap(gtk4::cairo::LineCap::Round);
    cr.move_to(ox + 4.5 * s, oy + 19.5 * s);
    cr.line_to(ox + 19.5 * s, oy + 4.5 * s);
    let _ = cr.stroke();
    cr.move_to(ox + 19.5 * s, oy + 19.5 * s);
    cr.line_to(ox + 4.5 * s, oy + 4.5 * s);
    let _ = cr.stroke();
    cr.arc(ox + 4.5 * s, oy + 4.5 * s, 1.8 * s, 0.0, 2.0 * std::f64::consts::PI);
    let _ = cr.fill();
    cr.arc(ox + 19.5 * s, oy + 4.5 * s, 1.8 * s, 0.0, 2.0 * std::f64::consts::PI);
    let _ = cr.fill();
}

fn draw_nav_settings(cr: &gtk4::cairo::Context, w: f64, h: f64, c: (f64, f64, f64)) {
    let (s, ox, oy) = grid(w, h);
    cr.set_source_rgb(c.0, c.1, c.2);
    let cx = ox + 12.0 * s;
    let cy = oy + 12.0 * s;
    cr.set_line_width(2.2 * s);
    cr.arc(cx, cy, 6.2 * s, 0.0, 2.0 * std::f64::consts::PI);
    let _ = cr.stroke();
    cr.set_line_width(2.4 * s);
    cr.set_line_cap(gtk4::cairo::LineCap::Round);
    for i in 0..6 {
        let a = i as f64 * std::f64::consts::PI / 3.0;
        let x1 = cx + a.cos() * 6.0 * s;
        let y1 = cy + a.sin() * 6.0 * s;
        let x2 = cx + a.cos() * 8.8 * s;
        let y2 = cy + a.sin() * 8.8 * s;
        cr.move_to(x1, y1);
        cr.line_to(x2, y2);
        let _ = cr.stroke();
    }
    cr.arc(cx, cy, 2.2 * s, 0.0, 2.0 * std::f64::consts::PI);
    let _ = cr.fill();
}
