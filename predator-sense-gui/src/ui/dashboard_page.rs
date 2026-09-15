use gtk4::prelude::*;
use gtk4::{self as gtk, glib};

use crate::hardware::sysinfo::{self, SystemInfo};

/// Dashboard principal: hero com foto do notebook + especificações técnicas.
pub fn build() -> gtk::ScrolledWindow {
    let scroll = gtk::ScrolledWindow::new();
    scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    scroll.set_hexpand(true);
    scroll.set_vexpand(true);

    let info = sysinfo::read_system_info();

    let page = gtk::Box::new(gtk::Orientation::Vertical, 16);
    page.set_margin_top(18);
    page.set_margin_bottom(18);
    page.set_margin_start(24);
    page.set_margin_end(24);

    // === Hero header: foto + nome/modelo ===
    let hero_card =
        crate::ui::faceted_card::build_simple(crate::ui::brand_theme::accent().bright, None);
    let hero = hero_card.content;
    hero.set_spacing(24);
    hero.set_halign(gtk::Align::Fill);

    // No laptop photo here - we already know what the hardware looks like.
    // The hero is identity text; live contextual instruments live below it.
    let hero_info = gtk::Box::new(gtk::Orientation::Vertical, 6);
    hero_info.set_valign(gtk::Align::Center);
    hero_info.set_hexpand(true);

    let vendor = gtk::Label::new(Some(&info.vendor));
    vendor.add_css_class("dashboard-vendor");
    vendor.set_halign(gtk::Align::Start);
    hero_info.append(&vendor);

    let product = gtk::Label::new(Some(&info.product_name));
    product.add_css_class("dashboard-product");
    product.set_halign(gtk::Align::Start);
    product.set_wrap(true);
    hero_info.append(&product);

    let summary = gtk::Label::new(Some(&build_short_summary(&info)));
    summary.add_css_class("dashboard-summary");
    summary.set_halign(gtk::Align::Start);
    summary.set_wrap(true);
    hero_info.append(&summary);

    hero.append(&hero_info);
    page.append(&hero_card.widget);

    // === Live instrument cluster: color-coded readings at a glance ===
    page.append(&build_instrument_cluster());

    // === Specs grid ===
    let specs_title = gtk::Label::new(Some(crate::i18n::t("dashboard_specs")));
    specs_title.add_css_class("section-title");
    specs_title.set_halign(gtk::Align::Start);
    specs_title.set_margin_top(8);
    page.append(&specs_title);

    let grid = gtk::Grid::new();
    grid.set_column_spacing(12);
    grid.set_row_spacing(12);
    grid.set_column_homogeneous(true);
    grid.set_margin_top(6);

    let cpu_detail = if info.cpu_cores > 0 {
        crate::i18n::tf(
            "cpu_full_spec",
            &[
                &info.cpu_model,
                &info.cpu_cores.to_string(),
                &info.cpu_threads.to_string(),
                &format!("{:.2}", info.cpu_max_freq_mhz as f64 / 1000.0),
            ],
        )
    } else {
        info.cpu_model.clone()
    };

    let nvidia_available = crate::hardware::nvidia::is_available();
    let initial_gpu_status = nvidia_available.then(|| {
        if crate::hardware::nvidia::live_query_is_safe() {
            crate::i18n::t("gpu_loading_live")
        } else {
            crate::i18n::t("gpu_suspended_static")
        }
    });
    let gpu_detail = format_gpu_detail(
        &info.gpu_name,
        info.gpu_vram_mb,
        &info.gpu_driver,
        initial_gpu_status,
    );

    let ram_detail = if info.ram_total_gb > 0.0 {
        if info.ram_type.is_empty() {
            format!("{:.0} GB total", info.ram_total_gb)
        } else {
            format!("{:.0} GB · {}", info.ram_total_gb, info.ram_type)
        }
    } else {
        "—".into()
    };

    let storage_detail = if info.storage.is_empty() {
        "—".into()
    } else {
        // Limit to 2 disks so a many-disk machine doesn't make this card tall
        // and misalign the grid; append a "+N" summary for the rest.
        let mut lines: Vec<String> = info
            .storage
            .iter()
            .take(2)
            .map(|s| format!("{} · {:.0} GB · {}", s.model.trim(), s.size_gb, s.kind))
            .collect();
        if info.storage.len() > 2 {
            lines.push(format!("+{} ...", info.storage.len() - 2));
        }
        lines.join("\n")
    };

    let net_detail = if info.net_interface.is_empty() {
        crate::i18n::t("no_active_interface").to_string()
    } else {
        let ip = crate::ui::network_page::local_ip();
        format!(
            "{} · {}\n{}\n{} {}",
            info.net_type,
            info.net_interface,
            info.net_mac,
            crate::i18n::t("local_ip"),
            ip.as_deref().unwrap_or("--"),
        )
    };

    let os_detail = format!("{}\nKernel {}", info.os_pretty, info.kernel);

    let bios_detail = if info.bios_version.is_empty() {
        "—".into()
    } else {
        format!("BIOS {}", info.bios_version)
    };

    let cards = [
        ("CPU", "💻", Some("cpu.svg"), cpu_detail),
        ("GPU", "🎮", Some("gpu.svg"), gpu_detail),
        (
            crate::i18n::t("memory"),
            "🧠",
            Some("memoria-ram.svg"),
            ram_detail,
        ),
        (
            crate::i18n::t("storage"),
            "💾",
            Some("ssd.svg"),
            storage_detail,
        ),
        (
            crate::i18n::t("network"),
            "🌐",
            Some("internet.svg"),
            net_detail,
        ),
        (
            crate::i18n::t("system_os"),
            "🐧",
            Some("linux.svg"),
            os_detail,
        ),
        ("BIOS", "⚙", Some("bios.svg"), bios_detail),
    ];

    let custom_icons = crate::config::load_app_config().custom_icons_enabled;
    let mut gpu_value_label = None;
    for (i, (title, icon, image, value)) in cards.iter().enumerate() {
        let image = if custom_icons { *image } else { None };
        let (card, value_label) = create_spec_card(icon, image, title, value);
        if *title == "GPU" {
            gpu_value_label = Some(value_label);
        }
        let col = (i % 2) as i32;
        let row = (i / 2) as i32;
        grid.attach(&card, col, row, 1, 1);
    }

    page.append(&grid);

    // Passive Dashboard refreshes never runtime-resume a suspended dGPU.
    // They can consume an already-live sample (including one loaded after the
    // user explicitly opens the GPU page), or query an already-active device
    // on the shared background path without blocking GTK.
    if nvidia_available {
        if let Some(gpu_value_label) = gpu_value_label {
            let map_label = gpu_value_label.clone();
            scroll.connect_map(move |_| refresh_gpu_detail(&map_label));

            let scroll = scroll.clone();
            glib::timeout_add_seconds_local(4, move || {
                if scroll.is_mapped() {
                    refresh_gpu_detail(&gpu_value_label);
                }
                glib::ControlFlow::Continue
            });
        }
    }

    scroll.set_child(Some(&page));
    scroll
}

