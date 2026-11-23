// Copyright (C) 2026 CRV Science Network (https://crvscience.com)
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://gnu.org/licenses/>.

#![forbid(unsafe_code)]

//! Procedural 64x64 RGBA window/taskbar icon for Principia Studio
//! featuring a dark-slate rounded badge with a glowing cyan/gold trefoil
//! knot & compass-orbit emblem.

/// Returns the 64x64 RGBA window/taskbar icon for Principia Studio.
pub fn app_egui_icon() -> egui::IconData {
    let size = 64u32;
    egui::IconData {
        rgba: render_principia_icon_rgba(size),
        width: size,
        height: size,
    }
}

fn sd_rounded_box(px: f32, py: f32, bx: f32, by: f32, r: f32) -> f32 {
    let qx = px.abs() - bx + r;
    let qy = py.abs() - by + r;
    let ox = qx.max(0.0);
    let oy = qy.max(0.0);
    (ox * ox + oy * oy).sqrt() + qx.max(qy).min(0.0) - r
}

fn blend_rgba(dst: &mut [u8], src: [f32; 3], alpha: f32) {
    let a = alpha.clamp(0.0, 1.0);
    if a <= 0.001 {
        return;
    }
    let inv = 1.0 - a;
    dst[0] = ((dst[0] as f32) * inv + src[0] * a).round().clamp(0.0, 255.0) as u8;
    dst[1] = ((dst[1] as f32) * inv + src[1] * a).round().clamp(0.0, 255.0) as u8;
    dst[2] = ((dst[2] as f32) * inv + src[2] * a).round().clamp(0.0, 255.0) as u8;
}

/// Renders the Principia Studio geometric badge at arbitrary square resolution.
pub fn render_principia_icon_rgba(size: u32) -> Vec<u8> {
    let n = size as usize;
    let mut out = vec![0u8; n * n * 4];
    let trefoil_samples = 160usize;
    let mut knot_pts = Vec::with_capacity(trefoil_samples);
    for s in 0..trefoil_samples {
        let t = (s as f32) * std::f32::consts::TAU / (trefoil_samples as f32);
        let kx = (t.sin() + 2.0 * (2.0 * t).sin()) * 0.165;
        let ky = -(t.cos() - 2.0 * (2.0 * t).cos()) * 0.165;
        let kz = -(3.0 * t).sin();
        knot_pts.push((kx, ky, kz));
    }

    for py_i in 0..n {
        for px_i in 0..n {
            let nx = ((px_i as f32) + 0.5) / (n as f32) * 2.0 - 1.0;
            let ny = ((py_i as f32) + 0.5) / (n as f32) * 2.0 - 1.0;

            let sd_badge = sd_rounded_box(nx, ny, 0.90, 0.90, 0.24);
            let aa = (1.5 / (n as f32)).max(0.025);
            let badge_alpha = ((-sd_badge / aa) + 0.5).clamp(0.0, 1.0);
            if badge_alpha <= 0.001 {
                continue;
            }

            let off = (py_i * n + px_i) * 4;
            let radial = (nx * nx + ny * ny).sqrt();
            let mut rgb = [
                10.0 + (1.0 - radial.min(1.0)) * 12.0,
                16.0 + (1.0 - radial.min(1.0)) * 20.0,
                34.0 + (1.0 - radial.min(1.0)) * 30.0,
            ];

            // Subtle slate rim around rounded badge
            if sd_badge.abs() < 0.045 {
                let rim_w = 1.0 - (sd_badge.abs() / 0.045);
                rgb[0] += 45.0 * rim_w;
                rgb[1] += 70.0 * rim_w;
                rgb[2] += 110.0 * rim_w;
            }

            out[off] = rgb[0].round().clamp(0.0, 255.0) as u8;
            out[off + 1] = rgb[1].round().clamp(0.0, 255.0) as u8;
            out[off + 2] = rgb[2].round().clamp(0.0, 255.0) as u8;
            out[off + 3] = (badge_alpha * 255.0).round().clamp(0.0, 255.0) as u8;

            // Outer orbital compass ring
            let orbit_d = (radial - 0.67).abs();
            if orbit_d < 0.036 {
                let glow = ((0.036 - orbit_d) / 0.036).powf(0.8);
                blend_rgba(&mut out[off..off + 4], [56.0, 189.0, 248.0], glow * 0.55);
            }

            // Gold compass ticks along cardinal directions
            let on_axis = (nx.abs() < 0.026 && radial > 0.58 && radial < 0.77)
                || (ny.abs() < 0.026 && radial > 0.58 && radial < 0.77);
            if on_axis {
                blend_rgba(&mut out[off..off + 4], [250.0, 204.0, 21.0], 0.85);
            }

            // Projected trefoil ribbon distance check with depth shading
            let mut best_d = f32::INFINITY;
            let mut best_z = 0.0f32;
            for &(kx, ky, kz) in &knot_pts {
                let dx = nx - kx;
                let dy = ny - ky;
                let d2 = dx * dx + dy * dy;
                if d2 < best_d {
                    best_d = d2;
                    best_z = kz;
                }
            }
            let d = best_d.sqrt();
            if d < 0.13 {
                let halo = ((0.13 - d) / 0.13).powi(2) * 0.42;
                blend_rgba(&mut out[off..off + 4], [56.0, 189.0, 248.0], halo);
            }
            if d < 0.072 {
                let core = ((0.072 - d) / 0.072).powf(0.65);
                let z_t = ((best_z + 1.0) * 0.5).clamp(0.0, 1.0);
                let col = [
                    56.0 + z_t * 194.0,  // cyan (56) -> gold (250)
                    189.0 + z_t * 25.0,  // 189 -> 214
                    248.0 - z_t * 195.0, // 248 -> 53
                ];
                blend_rgba(&mut out[off..off + 4], col, core * 0.96);
            }

            // Central lattice origin dot
            if radial < 0.068 {
                let center_w = ((0.068 - radial) / 0.068).powf(0.5);
                blend_rgba(&mut out[off..off + 4], [248.0, 250.0, 252.0], center_w * 0.95);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{app_egui_icon, render_principia_icon_rgba};

    #[test]
    fn icon_renders_valid_64x64_rgba() {
        let icon = app_egui_icon();
        assert_eq!(icon.width, 64);
        assert_eq!(icon.height, 64);
        assert_eq!(icon.rgba.len(), 64 * 64 * 4);
        // Center pixel alpha should be opaque
        let center = (32 * 64 + 32) * 4;
        assert_eq!(icon.rgba[center + 3], 255);
        // Outer corner outside rounded badge should be transparent
        assert_eq!(icon.rgba[3], 0);
        // Multi-size helper smoke check
        assert_eq!(render_principia_icon_rgba(32).len(), 32 * 32 * 4);
    }
}
