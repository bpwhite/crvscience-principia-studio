#![forbid(unsafe_code)]

//! Domain 7: Geometric Proofs & Visualizations (`P1`..`P3`)
//! - `P1`: Visual Area & Dissection Proofs — Pythagorean Dissection (Bhāskara / Zhoubi Suanjing),
//!   Euclid's Elements I.47 Windmill & Altitude Shear, Geometric Series Unit-Square Tiling, and Nicomachus Sum of Cubes
//! - `P2`: Circle, Conic & Non-Euclidean Proofs — Archimedes' Method of Exhaustion for `pi`,
//!   Thales / Inscribed Angle / Ptolemy Cyclic Theorems, 3D Dandelin Spheres (`|PF_1| + |PF_2| = 2a`),
//!   and Poincaré Disk Hyperbolic Triangle & Gauss-Bonnet Defect
//! - `P3`: 3D Cavalieri & Polyhedral Proofs — Archimedes' `1 : 2 : 3` Sphere–Cylinder–Cone Slice Identity,
//!   Platonic Solids Euler (`V - E + F = 2`) & Descartes Total Angular Defect (`4*pi`),
//!   Viviani's 2D/3D Regular Simplex Altitude Conservation, and Pappus's Centroid Torus Theorem

use crate::render3d::cross_normal;
use crate::{
    ComputationReport, Edge3D, ExplorerParams, Face3D, MetricCard, ModuleId, Node3D, Point2D,
    Polyline2D, Rect2D, RgbaColor, Scene2D, Scene3D,
};
use rayon::prelude::*;
use std::f64::consts::PI;

