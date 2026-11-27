#![forbid(unsafe_code)]

//! Headless Multithreaded CLI (`clap` v4.5) supporting `list`, `run`, `bench`, `digest`, `--skip-hub`, and SVG/OBJ/JSON/CSV Exports.

use crate::digest::{build_cross_domain_digest, digest_to_csv, digest_to_json};
use crate::render3d::{export_report_svg, export_scene3d_obj};
use crate::{execute_module, DomainCategory, ExplorerParams, ModuleId};
use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "principia-studio",
    version = "1.2.0",
    about = "Multithreaded Safe-Rust Computational Mathematics Explorer (Menu-Driven 2D/3D egui Workstation + Headless CLI)"
)]
pub struct RootCli {
    #[command(subcommand)]
    pub command: Option<TopCommand>,

    /// Worker thread count when launching the GUI directly (`default = hardware concurrency`)
    #[arg(long)]
    pub threads: Option<usize>,

    /// Bypass the interactive 7-Domain Launcher Hub and open the 2D/3D canvas directly
    #[arg(long, default_value_t = false)]
    pub skip_hub: bool,
}

#[derive(Subcommand, Debug)]
pub enum TopCommand {
    /// Launch the interactive 2D/3D egui desktop workstation
    Gui {
        /// Initial module code (`n1`..`p3`)
        #[arg(short, long, default_value = "n1")]
        module: String,
        /// Rayon worker threads
        #[arg(short, long)]
        threads: Option<usize>,
        /// Bypass the Startup Domain Launcher Hub and open the active module directly
        #[arg(long, default_value_t = false)]
        skip_hub: bool,
    },
    /// Execute headless CLI commands (`list`, `run`, `bench`)
    Cli {
        #[command(subcommand)]
        action: CliAction,
    },
}

#[derive(Subcommand, Debug)]
pub enum CliAction {
    /// List all 7 mathematical domains and 22 computational modules
    List,
    /// Run a specific mathematical module and optionally export SVG, Wavefront OBJ, or JSON
    Run {
        /// Module code (`n1`..`n4`, `f1`..`f3`, `s1`..`s3`, `d1`..`d3`, `c1`..`c3`, `g1`..`g3`, `p1`..`p3`)
        #[arg(short, long, default_value = "n1")]
        module: String,
        /// Worker threads (`1..128`)
        #[arg(short, long)]
        threads: Option<usize>,
        /// Preset index (`0..3`)
        #[arg(long, default_value_t = 0)]
        preset: usize,
        /// Primary problem scale parameter `N`
        #[arg(short, long)]
        n: Option<u64>,
        /// Secondary span / step count / grid resolution
        #[arg(long)]
        span: Option<u64>,
        /// Continuous parameter (`dt`, filtration `epsilon`, tube radius, etc.)
        #[arg(long)]
        param: Option<f64>,
        /// Algebraic / logical expression string (for `f1`, `s1`, `s2`, `s3`)
        #[arg(long)]
        expr: Option<String>,
        /// Export rendered 2D/3D visualization to standalone SVG file
        #[arg(long)]
        export_svg: Option<PathBuf>,
        /// Export 3D mesh/ribbon geometry to Wavefront `.obj` file
        #[arg(long)]
        export_obj: Option<PathBuf>,
        /// Export full structured `ComputationReport` JSON
        #[arg(long)]
        export_json: Option<PathBuf>,
    },
    /// Benchmark multithreaded scaling from `1` thread up to `max_threads`
    Bench {
        /// Module code to benchmark (`n1`, `n4`, `d1`, `d2`, `g2`, etc.)
        #[arg(short, long, default_value = "n4")]
        module: String,
        /// Maximum thread count to test
        #[arg(long, default_value_t = 8)]
        max_threads: usize,
    },
    /// Generate reproducible cross-domain verification & benchmark digest (JSON / CSV)
    Digest {
        /// Worker threads (`1..128`)
        #[arg(short, long)]
        threads: Option<usize>,
        /// Export full cross-domain JSON digest
        #[arg(long)]
        export_json: Option<PathBuf>,
        /// Export RFC 4180 CSV digest across 22 modules and 12 P1-P3 proof invariants
        #[arg(long)]
        export_csv: Option<PathBuf>,
    },
}

