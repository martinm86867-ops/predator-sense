use gtk4::prelude::*;
use gtk4::{self as gtk, glib};
use std::cell::RefCell;
use std::f64::consts::{PI, TAU};
use std::rc::Rc;

use crate::hardware::procs::{self, ProcessInfo, UsageSample, UsageSampler};
use crate::hardware::storage::{self, DiskUsage};

const TOP_N: usize = 8;

struct AnimState {
    sample: UsageSample,
    // Valores "exibidos" (interpolados) para animação suave
    cpu_total_shown: f64,
    cpu_temp_shown: f64,
    cpu_per_core_shown: Vec<f64>,
    mem_used_shown: f64,
    swap_used_shown: f64,
    gpu_util_shown: f64,
    gpu_temp_shown: f64,
    gpu_vram_shown: f64,
    gpu_power_shown: f64,
    disks: Vec<DiskUsage>,
    disk_shown: Vec<f64>,
    proc_shown_cpu: Vec<f64>,
    proc_shown_mem: Vec<f64>,
    expanded_pid_cpu: Option<u32>,
    expanded_pid_mem: Option<u32>,
    expanded_disk: Option<String>,
    pulse_phase: f64,
}

pub fn build() -> gtk::Box {
    let page = gtk::Box::new(gtk::Orientation::Vertical, 12);
    page.set_margin_top(16);
    page.set_margin_bottom(16);
    page.set_margin_start(22);
    page.set_margin_end(22);

    let mut sampler = UsageSampler::new();
    let initial = sampler.sample();
    let ncpu = sampler.num_cpus();
    let disks = storage::list_disks();

    let state = Rc::new(RefCell::new(AnimState {
        sample: initial.clone(),
        cpu_total_shown: 0.0,
        cpu_temp_shown: 0.0,
        cpu_per_core_shown: vec![0.0; ncpu],
        mem_used_shown: 0.0,
        swap_used_shown: 0.0,
        gpu_util_shown: 0.0,
        gpu_temp_shown: 0.0,
        gpu_vram_shown: 0.0,
        gpu_power_shown: 0.0,
        disk_shown: vec![0.0; disks.len()],
        disks,
        proc_shown_cpu: vec![0.0; TOP_N],
        proc_shown_mem: vec![0.0; TOP_N],
        expanded_pid_cpu: None,
        expanded_pid_mem: None,
        expanded_disk: None,
        pulse_phase: 0.0,
    }));

    let sampler = Rc::new(RefCell::new(sampler));

    // Tabs
    let tab_bar = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    tab_bar.set_halign(gtk::Align::Start);

    let stack = gtk::Stack::new();
    stack.set_transition_type(gtk::StackTransitionType::Crossfade);
    stack.set_transition_duration(200);
    stack.set_hexpand(true);
    stack.set_vexpand(true);

    let cpu_tab = build_cpu_tab(state.clone());
    let gpu_tab = build_gpu_tab(state.clone());
    let mem_tab = build_mem_tab(state.clone());
    let storage_tab = build_storage_tab(state.clone());

    stack.add_named(&cpu_tab, Some("cpu"));
    stack.add_named(&gpu_tab, Some("gpu"));
    stack.add_named(&mem_tab, Some("mem"));
    stack.add_named(&storage_tab, Some("storage"));

    let tabs = [
        (crate::i18n::t("usage_cpu"), "cpu"),
        ("GPU", "gpu"),
        (crate::i18n::t("usage_memory"), "mem"),
        (crate::i18n::t("usage_storage"), "storage"),
    ];
    let buttons: Rc<RefCell<Vec<gtk::Button>>> = Rc::new(RefCell::new(Vec::new()));
    for (i, (label, key)) in tabs.iter().enumerate() {
        let btn = gtk::Button::with_label(label);
        btn.add_css_class("usage-tab");
        if i == 0 {
            btn.add_css_class("usage-tab-active");
        }
        let stack_c = stack.clone();
        let k = key.to_string();
        let btns_c = buttons.clone();
        btn.connect_clicked(move |_| {
            stack_c.set_visible_child_name(&k);
            for b in btns_c.borrow().iter() {
                b.remove_css_class("usage-tab-active");
            }
        });
        tab_bar.append(&btn);
        buttons.borrow_mut().push(btn);
    }
    // Click handler needs to know which button was clicked to add active class
    {
        let btns = buttons.borrow().clone();
        for (i, b) in btns.iter().enumerate() {
            let btns2 = buttons.clone();
            let idx = i;
            b.connect_clicked(move |_| {
                for (j, bj) in btns2.borrow().iter().enumerate() {
                    if j == idx {
                        bj.add_css_class("usage-tab-active");
                    } else {
                        bj.remove_css_class("usage-tab-active");
                    }
                }
            });
        }
    }

    page.append(&tab_bar);
    page.append(&stack);

    // Sampling periódico a cada 2s. Só executa quando esta página é a aba
    // ativa no Stack principal — evita procs.sample() rodando em background.
    {
        let state_c = state.clone();
        let sampler_c = sampler.clone();
        let page_c = page.clone();
        glib::timeout_add_seconds_local(2, move || {
            if !crate::app_state::is_window_visible() || !page_c.is_mapped() {
                return glib::ControlFlow::Continue;
            }
            let mut s = sampler_c.borrow_mut();
            let sample = s.sample();
            let disks = storage::list_disks();
            let mut st = state_c.borrow_mut();
            // Sincronizar tamanho dos vetores
            let dlen = disks.len();
            if st.disk_shown.len() != dlen {
                st.disk_shown.resize(dlen, 0.0);
            }
            st.sample = sample;
            st.disks = disks;
            glib::ControlFlow::Continue
        });
    }

    // Tick de animação (reduzido para 200ms para poupar ciclos de CPU)
    {
        let state_c = state.clone();
        let page_c = page.clone();
        glib::timeout_add_local(std::time::Duration::from_millis(200), move || {
            if !crate::app_state::is_window_visible() || !page_c.is_mapped() {
                return glib::ControlFlow::Continue;
            }
            let mut st = state_c.borrow_mut();
            st.pulse_phase += 0.02;
            if st.pulse_phase > 1000.0 {
                st.pulse_phase = 0.0;
            }
            let a = 0.12_f64; // lerp factor — suavidade
            st.cpu_total_shown = lerp(st.cpu_total_shown, st.sample.cpu_total_pct, a);
            let cpu_temp_target = st.sample.cpu_temp.unwrap_or(0.0);
            st.cpu_temp_shown = lerp(st.cpu_temp_shown, cpu_temp_target, a);
            for i in 0..st.cpu_per_core_shown.len() {
                let target = st.sample.cpu_per_core_pct.get(i).copied().unwrap_or(0.0);
                st.cpu_per_core_shown[i] = lerp(st.cpu_per_core_shown[i], target, a);
            }
            st.mem_used_shown = lerp(st.mem_used_shown, st.sample.mem.used_pct(), a);
            // GPU interpolation
            let gpu_targets = st
                .sample
                .gpu
                .as_ref()
                .map(|g| (g.util_gpu_pct as f64, g.temp, g.vram_pct(), g.power_draw_w));
            if let Some((util, temp, vram, power)) = gpu_targets {
                st.gpu_util_shown = lerp(st.gpu_util_shown, util, a);
                st.gpu_temp_shown = lerp(st.gpu_temp_shown, temp, a);
                st.gpu_vram_shown = lerp(st.gpu_vram_shown, vram, a);
                st.gpu_power_shown = lerp(st.gpu_power_shown, power, a);
            }
            let swap_pct = if st.sample.mem.swap_total_kb == 0 {
                0.0
            } else {
                st.sample.mem.swap_used_kb() as f64 / st.sample.mem.swap_total_kb as f64 * 100.0
            };
            st.swap_used_shown = lerp(st.swap_used_shown, swap_pct, a);
            for i in 0..st.disk_shown.len() {
                let target = st.disks.get(i).map(|d| d.percent).unwrap_or(0.0);
                st.disk_shown[i] = lerp(st.disk_shown[i], target, a);
            }

            // Top process lists: basta interpolar valores atuais dos top N
            let top_cpu = procs::top_by_cpu(&st.sample.processes, TOP_N);
            let top_mem = procs::top_by_mem(&st.sample.processes, TOP_N);
            for i in 0..TOP_N {
                let target_cpu = top_cpu.get(i).map(|p| p.cpu_pct).unwrap_or(0.0);
                let target_mem = top_mem
                    .get(i)
                    .and_then(|p| {
                        if st.sample.mem.total_kb > 0 {
                            Some(p.mem_kb as f64 / st.sample.mem.total_kb as f64 * 100.0)
                        } else {
                            None
                        }
                    })
                    .unwrap_or(0.0);
                st.proc_shown_cpu[i] = lerp(st.proc_shown_cpu[i], target_cpu, a);
                st.proc_shown_mem[i] = lerp(st.proc_shown_mem[i], target_mem, a);
            }
            glib::ControlFlow::Continue
        });
    }

    page
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

// ============ CPU TAB ============

fn build_cpu_tab(state: Rc<RefCell<AnimState>>) -> gtk::Box {
    let page = gtk::Box::new(gtk::Orientation::Vertical, 14);
    page.set_margin_top(10);

    // Linha superior: coluna esquerda (CPU% + CPU temp com fogo) + barras por core
    let top_row = gtk::Box::new(gtk::Orientation::Horizontal, 18);

    // Coluna esquerda: CPU% gauge + temp gauge com fogo
    let left_col = gtk::Box::new(gtk::Orientation::Vertical, 12);
    left_col.set_valign(gtk::Align::Start);

    // Gauge grande CPU%
    let gauge_faceted =
        crate::ui::faceted_card::build_simple(crate::ui::brand_theme::accent().bright, None);
    let gauge_box = gauge_faceted.content;
    gauge_box.set_orientation(gtk::Orientation::Vertical);
    gauge_box.set_spacing(4);
    let gauge_da = gtk::DrawingArea::new();
    gauge_da.set_size_request(220, 220);
    let st_gauge = state.clone();
    gauge_da.set_draw_func(move |_a, cr, w, h| {
        let st = st_gauge.borrow();
        draw_big_gauge(cr, w as f64, h as f64, st.cpu_total_shown, st.pulse_phase);
    });
    gauge_box.append(&gauge_da);
    let gauge_title = gtk::Label::new(Some("CPU Total"));
    gauge_title.add_css_class("usage-hero-title");
    gauge_title.set_halign(gtk::Align::Center);
    gauge_box.append(&gauge_title);
    left_col.append(&gauge_faceted.widget);

    // Gauge temperatura CPU com fogo
    let temp_faceted =
        crate::ui::faceted_card::build_simple(crate::ui::brand_theme::accent().bright, None);
    let temp_box = temp_faceted.content;
    temp_box.set_orientation(gtk::Orientation::Vertical);
    temp_box.set_spacing(4);
    let temp_da = gtk::DrawingArea::new();
    temp_da.set_size_request(220, 170);
    let st_temp = state.clone();
    temp_da.set_draw_func(move |_a, cr, w, h| {
        let st = st_temp.borrow();
        draw_temp_gauge(cr, w as f64, h as f64, st.cpu_temp_shown, st.pulse_phase);
    });
    temp_box.append(&temp_da);
    let temp_title = gtk::Label::new(Some(crate::i18n::t("cpu_temperature")));
    temp_title.add_css_class("usage-hero-title");
    temp_title.set_halign(gtk::Align::Center);
    temp_box.append(&temp_title);
    left_col.append(&temp_faceted.widget);

    top_row.append(&left_col);

    // Per-core
    let cores_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
    cores_box.set_hexpand(true);
    cores_box.set_vexpand(true);
    let cores_title = gtk::Label::new(Some(crate::i18n::t("per_core")));
    cores_title.add_css_class("section-title");
    cores_title.set_halign(gtk::Align::Start);
    cores_box.append(&cores_title);

    let cores_da = gtk::DrawingArea::new();
    cores_da.set_hexpand(true);
    cores_da.set_vexpand(true);
    cores_da.set_size_request(-1, 420);
    let cores_faceted =
        crate::ui::faceted_card::build_simple(crate::ui::brand_theme::accent().bright, None);
    let st_cores = state.clone();
    cores_da.set_draw_func(move |_a, cr, w, h| {
        let st = st_cores.borrow();
        draw_per_core(
            cr,
            w as f64,
            h as f64,
            &st.cpu_per_core_shown,
            st.pulse_phase,
        );
    });
    cores_faceted.content.append(&cores_da);
    cores_box.append(&cores_faceted.widget);
    top_row.append(&cores_box);

    page.append(&top_row);

    // Top processos (CPU)
    let procs_title = gtk::Label::new(Some(crate::i18n::t("top_processes_cpu")));
    procs_title.add_css_class("section-title");
    procs_title.set_halign(gtk::Align::Start);
    page.append(&procs_title);

    let scroll = gtk::ScrolledWindow::new();
    scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    scroll.set_vexpand(true);

    let list_container = gtk::Box::new(gtk::Orientation::Vertical, 4);
    list_container.add_css_class("usage-list");
    scroll.set_child(Some(&list_container));
    page.append(&scroll);

    // Atualiza a lista periodicamente (2s) reconstruindo
    let state_list = state.clone();
    let lc = list_container.clone();
    let gauge_da_c = gauge_da.clone();
    let cores_da_c = cores_da.clone();
    let temp_da_c = temp_da.clone();
    let lc_anim = list_container.clone();
    let page_anim = page.clone();
    glib::timeout_add_local(std::time::Duration::from_millis(200), move || {
        if !crate::app_state::is_window_visible() || !page_anim.is_mapped() {
            return glib::ControlFlow::Continue;
        }
        gauge_da_c.queue_draw();
        cores_da_c.queue_draw();
        temp_da_c.queue_draw();
        // Redesenha barras das linhas de processo (no per-row timer needed)
        let mut child = lc_anim.first_child();
        while let Some(c) = child {
            redraw_in_subtree(&c);
            child = c.next_sibling();
        }
        glib::ControlFlow::Continue
    });

    let page_list = page.clone();
    glib::timeout_add_seconds_local(2, move || {
        if !crate::app_state::is_window_visible() || !page_list.is_mapped() {
            return glib::ControlFlow::Continue;
        }
        rebuild_cpu_process_list(&lc, &state_list);
        glib::ControlFlow::Continue
    });

    // Build inicial
    glib::idle_add_local_once({
        let lc = list_container.clone();
        let st = state.clone();
        move || rebuild_cpu_process_list(&lc, &st)
    });

    page
}

fn rebuild_cpu_process_list(list: &gtk::Box, state: &Rc<RefCell<AnimState>>) {
    while let Some(c) = list.first_child() {
        list.remove(&c);
    }
    let st = state.borrow();
    let top = procs::top_by_cpu(&st.sample.processes, TOP_N);
    let expanded = st.expanded_pid_cpu;
    drop(st);
    for (i, p) in top.iter().enumerate() {
        let row = build_process_row(
            p,
            i,
            ProcMetric::Cpu,
            expanded == Some(p.pid),
            state.clone(),
        );
        list.append(&row);
    }
}

// ============ GPU TAB ============

fn build_gpu_tab(state: Rc<RefCell<AnimState>>) -> gtk::Box {
    let page = gtk::Box::new(gtk::Orientation::Vertical, 14);
    page.set_margin_top(10);

    // Header com nome e driver
    let header = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let gpu_name = gtk::Label::new(Some("GPU"));
    gpu_name.add_css_class("section-title");
    gpu_name.set_halign(gtk::Align::Start);
    let gpu_driver = gtk::Label::new(None);
    gpu_driver.add_css_class("monitor-subtitle");
    gpu_driver.set_hexpand(true);
    gpu_driver.set_halign(gtk::Align::End);
    header.append(&gpu_name);
    header.append(&gpu_driver);
    page.append(&header);

    // Linha superior: [util gauge + temp gauge] | [VRAM donut + Power gauge]
    let top_row = gtk::Box::new(gtk::Orientation::Horizontal, 18);
    top_row.set_homogeneous(true);

    // Coluna esquerda: util + temp
    let left_col = gtk::Box::new(gtk::Orientation::Vertical, 12);

    let util_faceted =
        crate::ui::faceted_card::build_simple(crate::ui::brand_theme::accent().bright, None);
    let util_box = util_faceted.content;
    util_box.set_orientation(gtk::Orientation::Vertical);
    util_box.set_spacing(4);
    let util_da = gtk::DrawingArea::new();
    util_da.set_size_request(220, 220);
    let st_util = state.clone();
    util_da.set_draw_func(move |_a, cr, w, h| {
        let st = st_util.borrow();
        draw_big_gauge(cr, w as f64, h as f64, st.gpu_util_shown, st.pulse_phase);
    });
    util_box.append(&util_da);
    let util_title = gtk::Label::new(Some(crate::i18n::t("gpu_util_label")));
    util_title.add_css_class("usage-hero-title");
    util_title.set_halign(gtk::Align::Center);
    util_box.append(&util_title);
    left_col.append(&util_faceted.widget);

    let gpu_temp_faceted =
        crate::ui::faceted_card::build_simple(crate::ui::brand_theme::accent().bright, None);
    let gpu_temp_box = gpu_temp_faceted.content;
    gpu_temp_box.set_orientation(gtk::Orientation::Vertical);
    gpu_temp_box.set_spacing(4);
    let gpu_temp_da = gtk::DrawingArea::new();
    gpu_temp_da.set_size_request(220, 170);
    let st_gtemp = state.clone();
    gpu_temp_da.set_draw_func(move |_a, cr, w, h| {
        let st = st_gtemp.borrow();
        draw_temp_gauge(cr, w as f64, h as f64, st.gpu_temp_shown, st.pulse_phase);
    });
    gpu_temp_box.append(&gpu_temp_da);
    let gpu_temp_title = gtk::Label::new(Some(crate::i18n::t("gpu_temperature_label")));
    gpu_temp_title.add_css_class("usage-hero-title");
    gpu_temp_title.set_halign(gtk::Align::Center);
    gpu_temp_box.append(&gpu_temp_title);
    left_col.append(&gpu_temp_faceted.widget);

    top_row.append(&left_col);

    // Coluna direita: VRAM donut + Power
    let right_col = gtk::Box::new(gtk::Orientation::Vertical, 12);

    let vram_faceted =
        crate::ui::faceted_card::build_simple(crate::ui::brand_theme::accent().bright, None);
    let vram_box = vram_faceted.content;
    vram_box.set_orientation(gtk::Orientation::Vertical);
    vram_box.set_spacing(4);
    let vram_da = gtk::DrawingArea::new();
    vram_da.set_size_request(220, 220);
    let st_vram = state.clone();
    vram_da.set_draw_func(move |_a, cr, w, h| {
        let st = st_vram.borrow();
        draw_vram_donut(
            cr,
            w as f64,
            h as f64,
            &st.sample.gpu,
            st.gpu_vram_shown,
            st.pulse_phase,
        );
    });
    vram_box.append(&vram_da);
    let vram_title = gtk::Label::new(Some("VRAM"));
    vram_title.add_css_class("usage-hero-title");
    vram_title.set_halign(gtk::Align::Center);
    vram_box.append(&vram_title);
    right_col.append(&vram_faceted.widget);

    let power_faceted =
        crate::ui::faceted_card::build_simple(crate::ui::brand_theme::accent().bright, None);
    let power_box = power_faceted.content;
    power_box.set_orientation(gtk::Orientation::Vertical);
    power_box.set_spacing(4);
    let power_da = gtk::DrawingArea::new();
    power_da.set_size_request(220, 170);
    let st_pow = state.clone();
    power_da.set_draw_func(move |_a, cr, w, h| {
        let st = st_pow.borrow();
        draw_power_gauge(
            cr,
            w as f64,
            h as f64,
            st.gpu_power_shown,
            &st.sample.gpu,
            st.pulse_phase,
        );
    });
    power_box.append(&power_da);
    let power_title = gtk::Label::new(Some(crate::i18n::t("gpu_power_label")));
    power_title.add_css_class("usage-hero-title");
    power_title.set_halign(gtk::Align::Center);
    power_box.append(&power_title);
    right_col.append(&power_faceted.widget);

    top_row.append(&right_col);
    page.append(&top_row);

    // Linha inferior: stats em grid
    let stats_row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    stats_row.set_homogeneous(true);
    stats_row.set_margin_top(4);

    let (core_card, core_v) = make_mem_stat(crate::i18n::t("clock_core"));
    let (mem_card, mem_v) = make_mem_stat(crate::i18n::t("clock_vram"));
    let (fan_card, fan_v) = make_mem_stat(crate::i18n::t("gpu_fan_label"));
    let (pstate_card, pstate_v) = make_mem_stat("P-State");
    let (pcie_card, pcie_v) = make_mem_stat("PCIe");

    stats_row.append(&core_card);
    stats_row.append(&mem_card);
    stats_row.append(&fan_card);
    stats_row.append(&pstate_card);
    stats_row.append(&pcie_card);
    page.append(&stats_row);

    // Updates: textos + redraw de todas áreas de desenho
    let st_u = state.clone();
    let name_c = gpu_name.clone();
    let drv_c = gpu_driver.clone();
    let core_c = core_v.clone();
    let mem_c = mem_v.clone();
    let fan_c = fan_v.clone();
    let ps_c = pstate_v.clone();
    let pcie_c = pcie_v.clone();
    let util_da_c = util_da.clone();
    let temp_da_c = gpu_temp_da.clone();
    let vram_da_c = vram_da.clone();
    let power_da_c = power_da.clone();
    let page_c = page.clone();
    glib::timeout_add_local(std::time::Duration::from_millis(200), move || {
        if !crate::app_state::is_window_visible() || !page_c.is_mapped() {
            return glib::ControlFlow::Continue;
        }
        let st = st_u.borrow();
        if let Some(g) = &st.sample.gpu {
            name_c.set_text(&format!("GPU · {}", g.name));
            drv_c.set_text(&format!("Driver {} · VBIOS {}", g.driver, g.vbios));
            core_c.set_text(&format!("{} MHz", g.clock_core_mhz));
            mem_c.set_text(&format!("{} MHz", g.clock_mem_mhz));
            fan_c.set_text(
                &g.fan_speed_pct
                    .map(|v| format!("{v}%"))
                    .unwrap_or_else(|| "--".into()),
            );
            ps_c.set_text(&g.pstate);
            pcie_c.set_text(&format!("Gen{} x{}", g.pcie_gen, g.pcie_width));
        } else {
            name_c.set_text(crate::i18n::t("gpu_live_unavailable"));
            drv_c.set_text("");
            core_c.set_text("--");
            mem_c.set_text("--");
            fan_c.set_text("--");
            ps_c.set_text("--");
            pcie_c.set_text("--");
        }
        util_da_c.queue_draw();
        temp_da_c.queue_draw();
        vram_da_c.queue_draw();
        power_da_c.queue_draw();
        glib::ControlFlow::Continue
    });

    page
}

// ============ MEM TAB ============

fn build_mem_tab(state: Rc<RefCell<AnimState>>) -> gtk::Box {
    let page = gtk::Box::new(gtk::Orientation::Vertical, 14);
    page.set_margin_top(10);

    // Card principal
    let mem_faceted =
        crate::ui::faceted_card::build_simple(crate::ui::brand_theme::accent().bright, None);
    let mem_card = mem_faceted.content;
    mem_card.set_orientation(gtk::Orientation::Vertical);
    mem_card.set_spacing(6);

    let mem_header = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let mem_title = gtk::Label::new(Some(crate::i18n::t("usage_memory")));
    mem_title.add_css_class("section-title");
    mem_title.set_hexpand(true);
    mem_title.set_halign(gtk::Align::Start);
    let mem_value = gtk::Label::new(Some("--"));
    mem_value.add_css_class("usage-big-number");
    mem_value.set_halign(gtk::Align::End);
    mem_header.append(&mem_title);
    mem_header.append(&mem_value);
    mem_card.append(&mem_header);

    let mem_bar = gtk::DrawingArea::new();
    mem_bar.set_content_height(36);
    mem_bar.set_hexpand(true);
    let st_bar = state.clone();
    mem_bar.set_draw_func(move |_a, cr, w, h| {
        let st = st_bar.borrow();
        draw_mem_bar(
            cr,
            w as f64,
            h as f64,
            &st.sample.mem,
            st.mem_used_shown,
            st.pulse_phase,
        );
    });
    mem_card.append(&mem_bar);

    // Stats row
    let stats = gtk::Box::new(gtk::Orientation::Horizontal, 14);
    stats.set_homogeneous(true);
    stats.set_margin_top(6);
    let (used_card, used_val) = make_mem_stat(crate::i18n::t("mem_used"));
    let (cached_card, cached_val) = make_mem_stat(crate::i18n::t("mem_cached"));
    let (avail_card, avail_val) = make_mem_stat(crate::i18n::t("mem_available"));
    let (swap_card, swap_val) = make_mem_stat(crate::i18n::t("mem_swap"));
    stats.append(&used_card);
    stats.append(&cached_card);
    stats.append(&avail_card);
    stats.append(&swap_card);
    mem_card.append(&stats);

    page.append(&mem_faceted.widget);

    // Top processos (memória)
    let title = gtk::Label::new(Some(crate::i18n::t("top_processes_mem")));
    title.add_css_class("section-title");
    title.set_halign(gtk::Align::Start);
    page.append(&title);

    let scroll = gtk::ScrolledWindow::new();
    scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    scroll.set_vexpand(true);
    let list = gtk::Box::new(gtk::Orientation::Vertical, 4);
    list.add_css_class("usage-list");
    scroll.set_child(Some(&list));
    page.append(&scroll);

    // Updates
    let st_u = state.clone();
    let val_c = mem_value.clone();
    let used_c = used_val.clone();
    let cached_c = cached_val.clone();
    let avail_c = avail_val.clone();
    let swap_c = swap_val.clone();
    let bar_c = mem_bar.clone();
    let list_anim = list.clone();
    let page_anim = page.clone();
    glib::timeout_add_local(std::time::Duration::from_millis(200), move || {
        if !crate::app_state::is_window_visible() || !page_anim.is_mapped() {
            return glib::ControlFlow::Continue;
        }
        let st = st_u.borrow();
        let mi = &st.sample.mem;
        val_c.set_text(&format!(
            "{:.1}% · {}/{}",
            mi.used_pct(),
            format_kb(mi.used_kb()),
            format_kb(mi.total_kb)
        ));
        used_c.set_text(&format_kb(mi.used_kb()));
        cached_c.set_text(&format_kb(mi.cached_kb + mi.buffers_kb));
        avail_c.set_text(&format_kb(mi.available_kb));
        swap_c.set_text(&format!(
            "{} / {}",
            format_kb(mi.swap_used_kb()),
            format_kb(mi.swap_total_kb)
        ));
        bar_c.queue_draw();
        let mut child = list_anim.first_child();
        while let Some(c) = child {
            redraw_in_subtree(&c);
            child = c.next_sibling();
        }
        glib::ControlFlow::Continue
    });

    let st_list = state.clone();
    let lc = list.clone();
    let page_list = page.clone();
    glib::timeout_add_seconds_local(2, move || {
        if !crate::app_state::is_window_visible() || !page_list.is_mapped() {
            return glib::ControlFlow::Continue;
        }
        rebuild_mem_process_list(&lc, &st_list);
        glib::ControlFlow::Continue
    });
    glib::idle_add_local_once({
        let lc = list.clone();
        let st = state.clone();
        move || rebuild_mem_process_list(&lc, &st)
    });

    page
}

fn rebuild_mem_process_list(list: &gtk::Box, state: &Rc<RefCell<AnimState>>) {
    while let Some(c) = list.first_child() {
        list.remove(&c);
    }
    let st = state.borrow();
    let top = procs::top_by_mem(&st.sample.processes, TOP_N);
    let expanded = st.expanded_pid_mem;
    drop(st);
    for (i, p) in top.iter().enumerate() {
        let row = build_process_row(
            p,
            i,
            ProcMetric::Mem,
            expanded == Some(p.pid),
            state.clone(),
        );
        list.append(&row);
    }
}

fn make_mem_stat(title: &str) -> (gtk::Box, gtk::Label) {
    let c = gtk::Box::new(gtk::Orientation::Vertical, 2);
    c.add_css_class("usage-stat");
    let t = gtk::Label::new(Some(title));
    t.add_css_class("stat-title");
    t.set_halign(gtk::Align::Start);
    let v = gtk::Label::new(Some("—"));
    v.add_css_class("usage-stat-value");
    v.set_halign(gtk::Align::Start);
    c.append(&t);
    c.append(&v);
    (c, v)
}

// ============ STORAGE TAB ============

fn build_storage_tab(state: Rc<RefCell<AnimState>>) -> gtk::Box {
    let outer = gtk::Box::new(gtk::Orientation::Vertical, 10);
    outer.set_margin_top(10);

    let title = gtk::Label::new(Some(crate::i18n::t("storage")));
    title.add_css_class("section-title");
    title.set_halign(gtk::Align::Start);
    outer.append(&title);

    let scroll = gtk::ScrolledWindow::new();
    scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    scroll.set_vexpand(true);

    let flow = gtk::FlowBox::new();
    flow.set_selection_mode(gtk::SelectionMode::None);
    flow.set_max_children_per_line(3);
    flow.set_min_children_per_line(1);
    flow.set_homogeneous(true);
    flow.set_column_spacing(10);
    flow.set_row_spacing(10);
    flow.add_css_class("usage-flow");
    scroll.set_child(Some(&flow));
    outer.append(&scroll);

    let st_list = state.clone();
    let flow_c = flow.clone();
    let outer_list = outer.clone();
    glib::timeout_add_seconds_local(3, move || {
        if !crate::app_state::is_window_visible() || !outer_list.is_mapped() {
            return glib::ControlFlow::Continue;
        }
        rebuild_storage_cards(&flow_c, &st_list);
        glib::ControlFlow::Continue
    });
    glib::idle_add_local_once({
        let flow = flow.clone();
        let st = state.clone();
        move || rebuild_storage_cards(&flow, &st)
    });

    // Redesenho contínuo para animar donuts
    let flow_c2 = flow.clone();
    let outer_anim = outer.clone();
    glib::timeout_add_local(std::time::Duration::from_millis(200), move || {
        if !crate::app_state::is_window_visible() || !outer_anim.is_mapped() {
            return glib::ControlFlow::Continue;
        }
        // Percorre filhos e força redraw no DrawingArea
        let mut child = flow_c2.first_child();
        while let Some(c) = child {
            redraw_in_subtree(&c);
            child = c.next_sibling();
        }
        glib::ControlFlow::Continue
    });

    outer
}

fn redraw_in_subtree(w: &gtk::Widget) {
    if let Some(da) = w.downcast_ref::<gtk::DrawingArea>() {
        da.queue_draw();
        return;
    }
    let mut child = w.first_child();
    while let Some(c) = child {
        redraw_in_subtree(&c);
        child = c.next_sibling();
    }
}

fn rebuild_storage_cards(flow: &gtk::FlowBox, state: &Rc<RefCell<AnimState>>) {
    while let Some(c) = flow.first_child() {
        flow.remove(&c);
    }
    let st = state.borrow();
    let disks: Vec<DiskUsage> = st.disks.clone();
    let expanded = st.expanded_disk.clone();
    drop(st);
    for (i, d) in disks.iter().enumerate() {
        let card = build_storage_card(
            d.clone(),
            i,
            expanded.as_deref() == Some(&d.mount),
            state.clone(),
        );
        flow.insert(&card, -1);
    }
}

fn build_storage_card(
    disk: DiskUsage,
    index: usize,
    expanded: bool,
    state: Rc<RefCell<AnimState>>,
) -> gtk::Widget {
    let faceted =
        crate::ui::faceted_card::build_simple(crate::ui::brand_theme::accent().bright, None);
    let card = faceted.content;
    card.set_orientation(gtk::Orientation::Vertical);
    card.set_spacing(6);
    faceted.widget.set_margin_top(4);
    faceted.widget.set_margin_bottom(4);

    // Top: donut + textos
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 14);
    row.set_margin_top(12);
    row.set_margin_bottom(6);
    row.set_margin_start(14);
    row.set_margin_end(14);

    let donut = gtk::DrawingArea::new();
    donut.set_size_request(110, 110);
    let st = state.clone();
    let idx = index;
    donut.set_draw_func(move |_a, cr, w, h| {
        let s = st.borrow();
        let pct = s.disk_shown.get(idx).copied().unwrap_or(0.0);
        draw_donut(cr, w as f64, h as f64, pct, s.pulse_phase);
    });
    row.append(&donut);

    let info = gtk::Box::new(gtk::Orientation::Vertical, 4);
    info.set_valign(gtk::Align::Center);
    info.set_hexpand(true);

    let label = gtk::Label::new(Some(&disk.label()));
    label.add_css_class("disk-title");
    label.set_halign(gtk::Align::Start);
    info.append(&label);

    let pct_l = gtk::Label::new(Some(&format!("{:.0}% usado", disk.percent)));
    pct_l.add_css_class("disk-pct");
    pct_l.set_halign(gtk::Align::Start);
    info.append(&pct_l);

    let used_free = gtk::Label::new(Some(&format!(
        "{:.1} GB usados · {:.1} GB livres",
        disk.used_gb(),
        disk.avail_gb()
    )));
    used_free.add_css_class("disk-sub");
    used_free.set_halign(gtk::Align::Start);
    info.append(&used_free);

    let total = gtk::Label::new(Some(&format!("Total: {:.1} GB", disk.total_gb())));
    total.add_css_class("disk-sub-dim");
    total.set_halign(gtk::Align::Start);
    info.append(&total);

    row.append(&info);
    card.append(&row);

    // Click para expandir
    let click = gtk::GestureClick::new();
    let state_c = state.clone();
    let mount_c = disk.mount.clone();
    click.connect_released(move |_, _, _, _| {
        let mut st = state_c.borrow_mut();
        if st.expanded_disk.as_deref() == Some(&mount_c) {
            st.expanded_disk = None;
        } else {
            st.expanded_disk = Some(mount_c.clone());
        }
    });
    card.add_controller(click);

    if expanded {
        let sep = gtk::Separator::new(gtk::Orientation::Horizontal);
        sep.add_css_class("dim-separator");
        card.append(&sep);
        let det = gtk::Box::new(gtk::Orientation::Vertical, 4);
        det.set_margin_top(8);
        det.set_margin_bottom(12);
        det.set_margin_start(14);
        det.set_margin_end(14);
        det.append(&detail_kv("Dispositivo", &disk.device));
        det.append(&detail_kv("Sistema de arquivos", &disk.fstype));
        det.append(&detail_kv("Ponto de montagem", &disk.mount));
        det.append(&detail_kv(
            "Usado",
            &format!("{} ({:.1}%)", format_bytes(disk.used_bytes), disk.percent),
        ));
        det.append(&detail_kv("Livre", &format_bytes(disk.avail_bytes)));
        det.append(&detail_kv("Total", &format_bytes(disk.total_bytes)));
        card.append(&det);
    }

    faceted.widget
}