/// Which live metric an instrument shows - determines its refresh source and
/// health color.
#[derive(Clone, Copy)]
enum InstrumentKind {
    CpuTemp,
    GpuTemp,
    CpuFan,
    GpuFan,
}

/// A compact, color-coded live reading (label + big value + unit) refreshing
/// on a timer - the "instrument cluster" the dashboard leads with, in place
/// of the old static laptop photo.
fn build_instrument_cluster() -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    row.set_homogeneous(true);
    row.set_hexpand(true);

    let specs: [(&str, &str, InstrumentKind); 4] = [
        ("CPU", "°C", InstrumentKind::CpuTemp),
        ("GPU", "°C", InstrumentKind::GpuTemp),
        ("CPU FAN", "RPM", InstrumentKind::CpuFan),
        ("GPU FAN", "RPM", InstrumentKind::GpuFan),
    ];

    let mut values: Vec<(gtk::Label, InstrumentKind)> = Vec::new();
    for (label, unit, kind) in specs {
        let card = gtk::Box::new(gtk::Orientation::Vertical, 4);
        card.add_css_class("spec-card");
        card.set_hexpand(true);
        card.set_valign(gtk::Align::Center);

        let l = gtk::Label::new(Some(label));
        l.add_css_class("spec-title");
        l.set_halign(gtk::Align::Start);

        let v = gtk::Label::new(Some("--"));
        v.add_css_class("monitor-temp-big");
        v.set_halign(gtk::Align::Start);

        let u = gtk::Label::new(Some(unit));
        u.add_css_class("spec-title");
        u.set_halign(gtk::Align::Start);

        card.append(&l);
        card.append(&v);
        card.append(&u);
        row.append(&card);
        values.push((v, kind));
    }

    refresh_instruments(&values);

    let row_c = row.clone();
    glib::timeout_add_seconds_local(4, move || {
        if row_c.is_mapped() {
            refresh_instruments(&values);
        }
        glib::ControlFlow::Continue
    });

    row
}

