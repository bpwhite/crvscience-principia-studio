# Principia Studio — Scientific Methods & Algorithmic Verification

This document summarizes the mathematical formulations, parallel execution models, and headless invariant checks across all **7 domains** and **22 modules** (`N1..P3`) of Principia Studio (`v1.2.0`). Every core kernel is implemented in safe Rust (`#![forbid(unsafe_code)]`) and parallelized across configurable `rayon::ThreadPool` workers (`execute_module`).

---

## Domain 1: Computational Number Theory (`src/number_theory.rs`)

| Code | Module | Mathematical Method & Complexity |
|---|---|---|
| `n1` | Segmented Prime Sieve | 32 KiB L1-cache segmented Sieve of Eratosthenes supporting intervals near `10^12` via base primes up to `sqrt(high)`, cross-checked against Sieve of Atkin mod-60/mod-12 quadratic forms and mapped onto an Ulam spiral. |
| `n2` | Primality & ECPP | Deterministic 12-base `u64` Miller-Rabin paired with Baillie-PSW (Miller-Rabin base-2 + Strong Lucas-Selfridge pseudoprime test), polynomial-ring AKS congruence witnesses `(x + a)^n == x^n + a (mod x^r - 1, n)`, and finite-field point lattice visualization. |
| `n3` | Factorization & DLP | Brent-Pollard `rho` cycle-finding factorization (`factorize_u64`), parallel Quadratic Sieve collecting `B`-smooth relations `x^2 - n` with GF(2) Gaussian elimination, and parallel Baby-Step Giant-Step (BSGS) solver for `g^x == h (mod p)`. |
| `n4` | Conjecture Verification | Parallel Collatz stopping-time & peak trajectory search, Goldbach comet partition counts `r(2k)`, Lucas-Lehmer Mersenne primality (`M_p = 2^p - 1`, `p <= 61`), and Korselt Carmichael criterion checks `(p - 1) | (n - 1)`. |

---

## Domain 2: Formal Verification & Automated Theorem Proving (`src/formal_verification.rs`)

