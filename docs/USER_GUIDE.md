# Principia Studio — User Guide (`v1.2.0`)

**Principia Studio** is a safe-Rust (`#![forbid(unsafe_code)]`) multithreaded computational mathematics workstation with an `eframe`/`egui` (`0.31`) 2D/3D desktop GUI, bundled `rusqlite` WAL persistence, SVG/Wavefront `.obj`/JSON exporters, and a deterministic headless CLI (`clap` `4.5`).

---

## 1. Building & Launching

```bash
# Run unit tests, integration suite, and headless mathematical validation
cargo test --release

# Launch interactive desktop GUI (opens 7-Domain Launcher Hub)
cargo run --release

# Bypass Launcher Hub and open a specific module directly
cargo run --release -- --skip-hub
cargo run --release -- gui --module g3 --threads 8 --skip-hub
cargo run --release -- gui --module p2 --threads 8 --skip-hub
```

When compiled on Windows, `build.rs` embeds [`assets/principia_studio.ico`](../assets/principia_studio.ico) and `VERSIONINFO` metadata into `principia-studio.exe`, and `src/icon.rs` supplies the 64x64 RGBA window icon.

---

## 2. Desktop Menu System & Interactive Controls

### Persistent Top Menu Bar
- **`File`**: Toggle the 7-Domain Launcher Hub (`H`), reset current module defaults, save/load SQLite WAL Workset Slots `1..4` (`F5` / `F9`), export standalone `.svg` / Wavefront `.obj` / `.json` / `.pworkset.json` artifacts, or quit.
- **`Domains`**: Cascading selector across all 7 mathematical domains and 22 modules (`[2D]` / `[3D]` tagged).
- **`View`**: Toggle between `2D Plot` (`2`) and `3D Viewport` (`3`), toggle `Zen Full-Canvas Mode` (`Z`), show/hide Navigator and Inspector side docks, toggle `Calm Auto-Orbit` (`O`), or `Reset 3D Camera` (`R`).
- **`Compute`**: Re-run multithreaded Rayon computation (`Space`), open the fuzzy `Ctrl+K` Command Palette, configure worker threads (`1..=32` in GUI slider; `1..=128` in CLI), or launch the `Multithreaded Scaling Benchmark` modal (`B`).
- **`Help`**: Open the `F1` Keyboard Shortcuts & CLI Quick Reference modal or About Principia Studio dialog.

### Keyboard & Canvas Shortcuts

All single-key shortcuts check `!ctx.wants_keyboard_input()`, so typing in expression or parameter inputs never triggers canvas actions.

| Shortcut | Action |
|---|---|
| `Ctrl+K` / `Cmd+K` | Open fuzzy Command Palette across all 22 modules |
| `H` | Toggle 7-Domain Launcher Hub |
| `Z` | Toggle Zen Full-Canvas Mode (hide side docks) |
| `2` / `3` | Switch between 2D Scientific Plot & 3D Viewport |
| `O` / `R` | Toggle Calm 3D Auto-Orbit / Reset 3D Camera |
| `Space` | Re-dispatch multithreaded Rayon computation |
| `B` | Run 1..N thread scaling benchmark sweep |
| `F5` / `F9` | Quick-Save / Quick-Load SQLite WAL Workset Slot 1 |
| `F1` / `Esc` | Open Shortcuts Guide / Close active modal |
| Mouse Drag / Wheel | Orbit, pan, and zoom the 3D scene camera |

---

## 3. Domain 7: Geometric Proofs & Visualizations (`P1`–`P3`)

Domain 7 provides 12 interactive visual proof presets pairing direct 2D diagrams, 3D spatial constructions, and machine-precision invariant verification logs:

| Module | Presets (`--preset 0..3`) | Interactive Controls & Visual Output |
|---|---|---|
| **`P1` Area & Dissection Proofs** (`p1`) | `#0` Pythagorean Dissection (Bhāskara / Zhoubi Suanjing)<br>`#1` Euclid Elements I.47 Windmill & Altitude Shear<br>`#2` Geometric Series Unit-Square Tiling (`sum 2^-k`)<br>`#3` Nicomachus Sum of Cubes (`sum k^3 = T_n^2`) | Adjust triangle legs (`Primary N`, `Span / Steps`) or series/gnomon depth (`Primary N`) alongside `Scale / Eps`; view 2D exact dissection tilings or toggle `3` for extruded 3D proof prisms. |
| **`P2` Circle, Conic & Hyperbolic** (`p2`) | `#0` Archimedes `pi` Exhaustion (`m = 6*2^k`-gons)<br>`#1` Thales Right Angle & Ptolemy Cyclic Quadrilateral<br>`#2` 3D Dandelin Spheres & Conic Focal Property<br>`#3` Poincaré Disk Hyperbolic Triangle & Gauss-Bonnet Defect | Adjust polygon doublings (`Primary N`), sweep semicircle point or Dandelin tilt / hyperbolic vertex radius via `Scale / Eps`, and inspect 3D Dandelin tangent spheres and hyperboloid sheets. |
| **`P3` Cavalieri & 3D Polyhedra** (`p3`) | `#0` Archimedes `1:2:3` Sphere–Cylinder–Cone Slices<br>`#1` Platonic Solids Euler (`V-E+F=2`) & Descartes `4pi` Defect<br>`#2` Viviani 2D Triangle & 3D Regular Tetrahedron<br>`#3` Pappus Centroid Theorem & Torus Mesh | Sweep horizontal Cavalieri slice plane `z` (`Scale / Eps`), cycle all 5 Platonic solids (`Primary N mod 5`), move interior Viviani point `P`, or inspect 3D/2D torus convergence and cross-sections. |

---

## 4. Headless CLI Reference

Inspect modules, run parameterized computations headlessly, export publication geometry/data, or benchmark Rayon scaling directly from the terminal:

```bash
# List all 7 mathematical domains and 22 module codes (n1..p3)
principia-studio cli list

# Computational Number Theory: 10^12 segmented prime sieve window + SVG & JSON export
principia-studio cli run --module n1 --threads 8 --n 1000000000000 --span 200000 --export-svg n1_ulam.svg --export-json n1_report.json

# Formal Verification: portfolio CDCL SAT/SMT & theorem prover
principia-studio cli run --module f1 --expr "((P -> Q) -> P) -> P"
principia-studio cli run --module f2 --n 60 --span 6 --threads 4

# Symbolic Computation & CAS: differentiation, Risch integration, and Groebner bases
principia-studio cli run --module s1 --expr "x^2 * exp(x) + sin(x) * cos(x)" --export-obj s1_surface.obj
principia-studio cli run --module s2 --expr "x^2 + y^2 + z^2 - 4; x*y - 1; z - x^2"

# Numerical PDEs, BigInt & Sparse Linear Algebra
principia-studio cli run --module d2 --n 500 --threads 8
principia-studio cli run --module d3 --n 64 --threads 4

# Combinatorics & Computational Topology (3D Knot Wavefront OBJ + SVG export)
principia-studio cli run --module c1 --n 26 --threads 8
principia-studio cli run --module g3 --preset 0 --threads 4 --export-obj trefoil_3_1.obj --export-svg trefoil_3_1.svg

# Geometric Proofs & Visualizations (Pythagorean, Dandelin Spheres, Cavalieri 1:2:3)
principia-studio cli run --module p1 --preset 0 --export-svg pythagoras_dissection.svg
principia-studio cli run --module p2 --preset 2 --export-obj dandelin_cone.obj --export-svg dandelin_cone.svg
principia-studio cli run --module p3 --preset 0 --export-obj archimedes_cavalieri.obj --export-svg archimedes_cavalieri.svg

# Multithreaded scaling sweep (1..max_threads)
principia-studio cli bench --module n4 --max-threads 8
```

---

## 5. Worksets, Validation & Outputs

- **SQLite WAL Vault**: Workset slots (`workset_slots` table in `rusqlite` WAL mode) persist `ExplorerParams`, `Camera3D`, schema version (`principia.workset.v1`), and a SHA-256 canonical digest (`sha256_hex`) verified automatically on reload.
- **Portable JSON Worksets**: Exported as `<slot>.pworkset.json` with identical SHA-256 checksum verification.
- **Headless Validation Artifact**: Running `cargo test --release` executes both `tests/integration_suite.rs` and `tests/headless_algorithm_validation.rs`, writing machine-readable 22-module timing and export telemetry to [`validation/principia_studio_validation_summary.json`](../validation/principia_studio_validation_summary.json).