fn detail_kv(k: &str, v: &str) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let kl = gtk::Label::new(Some(k));
    kl.add_css_class("detail-key");
    kl.set_halign(gtk::Align::Start);
    kl.set_width_chars(22);
    let vl = gtk::Label::new(Some(v));
    vl.add_css_class("detail-value");
    vl.set_halign(gtk::Align::Start);
    vl.set_hexpand(true);
    vl.set_wrap(true);
    vl.set_xalign(0.0);
    row.append(&kl);
    row.append(&vl);
    row
}

// ============ PROCESS ROW ============

#[derive(Clone, Copy, PartialEq)]
enum ProcMetric {
    Cpu,
    Mem,
}

fn build_process_row(
    p: &ProcessInfo,
    index: usize,
    metric: ProcMetric,
    expanded: bool,
    state: Rc<RefCell<AnimState>>,
) -> gtk::Box {
    let row_outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
    row_outer.add_css_class("proc-row");

    let top = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    top.set_margin_top(6);
    top.set_margin_bottom(6);
    top.set_margin_start(10);
    top.set_margin_end(10);

    // Rank
    let rank = gtk::Label::new(Some(&format!("{:>2}", index + 1)));
    rank.add_css_class("proc-rank");
    rank.set_valign(gtk::Align::Center);
    top.append(&rank);

    // Nome + PID
    let text = gtk::Box::new(gtk::Orientation::Vertical, 1);
    text.set_hexpand(true);
    text.set_halign(gtk::Align::Start);
    let name = gtk::Label::new(Some(&p.name));
    name.add_css_class("proc-name");
    name.set_halign(gtk::Align::Start);
    text.append(&name);
    let sub = gtk::Label::new(Some(&format!(
        "PID {} · {} · {}",
        p.pid,
        p.user,
        state_name(&p.state)
    )));
    sub.add_css_class("proc-sub");
    sub.set_halign(gtk::Align::Start);
    text.append(&sub);
    top.append(&text);

    // Bar + valor
    let metric_bar = gtk::DrawingArea::new();
    metric_bar.set_size_request(180, 20);
    metric_bar.set_valign(gtk::Align::Center);
    let idx = index;
    let st_bar = state.clone();
    metric_bar.set_draw_func(move |_a, cr, w, h| {
        let st = st_bar.borrow();
        let val = match metric {
            ProcMetric::Cpu => st.proc_shown_cpu.get(idx).copied().unwrap_or(0.0),
            ProcMetric::Mem => st.proc_shown_mem.get(idx).copied().unwrap_or(0.0),
        };
        let scale = match metric {
            ProcMetric::Cpu => 100.0_f64 * st.cpu_per_core_shown.len().max(1) as f64,
            ProcMetric::Mem => 100.0,
        };
        let color = match metric {
            ProcMetric::Cpu => crate::ui::brand_theme::accent().bright,
            ProcMetric::Mem => (0.35, 0.75, 1.0),
        };
        draw_proc_bar(cr, w as f64, h as f64, val / scale, color, st.pulse_phase);
    });
    top.append(&metric_bar);

    let val_label = match metric {
        ProcMetric::Cpu => gtk::Label::new(Some(&format!("{:.1}%", p.cpu_pct))),
        ProcMetric::Mem => gtk::Label::new(Some(&format_kb(p.mem_kb))),
    };
    val_label.add_css_class("proc-value");
    val_label.set_valign(gtk::Align::Center);
    val_label.set_width_chars(10);
    val_label.set_xalign(1.0);
    top.append(&val_label);

    row_outer.append(&top);

    // Click expande
    let click = gtk::GestureClick::new();
    let state_c = state.clone();
    let pid = p.pid;
    click.connect_released(move |_, _, _, _| {
        let mut st = state_c.borrow_mut();
        match metric {
            ProcMetric::Cpu => {
                if st.expanded_pid_cpu == Some(pid) {
                    st.expanded_pid_cpu = None;
                } else {
                    st.expanded_pid_cpu = Some(pid);
                }
            }
            ProcMetric::Mem => {
                if st.expanded_pid_mem == Some(pid) {
                    st.expanded_pid_mem = None;
                } else {
                    st.expanded_pid_mem = Some(pid);
                }
            }
        }
    });
    row_outer.add_controller(click);

    if expanded {
        let det = gtk::Box::new(gtk::Orientation::Vertical, 4);
        det.add_css_class("proc-detail");
        det.set_margin_start(44);
        det.set_margin_end(10);
        det.set_margin_top(2);
        det.set_margin_bottom(8);

        det.append(&detail_kv("PID", &p.pid.to_string()));
        det.append(&detail_kv(
            crate::i18n::t("proc_state"),
            &format!("{} ({})", p.state, state_name(&p.state)),
        ));
        det.append(&detail_kv("CPU", &format!("{:.2}%", p.cpu_pct)));
        det.append(&detail_kv(
            crate::i18n::t("proc_memory_rss"),
            &format_kb(p.mem_kb),
        ));

        // Lineage insight via witr
        if let Some(lineage) = crate::hardware::procs::trace_lineage_witr(p.pid) {
            det.append(&detail_kv("Ancestry (witr)", &lineage));
        }

        // Action buttons: Terminate (SIGTERM) & Force Kill (SIGKILL)
        let actions = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        actions.set_margin_top(6);

        let term_btn = gtk::Button::with_label("End Task");
        term_btn.add_css_class("secondary-button");
        let kill_pid = p.pid;
        term_btn.connect_clicked(move |_| {
            let _ = crate::hardware::procs::terminate_process(kill_pid, false);
        });

        let kill_btn = gtk::Button::with_label("Force Kill");
        kill_btn.add_css_class("danger-button");
        kill_btn.connect_clicked(move |_| {
            let _ = crate::hardware::procs::terminate_process(kill_pid, true);
        });

        actions.append(&term_btn);
        actions.append(&kill_btn);
        det.append(&actions);

        row_outer.append(&det);
    }

    // Note: NO per-row timer here. Process lists are rebuilt every 2s, and per-row
    // timers would leak (old row dropped, source not removed) — historically the
    // main cause of the CPU spin + RSS growth. Bars redraw through the tab-level
    // animation tick walking the subtree.

    row_outer
}

