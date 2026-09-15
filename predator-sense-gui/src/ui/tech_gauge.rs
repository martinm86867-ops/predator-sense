//! Circular "tech gauge": a clean, static ring stack behind a text core
//! (title, sub, value, unit). Built as loose parts - each ring is its own
//! [`GaugeRing`] value and [`draw_rings`] takes any subset - so a caller can
//! compose a one- or two-ring gauge directly instead of the default stack.
//! The current default is a calm activity-ring look (a faint full-circle
//! track plus a thin accent arc), replacing the earlier seven spinning
//! decorative layers.

use gtk4::prelude::*;
use gtk4::{self as gtk, pango};

/// One RGBA color tuple (`r`, `g`, `b`, `a`), each `0.0..=1.0`.
type Rgba = (f64, f64, f64, f64);
/// One lit arc segment: `(start_deg, end_deg, color)`.
type ArcSegment = (f64, f64, Rgba);

/// One animated ring: a set of arc segments at a fixed radius band, spinning
/// at its own speed. Absolute pixel band widths on purpose, matching the
/// reference's own fixed-px mask insets - these rings stay the same visual
/// thickness whether the gauge is drawn at 90px or 320px, same as the CSS.
#[derive(Clone)]
pub struct GaugeRing {
    /// Ring's own outer edge as a fraction of the gauge's radius (e.g. 0.96).
    pub outer_frac: f64,
    /// Distance in px from that outer edge to the band's outer boundary.
    pub band_outer_px: f64,
    /// Distance in px from that outer edge to the band's inner boundary
    /// (so the visible stroke width is `band_outer_px - band_inner_px`).
    pub band_inner_px: f64,
    /// Lit arc segments in local (pre-rotation) degrees, each with its own
    /// color - `(start_deg, end_deg, rgba)`.
    pub segments: Vec<ArcSegment>,
    /// Seconds per full revolution; negative spins counter-clockwise, `0.0`
    /// keeps the ring static (used for the plain inner border).
    pub period_s: f64,
    /// Extra-wide, low-alpha stroke drawn under the ring first, approximating
    /// the reference's `drop-shadow` glow on the cyan ring. `None` for no glow.
    pub glow: Option<Rgba>,
}

fn deg_to_rad(d: f64) -> f64 {
    d * std::f64::consts::PI / 180.0
}

/// A clean, modern gauge: a faint full-circle track and a thin accent arc
/// that leaves a gap at the bottom - a calm activity-ring look instead of the
/// earlier seven spinning decorative layers. `speed_scale` is accepted for
/// API compatibility, but the current design is deliberately still.
pub fn default_rings(accent: (f64, f64, f64), _speed_scale: f64) -> Vec<GaugeRing> {
    vec![
        // Faint full-circle track.
        GaugeRing {
            outer_frac: 0.88,
            band_outer_px: 5.0,
            band_inner_px: 3.0,
            segments: vec![(0.0, 360.0, (1.0, 1.0, 1.0, 0.10))],
            period_s: 0.0,
            glow: None,
        },
        // Accent arc: 270° ring, gap at the bottom (start 135° -> end 405°).
        GaugeRing {
            outer_frac: 0.88,
            band_outer_px: 5.0,
            band_inner_px: 3.0,
            segments: vec![(135.0, 405.0, (accent.0, accent.1, accent.2, 0.95))],
            period_s: 0.0,
            glow: None,
        },
    ]
}

/// Draws every ring in `rings` centered at `(cx, cy)` with outer radius
/// `radius`, at animation phase `phase_secs` (elapsed seconds since the
/// gauge started spinning - not a frame count, so speed stays correct
/// regardless of the redraw rate).
pub fn draw_rings(
    cr: &gtk4::cairo::Context,
    cx: f64,
    cy: f64,
    radius: f64,
    rings: &[GaugeRing],
    phase_secs: f64,
) {
    cr.set_line_cap(gtk4::cairo::LineCap::Round);
    for ring in rings {
        let outer_r = radius * ring.outer_frac;
        let band_w = (ring.band_outer_px - ring.band_inner_px).max(0.5);
        let r = (outer_r - (ring.band_outer_px + ring.band_inner_px) / 2.0).max(0.5);
        let rot_deg = if ring.period_s.abs() > 0.0001 {
            (phase_secs / ring.period_s) * 360.0
        } else {
            0.0
        };

        if let Some((gr, gg, gb, ga)) = ring.glow {
            cr.set_line_width(band_w + 3.0);
            cr.set_source_rgba(gr, gg, gb, ga);
            for &(start, end, _) in &ring.segments {
                cr.new_path();
                cr.arc(
                    cx,
                    cy,
                    r,
                    deg_to_rad(start + rot_deg),
                    deg_to_rad(end + rot_deg),
                );
                let _ = cr.stroke();
            }
        }

        cr.set_line_width(band_w);
        for &(start, end, (cr_, cg_, cb_, ca_)) in &ring.segments {
            cr.set_source_rgba(cr_, cg_, cb_, ca_);
            cr.new_path();
            cr.arc(
                cx,
                cy,
                r,
                deg_to_rad(start + rot_deg),
                deg_to_rad(end + rot_deg),
            );
            let _ = cr.stroke();
        }
    }
}

