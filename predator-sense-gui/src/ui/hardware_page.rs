//! "Hardware" report: what the runtime detection found on this exact machine
//! and what the reverse-engineering effort knows about driving it. The point
//! of an unsupported laptop is not just that the app works, but that it is
//! honest about *what* it detected and *how confident* it is in each write.

use gtk4::prelude::*;
use gtk4::{self as gtk};

use crate::hardware::{capabilities, rgb, sysinfo, thermal_profile};

pub fn build() -> gtk::ScrolledWindow {
    let scroll = gtk::ScrolledWindow::new();
    scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    scroll.set_propagate_natural_width(false);

    let page = gtk::Box::new(gtk::Orientation::Vertical, 14);
    page.set_margin_top(14);
    page.set_margin_bottom(20);
    page.set_margin_start(20);
    page.set_margin_end(20);

    let title = gtk::Label::new(Some(crate::i18n::t("hardware_title")));
    title.add_css_class("section-title");
    title.set_halign(gtk::Align::Start);
    page.append(&title);

    let subtitle = gtk::Label::new(Some(crate::i18n::t("hardware_subtitle")));
    subtitle.add_css_class("section-subtitle");
    subtitle.set_halign(gtk::Align::Start);
    subtitle.set_wrap(true);
    page.append(&subtitle);

    page.append(&build_identity());
    page.append(&build_capabilities());
    page.append(&build_reverse_engineering());

    scroll.set_child(Some(&page));
    scroll
}

/// One label/value line inside a card.
fn row(label: &str, value: &str) -> gtk::Box {
    let box_ = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    box_.set_margin_top(2);
    box_.set_margin_bottom(2);
    let l = gtk::Label::new(Some(label));
    l.add_css_class("info-text-dim");
    l.set_halign(gtk::Align::Start);
    l.set_hexpand(true);
    let v = gtk::Label::new(Some(value));
    v.add_css_class("info-text");
    v.set_halign(gtk::Align::End);
    v.set_xalign(1.0);
    v.set_wrap(true);
    v.set_selectable(true);
    box_.append(&l);
    box_.append(&v);
    box_
}

/// One capability line with a color-coded status.
fn status_row(label: &str, status: &str, good: bool, warn: bool) -> gtk::Box {
    let box_ = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    box_.set_margin_top(3);
    box_.set_margin_bottom(3);
    let l = gtk::Label::new(Some(label));
    l.add_css_class("info-text");
    l.set_halign(gtk::Align::Start);
    l.set_hexpand(true);
    let color = if good {
        "#3dc773"
    } else if warn {
        "#f29e29"
    } else {
        "#8b95a3"
    };
    let v = gtk::Label::new(None);
    v.set_markup(&format!("<span foreground=\"{color}\">{status}</span>"));
    v.set_halign(gtk::Align::End);
    box_.append(&l);
    box_.append(&v);
    box_
}

fn build_identity() -> gtk::Widget {
    let info = sysinfo::read_system_info();
    let card = crate::ui::faceted_card::build_simple(
        crate::ui::brand_theme::accent().bright,
        Some(crate::i18n::t("hardware_section_identity")),
    );
    card.content.set_orientation(gtk::Orientation::Vertical);
    card.content.set_spacing(2);

    card.content.append(&row(
        crate::i18n::t("system_os"),
        &format!("{} · {}", info.os_pretty, info.kernel),
    ));
    card.content.append(&row("CPU", &info.cpu_model));
    card.content.append(&row("GPU", &info.gpu_name));
    card.content.append(&row(
        crate::i18n::t("memory"),
        &format!("{:.0} GB · {}", info.ram_total_gb, info.ram_type),
    ));
    if let Some(first) = info.storage.first() {
        card.content.append(&row(
            crate::i18n::t("storage"),
            &format!("{} · {:.0} GB", first.model, first.size_gb),
        ));
    }
    card.content.append(&row("BIOS", &info.bios_version));

    card.widget
}

