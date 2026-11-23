#![forbid(unsafe_code)]

//! Native Safe-Rust `eframe` / `egui` 0.31 2D/3D Mathematical Explorer Workstation (`v1.1.0`).
//! Features a complete Desktop Menu System:
//! - Persistent `egui::menu::bar` (`File`, `Domains`, `View`, `Compute`, `Help`) with cascading submenus
//! - Interactive 6-Domain Campaign & Module Launcher Hub (`AppScreen::LauncherHub`, hotkey `H`, `--skip-hub`)
//! - `Ctrl+K` / `Cmd+K` Fuzzy Command Palette across all 19 modules, presets, and exports
//! - `B` Multithreaded Scaling Benchmark Modal (`1..N` Rayon threads speedup profiler)
//! - `Z` Zen Full-Canvas Mode + Independent Left/Right Dock toggles
//! - `F1` Keyboard Shortcuts & CLI Reference Modal + About Modal

use crate::render3d::{
    export_report_svg, export_scene3d_obj, scene3d_normalization_scale, shade_color, Camera3D,
};
use crate::storage::{WorksetEnvelope, WorksetVault};
use crate::{
    execute_module, ComputationReport, DomainCategory, ExplorerParams, ModuleId, RgbaColor,
    Scene2D, Scene3D,
};
use eframe::egui::{
    self, Align2, Color32, CornerRadius, FontId, Key, PointerButton, Pos2, Rect, RichText, Sense,
    Shape, Stroke, Vec2, ViewportCommand,
};
use std::cmp::Ordering;
use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