fn state_name(s: &str) -> &'static str {
    match s {
        "R" => crate::i18n::t("proc_state_running"),
        "S" => crate::i18n::t("proc_state_sleeping"),
        "D" => crate::i18n::t("proc_state_io_wait"),
        "Z" => crate::i18n::t("proc_state_zombie"),
        "T" => crate::i18n::t("proc_state_stopped"),
        "t" => crate::i18n::t("proc_state_trace_stop"),
        "X" => crate::i18n::t("proc_state_dead"),
        _ => "—",
    }
}

// ============ DRAWING ============

fn draw_big_gauge(cr: &gtk4::cairo::Context, w: f64, h: f64, value: f64, _phase: f64) {
    let cx = w / 2.0;
    let cy = h / 2.0;
    let r = w.min(h) / 2.0 - 18.0;
    let fraction = (value / 100.0).clamp(0.0, 1.0);

    // Anel de fundo
    cr.set_line_width(12.0);
    cr.set_dash(&[], 0.0);
    if crate::ui::brand_theme::is_light_mode() {
        cr.set_source_rgba(0.0, 0.0, 0.0, 0.08);
    } else {
        cr.set_source_rgba(1.0, 1.0, 1.0, 0.08);
    }
    cr.arc(cx, cy, r, 0.0, 2.0 * PI);
    let _ = cr.stroke();

    // Arco de progresso
    if fraction > 0.001 {
        let (cr0, cg0, cb0) = if fraction < 0.6 {
            crate::ui::brand_theme::accent().bright
        } else if fraction < 0.85 {
            (0.85, 0.50, 0.05)
        } else {
            (0.93, 0.26, 0.22)
        };
        cr.set_source_rgba(cr0, cg0, cb0, 1.0);
        cr.set_line_width(12.0);
        cr.set_line_cap(gtk4::cairo::LineCap::Round);
        cr.arc(cx, cy, r, -PI / 2.0, -PI / 2.0 + fraction * 2.0 * PI);
        let _ = cr.stroke();
    }

    // Texto central
    cr.set_dash(&[], 0.0);
    if crate::ui::brand_theme::is_light_mode() {
        cr.set_source_rgba(0.06, 0.09, 0.16, 1.0);
    } else {
        cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);
    }
    cr.select_font_face(
        "-apple-system, BlinkMacSystemFont, 'Inter', 'Segoe UI', 'Ubuntu', sans-serif",
        gtk4::cairo::FontSlant::Normal,
        gtk4::cairo::FontWeight::Bold,
    );
    cr.set_font_size(36.0);
    let txt = format!("{:.0}%", value);
    if let Ok(ext) = cr.text_extents(&txt) {
        cr.move_to(cx - ext.width() / 2.0, cy + ext.height() / 3.0);
        let _ = cr.show_text(&txt);
    }
}