fn refresh_instruments(values: &[(gtk::Label, InstrumentKind)]) {
    let sensors = crate::hardware::sensors::read_all_sensors();
    for (label, kind) in values {
        let (text, color) = match kind {
            InstrumentKind::CpuTemp => temp_reading(sensors.cpu_temp),
            InstrumentKind::GpuTemp => temp_reading(sensors.gpu_temp),
            InstrumentKind::CpuFan => fan_reading(sensors.cpu_fan_rpm),
            InstrumentKind::GpuFan => fan_reading(sensors.gpu_fan_rpm),
        };
        label.set_markup(&format!("<span foreground=\"{color}\">{text}</span>"));
    }
}

fn temp_reading(t: Option<f64>) -> (String, String) {
    match t {
        Some(v) => (
            format!("{v:.0}"),
            crate::ui::gauge_widget::temp_color_hex(v),
        ),
        None => ("--".to_string(), "#8b95a3".to_string()),
    }
}

fn fan_reading(rpm: Option<u32>) -> (String, String) {
    (
        rpm.map(|r| r.to_string()).unwrap_or_else(|| "--".into()),
        "#aeb7c2".to_string(),
    )
}

fn refresh_gpu_detail(label: &gtk::Label) {
    let metrics = crate::hardware::gpu::read_gpu_metrics();
    let status = if metrics.live {
        None
    } else {
        Some(match crate::hardware::gpu::live_data_state() {
            crate::hardware::gpu::GpuLiveState::Loading => crate::i18n::t("gpu_loading_live"),
            crate::hardware::gpu::GpuLiveState::Unavailable => {
                crate::i18n::t("gpu_live_unavailable")
            }
            crate::hardware::gpu::GpuLiveState::Static => {
                if crate::hardware::nvidia::live_query_is_safe() {
                    crate::i18n::t("gpu_loading_live")
                } else {
                    crate::i18n::t("gpu_suspended_static")
                }
            }
            crate::hardware::gpu::GpuLiveState::Live => crate::i18n::t("gpu_live_unavailable"),
        })
    };
    label.set_text(&format_gpu_detail(
        &metrics.name,
        metrics.vram_total_mb,
        &metrics.driver,
        status,
    ));
}

fn format_gpu_detail(name: &str, vram_mb: u32, driver: &str, status: Option<&str>) -> String {
    let mut metadata = Vec::new();
    if vram_mb > 0 {
        metadata.push(format!("{:.0} GB VRAM", vram_mb as f64 / 1024.0));
    }
    if !driver.is_empty() {
        metadata.push(format!("Driver {driver}"));
    }
    if let Some(status) = status {
        metadata.push(status.to_string());
    }
    if metadata.is_empty() {
        name.to_string()
    } else {
        format!("{name}\n{}", metadata.join(" · "))
    }
}

/// Reusable "supported features" FlowBox (used in Settings). Auto-detected for
/// the current model via capabilities.
pub fn build_features_flow() -> gtk::FlowBox {
    let caps = crate::hardware::capabilities::get();
    let feat_flow = gtk::FlowBox::new();
    feat_flow.set_selection_mode(gtk::SelectionMode::None);
    feat_flow.set_max_children_per_line(4);
    feat_flow.set_min_children_per_line(2);
    feat_flow.set_column_spacing(8);
    feat_flow.set_row_spacing(8);
    feat_flow.set_margin_top(6);
    feat_flow.set_homogeneous(true);

    let features: [(&str, bool); 8] = [
        (crate::i18n::t("feat_rgb"), caps.rgb),
        (crate::i18n::t("feat_cover_logo"), caps.cover_logo),
        (crate::i18n::t("feat_fan_rpm"), caps.fan_rpm),
        (crate::i18n::t("feat_fan_pwm"), caps.fan_pwm),
        (crate::i18n::t("feat_profiles"), caps.performance_profiles),
        (crate::i18n::t("feat_ec"), caps.ec),
        (crate::i18n::t("feat_gpu"), caps.nvidia_gpu),
        (crate::i18n::t("feat_battery"), caps.battery_charge_cap()),
    ];
    for (name, ok) in features {
        feat_flow.insert(&make_feature_chip(name, ok), -1);
    }
    feat_flow
}