fn to_color32(c: RgbaColor) -> Color32 {
    Color32::from_rgba_unmultiplied(c.0, c.1, c.2, c.3)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppScreen {
    LauncherHub,
    Workstation,
}

#[derive(Debug, Clone)]
pub struct BenchRow {
    pub threads: usize,
    pub elapsed_ms: f64,
    pub speedup: f64,
}

pub struct PrincipiaApp {
    pub screen: AppScreen,
    pub params: ExplorerParams,
    pub camera: Camera3D,
    pub show_3d_mode: bool,
    pub auto_orbit: bool,
    pub zen_mode: bool,
    pub show_left_nav: bool,
    pub show_right_inspector: bool,
    pub show_command_palette: bool,
    pub palette_query: String,
    pub show_bench_modal: bool,
    pub bench_rows: Vec<BenchRow>,
    pub show_shortcuts_modal: bool,
    pub show_about_modal: bool,
    pub latest_report: Option<ComputationReport>,
    pub is_computing: bool,
    pub status_banner: String,
    tx: Sender<ExplorerParams>,
    rx: Receiver<ComputationReport>,
    vault: Option<WorksetVault>,
}

impl PrincipiaApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        initial_module: ModuleId,
        threads_override: Option<usize>,
        skip_hub: bool,
    ) -> Self {
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = Color32::from_rgb(11, 17, 32);
        visuals.window_fill = Color32::from_rgb(15, 23, 42);
        visuals.widgets.noninteractive.corner_radius = CornerRadius::same(8);
        visuals.widgets.inactive.corner_radius = CornerRadius::same(6);
        visuals.widgets.hovered.corner_radius = CornerRadius::same(6);
        visuals.widgets.active.corner_radius = CornerRadius::same(6);
        cc.egui_ctx.set_visuals(visuals);

        let (job_tx, job_rx) = mpsc::channel::<ExplorerParams>();
        let (res_tx, res_rx) = mpsc::channel::<ComputationReport>();

        let ctx_clone = cc.egui_ctx.clone();
        thread::spawn(move || {
            while let Ok(params) = job_rx.recv() {
                let report = execute_module(&params);
                let _ = res_tx.send(report);
                ctx_clone.request_repaint();
            }
        });

        let mut params = ExplorerParams::default_for(initial_module);
        if let Some(t) = threads_override {
            params.threads = t.clamp(1, 128);
        }

        let vault = WorksetVault::open(&PathBuf::from("principia_worksets.db"))
            .or_else(|_| WorksetVault::open_in_memory())
            .ok();

        let mut app = Self {
            screen: if skip_hub {
                AppScreen::Workstation
            } else {
                AppScreen::LauncherHub
            },
            show_3d_mode: initial_module.is_3d_primary(),
            params,
            camera: Camera3D::default(),
            auto_orbit: false,
            zen_mode: false,
            show_left_nav: true,
            show_right_inspector: true,
            show_command_palette: false,
            palette_query: String::new(),
            show_bench_modal: false,
            bench_rows: Vec::new(),
            show_shortcuts_modal: false,
            show_about_modal: false,
            latest_report: None,
            is_computing: false,
            status_banner: "Ready — Press Ctrl+K for Command Palette, H for Domain Hub, F1 for Shortcuts.".to_string(),
            tx: job_tx,
            rx: res_rx,
            vault,
        };
        app.dispatch_compute();
        app
    }

    pub fn dispatch_compute(&mut self) {
        self.is_computing = true;
        self.status_banner = format!(
            "Running {} across {} Rayon worker threads...",
            self.params.module.short_name(),
            self.params.threads
        );
        let _ = self.tx.send(self.params.clone());
    }

    fn select_module(&mut self, module: ModuleId) {
        let keep_threads = self.params.threads;
        self.params = ExplorerParams::default_for(module);
        self.params.threads = keep_threads;
        self.show_3d_mode = module.is_3d_primary();
        self.camera = Camera3D::default();
        self.screen = AppScreen::Workstation;
        self.dispatch_compute();
    }

    fn run_scaling_benchmark(&mut self) {
        let max_t = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(8)
            .clamp(4, 32);
        let mut rows = Vec::new();
        let mut base_ms = 0.01f64;
        let mut t = 1usize;
        while t <= max_t {
            let mut p = self.params.clone();
            p.threads = t;
            let rep = execute_module(&p);
            if t == 1 {
                base_ms = rep.elapsed_ms.max(0.01);
            }
            rows.push(BenchRow {
                threads: t,
                elapsed_ms: rep.elapsed_ms,
                speedup: base_ms / rep.elapsed_ms.max(0.01),
            });
            if t == max_t {
                break;
            }
            t = (t * 2).min(max_t);
        }
        self.bench_rows = rows;
        self.show_bench_modal = true;
        self.status_banner = format!(
            "Completed 1..={max_t} Rayon thread scaling benchmark on {}.",
            self.params.module.short_name()
        );
    }

    fn export_svg_current(&mut self) {
        if let Some(ref rep) = self.latest_report {
            let path = format!("{}_view.svg", rep.module.code());
            let svg = export_report_svg(rep, 1280, 800);
            if fs::write(&path, svg).is_ok() {
                self.status_banner = format!("Saved vector plot to `{path}`");
            }
        }
    }

    fn export_obj_current(&mut self) {
        if let Some(ref rep) = self.latest_report {
            let path = format!("{}_mesh.obj", rep.module.code());
            let obj = export_scene3d_obj(&rep.scene_3d);
            if fs::write(&path, obj).is_ok() {
                self.status_banner = format!("Saved 3D Wavefront mesh to `{path}`");
            }
        }
    }

    fn export_json_current(&mut self) {
        if let Some(ref rep) = self.latest_report {
            let path = format!("{}_report.json", rep.module.code());
            if let Ok(json) = serde_json::to_string_pretty(rep) {
                if fs::write(&path, json).is_ok() {
                    self.status_banner = format!("Saved ComputationReport JSON to `{path}`");
                }
            }
        }
    }

    fn cli_command_string(&self) -> String {
        let mut cmd = format!(
            "cargo run --release -- cli run --module {} --threads {} --preset {} --n {} --span {}",
            self.params.module.code(),
            self.params.threads,
            self.params.preset_index,
            self.params.primary_n,
            self.params.span
        );
        if !self.params.expression_input.trim().is_empty() {
            cmd.push_str(&format!(" --expr \"{}\"", self.params.expression_input.trim()));
        }
        cmd
    }

    fn save_workset_slot(&mut self, slot: i64) {
        let env = WorksetEnvelope::new(
            format!("Slot {slot} — {}", self.params.module.short_name()),
            self.params.clone(),
            self.camera,
        );
        if let Some(ref v) = self.vault {
            if v.save_slot(slot, &env).is_ok() {
                let path = format!("slot_{slot}.pworkset.json");
                if let Ok(json) = serde_json::to_string_pretty(&env) {
                    let _ = fs::write(&path, json);
                }
                self.status_banner = format!(
                    "Saved SQLite WAL Slot {slot} + `{path}` (SHA-256: {}...)",
                    &env.sha256_hex[..12]
                );
            }
        }
    }

    fn load_workset_slot(&mut self, slot: i64) {
        if let Some(ref v) = self.vault {
            if let Ok(Some(env)) = v.load_slot(slot) {
                self.params = env.params;
                self.camera = env.camera;
                self.show_3d_mode = self.params.module.is_3d_primary();
                self.screen = AppScreen::Workstation;
                self.status_banner = format!(
                    "Loaded verified Workset '{}' (SHA-256 OK)",
                    env.slot_name
                );
                self.dispatch_compute();
            } else {
                self.status_banner = format!("Slot {slot} is empty or checksum failed.");
            }
        }
    }

    fn draw_persistent_menu_bar(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        egui::menu::bar(ui, |ui| {
            ui.label(
                RichText::new("PRINCIPIA")
                    .strong()
                    .monospace()
                    .color(Color32::from_rgb(56, 189, 248)),
            );
            ui.separator();

            // 1. FILE MENU
            ui.menu_button("File", |ui| {
                if ui.button("Domain Launcher Hub... (H)").clicked() {
                    self.screen = match self.screen {
                        AppScreen::LauncherHub => AppScreen::Workstation,
                        AppScreen::Workstation => AppScreen::LauncherHub,
                    };
                    ui.close_menu();
                }
                if ui.button("Reset Current Module Defaults").clicked() {
                    self.select_module(self.params.module);
                    ui.close_menu();
                }
                ui.separator();
                ui.menu_button("Save SQLite Workset Slot", |ui| {
                    for slot in 1..=4 {
                        let hk = if slot == 1 { " (F5)" } else { "" };
                        if ui.button(format!("Save Slot {slot}{hk}")).clicked() {
                            self.save_workset_slot(slot);
                            ui.close_menu();
                        }
                    }
                });
                ui.menu_button("Load SQLite Workset Slot", |ui| {
                    for slot in 1..=4 {
                        let hk = if slot == 1 { " (F9)" } else { "" };
                        if ui.button(format!("Load Slot {slot}{hk}")).clicked() {
                            self.load_workset_slot(slot);
                            ui.close_menu();
                        }
                    }
                });
                ui.separator();
                ui.menu_button("Export Artifacts", |ui| {
                    if ui.button("Export Vector Plot (.SVG)").clicked() {
                        self.export_svg_current();
                        ui.close_menu();
                    }
                    if ui.button("Export 3D Wavefront Mesh (.OBJ)").clicked() {
                        self.export_obj_current();
                        ui.close_menu();
                    }
                    if ui.button("Export ComputationReport (.JSON)").clicked() {
                        self.export_json_current();
                        ui.close_menu();
                    }
                });
                ui.separator();
                if ui.button("Quit Workstation").clicked() {
                    ctx.send_viewport_cmd(ViewportCommand::Close);
                    ui.close_menu();
                }
            });

            // 2. DOMAINS CASCADING MENU (All 6 Domains & 19 Modules)
            ui.menu_button("Domains", |ui| {
                for cat in DomainCategory::ALL {
                    ui.menu_button(cat.title(), |ui| {
                        for m in ModuleId::ALL.iter().copied().filter(|m| m.category() == cat) {
                            let dim = if m.is_3d_primary() { "3D" } else { "2D" };
                            let active = self.params.module == m;
                            if ui
                                .selectable_label(active, format!("[{dim}] {}", m.short_name()))
                                .clicked()
                            {
                                self.select_module(m);
                                ui.close_menu();
                            }
                        }
                    });
                }
            });

            // 3. VIEW MENU
            ui.menu_button("View", |ui| {
                if ui
                    .selectable_label(self.show_3d_mode, "3D Interactive Viewport (3)")
                    .clicked()
                {
                    self.show_3d_mode = true;
                    self.screen = AppScreen::Workstation;
                    ui.close_menu();
                }
                if ui
                    .selectable_label(!self.show_3d_mode, "2D Scientific Plot (2)")
                    .clicked()
                {
                    self.show_3d_mode = false;
                    self.screen = AppScreen::Workstation;
                    ui.close_menu();
                }
                ui.separator();
                if ui
                    .checkbox(&mut self.zen_mode, "Zen Full-Canvas Mode (Z)")
                    .changed()
                {
                    self.screen = AppScreen::Workstation;
                    ui.close_menu();
                }
                ui.checkbox(&mut self.show_left_nav, "Show Left Domain Navigator");
                ui.checkbox(&mut self.show_right_inspector, "Show Right Inspector Dock");
                ui.separator();
                ui.checkbox(&mut self.auto_orbit, "Calm 3D Auto-Orbit (O)");
                if ui.button("Reset 3D Camera (R)").clicked() {
                    self.camera = Camera3D::default();
                    ui.close_menu();
                }
            });

            // 4. COMPUTE MENU
            ui.menu_button("Compute", |ui| {
                if ui.button("Re-Run Parallel Compute (Space)").clicked() {
                    self.dispatch_compute();
                    ui.close_menu();
                }
                if ui.button("Command Palette... (Ctrl+K)").clicked() {
                    self.show_command_palette = true;
                    ui.close_menu();
                }
                ui.separator();
                ui.menu_button("Worker Thread Pool", |ui| {
                    for t in [1usize, 2, 4, 8, 16, 32] {
                        let selected = self.params.threads == t;
                        if ui.selectable_label(selected, format!("{t} Rayon Threads")).clicked() {
                            self.params.threads = t;
                            self.dispatch_compute();
                            ui.close_menu();
                        }
                    }
                });
                ui.separator();
                if ui.button("Multithreaded Scaling Benchmark... (B)").clicked() {
                    self.run_scaling_benchmark();
                    ui.close_menu();
                }
            });

            // 5. HELP MENU
            ui.menu_button("Help", |ui| {
                if ui.button("Keyboard Shortcuts & CLI Guide... (F1)").clicked() {
                    self.show_shortcuts_modal = true;
                    ui.close_menu();
                }
                if ui.button("Copy Equivalent CLI Command").clicked() {
                    ctx.copy_text(self.cli_command_string());
                    self.status_banner = "Copied headless CLI command to clipboard.".to_string();
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("About Principia Studio v1.1.0...").clicked() {
                    self.show_about_modal = true;
                    ui.close_menu();
                }
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Ctrl+K Palette").clicked() {
                    self.show_command_palette = !self.show_command_palette;
                }
                let hub_lbl = match self.screen {
                    AppScreen::LauncherHub => "Return to Canvas (H)",
                    AppScreen::Workstation => "Launcher Hub (H)",
                };
                if ui.button(hub_lbl).clicked() {
                    self.screen = match self.screen {
                        AppScreen::LauncherHub => AppScreen::Workstation,
                        AppScreen::Workstation => AppScreen::LauncherHub,
                    };
                }
            });
        });
    }

    fn draw_launcher_hub(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                ui.heading(
                    RichText::new("Principia Studio v1.1.0 — 6-Domain Computational Mathematics Hub")
                        .color(Color32::from_rgb(248, 250, 252)),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .button(
                            RichText::new("Open Active Workstation ->")
                                .strong()
                                .color(Color32::from_rgb(52, 211, 153)),
                        )
                        .clicked()
                    {
                        self.screen = AppScreen::Workstation;
                    }
                });
            });
            ui.label(
                RichText::new(
                    "Select any of the 19 multithreaded modules below to launch its 2D or 3D interactive observatory, or resume a saved SQLite WAL Workset slot.",
                )
                .color(Color32::from_rgb(148, 163, 184)),
            );
            ui.add_space(8.0);

            // Pre-flight thread & Workset Resume Bar
            egui::Frame::group(ui.style())
                .fill(Color32::from_rgb(15, 23, 42))
                .corner_radius(CornerRadius::same(8))
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(RichText::new("Pre-Flight Rayon Thread Pool:").strong());
                        ui.add(egui::Slider::new(&mut self.params.threads, 1..=32));
                        ui.separator();
                        ui.label(RichText::new("Quick Resume SQLite Slots:").strong());
                        for s in 1..=4 {
                            if ui.button(format!("Load Slot {s}")).clicked() {
                                self.load_workset_slot(s);
                            }
                        }
                    });
                });

            ui.add_space(10.0);

            // 2-Column Domain Card Grid
            ui.columns(2, |cols| {
                for (idx, cat) in DomainCategory::ALL.iter().copied().enumerate() {
                    let col_ui = &mut cols[idx % 2];
                    egui::Frame::group(col_ui.style())
                        .fill(Color32::from_rgb(15, 23, 42))
                        .corner_radius(CornerRadius::same(8))
                        .show(col_ui, |ui| {
                            ui.label(
                                RichText::new(cat.title())
                                    .strong()
                                    .size(14.0)
                                    .color(Color32::from_rgb(250, 204, 21)),
                            );
                            ui.add_space(4.0);
                            for m in ModuleId::ALL.iter().copied().filter(|m| m.category() == cat) {
                                let dim = if m.is_3d_primary() { "3D" } else { "2D" };
                                ui.horizontal(|ui| {
                                    if ui
                                        .button(
                                            RichText::new(format!("[{dim}] {}", m.short_name()))
                                                .strong()
                                                .color(Color32::from_rgb(56, 189, 248)),
                                        )
                                        .clicked()
                                    {
                                        self.select_module(m);
                                    }
                                });
                                ui.label(
                                    RichText::new(m.subtitle())
                                        .size(11.0)
                                        .color(Color32::from_rgb(148, 163, 184)),
                                );
                                ui.add_space(4.0);
                            }
                        });
                    col_ui.add_space(8.0);
                }
            });
        });
    }

    fn draw_modals(&mut self, ctx: &egui::Context) {
        // 1. Ctrl+K Command Palette Modal
        if self.show_command_palette {
            let mut open = self.show_command_palette;
            let mut chosen_mod: Option<ModuleId> = None;
            let mut trigger_bench = false;
            let mut trigger_svg = false;
            let mut trigger_obj = false;
            let mut toggle_zen = false;

            egui::Window::new("Command Palette (Ctrl+K)")
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .default_width(520.0)
                .anchor(Align2::CENTER_TOP, [0.0, 68.0])
                .show(ctx, |ui| {
                    let resp = ui.add(
                        egui::TextEdit::singleline(&mut self.palette_query)
                            .hint_text("Type module code (`n1`..`g3`), topic (`sieve`, `groebner`, `knot`, `tda`), or action..."),
                    );
                    resp.request_focus();
                    ui.separator();
                    let q = self.palette_query.trim().to_ascii_lowercase();
                    egui::ScrollArea::vertical().max_height(320.0).show(ui, |ui| {
                        for m in ModuleId::ALL {
                            let hay = format!(
                                "{} {} {}",
                                m.code(),
                                m.short_name().to_ascii_lowercase(),
                                m.subtitle().to_ascii_lowercase()
                            );
                            if q.is_empty() || hay.contains(&q) {
                                let dim = if m.is_3d_primary() { "3D" } else { "2D" };
                                if ui
                                    .button(format!(
                                        "[{:>2} | {dim}] {} — {}",
                                        m.code(),
                                        m.short_name(),
                                        m.category().title()
                                    ))
                                    .clicked()
                                {
                                    chosen_mod = Some(m);
                                }
                            }
                        }
                        ui.separator();
                        if (q.is_empty() || "benchmark threads scaling".contains(&q))
                            && ui.button("[Action] Run Multithreaded Scaling Benchmark (B)").clicked()
                        {
                            trigger_bench = true;
                        }
                        if (q.is_empty() || "export svg vector plot".contains(&q))
                            && ui.button("[Action] Export Current Visualization (.SVG)").clicked()
                        {
                            trigger_svg = true;
                        }
                        if (q.is_empty() || "export obj 3d mesh".contains(&q))
                            && ui.button("[Action] Export 3D Geometry (.OBJ)").clicked()
                        {
                            trigger_obj = true;
                        }
                        if (q.is_empty() || "zen fullscreen toggle".contains(&q))
                            && ui.button("[View] Toggle Zen Full-Canvas Mode (Z)").clicked()
                        {
                            toggle_zen = true;
                        }
                    });
                });

            if let Some(m) = chosen_mod {
                self.select_module(m);
                open = false;
            }
            if trigger_bench {
                self.run_scaling_benchmark();
                open = false;
            }
            if trigger_svg {
                self.export_svg_current();
                open = false;
            }
            if trigger_obj {
                self.export_obj_current();
                open = false;
            }
            if toggle_zen {
                self.zen_mode = !self.zen_mode;
                self.screen = AppScreen::Workstation;
                open = false;
            }
            self.show_command_palette = open;
        }

        // 2. Multithreaded Scaling Benchmark Modal
        if self.show_bench_modal {
            let mut open = self.show_bench_modal;
            let mut rerun = false;
            let rows_snapshot = self.bench_rows.clone();
            let mod_name = self.params.module.short_name();

            egui::Window::new(format!("Multithreaded Rayon Scaling — {mod_name}"))
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .default_width(440.0)
                .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label("Measured wall-clock execution latency across dedicated Rayon thread pools:");
                    ui.add_space(6.0);
                    egui::Grid::new("bench_grid").striped(true).show(ui, |ui| {
                        ui.strong("Worker Threads");
                        ui.strong("Elapsed (ms)");
                        ui.strong("Speedup vs 1T");
                        ui.end_row();
                        for r in &rows_snapshot {
                            ui.monospace(format!("{} threads", r.threads));
                            ui.monospace(format!("{:.2} ms", r.elapsed_ms));
                            ui.monospace(format!("{:.2}x", r.speedup));
                            ui.end_row();
                        }
                    });
                    ui.add_space(8.0);
                    if ui.button("Re-Run Scaling Sweep").clicked() {
                        rerun = true;
                    }
                });

            self.show_bench_modal = open;
            if rerun {
                self.run_scaling_benchmark();
            }
        }

        // 3. F1 Keyboard Shortcuts & CLI Reference Modal
        if self.show_shortcuts_modal {
            let mut open = self.show_shortcuts_modal;
            egui::Window::new("Keyboard Shortcuts & CLI Quick Reference (F1)")
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .default_width(480.0)
                .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    egui::Grid::new("hk_grid").striped(true).show(ui, |ui| {
                        for (k, desc) in [
                            ("Ctrl+K / Cmd+K", "Open fuzzy Command Palette across all 19 modules"),
                            ("H", "Toggle 6-Domain Startup & Campaign Launcher Hub"),
                            ("Z", "Toggle Zen Full-Canvas Mode (hide side docks)"),
                            ("2 / 3", "Switch between 2D Scientific Plot & 3D Viewport"),
                            ("O / R", "Toggle Calm 3D Auto-Orbit / Reset 3D Camera"),
                            ("Space", "Re-dispatch multithreaded Rayon computation"),
                            ("B", "Run 1..N thread scaling benchmark sweep"),
                            ("F5 / F9", "Quick-Save / Quick-Load SQLite WAL Workset Slot 1"),
                            ("F1 / Esc", "Open Shortcuts Guide / Close active modal"),
                        ] {
                            ui.monospace(RichText::new(k).color(Color32::from_rgb(56, 189, 248)));
                            ui.label(desc);
                            ui.end_row();
                        }
                    });
                    ui.separator();
                    ui.label(RichText::new("Headless CLI Examples:").strong());
                    ui.code("cargo run --release -- --skip-hub\ncargo run --release -- cli list\ncargo run --release -- cli bench --module n4 --max-threads 16");
                });
            self.show_shortcuts_modal = open;
        }

        // 4. About Principia Studio Modal
        if self.show_about_modal {
            let mut open = self.show_about_modal;
            egui::Window::new("About Principia Studio v1.1.0")
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .default_width(440.0)
                .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.heading("Principia Studio v1.1.0");
                    ui.label("Multithreaded Safe-Rust Computational Mathematics Observatory");
                    ui.separator();
                    ui.label("• Safety: `#![forbid(unsafe_code)]` enforced across all 13 Rust modules.");
                    ui.label("• Parallelism: Dedicated `rayon` work-stealing thread pools + non-blocking `mpsc` GUI worker bridge.");
                    ui.label("• Scope: 6 Mathematical Domains, 19 Interactive 2D/3D Modules (`N1..G3`).");
                    ui.label("• Persistence: Bundled `rusqlite` WAL slots (`F5`/`F9`) + SHA-256 `.pworkset.json` envelopes.");
                });
            self.show_about_modal = open;
        }
    }

    fn draw_scene_2d(&self, ui: &mut egui::Ui, scene: &Scene2D) {
        let avail = ui.available_size_before_wrap();
        let size = Vec2::new(avail.x.max(320.0), avail.y.max(280.0));
        let (_resp, painter) = ui.allocate_painter(size, Sense::hover());
        let rect = painter.clip_rect();

        painter.rect_filled(rect, CornerRadius::same(8), Color32::from_rgb(9, 15, 29));

        let pad = 48.0f32;
        let plot = Rect::from_min_max(
            Pos2::new(rect.min.x + pad, rect.min.y + 38.0),
            Pos2::new(rect.max.x - 24.0, rect.max.y - 34.0),
        );

        let mut min_x = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_y = f64::NEG_INFINITY;

        for p in &scene.points {
            min_x = min_x.min(p.x);
            max_x = max_x.max(p.x);
            min_y = min_y.min(p.y);
            max_y = max_y.max(p.y);
        }
        for l in &scene.polylines {
            for &(x, y) in &l.points {
                min_x = min_x.min(x);
                max_x = max_x.max(x);
                min_y = min_y.min(y);
                max_y = max_y.max(y);
            }
        }
        for r in &scene.rects {
            min_x = min_x.min(r.x0.min(r.x1));
            max_x = max_x.max(r.x0.max(r.x1));
            min_y = min_y.min(r.y0.min(r.y1));
            max_y = max_y.max(r.y0.max(r.y1));
        }
        if !min_x.is_finite() {
            min_x = -1.0;
            max_x = 1.0;
            min_y = -1.0;
            max_y = 1.0;
        }
        let dx = (max_x - min_x).max(1e-9);
        let dy = (max_y - min_y).max(1e-9);

        let to_screen = |x: f64, y: f64| -> Pos2 {
            let sx = plot.min.x + (((x - min_x) / dx) as f32) * plot.width();
            let sy = plot.max.y - (((y - min_y) / dy) as f32) * plot.height();
            Pos2::new(sx, sy)
        };

        for g in 0..=4 {
            let t = (g as f32) / 4.0;
            let gx = plot.min.x + t * plot.width();
            let gy = plot.min.y + t * plot.height();
            painter.line_segment(
                [Pos2::new(gx, plot.min.y), Pos2::new(gx, plot.max.y)],
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(51, 65, 85, 80)),
            );
            painter.line_segment(
                [Pos2::new(plot.min.x, gy), Pos2::new(plot.max.x, gy)],
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(51, 65, 85, 80)),
            );
        }

        painter.text(
            Pos2::new(rect.min.x + 16.0, rect.min.y + 16.0),
            Align2::LEFT_CENTER,
            &scene.title,
            FontId::monospace(13.5),
            Color32::from_rgb(241, 245, 249),
        );

        for r in &scene.rects {
            let a = to_screen(r.x0.min(r.x1), r.y0.max(r.y1));
            let b = to_screen(r.x0.max(r.x1), r.y0.min(r.y1));
            let rr = Rect::from_min_max(a, b);
            painter.rect_filled(rr, CornerRadius::same(3), to_color32(r.fill));
            let border = vec![
                rr.left_top(),
                rr.right_top(),
                rr.right_bottom(),
                rr.left_bottom(),
            ];
            painter.add(Shape::closed_line(
                border,
                Stroke::new(1.0_f32, to_color32(r.stroke)),
            ));
            if let Some(ref lbl) = r.label {
                painter.text(
                    Pos2::new(rr.min.x + 4.0, rr.min.y + 10.0),
                    Align2::LEFT_CENTER,
                    lbl,
                    FontId::monospace(10.0),
                    Color32::from_rgb(226, 232, 240),
                );
            }
        }

        for line in &scene.polylines {
            if line.points.len() < 2 {
                continue;
            }
            let pts: Vec<Pos2> = line.points.iter().map(|&(x, y)| to_screen(x, y)).collect();
            let stroke = Stroke::new(line.width, to_color32(line.color));
            if line.closed {
                painter.add(Shape::closed_line(pts, stroke));
            } else {
                painter.add(Shape::line(pts, stroke));
            }
        }

        for p in &scene.points {
            let sp = to_screen(p.x, p.y);
            painter.circle_filled(sp, p.radius, to_color32(p.color));
            if let Some(ref lbl) = p.label {
                painter.text(
                    Pos2::new(sp.x + p.radius + 4.0, sp.y - 2.0),
                    Align2::LEFT_CENTER,
                    lbl,
                    FontId::monospace(10.0),
                    Color32::from_rgb(203, 213, 225),
                );
            }
        }

        painter.text(
            Pos2::new(plot.center().x, rect.max.y - 12.0),
            Align2::CENTER_CENTER,
            &scene.x_label,
            FontId::monospace(11.0),
            Color32::from_rgb(148, 163, 184),
        );
    }

    fn draw_scene_3d(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, scene: &Scene3D) {
        let avail = ui.available_size_before_wrap();
        let size = Vec2::new(avail.x.max(320.0), avail.y.max(280.0));
        let (response, painter) = ui.allocate_painter(size, Sense::click_and_drag());
        let rect = painter.clip_rect();

        if response.dragged_by(PointerButton::Primary) {
            let d = response.drag_delta();
            self.camera.yaw += (d.x as f64) * 0.0085;
            self.camera.pitch = (self.camera.pitch + (d.y as f64) * 0.0085).clamp(-1.48, 1.48);
        } else if response.dragged_by(PointerButton::Secondary) {
            let d = response.drag_delta();
            self.camera.pan_x += d.x as f64;
            self.camera.pan_y += d.y as f64;
        }
        if response.hovered() {
            let scroll = ctx.input(|i| i.raw_scroll_delta.y);
            if scroll.abs() > 0.1 {
                self.camera.zoom =
                    (self.camera.zoom * (1.0 + (scroll as f64) * 0.0015)).clamp(0.35, 3.5);
            }
        }
        if self.auto_orbit {
            self.camera.yaw += 0.006;
            ctx.request_repaint();
        }

        painter.rect_filled(rect, CornerRadius::same(8), Color32::from_rgb(8, 13, 26));

        let (center, scale) = scene3d_normalization_scale(scene);
        self.camera.target = center;
        let w = rect.width() as f64;
        let h = rect.height() as f64;

        let to_pos = |(sx, sy): (f64, f64)| -> Pos2 {
            Pos2::new(rect.min.x + (sx as f32), rect.min.y + (sy as f32))
        };

        for enc in &scene.enclosures {
            let c = enc.center;
            let he = enc.half_extents;
            let corners = [
                [c[0] - he[0], c[1] - he[1], c[2] - he[2]],
                [c[0] + he[0], c[1] - he[1], c[2] - he[2]],
                [c[0] + he[0], c[1] - he[1], c[2] + he[2]],
                [c[0] - he[0], c[1] - he[1], c[2] + he[2]],
                [c[0] - he[0], c[1] + he[1], c[2] - he[2]],
                [c[0] + he[0], c[1] + he[1], c[2] - he[2]],
                [c[0] + he[0], c[1] + he[1], c[2] + he[2]],
                [c[0] - he[0], c[1] + he[1], c[2] + he[2]],
            ];
            let box_edges = [
                (0, 1),
                (1, 2),
                (2, 3),
                (3, 0),
                (4, 5),
                (5, 6),
                (6, 7),
                (7, 4),
                (0, 4),
                (1, 5),
                (2, 6),
                (3, 7),
            ];
            let col = Color32::from_rgba_unmultiplied(enc.color.0, enc.color.1, enc.color.2, 65);
            for (u, v) in box_edges {
                if let (Some(p0), Some(p1)) = (
                    self.camera.project(corners[u], w, h, scale),
                    self.camera.project(corners[v], w, h, scale),
                ) {
                    painter.line_segment(
                        [to_pos((p0.0, p0.1)), to_pos((p1.0, p1.1))],
                        Stroke::new(1.0_f32, col),
                    );
                }
            }
            if let Some(lbl_p) =
                self.camera
                    .project([c[0] - he[0], c[1] + he[1], c[2]], w, h, scale)
            {
                painter.text(
                    to_pos((lbl_p.0, lbl_p.1 - 8.0)),
                    Align2::LEFT_BOTTOM,
                    &enc.label,
                    FontId::monospace(11.0),
                    to_color32(enc.color),
                );
            }
        }

        enum Prim3 {
            Tri {
                depth: f64,
                pts: [Pos2; 3],
                color: Color32,
            },
            Seg {
                depth: f64,
                a: Pos2,
                b: Pos2,
                width: f32,
                color: Color32,
            },
            Sphere {
                depth: f64,
                center: Pos2,
                radius: f32,
                color: Color32,
                label: Option<String>,
            },
        }

        let mut prims =
            Vec::with_capacity(scene.faces.len() + scene.edges.len() + scene.nodes.len());
        for f in &scene.faces {
            if let (Some(p0), Some(p1), Some(p2)) = (
                self.camera.project(f.verts[0], w, h, scale),
                self.camera.project(f.verts[1], w, h, scale),
                self.camera.project(f.verts[2], w, h, scale),
            ) {
                let shaded = shade_color(f.color, f.normal);
                prims.push(Prim3::Tri {
                    depth: (p0.2 + p1.2 + p2.2) / 3.0,
                    pts: [
                        to_pos((p0.0, p0.1)),
                        to_pos((p1.0, p1.1)),
                        to_pos((p2.0, p2.1)),
                    ],
                    color: to_color32(shaded),
                });
            }
        }
        for e in &scene.edges {
            if let (Some(p0), Some(p1)) = (
                self.camera.project(e.a, w, h, scale),
                self.camera.project(e.b, w, h, scale),
            ) {
                prims.push(Prim3::Seg {
                    depth: 0.5 * (p0.2 + p1.2),
                    a: to_pos((p0.0, p0.1)),
                    b: to_pos((p1.0, p1.1)),
                    width: e.width,
                    color: to_color32(e.color),
                });
            }
        }
        for n in &scene.nodes {
            if let Some(p) = self.camera.project(n.pos, w, h, scale) {
                let r = (n.radius * 5.2 / (p.2 as f32)).clamp(1.8, 16.0);
                prims.push(Prim3::Sphere {
                    depth: p.2,
                    center: to_pos((p.0, p.1)),
                    radius: r,
                    color: to_color32(n.color),
                    label: n.label.clone(),
                });
            }
        }

        prims.sort_by(|a, b| {
            let da = match a {
                Prim3::Tri { depth, .. }
                | Prim3::Seg { depth, .. }
                | Prim3::Sphere { depth, .. } => *depth,
            };
            let db = match b {
                Prim3::Tri { depth, .. }
                | Prim3::Seg { depth, .. }
                | Prim3::Sphere { depth, .. } => *depth,
            };
            db.partial_cmp(&da).unwrap_or(Ordering::Equal)
        });

        for prim in prims {
            match prim {
                Prim3::Tri { pts, color, .. } => {
                    painter.add(Shape::convex_polygon(
                        vec![pts[0], pts[1], pts[2]],
                        color,
                        Stroke::NONE,
                    ));
                }
                Prim3::Seg {
                    a, b, width, color, ..
                } => {
                    painter.line_segment([a, b], Stroke::new(width, color));
                }
                Prim3::Sphere {
                    center,
                    radius,
                    color,
                    label,
                    ..
                } => {
                    painter.circle_filled(center, radius, color);
                    if let Some(lbl) = label {
                        painter.text(
                            Pos2::new(center.x + radius + 4.0, center.y),
                            Align2::LEFT_CENTER,
                            lbl,
                            FontId::monospace(10.5),
                            Color32::from_rgb(226, 232, 240),
                        );
                    }
                }
            }
        }

        painter.text(
            Pos2::new(rect.min.x + 16.0, rect.min.y + 18.0),
            Align2::LEFT_CENTER,
            &scene.title,
            FontId::monospace(13.5),
            Color32::from_rgb(248, 250, 252),
        );
        painter.text(
            Pos2::new(rect.min.x + 16.0, rect.max.y - 14.0),
            Align2::LEFT_CENTER,
            "3D Controls: Left-Drag Orbit | Right-Drag Pan | Scroll Zoom | Z Zen Mode",
            FontId::monospace(10.5),
            Color32::from_rgb(148, 163, 184),
        );
    }
}