fn draw_per_core(cr: &gtk4::cairo::Context, w: f64, h: f64, cores: &[f64], _phase: f64) {
    let is_light = crate::ui::brand_theme::is_light_mode();

    // Fundo
    crate::ui::brand_theme::set_surface_source(cr);
    cr.rectangle(0.0, 0.0, w, h);
    let _ = cr.fill();

    let margin = 14.0;
    let n = cores.len().max(1);
    let usable_w = w - margin * 2.0;
    let spacing = 6.0;
    let bar_w = ((usable_w - spacing * (n as f64 - 1.0)) / n as f64).max(6.0);
    let bar_h_max = h - margin * 2.0 - 18.0;

    for (i, &v) in cores.iter().enumerate() {
        let x = margin + i as f64 * (bar_w + spacing);
        let fill = (v / 100.0).clamp(0.0, 1.0);
        let bar_h = bar_h_max * fill;
        let y = margin + bar_h_max - bar_h;

        // Trilho do core: elegante e sutil
        if is_light {
            cr.set_source_rgba(0.0, 0.0, 0.0, 0.06);
        } else {
            cr.set_source_rgba(1.0, 1.0, 1.0, 0.07);
        }
        rounded_rect(cr, x, margin, bar_w, bar_h_max, 3.0);
        let _ = cr.fill();

        // Cor por intensidade
        let (rc, gc, bc) = if fill < 0.5 {
            crate::ui::brand_theme::accent().bright
        } else if fill < 0.8 {
            (0.85, 0.50, 0.05)
        } else {
            (0.93, 0.26, 0.22)
        };

        // Barra de atividade sólida e limpa
        if fill > 0.005 {
            cr.set_source_rgb(rc, gc, bc);
            rounded_rect(cr, x, y, bar_w, bar_h, 3.0);
            let _ = cr.fill();
        }

        // Label do Core
        if is_light {
            cr.set_source_rgba(0.3, 0.35, 0.45, 1.0);
        } else {
            cr.set_source_rgba(0.6, 0.65, 0.75, 1.0);
        }
        cr.set_font_size(10.0);
        let label = format!("{}", i);
        if let Ok(ext) = cr.text_extents(&label) {
            cr.move_to(x + (bar_w - ext.width()) / 2.0, h - 4.0);
            let _ = cr.show_text(&label);
        }
    }
}

