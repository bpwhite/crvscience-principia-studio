#![forbid(unsafe_code)]

//! Safe-Rust 3D Perspective Camera, Depth-Sorted Painter Projection, and SVG / Wavefront OBJ Exporters.

use crate::{ComputationReport, RgbaColor, Scene2D, Scene3D};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Camera3D {
    pub yaw: f64,
    pub pitch: f64,
    pub zoom: f64,
    pub pan_x: f64,
    pub pan_y: f64,
    pub target: [f64; 3],
}

impl Default for Camera3D {
    fn default() -> Self {
        Self {
            yaw: 0.62,
            pitch: 0.42,
            zoom: 1.0,
            pan_x: 0.0,
            pan_y: 0.0,
            target: [0.0, 0.0, 0.0],
        }
    }
}

impl Camera3D {
    pub fn project(&self, p: [f64; 3], width: f64, height: f64, scale: f64) -> Option<(f64, f64, f64)> {
        let dx = p[0] - self.target[0];
        let dy = p[1] - self.target[1];
        let dz = p[2] - self.target[2];

        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();

        let x1 = dx * cy + dz * sy;
        let z1 = -dx * sy + dz * cy;
        let y2 = dy * cp - z1 * sp;
        let z2 = dy * sp + z1 * cp;

        let cam_dist = 5.5;
        let depth = cam_dist + z2 * scale;
        if depth <= 0.25 {
            return None;
        }
        let focal = width.min(height) * 0.78 * self.zoom;
        let sx = width * 0.5 + self.pan_x + (x1 * scale * focal) / depth;
        let sy_screen = height * 0.5 + self.pan_y - (y2 * scale * focal) / depth;
        Some((sx, sy_screen, depth))
    }
}

pub fn scene3d_normalization_scale(scene: &Scene3D) -> ([f64; 3], f64) {
    let mut min_v = [f64::INFINITY; 3];
    let mut max_v = [f64::NEG_INFINITY; 3];
    let mut count = 0usize;

    let mut push = |p: [f64; 3]| {
        for i in 0..3 {
            if p[i].is_finite() {
                min_v[i] = min_v[i].min(p[i]);
                max_v[i] = max_v[i].max(p[i]);
            }
        }
        count += 1;
    };

    for n in &scene.nodes {
        push(n.pos);
    }
    for e in &scene.edges {
        push(e.a);
        push(e.b);
    }
    for f in &scene.faces {
        for v in f.verts {
            push(v);
        }
    }
    for enc in &scene.enclosures {
        push([
            enc.center[0] - enc.half_extents[0],
            enc.center[1] - enc.half_extents[1],
            enc.center[2] - enc.half_extents[2],
        ]);
        push([
            enc.center[0] + enc.half_extents[0],
            enc.center[1] + enc.half_extents[1],
            enc.center[2] + enc.half_extents[2],
        ]);
    }

    if count == 0 {
        return ([0.0, 0.0, 0.0], 1.0);
    }
    let center = [
        0.5 * (min_v[0] + max_v[0]),
        0.5 * (min_v[1] + max_v[1]),
        0.5 * (min_v[2] + max_v[2]),
    ];
    let span = (max_v[0] - min_v[0])
        .max(max_v[1] - min_v[1])
        .max(max_v[2] - min_v[2])
        .max(1e-6);
    (center, 2.4 / span)
}

pub fn shade_color(base: RgbaColor, normal: [f64; 3]) -> RgbaColor {
    let light = [-0.45, 0.72, -0.52];
    let n_len = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2])
        .sqrt()
        .max(1e-9);
    let dot = ((normal[0] * light[0] + normal[1] * light[1] + normal[2] * light[2]) / n_len).abs();
    let factor = (0.48 + 0.52 * dot).clamp(0.35, 1.12);
    RgbaColor(
        ((base.0 as f64) * factor).min(255.0) as u8,
        ((base.1 as f64) * factor).min(255.0) as u8,
        ((base.2 as f64) * factor).min(255.0) as u8,
        base.3,
    )
}

