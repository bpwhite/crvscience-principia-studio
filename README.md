# Principia Studio (`principia-studio` v1.1.0)

Native safe-Rust (`#![forbid(unsafe_code)]`) multithreaded computational mathematics workstation with a full **Desktop GUI Menu System** (`File`, `Domains`, `View`, `Compute`, `Help`), interactive **6-Domain Launcher Hub** (`H`, bypassable via `--skip-hub`), **`Ctrl+K` Fuzzy Command Palette**, **`B` Multithreaded Scaling Benchmark Modal**, **`Z` Zen Full-Canvas Mode**, 2D/3D `eframe` (`egui 0.31`) rendering, headless `clap` (`4.5`) CLI, SVG / Wavefront `.obj` / JSON exporters, and bundled SQLite WAL 4-slot workset persistence (`F5` / `F9` + SHA-256 `.pworkset.json` envelopes).

- **Website**: [CRV Science Network](https://crvscience.com) · [CRV Science Public Apps](https://crvscience.com/apps)
- **User Guide**: [docs/USER_GUIDE.md](docs/USER_GUIDE.md)
- **Scientific Methods**: [docs/SCIENTIFIC_METHODS.md](docs/SCIENTIFIC_METHODS.md)
- **Validation Telemetry**: [validation/principia_studio_validation_summary.json](validation/principia_studio_validation_summary.json)
- **License**: GNU General Public License v3.0 or later (`GPL-3.0-or-later`, see [LICENSE](LICENSE))

---

## Quick Start

```bash
# 1. Run full unit, integration, and headless algorithm verification suites across all 6 domains / 19 modules
cargo test --release

# 2. Launch GUI into the 6-Domain Launcher Hub (or pass --skip-hub to open canvas directly)
cargo run --release
cargo run --release -- --skip-hub
cargo run --release -- gui --module g3 --threads 8 --skip-hub

# 3. Inspect all 19 CLI module codes and 2D/3D visualization modes
cargo run --release -- cli list

# 4. Run headless multithreaded modules and export 2D/3D visualizations
cargo run --release -- cli run --module n1 --threads 8 --n 1000000000000 --span 200000 --export-svg n1_ulam.svg
cargo run --release -- cli run --module g3 --threads 8 --preset 0 --export-obj trefoil_3_1.obj

# 5. Benchmark thread-pool scaling from 1..N cores
cargo run --release -- cli bench --module n4 --max-threads 16
```

---

## GUI Menu System & Hotkeys (`v1.1.0`)

- **Persistent Top Menu Bar**:
  - `File`: Toggle Domain Launcher Hub (`H`), Reset Module Defaults, Save/Load SQLite Workset Slots `1..4` (`F5`/`F9`), Export `.SVG` / `.OBJ` / `.JSON` / `.pworkset.json`, Quit.
  - `Domains`: Cascading submenus across all 6 mathematical domains and 19 modules (`[2D]` / `[3D]`).
  - `View`: Switch `3D Viewport` (`3`) / `2D Plot` (`2`), `Zen Full-Canvas Mode` (`Z`), show/hide Left Navigator & Right Inspector docks, `Calm Auto-Orbit` (`O`), `Reset 3D Camera` (`R`).
  - `Compute`: Re-Run Parallel Compute (`Space`), `Ctrl+K` Fuzzy Command Palette, Worker Thread Pool presets (`1..32`), `Multithreaded Scaling Benchmark` (`B`).
  - `Help`: `F1` Keyboard Shortcuts & CLI Quick Reference modal, Copy CLI Command, About Principia Studio `v1.1.0`.

---

## Interactive Controls & CLI Subcommands

| Shortcut | Action |
|---|---|
| `Ctrl+K` / `Cmd+K` | Fuzzy Command Palette across all 19 modules |
| `H` | Toggle 6-Domain Launcher Hub |
| `Z` | Toggle Zen Full-Canvas Mode |
| `2` / `3` | Switch between 2D Scientific Plot & 3D Viewport |
| `O` / `R` | Toggle Calm Auto-Orbit / Reset 3D Camera |
| `Space` | Re-run parallel Rayon compute |
| `B` | Run 1..N thread scaling benchmark modal |
| `F5` / `F9` | Quick-Save / Quick-Load SQLite WAL Workset Slot 1 |
| `F1` / `Esc` | Open Shortcuts Guide / Close active modal |

| CLI Command | Description |
|---|---|
| `cli list` | List all 6 domains and 19 modules (`n1`..`g3`) with 2D/3D mode indicators |
| `cli run --module <code>` | Execute module with `--threads`, `--preset`, `--n`, `--span`, `--param`, `--expr`, and `--export-svg`/`--export-obj`/`--export-json` |
| `cli bench --module <code>` | Measure wall-clock speedup from `1` up to `--max-threads` Rayon workers |
| `gui [--module <code>] [--skip-hub]` | Open desktop workstation directly or inside the 6-Domain Hub |

---

## 6 Domains & 19 Computational Modules

| Domain | Modules | Core Capabilities |
|---|---|---|
| **1. Computational Number Theory** (`src/number_theory.rs`) | `N1`–`N4` | 32 KiB segmented Eratosthenes & Atkin sieves up to `10^12`, Miller-Rabin, Baillie-PSW, AKS witnesses, Brent-Pollard rho, Quadratic Sieve GF(2), BSGS DLP, Collatz/Goldbach/Lucas-Lehmer/Carmichael scans |
| **2. Formal Verification & ATP** (`src/formal_verification.rs`) | `F1`–`F3` | Gentzen LK propositional sequent prover, multithreaded portfolio CDCL SAT solver (`K_5` SAT / `K_6` UNSAT Ramsey & Pythagorean triples), polynomial continued fraction hunter |
| **3. Symbolic Computation & CAS** (`src/symbolic_algebra.rs`) | `S1`–`S3` | Exact AST calculus & Liouvillian/Hermite symbolic integration, `Q[x,y,z]` Buchberger Gröbner bases (`Lex`/`GrLex`/`GRevLex`), bounded equality saturation |
| **4. Numerical Analysis & PDEs** (`src/numerical_pde.rs`) | `D1`–`D3` | Parallel 2D vorticity-streamfunction FDM, P1 FEM Poisson, RK45 Lorenz ensembles, Karatsuba + NTT `BigInt` & binary-splitting Chudnovsky `pi`, CSR Conjugate Gradient & one-sided Jacobi SVD |
| **5. Combinatorics, Graphs & Opt** (`src/combinatorics_opt.rs`) | `C1`–`C3` | Exact Branch-and-Bound vs. 2-Opt Island Annealing TSP, FFD bin packing, Primal Simplex, log-barrier central path, MILP, OEIS polyomino/Latin-square/non-isomorphic graph enumeration |
| **6. Computational Geometry & Topology** (`src/geometry_topology.rs`) | `G1`–`G3` | Bowyer-Watson 2D Delaunay/Voronoi, 3D Convex Hull (`V - E + F = 2`), Vietoris-Rips `Z_2` persistent homology (`beta_0, beta_1, beta_2`), parallel Kauffman bracket Jones polynomial `V_K(t)` & 3D knot meshes |

---

## Citation

If you use **Principia Studio** in academic or software work, please cite [CITATION.cff](CITATION.cff).