fn sub3(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn add3(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn scale3(a: [f64; 3], s: f64) -> [f64; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

fn dot3(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross3(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn norm3(a: [f64; 3]) -> f64 {
    dot3(a, a).sqrt()
}

fn normalize3(a: [f64; 3]) -> [f64; 3] {
    let l = norm3(a).max(1e-12);
    scale3(a, 1.0 / l)
}

/// Pads horizontal `Scene2D` bounds using subtle corner ticks so squares/circles keep true ~1.65:1 viewport proportions.
fn pad_aspect_2d(scene: &mut Scene2D, target_aspect: f64) {
    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    let mut push = |x: f64, y: f64| {
        if x.is_finite() && y.is_finite() {
            min_x = min_x.min(x);
            max_x = max_x.max(x);
            min_y = min_y.min(y);
            max_y = max_y.max(y);
        }
    };
    for p in &scene.points {
        push(p.x, p.y);
    }
    for l in &scene.polylines {
        for &(x, y) in &l.points {
            push(x, y);
        }
    }
    for r in &scene.rects {
        push(r.x0, r.y0);
        push(r.x1, r.y1);
    }
    if !min_x.is_finite() {
        return;
    }
    let dx = (max_x - min_x).max(1e-6);
    let dy = (max_y - min_y).max(1e-6);
    let mid_x = 0.5 * (min_x + max_x);
    let mid_y = 0.5 * (min_y + max_y);
    if dx / dy < target_aspect {
        let half_w = 0.5 * dy * target_aspect;
        let pad_col = RgbaColor(51, 65, 85, 90);
        scene.points.push(Point2D {
            x: mid_x - half_w,
            y: mid_y,
            color: pad_col,
            radius: 1.0,
            label: None,
        });
        scene.points.push(Point2D {
            x: mid_x + half_w,
            y: mid_y,
            color: pad_col,
            radius: 1.0,
            label: None,
        });
    } else {
        let half_h = 0.5 * dx / target_aspect;
        let pad_col = RgbaColor(51, 65, 85, 90);
        scene.points.push(Point2D {
            x: mid_x,
            y: mid_y - half_h,
            color: pad_col,
            radius: 1.0,
            label: None,
        });
        scene.points.push(Point2D {
            x: mid_x,
            y: mid_y + half_h,
            color: pad_col,
            radius: 1.0,
            label: None,
        });
    }
}

fn push_box_3d(
    scene: &mut Scene3D,
    center: [f64; 3],
    size: [f64; 3],
    color: RgbaColor,
    edge_color: RgbaColor,
) {
    let hx = 0.5 * size[0];
    let hy = 0.5 * size[1];
    let hz = 0.5 * size[2];
    let cx = center[0];
    let cy = center[1];
    let cz = center[2];
    let v = [
        [cx - hx, cy - hy, cz - hz],
        [cx + hx, cy - hy, cz - hz],
        [cx + hx, cy - hy, cz + hz],
        [cx - hx, cy - hy, cz + hz],
        [cx - hx, cy + hy, cz - hz],
        [cx + hx, cy + hy, cz - hz],
        [cx + hx, cy + hy, cz + hz],
        [cx - hx, cy + hy, cz + hz],
    ];
    let quads = [
        [0, 3, 2, 1],
        [4, 5, 6, 7],
        [0, 1, 5, 4],
        [1, 2, 6, 5],
        [2, 3, 7, 6],
        [3, 0, 4, 7],
    ];
    for q in quads {
        let p0 = v[q[0]];
        let p1 = v[q[1]];
        let p2 = v[q[2]];
        let p3 = v[q[3]];
        scene.faces.push(Face3D {
            verts: [p0, p1, p2],
            normal: cross_normal(p0, p1, p2),
            color,
        });
        scene.faces.push(Face3D {
            verts: [p0, p2, p3],
            normal: cross_normal(p0, p2, p3),
            color,
        });
    }
    for (i, j) in [
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
    ] {
        scene.edges.push(Edge3D {
            a: v[i],
            b: v[j],
            color: edge_color,
            width: 1.2,
        });
    }
}

fn push_extruded_tri_3d(
    scene: &mut Scene3D,
    tri_xy: [(f64, f64); 3],
    y0: f64,
    y1: f64,
    color: RgbaColor,
    edge_color: RgbaColor,
) {
    let b = [
        [tri_xy[0].0, y0, tri_xy[0].1],
        [tri_xy[1].0, y0, tri_xy[1].1],
        [tri_xy[2].0, y0, tri_xy[2].1],
    ];
    let t = [
        [tri_xy[0].0, y1, tri_xy[0].1],
        [tri_xy[1].0, y1, tri_xy[1].1],
        [tri_xy[2].0, y1, tri_xy[2].1],
    ];
    scene.faces.push(Face3D {
        verts: [b[0], b[2], b[1]],
        normal: cross_normal(b[0], b[2], b[1]),
        color,
    });
    scene.faces.push(Face3D {
        verts: [t[0], t[1], t[2]],
        normal: cross_normal(t[0], t[1], t[2]),
        color,
    });
    for i in 0..3 {
        let j = (i + 1) % 3;
        scene.faces.push(Face3D {
            verts: [b[i], b[j], t[j]],
            normal: cross_normal(b[i], b[j], t[j]),
            color,
        });
        scene.faces.push(Face3D {
            verts: [b[i], t[j], t[i]],
            normal: cross_normal(b[i], t[j], t[i]),
            color,
        });
        scene.edges.push(Edge3D {
            a: t[i],
            b: t[j],
            color: edge_color,
            width: 1.4,
        });
        scene.edges.push(Edge3D {
            a: b[i],
            b: b[j],
            color: edge_color,
            width: 1.0,
        });
        scene.edges.push(Edge3D {
            a: b[i],
            b: t[i],
            color: edge_color,
            width: 1.0,
        });
    }
}

fn push_sphere_3d(
    scene: &mut Scene3D,
    center: [f64; 3],
    radius: f64,
    stacks: usize,
    slices: usize,
    color: RgbaColor,
) {
    let mut ring_pts = Vec::with_capacity(stacks + 1);
    for st in 0..=stacks {
        let phi = PI * (st as f64) / (stacks as f64) - 0.5 * PI;
        let (sp, cp) = phi.sin_cos();
        let mut row = Vec::with_capacity(slices + 1);
        for sl in 0..=slices {
            let th = 2.0 * PI * (sl as f64) / (slices as f64);
            let (sth, cth) = th.sin_cos();
            row.push([
                center[0] + radius * cp * cth,
                center[1] + radius * cp * sth,
                center[2] + radius * sp,
            ]);
        }
        ring_pts.push(row);
    }
    for st in 0..stacks {
        for sl in 0..slices {
            let p00 = ring_pts[st][sl];
            let p10 = ring_pts[st + 1][sl];
            let p11 = ring_pts[st + 1][sl + 1];
            let p01 = ring_pts[st][sl + 1];
            if st > 0 {
                scene.faces.push(Face3D {
                    verts: [p00, p11, p01],
                    normal: cross_normal(p00, p11, p01),
                    color,
                });
            }
            if st + 1 < stacks {
                scene.faces.push(Face3D {
                    verts: [p00, p10, p11],
                    normal: cross_normal(p00, p10, p11),
                    color,
                });
            }
        }
    }
}

// ==================== P1 KERNELS: AREA & DISSECTION PROOFS ====================

/// Bhāskara / Zhoubi Suanjing Pythagorean dissection: `(c, c^2, 4*(ab/2) + (b-a)^2, abs_residual)`.
pub fn pythagorean_dissection_verify(a: f64, b: f64) -> (f64, f64, f64, f64) {
    let a = a.abs().max(1e-6);
    let b = b.abs().max(1e-6);
    let c = (a * a + b * b).sqrt();
    let outer_sq = c * c;
    let dissected = 4.0 * (0.5 * a * b) + (b - a).powi(2);
    let ab_sq = a * a + b * b;
    let abs_residual = (outer_sq - dissected).abs().max((outer_sq - ab_sq).abs());
    (c, outer_sq, dissected, abs_residual)
}

/// Euclid Elements I.47 altitude projection identity: `(c, p, q, max_err)` with `c*p = a^2`, `c*q = b^2`.
pub fn euclid_i47_altitude_verify(a: f64, b: f64) -> (f64, f64, f64, f64) {
    let a = a.abs().max(1e-6);
    let b = b.abs().max(1e-6);
    let c = (a * a + b * b).sqrt();
    let p = (a * a) / c;
    let q = (b * b) / c;
    let max_err = (c * p - a * a)
        .abs()
        .max((c * q - b * b).abs())
        .max((p + q - c).abs());
    (c, p, q, max_err)
}

/// Geometric series unit-square tiling: `(partial_sum, exact_formula, remaining_area)`.
pub fn geometric_series_tiling_verify(depth: usize) -> (f64, f64, f64) {
    let d = depth.clamp(1, 50);
    let mut sum = 0.0_f64;
    let mut term = 0.5_f64;
    for _ in 0..d {
        sum += term;
        term *= 0.5;
    }
    let rem = 2.0_f64.powi(-(d as i32));
    let exact = 1.0 - rem;
    (sum, exact, rem)
}

/// Nicomachus's theorem: `sum_{k=1..=n} k^3 == (sum_{k=1..=n} k)^2`.
pub fn nicomachus_cubes_verify(n: u64) -> (u128, u128, bool) {
    let n_c = n.clamp(1, 10_000) as u128;
    let cube_sum: u128 = (1..=n_c).map(|k| k * k * k).sum();
    let t_n = n_c * (n_c + 1) / 2;
    let tri_sq = t_n * t_n;
    (cube_sum, tri_sq, cube_sum == tri_sq)
}

// ==================== P2 KERNELS: CIRCLE, CONIC & NON-EUCLIDEAN PROOFS ====================

/// Archimedes' method of exhaustion bounds for `pi` using `m = 6 * 2^doublings` regular polygons.
pub fn archimedes_exhaustion_pi(doublings: u32) -> (u64, f64, f64, f64) {
    let k = doublings.min(16);
    let m = 6u64 << k;
    let mf = m as f64;
    let ang = PI / mf;
    let lower = mf * ang.sin();
    let upper = mf * ang.tan();
    (m, lower, upper, upper - lower)
}

/// Thales semicircle right-angle and Ptolemy cyclic quadrilateral identities on the unit circle.
pub fn thales_ptolemy_verify(theta_deg: f64) -> (f64, f64, f64) {
    let a = (-1.0_f64, 0.0_f64);
    let b = (1.0_f64, 0.0_f64);
    let th = theta_deg.to_radians();
    let p = (th.cos(), th.sin());
    let q_ang = -0.7_f64;
    let q = (q_ang.cos(), q_ang.sin());

    let pa = (a.0 - p.0, a.1 - p.1);
    let pb = (b.0 - p.0, b.1 - p.1);
    let thales_dot_abs = (pa.0 * pb.0 + pa.1 * pb.1).abs();
    let norm_pa = (pa.0 * pa.0 + pa.1 * pa.1).sqrt();
    let norm_pb = (pb.0 * pb.0 + pb.1 * pb.1).sqrt();
    let inscribed_deg = if norm_pa > 1e-11 && norm_pb > 1e-11 {
        let cos_val = ((pa.0 * pb.0 + pa.1 * pb.1) / (norm_pa * norm_pb)).clamp(-1.0, 1.0);
        cos_val.acos().to_degrees()
    } else {
        90.0
    };

    let mut cyc = [a, b, p, q];
    cyc.sort_by(|u, v| {
        u.1.atan2(u.0)
            .partial_cmp(&v.1.atan2(v.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let dist = |u: (f64, f64), v: (f64, f64)| ((u.0 - v.0).powi(2) + (u.1 - v.1).powi(2)).sqrt();
    let diag_prod = dist(cyc[0], cyc[2]) * dist(cyc[1], cyc[3]);
    let opp_sum = dist(cyc[0], cyc[1]) * dist(cyc[2], cyc[3])
        + dist(cyc[1], cyc[2]) * dist(cyc[3], cyc[0]);
    let ptolemy_err = (diag_prod - opp_sum).abs();

    (thales_dot_abs, ptolemy_err, inscribed_deg)
}

#[derive(Debug, Clone)]
struct DandelinSystem {
    alpha: f64,
    cz1: f64,
    r1: f64,
    f1: [f64; 3],
    cz2: f64,
    r2: f64,
    f2: [f64; 3],
    two_a: f64,
    eccentricity: f64,
    max_focal_err: f64,
    tangency_err: f64,
    ellipse_pts: Vec<[f64; 3]>,
}

fn solve_dandelin_system(
    cone_half_angle_rad: f64,
    plane_tilt_rad: f64,
    samples: usize,
) -> DandelinSystem {
    let alpha = cone_half_angle_rad.clamp(0.18, 0.95);
    let max_beta = (0.5 * PI - alpha) * 0.85;
    let beta = plane_tilt_rad.clamp(0.05, max_beta.max(0.08));
    let z0 = 2.0_f64;
    let normal = [-beta.sin(), 0.0, beta.cos()];

    let cz1 = z0 * beta.cos() / (beta.cos() + alpha.sin());
    let r1 = cz1 * alpha.sin();
    let f1 = add3([0.0, 0.0, cz1], scale3(normal, r1));

    let cz2 = z0 * beta.cos() / (beta.cos() - alpha.sin());
    let r2 = cz2 * alpha.sin();
    let f2 = add3([0.0, 0.0, cz2], scale3(normal, -r2));

    let two_a = (cz2 - cz1) * alpha.cos();
    let eccentricity = beta.sin() / alpha.cos();

    let n_s = samples.max(16);
    let k = alpha.tan() * beta.tan();
    let tan_a = alpha.tan();

    let mut ellipse_pts = Vec::with_capacity(n_s);
    for i in 0..n_s {
        let t = 2.0 * PI * (i as f64) / (n_s as f64);
        let z = z0 / (1.0 - k * t.cos());
        let x = z * tan_a * t.cos();
        let y = z * tan_a * t.sin();
        ellipse_pts.push([x, y, z]);
    }

    let max_focal_err = ellipse_pts
        .par_iter()
        .map(|&p| {
            let sum_d = norm3(sub3(p, f1)) + norm3(sub3(p, f2));
            (sum_d - two_a).abs()
        })
        .reduce(|| 0.0, f64::max);

    let d_plane = z0 * beta.cos();
    let sphere1_err = (norm3(sub3(f1, [0.0, 0.0, cz1])) - r1)
        .abs()
        .max((dot3(normal, f1) - d_plane).abs());
    let sphere2_err = (norm3(sub3(f2, [0.0, 0.0, cz2])) - r2)
        .abs()
        .max((dot3(normal, f2) - d_plane).abs());
    let tangency_err = sphere1_err.max(sphere2_err);

    DandelinSystem {
        alpha,
        cz1,
        r1,
        f1,
        cz2,
        r2,
        f2,
        two_a,
        eccentricity,
        max_focal_err,
        tangency_err,
        ellipse_pts,
    }
}

/// Dandelin spheres & conic focal constant `|PF_1| + |PF_2| = 2a` on a 3D cone section.
pub fn dandelin_ellipse_verify(
    cone_half_angle_rad: f64,
    plane_tilt_rad: f64,
    samples: usize,
) -> (f64, f64, f64, f64) {
    let sys = solve_dandelin_system(cone_half_angle_rad, plane_tilt_rad, samples);
    (
        sys.two_a,
        sys.eccentricity,
        sys.max_focal_err,
        sys.tangency_err,
    )
}

fn poincare_orthogonal_circle(a: (f64, f64), b: (f64, f64)) -> Option<(f64, f64, f64)> {
    let det = a.0 * b.1 - a.1 * b.0;
    if det.abs() < 1e-11 {
        return None;
    }
    let rhs_a = 0.5 * (1.0 + a.0 * a.0 + a.1 * a.1);
    let rhs_b = 0.5 * (1.0 + b.0 * b.0 + b.1 * b.1);
    let cx = (rhs_a * b.1 - rhs_b * a.1) / det;
    let cy = (a.0 * rhs_b - b.0 * rhs_a) / det;
    let rg2 = cx * cx + cy * cy - 1.0;
    if rg2 <= 0.0 {
        return None;
    }
    Some((cx, cy, rg2.sqrt()))
}

fn poincare_tangent_at(a: (f64, f64), toward: (f64, f64), cx: f64, cy: f64) -> (f64, f64) {
    let rad = (a.0 - cx, a.1 - cy);
    let mut tan = (-rad.1, rad.0);
    let chord = (toward.0 - a.0, toward.1 - a.1);
    if tan.0 * chord.0 + tan.1 * chord.1 < 0.0 {
        tan = (-tan.0, -tan.1);
    }
    let len = (tan.0 * tan.0 + tan.1 * tan.1).sqrt().max(1e-12);
    (tan.0 / len, tan.1 / len)
}

/// Poincaré unit-disk hyperbolic geodesic triangle & Gauss-Bonnet angular defect (`Area = pi - (alpha + beta + gamma)`).
pub fn poincare_hyperbolic_triangle_verify(r: f64) -> (f64, f64, f64, f64) {
    let r = r.clamp(0.15, 0.92);
    let verts = [
        (r, 0.0),
        (r * (2.0 * PI / 3.0).cos(), r * (2.0 * PI / 3.0).sin()),
        (r * (4.0 * PI / 3.0).cos(), r * (4.0 * PI / 3.0).sin()),
    ];

    let mut angle_sum = 0.0_f64;
    let mut alpha_deg = 60.0_f64;
    for i in 0..3 {
        let cur = verts[i];
        let nxt = verts[(i + 1) % 3];
        let prv = verts[(i + 2) % 3];
        if let (Some((c1x, c1y, _)), Some((c2x, c2y, _))) = (
            poincare_orthogonal_circle(cur, nxt),
            poincare_orthogonal_circle(cur, prv),
        ) {
            let t1 = poincare_tangent_at(cur, nxt, c1x, c1y);
            let t2 = poincare_tangent_at(cur, prv, c2x, c2y);
            let ang = (t1.0 * t2.0 + t1.1 * t2.1).clamp(-1.0, 1.0).acos();
            angle_sum += ang;
            if i == 0 {
                alpha_deg = ang.to_degrees();
            }
        }
    }
    let defect_area = (PI - angle_sum).max(0.0);
    (angle_sum, defect_area, alpha_deg, -1.0)
}

// ==================== P3 KERNELS: CAVALIERI, PLATONIC, VIVIANI, PAPPUS ====================

/// Archimedes `1 : 2 : 3` Cavalieri principle across horizontal slices `z in [-R, R]`.
pub fn cavalieri_sphere_cylinder_cone_verify(radius: f64, samples: usize) -> (f64, f64, f64, f64) {
    let r = radius.abs().max(0.1);
    let intervals = (samples.max(20) / 2) * 2;
    let h = 2.0 * r / (intervals as f64);

    let slice_res: Vec<(f64, f64, f64, f64)> = (0..=intervals)
        .into_par_iter()
        .map(|i| {
            let z = -r + (i as f64) * h;
            let a_sphere = (PI * (r * r - z * z)).max(0.0);
            let a_cone = PI * z * z;
            let a_cyl = PI * r * r;
            let err = (a_sphere + a_cone - a_cyl).abs();
            let weight = if i == 0 || i == intervals {
                1.0
            } else if i % 2 == 1 {
                4.0
            } else {
                2.0
            };
            (err, weight * a_cone, weight * a_sphere, weight * a_cyl)
        })
        .collect();

    let mut max_slice_err = 0.0_f64;
    let mut sum_cone = 0.0_f64;
    let mut sum_sphere = 0.0_f64;
    let mut sum_cyl = 0.0_f64;
    for (err, wc, ws, wy) in slice_res {
        max_slice_err = max_slice_err.max(err);
        sum_cone += wc;
        sum_sphere += ws;
        sum_cyl += wy;
    }
    let v_cone = (h / 3.0) * sum_cone;
    let v_sphere = (h / 3.0) * sum_sphere;
    let v_cyl = (h / 3.0) * sum_cyl;
    let ratio_err = ((v_sphere / v_cone) - 2.0)
        .abs()
        .max(((v_cyl / v_cone) - 3.0).abs());
    (max_slice_err, v_sphere, v_cyl, ratio_err)
}

#[derive(Debug, Clone)]
struct PolyhedronMesh {
    name: &'static str,
    verts: Vec<[f64; 3]>,
    faces: Vec<Vec<usize>>,
    edges: Vec<(usize, usize)>,
}

fn build_platonic_mesh(index: usize) -> PolyhedronMesh {
    let phi = 0.5 * (1.0 + 5.0_f64.sqrt());
    let inv_phi = 1.0 / phi;

    let (name, mut verts, faces): (&'static str, Vec<[f64; 3]>, Vec<Vec<usize>>) = match index % 5 {
        0 => (
            "Tetrahedron",
            vec![
                [1.0, 1.0, 1.0],
                [1.0, -1.0, -1.0],
                [-1.0, 1.0, -1.0],
                [-1.0, -1.0, 1.0],
            ],
            vec![
                vec![0, 2, 1],
                vec![0, 1, 3],
                vec![0, 3, 2],
                vec![1, 2, 3],
            ],
        ),
        1 => (
            "Cube (Hexahedron)",
            vec![
                [-1.0, -1.0, -1.0],
                [1.0, -1.0, -1.0],
                [1.0, 1.0, -1.0],
                [-1.0, 1.0, -1.0],
                [-1.0, -1.0, 1.0],
                [1.0, -1.0, 1.0],
                [1.0, 1.0, 1.0],
                [-1.0, 1.0, 1.0],
            ],
            vec![
                vec![0, 3, 2, 1],
                vec![4, 5, 6, 7],
                vec![0, 1, 5, 4],
                vec![1, 2, 6, 5],
                vec![2, 3, 7, 6],
                vec![3, 0, 4, 7],
            ],
        ),
        2 => (
            "Octahedron",
            vec![
                [1.0, 0.0, 0.0],
                [-1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, -1.0, 0.0],
                [0.0, 0.0, 1.0],
                [0.0, 0.0, -1.0],
            ],
            vec![
                vec![0, 2, 4],
                vec![2, 1, 4],
                vec![1, 3, 4],
                vec![3, 0, 4],
                vec![2, 0, 5],
                vec![1, 2, 5],
                vec![3, 1, 5],
                vec![0, 3, 5],
            ],
        ),
        3 => {
            let v = vec![
                [1.0, 1.0, 1.0],
                [1.0, 1.0, -1.0],
                [1.0, -1.0, 1.0],
                [1.0, -1.0, -1.0],
                [-1.0, 1.0, 1.0],
                [-1.0, 1.0, -1.0],
                [-1.0, -1.0, 1.0],
                [-1.0, -1.0, -1.0],
                [0.0, inv_phi, phi],
                [0.0, inv_phi, -phi],
                [0.0, -inv_phi, phi],
                [0.0, -inv_phi, -phi],
                [inv_phi, phi, 0.0],
                [inv_phi, -phi, 0.0],
                [-inv_phi, phi, 0.0],
                [-inv_phi, -phi, 0.0],
                [phi, 0.0, inv_phi],
                [phi, 0.0, -inv_phi],
                [-phi, 0.0, inv_phi],
                [-phi, 0.0, -inv_phi],
            ];
            let f = vec![
                vec![0, 8, 10, 2, 16],
                vec![0, 16, 17, 1, 12],
                vec![0, 12, 14, 4, 8],
                vec![8, 4, 18, 6, 10],
                vec![10, 6, 15, 13, 2],
                vec![2, 13, 3, 17, 16],
                vec![1, 17, 3, 11, 9],
                vec![1, 9, 5, 14, 12],
                vec![4, 14, 5, 19, 18],
                vec![18, 19, 7, 15, 6],
                vec![13, 15, 7, 11, 3],
                vec![9, 11, 7, 19, 5],
            ];
            ("Dodecahedron", v, f)
        }
        _ => {
            let v = vec![
                [-1.0, phi, 0.0],
                [1.0, phi, 0.0],
                [-1.0, -phi, 0.0],
                [1.0, -phi, 0.0],
                [0.0, -1.0, phi],
                [0.0, 1.0, phi],
                [0.0, -1.0, -phi],
                [0.0, 1.0, -phi],
                [phi, 0.0, -1.0],
                [phi, 0.0, 1.0],
                [-phi, 0.0, -1.0],
                [-phi, 0.0, 1.0],
            ];
            let f = vec![
                vec![0, 11, 5],
                vec![0, 5, 1],
                vec![0, 1, 7],
                vec![0, 7, 10],
                vec![0, 10, 11],
                vec![1, 5, 9],
                vec![5, 11, 4],
                vec![11, 10, 2],
                vec![10, 7, 6],
                vec![7, 1, 8],
                vec![3, 9, 4],
                vec![3, 4, 2],
                vec![3, 2, 6],
                vec![3, 6, 8],
                vec![3, 8, 9],
                vec![4, 9, 5],
                vec![2, 4, 11],
                vec![6, 2, 10],
                vec![8, 6, 7],
                vec![9, 8, 1],
            ];
            ("Icosahedron", v, f)
        }
    };

    // Normalize radius to ~1.35 for uniform display
    for v in &mut verts {
        let len = norm3(*v).max(1e-12);
        *v = scale3(*v, 1.35 / len);
    }

    let mut edge_set = std::collections::BTreeSet::new();
    for f in &faces {
        for idx in 0..f.len() {
            let a = f[idx];
            let b = f[(idx + 1) % f.len()];
            edge_set.insert((a.min(b), a.max(b)));
        }
    }
    let edges = edge_set.into_iter().collect();

    PolyhedronMesh {
        name,
        verts,
        faces,
        edges,
    }
}

/// Evaluates Euler's formula `V - E + F = 2` and Descartes' vertex angular defect directly from 3D Platonic meshes.
pub fn platonic_solids_euler_descartes() -> Vec<(&'static str, i64, i64, i64, i64, f64)> {
    let mut out = Vec::with_capacity(5);
    for idx in 0..5 {
        let mesh = build_platonic_mesh(idx);
        let v_cnt = mesh.verts.len() as i64;
        let e_cnt = mesh.edges.len() as i64;
        let f_cnt = mesh.faces.len() as i64;
        let mut incident_sum = vec![0.0_f64; mesh.verts.len()];
        for f in &mesh.faces {
            let k = f.len();
            for pos in 0..k {
                let prev = mesh.verts[f[(pos + k - 1) % k]];
                let cur = mesh.verts[f[pos]];
                let next = mesh.verts[f[(pos + 1) % k]];
                let u = normalize3(sub3(prev, cur));
                let w = normalize3(sub3(next, cur));
                incident_sum[f[pos]] += dot3(u, w).clamp(-1.0, 1.0).acos();
            }
        }
        let total_defect: f64 = incident_sum.into_iter().map(|s| 2.0 * PI - s).sum();
        out.push((
            mesh.name,
            v_cnt,
            e_cnt,
            f_cnt,
            v_cnt - e_cnt + f_cnt,
            total_defect,
        ));
    }
    out
}

/// Viviani's theorem on 2D equilateral triangles and 3D regular tetrahedra (`edge = 1.0`).
pub fn viviani_simplex_verify(u: f64, v: f64, w: f64) -> (f64, f64, f64) {
    let w0 = u.abs().max(0.02);
    let w1 = v.abs().max(0.02);
    let w2 = w.abs().max(0.02);
    let w3 = ((u.abs() + v.abs() + w.abs()) * 0.35).max(0.03);

    // 2D equilateral triangle of side 1.0
    let tri = [
        (0.0_f64, 0.0_f64),
        (1.0_f64, 0.0_f64),
        (0.5_f64, 0.5 * 3.0_f64.sqrt()),
    ];
    let s2 = w0 + w1 + w2;
    let p2 = (
        (w0 * tri[0].0 + w1 * tri[1].0 + w2 * tri[2].0) / s2,
        (w0 * tri[0].1 + w1 * tri[1].1 + w2 * tri[2].1) / s2,
    );
    let mut sum_2d = 0.0;
    for i in 0..3 {
        let a = tri[i];
        let b = tri[(i + 1) % 3];
        let e = (b.0 - a.0, b.1 - a.1);
        let elen = (e.0 * e.0 + e.1 * e.1).sqrt();
        let cross_abs = ((p2.0 - a.0) * e.1 - (p2.1 - a.1) * e.0).abs();
        sum_2d += cross_abs / elen;
    }
    let h2 = 0.5 * 3.0_f64.sqrt();
    let err_2d = (sum_2d - h2).abs();

    // 3D regular tetrahedron of edge 1.0
    let h3 = (2.0_f64 / 3.0).sqrt();
    let tet = [
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.5, 0.5 * 3.0_f64.sqrt(), 0.0],
        [0.5, 3.0_f64.sqrt() / 6.0, h3],
    ];
    let s3 = w0 + w1 + w2 + w3;
    let p3 = [
        (w0 * tet[0][0] + w1 * tet[1][0] + w2 * tet[2][0] + w3 * tet[3][0]) / s3,
        (w0 * tet[0][1] + w1 * tet[1][1] + w2 * tet[2][1] + w3 * tet[3][1]) / s3,
        (w0 * tet[0][2] + w1 * tet[1][2] + w2 * tet[2][2] + w3 * tet[3][2]) / s3,
    ];
    let mut sum_3d = 0.0;
    for opp in 0..4 {
        let idxs: Vec<usize> = (0..4).filter(|&k| k != opp).collect();
        let a = tet[idxs[0]];
        let b = tet[idxs[1]];
        let c = tet[idxs[2]];
        let mut nrm = normalize3(cross3(sub3(b, a), sub3(c, a)));
        if dot3(nrm, sub3(tet[opp], a)) < 0.0 {
            nrm = scale3(nrm, -1.0);
        }
        sum_3d += dot3(sub3(p3, a), nrm);
    }
    let err_3d = (sum_3d - h3).abs();

    (err_2d, err_3d, h3)
}

/// Pappus's centroid theorem vs. discrete triangulated 3D torus volume (`V = 2*pi^2 * R * r^2`).
pub fn pappus_torus_verify(
    major_r: f64,
    minor_r: f64,
    seg_u: usize,
    seg_v: usize,
) -> (f64, f64, f64, f64) {
    let r_maj = major_r.abs().max(0.5);
    let r_min = minor_r.abs().clamp(0.05, r_maj * 0.9);
    let nu = seg_u.clamp(8, 256);
    let nv = seg_v.clamp(8, 256);

    let v_exact = 2.0 * PI * PI * r_maj * r_min * r_min;
    let a_exact = 4.0 * PI * PI * r_maj * r_min;

    let vertex = |i: usize, j: usize| -> [f64; 3] {
        let u = 2.0 * PI * ((i % nu) as f64) / (nu as f64);
        let v = 2.0 * PI * ((j % nv) as f64) / (nv as f64);
        let rad = r_maj + r_min * v.cos();
        [rad * u.cos(), r_min * v.sin(), rad * u.sin()]
    };

    let signed_vol: f64 = (0..nu)
        .into_par_iter()
        .map(|i| {
            let mut local = 0.0_f64;
            for j in 0..nv {
                let p00 = vertex(i, j);
                let p10 = vertex(i + 1, j);
                let p11 = vertex(i + 1, j + 1);
                let p01 = vertex(i, j + 1);
                local += dot3(p00, cross3(p10, p11)) / 6.0;
                local += dot3(p00, cross3(p11, p01)) / 6.0;
            }
            local
        })
        .sum();
    let v_mesh = signed_vol.abs();
    let rel_err = (v_mesh - v_exact).abs() / v_exact;
    (v_exact, v_mesh, a_exact, rel_err)
}

// ==================== MODULE RUNNERS ====================

pub fn run_p1_area_dissection(params: &ExplorerParams) -> ComputationReport {
    let preset = params.preset_index % 4;
    let mut scene_2d = Scene2D::default();
    let mut scene_3d = Scene3D::default();
    let mut logs = Vec::new();
    let metrics;
    let headline;

    match preset {
        0 => {
            // Bhāskara / Zhoubi Suanjing Pythagorean Dissection
            let scale = params.secondary_param.clamp(0.4, 2.2);
            let a_raw = (params.primary_n.clamp(1, 20) as f64) * scale;
            let b_raw = (params.span.clamp(2, 24) as f64) * scale;
            let (a, b) = if a_raw <= b_raw {
                (a_raw, b_raw)
            } else {
                (b_raw, a_raw)
            };
            let (c, outer_area, dissected, res) = pythagorean_dissection_verify(a, b);
            let side = a + b;
            let outer_sq = vec![(0.0, 0.0), (side, 0.0), (side, side), (0.0, side)];
            scene_2d.polylines.push(Polyline2D {
                points: outer_sq,
                color: RgbaColor::SLATE,
                width: 2.0,
                label: Some(format!("(a+b)^2 = {:.2}", side * side)),
                closed: true,
            });

            // 4 corner right triangles leaving tilted central square c^2
            let tris = [
                [(0.0, 0.0), (a, 0.0), (0.0, b)],
                [(a, 0.0), (side, 0.0), (side, a)],
                [(side, a), (side, side), (b, side)],
                [(b, side), (0.0, side), (0.0, b)],
            ];
            let tri_cols = [
                RgbaColor::CYAN,
                RgbaColor::EMERALD,
                RgbaColor::VIOLET,
                RgbaColor::AMBER,
            ];
            for (idx, t) in tris.iter().enumerate() {
                scene_2d.polylines.push(Polyline2D {
                    points: t.to_vec(),
                    color: tri_cols[idx],
                    width: 2.2,
                    label: None,
                    closed: true,
                });
                push_extruded_tri_3d(&mut scene_3d, *t, 0.0, 0.35, tri_cols[idx], RgbaColor::SLATE);
            }
            // Inner c^2 square
            scene_2d.polylines.push(Polyline2D {
                points: vec![(a, 0.0), (side, a), (b, side), (0.0, b)],
                color: RgbaColor::GOLD,
                width: 2.8,
                label: Some(format!("c^2 = {:.2}", outer_area)),
                closed: true,
            });
            // Bhāskara inner (b - a)^2 core inside c^2
            let center_diff = b - a;
            if center_diff > 1e-6 {
                scene_2d.rects.push(Rect2D {
                    x0: 0.5 * (side - center_diff),
                    y0: 0.5 * (side - center_diff),
                    x1: 0.5 * (side + center_diff),
                    y1: 0.5 * (side + center_diff),
                    fill: RgbaColor(250, 204, 21, 65),
                    stroke: RgbaColor::GOLD,
                    label: Some(format!("(b-a)^2={:.2}", center_diff * center_diff)),
                });
            }
            push_box_3d(
                &mut scene_3d,
                [0.5 * side, 0.28, 0.5 * side],
                [center_diff.max(0.25), 0.56, center_diff.max(0.25)],
                RgbaColor(250, 204, 21, 180),
                RgbaColor::GOLD,
            );
            scene_2d.points.push(Point2D {
                x: a,
                y: 0.0,
                color: RgbaColor::GOLD,
                radius: 4.5,
                label: Some(format!("a={a:.2}, b={b:.2}, c={c:.4}")),
            });

            scene_2d.title = format!("Pythagorean Dissection Proof (a={a:.2}, b={b:.2}, c={c:.4})");
            scene_2d.x_label = "Horizontal coordinate (side a + b)".to_string();
            scene_3d.title = format!("3D Extruded Bhaskara Pythagorean Dissection (c^2 = {outer_area:.2})");
            headline = format!(
                "Pythagorean Dissection (a={a:.2}, b={b:.2}, c={c:.4}): c^2 = {outer_area:.4} vs 4(ab/2)+(b-a)^2 = {dissected:.4} (res={res:.2e})"
            );
            metrics = vec![
                MetricCard {
                    label: "Right Triangle".to_string(),
                    value: format!("a={a:.2}, b={b:.2}, c={c:.4}"),
                    detail: "Euclidean hypotenuse magnitude".to_string(),
                },
                MetricCard {
                    label: "Area c^2".to_string(),
                    value: format!("{outer_area:.6}"),
                    detail: "Hypotenuse square inside (a+b)^2".to_string(),
                },
                MetricCard {
                    label: "Bhaskara Sum".to_string(),
                    value: format!("{dissected:.6}"),
                    detail: "4*(a*b/2) + (b - a)^2".to_string(),
                },
                MetricCard {
                    label: "Exact Residual".to_string(),
                    value: format!("{res:.2e}"),
                    detail: "Area conservation invariant".to_string(),
                },
            ];
            logs.push(format!("Embedded 4 congruent right triangles of area {:.4} around central square (b-a)^2 = {:.4}.", 0.5 * a * b, (b - a).powi(2)));
            logs.push(format!("Verified (a+b)^2 - 2ab = a^2 + b^2 = c^2 to {res:.3e} precision."));
        }
        1 => {
            // Euclid Elements I.47 Windmill & Altitude Shear
            let scale = params.secondary_param.clamp(0.4, 2.2);
            let a = (params.primary_n.clamp(1, 20) as f64) * scale;
            let b = (params.span.clamp(2, 24) as f64) * scale;
            let (c, p, q, max_err) = euclid_i47_altitude_verify(a, b);
            // Put hypotenuse on x-axis [0, c], with right-angle vertex C above at (p, h)
            let h = (p * q).sqrt();
            // Triangle C(p, h), A(0, 0), B(c, 0): length AC = sqrt(p^2 + p*q) = sqrt(p*c) = a!
            // Length BC = sqrt(q^2 + p*q) = b!
            scene_2d.polylines.push(Polyline2D {
                points: vec![(0.0, 0.0), (c, 0.0), (p, h)],
                color: RgbaColor::GOLD,
                width: 2.6,
                label: Some("Right Triangle ABC".to_string()),
                closed: true,
            });
            // Hypotenuse square below axis, split by altitude x = p into rectangles p*c and q*c
            scene_2d.rects.push(Rect2D {
                x0: 0.0,
                y0: -c,
                x1: p,
                y1: 0.0,
                fill: RgbaColor(56, 189, 248, 85),
                stroke: RgbaColor::CYAN,
                label: Some(format!("c*p={:.2}", c * p)),
            });
            scene_2d.rects.push(Rect2D {
                x0: p,
                y0: -c,
                x1: c,
                y1: 0.0,
                fill: RgbaColor(52, 211, 153, 85),
                stroke: RgbaColor::EMERALD,
                label: Some(format!("c*q={:.2}", c * q)),
            });
            // Square on AC (side a) rotated outward from A->C vector
            let vx = -p;
            let vy = -h;
            let nx_ac = vy;
            let ny_ac = -vx; // outward from triangle
            scene_2d.polylines.push(Polyline2D {
                points: vec![
                    (0.0, 0.0),
                    (p, h),
                    (p + nx_ac, h + ny_ac),
                    (nx_ac, ny_ac),
                ],
                color: RgbaColor::CYAN,
                width: 2.0,
                label: Some(format!("a^2={:.2}", a * a)),
                closed: true,
            });
            // Square on BC (side b) rotated outward
            let ux = c - p;
            let uy = -h;
            let nx_bc = -uy;
            let ny_bc = ux;
            scene_2d.polylines.push(Polyline2D {
                points: vec![
                    (p, h),
                    (c, 0.0),
                    (c + nx_bc, ny_bc),
                    (p + nx_bc, h + ny_bc),
                ],
                color: RgbaColor::EMERALD,
                width: 2.0,
                label: Some(format!("b^2={:.2}", b * b)),
                closed: true,
            });
            scene_2d.polylines.push(Polyline2D {
                points: vec![(p, h), (p, -c)],
                color: RgbaColor::GOLD,
                width: 1.8,
                label: None,
                closed: false,
            });
            scene_2d.points.push(Point2D {
                x: p,
                y: h,
                color: RgbaColor::CORAL,
                radius: 4.5,
                label: Some(format!("C (h={h:.2})")),
            });
            scene_2d.title = format!("Euclid Elements I.47 Altitude Shear: c*p=a^2={:.2}, c*q=b^2={:.2}", a * a, b * b);
            scene_2d.x_label = "Hypotenuse projection p + q = c".to_string();

            push_box_3d(
                &mut scene_3d,
                [0.5 * p, 0.25, -0.5 * c],
                [p, 0.5, c],
                RgbaColor(56, 189, 248, 175),
                RgbaColor::CYAN,
            );
            push_box_3d(
                &mut scene_3d,
                [p + 0.5 * q, 0.25, -0.5 * c],
                [q, 0.5, c],
                RgbaColor(52, 211, 153, 175),
                RgbaColor::EMERALD,
            );
            push_extruded_tri_3d(
                &mut scene_3d,
                [(0.0, 0.0), (c, 0.0), (p, h)],
                0.0,
                0.65,
                RgbaColor::GOLD,
                RgbaColor::AMBER,
            );
            scene_3d.title = "3D Euclid I.47 Area Projection Slabs (c*p = a^2, c*q = b^2)".to_string();

            headline = format!(
                "Euclid I.47 Altitude Shear: c*p = {:.4} (a^2={:.4}), c*q = {:.4} (b^2={:.4}), max err = {max_err:.2e}",
                c * p,
                a * a,
                c * q,
                b * b
            );
            metrics = vec![
                MetricCard {
                    label: "Left Projection c*p".to_string(),
                    value: format!("{:.6}", c * p),
                    detail: format!("Equals a^2 = {:.6}", a * a),
                },
                MetricCard {
                    label: "Right Projection c*q".to_string(),
                    value: format!("{:.6}", c * q),
                    detail: format!("Equals b^2 = {:.6}", b * b),
                },
                MetricCard {
                    label: "Altitude h".to_string(),
                    value: format!("{h:.6}"),
                    detail: format!("p={p:.4}, q={q:.4}"),
                },
                MetricCard {
                    label: "Max Shear Error".to_string(),
                    value: format!("{max_err:.2e}"),
                    detail: "triangle congruence + shear proof".to_string(),
                },
            ];
            logs.push(format!("Altitude from C divides c={c:.4} into p={p:.4} and q={q:.4}."));
            logs.push(format!("Verified c*p = a^2 = {:.4} and c*q = b^2 = {:.4}.", a * a, b * b));
        }
        2 => {
            // Geometric Series Square Tiling: sum_{k=1..=N} 2^-k = 1 - 2^-N
            let depth = (params.span as usize).clamp(3, 16);
            let (partial_sum, exact, rem) = geometric_series_tiling_verify(depth);
            let palette = [
                RgbaColor(56, 189, 248, 175),
                RgbaColor(52, 211, 153, 175),
                RgbaColor(250, 204, 21, 175),
                RgbaColor(167, 139, 250, 175),
                RgbaColor(251, 113, 133, 175),
                RgbaColor(245, 158, 11, 175),
            ];
            let mut x0 = 0.0_f64;
            let y0 = 0.0_f64;
            let x1 = 1.0_f64;
            let mut y1 = 1.0_f64;
            for k in 1..=depth {
                let col = palette[(k - 1) % palette.len()];
                let lbl = if k <= 5 { Some(format!("1/{}", 1usize << k)) } else { None };
                let (rx0, ry0, rx1, ry1) = if k % 2 == 1 {
                    let xm = 0.5 * (x0 + x1);
                    let r = (x0, y0, xm, y1);
                    x0 = xm;
                    r
                } else {
                    let ym = 0.5 * (y0 + y1);
                    let r = (x0, ym, x1, y1);
                    y1 = ym;
                    r
                };
                scene_2d.rects.push(Rect2D {
                    x0: rx0,
                    y0: ry0,
                    x1: rx1,
                    y1: ry1,
                    fill: col,
                    stroke: RgbaColor::SLATE,
                    label: lbl,
                });
                let height = 0.15 * (k as f64);
                push_box_3d(
                    &mut scene_3d,
                    [0.5 * (rx0 + rx1) - 0.5, 0.5 * height, 0.5 * (ry0 + ry1) - 0.5],
                    [(rx1 - rx0).max(0.01), height, (ry1 - ry0).max(0.01)],
                    col,
                    RgbaColor::GOLD,
                );
            }
            scene_2d.polylines.push(Polyline2D {
                points: vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)],
                color: RgbaColor::GOLD,
                width: 2.4,
                label: None,
                closed: true,
            });
            scene_2d.title = format!("Unit Square Geometric Series Tiling (N = {depth} terms, S_N = {partial_sum:.6})");
            scene_2d.x_label = "Unit interval x in [0, 1]".to_string();
            scene_3d.title = format!("3D Stepped Halving Tower for sum 2^-k (N = {depth})");

            headline = format!(
                "Geometric Series Tiling (N={depth}): S_N = {partial_sum:.10} vs 1 - 2^(-{depth}) = {exact:.10} (rem = {rem:.3e})"
            );
            metrics = vec![
                MetricCard {
                    label: format!("Partial Sum S_{depth}"),
                    value: format!("{partial_sum:.10}"),
                    detail: "Direct sum of geometric rectangles".to_string(),
                },
                MetricCard {
                    label: "Exact 1 - 2^-N".to_string(),
                    value: format!("{exact:.10}"),
                    detail: format!("Remaining corner = {rem:.4e}"),
                },
                MetricCard {
                    label: "Parity Residual".to_string(),
                    value: format!("{:.2e}", (partial_sum - exact).abs()),
                    detail: "Area conservation inside [0,1]^2".to_string(),
                },
                MetricCard {
                    label: "Asymptotic Limit".to_string(),
                    value: "1.000000".to_string(),
                    detail: "Monotone visual proof without limits calculus".to_string(),
                },
            ];
            logs.push(format!("Tiled {depth} alternating halving rectangles into [0,1] x [0,1]."));
            logs.push(format!("Uncovered corner rectangle has exact area 2^(-{depth}) = {rem:.6e}."));
        }
        _ => {
            // Nicomachus Sum of Cubes: sum_{k=1..=n} k^3 = (sum_{k=1..=n} k)^2
            let n = params.primary_n.clamp(2, 10);
            let (cube_sum, tri_sq, exact_ok) = nicomachus_cubes_verify(n);
            let t_n = (n * (n + 1) / 2) as f64;
            let palette = [
                RgbaColor(56, 189, 248, 170),
                RgbaColor(52, 211, 153, 170),
                RgbaColor(250, 204, 21, 170),
                RgbaColor(167, 139, 250, 170),
                RgbaColor(251, 113, 133, 170),
                RgbaColor(245, 158, 11, 170),
            ];
            for k in 1..=n {
                let s_prev = ((k - 1) * k / 2) as f64;
                let s_cur = (k * (k + 1) / 2) as f64;
                let col = palette[((k - 1) as usize) % palette.len()];
                // Vertical gnomon strip: [s_prev, s_cur] x [0, s_cur]
                scene_2d.rects.push(Rect2D {
                    x0: s_prev,
                    y0: 0.0,
                    x1: s_cur,
                    y1: s_cur,
                    fill: col,
                    stroke: RgbaColor::GOLD,
                    label: if k <= 6 { Some(format!("{k}^3={}", k * k * k)) } else { None },
                });
                // Horizontal companion wing: [0, s_prev] x [s_prev, s_cur]
                if k > 1 {
                    scene_2d.rects.push(Rect2D {
                        x0: 0.0,
                        y0: s_prev,
                        x1: s_prev,
                        y1: s_cur,
                        fill: col,
                        stroke: RgbaColor::GOLD,
                        label: None,
                    });
                }
                let h = (k as f64) * 0.45;
                push_box_3d(
                    &mut scene_3d,
                    [0.5 * (s_prev + s_cur) - 0.5 * t_n, 0.5 * h, 0.5 * s_cur - 0.5 * t_n],
                    [(s_cur - s_prev), h, s_cur],
                    col,
                    RgbaColor::GOLD,
                );
                if k > 1 {
                    push_box_3d(
                        &mut scene_3d,
                        [0.5 * s_prev - 0.5 * t_n, 0.5 * h, 0.5 * (s_prev + s_cur) - 0.5 * t_n],
                        [s_prev, h, (s_cur - s_prev)],
                        col,
                        RgbaColor::GOLD,
                    );
                }
            }
            scene_2d.title = format!("Nicomachus Gnomon Proof (n={n}): sum k^3 = {cube_sum} = ({})^2", (n * (n + 1) / 2));
            scene_2d.x_label = format!("Triangular side T_{n} = {}", n * (n + 1) / 2);
            scene_3d.title = format!("3D Nicomachus Gnomon Shells (k=1..{n}, Sum = {cube_sum})");

            headline = format!(
                "Nicomachus Theorem (n={n}): 1^3+...+{n}^3 = {cube_sum} == T_{n}^2 = {tri_sq} (exact: {exact_ok})"
            );
            metrics = vec![
                MetricCard {
                    label: format!("Sum of Cubes (1..{n})"),
                    value: format!("{cube_sum}"),
                    detail: "Total volume / gnomon shell area".to_string(),
                },
                MetricCard {
                    label: format!("Squared Triangle T_{n}^2"),
                    value: format!("{tri_sq}"),
                    detail: format!("T_{n} = {}", n * (n + 1) / 2),
                },
                MetricCard {
                    label: "Exact Equality".to_string(),
                    value: format!("{exact_ok}"),
                    detail: "128-bit exact integer verification".to_string(),
                },
                MetricCard {
                    label: "Gnomon Layer k".to_string(),
                    value: format!("{} = {n}^3", n * n * n),
                    detail: "Two k*(T_{k-1}) rectangles + corner or k-odd symmetry".to_string(),
                },
            ];
            logs.push(format!("Tiled {n} nested L-shaped gnomons filling T_{n} x T_{n} square."));
            logs.push(format!("Verified sum k^3 = {cube_sum} equals T_{n}^2 = {tri_sq}."));
        }
    }

    pad_aspect_2d(&mut scene_2d, 1.65);

    ComputationReport {
        module: ModuleId::P1AreaDissectionProofs,
        threads_used: params.threads,
        elapsed_ms: 0.0,
        headline,
        metrics,
        log_lines: logs,
        prefer_3d: false,
        scene_2d,
        scene_3d,
    }
}

pub fn run_p2_circle_conic_non_euclidean(params: &ExplorerParams) -> ComputationReport {
    let preset = params.preset_index % 4;
    let mut scene_2d = Scene2D::default();
    let mut scene_3d = Scene3D::default();
    let mut logs = Vec::new();
    let metrics;
    let headline;
    let mut prefer_3d = false;

    match preset {
        0 => {
            // Archimedes Method of Exhaustion for Pi
            let doublings = (params.primary_n as u32).clamp(0, 5);
            let (m, lower, upper, gap) = archimedes_exhaustion_pi(doublings);
            let mut circle_pts = Vec::with_capacity(128);
            for i in 0..128 {
                let th = 2.0 * PI * (i as f64) / 128.0;
                circle_pts.push((th.cos(), th.sin()));
            }
            scene_2d.polylines.push(Polyline2D {
                points: circle_pts,
                color: RgbaColor::GOLD,
                width: 2.2,
                label: Some("Unit Circle (Half-Circ = pi)".to_string()),
                closed: true,
            });
            let draw_m = m.min(96);
            let mut ins_pts = Vec::with_capacity(draw_m as usize);
            let mut cir_pts = Vec::with_capacity(draw_m as usize);
            let half_a = PI / (draw_m as f64);
            let r_cir = 1.0 / half_a.cos();
            for i in 0..draw_m {
                let th = 2.0 * PI * (i as f64) / (draw_m as f64);
                let px = th.cos();
                let py = th.sin();
                ins_pts.push((px, py));
                let phi = th + half_a;
                cir_pts.push((r_cir * phi.cos(), r_cir * phi.sin()));
                scene_2d.points.push(Point2D {
                    x: px,
                    y: py,
                    color: RgbaColor::CYAN,
                    radius: 2.8,
                    label: None,
                });
            }
            scene_2d.polylines.push(Polyline2D {
                points: ins_pts,
                color: RgbaColor::CYAN,
                width: 1.8,
                label: Some(format!("Inscribed {m}-gon")),
                closed: true,
            });
            scene_2d.polylines.push(Polyline2D {
                points: cir_pts,
                color: RgbaColor::CORAL,
                width: 1.8,
                label: Some(format!("Circumscribed {m}-gon")),
                closed: true,
            });
            scene_2d.title = format!("Archimedes Exhaustion: {m}-gon Bounds ({lower:.6} < pi < {upper:.6})");
            scene_2d.x_label = "Cyan = Inscribed | Gold = Circle | Coral = Circumscribed".to_string();

            // 3D Convergence Prism Stack across doublings 0..=4
            for k in 0..=4u32 {
                let (mk, lk, uk, _) = archimedes_exhaustion_pi(k);
                let y_lvl = (k as f64) * 0.45 - 0.9;
                let mut ring = Vec::new();
                let sides = (mk.min(48)) as usize;
                for s in 0..sides {
                    let th = 2.0 * PI * (s as f64) / (sides as f64);
                    ring.push([th.cos(), y_lvl, th.sin()]);
                }
                for s in 0..sides {
                    let a = ring[s];
                    let b = ring[(s + 1) % sides];
                    scene_3d.edges.push(Edge3D {
                        a,
                        b,
                        color: if k == doublings { RgbaColor::GOLD } else { RgbaColor::CYAN },
                        width: 1.6,
                    });
                }
                scene_3d.nodes.push(Node3D {
                    pos: [1.35, y_lvl, 0.0],
                    radius: 4.0,
                    color: RgbaColor::EMERALD,
                    label: Some(format!("{mk}-gon: [{lk:.5}, {uk:.5}]")),
                });
            }
            scene_3d.title = "3D Archimedes Polygon Doubling Convergence Stack".to_string();

            headline = format!(
                "Archimedes {m}-gon Exhaustion: {lower:.8} < pi ({:.8}) < {upper:.8} | Bracket width = {gap:.3e}",
                PI
            );
            metrics = vec![
                MetricCard {
                    label: "Polygon Sides m".to_string(),
                    value: format!("{m} (6*2^{doublings})"),
                    detail: "Archimedes classical hexagon doubling".to_string(),
                },
                MetricCard {
                    label: "Lower Bound L_m".to_string(),
                    value: format!("{lower:.10}"),
                    detail: format!("err = {:.3e}", (PI - lower).abs()),
                },
                MetricCard {
                    label: "Upper Bound U_m".to_string(),
                    value: format!("{upper:.10}"),
                    detail: format!("err = {:.3e}", (upper - PI).abs()),
                },
                MetricCard {
                    label: "Bracket Width".to_string(),
                    value: format!("{gap:.3e}"),
                    detail: "O(1 / m^2) quadratic convergence".to_string(),
                },
            ];
            logs.push(format!("Inscribed {m}-gon gives pi > {lower:.8}; circumscribed gives pi < {upper:.8}."));
            logs.push(format!("Archimedes 96-gon corresponds to doublings=4 (3.14103 < pi < 3.14271)."));
        }
        1 => {
            // Thales Theorem + Inscribed Angle + Ptolemy Cyclic Quadrilateral
            let theta_deg = (params.secondary_param * 100.0).clamp(22.0, 158.0);
            let (thales_dot, ptolemy_err, inscribed_deg) = thales_ptolemy_verify(theta_deg);
            for i in 0..96 {
                let th0 = 2.0 * PI * (i as f64) / 96.0;
                let th1 = 2.0 * PI * ((i + 1) as f64) / 96.0;
                scene_2d.polylines.push(Polyline2D {
                    points: vec![(th0.cos(), th0.sin()), (th1.cos(), th1.sin())],
                    color: RgbaColor::SLATE,
                    width: 1.5,
                    label: None,
                    closed: false,
                });
            }
            let th = theta_deg.to_radians();
            let a = (-1.0, 0.0);
            let b = (1.0, 0.0);
            let p = (th.cos(), th.sin());
            let q = ((-0.7_f64).cos(), (-0.7_f64).sin());
            scene_2d.polylines.push(Polyline2D {
                points: vec![a, b],
                color: RgbaColor::GOLD,
                width: 2.2,
                label: None,
                closed: false,
            });
            scene_2d.polylines.push(Polyline2D {
                points: vec![a, p, b, q],
                color: RgbaColor::CYAN,
                width: 2.2,
                label: None,
                closed: true,
            });
            scene_2d.polylines.push(Polyline2D {
                points: vec![p, q],
                color: RgbaColor::EMERALD,
                width: 1.8,
                label: None,
                closed: false,
            });
            // Central angle radii O->A, O->P, O->B
            scene_2d.polylines.push(Polyline2D {
                points: vec![(0.0, 0.0), p],
                color: RgbaColor::VIOLET,
                width: 1.4,
                label: None,
                closed: false,
            });
            for (pt, lbl, col) in [
                (a, "A(-1,0)", RgbaColor::GOLD),
                (b, "B(1,0)", RgbaColor::GOLD),
                (p, "P (90 deg)", RgbaColor::CORAL),
                (q, "Q (Cyclic)", RgbaColor::EMERALD),
                ((0.0, 0.0), "O", RgbaColor::VIOLET),
            ] {
                scene_2d.points.push(Point2D {
                    x: pt.0,
                    y: pt.1,
                    color: col,
                    radius: 4.5,
                    label: Some(lbl.to_string()),
                });
            }
            scene_2d.title = format!("Thales Right Angle (dot={thales_dot:.1e}) & Ptolemy Cyclic Identity");
            scene_2d.x_label = format!("theta = {theta_deg:.1} deg | AC*BD = AB*CD + BC*AD");

            // 3D lifted chord cone around circle
            for &(x, y) in &[a, p, b, q] {
                scene_3d.edges.push(Edge3D {
                    a: [x, 0.0, y],
                    b: [0.0, 1.1, 0.0],
                    color: RgbaColor::VIOLET,
                    width: 1.4,
                });
            }
            for (u, v) in [(a, p), (p, b), (b, q), (q, a), (a, b), (p, q)] {
                scene_3d.edges.push(Edge3D {
                    a: [u.0, 0.0, u.1],
                    b: [v.0, 0.0, v.1],
                    color: RgbaColor::CYAN,
                    width: 2.0,
                });
            }
            scene_3d.title = "3D Thales & Ptolemy Cyclic Pyramid".to_string();

            headline = format!(
                "Thales Angle APB = {inscribed_deg:.6} deg (|PA.PB|={thales_dot:.2e}) | Ptolemy err = {ptolemy_err:.2e}"
            );
            metrics = vec![
                MetricCard {
                    label: "Thales Angle APB".to_string(),
                    value: format!("{inscribed_deg:.6} deg"),
                    detail: "Inscribed triangle on diameter AB".to_string(),
                },
                MetricCard {
                    label: "Orthogonality |PA.PB|".to_string(),
                    value: format!("{thales_dot:.2e}"),
                    detail: "Radius equality isosceles proof".to_string(),
                },
                MetricCard {
                    label: "Ptolemy Residual".to_string(),
                    value: format!("{ptolemy_err:.2e}"),
                    detail: "|AC||BD| - (|AB||CD|+|BC||AD|)".to_string(),
                },
                MetricCard {
                    label: "Point P Angle".to_string(),
                    value: format!("{theta_deg:.1} deg"),
                    detail: "Adjust continuous slider to sweep semicircle".to_string(),
                },
            ];
            logs.push(format!("Verified PA . PB = {thales_dot:.3e} => angle APB = {inscribed_deg:.6} deg."));
            logs.push(format!("Ptolemy's Theorem on cyclic quadrilateral APBQ holds within {ptolemy_err:.3e}."));
        }
        2 => {
            // 3D Dandelin Spheres & Conic Focal Property (|PF_1| + |PF_2| = 2a)
            prefer_3d = true;
            let cone_alpha = 0.48_f64;
            let beta = (params.secondary_param * 0.42).clamp(0.12, 0.78);
            let n_s = (params.span as usize).clamp(24, 96);
            let sys = solve_dandelin_system(cone_alpha, beta, n_s);

            // Wireframe cone generators
            let z_top = 3.8_f64;
            let r_top = z_top * sys.alpha.tan();
            let cone_gens = 24usize;
            for i in 0..cone_gens {
                let th = 2.0 * PI * (i as f64) / (cone_gens as f64);
                let rim = [r_top * th.cos(), r_top * th.sin(), z_top];
                scene_3d.edges.push(Edge3D {
                    a: [0.0, 0.0, 0.0],
                    b: rim,
                    color: RgbaColor(148, 163, 184, 130),
                    width: 1.0,
                });
            }
            // Dandelin spheres (lower and upper tangent spheres)
            push_sphere_3d(
                &mut scene_3d,
                [0.0, 0.0, sys.cz1],
                sys.r1,
                10,
                16,
                RgbaColor(56, 189, 248, 115),
            );
            push_sphere_3d(
                &mut scene_3d,
                [0.0, 0.0, sys.cz2],
                sys.r2,
                10,
                16,
                RgbaColor(167, 139, 250, 115),
            );
            // Ellipse curve
            for i in 0..sys.ellipse_pts.len() {
                let p0 = sys.ellipse_pts[i];
                let p1 = sys.ellipse_pts[(i + 1) % sys.ellipse_pts.len()];
                scene_3d.edges.push(Edge3D {
                    a: p0,
                    b: p1,
                    color: RgbaColor::GOLD,
                    width: 2.6,
                });
            }
            // Highlight tangent rays from a sample point P on the ellipse to F1 & F2
            let sample_p = sys.ellipse_pts[sys.ellipse_pts.len() / 6];
            scene_3d.edges.push(Edge3D {
                a: sample_p,
                b: sys.f1,
                color: RgbaColor::EMERALD,
                width: 2.5,
            });
            scene_3d.edges.push(Edge3D {
                a: sample_p,
                b: sys.f2,
                color: RgbaColor::EMERALD,
                width: 2.5,
            });
            scene_3d.edges.push(Edge3D {
                a: [0.0, 0.0, 0.0],
                b: sample_p,
                color: RgbaColor::CORAL,
                width: 1.8,
            });
            scene_3d.nodes.push(Node3D {
                pos: sys.f1,
                radius: 5.5,
                color: RgbaColor::CYAN,
                label: Some("F_1".to_string()),
            });
            scene_3d.nodes.push(Node3D {
                pos: sys.f2,
                radius: 5.5,
                color: RgbaColor::VIOLET,
                label: Some("F_2".to_string()),
            });
            scene_3d.nodes.push(Node3D {
                pos: sample_p,
                radius: 5.0,
                color: RgbaColor::EMERALD,
                label: Some("P".to_string()),
            });
            scene_3d.title = format!(
                "3D Dandelin Spheres & Cone Ellipse (|PF_1|+|PF_2| = {:.4}, e={:.3})",
                sys.two_a, sys.eccentricity
            );

            // 2D ellipse projection onto tilted cutting plane frame
            let u_axis = normalize3([beta.cos(), 0.0, beta.sin()]);
            let v_axis = [0.0, 1.0, 0.0];
            let c_mid = scale3(add3(sys.f1, sys.f2), 0.5);
            let el_2d: Vec<(f64, f64)> = sys
                .ellipse_pts
                .iter()
                .map(|&p| {
                    let d = sub3(p, c_mid);
                    (dot3(d, u_axis), dot3(d, v_axis))
                })
                .collect();
            let f1_2 = (dot3(sub3(sys.f1, c_mid), u_axis), dot3(sub3(sys.f1, c_mid), v_axis));
            let f2_2 = (dot3(sub3(sys.f2, c_mid), u_axis), dot3(sub3(sys.f2, c_mid), v_axis));
            let sp_2 = (dot3(sub3(sample_p, c_mid), u_axis), dot3(sub3(sample_p, c_mid), v_axis));
            scene_2d.polylines.push(Polyline2D {
                points: el_2d,
                color: RgbaColor::GOLD,
                width: 2.4,
                label: Some("Conic Ellipse Section".to_string()),
                closed: true,
            });
            scene_2d.polylines.push(Polyline2D {
                points: vec![f1_2, sp_2, f2_2],
                color: RgbaColor::EMERALD,
                width: 2.0,
                label: Some("|PF_1|+|PF_2|=2a".to_string()),
                closed: false,
            });
            scene_2d.points.push(Point2D {
                x: f1_2.0,
                y: f1_2.1,
                color: RgbaColor::CYAN,
                radius: 4.8,
                label: Some("F_1".to_string()),
            });
            scene_2d.points.push(Point2D {
                x: f2_2.0,
                y: f2_2.1,
                color: RgbaColor::VIOLET,
                radius: 4.8,
                label: Some("F_2".to_string()),
            });
            scene_2d.points.push(Point2D {
                x: sp_2.0,
                y: sp_2.1,
                color: RgbaColor::CORAL,
                radius: 4.5,
                label: Some("P".to_string()),
            });
            scene_2d.title = format!("2D Cutting-Plane Ellipse: e = {:.4}, 2a = {:.4}", sys.eccentricity, sys.two_a);
            scene_2d.x_label = "Major axis coordinate along cutting plane".to_string();

            headline = format!(
                "Dandelin Spheres: e = sin(beta)/cos(alpha) = {:.4}, |PF_1|+|PF_2| = {:.6} (max err = {:.2e})",
                sys.eccentricity, sys.two_a, sys.max_focal_err
            );
            metrics = vec![
                MetricCard {
                    label: "Constant 2a = |PF1|+|PF2|".to_string(),
                    value: format!("{:.6}", sys.two_a),
                    detail: "Equal tangents to Dandelin spheres".to_string(),
                },
                MetricCard {
                    label: "Eccentricity e".to_string(),
                    value: format!("{:.6}", sys.eccentricity),
                    detail: "sin(tilt) / cos(cone_half_angle)".to_string(),
                },
                MetricCard {
                    label: "Max Focal Err".to_string(),
                    value: format!("{:.2e}", sys.max_focal_err),
                    detail: format!("Sampled across {} ellipse points", sys.ellipse_pts.len()),
                },
                MetricCard {
                    label: "Sphere Tangency".to_string(),
                    value: format!("{:.2e}", sys.tangency_err),
                    detail: format!("R1={:.3}, R2={:.3}", sys.r1, sys.r2),
                },
            ];
            logs.push(format!("Fitted lower sphere (cz={:.3}, R={:.3}) & upper sphere (cz={:.3}, R={:.3}).", sys.cz1, sys.r1, sys.cz2, sys.r2));
            logs.push(format!("Evaluated |PF_1| + |PF_2| across {} points: max error = {:.3e}.", sys.ellipse_pts.len(), sys.max_focal_err));
        }
        _ => {
            // Poincaré Disk Hyperbolic Triangle & Gauss-Bonnet Defect
            let r = (params.secondary_param * 0.72).clamp(0.22, 0.90);
            let (angle_sum, defect_area, alpha_deg, curvature) = poincare_hyperbolic_triangle_verify(r);
            let mut disk_pts = Vec::with_capacity(120);
            for i in 0..120 {
                let th = 2.0 * PI * (i as f64) / 120.0;
                disk_pts.push((th.cos(), th.sin()));
            }
            scene_2d.polylines.push(Polyline2D {
                points: disk_pts,
                color: RgbaColor::GOLD,
                width: 2.2,
                label: Some("Poincare Horizon |z| = 1".to_string()),
                closed: true,
            });
            let verts = [
                (r, 0.0),
                (r * (2.0 * PI / 3.0).cos(), r * (2.0 * PI / 3.0).sin()),
                (r * (4.0 * PI / 3.0).cos(), r * (4.0 * PI / 3.0).sin()),
            ];
            for e in 0..3 {
                let a = verts[e];
                let b = verts[(e + 1) % 3];
                if let Some((cx, cy, rg)) = poincare_orthogonal_circle(a, b) {
                    let mut th0 = (a.1 - cy).atan2(a.0 - cx);
                    let mut th1 = (b.1 - cy).atan2(b.0 - cx);
                    let mut dth = th1 - th0;
                    if dth > PI {
                        th1 -= 2.0 * PI;
                        dth = th1 - th0;
                    } else if dth < -PI {
                        th0 -= 2.0 * PI;
                        dth = th1 - th0;
                    }
                    let mut arc = Vec::new();
                    let mut arc_3d = Vec::new();
                    for s in 0..=40 {
                        let t = th0 + dth * (s as f64) / 40.0;
                        let px = cx + rg * t.cos();
                        let py = cy + rg * t.sin();
                        arc.push((px, py));
                        let nr2 = (px * px + py * py).min(0.96);
                        let lift = (1.0 + nr2) / (1.0 - nr2);
                        arc_3d.push([px, (lift - 1.0) * 0.35, py]);
                    }
                    scene_2d.polylines.push(Polyline2D {
                        points: arc,
                        color: RgbaColor::CYAN,
                        width: 2.4,
                        label: None,
                        closed: false,
                    });
                    for w in arc_3d.windows(2) {
                        scene_3d.edges.push(Edge3D {
                            a: w[0],
                            b: w[1],
                            color: RgbaColor::CYAN,
                            width: 2.4,
                        });
                    }
                }
            }
            for (idx, v) in verts.iter().enumerate() {
                scene_2d.points.push(Point2D {
                    x: v.0,
                    y: v.1,
                    color: RgbaColor::CORAL,
                    radius: 5.0,
                    label: Some(format!("V{idx} ({alpha_deg:.1} deg)")),
                });
                let lift = (1.0 + r * r) / (1.0 - r * r);
                scene_3d.nodes.push(Node3D {
                    pos: [v.0, (lift - 1.0) * 0.35, v.1],
                    radius: 5.0,
                    color: RgbaColor::CORAL,
                    label: Some(format!("V{idx}")),
                });
            }
            // Add ambient Poincaré disk orthogonal geodesic family for visual reference
            for k in 0..6 {
                let ang = 2.0 * PI * (k as f64) / 6.0;
                scene_2d.polylines.push(Polyline2D {
                    points: vec![(-ang.cos(), -ang.sin()), (ang.cos(), ang.sin())],
                    color: RgbaColor(148, 163, 184, 70),
                    width: 1.0,
                    label: None,
                    closed: false,
                });
            }
            scene_2d.title = format!("Poincare Disk Geodesic Triangle (sum = {:.2} deg < 180 deg)", angle_sum.to_degrees());
            scene_2d.x_label = format!("Gauss-Bonnet Area = pi - (a+b+c) = {defect_area:.5} (K = {curvature:.1})");
            scene_3d.title = "3D Hyperboloid Sheet Lift of Geodesic Triangle".to_string();

            headline = format!(
                "Hyperbolic Poincare Triangle (r={r:.2}): sum angles = {:.4} deg (<180), Gauss-Bonnet Area = {defect_area:.6}",
                angle_sum.to_degrees()
            );
            metrics = vec![
                MetricCard {
                    label: "Angle Sum".to_string(),
                    value: format!("{:.4} deg", angle_sum.to_degrees()),
                    detail: format!("Each vertex = {alpha_deg:.3} deg (< 60 deg)"),
                },
                MetricCard {
                    label: "Gauss-Bonnet Area".to_string(),
                    value: format!("{defect_area:.6}"),
                    detail: "Area = pi - (alpha + beta + gamma) > 0".to_string(),
                },
                MetricCard {
                    label: "Gaussian Curvature".to_string(),
                    value: format!("{curvature:.1}"),
                    detail: "Constant hyperbolic curvature K = -1".to_string(),
                },
                MetricCard {
                    label: "Euclidean Radius r".to_string(),
                    value: format!("{r:.3}"),
                    detail: "r -> 0 recovers 180 deg; r -> 1 ideal triangle".to_string(),
                },
            ];
            logs.push(format!("Solved orthogonal geodesics on unit disk at r = {r:.3}."));
            logs.push(format!("Vertex angle = {alpha_deg:.4} deg, defect area = {defect_area:.6} steradians."));
        }
    }

    pad_aspect_2d(&mut scene_2d, 1.65);

    ComputationReport {
        module: ModuleId::P2CircleConicNonEuclidean,
        threads_used: params.threads,
        elapsed_ms: 0.0,
        headline,
        metrics,
        log_lines: logs,
        prefer_3d,
        scene_2d,
        scene_3d,
    }
}

pub fn run_p3_solid_cavalieri_polyhedra(params: &ExplorerParams) -> ComputationReport {
    let preset = params.preset_index % 4;
    let mut scene_2d = Scene2D::default();
    let mut scene_3d = Scene3D::default();
    let mut logs = Vec::new();
    let metrics;
    let headline;

    match preset {
        0 => {
            // Archimedes Sphere - Cylinder - Cone (1 : 2 : 3 Cavalieri Slice Proof)
            let radius = 1.0_f64;
            let samples = (params.span as usize).clamp(24, 200);
            let (max_slice_err, v_sphere, v_cyl, ratio_err) =
                cavalieri_sphere_cylinder_cone_verify(radius, samples);
            let z_slice = (params.secondary_param * 0.85).clamp(-0.92, 0.92);

            // 1. Left: Double Cone (x = -2.4), 2. Center: Sphere (x = 0.0), 3. Right: Cylinder (x = +2.4)
            let ring_segs = 28usize;
            for i in 0..ring_segs {
                let th0 = 2.0 * PI * (i as f64) / (ring_segs as f64);
                let th1 = 2.0 * PI * ((i + 1) as f64) / (ring_segs as f64);
                // Cylinder rings at top & bottom (x=+2.4)
                for y in [-1.0, 1.0] {
                    scene_3d.edges.push(Edge3D {
                        a: [2.4 + th0.cos(), y, th0.sin()],
                        b: [2.4 + th1.cos(), y, th1.sin()],
                        color: RgbaColor::CYAN,
                        width: 1.5,
                    });
                }
                if i % 4 == 0 {
                    scene_3d.edges.push(Edge3D {
                        a: [2.4 + th0.cos(), -1.0, th0.sin()],
                        b: [2.4 + th0.cos(), 1.0, th0.sin()],
                        color: RgbaColor(56, 189, 248, 120),
                        width: 1.0,
                    });
                    // Cone generators at x = -2.4
                    scene_3d.edges.push(Edge3D {
                        a: [-2.4, 0.0, 0.0],
                        b: [-2.4 + th0.cos(), 1.0, th0.sin()],
                        color: RgbaColor(251, 113, 133, 140),
                        width: 1.0,
                    });
                    scene_3d.edges.push(Edge3D {
                        a: [-2.4, 0.0, 0.0],
                        b: [-2.4 + th0.cos(), -1.0, th0.sin()],
                        color: RgbaColor(251, 113, 133, 140),
                        width: 1.0,
                    });
                }
                // Double-cone base rims
                for y in [-1.0, 1.0] {
                    scene_3d.edges.push(Edge3D {
                        a: [-2.4 + th0.cos(), y, th0.sin()],
                        b: [-2.4 + th1.cos(), y, th1.sin()],
                        color: RgbaColor::CORAL,
                        width: 1.5,
                    });
                }
            }
            // Center sphere mesh
            push_sphere_3d(&mut scene_3d, [0.0, 0.0, 0.0], 1.0, 10, 18, RgbaColor(52, 211, 153, 105));

            // Highlighted Cavalieri slice rings at height z_slice
            let r_cone = z_slice.abs();
            let r_sph = (1.0 - z_slice * z_slice).max(0.0).sqrt();
            for i in 0..ring_segs {
                let th0 = 2.0 * PI * (i as f64) / (ring_segs as f64);
                let th1 = 2.0 * PI * ((i + 1) as f64) / (ring_segs as f64);
                // Cone slice ring
                scene_3d.edges.push(Edge3D {
                    a: [-2.4 + r_cone * th0.cos(), z_slice, r_cone * th0.sin()],
                    b: [-2.4 + r_cone * th1.cos(), z_slice, r_cone * th1.sin()],
                    color: RgbaColor::GOLD,
                    width: 2.6,
                });
                // Sphere slice ring
                scene_3d.edges.push(Edge3D {
                    a: [r_sph * th0.cos(), z_slice, r_sph * th0.sin()],
                    b: [r_sph * th1.cos(), z_slice, r_sph * th1.sin()],
                    color: RgbaColor::GOLD,
                    width: 2.6,
                });
                // Cylinder slice ring (equal to sum of cone + sphere!)
                scene_3d.edges.push(Edge3D {
                    a: [2.4 + th0.cos(), z_slice, th0.sin()],
                    b: [2.4 + th1.cos(), z_slice, th1.sin()],
                    color: RgbaColor::GOLD,
                    width: 2.6,
                });
            }
            scene_3d.nodes.push(Node3D {
                pos: [-2.4, 1.35, 0.0],
                radius: 4.0,
                color: RgbaColor::CORAL,
                label: Some("Cone (1V)".to_string()),
            });
            scene_3d.nodes.push(Node3D {
                pos: [0.0, 1.35, 0.0],
                radius: 4.0,
                color: RgbaColor::EMERALD,
                label: Some("Sphere (2V)".to_string()),
            });
            scene_3d.nodes.push(Node3D {
                pos: [2.4, 1.35, 0.0],
                radius: 4.0,
                color: RgbaColor::CYAN,
                label: Some("Cylinder (3V)".to_string()),
            });
            scene_3d.title = format!("3D Archimedes 1:2:3 Cavalieri Slice @ z = {z_slice:.2}");

            // 2D Cross-sectional Area Curves A(z) for z in [-1, 1]
            let mut c_sph = Vec::new();
            let mut c_con = Vec::new();
            let mut c_cyl = Vec::new();
            for s in 0..=50 {
                let z = -1.0 + 2.0 * (s as f64) / 50.0;
                c_sph.push((z, PI * (1.0 - z * z)));
                c_con.push((z, PI * z * z));
                c_cyl.push((z, PI));
            }
            scene_2d.polylines.push(Polyline2D {
                points: c_sph,
                color: RgbaColor::EMERALD,
                width: 2.4,
                label: Some("A_sphere(z) = pi(R^2-z^2)".to_string()),
                closed: false,
            });
            scene_2d.polylines.push(Polyline2D {
                points: c_con,
                color: RgbaColor::CORAL,
                width: 2.4,
                label: Some("A_cone(z) = pi z^2".to_string()),
                closed: false,
            });
            scene_2d.polylines.push(Polyline2D {
                points: c_cyl,
                color: RgbaColor::CYAN,
                width: 2.4,
                label: Some("A_cyl(z) = A_sph+A_con = pi R^2".to_string()),
                closed: false,
            });
            scene_2d.polylines.push(Polyline2D {
                points: vec![(z_slice, 0.0), (z_slice, PI * 1.05)],
                color: RgbaColor::GOLD,
                width: 1.8,
                label: None,
                closed: false,
            });
            scene_2d.title = "Cavalieri Cross-Section Area Proof: A_sphere(z) + A_cone(z) == pi*R^2".to_string();
            scene_2d.x_label = "Vertical slice coordinate z in [-R, R]".to_string();

            headline = format!(
                "Archimedes Cavalieri 1:2:3: V_sphere={v_sphere:.6}, V_cyl={v_cyl:.6}, max slice err={max_slice_err:.2e}, ratio err={ratio_err:.2e}"
            );
            metrics = vec![
                MetricCard {
                    label: "Slice Identity Err".to_string(),
                    value: format!("{max_slice_err:.2e}"),
                    detail: "pi(R^2-z^2) + pi*z^2 == pi*R^2".to_string(),
                },
                MetricCard {
                    label: "1 : 2 : 3 Ratio Err".to_string(),
                    value: format!("{ratio_err:.2e}"),
                    detail: "V_cone : V_sphere : V_cyl = 1 : 2 : 3".to_string(),
                },
                MetricCard {
                    label: "Sphere Vol (R=1)".to_string(),
                    value: format!("{v_sphere:.6}"),
                    detail: format!("Exact 4*pi/3 = {:.6}", 4.0 * PI / 3.0),
                },
                MetricCard {
                    label: "Active Slice z".to_string(),
                    value: format!("{z_slice:.2}"),
                    detail: format!("r_sph={r_sph:.3}, r_cone={r_cone:.3}"),
                },
            ];
            logs.push(format!("Swept {samples} horizontal Cavalieri slices over [-1, 1]: max slice discrepancy = {max_slice_err:.3e}."));
            logs.push(format!("Simpson 3D integration confirms V_cone : V_sphere : V_cyl = 1 : 2 : 3 within {ratio_err:.3e}."));
        }
        1 => {
            // Platonic Solids Euler (V - E + F = 2) & Descartes Angular Defect (4*pi)
            let solid_idx = (params.primary_n as usize) % 5;
            let mesh = build_platonic_mesh(solid_idx);
            let all_stats = platonic_solids_euler_descartes();
            let (name, v_c, e_c, f_c, euler, defect) = all_stats[solid_idx];

            for f in &mesh.faces {
                // Fan-triangulate polygon face
                for k in 1..(f.len() - 1) {
                    let p0 = mesh.verts[f[0]];
                    let p1 = mesh.verts[f[k]];
                    let p2 = mesh.verts[f[k + 1]];
                    let mut nrm = cross_normal(p0, p1, p2);
                    if dot3(nrm, p0) < 0.0 {
                        nrm = scale3(nrm, -1.0);
                    }
                    scene_3d.faces.push(Face3D {
                        verts: [p0, p1, p2],
                        normal: nrm,
                        color: RgbaColor(56, 189, 248, 175),
                    });
                }
            }
            for &(u, v) in &mesh.edges {
                scene_3d.edges.push(Edge3D {
                    a: mesh.verts[u],
                    b: mesh.verts[v],
                    color: RgbaColor::GOLD,
                    width: 2.0,
                });
            }
            for (idx, &pos) in mesh.verts.iter().enumerate() {
                scene_3d.nodes.push(Node3D {
                    pos,
                    radius: 4.5,
                    color: RgbaColor::EMERALD,
                    label: if idx < 4 { Some(format!("v{idx}")) } else { None },
                });
            }
            scene_3d.title = format!("Platonic Solid: {name} (V={v_c}, E={e_c}, F={f_c}, chi={euler})");

            // 2D comparison of all 5 Platonic solids Descartes angular defect = 4*pi
            for (k, st) in all_stats.iter().enumerate() {
                let x = (k as f64) * 2.0;
                scene_2d.rects.push(Rect2D {
                    x0: x,
                    y0: 0.0,
                    x1: x + 1.2,
                    y1: st.5,
                    fill: if k == solid_idx {
                        RgbaColor::GOLD
                    } else {
                        RgbaColor(56, 189, 248, 160)
                    },
                    stroke: RgbaColor::SLATE,
                    label: Some(format!("{} (V-E+F={})", st.0, st.4)),
                });
            }
            scene_2d.polylines.push(Polyline2D {
                points: vec![(-0.2, 4.0 * PI), (9.4, 4.0 * PI)],
                color: RgbaColor::EMERALD,
                width: 2.0,
                label: Some("Descartes 4*pi Constant".to_string()),
                closed: false,
            });
            scene_2d.title = "Platonic Solids: Euler V - E + F = 2 & Descartes Angular Defect = 4*pi".to_string();
            scene_2d.x_label = "Platonic Solid index (Tetrahedron, Cube, Octahedron, Dodecahedron, Icosahedron)".to_string();

            let err_4pi = (defect - 4.0 * PI).abs();
            headline = format!(
                "{name}: V={v_c}, E={e_c}, F={f_c} (V-E+F={euler}) | Descartes Defect = {defect:.6} (4pi err={err_4pi:.2e})"
            );
            metrics = vec![
                MetricCard {
                    label: "Active Platonic Solid".to_string(),
                    value: name.to_string(),
                    detail: "Set Primary N mod 5 to switch solid".to_string(),
                },
                MetricCard {
                    label: "Euler Characteristic".to_string(),
                    value: format!("{v_c} - {e_c} + {f_c} = {euler}"),
                    detail: "Topological genus-0 S^2 invariant".to_string(),
                },
                MetricCard {
                    label: "Descartes Total Defect".to_string(),
                    value: format!("{defect:.8}"),
                    detail: format!("4*pi = {:.8}", 4.0 * PI),
                },
                MetricCard {
                    label: "Defect Residual".to_string(),
                    value: format!("{err_4pi:.2e}"),
                    detail: "Computed directly from 3D face corners".to_string(),
                },
            ];
            logs.push(format!("Evaluated all 5 Platonic polyhedral meshes: all obey V - E + F = 2."));
            logs.push(format!("{name} 3D vertex angular defect sum = {defect:.8} (|sum - 4pi| = {err_4pi:.3e})."));
        }
        2 => {
            // Viviani Theorem in 2D Equilateral Triangle & 3D Regular Tetrahedron
            let shift = params.secondary_param;
            let u = 0.25 + 0.15 * shift.sin();
            let v = 0.35 + 0.12 * shift.cos();
            let w = 0.40;
            let (err_2d, err_3d, h3) = viviani_simplex_verify(u, v, w);

            // 3D Tetrahedron + interior point P + perpendicular altitude segments to all 4 faces
            let tet = [
                [-0.5, -0.35 * h3, -3.0_f64.sqrt() / 6.0],
                [0.5, -0.35 * h3, -3.0_f64.sqrt() / 6.0],
                [0.0, -0.35 * h3, 3.0_f64.sqrt() / 3.0],
                [0.0, 0.65 * h3, 0.0],
            ];
            let s3 = u + v + w + 0.35;
            let p3 = [
                (u * tet[0][0] + v * tet[1][0] + w * tet[2][0] + 0.35 * tet[3][0]) / s3,
                (u * tet[0][1] + v * tet[1][1] + w * tet[2][1] + 0.35 * tet[3][1]) / s3,
                (u * tet[0][2] + v * tet[1][2] + w * tet[2][2] + 0.35 * tet[3][2]) / s3,
            ];
            for opp in 0..4 {
                let idxs: Vec<usize> = (0..4).filter(|&k| k != opp).collect();
                let a = tet[idxs[0]];
                let b = tet[idxs[1]];
                let c = tet[idxs[2]];
                let mut nrm = normalize3(cross3(sub3(b, a), sub3(c, a)));
                if dot3(nrm, sub3(tet[opp], a)) < 0.0 {
                    nrm = scale3(nrm, -1.0);
                }
                scene_3d.faces.push(Face3D {
                    verts: [a, b, c],
                    normal: scale3(nrm, -1.0),
                    color: RgbaColor(56, 189, 248, 85),
                });
                let dist = dot3(sub3(p3, a), nrm);
                let foot = sub3(p3, scale3(nrm, dist));
                scene_3d.edges.push(Edge3D {
                    a: p3,
                    b: foot,
                    color: RgbaColor::GOLD,
                    width: 2.6,
                });
                scene_3d.nodes.push(Node3D {
                    pos: foot,
                    radius: 4.0,
                    color: RgbaColor::EMERALD,
                    label: None,
                });
            }
            for i in 0..4 {
                for j in (i + 1)..4 {
                    scene_3d.edges.push(Edge3D {
                        a: tet[i],
                        b: tet[j],
                        color: RgbaColor::CYAN,
                        width: 1.8,
                    });
                }
            }
            scene_3d.nodes.push(Node3D {
                pos: p3,
                radius: 5.5,
                color: RgbaColor::CORAL,
                label: Some("Interior P".to_string()),
            });
            scene_3d.title = format!("3D Regular Tetrahedron Viviani Theorem (Sum d_i == H = {h3:.4})");

            // 2D Equilateral Triangle Viviani Plot
            let tri = [(0.0, 0.0), (1.0, 0.0), (0.5, 0.5 * 3.0_f64.sqrt())];
            let s2 = u + v + w;
            let p2 = (
                (u * tri[0].0 + v * tri[1].0 + w * tri[2].0) / s2,
                (u * tri[0].1 + v * tri[1].1 + w * tri[2].1) / s2,
            );
            scene_2d.polylines.push(Polyline2D {
                points: tri.to_vec(),
                color: RgbaColor::CYAN,
                width: 2.4,
                label: Some("Equilateral Triangle".to_string()),
                closed: true,
            });
            let ray_cols = [RgbaColor::GOLD, RgbaColor::EMERALD, RgbaColor::CORAL];
            for i in 0..3 {
                let a = tri[i];
                let b = tri[(i + 1) % 3];
                let ex = b.0 - a.0;
                let ey = b.1 - a.1;
                let t = ((p2.0 - a.0) * ex + (p2.1 - a.1) * ey) / (ex * ex + ey * ey);
                let foot = (a.0 + t * ex, a.1 + t * ey);
                scene_2d.polylines.push(Polyline2D {
                    points: vec![p2, foot],
                    color: ray_cols[i],
                    width: 2.2,
                    label: None,
                    closed: false,
                });
            }
            scene_2d.points.push(Point2D {
                x: p2.0,
                y: p2.1,
                color: RgbaColor::GOLD,
                radius: 4.8,
                label: Some("P (d1+d2+d3 = H)".to_string()),
            });
            scene_2d.title = format!("2D Viviani Theorem: Sum Perpendiculars = sqrt(3)/2 (err={err_2d:.2e})");
            scene_2d.x_label = "Volume / Area conservation via barycentric split".to_string();

            headline = format!(
                "Viviani Simplex Conservation: 2D err = {err_2d:.2e}, 3D Tetrahedron err = {err_3d:.2e} (H={h3:.6})"
            );
            metrics = vec![
                MetricCard {
                    label: "2D Viviani Residual".to_string(),
                    value: format!("{err_2d:.2e}"),
                    detail: "Sum of 3 perpendiculars vs sqrt(3)/2".to_string(),
                },
                MetricCard {
                    label: "3D Simplex Residual".to_string(),
                    value: format!("{err_3d:.2e}"),
                    detail: format!("Sum of 4 face distances vs {h3:.6}"),
                },
                MetricCard {
                    label: "Tetrahedron Altitude".to_string(),
                    value: format!("{h3:.6}"),
                    detail: "sqrt(2/3) for unit-edge regular tetrahedron".to_string(),
                },
                MetricCard {
                    label: "Proof Mechanism".to_string(),
                    value: "Volume Decomposition".to_string(),
                    detail: "4 sub-pyramids sum to constant V = (1/3)*A_base*H".to_string(),
                },
            ];
            logs.push(format!("Tested Viviani conservation on interior point weighted ({u:.3}, {v:.3}, {w:.3})."));
            logs.push(format!("2D altitude residual = {err_2d:.3e}; 3D tetrahedral altitude residual = {err_3d:.3e}."));
        }
        _ => {
            // Pappus's Centroid Theorem & Triangulated 3D Torus
            let r_maj = 1.65_f64;
            let r_min = (params.secondary_param * 0.45).clamp(0.22, 0.72);
            let nu = (params.span as usize).clamp(24, 64);
            let nv = (params.span as usize / 2).clamp(16, 40);
            let (v_exact, v_mesh, a_exact, rel_err) = pappus_torus_verify(r_maj, r_min, nu, nv);

            let vertex = |i: usize, j: usize| -> [f64; 3] {
                let u = 2.0 * PI * ((i % nu) as f64) / (nu as f64);
                let v = 2.0 * PI * ((j % nv) as f64) / (nv as f64);
                let rad = r_maj + r_min * v.cos();
                [rad * u.cos(), r_min * v.sin(), rad * u.sin()]
            };
            for i in 0..nu {
                let u_frac = (i as f64) / (nu as f64);
                let col = RgbaColor(
                    (56.0 + 180.0 * (2.0 * PI * u_frac).cos().abs()) as u8,
                    (170.0 + 50.0 * (2.0 * PI * u_frac).sin().abs()) as u8,
                    245,
                    195,
                );
                for j in 0..nv {
                    let p00 = vertex(i, j);
                    let p10 = vertex(i + 1, j);
                    let p11 = vertex(i + 1, j + 1);
                    let p01 = vertex(i, j + 1);
                    scene_3d.faces.push(Face3D {
                        verts: [p00, p10, p11],
                        normal: cross_normal(p00, p10, p11),
                        color: col,
                    });
                    scene_3d.faces.push(Face3D {
                        verts: [p00, p11, p01],
                        normal: cross_normal(p00, p11, p01),
                        color: col,
                    });
                }
                // Centroid orbit path 2*pi*R
                let u0 = 2.0 * PI * (i as f64) / (nu as f64);
                let u1 = 2.0 * PI * ((i + 1) as f64) / (nu as f64);
                scene_3d.edges.push(Edge3D {
                    a: [r_maj * u0.cos(), 0.0, r_maj * u0.sin()],
                    b: [r_maj * u1.cos(), 0.0, r_maj * u1.sin()],
                    color: RgbaColor::GOLD,
                    width: 2.4,
                });
            }
            scene_3d.title = format!(
                "3D Pappus Torus (R={r_maj:.2}, r={r_min:.2}): V = 2*pi^2*R*r^2 = {v_exact:.4}"
            );

            // 2D Disk cross-section & mesh convergence vs segment count
            let mut disk = Vec::new();
            for s in 0..=48 {
                let th = 2.0 * PI * (s as f64) / 48.0;
                disk.push((r_maj + r_min * th.cos(), r_min * th.sin()));
            }
            scene_2d.polylines.push(Polyline2D {
                points: disk,
                color: RgbaColor::CYAN,
                width: 2.2,
                label: Some(format!("Generating Disk r={r_min:.2}")),
                closed: true,
            });
            scene_2d.polylines.push(Polyline2D {
                points: vec![(0.0, -r_min * 1.3), (0.0, r_min * 1.3)],
                color: RgbaColor::GOLD,
                width: 2.2,
                label: Some("Axis of Revolution".to_string()),
                closed: false,
            });
            scene_2d.points.push(Point2D {
                x: r_maj,
                y: 0.0,
                color: RgbaColor::CORAL,
                radius: 5.0,
                label: Some(format!("Centroid R={r_maj:.2}")),
            });
            scene_2d.title = format!("2D Pappus Generating Cross-Section (A=pi r^2={:.3}, Path=2pi R={:.3})", PI * r_min * r_min, 2.0 * PI * r_maj);
            scene_2d.x_label = "Radial distance from revolution axis".to_string();

            headline = format!(
                "Pappus Torus (R={r_maj:.2}, r={r_min:.2}): Exact V={v_exact:.5} vs Mesh V={v_mesh:.5} (rel err={rel_err:.3e})"
            );
            metrics = vec![
                MetricCard {
                    label: "Exact Pappus Vol".to_string(),
                    value: format!("{v_exact:.6}"),
                    detail: "2 * pi^2 * R * r^2".to_string(),
                },
                MetricCard {
                    label: "Discrete Mesh Vol".to_string(),
                    value: format!("{v_mesh:.6}"),
                    detail: format!("{nu}x{nv} quad grid ({} tris)", 2 * nu * nv),
                },
                MetricCard {
                    label: "Relative Vol Err".to_string(),
                    value: format!("{rel_err:.3e}"),
                    detail: "Discrete divergence theorem integral".to_string(),
                },
                MetricCard {
                    label: "Surface Area".to_string(),
                    value: format!("{a_exact:.6}"),
                    detail: "4 * pi^2 * R * r".to_string(),
                },
            ];
            logs.push(format!("Swept {nu}x{nv} torus mesh: signed volume = {v_mesh:.6} vs Pappus exact = {v_exact:.6}."));
            logs.push(format!("Pappus area theorem: A = (2*pi*r)*(2*pi*R) = {a_exact:.6}."));
        }
    }

    pad_aspect_2d(&mut scene_2d, 1.65);

    ComputationReport {
        module: ModuleId::P3SolidCavalieriPolyhedra,
        threads_used: params.threads,
        elapsed_ms: 0.0,
        headline,
        metrics,
        log_lines: logs,
        prefer_3d: true,
        scene_2d,
        scene_3d,
    }
}