fn draw_mem_bar(
    cr: &gtk4::cairo::Context,
    w: f64,
    h: f64,
    mi: &crate::hardware::procs::MemInfo,
    _used_shown: f64,
    phase: f64,
) {
    let margin = 2.0;
    let bw = w - margin * 2.0;
    let bh = h - margin * 2.0;

    // Trilho
    cr.set_source_rgba(0.1, 0.1, 0.12, 1.0);
    rounded_rect(cr, margin, margin, bw, bh, 6.0);
    let _ = cr.fill();

    if mi.total_kb == 0 {
        return;
    }

    // Composição: used (sem cached) + cached/buffers + available
    let used_app = mi.total_kb.saturating_sub(mi.available_kb) as f64;
    let cached = (mi.cached_kb + mi.buffers_kb) as f64;
    let total = mi.total_kb as f64;
    // Use da aplicação real = used_app - cached? Não. used_app já inclui cache parcialmente.
    // Padrão: apresentar "Apps" = total - available - cached
    let apps = (used_app - cached).max(0.0);
    let apps_w = bw * (apps / total).clamp(0.0, 1.0);
    let cached_w = bw * (cached / total).clamp(0.0, 1.0);

    let pulse = 0.85 + 0.15 * ((phase * TAU).sin() * 0.5 + 0.5);

    // Apps (azul neon)
    let grad = gtk4::cairo::LinearGradient::new(margin, margin, margin + apps_w, margin);
    grad.add_color_stop_rgba(0.0, 0.0, 0.6, 0.8, 0.95 * pulse);
    grad.add_color_stop_rgba(1.0, 0.0, 0.85, 0.95, 0.95 * pulse);
    let _ = cr.set_source(&grad);
    rounded_rect(cr, margin, margin, apps_w, bh, 6.0);
    let _ = cr.fill();

    // Cache (azul suave)
    cr.set_source_rgba(0.15, 0.45, 0.65, 0.55);
    rounded_rect(cr, margin + apps_w, margin, cached_w, bh, 0.0);
    let _ = cr.fill();

    // Texto central
    cr.set_source_rgba(1.0, 1.0, 1.0, 0.9);
    cr.select_font_face(
        "Sans",
        gtk4::cairo::FontSlant::Normal,
        gtk4::cairo::FontWeight::Bold,
    );
    cr.set_font_size(12.0);
    let txt = format!(
        "Apps {} · Cache {} · Livre {}",
        format_kb(apps as u64),
        format_kb(cached as u64),
        format_kb(mi.available_kb)
    );
    if let Ok(ext) = cr.text_extents(&txt) {
        cr.move_to(w / 2.0 - ext.width() / 2.0, h / 2.0 + ext.height() / 3.0);
        let _ = cr.show_text(&txt);
    }
}