pub fn run_cli_action(action: CliAction) -> Result<(), String> {
    match action {
        CliAction::List => {
            println!("Principia Studio v1.2.0 — 22 Multithreaded Mathematical Modules:");
            for cat in DomainCategory::ALL {
                println!("\n  {}", cat.title());
                for m in ModuleId::ALL.iter().copied().filter(|m| m.category() == cat) {
                    let dim = if m.is_3d_primary() { "3D" } else { "2D" };
                    println!(
                        "    [{:>2}] {:<32} ({dim}) — {}",
                        m.code(),
                        m.short_name(),
                        m.subtitle()
                    );
                }
            }
            Ok(())
        }
        CliAction::Run {
            module,
            threads,
            preset,
            n,
            span,
            param,
            expr,
            export_svg,
            export_obj,
            export_json,
        } => {
            let mod_id = ModuleId::from_code(&module).ok_or_else(|| {
                format!(
                    "Unknown module code '{module}'. Run `cli list` to inspect valid codes (`n1`..`p3`)."
                )
            })?;
            let mut params = ExplorerParams::default_for(mod_id);
            if let Some(t) = threads {
                params.threads = t.clamp(1, 128);
            }
            params.preset_index = preset;
            if let Some(nv) = n {
                params.primary_n = nv;
            }
            if let Some(sv) = span {
                params.span = sv;
            }
            if let Some(pv) = param {
                params.secondary_param = pv;
            }
            if let Some(ev) = expr {
                params.expression_input = ev;
            }

            let report = execute_module(&params);
            println!("================================================================================");
            println!("Module   : {}", report.module);
            println!(
                "Threads  : {} Rayon workers | Elapsed: {:.2} ms",
                report.threads_used, report.elapsed_ms
            );
            println!("Headline : {}", report.headline);
            println!("--------------------------------------------------------------------------------");
            for m in &report.metrics {
                println!("  * {:<22} : {:<28} ({})", m.label, m.value, m.detail);
            }
            for line in &report.log_lines {
                println!("  > {line}");
            }

            if let Some(svg_path) = export_svg {
                let svg = export_report_svg(&report, 1280, 800);
                fs::write(&svg_path, svg)
                    .map_err(|e| format!("Failed writing SVG to {:?}: {e}", svg_path))?;
                println!("  [Saved SVG] {:?}", svg_path);
            }
            if let Some(obj_path) = export_obj {
                let obj = export_scene3d_obj(&report.scene_3d);
                fs::write(&obj_path, obj)
                    .map_err(|e| format!("Failed writing OBJ to {:?}: {e}", obj_path))?;
                println!("  [Saved OBJ] {:?}", obj_path);
            }
            if let Some(json_path) = export_json {
                let json = serde_json::to_string_pretty(&report)
                    .map_err(|e| format!("JSON serialization error: {e}"))?;
                fs::write(&json_path, json)
                    .map_err(|e| format!("Failed writing JSON to {:?}: {e}", json_path))?;
                println!("  [Saved JSON] {:?}", json_path);
            }
            Ok(())
        }
        CliAction::Bench {
            module,
            max_threads,
        } => {
            let mod_id = ModuleId::from_code(&module)
                .ok_or_else(|| format!("Unknown module code '{module}'."))?;
            println!(
                "Benchmarking Multithreaded Scaling for {} (1..={} threads):",
                mod_id, max_threads
            );
            let mut base_ms = 0.0f64;
            let mut t = 1usize;
            while t <= max_threads.max(1) {
                let mut p = ExplorerParams::default_for(mod_id);
                p.threads = t;
                let rep = execute_module(&p);
                if t == 1 {
                    base_ms = rep.elapsed_ms.max(0.01);
                }
                let speedup = base_ms / rep.elapsed_ms.max(0.01);
                println!(
                    "  threads = {:>2} -> {:>8.2} ms  (speedup = {:.2}x) | {}",
                    t, rep.elapsed_ms, speedup, rep.headline
                );
                if t == max_threads {
                    break;
                }
                t = (t * 2).min(max_threads);
            }
            Ok(())
        }
        CliAction::Digest {
            threads,
            export_json,
            export_csv,
        } => {
            let t = threads.unwrap_or_else(|| {
                std::thread::available_parallelism()
                    .map(|n| n.get())
                    .unwrap_or(4)
            });
            let digest = build_cross_domain_digest(t);
            println!("================================================================================");
            println!(
                "Principia Studio v{} Cross-Domain Verification Digest ({} Domains / {} Modules)",
                digest.version, digest.domains_count, digest.modules_count
            );
            println!(
                "Threads: {} | Total Elapsed: {:.2} ms | P1-P3 Invariants: {} ({}/{}) | SHA-256: {}",
                digest.threads_requested,
                digest.total_elapsed_ms,
                if digest.all_p1_p3_invariants_passed {
                    "PASS"
                } else {
                    "FAIL"
                },
                digest.p1_p3_invariants.iter().filter(|i| i.passed).count(),
                digest.p1_p3_invariants.len(),
                digest.digest_sha256
            );
            println!("--------------------------------------------------------------------------------");
            for m in &digest.modules {
                println!(
                    "  [{:>2}] {:<32} {:>7.2} ms | {}",
                    m.code, m.short_name, m.elapsed_ms, m.headline
                );
            }
            if let Some(json_path) = export_json {
                let json = digest_to_json(&digest)?;
                fs::write(&json_path, json)
                    .map_err(|e| format!("Failed writing digest JSON to {:?}: {e}", json_path))?;
                println!("  [Saved Digest JSON] {:?}", json_path);
            }
            if let Some(csv_path) = export_csv {
                let csv = digest_to_csv(&digest);
                fs::write(&csv_path, csv)
                    .map_err(|e| format!("Failed writing digest CSV to {:?}: {e}", csv_path))?;
                println!("  [Saved Digest CSV] {:?}", csv_path);
            }
            Ok(())
        }
    }
}
