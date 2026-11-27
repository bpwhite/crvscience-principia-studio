#![forbid(unsafe_code)]

//! Cross-Domain Reproducible Verification & Benchmark Digest (`N1`..`P3`).
//! Executes all 7 mathematical domains / 22 computational modules plus Domain 7
//! (`P1`..`P3`) geometric proof invariant checks and serializes deterministic
//! JSON and RFC 4180 CSV digests.

use crate::geometric_proofs::{
    archimedes_exhaustion_pi, cavalieri_sphere_cylinder_cone_verify, dandelin_ellipse_verify,
    euclid_i47_altitude_verify, geometric_series_tiling_verify, nicomachus_cubes_verify,
    pappus_torus_verify, platonic_solids_euler_descartes, poincare_hyperbolic_triangle_verify,
    pythagorean_dissection_verify, thales_ptolemy_verify, viviani_simplex_verify,
};
use crate::{execute_module, DomainCategory, ExplorerParams, ModuleId};
use serde::{Deserialize, Serialize};
use sha2::{Digest as ShaDigest, Sha256};
use std::f64::consts::PI;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleDigestRow {
    pub code: String,
    pub short_name: String,
    pub domain: String,
    pub prefer_3d: bool,
    pub threads_used: usize,
    pub elapsed_ms: f64,
    pub metric_count: usize,
    pub scene_2d_primitives: usize,
    pub scene_3d_primitives: usize,
    pub headline: String,
    pub invariant_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeometricProofInvariantSummary {
    pub module_code: String,
    pub preset_index: usize,
    pub theorem_name: String,
    pub primary_value: String,
    pub residual: f64,
    pub passed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossDomainDigest {
    pub schema_version: String,
    pub app: String,
    pub version: String,
    pub threads_requested: usize,
    pub domains_count: usize,
    pub modules_count: usize,
    pub all_modules_non_empty: bool,
    pub all_p1_p3_invariants_passed: bool,
    pub total_elapsed_ms: f64,
    pub digest_sha256: String,
    pub modules: Vec<ModuleDigestRow>,
    pub p1_p3_invariants: Vec<GeometricProofInvariantSummary>,
}

fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let bytes = hasher.finalize();
    let mut hex = String::with_capacity(64);
    for b in bytes {
        hex.push_str(&format!("{b:02x}"));
    }
    hex
}

fn csv_escape(field: &str) -> String {
    if field.contains(',')
        || field.contains('"')
        || field.contains('\n')
        || field.contains('\r')
        || field.starts_with(' ')
        || field.ends_with(' ')
    {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}

pub fn verify_domain7_p1_p3_invariants() -> Vec<GeometricProofInvariantSummary> {
    let mut out = Vec::with_capacity(12);

    // P1 Preset 0: Pythagorean Dissection
    let (c, outer, dissected, pyth_res) = pythagorean_dissection_verify(3.0, 4.0);
    out.push(GeometricProofInvariantSummary {
        module_code: "p1".to_string(),
        preset_index: 0,
        theorem_name: "Pythagorean Dissection (3-4-5)".to_string(),
        primary_value: format!("c={c:.6}, area={outer:.6}=={dissected:.6}"),
        residual: pyth_res,
        passed: pyth_res < 1e-12 && (c - 5.0).abs() < 1e-12,
    });

    // P1 Preset 1: Euclid I.47 Windmill & Altitude Shear
    let (ce, p, q, euclid_res) = euclid_i47_altitude_verify(5.0, 12.0);
    out.push(GeometricProofInvariantSummary {
        module_code: "p1".to_string(),
        preset_index: 1,
        theorem_name: "Euclid I.47 Altitude Shear (5-12-13)".to_string(),
        primary_value: format!("c={ce:.6}, p={p:.6}, q={q:.6}"),
        residual: euclid_res,
        passed: euclid_res < 1e-12 && (ce - 13.0).abs() < 1e-12,
    });

    // P1 Preset 2: Geometric Series Unit-Square Tiling
    let (sum, exact, rem) = geometric_series_tiling_verify(12);
    let series_res = (sum - exact).abs() + (sum + rem - 1.0).abs();
    out.push(GeometricProofInvariantSummary {
        module_code: "p1".to_string(),
        preset_index: 2,
        theorem_name: "Geometric Series Tiling (depth=12)".to_string(),
        primary_value: format!("sum={sum:.9}, rem={rem:.9}"),
        residual: series_res,
        passed: series_res < 1e-12,
    });

    // P1 Preset 3: Nicomachus Sum of Cubes
    let (cube_sum, tri_sq, nic_ok) = nicomachus_cubes_verify(12);
    let nic_res = cube_sum.abs_diff(tri_sq) as f64;
    out.push(GeometricProofInvariantSummary {
        module_code: "p1".to_string(),
        preset_index: 3,
        theorem_name: "Nicomachus Sum of Cubes (n=12)".to_string(),
        primary_value: format!("sum_k3={cube_sum}, T12^2={tri_sq}"),
        residual: nic_res,
        passed: nic_ok && nic_res == 0.0,
    });

    // P2 Preset 0: Archimedes Exhaustion for Pi
    let (m_sides, lower_pi, upper_pi, gap_pi) = archimedes_exhaustion_pi(6);
    out.push(GeometricProofInvariantSummary {
        module_code: "p2".to_string(),
        preset_index: 0,
        theorem_name: "Archimedes Pi Exhaustion (384-gon)".to_string(),
        primary_value: format!("m={m_sides}, [{lower_pi:.6}, {upper_pi:.6}]"),
        residual: gap_pi,
        passed: m_sides == 384 && lower_pi < PI && PI < upper_pi && gap_pi < 2e-4,
    });

    // P2 Preset 1: Thales & Ptolemy Cyclic Quadrilaterals
    let (thales_dot, ptolemy_err, inscribed_deg) = thales_ptolemy_verify(48.5);
    let cyclic_res = thales_dot + ptolemy_err + (inscribed_deg - 90.0).abs();
    out.push(GeometricProofInvariantSummary {
        module_code: "p2".to_string(),
        preset_index: 1,
        theorem_name: "Thales & Ptolemy Cyclic Identities".to_string(),
        primary_value: format!("angle={inscribed_deg:.4} deg"),
        residual: cyclic_res,
        passed: thales_dot < 1e-12 && ptolemy_err < 1e-12 && (inscribed_deg - 90.0).abs() < 1e-9,
    });

    // P2 Preset 2: 3D Dandelin Spheres
    let (two_a, ecc, focal_err, tan_err) = dandelin_ellipse_verify(0.45, 0.25, 128);
    let dand_res = focal_err + tan_err;
    out.push(GeometricProofInvariantSummary {
        module_code: "p2".to_string(),
        preset_index: 2,
        theorem_name: "3D Dandelin Spheres Ellipse".to_string(),
        primary_value: format!("2a={two_a:.6}, e={ecc:.6}"),
        residual: dand_res,
        passed: two_a > 0.0 && ecc > 0.0 && ecc < 1.0 && focal_err < 1e-9 && tan_err < 1e-9,
    });

    // P2 Preset 3: Poincare Disk Hyperbolic Triangle & Gauss-Bonnet Defect
    let (ang_sum, defect_area, alpha_deg, k_hyp) = poincare_hyperbolic_triangle_verify(0.72);
    let hyp_res = (ang_sum + defect_area - PI).abs();
    out.push(GeometricProofInvariantSummary {
        module_code: "p2".to_string(),
        preset_index: 3,
        theorem_name: "Poincare Hyperbolic Gauss-Bonnet".to_string(),
        primary_value: format!("alpha={alpha_deg:.3} deg, area={defect_area:.6}, K={k_hyp:.1}"),
        residual: hyp_res,
        passed: k_hyp == -1.0 && alpha_deg < 60.0 && defect_area > 0.0 && hyp_res < 1e-12,
    });

    // P3 Preset 0: Cavalieri Sphere-Cylinder-Cone 1:2:3
    let (slice_err, v_sph, v_cyl, ratio_err) = cavalieri_sphere_cylinder_cone_verify(1.5, 120);
    out.push(GeometricProofInvariantSummary {
        module_code: "p3".to_string(),
        preset_index: 0,
        theorem_name: "Archimedes 1:2:3 Cavalieri Slices".to_string(),
        primary_value: format!("Vsph={v_sph:.5}, Vcyl={v_cyl:.5}"),
        residual: slice_err + ratio_err,
        passed: slice_err < 1e-12 && ratio_err < 1e-4,
    });

    // P3 Preset 1: Platonic Solids Euler & Descartes Defect
    let platonic = platonic_solids_euler_descartes();
    let max_defect_err = platonic
        .iter()
        .map(|(_, _, _, _, _, def)| (def - 4.0 * PI).abs())
        .fold(0.0_f64, f64::max);
    let all_chi_2 = platonic.len() == 5 && platonic.iter().all(|(_, _, _, _, chi, _)| *chi == 2);
    out.push(GeometricProofInvariantSummary {
        module_code: "p3".to_string(),
        preset_index: 1,
        theorem_name: "Platonic Euler chi=2 & Descartes 4pi".to_string(),
        primary_value: format!("5 solids, chi=2, max_err={max_defect_err:.3e}"),
        residual: max_defect_err,
        passed: all_chi_2 && max_defect_err < 1e-10,
    });

    // P3 Preset 2: Viviani 2D & 3D Simplex Altitude Conservation
    let (err_2d, err_3d, h3) = viviani_simplex_verify(0.3, 0.4, 0.3);
    let viv_res = err_2d + err_3d;
    out.push(GeometricProofInvariantSummary {
        module_code: "p3".to_string(),
        preset_index: 2,
        theorem_name: "Viviani 2D/3D Regular Simplex".to_string(),
        primary_value: format!("h3={h3:.6}"),
        residual: viv_res,
        passed: err_2d < 1e-12 && err_3d < 1e-12,
    });

    // P3 Preset 3: Pappus Centroid Torus Theorem
    let (v_exact, v_mesh, a_exact, rel_err) = pappus_torus_verify(2.0, 0.5, 96, 48);
    out.push(GeometricProofInvariantSummary {
        module_code: "p3".to_string(),
        preset_index: 3,
        theorem_name: "Pappus Centroid Torus Volume".to_string(),
        primary_value: format!("V={v_exact:.6} (mesh={v_mesh:.6}, A={a_exact:.4})"),
        residual: rel_err,
        passed: rel_err < 1.5e-2,
    });

    out
}

pub fn build_cross_domain_digest(threads: usize) -> CrossDomainDigest {
    let clamped_threads = threads.clamp(1, 128);
    let mut rows = Vec::with_capacity(ModuleId::ALL.len());
    let mut total_elapsed_ms = 0.0;
    let mut all_non_empty = true;
    let mut digest_material = String::new();

    for &m in &ModuleId::ALL {
        let mut params = ExplorerParams::default_for(m);
        params.threads = clamped_threads;
        let rep = execute_module(&params);

        let scene_2d_primitives =
            rep.scene_2d.points.len() + rep.scene_2d.polylines.len() + rep.scene_2d.rects.len();
        let scene_3d_primitives =
            rep.scene_3d.nodes.len() + rep.scene_3d.edges.len() + rep.scene_3d.faces.len();

        if rep.headline.trim().is_empty() || rep.metrics.is_empty() {
            all_non_empty = false;
        }
        total_elapsed_ms += rep.elapsed_ms;

        let row_payload = format!(
            "{}|{}|{}|{}|{}|{}|{}",
            m.code(),
            m.short_name(),
            m.category().title(),
            rep.prefer_3d,
            rep.metrics.len(),
            scene_2d_primitives,
            scene_3d_primitives,
        );
        let invariant_sha256 = sha256_hex(row_payload.as_bytes());
        digest_material.push_str(&invariant_sha256);
        digest_material.push('\n');

        rows.push(ModuleDigestRow {
            code: m.code().to_string(),
            short_name: m.short_name().to_string(),
            domain: m.category().title().to_string(),
            prefer_3d: rep.prefer_3d,
            threads_used: rep.threads_used,
            elapsed_ms: rep.elapsed_ms,
            metric_count: rep.metrics.len(),
            scene_2d_primitives,
            scene_3d_primitives,
            headline: rep.headline,
            invariant_sha256,
        });
    }

    let p1_p3_invariants = verify_domain7_p1_p3_invariants();
    let all_p1_p3_invariants_passed = p1_p3_invariants.iter().all(|inv| inv.passed);
    for inv in &p1_p3_invariants {
        digest_material.push_str(&format!(
            "{}:{}:{}:{}\n",
            inv.module_code, inv.preset_index, inv.theorem_name, inv.passed
        ));
    }

    let digest_sha256 = sha256_hex(digest_material.as_bytes());

    CrossDomainDigest {
        schema_version: "1.0".to_string(),
        app: "principia-studio".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        threads_requested: clamped_threads,
        domains_count: DomainCategory::ALL.len(),
        modules_count: rows.len(),
        all_modules_non_empty: all_non_empty,
        all_p1_p3_invariants_passed,
        total_elapsed_ms,
        digest_sha256,
        modules: rows,
        p1_p3_invariants,
    }
}

pub fn digest_to_json(digest: &CrossDomainDigest) -> Result<String, String> {
    serde_json::to_string_pretty(digest).map_err(|e| format!("Digest JSON error: {e}"))
}

pub fn digest_to_csv(digest: &CrossDomainDigest) -> String {
    let mut lines = Vec::with_capacity(1 + digest.modules.len() + digest.p1_p3_invariants.len());
    lines.push(
        "record_type,code,domain_or_preset,name,prefer_3d,threads_used,elapsed_ms,metric_count,scene_2d_primitives,scene_3d_primitives,residual,passed,invariant_sha256,summary"
            .to_string(),
    );
    for m in &digest.modules {
        lines.push(format!(
            "module,{},{},{},{},{},{:.3},{},{},{},0.0,true,{},{}",
            csv_escape(&m.code),
            csv_escape(&m.domain),
            csv_escape(&m.short_name),
            m.prefer_3d,
            m.threads_used,
            m.elapsed_ms,
            m.metric_count,
            m.scene_2d_primitives,
            m.scene_3d_primitives,
            csv_escape(&m.invariant_sha256),
            csv_escape(&m.headline),
        ));
    }
    for inv in &digest.p1_p3_invariants {
        lines.push(format!(
            "p1_p3_invariant,{},preset_{},{},false,{},0.0,1,0,0,{:.3e},{},{},{}",
            csv_escape(&inv.module_code),
            inv.preset_index,
            csv_escape(&inv.theorem_name),
            digest.threads_requested,
            inv.residual,
            inv.passed,
            csv_escape(&digest.digest_sha256),
            csv_escape(&inv.primary_value),
        ));
    }
    lines.join("\n") + "\n"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digest_covers_all_7_domains_and_22_modules() {
        let d = build_cross_domain_digest(2);
        assert_eq!(d.domains_count, 7);
        assert_eq!(d.modules_count, 22);
        assert_eq!(d.modules.len(), 22);
        assert!(d.all_modules_non_empty);
        assert_eq!(d.digest_sha256.len(), 64);
    }

    #[test]
    fn digest_domain7_p1_p3_invariants_all_pass() {
        let invs = verify_domain7_p1_p3_invariants();
        assert_eq!(invs.len(), 12);
        for inv in &invs {
            assert!(
                inv.passed,
                "Invariant failed for {} preset {}: residual={}",
                inv.theorem_name, inv.preset_index, inv.residual
            );
        }
    }

    #[test]
    fn digest_json_and_csv_deterministic_and_well_formed() {
        let d1 = build_cross_domain_digest(2);
        let d2 = build_cross_domain_digest(2);
        assert_eq!(d1.digest_sha256, d2.digest_sha256);
        assert_eq!(d1.modules.len(), d2.modules.len());

        let json = digest_to_json(&d1).expect("JSON serialization");
        let parsed: CrossDomainDigest = serde_json::from_str(&json).expect("JSON parse");
        assert_eq!(parsed.modules_count, 22);
        assert!(parsed.all_p1_p3_invariants_passed);

        let csv = digest_to_csv(&d1);
        let lines: Vec<&str> = csv.lines().collect();
        // 1 header + 22 modules + 12 P1-P3 invariants = 35 lines
        assert_eq!(lines.len(), 35);
        assert!(lines[0].starts_with("record_type,code,domain_or_preset"));
    }
}