fn draw_donut(cr: &gtk4::cairo::Context, w: f64, h: f64, pct: f64, phase: f64) {
    let cx = w / 2.0;
    let cy = h / 2.0;
    let r = w.min(h) / 2.0 - 8.0;
    let fraction = (pct / 100.0).clamp(0.0, 1.0);

    // Base
    cr.set_line_width(10.0);
    cr.set_dash(&[], 0.0);
    cr.set_source_rgba(1.0, 1.0, 1.0, 0.08);
    cr.arc(cx, cy, r, 0.0, 2.0 * PI);
    let _ = cr.stroke();

    // Arco usado
    if fraction > 0.001 {
        let (rc, gc, bc) = if fraction < 0.7 {
            crate::ui::brand_theme::accent().bright
        } else if fraction < 0.88 {
            (0.95, 0.7, 0.1)
        } else {
            (0.95, 0.25, 0.2)
        };
        let pulse = 0.8 + 0.2 * ((phase * TAU).sin() * 0.5 + 0.5);
        // Glow
        cr.set_source_rgba(rc, gc, bc, 0.22 * pulse);
        cr.set_line_width(14.0);
        cr.arc(cx, cy, r, -PI / 2.0, -PI / 2.0 + fraction * 2.0 * PI);
        let _ = cr.stroke();
        // Main
        cr.set_source_rgba(rc, gc, bc, 1.0);
        cr.set_line_width(10.0);
        cr.arc(cx, cy, r, -PI / 2.0, -PI / 2.0 + fraction * 2.0 * PI);
        let _ = cr.stroke();
    }

    // Texto central
    if crate::ui::brand_theme::is_light_mode() {
        cr.set_source_rgba(0.06, 0.09, 0.16, 1.0);
    } else {
        cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);
    }
    cr.select_font_face(
        "-apple-system, BlinkMacSystemFont, 'Inter', 'Segoe UI', 'Ubuntu', sans-serif",
        gtk4::cairo::FontSlant::Normal,
        gtk4::cairo::FontWeight::Bold,
    );
    cr.set_font_size(20.0);
    let txt = format!("{:.0}%", pct);
    if let Ok(ext) = cr.text_extents(&txt) {
        cr.move_to(cx - ext.width() / 2.0, cy + ext.height() / 3.0);
        let _ = cr.show_text(&txt);
    }
}