fn build_capabilities() -> gtk::Widget {
    let caps = capabilities::get();
    let card = crate::ui::faceted_card::build_simple(
        crate::ui::brand_theme::accent().bright,
        Some(crate::i18n::t("hardware_section_capabilities")),
    );
    card.content.set_orientation(gtk::Orientation::Vertical);
    card.content.set_spacing(2);

    let avail = crate::i18n::t("status_available");
    let unavail = crate::i18n::t("status_not_available");

    for (label, ok) in [
        (crate::i18n::t("feat_rgb"), caps.rgb),
        (crate::i18n::t("feat_cover_logo"), caps.cover_logo),
        (crate::i18n::t("feat_fan_rpm"), caps.fan_rpm),
        (crate::i18n::t("feat_fan_pwm"), caps.fan_pwm),
        (crate::i18n::t("feat_profiles"), caps.performance_profiles),
        (crate::i18n::t("feat_ec"), caps.ec),
        (crate::i18n::t("feat_gpu"), caps.nvidia_gpu),
        (crate::i18n::t("feat_battery"), caps.battery_charge_cap()),
    ] {
        card.content.append(&status_row(
            label,
            if ok { avail } else { unavail },
            ok,
            false,
        ));
    }

    // Fan preset is the one capability with a three-way confidence, not a
    // simple on/off: verified vs unverified vs known-incompatible.
    let (preset_status, preset_good, preset_warn) = match caps.fan_preset_status {
        capabilities::FanPresetStatus::Verified => {
            (crate::i18n::t("status_verified"), true, false)
        }
        capabilities::FanPresetStatus::KnownIncompatible => {
            (crate::i18n::t("status_incompatible"), false, true)
        }
        capabilities::FanPresetStatus::Unverified => {
            (crate::i18n::t("status_unverified"), false, true)
        }
    };
    card.content
        .append(&status_row(crate::i18n::t("cap_fan_preset"), preset_status, preset_good, preset_warn));

    // Kernel module load state.
    let module_loaded = rgb::is_module_loaded();
    card.content.append(&status_row(
        crate::i18n::t("cap_module"),
        crate::i18n::t(if module_loaded {
            "status_loaded"
        } else {
            "status_not_loaded"
        }),
        module_loaded,
        !module_loaded,
    ));

    // Thermal calibration (firmware profile ranking).
    let cal = thermal_profile::load();
    let (cal_status, cal_good) = match &cal {
        Some(c) if c.profiles.len() > 1 => {
            let kind = if c.is_ranked() {
                crate::i18n::t("hardware_cal_ranked")
            } else {
                crate::i18n::t("hardware_cal_unranked")
            };
            (format!("{kind} · {}", c.profiles.len()), true)
        }
        _ => (crate::i18n::t("status_not_available").to_string(), false),
    };
    card.content
        .append(&status_row(crate::i18n::t("cap_thermal_cal"), &cal_status, cal_good, false));

    card.widget
}

fn build_reverse_engineering() -> gtk::Widget {
    let card = crate::ui::faceted_card::build_simple(
        crate::ui::brand_theme::accent().bright,
        Some(crate::i18n::t("hardware_section_re")),
    );
    card.content.set_orientation(gtk::Orientation::Vertical);
    card.content.set_spacing(2);

    // Stable reverse-engineering facts, language-independent identifiers.
    card.content.append(&row(
        "WMI",
        "GUID 7A4DDFE7-5B5D-40B4-8595-4408E0CC7F56 · object \"BG\" → WMBG → WSMI → EC 0xD0",
    ));
    card.content.append(&row(
        "Fan behavior",
        "WMID_gaming_set/get_fan_behavior (methods 14–15) · pwm_enable 0=Turbo / 1=Custom / 2=Auto",
    ));
    card.content.append(&row(
        "Turbo key",
        "OC (0x205/0x207) + turbo fan (method 14) + turbo LED over gaming WMI",
    ));
    card.content.append(&row(
        "Kernel modules",
        "facer · acer-wmi-battery · acpi_ec (DKMS)",
    ));

    card.widget
}