pub fn cross_normal(a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> [f64; 3] {
    let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let nx = u[1] * v[2] - u[2] * v[1];
    let ny = u[2] * v[0] - u[0] * v[2];
    let nz = u[0] * v[1] - u[1] * v[0];
    let len = (nx * nx + ny * ny + nz * nz).sqrt().max(1e-12);
    [nx / len, ny / len, nz / len]
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Exports either the primary 3D or 2D visualization of `report` to a standalone vector SVG.
pub fn export_report_svg(report: &ComputationReport, width: usize, height: usize) -> String {
    if report.prefer_3d
        && (!report.scene_3d.nodes.is_empty()
            || !report.scene_3d.edges.is_empty()
            || !report.scene_3d.faces.is_empty())
    {
        export_scene3d_svg(&report.scene_3d, &report.headline, width, height)
    } else {
        export_scene2d_svg(&report.scene_2d, &report.headline, width, height)
    }
}

pub fn export_scene2d_svg(scene: &Scene2D, subtitle: &str, width: usize, height: usize) -> String {
    let w = width as f64;
    let h = height as f64;
    let pad = 64.0;

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
    for line in &scene.polylines {
        for &(x, y) in &line.points {
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

    let map_x = |x: f64| pad + ((x - min_x) / dx) * (w - 2.0 * pad);
    let map_y = |y: f64| h - pad - ((y - min_y) / dy) * (h - 2.0 * pad);

    let mut out = String::with_capacity(16_384);
    out.push_str(&format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} {height}" width="{width}" height="{height}">"#
    ));
    out.push_str(&format!(
        r##"<rect width="{width}" height="{height}" fill="#0b1120"/>"##
    ));
    out.push_str(&format!(
        r##"<text x="{pad}" y="28" fill="#f8fafc" font-family="monospace" font-size="16" font-weight="bold">{}</text>"##,
        xml_escape(&scene.title)
    ));
    out.push_str(&format!(
        r##"<text x="{pad}" y="46" fill="#94a3b8" font-family="monospace" font-size="12">{}</text>"##,
        xml_escape(subtitle)
    ));

    for r in &scene.rects {
        let x0 = map_x(r.x0.min(r.x1));
        let x1 = map_x(r.x0.max(r.x1));
        let y0 = map_y(r.y0.max(r.y1));
        let y1 = map_y(r.y0.min(r.y1));
        let fill_op = (r.fill.3 as f64) / 255.0;
        out.push_str(&format!(
            r#"<rect x="{:.2}" y="{:.2}" width="{:.2}" height="{:.2}" fill="{}" fill-opacity="{:.2}" stroke="{}" stroke-width="1"/>"#,
            x0,
            y0,
            (x1 - x0).max(1.0),
            (y1 - y0).max(1.0),
            r.fill.to_hex(),
            fill_op,
            r.stroke.to_hex()
        ));
        if let Some(lbl) = &r.label {
            out.push_str(&format!(
                r##"<text x="{:.2}" y="{:.2}" fill="#e2e8f0" font-family="monospace" font-size="10">{}</text>"##,
                x0 + 4.0,
                y0 + 13.0,
                xml_escape(lbl)
            ));
        }
    }

    for line in &scene.polylines {
        if line.points.len() < 2 {
            continue;
        }
        let mut pts = String::new();
        for &(x, y) in &line.points {
            pts.push_str(&format!("{:.2},{:.2} ", map_x(x), map_y(y)));
        }
        if line.closed {
            let (fx, fy) = line.points[0];
            pts.push_str(&format!("{:.2},{:.2}", map_x(fx), map_y(fy)));
        }
        out.push_str(&format!(
            r#"<polyline fill="none" stroke="{}" stroke-width="{:.2}" stroke-opacity="{:.2}" points="{}"/>"#,
            line.color.to_hex(),
            line.width,
            (line.color.3 as f64) / 255.0,
            pts.trim()
        ));
    }

    for p in &scene.points {
        let cx = map_x(p.x);
        let cy = map_y(p.y);
        out.push_str(&format!(
            r#"<circle cx="{:.2}" cy="{:.2}" r="{:.2}" fill="{}" fill-opacity="{:.2}"/>"#,
            cx,
            cy,
            p.radius.max(1.2),
            p.color.to_hex(),
            (p.color.3 as f64) / 255.0
        ));
        if let Some(lbl) = &p.label {
            out.push_str(&format!(
                r##"<text x="{:.2}" y="{:.2}" fill="#cbd5e1" font-family="monospace" font-size="10">{}</text>"##,
                cx + 5.0,
                cy - 4.0,
                xml_escape(lbl)
            ));
        }
    }

    out.push_str("</svg>");
    out
}

pub fn export_scene3d_svg(scene: &Scene3D, subtitle: &str, width: usize, height: usize) -> String {
    let w = width as f64;
    let h = height as f64;
    let (center, scale) = scene3d_normalization_scale(scene);
    let mut cam = Camera3D::default();
    cam.target = center;

    enum DrawPrim {
        Face {
            depth: f64,
            pts: [(f64, f64); 3],
            color: RgbaColor,
        },
        Edge {
            depth: f64,
            a: (f64, f64),
            b: (f64, f64),
            width: f32,
            color: RgbaColor,
        },
        Node {
            depth: f64,
            p: (f64, f64),
            r: f32,
            color: RgbaColor,
            label: Option<String>,
        },
    }

    let mut prims = Vec::new();
    for f in &scene.faces {
        if let (Some(p0), Some(p1), Some(p2)) = (
            cam.project(f.verts[0], w, h, scale),
            cam.project(f.verts[1], w, h, scale),
            cam.project(f.verts[2], w, h, scale),
        ) {
            let depth = (p0.2 + p1.2 + p2.2) / 3.0;
            prims.push(DrawPrim::Face {
                depth,
                pts: [(p0.0, p0.1), (p1.0, p1.1), (p2.0, p2.1)],
                color: shade_color(f.color, f.normal),
            });
        }
    }
    for e in &scene.edges {
        if let (Some(p0), Some(p1)) = (
            cam.project(e.a, w, h, scale),
            cam.project(e.b, w, h, scale),
        ) {
            prims.push(DrawPrim::Edge {
                depth: 0.5 * (p0.2 + p1.2),
                a: (p0.0, p0.1),
                b: (p1.0, p1.1),
                width: e.width,
                color: e.color,
            });
        }
    }
    for n in &scene.nodes {
        if let Some(p) = cam.project(n.pos, w, h, scale) {
            let persp_r = (n.radius * 5.0 / (p.2 as f32)).clamp(1.5, 16.0);
            prims.push(DrawPrim::Node {
                depth: p.2,
                p: (p.0, p.1),
                r: persp_r,
                color: n.color,
                label: n.label.clone(),
            });
        }
    }

    prims.sort_by(|a, b| {
        let da = match a {
            DrawPrim::Face { depth, .. }
            | DrawPrim::Edge { depth, .. }
            | DrawPrim::Node { depth, .. } => *depth,
        };
        let db = match b {
            DrawPrim::Face { depth, .. }
            | DrawPrim::Edge { depth, .. }
            | DrawPrim::Node { depth, .. } => *depth,
        };
        db.partial_cmp(&da).unwrap_or(Ordering::Equal)
    });

    let mut out = String::with_capacity(24_576);
    out.push_str(&format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} {height}" width="{width}" height="{height}">"#
    ));
    out.push_str(&format!(
        r##"<rect width="{width}" height="{height}" fill="#090f1d"/>"##
    ));
    out.push_str(&format!(
        r##"<text x="32" y="28" fill="#f8fafc" font-family="monospace" font-size="16" font-weight="bold">{}</text>"##,
        xml_escape(&scene.title)
    ));
    out.push_str(&format!(
        r##"<text x="32" y="46" fill="#94a3b8" font-family="monospace" font-size="12">{}</text>"##,
        xml_escape(subtitle)
    ));

    for p in prims {
        match p {
            DrawPrim::Face { pts, color, .. } => {
                out.push_str(&format!(
                    r#"<polygon points="{:.2},{:.2} {:.2},{:.2} {:.2},{:.2}" fill="{}" fill-opacity="{:.2}" stroke="{}" stroke-width="0.4"/>"#,
                    pts[0].0,
                    pts[0].1,
                    pts[1].0,
                    pts[1].1,
                    pts[2].0,
                    pts[2].1,
                    color.to_hex(),
                    (color.3 as f64) / 255.0,
                    color.to_hex()
                ));
            }
            DrawPrim::Edge {
                a, b, width, color, ..
            } => {
                out.push_str(&format!(
                    r#"<line x1="{:.2}" y1="{:.2}" x2="{:.2}" y2="{:.2}" stroke="{}" stroke-width="{:.2}" stroke-opacity="{:.2}"/>"#,
                    a.0,
                    a.1,
                    b.0,
                    b.1,
                    color.to_hex(),
                    width,
                    (color.3 as f64) / 255.0
                ));
            }
            DrawPrim::Node {
                p, r, color, label, ..
            } => {
                out.push_str(&format!(
                    r#"<circle cx="{:.2}" cy="{:.2}" r="{:.2}" fill="{}" fill-opacity="{:.2}"/>"#,
                    p.0,
                    p.1,
                    r,
                    color.to_hex(),
                    (color.3 as f64) / 255.0
                ));
                if let Some(lbl) = label {
                    out.push_str(&format!(
                        r##"<text x="{:.2}" y="{:.2}" fill="#e2e8f0" font-family="monospace" font-size="10">{}</text>"##,
                        p.0 + (r as f64) + 4.0,
                        p.1 + 3.0,
                        xml_escape(&lbl)
                    ));
                }
            }
        }
    }
    out.push_str("</svg>");
    out
}