fn draw_proc_bar(
    cr: &gtk4::cairo::Context,
    w: f64,
    h: f64,
    fraction: f64,
    color: (f64, f64, f64),
    _phase: f64,
) {
    let f = fraction.clamp(0.0, 1.0);
    let margin = 1.0;
    let bw = w - margin * 2.0;
    let bh = h - margin * 2.0;
    let is_light = crate::ui::brand_theme::is_light_mode();
    // Trilho
    if is_light {
        cr.set_source_rgba(0.0, 0.0, 0.0, 0.08);
    } else {
        cr.set_source_rgba(0.1, 0.1, 0.12, 1.0);
    }
    rounded_rect(cr, margin, margin, bw, bh, 3.0);
    let _ = cr.fill();
    if f < 0.001 {
        return;
    }
    cr.set_source_rgba(color.0, color.1, color.2, 0.95);
    rounded_rect(cr, margin, margin, bw * f, bh, 3.0);
    let _ = cr.fill();
}

fn rounded_rect(cr: &gtk4::cairo::Context, x: f64, y: f64, w: f64, h: f64, r: f64) {
    let r = r.min(w.min(h) / 2.0);
    cr.new_sub_path();
    cr.arc(x + w - r, y + r, r, -PI / 2.0, 0.0);
    cr.arc(x + w - r, y + h - r, r, 0.0, PI / 2.0);
    cr.arc(x + r, y + h - r, r, PI / 2.0, PI);
    cr.arc(x + r, y + r, r, PI, 3.0 * PI / 2.0);
    cr.close_path();
}

/// Desenha um gauge de temperatura limpo e profissional com barra de progresso horizontal e escala
fn draw_temp_gauge(cr: &gtk4::cairo::Context, w: f64, h: f64, temp_c: f64, _phase: f64) {
    crate::ui::brand_theme::set_surface_source(cr);
    rounded_rect(cr, 0.0, 0.0, w, h, 6.0);
    let _ = cr.fill();

    let is_light = crate::ui::brand_theme::is_light_mode();

    // Valor numérico grande à esquerda
    let txt = if temp_c > 0.0 {
        format!("{:.0}°C", temp_c)
    } else {
        "—".into()
    };

    if is_light {
        cr.set_source_rgba(0.06, 0.09, 0.16, 1.0);
    } else {
        cr.set_source_rgba(0.95, 0.96, 0.98, 1.0);
    }
    cr.select_font_face(
        "-apple-system, BlinkMacSystemFont, 'Inter', 'Segoe UI', 'Ubuntu', sans-serif",
        gtk4::cairo::FontSlant::Normal,
        gtk4::cairo::FontWeight::Bold,
    );
    cr.set_font_size(36.0);
    if let Ok(ext) = cr.text_extents(&txt) {
        cr.move_to(20.0, h / 2.0 - 10.0 + ext.height() / 2.0);
        let _ = cr.show_text(&txt);
    }

    // Status label
    cr.set_font_size(12.0);
    let (status_text, rc, gc, bc) = if temp_c >= 85.0 {
        ("High (Thermal Throttle Risk)", 0.93, 0.26, 0.22)
    } else if temp_c >= 70.0 {
        ("Warm", 0.85, 0.50, 0.05)
    } else {
        ("Optimal", 0.10, 0.65, 0.38)
    };
    cr.set_source_rgb(rc, gc, bc);
    cr.move_to(20.0, h / 2.0 + 20.0);
    let _ = cr.show_text(status_text);

    // Barra de progresso horizontal
    let bar_x = 20.0;
    let bar_y = h - 28.0;
    let bar_w = w - 40.0;
    let bar_h = 8.0;

    // Track de fundo
    if is_light {
        cr.set_source_rgba(0.0, 0.0, 0.0, 0.08);
    } else {
        cr.set_source_rgba(1.0, 1.0, 1.0, 0.08);
    }
    rounded_rect(cr, bar_x, bar_y, bar_w, bar_h, 4.0);
    let _ = cr.fill();

    // Fill proporcional (20°C a 100°C)
    let frac = ((temp_c - 20.0) / 80.0).clamp(0.0, 1.0);
    if frac > 0.01 {
        cr.set_source_rgb(rc, gc, bc);
        rounded_rect(cr, bar_x, bar_y, bar_w * frac, bar_h, 4.0);
        let _ = cr.fill();
    }
}