/// A ready-built gauge: the ring stack behind a text core. Only `widget`
/// (for layout) and `value_label` (via [`set_value`]) are exposed for
/// mutation; the title/sub/unit lines are fixed at build time.
#[derive(Clone)]
pub struct TechGauge {
    /// Place this in your layout.
    pub widget: gtk::Widget,
    pub value_label: gtk::Label,
}

impl TechGauge {
    /// Updates the big center value text (e.g. a fresh MHz/W reading).
    pub fn set_value(&self, text: &str) {
        self.value_label.set_label(text);
    }
}

/// Parses a `"#rrggbb"` string into 16-bit-per-channel components, the form
/// [`pango::AttrColor::new_foreground`] wants.
fn hex_color(hex: &str) -> (u16, u16, u16) {
    let h = hex.trim_start_matches('#');
    let byte = |i: usize| u8::from_str_radix(&h[i..i + 2], 16).unwrap_or(0);
    let scale = |b: u8| (u16::from(b) << 8) | u16::from(b);
    (scale(byte(0)), scale(byte(2)), scale(byte(4)))
}

/// `predator`: use the app's own display font (only Bold and Regular are
/// actually registered - see `register_predator_font` in `main.rs`) for the
/// title/value lines, matching `.monitor-title`/`.monitor-temp-big`
/// elsewhere; `false` leaves the sub/unit lines on the default UI font,
/// matching plain body-text classes like `.fan-rpm`/`.stat-unit`.
///
/// Sets real [`pango::Attribute`]s (`label.set_attributes`) instead of a
/// `<span font_desc="...">` markup string - markup has to round-trip the
/// font description through `to_string()`/parse, and at the point this was
/// written the *value* line (large, bold, `predator: true`) was rendering
/// noticeably lighter than the *title* line built the exact same way -
/// attributes apply the same [`pango::FontDescription`] object directly, no
/// stringify/reparse step to lose weight or family on the way.
fn sized_label(
    text: &str,
    px: f64,
    weight: pango::Weight,
    predator: bool,
    hex: &str,
) -> gtk::Label {
    let label = gtk::Label::new(Some(text));
    let mut fd = pango::FontDescription::new();
    if predator {
        fd.set_family("Predator");
    }
    fd.set_absolute_size(px * f64::from(pango::SCALE));
    fd.set_weight(weight);

    let attrs = pango::AttrList::new();
    attrs.insert(pango::AttrFontDesc::new(&fd));
    let (r, g, b) = hex_color(hex);
    attrs.insert(pango::AttrColor::new_foreground(r, g, b));
    label.set_attributes(Some(&attrs));
    label
}

/// `size_px`: the gauge's own diameter. `accent`: the cyan ring's color -
/// pass `brand_theme::accent().bright` for the app's own cyan/orange.
/// `fast`: matches the reference's `.is-fast` modifier (~2.2x speed).
pub fn build(
    size_px: i32,
    accent: (f64, f64, f64),
    title: &str,
    sub: &str,
    value: &str,
    unit: &str,
    fast: bool,
) -> TechGauge {
    let overlay = gtk::Overlay::new();
    overlay.set_size_request(size_px, size_px);

    let rings = default_rings(accent, if fast { 0.46 } else { 1.0 });

    let da = gtk::DrawingArea::new();
    da.set_hexpand(true);
    da.set_vexpand(true);
    da.set_draw_func(move |_a, cr, w, h| {
        let cx = w as f64 / 2.0;
        let cy = h as f64 / 2.0;
        let radius = (w.min(h) as f64) / 2.0;
        draw_rings(cr, cx, cy, radius, &rings, 0.0);
    });
    overlay.set_child(Some(&da));

    let core = gtk::Box::new(gtk::Orientation::Vertical, 0);
    core.set_halign(gtk::Align::Center);
    core.set_valign(gtk::Align::Center);
    core.set_can_target(false);

    // Title/value: Predator-Bold, proportional to the gauge's own size (with
    // a floor - below ~150px the proportional size alone shrank to
    // near-unreadable) - matches `.monitor-title`/`.monitor-temp-big`
    // elsewhere. Sub/unit: plain body-text tokens already used all over the
    // app (`.fan-rpm`'s Bold 14px, `.stat-unit`'s Regular 12px) - fixed, not
    // scaled by gauge size, same as those classes are everywhere else.
    let title_px = (size_px as f64 * 0.22).max(24.0);
    let value_px = (size_px as f64 * 0.34).max(42.0);

    let title_label = sized_label(title, title_px, pango::Weight::Bold, true, "#cdd3d4");
    title_label.set_margin_bottom((size_px as f64 * 0.012) as i32);
    let sub_label = sized_label(sub, 14.0, pango::Weight::Bold, false, "#adb5b6");
    sub_label.set_margin_bottom((size_px as f64 * 0.012) as i32);
    let value_label = sized_label(value, value_px, pango::Weight::Bold, true, "#ffffff");
    let unit_label = sized_label(unit, 12.0, pango::Weight::Normal, false, "#8f9899");
    unit_label.set_margin_top((size_px as f64 * 0.018) as i32);

    core.append(&title_label);
    core.append(&sub_label);
    core.append(&value_label);
    core.append(&unit_label);
    overlay.add_overlay(&core);
    overlay.set_measure_overlay(&core, true);

    TechGauge {
        widget: overlay.upcast(),
        value_label,
    }
}