/// Exports 3D geometry (`Scene3D` faces, edges, and nodes) to a standard Wavefront `.obj` mesh.
pub fn export_scene3d_obj(scene: &Scene3D) -> String {
    let mut out = String::with_capacity(16_384);
    out.push_str(&format!("# Principia Studio 3D Export: {}\n", scene.title));
    let mut v_idx = 1usize;

    for f in &scene.faces {
        for v in f.verts {
            out.push_str(&format!("v {:.6} {:.6} {:.6}\n", v[0], v[1], v[2]));
        }
        out.push_str(&format!(
            "vn {:.6} {:.6} {:.6}\n",
            f.normal[0], f.normal[1], f.normal[2]
        ));
        out.push_str(&format!("f {} {} {}\n", v_idx, v_idx + 1, v_idx + 2));
        v_idx += 3;
    }

    for e in &scene.edges {
        out.push_str(&format!("v {:.6} {:.6} {:.6}\n", e.a[0], e.a[1], e.a[2]));
        out.push_str(&format!("v {:.6} {:.6} {:.6}\n", e.b[0], e.b[1], e.b[2]));
        out.push_str(&format!("l {} {}\n", v_idx, v_idx + 1));
        v_idx += 2;
    }

    for n in &scene.nodes {
        out.push_str(&format!(
            "v {:.6} {:.6} {:.6}\n",
            n.pos[0], n.pos[1], n.pos[2]
        ));
        out.push_str(&format!("p {}\n", v_idx));
        v_idx += 1;
    }

    out
}