fn make_feature_chip(name: &str, supported: bool) -> gtk::Box {
    let chip = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    chip.add_css_class("feature-chip");
    chip.add_css_class(if supported {
        "feature-on"
    } else {
        "feature-off"
    });
    chip.set_margin_top(2);
    chip.set_margin_bottom(2);
    let icon = gtk::Label::new(Some(if supported { "✓" } else { "—" }));
    icon.add_css_class("feature-icon");
    let label = gtk::Label::new(Some(name));
    label.add_css_class("feature-label");
    label.set_halign(gtk::Align::Start);
    label.set_hexpand(true);
    label.set_xalign(0.0);
    chip.append(&icon);
    chip.append(&label);
    chip
}

fn build_short_summary(info: &SystemInfo) -> String {
    let mut parts: Vec<String> = Vec::new();
    if info.cpu_cores > 0 {
        parts.push(crate::i18n::tf(
            "cores_threads_short",
            &[&info.cpu_cores.to_string(), &info.cpu_threads.to_string()],
        ));
    }
    if info.ram_total_gb > 0.0 {
        parts.push(format!("{:.0} GB RAM", info.ram_total_gb));
    }
    if !info.gpu_name.is_empty() && info.gpu_name != crate::i18n::t("unknown") {
        parts.push(info.gpu_name.clone());
    }
    parts.join(" · ")
}

fn create_spec_card(
    icon: &str,
    image: Option<&str>,
    title: &str,
    value: &str,
) -> (gtk::Widget, gtk::Label) {
    let accent = crate::ui::brand_theme::accent().bright;
    let faceted = crate::ui::faceted_card::build_simple(accent, None);
    let card = faceted.content;
    card.set_valign(gtk::Align::Fill);
    card.set_vexpand(true);

    let icon_w: gtk::Widget = match image.and_then(|name| crate::ui::icon::icon(name, 40)) {
        Some(da) => {
            da.add_css_class("spec-icon-img");
            da.upcast()
        }
        None => {
            let l = gtk::Label::new(Some(icon));
            l.add_css_class("spec-icon");
            l.upcast()
        }
    };
    icon_w.set_valign(gtk::Align::Start);
    card.append(&icon_w);

    let text = gtk::Box::new(gtk::Orientation::Vertical, 2);
    text.set_hexpand(true);

    let t = gtk::Label::new(Some(title));
    t.add_css_class("spec-title");
    t.set_halign(gtk::Align::Start);
    text.append(&t);

    let v = gtk::Label::new(Some(value));
    v.add_css_class("spec-value");
    v.set_halign(gtk::Align::Start);
    v.set_wrap(true);
    // A wrapping label's *natural* width is its full unwrapped line by
    // default, regardless of set_wrap - long values like a CPU or storage
    // model name were silently inflating the window's initial size far
    // past the requested default. Cap it so wrapping is what it actually
    // does, not just what it's allowed to do.
    v.set_max_width_chars(34);
    v.set_xalign(0.0);
    text.append(&v);

    card.append(&text);
    (faceted.widget, v)
}

#[cfg(test)]
mod tests {
    use super::format_gpu_detail;

    #[test]
    fn keeps_static_nvidia_identity_visible_while_loading() {
        let detail = format_gpu_detail(
            "NVIDIA GeForce RTX 5070 Laptop GPU",
            0,
            "610.43.03",
            Some("Loading live NVIDIA data..."),
        );

        assert!(detail.contains("NVIDIA GeForce RTX 5070 Laptop GPU"));
        assert!(detail.contains("Driver 610.43.03"));
        assert!(detail.contains("Loading live NVIDIA data..."));
    }

    #[test]
    fn adds_vram_when_live_hydration_finishes() {
        let detail = format_gpu_detail(
            "NVIDIA GeForce RTX 5070 Laptop GPU",
            8192,
            "610.43.03",
            None,
        );

        assert!(detail.contains("8 GB VRAM"));
        assert!(detail.contains("Driver 610.43.03"));
    }
}