/// Donut de VRAM
fn draw_vram_donut(
    cr: &gtk4::cairo::Context,
    w: f64,
    h: f64,
    gpu: &Option<crate::hardware::gpu::GpuMetrics>,
    pct_shown: f64,
    _phase: f64,
) {
    let cx = w / 2.0;
    let cy = h / 2.0;
    let r = w.min(h) / 2.0 - 22.0;
    let fraction = (pct_shown / 100.0).clamp(0.0, 1.0);

    let is_light = crate::ui::brand_theme::is_light_mode();

    // Track
    cr.set_line_width(12.0);
    cr.set_dash(&[], 0.0);
    if is_light {
        cr.set_source_rgba(0.0, 0.0, 0.0, 0.08);
    } else {
        cr.set_source_rgba(1.0, 1.0, 1.0, 0.08);
    }
    cr.arc(cx, cy, r, 0.0, 2.0 * PI);
    let _ = cr.stroke();

    if fraction > 0.001 {
        let (rc, gc, bc) = if fraction < 0.6 {
            (0.22, 0.68, 1.0)
        } else if fraction < 0.85 {
            (0.85, 0.50, 0.05)
        } else {
            (0.93, 0.26, 0.22)
        };
        cr.set_source_rgba(rc, gc, bc, 1.0);
        cr.set_line_width(12.0);
        cr.set_line_cap(gtk4::cairo::LineCap::Round);
        cr.arc(cx, cy, r, -PI / 2.0, -PI / 2.0 + fraction * 2.0 * PI);
        let _ = cr.stroke();
    }

    cr.set_dash(&[], 0.0);
    // Texto central: %
    if is_light {
        cr.set_source_rgba(0.06, 0.09, 0.16, 1.0);
    } else {
        cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);
    }
    cr.select_font_face(
        "-apple-system, BlinkMacSystemFont, 'Inter', 'Segoe UI', 'Ubuntu', sans-serif",
        gtk4::cairo::FontSlant::Normal,
        gtk4::cairo::FontWeight::Bold,
    );
    cr.set_font_size(30.0);
    let txt = format!("{:.0}%", pct_shown);
    if let Ok(ext) = cr.text_extents(&txt) {
        cr.move_to(cx - ext.width() / 2.0, cy);
        let _ = cr.show_text(&txt);
    }
    // MB used/total
    cr.set_source_rgba(0.85, 0.85, 0.85, 0.9);
    cr.set_font_size(13.0);
    let sub = if let Some(g) = gpu {
        format!(
            "{:.1}/{:.1} GB",
            g.vram_used_mb as f64 / 1024.0,
            g.vram_total_mb as f64 / 1024.0
        )
    } else {
        "—".into()
    };
    if let Ok(ext) = cr.text_extents(&sub) {
        cr.move_to(cx - ext.width() / 2.0, cy + 22.0);
        let _ = cr.show_text(&sub);
    }
}

/// Gauge de power draw em watts (barra com glow animado).
fn draw_power_gauge(
    cr: &gtk4::cairo::Context,
    w: f64,
    h: f64,
    power_shown: f64,
    gpu: &Option<crate::hardware::gpu::GpuMetrics>,
    phase: f64,
) {
    crate::ui::brand_theme::set_surface_source(cr);
    rounded_rect(cr, 0.0, 0.0, w, h, 4.0);
    let _ = cr.fill();

    let max_w = gpu
        .as_ref()
        .map(|g| g.power_max_w)
        .unwrap_or(130.0)
        .max(1.0);
    let frac = (power_shown / max_w).clamp(0.0, 1.0);

    // Barra horizontal
    let bar_x = 22.0;
    let bar_y = h / 2.0 + 4.0;
    let bar_w = w - 44.0;
    let bar_h = 14.0;

    cr.set_source_rgba(0.1, 0.1, 0.12, 1.0);
    rounded_rect(cr, bar_x, bar_y, bar_w, bar_h, bar_h / 2.0);
    let _ = cr.fill();

    if frac > 0.001 {
        let pulse = 0.8 + 0.2 * ((phase * TAU).sin() * 0.5 + 0.5);
        let grad = gtk4::cairo::LinearGradient::new(bar_x, 0.0, bar_x + bar_w, 0.0);
        grad.add_color_stop_rgba(0.0, 0.15, 0.65, 0.95, 1.0);
        grad.add_color_stop_rgba(0.7, 0.0, 0.8, 0.95, 1.0);
        grad.add_color_stop_rgba(1.0, 0.2, 1.0, 0.9, 1.0);
        let _ = cr.set_source(&grad);
        rounded_rect(cr, bar_x, bar_y, bar_w * frac, bar_h, bar_h / 2.0);
        let _ = cr.fill();
        // Glow
        cr.set_source_rgba(0.0, 0.8, 0.95, 0.35 * pulse);
        rounded_rect(
            cr,
            bar_x - 1.0,
            bar_y - 1.0,
            bar_w * frac + 2.0,
            bar_h + 2.0,
            bar_h / 2.0,
        );
        let _ = cr.fill();
    }

    // Texto grande "XX W"
    cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);
    cr.select_font_face(
        "Sans",
        gtk4::cairo::FontSlant::Normal,
        gtk4::cairo::FontWeight::Bold,
    );
    cr.set_font_size(40.0);
    let txt = format!("{:.1} W", power_shown);
    if let Ok(ext) = cr.text_extents(&txt) {
        cr.move_to(w / 2.0 - ext.width() / 2.0, h / 2.0 - 8.0);
        let _ = cr.show_text(&txt);
    }

    // Subtexto com limite
    if let Some(g) = gpu {
        cr.set_source_rgba(0.8, 0.8, 0.8, 0.85);
        cr.set_font_size(11.0);
        let sub = crate::i18n::tf(
            "gpu_power_sub",
            &[
                &format!("{:.0}", g.power_limit_w),
                &format!("{:.0}", g.power_max_w),
            ],
        );
        if let Ok(ext) = cr.text_extents(&sub) {
            cr.move_to(w / 2.0 - ext.width() / 2.0, bar_y + bar_h + 16.0);
            let _ = cr.show_text(&sub);
        }
    }

    // Borda sutil
    cr.set_source_rgba(0.0, 0.7, 0.9, 0.35);
    cr.set_line_width(2.0);
    rounded_rect(cr, 1.0, 1.0, w - 2.0, h - 2.0, 4.0);
    let _ = cr.stroke();
}

fn format_kb(kb: u64) -> String {
    format_bytes(kb * 1024)
}

fn format_bytes(b: u64) -> String {
    let bf = b as f64;
    let k = 1024.0;
    if bf >= k * k * k {
        format!("{:.2} GB", bf / (k * k * k))
    } else if bf >= k * k {
        format!("{:.1} MB", bf / (k * k))
    } else if bf >= k {
        format!("{:.0} KB", bf / k)
    } else {
        format!("{} B", b)
    }
}