impl eframe::App for PrincipiaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Ok(report) = self.rx.try_recv() {
            self.status_banner = format!(
                "Completed {} in {:.2} ms across {} Rayon threads.",
                report.module.short_name(),
                report.elapsed_ms,
                report.threads_used
            );
            self.is_computing = false;
            self.latest_report = Some(report);
        }

        // Global Function & Command Hotkeys
        if ctx.input(|i| i.modifiers.command && i.key_pressed(Key::K)) {
            self.show_command_palette = !self.show_command_palette;
        }
        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            self.show_command_palette = false;
            self.show_bench_modal = false;
            self.show_shortcuts_modal = false;
            self.show_about_modal = false;
        }
        if ctx.input(|i| i.key_pressed(Key::F1)) {
            self.show_shortcuts_modal = !self.show_shortcuts_modal;
        }
        if ctx.input(|i| i.key_pressed(Key::F5)) {
            self.save_workset_slot(1);
        }
        if ctx.input(|i| i.key_pressed(Key::F9)) {
            self.load_workset_slot(1);
        }

        // Single-key shortcuts active only when no text input has focus
        if !ctx.wants_keyboard_input() {
            if ctx.input(|i| i.key_pressed(Key::H)) {
                self.screen = match self.screen {
                    AppScreen::LauncherHub => AppScreen::Workstation,
                    AppScreen::Workstation => AppScreen::LauncherHub,
                };
            }
            if ctx.input(|i| i.key_pressed(Key::Z)) {
                self.zen_mode = !self.zen_mode;
                self.screen = AppScreen::Workstation;
            }
            if ctx.input(|i| i.key_pressed(Key::O)) {
                self.auto_orbit = !self.auto_orbit;
            }
            if ctx.input(|i| i.key_pressed(Key::R)) {
                self.camera = Camera3D::default();
            }
            if ctx.input(|i| i.key_pressed(Key::B)) {
                self.run_scaling_benchmark();
            }
            if ctx.input(|i| i.key_pressed(Key::Num2)) {
                self.show_3d_mode = false;
            }
            if ctx.input(|i| i.key_pressed(Key::Num3)) {
                self.show_3d_mode = true;
            }
            if ctx.input(|i| i.key_pressed(Key::Space)) {
                self.dispatch_compute();
            }
        }

        // Persistent Desktop Menu Bar + Optional Action Toolbar
        egui::TopBottomPanel::top("top_menu_and_toolbar").show(ctx, |ui| {
            self.draw_persistent_menu_bar(ctx, ui);
            if self.screen == AppScreen::Workstation && !self.zen_mode {
                ui.separator();
                ui.horizontal_wrapped(|ui| {
                    ui.label("Threads:");
                    if ui
                        .add(egui::Slider::new(&mut self.params.threads, 1..=32))
                        .changed()
                    {
                        self.dispatch_compute();
                    }
                    if ui.button("Run Parallel Compute").clicked() {
                        self.dispatch_compute();
                    }
                    ui.separator();
                    ui.selectable_value(&mut self.show_3d_mode, true, "3D Viewport");
                    ui.selectable_value(&mut self.show_3d_mode, false, "2D Plot");
                    if self.show_3d_mode {
                        ui.checkbox(&mut self.auto_orbit, "Calm Orbit");
                        if ui.button("Reset Camera").clicked() {
                            self.camera = Camera3D::default();
                        }
                    }
                    ui.separator();
                    if ui.button("Save Slot (F5)").clicked() {
                        self.save_workset_slot(1);
                    }
                    if ui.button("Load Slot (F9)").clicked() {
                        self.load_workset_slot(1);
                    }
                });
            }
        });

        egui::TopBottomPanel::bottom("bottom_status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let indicator = if self.is_computing { "[RUNNING]" } else { "[IDLE]" };
                ui.label(
                    RichText::new(format!("{indicator} {}", self.status_banner))
                        .monospace()
                        .size(11.5)
                        .color(Color32::from_rgb(148, 163, 184)),
                );
            });
        });

        self.draw_modals(ctx);

        if self.screen == AppScreen::LauncherHub {
            egui::CentralPanel::default().show(ctx, |ui| {
                self.draw_launcher_hub(ui);
            });
            return;
        }

        if !self.zen_mode && self.show_left_nav {
            egui::SidePanel::left("domain_navigator")
                .resizable(true)
                .default_width(275.0)
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for cat in DomainCategory::ALL {
                            ui.add_space(6.0);
                            ui.label(
                                RichText::new(cat.title())
                                    .strong()
                                    .size(12.5)
                                    .color(Color32::from_rgb(250, 204, 21)),
                            );
                            for m in ModuleId::ALL.iter().copied().filter(|m| m.category() == cat) {
                                let selected = self.params.module == m;
                                let badge = if m.is_3d_primary() { "3D" } else { "2D" };
                                let label = format!("[{badge}] {}", m.short_name());
                                if ui.selectable_label(selected, label).clicked() {
                                    self.select_module(m);
                                }
                            }
                        }
                    });
                });
        }

        if !self.zen_mode && self.show_right_inspector {
            egui::SidePanel::right("inspector_panel")
                .resizable(true)
                .default_width(335.0)
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.heading(self.params.module.short_name());
                        ui.label(
                            RichText::new(self.params.module.subtitle())
                                .size(11.5)
                                .color(Color32::from_rgb(148, 163, 184)),
                        );
                        ui.separator();

                        let mut changed = false;
                        ui.horizontal(|ui| {
                            ui.label("Preset Variant:");
                            for p in 0..4 {
                                if ui
                                    .selectable_value(
                                        &mut self.params.preset_index,
                                        p,
                                        format!("#{p}"),
                                    )
                                    .changed()
                                {
                                    changed = true;
                                }
                            }
                        });

                        changed |= ui
                            .add(
                                egui::DragValue::new(&mut self.params.primary_n)
                                    .speed(10.0)
                                    .prefix("Primary N = "),
                            )
                            .changed();
                        changed |= ui
                            .add(
                                egui::DragValue::new(&mut self.params.span)
                                    .speed(5.0)
                                    .prefix("Span / Steps = "),
                            )
                            .changed();
                        changed |= ui
                            .add(
                                egui::Slider::new(&mut self.params.secondary_param, 0.05..=2.20)
                                    .text("Scale / Eps"),
                            )
                            .changed();

                        if matches!(
                            self.params.module,
                            ModuleId::F1InteractiveTheoremProving
                                | ModuleId::S1SymbolicDiffInt
                                | ModuleId::S2GroebnerPolynomials
                                | ModuleId::S3ExpressionSimplification
                        ) {
                            ui.label("Symbolic / Logical Formula:");
                            if ui
                                .text_edit_singleline(&mut self.params.expression_input)
                                .lost_focus()
                            {
                                changed = true;
                            }
                        }

                        if changed {
                            self.dispatch_compute();
                        }

                        ui.separator();
                        ui.horizontal(|ui| {
                            if ui.button("Export SVG").clicked() {
                                self.export_svg_current();
                            }
                            if ui.button("Export 3D .OBJ").clicked() {
                                self.export_obj_current();
                            }
                            if ui.button("Bench (B)").clicked() {
                                self.run_scaling_benchmark();
                            }
                        });

                        ui.add_space(6.0);
                        ui.label(
                            RichText::new("Equivalent Headless CLI Command:")
                                .strong()
                                .size(11.5),
                        );
                        let cmd_str = self.cli_command_string();
                        ui.code(&cmd_str);
                        if ui.button("Copy CLI Command").clicked() {
                            ctx.copy_text(cmd_str);
                        }

                        if let Some(ref rep) = self.latest_report {
                            ui.separator();
                            ui.label(
                                RichText::new(format!(
                                    "Telemetry ({:.2} ms | {} threads)",
                                    rep.elapsed_ms, rep.threads_used
                                ))
                                .strong(),
                            );
                            for card in &rep.metrics {
                                egui::Frame::group(ui.style())
                                    .fill(Color32::from_rgb(15, 23, 42))
                                    .corner_radius(CornerRadius::same(6))
                                    .show(ui, |ui| {
                                        ui.label(
                                            RichText::new(&card.label)
                                                .size(11.0)
                                                .color(Color32::from_rgb(148, 163, 184)),
                                        );
                                        ui.label(
                                            RichText::new(&card.value)
                                                .monospace()
                                                .strong()
                                                .color(Color32::from_rgb(56, 189, 248)),
                                        );
                                        ui.label(RichText::new(&card.detail).size(10.5));
                                    });
                            }
                            ui.add_space(6.0);
                            ui.label(RichText::new("Verification Log:").strong().size(11.5));
                            for line in &rep.log_lines {
                                ui.label(RichText::new(format!("• {line}")).monospace().size(10.5));
                            }
                        }
                    });
                });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(rep) = self.latest_report.clone() {
                ui.label(
                    RichText::new(&rep.headline)
                        .strong()
                        .monospace()
                        .size(13.0)
                        .color(Color32::from_rgb(52, 211, 153)),
                );
                ui.add_space(4.0);
                let has_3d = !rep.scene_3d.nodes.is_empty()
                    || !rep.scene_3d.edges.is_empty()
                    || !rep.scene_3d.faces.is_empty();
                let has_2d = !rep.scene_2d.points.is_empty()
                    || !rep.scene_2d.polylines.is_empty()
                    || !rep.scene_2d.rects.is_empty();

                if self.show_3d_mode && has_3d {
                    self.draw_scene_3d(ctx, ui, &rep.scene_3d);
                } else if has_2d {
                    self.draw_scene_2d(ui, &rep.scene_2d);
                } else if has_3d {
                    self.draw_scene_3d(ctx, ui, &rep.scene_3d);
                }
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Initializing multithreaded Rayon mathematics engine...");
                });
            }
        });
    }
}

pub fn launch_gui(
    initial_module: ModuleId,
    threads: Option<usize>,
    skip_hub: bool,
) -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1460.0, 920.0])
            .with_min_inner_size([980.0, 640.0])
            .with_icon(std::sync::Arc::new(crate::icon::app_egui_icon())),
        ..Default::default()
    };
    eframe::run_native(
        "Principia Studio v1.1.0 — Multithreaded Computational Mathematics Explorer",
        options,
        Box::new(move |cc| {
            Ok(Box::new(PrincipiaApp::new(
                cc,
                initial_module,
                threads,
                skip_hub,
            )))
        }),
    )
}
