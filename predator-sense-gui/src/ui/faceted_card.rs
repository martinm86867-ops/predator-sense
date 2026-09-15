use gtk4::prelude::*;
use gtk4::{self as gtk};
use std::cell::Cell;
use std::rc::Rc;

/// A modern surface card: a rounded rectangle with a layered dark fill and a
/// subtle accent-tinted border. This replaced the earlier cut-corner
/// "tech-panel" polygon with its chevron/slash decorations - the rounded
/// form reads cleaner and stays legible at any size, with no geometry that
/// only works at one fixed aspect ratio.
///
/// Reusable by design: color and title are caller-supplied, so the same
/// shape frames a temperature gauge today and some other section tomorrow.
#[derive(Clone)]
pub struct FacetedCard {
    /// Place this in your layout.
    pub widget: gtk::Widget,
    /// Append your own content into this.
    pub content: gtk::Box,
    bg: gtk::DrawingArea,
    accent: Rc<Cell<(f64, f64, f64)>>,
}

impl FacetedCard {
    /// Recolors an already-built card in place (e.g. a selection state
    /// changing at runtime) - redraws immediately, no rebuild needed.
    pub fn set_accent(&self, accent: (f64, f64, f64)) {
        self.accent.set(accent);
        self.bg.queue_draw();
    }
}

/// Builds an empty card. `accent` is the frame/tint color as `(r, g, b)` in
/// `0.0..=1.0` - pass `brand_theme::accent().bright` for the app's own
/// cyan/orange, or any other color for a card that must stand out. `title`
/// draws a small label near the top-left corner; pass `None` for a plain
/// card with no built-in heading.
pub fn build(accent: (f64, f64, f64), title: Option<&str>) -> FacetedCard {
    let overlay = gtk::Overlay::new();
    overlay.add_css_class("faceted-card");

    let accent_cell = Rc::new(Cell::new(accent));
    let bg = gtk::DrawingArea::new();
    bg.set_hexpand(true);
    bg.set_vexpand(true);
    {
        let accent_cell = accent_cell.clone();
        bg.set_draw_func(move |_area, cr, w, h| {
            draw_background(cr, w as f64, h as f64, accent_cell.get());
        });
    }
    overlay.set_child(Some(&bg));

    let content = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    content.set_hexpand(true);
    content.set_vexpand(true);
    content.set_margin_top(if title.is_some() { 34 } else { 26 });
    content.set_margin_bottom(22);
    content.set_margin_start(24);
    content.set_margin_end(24);
    overlay.add_overlay(&content);
    overlay.set_measure_overlay(&content, true);

    if let Some(text) = title {
        let label = gtk::Label::new(Some(text));
        label.add_css_class("faceted-card-title");
        label.set_halign(gtk::Align::Start);
        label.set_valign(gtk::Align::Start);
        label.set_margin_start(18);
        label.set_margin_top(8);
        label.set_can_target(false);
        overlay.add_overlay(&label);
    }

    FacetedCard {
        widget: overlay.upcast(),
        content,
        bg,
        accent: accent_cell,
    }
}

/// The compact card used for smaller tiles (e.g. the Dashboard's spec
/// cards): same rounded surface, slightly smaller radius.
pub fn build_simple(accent: (f64, f64, f64), title: Option<&str>) -> FacetedCard {
    let overlay = gtk::Overlay::new();
    overlay.add_css_class("faceted-card-simple");

    let accent_cell = Rc::new(Cell::new(accent));
    let bg = gtk::DrawingArea::new();
    bg.set_hexpand(true);
    bg.set_vexpand(true);
    {
        let accent_cell = accent_cell.clone();
        bg.set_draw_func(move |_area, cr, w, h| {
            draw_background_simple(cr, w as f64, h as f64, accent_cell.get());
        });
    }
    overlay.set_child(Some(&bg));

    let content = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    content.set_hexpand(true);
    content.set_vexpand(true);
    content.set_margin_top(if title.is_some() { 26 } else { 14 });
    content.set_margin_bottom(14);
    content.set_margin_start(14);
    content.set_margin_end(18);
    overlay.add_overlay(&content);
    overlay.set_measure_overlay(&content, true);

    if let Some(text) = title {
        let label = gtk::Label::new(Some(text));
        label.add_css_class("faceted-card-title");
        label.set_halign(gtk::Align::Start);
        label.set_valign(gtk::Align::Start);
        label.set_margin_start(12);
        label.set_margin_top(6);
        label.set_can_target(false);
        overlay.add_overlay(&label);
    }

    FacetedCard {
        widget: overlay.upcast(),
        content,
        bg,
        accent: accent_cell,
    }
}

const RADIUS: f64 = 14.0;
const RADIUS_SIMPLE: f64 = 12.0;

/// Traces a rounded rectangle into the current path (no fill/stroke).
fn rounded_rect(cr: &gtk4::cairo::Context, w: f64, h: f64, r: f64) {
    let r = r.min(w / 2.0).min(h / 2.0);
    if r <= 0.0 {
        cr.rectangle(0.0, 0.0, w, h);
        return;
    }
    use std::f64::consts::{FRAC_PI_2, PI};
    cr.move_to(r, 0.0);
    cr.line_to(w - r, 0.0);
    cr.arc(w - r, r, r, -FRAC_PI_2, 0.0);
    cr.line_to(w, h - r);
    cr.arc(w - r, h - r, r, 0.0, FRAC_PI_2);
    cr.line_to(r, h);
    cr.arc(r, h - r, r, FRAC_PI_2, PI);
    cr.line_to(0.0, r);
    cr.arc(r, r, r, PI, 3.0 * FRAC_PI_2);
    cr.close_path();
}

/// Fills a rounded card with a subtle vertical gradient (dark surface with a
/// faint accent tint) and traces a 1px accent-tinted border on top.
fn draw_card(cr: &gtk4::cairo::Context, w: f64, h: f64, accent: (f64, f64, f64), radius: f64) {
    if w <= 0.0 || h <= 0.0 {
        return;
    }
    let (r, g, b) = accent;

    rounded_rect(cr, w, h, radius);

    // Layered surface: slightly lighter at the top, darker at the bottom.
    let fill = gtk4::cairo::LinearGradient::new(0.0, 0.0, 0.0, h);
    fill.add_color_stop_rgba(0.0, 0.094 + r * 0.030, 0.118 + g * 0.030, 0.153 + b * 0.030, 1.0);
    fill.add_color_stop_rgba(1.0, 0.078 + r * 0.020, 0.098 + g * 0.020, 0.129 + b * 0.020, 1.0);
    let _ = cr.set_source(&fill);
    let _ = cr.fill_preserve();

    // 1px accent-tinted border - subtle, keeps the brand color without a glow.
    cr.set_source_rgba(r, g, b, 0.16);
    cr.set_line_width(1.0);
    let _ = cr.stroke();
}

fn draw_background_simple(cr: &gtk4::cairo::Context, w: f64, h: f64, accent: (f64, f64, f64)) {
    draw_card(cr, w, h, accent, RADIUS_SIMPLE);
}

fn draw_background(cr: &gtk4::cairo::Context, w: f64, h: f64, accent: (f64, f64, f64)) {
    draw_card(cr, w, h, accent, RADIUS);
}
