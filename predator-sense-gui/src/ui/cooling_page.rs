//! "Cooling" hub: performance modes, fan control, the GPU dashboard, and the
//! firmware/thermal controls under one sidebar entry with a flat tab bar.
//! Each tab beyond the first is built lazily on first visit, so opening the
//! hub does not pay for four pages' worth of widgets and hardware reads up
//! front (the earlier version built them all eagerly and caused a navigation
//! spike).

use gtk4::prelude::*;
use gtk4::{self as gtk};
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use crate::ui::{fan_control_page, fan_page, gpu_page};

pub fn build() -> gtk::Box {
    let page = gtk::Box::new(gtk::Orientation::Vertical, 8);
    page.set_margin_top(8);
    page.set_margin_bottom(8);
    page.set_margin_start(20);
    page.set_margin_end(20);

    let tab_bar = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    tab_bar.set_halign(gtk::Align::Start);
    tab_bar.set_margin_bottom(6);

    let stack = gtk::Stack::new();
    stack.set_transition_type(gtk::StackTransitionType::Crossfade);
    stack.set_transition_duration(200);
    stack.set_hexpand(true);
    stack.set_vexpand(true);
    stack.set_hhomogeneous(false);
    stack.set_vhomogeneous(false);

    // The first tab is the hub's primary action (performance modes), so it
    // builds eagerly; the rest build on first visit.
    stack.add_named(&fan_page::build_modes(), Some("performance"));
    let built: Rc<RefCell<HashSet<String>>> =
        Rc::new(RefCell::new(HashSet::from(["performance".to_string()])));

    let tabs: [(&str, &str); 4] = [
        (crate::i18n::t("perf_title"), "performance"),
        (crate::i18n::t("fan_control"), "fans"),
        (crate::i18n::t("gpu_menu"), "gpu"),
        (crate::i18n::t("firmware_profiles"), "firmware"),
    ];

    let buttons: Rc<RefCell<Vec<gtk::Button>>> = Rc::new(RefCell::new(Vec::new()));
    for (i, (label, key)) in tabs.iter().enumerate() {
        let btn = gtk::Button::new();
        btn.add_css_class("usage-tab");
        if i == 0 {
            btn.add_css_class("usage-tab-active");
        }
        let btn_label = gtk::Label::new(Some(label));
        btn.set_child(Some(&btn_label));

        let stack_c = stack.clone();
        let built_c = built.clone();
        let key = key.to_string();
        let btns_c = buttons.clone();
        // Single handler does the whole switch: lazily build the target page
        // (if this is its first visit), move the stack, and repaint exactly
        // one active tab.
        btn.connect_clicked(move |_| {
            ensure_built(&stack_c, &built_c, &key);
            stack_c.set_visible_child_name(&key);
            for (j, b) in btns_c.borrow().iter().enumerate() {
                if j == i {
                    b.add_css_class("usage-tab-active");
                } else {
                    b.remove_css_class("usage-tab-active");
                }
            }
        });
        tab_bar.append(&btn);
        buttons.borrow_mut().push(btn);
    }

    page.append(&tab_bar);
    page.append(&stack);

    page
}

/// Builds a tab's page on its first visit and caches the fact, so the other
/// pages cost nothing until the user actually opens them.
fn ensure_built(stack: &gtk::Stack, built: &Rc<RefCell<HashSet<String>>>, key: &str) {
    if built.borrow().contains(key) {
        return;
    }
    let child: gtk::Widget = match key {
        "performance" => fan_page::build_modes().upcast(),
        "fans" => fan_control_page::build().upcast(),
        "gpu" => gpu_page::build().upcast(),
        "firmware" => fan_page::build_firmware().upcast(),
        _ => return,
    };
    stack.add_named(&child, Some(key));
    built.borrow_mut().insert(key.to_string());
}
