//! "Cooling" hub: performance profiles, fan control, and the GPU dashboard
//! under one sidebar entry with a tab bar (same pattern as `tools_page.rs`).
//! These three were separate top-level tabs; a hardware-control utility
//! needs one cooling/performance home, not three.

use gtk4::prelude::*;
use gtk4::{self as gtk};
use std::cell::RefCell;
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

    stack.add_named(&fan_page::build(), Some("profiles"));
    stack.add_named(&fan_control_page::build(), Some("fans"));
    stack.add_named(&gpu_page::build(), Some("gpu"));

    let tabs: [(&str, &str); 3] = [
        (crate::i18n::t("perf_mode"), "profiles"),
        (crate::i18n::t("fan_control"), "fans"),
        (crate::i18n::t("gpu_menu"), "gpu"),
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
        let key = key.to_string();
        let btns_c = buttons.clone();
        // Single handler does the whole switch: move the stack AND repaint
        // exactly one active tab. (The earlier two-handler version only
        // worked because GTK runs handlers in registration order.)
        btn.connect_clicked(move |_| {
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