| Code | Module | Mathematical Method & Complexity |
|---|---|---|
| `f1` | Interactive Theorem Proving | Recursive-descent parser for intuitionistic/classical propositional logic (`->`, `/\`, `\/`, `~`) and depth-bounded Gentzen-style LK sequent calculus proof search emitting inference trees (`ProofNode`) and 3D universe dependency DAGs. |
| `f2` | Portfolio CDCL SAT/SMT | Multithreaded `cdcl_worker` portfolio racing unit propagation, activity-based branching with diversified phase heuristics, and learned conflict clauses across Schur / Pythagorean triples (`{a, b, c}`, `a^2 + b^2 = c^2`) and Ramsey `R(3,3; n)` edge 2-coloring CNFs (`K_5` SAT, `K_6` UNSAT). |
| `f3` | Automated Conjecture Gen | Parallel hunter over polynomial continued fractions `a_0 + b_1 / (a_1 + b_2 / ...)` where `a_n, b_n` are degree-2 integer polynomials, scored against target mathematical constants (`4 / pi`, `e`, `phi`) by digit accuracy vs. complexity Pareto frontier. |

---

## Domain 3: Symbolic Computation & Computer Algebra (`src/symbolic_algebra.rs`)

| Code | Module | Mathematical Method & Complexity |
|---|---|---|
| `s1` | Symbolic Diff & Risch Int | Recursive AST calculus (`SymExpr`) supporting exact rational arithmetic, product/quotient/chain-rule differentiation, Liouvillian & Hermite-style symbolic antiderivatives (`integrate_symbolic`), numerical derivative roundtrip verification, and complex domain phase-magnitude surfaces. |
| `s2` | Gröbner Bases & Varieties | Multivariate exact-rational polynomials `Q[x, y, z]` (`MultiPoly`), multivariate polynomial division (`reduce_by`), S-polynomial formation, and Buchberger's completion algorithm parameterized by `MonomialOrder::Lex`, `GrLex`, and `GRevLex`. |
| `s3` | E-Graph Term Rewriting | Bounded multi-rule equality saturation (`saturate_equivalences`) combining commutativity, distributivity, Pythagorean trig identities (`sin^2 + cos^2 -> 1`, `1 - sin^2 -> cos^2`), and algebraic canonicalization ranked by minimal AST node count. |

---

## Domain 4: Numerical Analysis & Differential Equations (`src/numerical_pde.rs`)

| Code | Module | Mathematical Method & Complexity |
|---|---|---|
| `d1` | PDE/FEM & RK45 Solvers | Explicit parallel 2D vorticity-streamfunction Navier-Stokes finite-difference solver with Jacobi Poisson sweeps for `Laplacian(psi) = -omega`, P1 Galerkin Poisson center check (`-Laplacian(u) = 1` on `[0,1]^2`), and 4th-order Runge-Kutta (`integrate_lorenz_rk4`) chaotic attractor ensembles. |
| `d2` | BigInt NTT & Chudnovsky Pi | Arbitrary-precision base-`10^4` `BigInt` supporting Karatsuba recursive multiplication (`mul_karatsuba_limbs`), Number-Theoretic Transform convolution (`mul_ntt_limbs`), integer square root (`isqrt`), and parallel binary-splitting Chudnovsky series (`chudnovsky_bs`) for rapid `pi` calculation. |
| `d3` | Sparse Krylov, QR & SVD | Compressed Sparse Row (`CsrMatrix`) parallel `spmv`, Preconditioned Conjugate Gradient (`sparse_conjugate_gradient`) on symmetric positive-definite linear systems, and one-sided Jacobi Singular Value Decomposition (`dense_svd`) producing orthogonal `U, V` and sorted singular values `sigma`. |

---

## Domain 5: Combinatorics, Graph Theory & Optimization (`src/combinatorics_opt.rs`)

| Code | Module | Mathematical Method & Complexity |
|---|---|---|
| `c1` | TSP, Coloring & Bin Packing | Exact Branch-and-Bound TSP solver (`tsp_exact_branch_and_bound`), parallel multi-island simulated annealing with 2-Opt moves (`tsp_parallel_annealing`), and First-Fit Decreasing (FFD) 1D bin packing. |
| `c2` | Simplex, Barrier & MILP | Tableau-based Primal Simplex solver (`simplex_maximize`), log-barrier interior-point central path follower (`interior_point_central_path`), and parallel MILP integer lattice search on 3D linear polytopes. |
| `c3` | Symmetry-Broken Enumeration | Canonical dihedral/translation orbit-reduced free polyomino enumeration (OEIS `A000105`), backtracking reduced Latin square counting, and parallel canonical-labeling non-isomorphic simple graph enumeration. |

---

## Domain 6: Computational Geometry & Topology (`src/geometry_topology.rs`)

| Code | Module | Mathematical Method & Complexity |
|---|---|---|
| `g1` | 3D Hull, Voronoi & Delaunay | Exact in-circumcircle 2D Bowyer-Watson Delaunay triangulation with bounded dual Voronoi diagram extraction, plus incremental horizon-conflict 3D Convex Hull satisfying Euler's formula `V - E + F = 2`. |
| `g2` | Vietoris-Rips TDA & Betti | Parallel distance-matrix Vietoris-Rips simplex complex builder up to dimension 2 with `Z_2` boundary matrix reduction yielding persistence intervals `(birth, death)` and Betti triples `(beta_0, beta_1, beta_2)`. |
| `g3` | 3D Knots & Jones Polynomial | Planar diagram (`CrossingPD`) writhe normalization and parallel `2^c` state-sum Kauffman bracket / Jones Laurent polynomial (`LaurentPoly` in `Z[t, t^-1]`), Alexander invariant reporting, and 3D Frenet-frame tubular ribbon meshing. |

---

## Domain 7: Geometric Proofs & Visualizations (`src/geometric_proofs.rs`)

| Code | Module | Mathematical Method & Verification Invariants |
|---|---|---|
| `p1` | Area & Dissection Proofs | Bhāskara / Zhoubi Suanjing outer `(a+b)^2` frame enclosing 4 congruent right triangles plus tilted inner `(b-a)^2` square verifying `c^2 = 4(ab/2) + (b-a)^2` to `< 1e-12`; Euclid Elements I.47 altitude shear splitting `c^2` into `c*p = a^2` and `c*q = b^2`; alternating geometric series unit-square halving verifying `S_N = 1 - 2^-N`; exact `u128` Nicomachus cube-to-square gnomon conservation `sum_{k=1..n} k^3 == (n(n+1)/2)^2`. |
| `p2` | Circle, Conic & Hyperbolic | Archimedes' hexagon-doubling polygon exhaustion (`m = 6*2^k`) enforcing `m*sin(pi/m) < pi < m*tan(pi/m)` with monotonic quadratic gap shrinkage; Thales semicircle dot-product orthogonality (`|PA.PB| < 1e-12`) and Ptolemy cyclic quadrilateral diagonal identity (`|AC||BD| = |AB||CD| + |BC||AD|`); exact 3D Dandelin tangent spheres on an oblique circular cone confirming `|PF_1| + |PF_2| = 2a = (c_{z2}-c_{z1})cos(alpha)` and eccentricity `e = sin(beta)/cos(alpha)`; orthogonal circle Poincaré disk geodesics (`K = -1`) computing positive Gauss-Bonnet angular defect `Area = pi - (alpha + beta + gamma) > 0`. |
| `p3` | Cavalieri & 3D Polyhedra | Archimedes `1 : 2 : 3` double-cone, sphere, and circumscribed cylinder cross-sectional identity `A_sphere(z) + A_cone(z) = pi*R^2` verified across `z in [-R, R]` with Simpson 3D volume quadrature; all 5 normalized Platonic polyhedra verified directly from 3D face corners for Euler `V - E + F = 2` and Descartes total angular defect `sum delta_v = 4*pi`; 2D/3D Viviani regular simplex perpendicular conservation (`sum d_i == H`); discrete divergence-theorem triangulated 3D torus signed volume convergence against Pappus's Centroid Theorem `V = 2*pi^2 * R * r^2`. |

---

## Verification & Telemetry Summary

All mathematical invariants are checked automatically via:
1. [`tests/integration_suite.rs`](../tests/integration_suite.rs) (6 module/persistence integration tests across all 22 modules)
2. [`tests/headless_algorithm_validation.rs`](../tests/headless_algorithm_validation.rs) (7 domain invariant suites + 22-module multi-thread & exporter sweep writing [`validation/principia_studio_validation_summary.json`](../validation/principia_studio_validation_summary.json))
